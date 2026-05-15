/**
 * Documents store — multi-tab project state.
 *
 * Houdt een lijst van geopende projecten als tabs. Elke tab heeft een
 * snapshot van projectStore + modellerStore state. Bij tab-switch wordt
 * de huidige snapshot bijgewerkt en de nieuwe ingeladen.
 *
 * MVP scope:
 * - Snapshots: project + result + currentLocalPath + isDirty + sharedExtra
 *   (projectStore), + alle modellerStore data-velden
 * - Undo/redo history wordt NIET per-tab bewaard (zou complex zijn).
 *   Bij tab-switch begint de history van die tab opnieuw.
 * - Modeller-tool UI state (huidige tool, view-mode) is global → wordt
 *   niet gereset bij tab-switch
 *
 * Persist: tabs + snapshots worden bewaard in localStorage onder
 * "ohs-documents", dus de geopende set tabs overleeft een herstart.
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

import type {
  ImportedBoundary,
  ModelDoor,
  ModelRoom,
  ModelWindow,
  ProjectConstruction,
  WallBoundaryType,
} from "../components/modeller/types";
import type { UnderlayImage } from "../components/modeller/modellerStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import { useProjectStore } from "./projectStore";
import type { Project, ProjectResult } from "../types";
import type { SharedExtra } from "../types/projectV2";

export interface TabInfo {
  id: string;
  /** Display name in the tab (project name, of "Naamloos N"). */
  name: string;
  /** Dirty flag — wordt bij elke tab-snapshot bijgewerkt. */
  isDirty: boolean;
}

interface ProjectSnapshot {
  project: Project;
  result: ProjectResult | null;
  currentLocalPath: string | null;
  isDirty: boolean;
  sharedExtra: SharedExtra | null;
}

interface ModellerSnapshot {
  rooms: ModelRoom[];
  windows: ModelWindow[];
  doors: ModelDoor[];
  projectConstructions: ProjectConstruction[];
  underlay: UnderlayImage | null;
  wallConstructions: Record<string, string>;
  floorConstructions: Record<string, string>;
  roofConstructions: Record<string, string>;
  wallBoundaryTypes: Record<string, WallBoundaryType>;
  importedBoundaries: ImportedBoundary[];
}

interface DocumentSnapshot {
  project: ProjectSnapshot;
  modeller: ModellerSnapshot;
}

interface DocumentsState {
  tabs: TabInfo[];
  snapshots: Record<string, DocumentSnapshot>;
  activeId: string | null;
  nextNamelessIndex: number;
}

interface DocumentsActions {
  /** Nieuwe lege tab. Vooraf wordt de huidige tab gesnapshot. */
  newTab: (name?: string) => string;
  /** Tab sluiten. Wanneer 't de actieve was wordt een buur de nieuwe actieve. */
  closeTab: (id: string) => void;
  /** Wisselen van tab. Huidige wordt gesnapshot, nieuwe ingeladen. */
  switchTab: (id: string) => void;
  /** Naam van de actieve tab updaten (bv. na project-info wijziging). */
  setActiveName: (name: string) => void;
  /** Dirty-flag van de actieve tab updaten. */
  setActiveDirty: (dirty: boolean) => void;
  /** Force-snapshot van de actieve tab (na save, of voor switch). */
  snapshotActive: () => void;
}

type DocumentsStore = DocumentsState & DocumentsActions;

