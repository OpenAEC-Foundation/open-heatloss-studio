/**
 * JSON import/export for ISSO 51 projects.
 *
 * Export wraps the project + result in a versioned envelope.
 * Import accepts both the envelope format and raw Project JSON.
 * Auto-detects thermal import files (Revit/IFC) and signals the caller.
 */
import type {
  ConstructionElement,
  HeatingSystem,
  Project,
  ProjectResult,
  Room,
  VerticalPosition,
} from "../types";
import type { CatalogueCategory } from "./constructionCatalogue";
import type {
  ModelDoor,
  ModelRoom,
  ModelWindow,
  ProjectConstruction,
} from "../components/modeller/types";
import { useModellerStore } from "../components/modeller/modellerStore";
import {
  buildIfcEnergyDocument,
  detectFormat,
  parseIfcEnergy,
  serializeIfcEnergy,
  type ModellerSnapshot,
} from "./ifcenergy";
import { isTauri } from "./backend";

const SCHEMA_ID = "isso51-project-v1";
const EXPORT_VERSION = "1.0.0";

/** Sources that indicate a thermal import file (Revit/IFC export). */
const THERMAL_SOURCES = ["revit-eam", "revit-raycast", "ifc"] as const;

/** Returned when the imported JSON is a thermal import file, not a regular project. */
export interface ThermalImportDetected {
  type: "thermal";
  /** Raw JSON string to pass to the thermal import wizard. */
  rawJson: string;
}

/** Modeller geometry section of the envelope. */
interface ModellerEnvelope {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
}

/** Envelope format written to disk. */
interface ProjectEnvelope {
  version: string;
  schema: string;
  exported_at: string;
  project: Project;
  result: ProjectResult | null;
  /**
   * Project-scoped construction library (per-project layer stacks etc.).
   * Lives in `useModellerStore` and is NOT part of the `Project` type.
   * Optional for backwards-compat with envelopes written before bug H fix.
   */
  project_constructions?: ProjectConstruction[];
  /**
   * Modeller geometry (2D/3D rooms, windows, doors). Optional —
   * envelopes written before this field existed don't have it; the
   * importer treats the absence as "this project has no modeller data"
   * and clears the modeller store accordingly.
   */
  modeller?: ModellerEnvelope;
}

/** Result of a successful regular project import. */
export interface ImportResult {
  type: "project";
  project: Project;
  result: ProjectResult | null;
}

/**
 * Export project + result as a downloadable `.isso51.json` file.
 */
export function exportProject(
  project: Project,
  result: ProjectResult | null,
): void {
  // Snapshot project constructions and modeller geometry from modellerStore.
  // Both live outside the Project type but are needed for a faithful re-import:
  //   - project_construction_id references on Room.constructions[]
  //   - 2D/3D room polygons, windows, doors drawn in the modeller
  // Without persisting modeller data, re-import would show a different
  // modeller state than what was authored (stale localStorage).
  const storeState = useModellerStore.getState();

  const envelope: ProjectEnvelope = {
    version: EXPORT_VERSION,
    schema: SCHEMA_ID,
    exported_at: new Date().toISOString(),
    project,
    result,
    project_constructions: storeState.projectConstructions,
    modeller: {
      rooms: storeState.rooms,
      windows: storeState.windows,
      doors: storeState.doors,
    },
  };

  const json = JSON.stringify(envelope, null, 2);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);

  const name = project.info.name || "project";
  const safeName = name.replace(/[^a-zA-Z0-9_\-\s]/g, "").trim() || "project";

  const a = document.createElement("a");
  a.href = url;
  a.download = `${safeName}.isso51.json`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * Snapshot the entire modellerStore state into a `ModellerSnapshot` for the
 * .ifcenergy envelope. Centralized here so future modeller fields land in
 * both legacy and new export paths consistently.
 */
