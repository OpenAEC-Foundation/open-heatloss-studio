/**
 * Project construction library panel — shows per-project constructions
 * (copies from catalogue or custom creations) with calculated U-values.
 */
import { useState } from "react";

import { useModellerStore } from "./modellerStore";
import type { ProjectConstruction } from "./types";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
} from "../../lib/constructionCatalogue";
import { calculateRc } from "../../lib/rcCalculation";

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

export function ProjectLibraryPanel() {
  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );
  const removeProjectConstruction = useModellerStore(
    (s) => s.removeProjectConstruction,
  );

  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  if (projectConstructions.length === 0) {
    return (
      <div className="px-3 py-3">
        <p className="text-[10px] text-stone-400">
          Nog geen constructies in dit project. Voeg ze toe vanuit de
          Bibliotheek, importeer via IFC, of maak een nieuwe aan via de
          Rc-calculator.
        </p>
      </div>
    );
  }

  // Group by category
  const grouped = new Map<CatalogueCategory, ProjectConstruction[]>();
  for (const pc of projectConstructions) {
    const list = grouped.get(pc.category) ?? [];
    list.push(pc);
    grouped.set(pc.category, list);
  }

  return (
    <div className="px-3 py-2">
      {CATEGORY_ORDER.map((cat) => {
        const entries = grouped.get(cat);
        if (!entries?.length) return null;

        return (
          <div key={cat} className="mb-3">
            <div className="mb-1 text-[10px] font-medium text-stone-400">
              {CATALOGUE_CATEGORY_LABELS[cat]}
            </div>
            <div className="space-y-1">
              {entries.map((pc) => {
                const rcResult = calculateRc(
                  pc.layers,
                  pc.verticalPosition,
                );
                const uValue =
                  Math.round(rcResult.uValue * 1000) / 1000;

                return (
                  <div
                    key={pc.id}
                    className="rounded border border-stone-100 px-2 py-1.5"
                  >
                    <div className="flex items-start justify-between gap-1">
                      <div className="min-w-0 flex-1">
                        <div className="truncate text-[10px] font-medium text-stone-700">
                          {pc.name}
                        </div>
                        <div className="mt-0.5 text-[10px] text-stone-500">
                          U = {uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                          {" \u2022 "}
                          {pc.layers.length} lagen
                        </div>
                        {pc.catalogueSourceId && (
                          <div className="mt-0.5 text-[9px] text-stone-400">
                            Bron: standaard catalogus
                          </div>
                        )}
                        {pc.ifcSource && (
                          <div className="mt-0.5 text-[9px] text-stone-400">
                            IFC: {pc.ifcSource.wallTypeName}
                          </div>
                        )}
                      </div>
                      <div className="flex shrink-0 gap-0.5">
                        {confirmDelete === pc.id ? (
                          <>
                            <button
                              onClick={() => {
                                removeProjectConstruction(pc.id);
                                setConfirmDelete(null);
                              }}
                              className="rounded px-1 py-0.5 text-[9px] text-red-600 hover:bg-red-50"
                            >
                              Bevestig
                            </button>
                            <button
                              onClick={() => setConfirmDelete(null)}
                              className="rounded px-1 py-0.5 text-[9px] text-stone-400 hover:bg-stone-50"
                            >
                              Annuleer
                            </button>
                          </>
                        ) : (
                          <button
                            onClick={() => setConfirmDelete(pc.id)}
                            className="rounded p-0.5 text-stone-400 hover:bg-red-50 hover:text-red-600"
                            title="Verwijderen"
                          >
                            <svg
                              className="h-3 w-3"
                              viewBox="0 0 20 20"
                              fill="currentColor"
                            >
                              <path
                                fillRule="evenodd"
                                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                clipRule="evenodd"
                              />
                            </svg>
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
