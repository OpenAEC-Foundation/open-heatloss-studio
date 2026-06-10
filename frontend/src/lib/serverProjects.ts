/**
 * Gedeelde server-save/-load helpers — envelope-pariteit met de file-flow.
 *
 * Vóór deze module stuurde elke save-flow (auto-save, Ctrl+S, Backstage,
 * Projects-pagina, ProjectSetup) een kaal `state.project` als `project_data`
 * naar de server, waardoor modeller-geometrie, sharedExtra, ISSO 53- en
 * ventilatie-sidecars verloren gingen bij openen op een andere machine.
 *
 * Nu geldt:
 *   - **Save** → {@link buildProjectEnvelope} (zelfde envelope als de
 *     `.heatloss.json` file-export) als `project_data`.
 *   - **Load** → {@link importServerProjectData} (zelfde import-pad als
 *     bestand-openen, inclusief modeller-store herstel/leging) +
 *     `loadServerProject` met alle sidecars.
 *
 * Alle helpers werken de {@link useSaveStatusStore} bij zodat de StatusBar
 * een persistente save-status toont (geen stil falen meer).
 *
 * `result_data` blijft een apart API-veld (gevuld door de server-side
 * rekenroute en legacy-rijen); de envelope draagt zijn eigen `result` mee.
 * Bij het laden wint `envelope.result`, met `result_data` als fallback.
 */
import {
  ConflictError,
  SessionExpiredError,
  createProject as apiCreateProject,
  fetchProject,
  updateProject as apiUpdateProject,
  type UpdateProjectResponse,
} from "./backend";
import {
  buildProjectEnvelope,
  extractAndLinkConstructions,
  importServerProjectData,
  validateProjectResult,
  type ProjectEnvelope,
} from "./importExport";
import { useProjectStore } from "../store/projectStore";
import { useSaveStatusStore } from "../store/saveStatusStore";
import type { ProjectResponse, ProjectResult } from "../types";

/**
 * Bouw de server-payload (`project_data`) uit de huidige store-state.
 * Eén envelope voor file én server — zie {@link buildProjectEnvelope}.
 */
export function buildServerProjectData(): ProjectEnvelope {
  const state = useProjectStore.getState();
  return buildProjectEnvelope(
    state.project,
    // Store-result kan ook een Isso53ProjectResult zijn; de envelope is
    // daar agnostisch in (zelfde cast als de file-export flows).
    state.result as ProjectResult | null,
  );
}

/** Vertaal een save-fout naar de statusindicator. Gooit de fout door. */
function recordSaveFailure(err: unknown): void {
  const status = useSaveStatusStore.getState();
  if (err instanceof ConflictError) {
    status.setConflict();
    // Conflict centraal markeren zodat de ConflictDialog opent — alle
    // save-flows deden dit voorheen elk voor zich (of vergaten het).
    useProjectStore.setState({ hasConflict: true });
  } else if (err instanceof SessionExpiredError) {
    // Definitief verlopen Authentik-sessie: server-saves zijn onmogelijk tot
    // een nieuwe interactieve login. Serverbinding loskoppelen (R1 — een
    // gepersisteerde binding mag op een gedeelde browser niet overerven naar
    // de volgende gebruiker). Project + isDirty blijven staan; de caller
    // (useAutoSave/Backstage) toont de "log opnieuw in"-toast. Na re-login
    // heropent de user het project via de Projects-lijst.
    useProjectStore.getState().clearServerBinding();
  } else if (typeof navigator !== "undefined" && !navigator.onLine) {
    status.setOffline();
  } else {
    status.setError(err instanceof Error ? err.message : String(err));
  }
}

/**
 * Sla het actieve serverproject op (PUT /projects/:id) met de volledige
 * envelope. Werkt `isDirty`/`serverUpdatedAt` en de save-status bij.
 * Fouten worden doorgegooid zodat de caller zijn eigen UX (toast/dialog)
 * kan tonen; de statusindicator is dan al gezet.
 *
 * **Race-guard:** de payload wordt uit de huidige store-state gebouwd. Als
 * `id` niet (meer) het actieve project is — bv. een auto-save-debounce van
 * project A die vuurt nadat project B geladen is — zou project B-data onder
 * A's id worden weggeschreven. In dat geval breken we stil af (`null`, geen
 * status-update, geen user-facing fout): de stale timer is achterhaald en
 * het nieuwe project heeft zijn eigen save-cyclus.
 */
