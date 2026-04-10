/**
 * Combines global catalogue entries with project-specific constructions.
 *
 * Project constructions appear first with `isProjectEntry: true`.
 * U-values for project entries are computed on-the-fly from layers.
 */
import { useMemo } from "react";

import { useCatalogueStore } from "../store/catalogueStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import { calculateRc } from "../lib/rcCalculation";
import type { CatalogueEntry } from "../lib/constructionCatalogue";

export interface UnifiedConstructionEntry extends CatalogueEntry {
  isProjectEntry?: boolean;
}

export function useAllConstructions(): UnifiedConstructionEntry[] {
  const catalogueEntries = useCatalogueStore((s) => s.entries);
  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );

  return useMemo(() => {
    const projectEntries: UnifiedConstructionEntry[] =
      projectConstructions.map((pc) => {
        // Voor entries zonder lagen (kozijnen/vullingen) gebruiken we de
        // directe `pc.uValue` in plaats van calculateRc (die zou met lege
        // layers R = Rsi + Rse = 0.17 opleveren, ergo U ≈ 5.88).
        const uValue =
          pc.layers.length > 0
            ? Math.round(calculateRc(pc.layers, pc.verticalPosition).uValue * 1000) / 1000
            : pc.uValue ?? 0;
        return {
          id: pc.id,
          name: pc.name,
          category: pc.category,
          uValue,
          materialType: pc.materialType,
          verticalPosition: pc.verticalPosition,
          layers: pc.layers,
          isProjectEntry: true,
        };
      });

    return [...projectEntries, ...catalogueEntries];
  }, [catalogueEntries, projectConstructions]);
}
