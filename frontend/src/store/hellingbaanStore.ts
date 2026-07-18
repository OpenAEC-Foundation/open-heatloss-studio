/**
 * Hellingbaan (parkeergarage) — invoerstate voor de NEN 2443-dimensionering.
 *
 * Losse tool (route `/tools/hellingbaan`), zelfde patroon als
 * `store/hwaStore.ts`: persisted in localStorage (invoer is te veel werk
 * om bij een refresh kwijt te raken), de rekenkern zelf
 * (`lib/hellingbaanCalculation.ts`) blijft state-loos.
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

import { getGarageType } from "../lib/hellingbaanCalculation";
import type { HellingbaanGarageTypeId, HellingbaanInput } from "../types/hellingbaan";

const INITIAL_INPUT: HellingbaanInput = {
  hoogteMm: 3600,
  garageTypeId: "stalling",
  metOvergang: true,
  breedteMm: 2750,
  hellingOverridePercent: undefined,
};

interface HellingbaanStore extends HellingbaanInput {
  setHoogteMm: (value: number) => void;
  setGarageTypeId: (id: HellingbaanGarageTypeId) => void;
  setMetOvergang: (value: boolean) => void;
  setBreedteMm: (value: number) => void;
  setHellingOverridePercent: (value: number | undefined) => void;
  /** Zet alle invoer terug naar de defaults (3600 mm, stalling). */
  reset: () => void;
}

const STORAGE_KEY = "ohs-hellingbaan";

export const useHellingbaanStore = create<HellingbaanStore>()(
  persist(
    (set) => ({
      ...INITIAL_INPUT,

      setHoogteMm: (value) => set({ hoogteMm: value }),

      setGarageTypeId: (id) =>
        set({
          garageTypeId: id,
          // Override + breedte vallen terug op de defaults van het nieuwe
          // type, zelfde gedrag als de pyRevit-UI (`_on_garage_changed`
          // zet `helling_override`/`breedte_override` terug op `None`,
          // waarna `_get_breedte()` de type-minimumbreedte teruggeeft).
          hellingOverridePercent: undefined,
          breedteMm: getGarageType(id).breedteMinMm,
        }),

      setMetOvergang: (value) => set({ metOvergang: value }),

      setBreedteMm: (value) => set({ breedteMm: value }),

      setHellingOverridePercent: (value) => set({ hellingOverridePercent: value }),

      reset: () => set({ ...INITIAL_INPUT }),
    }),
    {
      name: STORAGE_KEY,
      version: 1,
    },
  ),
);
