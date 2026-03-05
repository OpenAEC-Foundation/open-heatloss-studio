import type {
  Project,
  ProjectResult,
  UserProfile,
  ProjectSummary,
  ProjectResponse,
} from "../types";
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

// ---------------------------------------------------------------------------
// Authenticated API helpers (web only, uses OIDC access token)
// ---------------------------------------------------------------------------

/** Fetch with Bearer token from OIDC session (if logged in). */
async function authFetch(url: string, init?: RequestInit): Promise<Response> {
  const headers = new Headers(init?.headers);
  headers.set("Content-Type", "application/json");

  try {
    const { getOidc } = await import("./oidc");
    const oidc = await getOidc();
    if (oidc.isUserLoggedIn) {
      const token = await oidc.getAccessToken();
      headers.set("Authorization", `Bearer ${token}`);
    }
  } catch {
    // OIDC not initialized (e.g. Tauri) — proceed without token.
  }

  return fetch(url, { ...init, headers });
}

/** Parse JSON response or throw with error detail. */
async function parseResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error((err as { detail?: string }).detail ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

// ---------------------------------------------------------------------------
// User API
// ---------------------------------------------------------------------------

/** GET /me — Fetch/upsert current user profile. */
export async function fetchProfile(): Promise<UserProfile> {
  const res = await authFetch(`${API_PREFIX}/me`);
  return parseResponse<UserProfile>(res);
}

// ---------------------------------------------------------------------------
// Projects API
// ---------------------------------------------------------------------------

/** GET /projects — List user's projects. */
export async function fetchProjects(): Promise<ProjectSummary[]> {
  const res = await authFetch(`${API_PREFIX}/projects`);
  return parseResponse<ProjectSummary[]>(res);
}

/** POST /projects — Create a new project. */
export async function createProject(
  name: string,
  projectData: Project,
): Promise<{ id: string; name: string }> {
  const res = await authFetch(`${API_PREFIX}/projects`, {
    method: "POST",
    body: JSON.stringify({ name, project_data: projectData }),
  });
  return parseResponse<{ id: string; name: string }>(res);
}

/** GET /projects/:id — Load a project. */
export async function fetchProject(id: string): Promise<ProjectResponse> {
  const res = await authFetch(`${API_PREFIX}/projects/${id}`);
  return parseResponse<ProjectResponse>(res);
}

/** PUT /projects/:id — Update a project. */
export async function updateProject(
  id: string,
  data: { name?: string; project_data?: Project },
): Promise<void> {
  const res = await authFetch(`${API_PREFIX}/projects/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  await parseResponse<unknown>(res);
}

/** DELETE /projects/:id — Soft-delete a project. */
export async function deleteProject(id: string): Promise<void> {
  const res = await authFetch(`${API_PREFIX}/projects/${id}`, {
    method: "DELETE",
  });
  await parseResponse<unknown>(res);
}

/** POST /projects/:id/calculate — Calculate and save result server-side. */
export async function calculateAndSave(id: string): Promise<ProjectResult> {
  const res = await authFetch(`${API_PREFIX}/projects/${id}/calculate`, {
    method: "POST",
  });
  return parseResponse<ProjectResult>(res);
}
