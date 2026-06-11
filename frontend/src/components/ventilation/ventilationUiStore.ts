/**
 * UI-voorkeuren voor de ventilatiebalans — losgekoppeld van projectdata.
 *
 * Bevat uitsluitend **weergave**-instellingen (zoals de debiet-eenheid
 * dm³/s ↔ m³/h); de projectstore en alle berekeningen blijven in dm³/s
 * (zie header `types/ventilation.ts`). Persistent via localStorage volgens
 * hetzelfde zustand-persist-patroon als `store/reportStore.ts`
 * (`ohs-report-options`) en `components/modeller/modellerStore.ts`.
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { FlowDisplayUnit } from "../../types/ventilation";

interface VentilationUiStore {
  /** Weergave-eenheid voor alle debiet-velden op de ventilatie-tab. */
  flowUnit: FlowDisplayUnit;
  setFlowUnit: (unit: FlowDisplayUnit) => void;
}

export const useVentilationUiStore = create<VentilationUiStore>()(
  persist(
    (set) => ({
      flowUnit: "dm3s",
      setFlowUnit: (flowUnit) => set({ flowUnit }),
    }),
    {
      name: "ohs-ventilation-ui",
      version: 1,
    },
  ),
);
