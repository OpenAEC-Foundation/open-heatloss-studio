/**
 * Norm-wissel helpers — fase 4 ISSO 53 UI werkpakket.
 *
 * Bevat:
 *   - Mapping van ISSO 51 `RoomFunction` → ISSO 53 (`GebruiksFunctie`,
 *     `RuimteType`) en omgekeerd (best-effort default).
 *   - Mapping van ISSO 51 `BuildingType` → ISSO 53 `Isso53BuildingPosition`
 *     en omgekeerd (best-effort default).
 *   - Back-up van het huidige project naar disk (Tauri) of blob-download
 *     (web fallback) vóór de wissel plaatsvindt.
 *
 * De wissel zelf wordt door `NormSwitchModal` aangeroepen die de
 * `projectStore`-mutaties uitvoert. Deze module bevat puur de
 * transformaties + I/O, geen store-coupling.
 */
import type {
  BuildingType,
  Project,
  ProjectResult,
  RoomFunction,
} from "../types";
import type {
  ActiveNorm,
  Isso53BuildingPosition,
  Isso53BuildingShape,
  Isso53BuildingState,
  Isso53RoomState,
  Isso53WindPressureType,
} from "../types/projectV2";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
} from "../types/projectV2";
import { isTauri } from "./backend";

// ---------------------------------------------------------------------------
// Mapping ISSO 51 → ISSO 53
// ---------------------------------------------------------------------------

/**
 * ISSO 51 `RoomFunction` → ISSO 53 (`GebruiksFunctie`, `RuimteType`).
 *
 * Alle woon-functies worden gemapt op `kantoor` als best-effort default
 * — utiliteitsbouw kent geen "slaapkamer" maar de norm-relevante eis
 * (verblijfsruimte vs. badruimte vs. verkeersruimte) blijft behouden.
 * User verfijnt na de wissel handmatig in de ruimte-tab.
 */
export const MAP_51_TO_53: Record<RoomFunction, Isso53RoomState> = {
  living_room: { gebruiksFunctie: "kantoor", ruimteType: "verblijfsruimte" },
  kitchen:     { gebruiksFunctie: "kantoor", ruimteType: "verblijfsruimte" },
  bedroom:     { gebruiksFunctie: "kantoor", ruimteType: "verblijfsruimte" },
  bathroom:    { gebruiksFunctie: "kantoor", ruimteType: "badruimte" },
  toilet:      { gebruiksFunctie: "kantoor", ruimteType: "toiletruimte" },
  hallway:     { gebruiksFunctie: "kantoor", ruimteType: "verkeersruimte" },
  landing:     { gebruiksFunctie: "kantoor", ruimteType: "verkeersruimte" },
  storage:     { gebruiksFunctie: "kantoor", ruimteType: "bergruimte" },
  attic:       { gebruiksFunctie: "kantoor", ruimteType: "onbenoemdeRuimte" },
  custom:      { gebruiksFunctie: "kantoor", ruimteType: "onbenoemdeRuimte" },
};

/**
 * ISSO 51 `BuildingType` → ISSO 53 `Isso53BuildingPosition`.
 *
 * Best-effort: vrijstaande/twee-onder-één-kap → enkellaagsKop/Tussen;
 * gestapelde bouw → meerlaags-equivalenten. User kan dit later
 * verfijnen in AlgemeenTab.
 */
const BUILDING_TYPE_TO_53_POSITION: Record<BuildingType, Isso53BuildingPosition> = {
  detached:       "enkellaagsVrijstaand",
  semi_detached:  "enkellaagsKop",
  terraced:       "enkellaagsTussen",
  end_of_terrace: "enkellaagsKop",
  porch:          "meerlaagsTussen",
  gallery:        "meerlaagsTussen",
  stacked:        "meerlaagsTussen",
};

