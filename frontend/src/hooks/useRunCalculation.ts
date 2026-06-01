import { useCallback } from "react";

import { useBackend } from "./useBackend";
import { useProjectStore } from "../store/projectStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import { prepareProjectForCalculation } from "../lib/frameOverride";
import { buildV2PayloadIsso53 } from "../lib/projectV2Migration";

/**
 * Gedeelde, norm-aware calc-dispatch voor alle "Berekenen"-ingangen.
 *
 * Routeert op `norm`: ISSO 53 gaat via de V2-payload naar `calculateV2`
 * (de isso51-kern crasht op de camelCase verwarmingssysteem-enum van
 * ISSO 53), alle overige normen via `calculate`. De hook beheert
 * `setCalculating`/`setResult`/`setError`; navigatie en toasts blijven
 * per caller.
 *
 * @returns `runCalculation()` — resolved met `true` bij succes, `false`
 *   bij fout (de foutmelding staat dan in de store via `setError`).
 */
export function useRunCalculation(): () => Promise<boolean> {
  const backend = useBackend();
  const {
    project,
    norm,
    sharedExtra,
    isso53Building,
    isso53Rooms,
    setCalculating,
    setResult,
    setError,
  } = useProjectStore();
  const projectConstructions = useModellerStore((s) => s.projectConstructions);

  return useCallback(async () => {
    setCalculating(true);
    try {
      if (norm === "isso53") {
        const payload = buildV2PayloadIsso53(
          project,
          sharedExtra,
          isso53Building,
          isso53Rooms,
        );
        const result = await backend.calculateV2(payload);
        setResult(result);
      } else {
        const payload = prepareProjectForCalculation(project, projectConstructions);
        const result = await backend.calculate(payload);
        setResult(result);
      }
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Berekening mislukt");
      return false;
    }
  }, [
    backend,
    norm,
    project,
    sharedExtra,
    isso53Building,
    isso53Rooms,
    projectConstructions,
    setCalculating,
    setResult,
    setError,
  ]);
}
