import type {
  Project,
  ProjectResult,
  UserProfile,
  ProjectSummary,
  ProjectResponse,
} from "../types";
import { API_PREFIX } from "./constants";

/** IFC import result from the Python sidecar. */
export interface IfcSidecarResult {
  rooms: Array<{
    name: string;
    function: string;
    polygon: Array<{ x: number; y: number }>;
    floor: number;
    height: number;
    elevation?: number | null;
    temperature?: number | null;
  }>;
  windows: Array<{
    roomId: string;
    wallIndex: number;
    offset: number;
    width: number;
    height?: number;
    sillHeight?: number;
  }>;
  doors: Array<{
    roomId: string;
    wallIndex: number;
    offset: number;
    width: number;
    height?: number;
    swing: "left" | "right";
  }>;
  wallTypes: Array<{
    name: string;
    globalId: string;
    layers: Array<{
      materialName: string;
      thicknessMm: number;
      match: string | null;
    }>;
    originalMaterialNames: string[];
  }>;
  sharedEdges: Array<{
    roomAIndex: number;
    wallAIndex: number;
    roomBIndex: number;
    wallBIndex: number;
    distanceMm: number;
    overlapMm: number;
  }>;
  warnings: Array<{ spaceName: string; message: string }>;
  diagnostics: Array<{
    spaceId: number;
    spaceName: string;
    strategy: string;
    polygonPoints: number;
    areaMm2: number;
  }>;
  stats: {
    spacesFound: number;
    spacesImported: number;
    spacesSkipped: number;
  };
}

/** Backend interface — same API for web (fetch) and Tauri (invoke). */
export interface Backend {
  calculate(project: Project): Promise<ProjectResult>;
  getSchema(name: "project" | "result"): Promise<unknown>;
  /** Import IFC via native sidecar (Tauri only). Returns null in web mode. */
  importIfc?(filePath: string): Promise<IfcSidecarResult>;
}

/**
 * Check if running inside Tauri.
 *
 * Tries multiple detection methods because Tauri 2's `__TAURI_INTERNALS__`
 * global is injected synchronously by the webview runtime — but in some
 * webview implementations there may be a tiny window where it is not yet
 * present when initial JS evaluates. The user-agent and `__TAURI__` fallback
 * cover those edge cases.
 */
export function isTauri(): boolean {
  if (typeof window === "undefined") return false;
  if ("__TAURI_INTERNALS__" in window) return true;
  if ("__TAURI__" in window) return true; // legacy
  if (typeof navigator !== "undefined" && /Tauri\//.test(navigator.userAgent)) return true;
  return false;
}

/**
 * Create a backend that re-detects the runtime environment on each call.
 *
 * Detecting per-call (instead of once at construction) avoids a subtle
 * footgun: if `useBackend()` was called before Tauri injected its globals,
 * the cached web-mode backend would survive forever — silently routing
 * desktop calls to a non-existent HTTP server. Per-call detection costs
 * almost nothing and guarantees correctness as soon as Tauri is ready.
 */
export function createBackend(): Backend {
  return {
    async calculate(project) {
      return isTauri()
        ? createTauriBackend().calculate(project)
        : createWebBackend().calculate(project);
    },
    async getSchema(name) {
      return isTauri()
        ? createTauriBackend().getSchema(name)
        : createWebBackend().getSchema(name);
    },
    async importIfc(filePath) {
      if (!isTauri()) {
        throw new Error("IFC import via sidecar vereist desktop-app");
      }
      // createTauriBackend().importIfc is always defined (zie createTauriBackend).
      return createTauriBackend().importIfc!(filePath);
    },
  };
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

    async importIfc(filePath: string) {
      return invokeAsync<IfcSidecarResult>("import_ifc", {
        filePath,
      });
    },
  };
}

// ---------------------------------------------------------------------------
// Server-side IFC import (web mode — same pipeline as Tauri sidecar)
// ---------------------------------------------------------------------------

/**
 * Upload an IFC file to the server for import via the Python sidecar.
 *
 * Authentik forward_auth (cookie-based) is added by the browser via
 * `credentials: "include"` — no Bearer token needed.
 */