function snapshotModellerState(): ModellerSnapshot {
  const s = useModellerStore.getState();
  return {
    rooms: s.rooms,
    windows: s.windows,
    doors: s.doors,
    projectConstructions: s.projectConstructions,
    wallConstructions: s.wallConstructions,
    floorConstructions: s.floorConstructions,
    roofConstructions: s.roofConstructions,
    wallBoundaryTypes: s.wallBoundaryTypes,
    underlay: s.underlay,
  };
}

/**
 * Export project + result + modeller as a `.ifcenergy` file.
 *
 * In Tauri-mode: opent een native Windows save-dialog (filter `.ifcenergy`)
 * en schrijft het document naar het gekozen pad via `@tauri-apps/plugin-fs`.
 * In web-mode: fall-back naar Blob + anchor-download (browser default).
 *
 * Het bestand is een geldige IFCX (IFC5 alpha) document — zie `ifcenergy.ts`
 * voor envelope-structuur. Legacy `.isso51.json` blijft beschikbaar via
 * `exportProject` voor backwards-compat use cases.
 */
/**
 * Schrijf het project + result als `.ifcenergy` IFCX document.
 *
 * Gedrag bij `targetPath`:
 *   - `undefined` (default) → Tauri: opent save-as dialog;
 *     web: blob-download via anchor click. Standaard "Opslaan als" flow.
 *   - `string` (Tauri-mode) → schrijf direct naar dit pad, géén dialog.
 *     Gebruikt voor "Opslaan" wanneer de file al een bekend pad heeft.
 *
 * Returns het pad dat geschreven werd (Tauri) of `null` (web / cancelled /
 * geen pad bekend). Caller kan dat terugschrijven naar
 * `projectStore.currentLocalPath` zodat een volgende "Opslaan" stil naar
 * dezelfde locatie schrijft.
 */