export async function saveExistingServerProject(
  id: string,
): Promise<UpdateProjectResponse | null> {
  const state = useProjectStore.getState();
  if (state.activeProjectId !== id) {
    // Stale save van een inmiddels gewisseld/gesloten project — no-op.
    return null;
  }
  const status = useSaveStatusStore.getState();
  status.setSaving();
  try {
    const response = await apiUpdateProject(id, {
      name: state.project.info.name || undefined,
      project_data: buildServerProjectData(),
      expected_updated_at: state.serverUpdatedAt ?? undefined,
    });
    useProjectStore.setState((prev) =>
      prev.activeProjectId === id
        ? { isDirty: false, serverUpdatedAt: response.updated_at }
        : {},
    );
    useSaveStatusStore.getState().setSaved();
    return response;
  } catch (err) {
    recordSaveFailure(err);
    throw err;
  } finally {
    // Vangnet: geen enkel exception-pad mag de indicator op "Opslaan…"
    // laten hangen (bv. een onverwachte throw buiten de API-call om).
    const s = useSaveStatusStore.getState();
    if (s.status === "saving") {
      s.setError("Opslaan onverwacht afgebroken");
    }
  }
}

/**
 * Sla het huidige project als nieuw serverproject op (POST /projects) met de
 * volledige envelope. Zet `activeProjectId` en de save-status.
 */
export async function saveNewServerProject(
  name: string,
): Promise<{ id: string; name: string }> {
  const status = useSaveStatusStore.getState();
  status.setSaving();
  try {
    const response = await apiCreateProject(name, buildServerProjectData());
    useProjectStore.setState({ activeProjectId: response.id, isDirty: false });
    useSaveStatusStore.getState().setSaved();
    return response;
  } catch (err) {
    recordSaveFailure(err);
    throw err;
  } finally {
    // Zelfde vangnet als saveExistingServerProject: nooit hangen in "saving".
    const s = useSaveStatusStore.getState();
    if (s.status === "saving") {
      s.setError("Opslaan onverwacht afgebroken");
    }
  }
}

/**
 * Pas een opgehaald serverproject toe op de stores — het server-equivalent
 * van de bestand-openen flow (openProjectFile → extractAndLinkConstructions
 * → setProject), maar dan via `loadServerProject` zodat `activeProjectId` +
 * `serverUpdatedAt` gezet blijven en `isDirty` op false staat.
 *
 * Backward-compat: legacy kaal `project_data` (alleen een Project-object)
 * laadt met defaults en leegt de modeller-store expliciet — geen stale
 * geometrie van het vorige project (zie {@link importServerProjectData}).
 *
 * Apart van {@link openServerProject} gehouden zodat tests dit pad zonder
 * fetch kunnen draaien.
 */
export function applyServerProjectResponse(
  id: string,
  response: Pick<ProjectResponse, "project_data" | "result_data" | "updated_at">,
): void {
  // Zelfde import-pad als bestand-openen — herstelt of leegt de modeller.
  const imported = importServerProjectData(response.project_data);

  // Zelfde nabewerking als de file-open flows (Backstage/AppShell):
  // project-constructies dedupliceren + koppelen aan room-elementen.
  extractAndLinkConstructions(imported.project);

  // Envelope-result wint; result_data is fallback voor legacy rijen en de
  // server-side rekenroute. Zie module-doc voor de keuze.
  const result =
    imported.result ?? validateProjectResult(response.result_data);

  useProjectStore.getState().loadServerProject(
    id,
    imported.project,
    result,
    response.updated_at,
    {
      norm: imported.norm,
      isso53Building: imported.isso53?.building,
      isso53Rooms: imported.isso53?.rooms,
      sharedExtra: imported.sharedExtra,
      ventilation: imported.ventilation,
    },
  );

  // Vers geladen project: geen openstaande save-status meer.
  useSaveStatusStore.getState().resetStatus();
}

/** Haal een serverproject op en laad het atomair in de stores. */
export async function openServerProject(id: string): Promise<void> {
  const response = await fetchProject(id);
  applyServerProjectResponse(id, response);
}