export async function importIfcServer(file: File): Promise<IfcSidecarResult> {
  const formData = new FormData();
  formData.append("file", file);

  const res = await fetch(`${API_PREFIX}/ifc/import`, {
    method: "POST",
    credentials: "include",
    body: formData,
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error(
      (err as { detail?: string }).detail ?? `IFC import mislukt (HTTP ${res.status})`,
    );
  }

  return res.json() as Promise<IfcSidecarResult>;
}

// ---------------------------------------------------------------------------
// TO-juli dispatch helpers (Tauri invoke or web fetch)
// ---------------------------------------------------------------------------

/** Dispatch helper that returns Tauri invoke or web fetch based on runtime. */
async function invokeOrFetch<TPayload, TResult>(
  tauriCmd: string,
  webPath: string,
  payload: TPayload,
): Promise<TResult> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<TResult>(tauriCmd, { req: payload });
  }
  const res = await fetch(`${API_PREFIX}${webPath}`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error((err as { detail?: string }).detail ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<TResult>;
}

/** TO-juli vereenvoudigde koelbehoefte (Tauri of web). */
export function simplifiedCooling<TReq, TRes>(req: TReq): Promise<TRes> {
  return invokeOrFetch<TReq, TRes>("simplified_cooling", "/cooling/simplified", req);
}

/** TO-juli volledige H.10 berekening (Tauri of web). */
export function tojuliCalculate<TReq, TRes>(req: TReq): Promise<TRes> {
  return invokeOrFetch<TReq, TRes>("tojuli_calculate", "/tojuli/calculate", req);
}

// ---------------------------------------------------------------------------
// Authenticated API helpers (web only — uses Authentik forward_auth cookie)
// ---------------------------------------------------------------------------

/**
 * Fetch helper that includes the `authentik_session` cookie. The backend
 * reads the user identity from the `X-Authentik-*` headers that Caddy
 * injects after a successful forward_auth handshake.
 */
export async function authFetch(url: string, init?: RequestInit): Promise<Response> {
  const headers = new Headers(init?.headers);
  if (!headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  return fetch(url, {
    ...init,
    credentials: "include",
    headers,
  });
}

/**
 * Detect whether a response indicates an expired/missing Authentik session.
 *
 * An `authFetch` call always targets a same-origin API endpoint, so the only
 * realistic reason it gets redirected or returns the Authentik login HTML is
 * the forward_auth proxy bouncing the (cookie-authenticated) request to its
 * login page. We treat three signals as "session expired":
 *
 *  1. `res.redirected` — `fetch` transparently followed a 30x to the Authentik
 *     login screen; the request never reached the API.
 *  2. `res.status` 401 / 403 — backend (or proxy) explicitly rejected us.
 *  3. `res.ok` but the body is HTML — the Authentik login page came back with
 *     a 200, which would otherwise crash `res.json()` with a cryptic
 *     SyntaxError. We match HTML *positively* (not "non-JSON") so that a
 *     legitimate 204 No Content / empty-body 2xx response is not flagged.
 */
function isSessionExpired(res: Response): boolean {
  if (res.redirected) return true;
  if (res.status === 401 || res.status === 403) return true;
  if (res.ok) {
    const contentType = res.headers.get("content-type") ?? "";
    if (contentType.includes("html")) return true;
  }
  return false;
}

/** Parse JSON response or throw with error detail. */
async function parseResponse<T>(res: Response): Promise<T> {
  if (isSessionExpired(res)) {
    throw new SessionExpiredError("Je sessie is verlopen — log opnieuw in.");
  }
  if (!res.ok) {
    const err = await res.json().catch(() => ({ detail: res.statusText }));
    throw new Error((err as { detail?: string }).detail ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

/**
 * Thrown when an authenticated API call hits an expired Authentik session.
 *
 * The UI catches this to show a "log in again" message instead of a cryptic
 * JSON parse error (the Authentik login page is HTML, not JSON).
 */
export class SessionExpiredError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SessionExpiredError";
  }
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

/** Response from PUT /projects/:id. */
interface UpdateProjectResponse {
  ok: boolean;
  updated_at: string;
}

/** PUT /projects/:id — Update a project. */
export async function updateProject(
  id: string,
  data: { name?: string; project_data?: Project; expected_updated_at?: string },
): Promise<UpdateProjectResponse> {
  const res = await authFetch(`${API_PREFIX}/projects/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (res.status === 409) {
    throw new ConflictError("Project is elders gewijzigd");
  }
  return parseResponse<UpdateProjectResponse>(res);
}

/** Thrown when the server detects a conflict (409). */
export class ConflictError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ConflictError";
  }
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

