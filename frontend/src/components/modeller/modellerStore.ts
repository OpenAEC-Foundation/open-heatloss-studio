/**
 * Zustand store for modeller model data.
 *
 * Manages rooms, windows, doors, underlay, construction assignments,
 * and undo/redo history.
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { ModelRoom, ModelWindow, ModelDoor, WallBoundaryType, ProjectConstruction } from "./types";
import type { CatalogueEntry } from "../../lib/constructionCatalogue";
import { EXAMPLE_ROOMS, EXAMPLE_WINDOWS } from "./exampleData";

// ---------------------------------------------------------------------------
// Underlay
// ---------------------------------------------------------------------------

export interface UnderlayImage {
  dataUrl: string;
  fileName: string;
  /** Position in mm (top-left corner). */
  x: number;
  y: number;
  /** Display size in mm. */
  width: number;
  height: number;
  opacity: number;
  rotation: number;
  locked: boolean;
}

// ---------------------------------------------------------------------------
// History snapshot (for undo/redo)
// ---------------------------------------------------------------------------

interface Snapshot {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  projectConstructions: ProjectConstruction[];
}

function takeSnapshot(state: {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  projectConstructions: ProjectConstruction[];
}): Snapshot {
  return {
    rooms: structuredClone(state.rooms),
    windows: structuredClone(state.windows),
    doors: structuredClone(state.doors),
    projectConstructions: structuredClone(state.projectConstructions),
  };
}

// ---------------------------------------------------------------------------
// Store interface
// ---------------------------------------------------------------------------

const MAX_HISTORY = 50;

interface ModellerStore {
  // Model data
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];

  // Project constructions (per-project layer-based constructions)
  projectConstructions: ProjectConstruction[];

  // Underlay
  underlay: UnderlayImage | null;

  // Construction assignments: "roomId:wallIndex" -> catalogueEntryId or projectConstructionId
  wallConstructions: Record<string, string>;
  floorConstructions: Record<string, string>;
  roofConstructions: Record<string, string>;

  // Wall boundary type overrides: "roomId:wallIndex" -> WallBoundaryType
  wallBoundaryTypes: Record<string, WallBoundaryType>;

  // History (not persisted)
  _past: Snapshot[];
  _future: Snapshot[];

  // Room CRUD
  addRoom: (room: Omit<ModelRoom, "id">) => string;
  updateRoom: (id: string, updates: Partial<Omit<ModelRoom, "id">>) => void;
  removeRoom: (id: string) => void;

  // Window CRUD
  addWindow: (win: ModelWindow) => void;
  updateWindow: (roomId: string, wallIndex: number, offset: number, updates: Partial<ModelWindow>) => void;
  removeWindow: (roomId: string, wallIndex: number, offset: number) => void;

  // Door CRUD
  addDoor: (door: ModelDoor) => void;
  removeDoor: (roomId: string, wallIndex: number, offset: number) => void;

  // Underlay
  setUnderlay: (underlay: UnderlayImage | null) => void;
  updateUnderlay: (updates: Partial<UnderlayImage>) => void;

  // Construction assignment
  assignWallConstruction: (roomId: string, wallIndex: number, entryId: string | null) => void;
  assignFloorConstruction: (roomId: string, entryId: string | null) => void;
  assignRoofConstruction: (roomId: string, entryId: string | null) => void;

  // Wall boundary type
  assignWallBoundaryType: (roomId: string, wallIndex: number, boundaryType: WallBoundaryType) => void;

  // Project construction CRUD
  addProjectConstruction: (construction: Omit<ProjectConstruction, "id">) => string;
  updateProjectConstruction: (id: string, updates: Partial<Omit<ProjectConstruction, "id">>) => void;
  removeProjectConstruction: (id: string) => void;
  importProjectConstructions: (constructions: Omit<ProjectConstruction, "id">[]) => void;

  /**
   * Copy a catalogue entry into the project library.
   * If already copied (same catalogueSourceId), returns existing project ID.
   * The entry must have layers — entries without layers (e.g. glazing) are
   * assigned directly without copying.
   */
  copyFromCatalogue: (entry: CatalogueEntry) => string;

  // History
  undo: () => void;
  redo: () => void;

  // Bulk import (replaces all model data in one undo step)
  importModel: (rooms: ModelRoom[], windows?: ModelWindow[], doors?: ModelDoor[]) => void;

  // Utility
  nextRoomId: (floor: number) => string;
  resetToExample: () => void;
}