const BUILDING_TYPE_TO_53_SHAPE: Record<BuildingType, Isso53BuildingShape> = {
  detached:       "eenLaagMetKap",
  semi_detached:  "eenLaagMetKap",
  terraced:       "eenLaagMetKap",
  end_of_terrace: "eenLaagMetKap",
  porch:          "meerlaags",
  gallery:        "meerlaags",
  stacked:        "meerlaags",
};

const BUILDING_TYPE_TO_53_WIND: Record<BuildingType, Isso53WindPressureType> = {
  detached:       "eenlaagsMetKap",
  semi_detached:  "eenlaagsMetKap",
  terraced:       "eenlaagsMetKap",
  end_of_terrace: "eenlaagsMetKap",
  porch:          "meerlaagsStandaard",
  gallery:        "meerlaagsVolgevelBinnengalerij",
  stacked:        "meerlaagsStandaard",
};

/**
 * Bouw een complete `Isso53BuildingState` op uit de huidige ISSO 51
 * building-configuratie. Niet-mappable velden vallen terug op
 * `DEFAULT_ISSO53_BUILDING`.
 */
export function deriveIsso53BuildingFromIsso51(
  project: Project,
): Isso53BuildingState {
  const bt = project.building.building_type;
  return {
    ...DEFAULT_ISSO53_BUILDING,
    buildingShape: BUILDING_TYPE_TO_53_SHAPE[bt] ?? DEFAULT_ISSO53_BUILDING.buildingShape,
    buildingPosition: BUILDING_TYPE_TO_53_POSITION[bt] ?? DEFAULT_ISSO53_BUILDING.buildingPosition,
    windPressureType: BUILDING_TYPE_TO_53_WIND[bt] ?? DEFAULT_ISSO53_BUILDING.windPressureType,
  };
}

/** Bouw de `isso53Rooms`-sidecar map op vanuit een ISSO 51 project. */
export function deriveIsso53RoomsFromIsso51(
  project: Project,
): Record<string, Isso53RoomState> {
  const out: Record<string, Isso53RoomState> = {};
  for (const room of project.rooms) {
    out[room.id] = MAP_51_TO_53[room.function] ?? { ...DEFAULT_ISSO53_ROOM };
  }
  return out;
}

// ---------------------------------------------------------------------------
// Mapping ISSO 53 → ISSO 51 (best-effort terug)
// ---------------------------------------------------------------------------

/**
 * ISSO 53 → ISSO 51 mapt alles op `living_room` als best-effort default.
 * Utiliteitsfuncties (kantoor, lesruimte etc.) hebben geen 1-op-1
 * equivalent in `RoomFunction`. User verfijnt na de wissel handmatig.
 */
export function mapRoom53To51(_state: Isso53RoomState): RoomFunction {
  return "living_room";
}

/**
 * ISSO 53 `BuildingPosition` → ISSO 51 `BuildingType` (best-effort).
 *
 * Meerlaagse posities komen het dichtst bij `stacked`; enkellaagse
 * posities mappen op de bijbehorende grondgebonden types.
 */
const POSITION_53_TO_BUILDING_TYPE: Record<Isso53BuildingPosition, BuildingType> = {
  enkellaagsTussen:     "terraced",
  enkellaagsKop:        "end_of_terrace",
  enkellaagsVrijstaand: "detached",
  meerlaagsGeheel:      "stacked",
  meerlaagsTop:         "stacked",
  meerlaagsTussen:      "stacked",
  meerlaagsOnder:       "stacked",
};

export function deriveIsso51BuildingTypeFromIsso53(
  building53: Isso53BuildingState,
): BuildingType {
  return POSITION_53_TO_BUILDING_TYPE[building53.buildingPosition] ?? "detached";
}

// ---------------------------------------------------------------------------
// Back-up
// ---------------------------------------------------------------------------

/** Envelope dat we naar disk schrijven als back-up vóór de wissel. */
interface BackupEnvelope {
  version: string;
  schema: "isso51-norm-switch-backup-v1";
  exported_at: string;
  norm: ActiveNorm;
  project: Project;
  result: ProjectResult | null;
  /** Sidecar — alleen relevant wanneer norm === "isso53". */
  isso53Building?: Isso53BuildingState;
  isso53Rooms?: Record<string, Isso53RoomState>;
}

