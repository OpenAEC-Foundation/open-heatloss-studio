import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { ConstructionElement, Project, ProjectResult, Room } from "../types";

// ---------------------------------------------------------------------------
// Undo/Redo history
// ---------------------------------------------------------------------------

const MAX_HISTORY = 50;

interface ProjectSnapshot {
  project: Project;
}

function takeProjectSnapshot(state: { project: Project }): ProjectSnapshot {
  return { project: structuredClone(state.project) };
}

/** Default project for a new calculation. */
const DEFAULT_PROJECT: Project = {
  info: {
    name: "",
  },
  building: {
    building_type: "terraced",
    qv10: 100,
    total_floor_area: 80,
    security_class: "b",
    has_night_setback: true,
    warmup_time: 2,
    num_floors: 1,
  },
  climate: {
    theta_e: -10,
    theta_b_residential: 17,
    theta_b_non_residential: 14,
    wind_factor: 1.0,
    theta_water: 5,
  },
  ventilation: {
    system_type: "system_c",
    has_heat_recovery: false,
  },
  rooms: [],
};

interface ProjectStore {
  /** Current project input data. */
  project: Project;
  /** Calculation result (null if not yet calculated). */
  result: ProjectResult | null;
  /** Error message from last calculation attempt. */
  error: string | null;
  /** Whether a calculation is in progress. */
  isCalculating: boolean;
  /** Whether the project has unsaved changes since last calculation. */
  isDirty: boolean;
  /** Server-side project ID (null for local-only projects). */
  activeProjectId: string | null;
  /** Server-side updated_at timestamp for conflict detection. */
  serverUpdatedAt: string | null;
  /** Whether a save conflict was detected. */
  hasConflict: boolean;

  /** Undo history (not persisted). */
  _past: ProjectSnapshot[];
  /** Redo history (not persisted). */
  _future: ProjectSnapshot[];

  /** Update project data (partial merge). */
  updateProject: (partial: Partial<Project>) => void;
  /**
   * Zet (of wist) de project-brede override voor de U-waarde van
   * kozijnen. Geef `undefined` mee om de override te wissen — de
   * individuele per-element waardes blijven dan intact.
   */
  setFrameUValueOverride: (value: number | undefined) => void;
  /** Replace the entire project. */
  setProject: (project: Project) => void;
  /** Set the active server-side project ID. */
  setActiveProjectId: (id: string | null) => void;
  /** Set the calculation result. */
  setResult: (result: ProjectResult) => void;
  /** Set an error from a failed calculation. */
  setError: (error: string) => void;
  /** Clear the current error. */
  clearError: () => void;
  /** Set calculating state. */
  setCalculating: (isCalculating: boolean) => void;
  /** Load a server project atomically (project + id + result in one set). */
  loadServerProject: (
    id: string,
    project: Project,
    result: ProjectResult | null,
    updatedAt?: string,
  ) => void;
  /** Update the server timestamp after a successful save. */
  setServerUpdatedAt: (updatedAt: string | null) => void;
  /** Reset to default state. */
  reset: () => void;

  /** Add a room to the project. */
  addRoom: (room: Room) => void;
  /** Update a room by id (partial merge). */
  updateRoom: (roomId: string, partial: Partial<Room>) => void;
  /** Remove a room by id. */
  removeRoom: (roomId: string) => void;
  /** Add a construction to a room. */
  addConstruction: (roomId: string, construction: ConstructionElement) => void;
  /** Update a construction in a room (partial merge). */
  updateConstruction: (
    roomId: string,
    constructionId: string,
    partial: Partial<ConstructionElement>,
  ) => void;
  /** Remove a construction from a room. */
  removeConstruction: (roomId: string, constructionId: string) => void;

  /** Undo last project mutation. */
  undo: () => void;
  /** Redo last undone project mutation. */
  redo: () => void;
}