/** Generate next room ID like "0.07" for the given floor. */
function generateNextRoomId(rooms: ModelRoom[], floor: number): string {
  const prefix = `${floor}.`;
  let maxNum = 0;
  for (const r of rooms) {
    if (r.id.startsWith(prefix)) {
      const num = parseInt(r.id.slice(prefix.length), 10);
      if (!isNaN(num) && num > maxNum) maxNum = num;
    }
  }
  return `${floor}.${String(maxNum + 1).padStart(2, "0")}`;
}

/** Push current state onto undo stack (call before mutations). */
function pushUndo(state: ModellerStore): Pick<ModellerStore, "_past" | "_future"> {
  const snap = takeSnapshot(state);
  const past = [...state._past, snap];
  if (past.length > MAX_HISTORY) past.shift();
  return { _past: past, _future: [] };
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

export const useModellerStore = create<ModellerStore>()(
  persist(
    (set, get) => ({
      rooms: [...EXAMPLE_ROOMS],
      windows: [...EXAMPLE_WINDOWS],
      doors: [],
      projectConstructions: [],
      underlay: null,
      wallConstructions: {},
      floorConstructions: {},
      roofConstructions: {},
      wallBoundaryTypes: {},
      _past: [],
      _future: [],

      // -- Room CRUD --
      addRoom: (room) => {
        const state = get();
        const id = generateNextRoomId(state.rooms, room.floor);
        const newRoom: ModelRoom = { ...room, id };
        set({
          ...pushUndo(state),
          rooms: [...state.rooms, newRoom],
        });
        return id;
      },

      updateRoom: (id, updates) => {
        const state = get();
        set({
          ...pushUndo(state),
          rooms: state.rooms.map((r) => (r.id === id ? { ...r, ...updates } : r)),
        });
      },

      removeRoom: (id) => {
        const state = get();
        set({
          ...pushUndo(state),
          rooms: state.rooms.filter((r) => r.id !== id),
          windows: state.windows.filter((w) => w.roomId !== id),
          doors: state.doors.filter((d) => d.roomId !== id),
        });
      },

      // -- Window CRUD --
      addWindow: (win) => {
        const state = get();
        set({
          ...pushUndo(state),
          windows: [...state.windows, win],
        });
      },

      updateWindow: (roomId, wallIndex, offset, updates) => {
        const state = get();
        set({
          ...pushUndo(state),
          windows: state.windows.map((w) =>
            w.roomId === roomId && w.wallIndex === wallIndex && Math.abs(w.offset - offset) < 1
              ? { ...w, ...updates }
              : w,
          ),
        });
      },

      removeWindow: (roomId, wallIndex, offset) => {
        const state = get();
        set({
          ...pushUndo(state),
          windows: state.windows.filter(
            (w) => !(w.roomId === roomId && w.wallIndex === wallIndex && Math.abs(w.offset - offset) < 1),
          ),
        });
      },

      // -- Door CRUD --
      addDoor: (door) => {
        const state = get();
        set({
          ...pushUndo(state),
          doors: [...state.doors, door],
        });
      },

      removeDoor: (roomId, wallIndex, offset) => {
        const state = get();
        set({
          ...pushUndo(state),
          doors: state.doors.filter(
            (d) => !(d.roomId === roomId && d.wallIndex === wallIndex && Math.abs(d.offset - offset) < 1),
          ),
        });
      },

      // -- Underlay --
      setUnderlay: (underlay) => set({ underlay }),
      updateUnderlay: (updates) => {
        const state = get();
        if (!state.underlay) return;
        set({ underlay: { ...state.underlay, ...updates } });
      },

      // -- Construction assignment --
      assignWallConstruction: (roomId, wallIndex, entryId) => {
        const key = `${roomId}:${wallIndex}`;
        set((state) => {
          const next = { ...state.wallConstructions };
          if (entryId) next[key] = entryId;
          else delete next[key];
          return { wallConstructions: next };
        });
      },
      assignFloorConstruction: (roomId, entryId) => {
        set((state) => {
          const next = { ...state.floorConstructions };
          if (entryId) next[roomId] = entryId;
          else delete next[roomId];
          return { floorConstructions: next };
        });
      },
      assignRoofConstruction: (roomId, entryId) => {
        set((state) => {
          const next = { ...state.roofConstructions };
          if (entryId) next[roomId] = entryId;
          else delete next[roomId];
          return { roofConstructions: next };
        });
      },
      // -- Wall boundary type --
      assignWallBoundaryType: (roomId, wallIndex, boundaryType) => {
        const key = `${roomId}:${wallIndex}`;
        set((state) => {
          const next = { ...state.wallBoundaryTypes };
          if (boundaryType === "auto") delete next[key];
          else next[key] = boundaryType;
          return { wallBoundaryTypes: next };
        });
      },

      // -- Project construction CRUD --
      addProjectConstruction: (construction) => {
        const state = get();
        const id = `proj-${crypto.randomUUID()}`;
        set({
          ...pushUndo(state),
          projectConstructions: [
            ...state.projectConstructions,
            { ...construction, id },
          ],
        });
        return id;
      },

      updateProjectConstruction: (id, updates) => {
        const state = get();
        set({
          ...pushUndo(state),
          projectConstructions: state.projectConstructions.map((c) =>
            c.id === id ? { ...c, ...updates } : c,
          ),
        });
      },

      removeProjectConstruction: (id) => {
        const state = get();
        // Clean up any assignment references pointing to this construction
        const cleanRecord = (rec: Record<string, string>) => {
          const next: Record<string, string> = {};
          for (const [k, v] of Object.entries(rec)) {
            if (v !== id) next[k] = v;
          }
          return next;
        };
        set({
          ...pushUndo(state),
          projectConstructions: state.projectConstructions.filter(
            (c) => c.id !== id,
          ),
          wallConstructions: cleanRecord(state.wallConstructions),
          floorConstructions: cleanRecord(state.floorConstructions),
          roofConstructions: cleanRecord(state.roofConstructions),
        });
      },

      copyFromCatalogue: (entry) => {
        const state = get();
        // Check if already copied
        const existing = state.projectConstructions.find(
          (c) => c.catalogueSourceId === entry.id,
        );
        if (existing) return existing.id;

        // Copy as new project construction
        const id = `proj-${crypto.randomUUID()}`;
        set({
          ...pushUndo(state),
          projectConstructions: [
            ...state.projectConstructions,
            {
              id,
              name: entry.name,
              category: entry.category,
              materialType: entry.materialType,
              verticalPosition: entry.verticalPosition,
              layers: structuredClone(entry.layers ?? []),
              catalogueSourceId: entry.id,
            },
          ],
        });
        return id;
      },

      importProjectConstructions: (constructions) => {
        const state = get();
        const newEntries = constructions.map((c) => ({
          ...c,
          id: `proj-${crypto.randomUUID()}`,
        }));
        set({
          ...pushUndo(state),
          projectConstructions: [
            ...state.projectConstructions,
            ...newEntries,
          ],
        });
      },

      // -- History --
      undo: () => {
        const state = get();
        if (state._past.length === 0) return;
        const snap = takeSnapshot(state);
        const prev = state._past[state._past.length - 1]!;
        set({
          rooms: prev.rooms,
          windows: prev.windows,
          doors: prev.doors,
          projectConstructions: prev.projectConstructions,
          _past: state._past.slice(0, -1),
          _future: [...state._future, snap],
        });
      },

      redo: () => {
        const state = get();
        if (state._future.length === 0) return;
        const snap = takeSnapshot(state);
        const next = state._future[state._future.length - 1]!;
        set({
          rooms: next.rooms,
          windows: next.windows,
          doors: next.doors,
          projectConstructions: next.projectConstructions,
          _past: [...state._past, snap],
          _future: state._future.slice(0, -1),
        });
      },

      // -- Bulk import --
      importModel: (rooms, windows = [], doors = []) => {
        const state = get();
        set({
          ...pushUndo(state),
          rooms,
          windows,
          doors,
          wallConstructions: {},
          floorConstructions: {},
          roofConstructions: {},
          wallBoundaryTypes: {},
          // Note: projectConstructions are intentionally preserved during model import
        });
      },

      // -- Utility --
      nextRoomId: (floor) => generateNextRoomId(get().rooms, floor),

      resetToExample: () =>
        set({
          rooms: [...EXAMPLE_ROOMS],
          windows: [...EXAMPLE_WINDOWS],
          doors: [],
          projectConstructions: [],
          wallConstructions: {},
          floorConstructions: {},
          roofConstructions: {},
          wallBoundaryTypes: {},
          _past: [],
          _future: [],
        }),
    }),
    {
      name: "isso51-modeller",
      version: 1,
      partialize: (state) => ({
        rooms: state.rooms,
        windows: state.windows,
        doors: state.doors,
        projectConstructions: state.projectConstructions,
        underlay: state.underlay,
        wallConstructions: state.wallConstructions,
        floorConstructions: state.floorConstructions,
        roofConstructions: state.roofConstructions,
        wallBoundaryTypes: state.wallBoundaryTypes,
      }),
    },
  ),
);
