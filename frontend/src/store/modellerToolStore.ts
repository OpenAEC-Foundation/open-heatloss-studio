import { create } from "zustand";

import type {
  ModellerTool,
  ViewMode,
  SnapSettings,
  SnapMode,
} from "../components/modeller/types";
import { DEFAULT_SNAP_SETTINGS } from "../components/modeller/types";

/**
 * Orientation of the imported model in the viewer.
 * - "north": rooms rotated north-up via true_north_deg (default, current behaviour)
 * - "orthogonal": axis-aligned, model shown straight (true_north_deg treated as 0)
 */
export type OrientationMode = "north" | "orthogonal";

interface ModellerToolStore {
  tool: ModellerTool;
  viewMode: ViewMode;
  orientationMode: OrientationMode;
  activeFloor: number;
  snap: SnapSettings;

  setTool: (tool: ModellerTool) => void;
  setViewMode: (mode: ViewMode) => void;
  setOrientationMode: (mode: OrientationMode) => void;
  setActiveFloor: (floor: number) => void;
  setSnap: (snap: SnapSettings) => void;
  toggleSnapMode: (mode: SnapMode) => void;
  toggleSnapEnabled: () => void;
}

export const useModellerToolStore = create<ModellerToolStore>()((set) => ({
  tool: "select",
  viewMode: "2d",
  orientationMode: "north",
  activeFloor: 0,
  snap: DEFAULT_SNAP_SETTINGS,

  setTool: (tool) => set({ tool }),
  setViewMode: (mode) => set({ viewMode: mode }),
  setOrientationMode: (mode) => set({ orientationMode: mode }),
  setActiveFloor: (floor) => set({ activeFloor: floor }),
  setSnap: (snap) => set({ snap }),

  toggleSnapMode: (mode) =>
    set((state) => {
      const modes = state.snap.modes.includes(mode)
        ? state.snap.modes.filter((m) => m !== mode)
        : [...state.snap.modes, mode];
      return { snap: { ...state.snap, modes } };
    }),

  toggleSnapEnabled: () =>
    set((state) => ({
      snap: { ...state.snap, enabled: !state.snap.enabled },
    })),
}));
