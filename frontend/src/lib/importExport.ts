/**
 * JSON import/export for ISSO 51 projects.
 *
 * Export wraps the project + result in a versioned envelope.
 * Import accepts both the envelope format and raw Project JSON.
 */
import type { Project, ProjectResult } from "../types";

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
    const result = obj.result as ProjectResult | null ?? null;
    return { project, result };
  }

  // Try as raw Project JSON.
  const project = validateProject(data);
  return { project, result: null };
}

/**
 * Validate that the data looks like a Project (basic structural checks).
 */
function validateProject(data: unknown): Project {
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
