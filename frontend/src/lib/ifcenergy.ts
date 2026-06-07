/**
 * `.ifcenergy` file-format builder/parser.
 *
 * Het bestand is een geldige IFCX (IFC5 alpha) document met de volledige
 * project-state als payload op één IfcProject entry. Voor PR B (Phase 1)
 * gebruiken we een "JSON-in-IFCX" aanpak: alle data in één
 * `isso51::envelope::v1` attribute. Een toekomstige PR kan dit splitsen in
 * per-IfcSpace/IfcWindow/IfcDoor attributen volgens de namespace-constants
 * gedefinieerd in `crates/isso51-ifcx/src/namespace.rs`.
 *
 * De .ifcenergy file is hierdoor:
 * - Een geldige IFCX (header + imports + data array)
 * - Roundtrip-correct (JSON serialize → parse → identieke state)
 * - Compatibel met toekomstige proper-IFCX migratie (één envelope splitten
 *   in vele namespace attributen behoudt het bestand parseerbaar via een
 *   migrate-functie)
 *
 * Modeller geometrie wordt meegenomen in `isso51::modeller::*` als forward
 * compat — momenteel meestal leeg (modeller is read-only viewer derived van
 * project.rooms sinds PR D), maar de envelope kan alle modellerStore state
 * dragen voor wanneer editable-modus terugkomt.
 */
import type {
  IfcxDataEntry,
  IfcxDocument,
} from "../components/modeller/ifcx";
import { createIfcxDocument, IFCX_NS, IFC_CLASS, uuid } from "../components/modeller/ifcx";
import type { Project, ProjectResult } from "../types";
import type {
  ModelDoor,
  ModelRoom,
  ModelWindow,
  ProjectConstruction,
  WallBoundaryType,
} from "../components/modeller/types";
import type { UnderlayImage } from "../components/modeller/modellerStore";
import type { SharedExtra } from "../types/projectV2";
import type { VentilationState } from "../types/ventilation";

// ---------------------------------------------------------------------------
// Namespace constants — mirror crates/isso51-ifcx/src/namespace.rs
// ---------------------------------------------------------------------------

export const ISSO51_ENVELOPE_NS = "isso51::envelope::v1";
export const ISSO51_MODELLER_ROOM_NS = "isso51::modeller::room";
export const ISSO51_MODELLER_WINDOW_NS = "isso51::modeller::window";
export const ISSO51_MODELLER_DOOR_NS = "isso51::modeller::door";
export const ISSO51_MODELLER_PC_NS = "isso51::modeller::project_constructions";
export const ISSO51_MODELLER_ASSIGNMENTS_NS = "isso51::modeller::assignments";
export const ISSO51_MODELLER_UNDERLAY_NS = "isso51::modeller::underlay";

// ---------------------------------------------------------------------------
// Modeller snapshot — full state to roundtrip
// ---------------------------------------------------------------------------

export type { UnderlayImage };

export interface ModellerSnapshot {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  projectConstructions: ProjectConstruction[];
  wallConstructions: Record<string, string>;
  floorConstructions: Record<string, string>;
  roofConstructions: Record<string, string>;
  wallBoundaryTypes: Record<string, WallBoundaryType>;
  underlay: UnderlayImage | null;
}

/** Empty modeller snapshot — used when no geometry was authored. */
export function emptyModellerSnapshot(): ModellerSnapshot {
  return {
    rooms: [],
    windows: [],
    doors: [],
    projectConstructions: [],
    wallConstructions: {},
    floorConstructions: {},
    roofConstructions: {},
    wallBoundaryTypes: {},
    underlay: null,
  };
}

// ---------------------------------------------------------------------------
// Envelope payload — version-tagged for forward migration
// ---------------------------------------------------------------------------

const ENVELOPE_VERSION = "1.0.0";

interface IfcEnergyEnvelope {
  /** Schema version for forward migration. */
  version: string;
  /** When the file was authored (ISO 8601). */
  exportedAt: string;
  project: Project;
  /** Calculation result if available; null when project hasn't been calculated yet. */
  result: ProjectResult | null;
  modeller: ModellerSnapshot;
  /**
   * V2-only sidecar-velden (`construction_year`, postcode, …) die buiten het
   * V1 `Project` type in `projectStore.sharedExtra` leven. Optioneel — oude
   * `.ifcenergy`-bestanden hebben dit veld niet; de parser valt dan terug op
   * defaults. Zonder dit veld ging o.a. het bouwjaar verloren bij heropenen.
   */
  sharedExtra?: SharedExtra;
  /**
   * Ventilatiebalans-sidecar (ventielen + per-room ventilatie-velden).
   * Optioneel — oude `.ifcenergy`-bestanden hebben dit veld niet. Zonder dit
   * veld gingen ventielen verloren bij heropenen (valkuil commit `8ccff9f`).
   */
  ventilation?: VentilationState;
}

