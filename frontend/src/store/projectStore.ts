import { create } from "zustand";

import type { Project, ProjectResult } from "../types";

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

  /** Update project data (partial merge). */
  updateProject: (partial: Partial<Project>) => void;
  /** Replace the entire project. */
  setProject: (project: Project) => void;
  /** Set the calculation result. */
  setResult: (result: ProjectResult) => void;
  /** Set an error from a failed calculation. */
  setError: (error: string) => void;
  /** Set calculating state. */
  setCalculating: (isCalculating: boolean) => void;
  /** Reset to default state. */
  reset: () => void;
}

export const useProjectStore = create<ProjectStore>((set) => ({
  project: DEFAULT_PROJECT,
  result: null,
  error: null,
  isCalculating: false,
  isDirty: true,

  updateProject: (partial) =>
    set((state) => ({
      project: { ...state.project, ...partial },
      isDirty: true,
      error: null,
    })),

  setProject: (project) =>
    set({ project, isDirty: true, result: null, error: null }),

  setResult: (result) =>
    set({ result, isDirty: false, error: null, isCalculating: false }),

  setError: (error) =>
    set({ error, isCalculating: false }),

  setCalculating: (isCalculating) =>
    set({ isCalculating }),

  reset: () =>
    set({
      project: DEFAULT_PROJECT,
      result: null,
      error: null,
      isCalculating: false,
      isDirty: true,
    }),
}));