export const useProjectStore = create<ProjectStore>()(
  persist(
    (set, get) => ({
      project: DEFAULT_PROJECT,
      result: null,
      error: null,
      isCalculating: false,
      isDirty: true,
      activeProjectId: null,
      serverUpdatedAt: null,
      hasConflict: false,
      _past: [],
      _future: [],

      setActiveProjectId: (id) => set({ activeProjectId: id }),
      setServerUpdatedAt: (updatedAt) => set({ serverUpdatedAt: updatedAt }),

      updateProject: (partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: { ...state.project, ...partial },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      setFrameUValueOverride: (value) => {
        const snap = takeProjectSnapshot(get());
        set((state) => {
          const next: Project = { ...state.project };
          if (value === undefined || !Number.isFinite(value) || value <= 0) {
            delete next.frameUValueOverride;
          } else {
            next.frameUValueOverride = value;
          }
          return {
            project: next,
            isDirty: true,
            error: null,
            _past: [...state._past, snap].slice(-MAX_HISTORY),
            _future: [],
          };
        });
      },

      setProject: (project) =>
        set({ project, isDirty: true, result: null, error: null, activeProjectId: null, serverUpdatedAt: null, hasConflict: false, _past: [], _future: [] }),

      loadServerProject: (id, project, result, updatedAt) =>
        set({
          project,
          activeProjectId: id,
          result,
          isDirty: false,
          error: null,
          isCalculating: false,
          serverUpdatedAt: updatedAt ?? null,
          hasConflict: false,
          _past: [],
          _future: [],
        }),

      setResult: (result) =>
        set({ result, isDirty: false, error: null, isCalculating: false }),

      setError: (error) =>
        set({ error, isCalculating: false }),

      clearError: () =>
        set({ error: null }),

      setCalculating: (isCalculating) =>
        set({ isCalculating }),

      reset: () =>
        set({
          project: DEFAULT_PROJECT,
          result: null,
          error: null,
          isCalculating: false,
          isDirty: true,
          activeProjectId: null,
          serverUpdatedAt: null,
          hasConflict: false,
          _past: [],
          _future: [],
        }),

      addRoom: (room) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: [...state.project.rooms, room],
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      updateRoom: (roomId, partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId ? { ...r, ...partial } : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      removeRoom: (roomId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.filter((r) => r.id !== roomId),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      addConstruction: (roomId, construction) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? { ...r, constructions: [...r.constructions, construction] }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      updateConstruction: (roomId, constructionId, partial) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? {
                    ...r,
                    constructions: r.constructions.map((c) =>
                      c.id === constructionId ? { ...c, ...partial } : c,
                    ),
                  }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      removeConstruction: (roomId, constructionId) => {
        const snap = takeProjectSnapshot(get());
        set((state) => ({
          project: {
            ...state.project,
            rooms: state.project.rooms.map((r) =>
              r.id === roomId
                ? {
                    ...r,
                    constructions: r.constructions.filter(
                      (c) => c.id !== constructionId,
                    ),
                  }
                : r,
            ),
          },
          isDirty: true,
          error: null,
          _past: [...state._past, snap].slice(-MAX_HISTORY),
          _future: [],
        }));
      },

      undo: () => {
        const state = get();
        if (state._past.length === 0) return;
        const currentSnap = takeProjectSnapshot(state);
        const prev = state._past[state._past.length - 1]!;
        set({
          project: prev.project,
          _past: state._past.slice(0, -1),
          _future: [...state._future, currentSnap],
          isDirty: true,
        });
      },

      redo: () => {
        const state = get();
        if (state._future.length === 0) return;
        const currentSnap = takeProjectSnapshot(state);
        const next = state._future[state._future.length - 1]!;
        set({
          project: next.project,
          _past: [...state._past, currentSnap],
          _future: state._future.slice(0, -1),
          isDirty: true,
        });
      },
    }),
    {
      name: "isso51-project",
      version: 1,
      partialize: (state) => ({
        project: state.project,
        result: state.result,
      }),
      merge: (persisted, current) => ({
        ...current,
        ...(persisted as Pick<ProjectStore, "project" | "result">),
        isDirty: false,
        isCalculating: false,
        error: null,
      }),
    },
  ),
);