// ---------------------------------------------------------------------------
// Build .ifcenergy document
// ---------------------------------------------------------------------------

export interface BuildIfcEnergyOptions {
  project: Project;
  result: ProjectResult | null;
  modeller: ModellerSnapshot;
  /**
   * V2-only sidecar-velden (bouwjaar etc.). Optioneel — alleen meegeven
   * wanneer er betekenisvolle V2-data is. Wordt 1:1 in de envelope opgenomen.
   */
  sharedExtra?: SharedExtra;
  /**
   * Ventilatiebalans-sidecar. Optioneel — alleen meegeven wanneer er
   * betekenisvolle ventilatie-data is. Wordt 1:1 in de envelope opgenomen.
   */
  ventilation?: VentilationState;
  author?: string;
}

/**
 * Build an IFCX document representing the entire project + result + modeller.
 *
 * Phase 1 design (PR B): single envelope attribute on IfcProject. Future PRs
 * can split the envelope into per-entry isso51:: attributes for true IFCX
 * structure — at that point the parser must accept BOTH the v1 envelope and
 * the v2 split form. Hence the version field.
 */
export function buildIfcEnergyDocument(opts: BuildIfcEnergyOptions): IfcxDocument {
  const author = opts.author ?? "Open Heatloss Studio";
  const doc = createIfcxDocument({ id: uuid(), author });

  const projectPath = uuid();
  const sitePath = uuid();
  const buildingPath = uuid();

  // Root entry — IfcProject with the envelope attached.
  const envelope: IfcEnergyEnvelope = {
    version: ENVELOPE_VERSION,
    exportedAt: new Date().toISOString(),
    project: opts.project,
    result: opts.result,
    modeller: opts.modeller,
    ...(opts.sharedExtra ? { sharedExtra: opts.sharedExtra } : {}),
    ...(opts.ventilation ? { ventilation: opts.ventilation } : {}),
  };

  const projectEntry: IfcxDataEntry = {
    path: projectPath,
    children: { Site: sitePath },
    attributes: {
      [IFCX_NS.class]: { code: IFC_CLASS.Project, uri: ifcUri(IFC_CLASS.Project) },
      [`${IFCX_NS.prop}::Name`]: opts.project.info.name || "Project",
      [ISSO51_ENVELOPE_NS]: envelope,
    },
  };

  const siteEntry: IfcxDataEntry = {
    path: sitePath,
    children: { Building: buildingPath },
    attributes: {
      [IFCX_NS.class]: { code: IFC_CLASS.Site, uri: ifcUri(IFC_CLASS.Site) },
    },
  };

  const buildingEntry: IfcxDataEntry = {
    path: buildingPath,
    attributes: {
      [IFCX_NS.class]: { code: IFC_CLASS.Building, uri: ifcUri(IFC_CLASS.Building) },
      [`${IFCX_NS.prop}::Name`]: opts.project.info.name || "Building",
    },
  };

  doc.data.push(projectEntry, siteEntry, buildingEntry);

  return doc;
}

/** Serialize an .ifcenergy document to a JSON string for file writing. */
export function serializeIfcEnergy(doc: IfcxDocument): string {
  return JSON.stringify(doc, null, 2);
}

// ---------------------------------------------------------------------------
// Parse .ifcenergy document
// ---------------------------------------------------------------------------

export interface ParsedIfcEnergy {
  project: Project;
  result: ProjectResult | null;
  modeller: ModellerSnapshot;
  /**
   * V2-only sidecar-velden uit de envelope, indien aanwezig. `undefined` voor
   * oude `.ifcenergy`-bestanden zonder dit veld — caller valt terug op defaults.
   */
  sharedExtra?: SharedExtra;
  /**
   * Ventilatiebalans-sidecar uit de envelope, indien aanwezig. `undefined`
   * voor bestanden zonder ventilatie-data — caller valt terug op leeg.
   */
  ventilation?: VentilationState;
  /** Schema version of the parsed envelope (for migration logic). */
  envelopeVersion: string;
}

/**
 * Parse a JSON string as a .ifcenergy document.
 *
 * Throws with a Dutch message on structural problems so the UI can surface
 * meaningful errors. Caller is responsible for distinguishing .ifcenergy from
 * legacy .isso51.json — see `detectFormat` in importExport.ts.
 */