function makeId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `doc-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

/** Pull current Zustand store states into a snapshot. */
function captureSnapshot(): DocumentSnapshot {
  const ps = useProjectStore.getState();
  const ms = useModellerStore.getState();
  return {
    project: {
      project: ps.project,
      result: ps.result,
      currentLocalPath: ps.currentLocalPath,
      isDirty: ps.isDirty,
      sharedExtra: ps.sharedExtra ?? null,
    },
    modeller: {
      rooms: ms.rooms,
      windows: ms.windows,
      doors: ms.doors,
      projectConstructions: ms.projectConstructions,
      underlay: ms.underlay,
      wallConstructions: ms.wallConstructions,
      floorConstructions: ms.floorConstructions,
      roofConstructions: ms.roofConstructions,
      wallBoundaryTypes: ms.wallBoundaryTypes,
      importedBoundaries: ms.importedBoundaries,
    },
  };
}

/** Push a snapshot into the Zustand stores. */
function loadSnapshot(snap: DocumentSnapshot): void {
  // Use Zustand's setState (bypasses our store's actions which would mark
  // dirty / add to history). We're restoring state, not editing it.
  useProjectStore.setState({
    project: snap.project.project,
    result: snap.project.result,
    currentLocalPath: snap.project.currentLocalPath,
    isDirty: snap.project.isDirty,
    sharedExtra: snap.project.sharedExtra ?? undefined,
    error: null,
    isCalculating: false,
    serverUpdatedAt: null,
    hasConflict: false,
    _past: [],
    _future: [],
  } as Partial<ReturnType<typeof useProjectStore.getState>>);

  useModellerStore.setState({
    rooms: snap.modeller.rooms,
    windows: snap.modeller.windows,
    doors: snap.modeller.doors,
    projectConstructions: snap.modeller.projectConstructions,
    underlay: snap.modeller.underlay,
    wallConstructions: snap.modeller.wallConstructions,
    floorConstructions: snap.modeller.floorConstructions,
    roofConstructions: snap.modeller.roofConstructions,
    wallBoundaryTypes: snap.modeller.wallBoundaryTypes,
    importedBoundaries: snap.modeller.importedBoundaries,
    _past: [],
    _future: [],
  } as Partial<ReturnType<typeof useModellerStore.getState>>);
}

export const useDocumentsStore = create<DocumentsStore>()(
  persist(
    (set, get) => ({
      tabs: [],
      snapshots: {},
      activeId: null,
      nextNamelessIndex: 1,

      newTab: (name) => {
        // Snapshot current first
        const state = get();
        if (state.activeId) {
          const snap = captureSnapshot();
          set({ snapshots: { ...state.snapshots, [state.activeId]: snap } });
        }
        // Create new
        const id = makeId();
        const tabName =
          name ??
          (() => {
            const ps = useProjectStore.getState();
            const projName = ps.project?.info?.name?.trim();
            return projName && projName.length > 0
              ? projName
              : `Naamloos ${state.nextNamelessIndex}`;
          })();
        const newTabInfo: TabInfo = { id, name: tabName, isDirty: false };
        // Reset stores naar lege staat als naam ontbrak (echte nieuwe tab)
        if (name === undefined) {
          useProjectStore.getState().reset();
        }
        const freshSnap = captureSnapshot();
        set({
          tabs: [...state.tabs, newTabInfo],
          snapshots: { ...state.snapshots, [id]: freshSnap },
          activeId: id,
          nextNamelessIndex:
            name === undefined
              ? state.nextNamelessIndex + 1
              : state.nextNamelessIndex,
        });
        return id;
      },

      closeTab: (id) => {
        const state = get();
        const idx = state.tabs.findIndex((t) => t.id === id);
        if (idx < 0) return;
        const newTabs = state.tabs.filter((t) => t.id !== id);
        const newSnapshots = { ...state.snapshots };
        delete newSnapshots[id];

        let newActive: string | null = state.activeId;
        if (state.activeId === id) {
          // Switch naar buur (prefer rechts, dan links)
          const neighbor = state.tabs[idx + 1] ?? state.tabs[idx - 1] ?? null;
          newActive = neighbor?.id ?? null;
          const neighborSnap = neighbor ? state.snapshots[neighbor.id] : undefined;
          if (neighborSnap) {
            loadSnapshot(neighborSnap);
          } else {
            useProjectStore.getState().reset();
          }
        }
        set({ tabs: newTabs, snapshots: newSnapshots, activeId: newActive });
      },

      switchTab: (id) => {
        const state = get();
        if (state.activeId === id) return;
        // Snapshot current
        if (state.activeId) {
          const snap = captureSnapshot();
          state.snapshots[state.activeId] = snap;
        }
        // Load new
        const target = state.snapshots[id];
        if (target) {
          loadSnapshot(target);
        }
        set({ activeId: id, snapshots: { ...state.snapshots } });
      },

      setActiveName: (name) => {
        const state = get();
        if (!state.activeId) return;
        set({
          tabs: state.tabs.map((t) =>
            t.id === state.activeId ? { ...t, name } : t,
          ),
        });
      },

      setActiveDirty: (dirty) => {
        const state = get();
        if (!state.activeId) return;
        set({
          tabs: state.tabs.map((t) =>
            t.id === state.activeId ? { ...t, isDirty: dirty } : t,
          ),
        });
      },

      snapshotActive: () => {
        const state = get();
        if (!state.activeId) return;
        const snap = captureSnapshot();
        set({
          snapshots: { ...state.snapshots, [state.activeId]: snap },
        });
      },
    }),
    {
      name: "ohs-documents",
      version: 1,
      // Persist alles behalve transient state. Snapshots kunnen groot
      // worden (rooms + constructions per tab), maar localStorage redt
      // 5MB makkelijk voor een handvol projecten.
      partialize: (state) => ({
        tabs: state.tabs,
        snapshots: state.snapshots,
        activeId: state.activeId,
        nextNamelessIndex: state.nextNamelessIndex,
      }),
    },
  ),
);
