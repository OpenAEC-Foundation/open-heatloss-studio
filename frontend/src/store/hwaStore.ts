/**
 * HWA (hemelwaterafvoer) — invoerstate voor de dakafvoer-dimensionering.
 *
 * Losse tool (route `/tools/hwa`), zelfde state-model als de deurspleet-
 * calculator qua onafhankelijkheid van het project, maar hier wél
 * persisted in localStorage (meerdere dakvlakken invoeren is te veel werk
 * om bij een refresh kwijt te raken — vgl. `recentFilesStore.ts`). De
 * rekenkern zelf (`lib/hwaCalculation.ts`) blijft state-loos; deze store
 * bewaart alleen de invoer (`HwaInput`).
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

import { DEFAULT_RAIN_INTENSITY_LP_MIN_M2 } from "../lib/hwaCalculation";
import type {
  HwaInput,
  HwaRoofSurface,
  HwaSystemMode,
} from "../types/hwa";

function makeSurface(index: number): HwaRoofSurface {
  return {
    id: crypto.randomUUID(),
    name: `Dakvlak ${index}`,
    areaInputMode: "lxb",
    lengthM: undefined,
    widthM: undefined,
    areaM2: undefined,
    pitchDeg: 0,
    flatRoofFinish: null,
    facadeContributionM2: 0,
    downpipeCount: 1,
  };
}

const INITIAL_INPUT: HwaInput = {
  surfaces: [],
  rainIntensityLpMinM2: DEFAULT_RAIN_INTENSITY_LP_MIN_M2.value,
  systemMode: "traditioneel",
  uvSystemCapacityLpMin: undefined,
};

interface HwaStore extends HwaInput {
  /** Voeg een nieuw dakvlak toe met defaults en een oplopende naam. */
  addSurface: () => void;
  /** Wijzig een bestaand dakvlak (partial update). */
  updateSurface: (id: string, partial: Partial<HwaRoofSurface>) => void;
  /** Verwijder een dakvlak. */
  removeSurface: (id: string) => void;
  setRainIntensity: (value: number) => void;
  setSystemMode: (mode: HwaSystemMode) => void;
  setUvSystemCapacity: (value: number | undefined) => void;
  /** Zet alle invoer terug naar de defaults (geen dakvlakken). */
  reset: () => void;
}

const STORAGE_KEY = "ohs-hwa";

export const useHwaStore = create<HwaStore>()(
  persist(
    (set) => ({
      ...INITIAL_INPUT,

      addSurface: () =>
        set((state) => ({
          surfaces: [...state.surfaces, makeSurface(state.surfaces.length + 1)],
        })),

      updateSurface: (id, partial) =>
        set((state) => ({
          surfaces: state.surfaces.map((s) =>
            s.id === id ? { ...s, ...partial } : s,
          ),
        })),

      removeSurface: (id) =>
        set((state) => ({
          surfaces: state.surfaces.filter((s) => s.id !== id),
        })),

      setRainIntensity: (value) => set({ rainIntensityLpMinM2: value }),

      setSystemMode: (mode) => set({ systemMode: mode }),

      setUvSystemCapacity: (value) => set({ uvSystemCapacityLpMin: value }),

      reset: () => set({ ...INITIAL_INPUT, surfaces: [] }),
    }),
    {
      name: STORAGE_KEY,
      version: 1,
    },
  ),
);