export async function exportIfcEnergy(
  project: Project,
  result: ProjectResult | null,
  targetPath?: string | null,
): Promise<string | null> {
  const doc = buildIfcEnergyDocument({
    project,
    result,
    modeller: snapshotModellerState(),
  });
  const json = serializeIfcEnergy(doc);

  const name = project.info.name || "project";
  const safeName = name.replace(/[^a-zA-Z0-9_\-\s]/g, "").trim() || "project";

  if (isTauri()) {
    try {
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");

      // Direct-write pad: geen dialog, alleen overschrijven
      if (targetPath) {
        await writeTextFile(targetPath, json);
        recordRecent(project.info.name || safeName, targetPath);
        return targetPath;
      }

      // Geen pad → save-as dialog
      const { save } = await import("@tauri-apps/plugin-dialog");
      const filePath = await save({
        defaultPath: `${safeName}.ifcenergy`,
        filters: [
          { name: "Open Heatloss Studio", extensions: ["ifcenergy"] },
        ],
      });
      if (!filePath) return null; // user cancelled
      await writeTextFile(filePath, json);
      recordRecent(project.info.name || safeName, filePath);
      return filePath;
    } catch (err) {
      console.error("Tauri save failed, falling back to browser download:", err);
    }
  }

  // Web-mode (of Tauri fallback): blob + anchor download (geen pad terug)
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${safeName}.ifcenergy`;
  a.click();
  URL.revokeObjectURL(url);
  return null;
}

/** Schuif een entry op de top van de recent-files lijst. */
function recordRecent(displayName: string, filePath: string): void {
  try {
    // Dynamic import om geen module-cycle te creëren met de store
    void import("../store/recentFilesStore").then(({ useRecentFilesStore }) => {
      const fileName = filePath.split(/[\\/]/).pop() ?? "project.ifcenergy";
      useRecentFilesStore.getState().push({
        name: displayName,
        fileName,
        path: filePath,
      });
    });
  } catch {
    // store/import missing — non-fatal
  }
}

/**
 * Top-level open dispatcher: route file content to the correct importer
 * based on shape detection.
 *
 * Returns the same shape as `importProject` for legacy compatibility, plus
 * an extra `format` field so callers can show "loaded as .ifcenergy" vs
 * "loaded as legacy .isso51.json" UI hints if desired.
 *
 * Side effects on the modellerStore are identical to `importProject` — geometry
 * arrays are replaced (or cleared if absent in the file) so the modeller stays
 * in sync with the loaded project.
 */
export function openProjectFile(
  jsonString: string,
): (ImportResult | ThermalImportDetected) & { format?: "ifcenergy" | "isso51-legacy" | "thermal-import" } {
  const fmt = detectFormat(jsonString);

  if (fmt === "ifcenergy") {
    const parsed = parseIfcEnergy(jsonString);
    const project = validateProject(parsed.project);
    const result = parsed.result ? validateProjectResult(parsed.result) : null;

    // Restore project constructions if present.
    if (parsed.modeller.projectConstructions.length > 0) {
      useModellerStore.getState().replaceProjectConstructions(
        parsed.modeller.projectConstructions,
      );
    }

    // Restore modeller geometry — same semantics as legacy: replace arrays
    // (or clear when empty) so tables and modeller stay in sync.
    useModellerStore.getState().importModel(
      parsed.modeller.rooms,
      parsed.modeller.windows,
      parsed.modeller.doors,
    );

    return { type: "project", project, result, format: "ifcenergy" };
  }

  // Fall back to legacy importer for `.isso51.json`, raw Project JSON,
  // or thermal-import files. The legacy importer also handles modeller
  // state side effects in importProject().
  const legacy = importProject(jsonString);
  if (legacy.type === "thermal") {
    return { ...legacy, format: "thermal-import" };
  }
  return { ...legacy, format: "isso51-legacy" };
}

/**
 * Import a project from a JSON file.
 *
 * Accepts:
 * - Wrapped format: `{ schema: "isso51-project-v1", project: {...} }`
 * - Raw Project JSON: `{ info: {...}, building: {...}, ... }`
 * - Thermal import JSON (auto-detected via `source` field) — returns
 *   `ThermalImportDetected` so the caller can redirect to the wizard.
 */
export function importProject(jsonString: string): ImportResult | ThermalImportDetected {
  let data: unknown;
  try {
    data = JSON.parse(jsonString);
  } catch {
    throw new Error("Ongeldig JSON bestand");
  }

  if (typeof data !== "object" || data === null) {
    throw new Error("Ongeldig bestandsformaat");
  }

  const obj = data as Record<string, unknown>;

  // Auto-detect thermal import format (Revit/IFC export).
  if (
    typeof obj.source === "string" &&
    (THERMAL_SOURCES as readonly string[]).includes(obj.source)
  ) {
    return { type: "thermal", rawJson: jsonString };
  }

  // Detect envelope format.
  if (obj.schema === SCHEMA_ID && obj.project) {
    const project = validateProject(obj.project);
    const result = validateProjectResult(obj.result);

    // Restore project constructions from the envelope if present. We do NOT
    // structurally validate the entries (envelope-level optional field) —
    // simply cast through and let the store handle it. For older envelopes
    // without this field we leave the current store state untouched
    // (least-destructive: preserves any work-in-progress constructions).
    if (Array.isArray(obj.project_constructions)) {
      const pcs = obj.project_constructions as ProjectConstruction[];
      useModellerStore.getState().replaceProjectConstructions(pcs);
    }

    // Replace modeller geometry to match the imported project. If the
    // envelope omits modeller data (legacy `.isso51.json` files exported
    // before this field existed), we clear the store so the user doesn't
    // see stale rooms/windows/doors from a previously-loaded project's
    // localStorage. Same root cause as the Memeleiland mismatch bug.
    const modeller = obj.modeller as ModellerEnvelope | undefined;
    useModellerStore.getState().importModel(
      modeller?.rooms ?? [],
      modeller?.windows ?? [],
      modeller?.doors ?? [],
    );

    return { type: "project", project, result };
  }

  // Try as raw Project JSON. No envelope means no modeller data either —
  // clear the store so tables and modeller stay in sync.
  const project = validateProject(data);
  useModellerStore.getState().importModel([], [], []);
  return { type: "project", project, result: null };
}

/**
 * Validate that the data looks like a ProjectResult (basic structural checks).
 * Returns null for null/undefined input, validated ProjectResult otherwise.
 */
export function validateProjectResult(data: unknown): ProjectResult | null {
  if (data == null) return null;

  if (typeof data !== "object") {
    throw new Error("Result data is geen geldig object");
  }

  const obj = data as Record<string, unknown>;

  if (!Array.isArray(obj.rooms)) {
    throw new Error("Result mist verplicht veld 'rooms' of is geen array");
  }

  if (!obj.summary || typeof obj.summary !== "object") {
    throw new Error("Result mist verplicht veld 'summary'");
  }

  return data as ProjectResult;
}

/**
 * Validate that the data looks like a Project (basic structural checks).
 * Exported so server responses can also be validated before casting.
 */
export function validateProject(data: unknown): Project {
  if (typeof data !== "object" || data === null) {
    throw new Error("Project data is geen geldig object");
  }

  const obj = data as Record<string, unknown>;

  if (!obj.building || typeof obj.building !== "object") {
    throw new Error("Verplicht veld 'building' ontbreekt");
  }

  if (!obj.climate || typeof obj.climate !== "object") {
    throw new Error("Verplicht veld 'climate' ontbreekt");
  }

  if (!obj.ventilation || typeof obj.ventilation !== "object") {
    throw new Error("Verplicht veld 'ventilation' ontbreekt");
  }
  // NB: V2 SharedProject ventilation kent extra optionele velden
  // (mechanical_supply_m3_per_h, mechanical_exhaust_m3_per_h) — in V2 schema
  // `Option<f64>` met `#[serde(default)]` in Rust, dus `?:` in TS. Legacy
  // JSONs zonder deze velden parsen correct; backend valt terug op
  // `default_ach` fallback in tojuli.rs als ze undefined zijn. Geen
  // backfill nodig op V1-niveau — V1 VentilationConfig heeft deze velden niet.

  if (!Array.isArray(obj.rooms)) {
    throw new Error("Verplicht veld 'rooms' ontbreekt of is geen array");
  }

  // Ensure info exists.
  if (!obj.info || typeof obj.info !== "object") {
    (obj as Record<string, unknown>).info = { name: "" };
  }

  const project = data as Project;

  // Backfill heating_system voor legacy JSONs van vóór de ISSO 51
  // installatie-UI. Het Rust core type vereist `heating_system` als
  // verplicht veld (geen serde default) — zonder fill crasht
  // `backend.calculate()` met een missing-field fout. Default = de
  // project-brede standaard als die al in de JSON stond, anders
  // radiator_ht (ISSO 51 meest voorkomend).
  //
  // NOTE: ISSO 53-projecten hebben camelCase keys (b.v.
  // `radiatorenConvHtEnLuchtverwarming`). De `default_heating_system`
  // bevat in dat geval al de juiste norm-key, zodat de fallback hier
  // automatisch klopt. Alleen als zowel default als per-room ontbreken
  // grijpen we naar de ISSO 51-default `radiator_ht`; voor pure
  // ISSO 53-imports zonder default is dat verkeerd, maar zo'n input
  // bestaat in praktijk niet (ISSO 53 UI vult altijd default in).
  const fallbackHs: HeatingSystem =
    project.building.default_heating_system ?? "radiator_ht";
  project.rooms = project.rooms.map((r: Room) => ({
    ...r,
    heating_system: r.heating_system ?? fallbackHs,
  }));

  // Backfill aggregation_method voor legacy JSONs van vóór de
  // VabiCompat/NormStrict keuze. Rust core heeft `serde(default)` =
  // `vabi_compat`, dus consistent met backend-gedrag.
  if (project.building.aggregation_method == null) {
    project.building.aggregation_method = "vabi_compat";
  }

  return project;
}

