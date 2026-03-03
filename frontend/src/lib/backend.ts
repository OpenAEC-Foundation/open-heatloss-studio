import type { Project, ProjectResult } from "../types";
import { API_PREFIX } from "./constants";

/** Backend interface — same API for web (fetch) and Tauri (invoke). */
export interface Backend {
  calculate(project: Project): Promise<ProjectResult>;
  getSchema(name: "project" | "result"): Promise<unknown>;
}

/** Check if running inside Tauri. */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/** Create the appropriate backend for the current environment. */
export function createBackend(): Backend {
  return isTauri() ? createTauriBackend() : createWebBackend();
}

function createWebBackend(): Backend {
  return {
    async calculate(project) {
      const res = await fetch(`${API_PREFIX}/calculate`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(project),
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ detail: res.statusText }));
        throw new Error((err as { detail?: string }).detail ?? "Berekening mislukt");
      }
      return res.json() as Promise<ProjectResult>;
    },

    async getSchema(name) {
      const res = await fetch(`${API_PREFIX}/schemas/${name}`);
      if (!res.ok) {
        throw new Error(`Schema '${name}' niet gevonden`);
      }
      return res.json();
    },
  };
}

function createTauriBackend(): Backend {
  // Dynamic import so Tauri modules are tree-shaken in web builds.
  const invokeAsync = async <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  };

  return {
    async calculate(project) {
      return invokeAsync<ProjectResult>("calculate", { project });
    },

    async getSchema(name) {
      const json = await invokeAsync<string>("get_schema", { which: name });
      return JSON.parse(json);
    },
  };
}
