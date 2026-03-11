/**
 * JSON import/export for ISSO 51 projects.
 *
 * Export wraps the project + result in a versioned envelope.
 * Import accepts both the envelope format and raw Project JSON.
 */
import type { ConstructionElement, Project, ProjectResult, VerticalPosition } from "../types";
import type { CatalogueCategory } from "./constructionCatalogue";
import type { ProjectConstruction } from "../components/modeller/types";
import { useModellerStore } from "../components/modeller/modellerStore";

const SCHEMA_ID = "isso51-project-v1";
const EXPORT_VERSION = "1.0.0";

/** Envelope format written to disk. */
interface ProjectEnvelope {
  version: string;
  schema: string;
  exported_at: string;
  project: Project;
  result: ProjectResult | null;
}

/** Result of a successful import. */
export interface ImportResult {
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
  const envelope: ProjectEnvelope = {
    version: EXPORT_VERSION,
    schema: SCHEMA_ID,
    exported_at: new Date().toISOString(),
    project,
    result,
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
 * Import a project from a JSON file.
 *
 * Accepts:
 * - Wrapped format: `{ schema: "isso51-project-v1", project: {...} }`
 * - Raw Project JSON: `{ info: {...}, building: {...}, ... }`
 */
export function importProject(jsonString: string): ImportResult {
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

  // Detect envelope format.
  if (obj.schema === SCHEMA_ID && obj.project) {
    const project = validateProject(obj.project);
    const result = validateProjectResult(obj.result);
    return { project, result };
  }

  // Try as raw Project JSON.
  const project = validateProject(data);
  return { project, result: null };
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

  if (!Array.isArray(obj.rooms)) {
    throw new Error("Verplicht veld 'rooms' ontbreekt of is geen array");
  }

  // Ensure info exists.
  if (!obj.info || typeof obj.info !== "object") {
    (obj as Record<string, unknown>).info = { name: "" };
  }

  return data as Project;
}

// ---------------------------------------------------------------------------
// Construction extraction — dedup + link on import
// ---------------------------------------------------------------------------

/** Fingerprint for deduplication: same type = same construction. */
function constructionFingerprint(c: ConstructionElement): string {
  return `${c.description}|${c.u_value}|${c.material_type}|${c.vertical_position ?? "wall"}|${c.boundary_type}`;
}

/** Map vertical_position to CatalogueCategory. */
function categoryFromPosition(vp: VerticalPosition | undefined): CatalogueCategory {
  if (vp === "ceiling") return "daken";
  if (vp === "floor") return "vloeren_plafonds";
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
  const existing = store.projectConstructions;

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
        category: categoryFromPosition(ce.vertical_position),
        materialType: ce.material_type,
        verticalPosition: (ce.vertical_position ?? "wall") as VerticalPosition,
        layers: ce.layers ? structuredClone(ce.layers) : [],
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