// ---------------------------------------------------------------------------
// Construction extraction — dedup + link on import
// ---------------------------------------------------------------------------

/** Fingerprint for deduplication: same type = same construction. */
function constructionFingerprint(c: ConstructionElement): string {
  return `${c.description}|${c.u_value}|${c.material_type}|${c.vertical_position ?? "wall"}|${c.boundary_type}`;
}

/** Map element to CatalogueCategory based on position and layer presence. */
function categoryFromElement(ce: ConstructionElement): CatalogueCategory {
  if (ce.vertical_position === "ceiling") return "daken";
  if (ce.vertical_position === "floor") return "vloeren_plafonds";
  // Elements without layers are typically kozijnen/vullingen (glass, doors)
  if (!ce.layers || ce.layers.length === 0) return "kozijnen_vullingen";
  return "wanden";
}

/**
 * Extract unique construction types from a project's rooms and
 * create ProjectConstruction entries in modellerStore.
 *
 * Each room's ConstructionElement gets a `project_construction_id`
 * linking back to the ProjectConstruction.
 *
 * Call this after `importProject()` and before `setProject()`.
 */
export function extractAndLinkConstructions(project: Project): void {
  const store = useModellerStore.getState();

  // Clear stale project constructions to ensure categories are re-evaluated.
  // Without this, persisted entries from localStorage retain outdated categories
  // (e.g. "wanden" for elements that should now be "kozijnen_vullingen").
  store.importProjectConstructions([]);
  const existing: readonly ProjectConstruction[] = [];

  // Map fingerprint → project construction ID (existing + new)
  const fpToId = new Map<string, string>();

  // Collect unique constructions from all rooms
  const newConstructions: Omit<ProjectConstruction, "id">[] = [];

  for (const room of project.rooms) {
    for (const ce of room.constructions) {
      const fp = constructionFingerprint(ce);

      if (fpToId.has(fp)) {
        // Already seen — just link
        ce.project_construction_id = fpToId.get(fp)!;
        continue;
      }

      // Check if an existing ProjectConstruction matches
      const existingMatch = existing.find(
        (pc) =>
          pc.name === ce.description &&
          pc.materialType === ce.material_type &&
          pc.verticalPosition === (ce.vertical_position ?? "wall"),
      );

      if (existingMatch) {
        fpToId.set(fp, existingMatch.id);
        ce.project_construction_id = existingMatch.id;
        continue;
      }

      // Create new project construction
      const id = `proj-${crypto.randomUUID()}`;
      fpToId.set(fp, id);
      ce.project_construction_id = id;

      newConstructions.push({
        name: ce.description,
        category: categoryFromElement(ce),
        materialType: ce.material_type,
        verticalPosition: (ce.vertical_position ?? "wall") as VerticalPosition,
        layers: ce.layers ? structuredClone(ce.layers) : [],
        uValue: (!ce.layers || ce.layers.length === 0) ? ce.u_value : undefined,
      });
    }
  }

  // Bulk-add new constructions to modellerStore
  if (newConstructions.length > 0) {
    store.importProjectConstructions(newConstructions);

    // importProjectConstructions generates new IDs, so we need to remap.
    // Re-read the store to get the actual IDs.
    const updated = useModellerStore.getState().projectConstructions;

    // Build name→id lookup from newly added entries
    const nameToId = new Map<string, string>();
    for (const pc of updated) {
      nameToId.set(
        `${pc.name}|${pc.materialType}|${pc.verticalPosition}`,
        pc.id,
      );
    }

    // Re-link construction elements to actual IDs
    for (const room of project.rooms) {
      for (const ce of room.constructions) {
        const key = `${ce.description}|${ce.material_type}|${ce.vertical_position ?? "wall"}`;
        const actualId = nameToId.get(key);
        if (actualId) {
          ce.project_construction_id = actualId;
        }
      }
    }
  }
}