const BACKUP_VERSION = "1.0.0";

/** Sanitize a project name to a filesystem-safe filename fragment. */
function safeName(name: string): string {
  return (name || "project").replace(/[^a-zA-Z0-9_\-\s]/g, "").trim() || "project";
}

/**
 * Construeer de back-up bestandsnaam:
 *   `<projectnaam> (v ISSO 51 backup).json`
 *
 * `currentNorm` is de norm waar het project NU op staat — dat is
 * de versie die back-up'd wordt vóór de wissel.
 */
export function buildBackupFileName(
  projectName: string,
  currentNorm: ActiveNorm,
): string {
  const normLabel = currentNorm === "isso51" ? "ISSO 51" : "ISSO 53";
  return `${safeName(projectName)} (v ${normLabel} backup).json`;
}

/**
 * Bepaal het back-up pad in Tauri-mode op basis van het huidige project-pad
 * — zelfde map als het origineel. Web-mode retourneert `null` (caller
 * doet blob-download).
 */
async function deriveBackupPath(
  currentLocalPath: string | null,
  projectName: string,
  currentNorm: ActiveNorm,
): Promise<string | null> {
  if (!isTauri()) return null;
  const fileName = buildBackupFileName(projectName, currentNorm);
  try {
    const { join, documentDir, dirname } = await import("@tauri-apps/api/path");
    if (currentLocalPath) {
      const dir = await dirname(currentLocalPath);
      return await join(dir, fileName);
    }
    // Geen huidig pad — val terug op Documents/Open Heatloss Studio
    const docs = await documentDir();
    const folder = await join(docs, "Open Heatloss Studio");
    const { mkdir } = await import("@tauri-apps/plugin-fs");
    await mkdir(folder, { recursive: true }).catch(() => {});
    return await join(folder, fileName);
  } catch {
    return null;
  }
}

/**
 * Schrijf de back-up naar disk (Tauri) of triggert een blob-download (web).
 *
 * Returns het pad waar geschreven werd (Tauri-succes) of `null` (web of
 * fout). Niet-fatale fouten worden door de caller opgevangen.
 */
export async function writeNormSwitchBackup(params: {
  project: Project;
  result: ProjectResult | null;
  currentNorm: ActiveNorm;
  currentLocalPath: string | null;
  isso53Building?: Isso53BuildingState;
  isso53Rooms?: Record<string, Isso53RoomState>;
}): Promise<string | null> {
  const envelope: BackupEnvelope = {
    version: BACKUP_VERSION,
    schema: "isso51-norm-switch-backup-v1",
    exported_at: new Date().toISOString(),
    norm: params.currentNorm,
    project: params.project,
    result: params.result,
    isso53Building: params.currentNorm === "isso53" ? params.isso53Building : undefined,
    isso53Rooms: params.currentNorm === "isso53" ? params.isso53Rooms : undefined,
  };
  const json = JSON.stringify(envelope, null, 2);
  const fileName = buildBackupFileName(
    params.project.info.name,
    params.currentNorm,
  );

  if (isTauri()) {
    try {
      const backupPath = await deriveBackupPath(
        params.currentLocalPath,
        params.project.info.name,
        params.currentNorm,
      );
      if (!backupPath) {
        throw new Error("Kon back-up pad niet bepalen");
      }
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(backupPath, json);
      return backupPath;
    } catch (err) {
      // Fall through to blob-download als Tauri-write faalt
      console.error("Back-up via Tauri-FS faalde, val terug op blob-download:", err);
    }
  }

  // Web-mode / Tauri-fallback: blob-download via anchor
  try {
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    a.click();
    URL.revokeObjectURL(url);
  } catch {
    // Geen DOM (SSR / test) — geen fatale fout
  }
  return null;
}
