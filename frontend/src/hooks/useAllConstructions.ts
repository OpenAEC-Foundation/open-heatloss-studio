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
        const rcResult = calculateRc(pc.layers, pc.verticalPosition);
        return {
          id: pc.id,
          name: pc.name,
          category: pc.category,
          uValue: Math.round(rcResult.uValue * 1000) / 1000,
          materialType: pc.materialType,
          verticalPosition: pc.verticalPosition,
          layers: pc.layers,
          isProjectEntry: true,
        };
      });

    return [...projectEntries, ...catalogueEntries];
  }, [catalogueEntries, projectConstructions]);
}