export function parseIfcEnergy(jsonString: string): ParsedIfcEnergy {
  let parsed: unknown;
  try {
    parsed = JSON.parse(jsonString);
  } catch {
    throw new Error("Ongeldig JSON-bestand");
  }
  if (!parsed || typeof parsed !== "object") {
    throw new Error("Bestand is geen IFCX-document");
  }

  const doc = parsed as Partial<IfcxDocument>;
  if (!doc.header?.ifcxVersion || !Array.isArray(doc.data)) {
    throw new Error("Bestand mist IFCX-header of data-array");
  }

  // Find the IfcProject entry that carries the envelope.
  for (const entry of doc.data) {
    const env = entry.attributes?.[ISSO51_ENVELOPE_NS];
    if (env && typeof env === "object") {
      const e = env as Partial<IfcEnergyEnvelope>;
      if (!e.project) {
        throw new Error(
          `IFCX envelope (${ISSO51_ENVELOPE_NS}) mist 'project' veld`,
        );
      }
      const modeller =
        e.modeller && typeof e.modeller === "object"
          ? mergeModellerSnapshot(e.modeller)
          : emptyModellerSnapshot();
      const sharedExtra =
        e.sharedExtra && typeof e.sharedExtra === "object"
          ? (e.sharedExtra as SharedExtra)
          : undefined;
      const ventilation =
        e.ventilation &&
        typeof e.ventilation === "object" &&
        Array.isArray((e.ventilation as VentilationState).terminals)
          ? (e.ventilation as VentilationState)
          : undefined;
      return {
        project: e.project as Project,
        result: (e.result ?? null) as ProjectResult | null,
        modeller,
        sharedExtra,
        ventilation,
        envelopeVersion: e.version ?? "unknown",
      };
    }
  }

  throw new Error(
    `Geen ${ISSO51_ENVELOPE_NS} attribute gevonden in IFCX-document`,
  );
}

/** Fill missing fields in a modeller snapshot with defaults — defensive parse. */
function mergeModellerSnapshot(partial: Partial<ModellerSnapshot>): ModellerSnapshot {
  const empty = emptyModellerSnapshot();
  return {
    rooms: Array.isArray(partial.rooms) ? partial.rooms : empty.rooms,
    windows: Array.isArray(partial.windows) ? partial.windows : empty.windows,
    doors: Array.isArray(partial.doors) ? partial.doors : empty.doors,
    projectConstructions: Array.isArray(partial.projectConstructions)
      ? partial.projectConstructions
      : empty.projectConstructions,
    wallConstructions:
      partial.wallConstructions && typeof partial.wallConstructions === "object"
        ? partial.wallConstructions
        : empty.wallConstructions,
    floorConstructions:
      partial.floorConstructions && typeof partial.floorConstructions === "object"
        ? partial.floorConstructions
        : empty.floorConstructions,
    roofConstructions:
      partial.roofConstructions && typeof partial.roofConstructions === "object"
        ? partial.roofConstructions
        : empty.roofConstructions,
    wallBoundaryTypes:
      partial.wallBoundaryTypes && typeof partial.wallBoundaryTypes === "object"
        ? partial.wallBoundaryTypes
        : empty.wallBoundaryTypes,
    underlay:
      partial.underlay && typeof partial.underlay === "object"
        ? (partial.underlay as UnderlayImage)
        : null,
  };
}

// ---------------------------------------------------------------------------
// Format detection
// ---------------------------------------------------------------------------

export type DetectedFormat = "ifcenergy" | "isso51-legacy" | "thermal-import" | "unknown";

/** Sources that indicate a thermal import file (Revit/IFC export). */
const THERMAL_SOURCES = new Set(["revit-eam", "revit-raycast", "ifc"]);

/**
 * Detect file format from JSON content shape. Used by the import flow to
 * route to the correct parser without relying on file extension.
 */
export function detectFormat(jsonString: string): DetectedFormat {
  let parsed: unknown;
  try {
    parsed = JSON.parse(jsonString);
  } catch {
    return "unknown";
  }
  if (!parsed || typeof parsed !== "object") return "unknown";
  const obj = parsed as Record<string, unknown>;

  // Thermal import (Revit/IFC) — check first because of legacy `source` field
  if (typeof obj.source === "string" && THERMAL_SOURCES.has(obj.source)) {
    return "thermal-import";
  }

  // IFCX shape — has header.ifcxVersion + data array
  const header = obj.header as Record<string, unknown> | undefined;
  if (header && typeof header.ifcxVersion === "string" && Array.isArray(obj.data)) {
    return "ifcenergy";
  }

  // Legacy envelope — has schema field
  if (obj.schema === "isso51-project-v1" && obj.project) {
    return "isso51-legacy";
  }

  // Raw Project JSON (no envelope)
  if (obj.building && obj.climate && obj.ventilation && Array.isArray(obj.rooms)) {
    return "isso51-legacy";
  }

  return "unknown";
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function ifcUri(classCode: string): string {
  return `https://identifier.buildingsmart.org/uri/buildingsmart/ifc/4.3/class/${classCode}`;
}
