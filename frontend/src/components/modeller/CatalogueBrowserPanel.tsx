/**
 * Catalogue browser panel — browse the standard construction catalogue.
 * Read-only view with "Toevoegen aan project" action per entry.
 */
import { useState } from "react";

import { useCatalogueStore } from "../../store/catalogueStore";
import { useModellerStore } from "./modellerStore";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
} from "../../lib/constructionCatalogue";

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

export function CatalogueBrowserPanel() {
  const entries = useCatalogueStore((s) => s.entries);
  const projectConstructions = useModellerStore(
    (s) => s.projectConstructions,
  );
  const copyFromCatalogue = useModellerStore((s) => s.copyFromCatalogue);
  const [search, setSearch] = useState("");

  // Group by category
  const byCategory = new Map<CatalogueCategory, typeof entries>();
  for (const entry of entries) {
    if (search && !entry.name.toLowerCase().includes(search.toLowerCase())) {
      continue;
    }
    const list = byCategory.get(entry.category) ?? [];
    list.push(entry);
    byCategory.set(entry.category, list);
  }

  /** Check if entry is already copied into the project. */
  const isInProject = (entryId: string): boolean =>
    projectConstructions.some((c) => c.catalogueSourceId === entryId);

  return (
    <div className="flex h-full flex-col">
      {/* Search */}
      <div className="border-b border-stone-100 px-3 py-2">
        <input
          placeholder="Zoeken..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded border border-stone-200 bg-white px-2 py-1 text-[11px] outline-none focus:border-amber-400"
        />
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto px-3 py-2">
        {CATEGORY_ORDER.map((cat) => {
          const catEntries = byCategory.get(cat);
          if (!catEntries?.length) return null;

          return (
            <div key={cat} className="mb-3">
              <div className="mb-1 text-[10px] font-medium text-stone-400">
                {CATALOGUE_CATEGORY_LABELS[cat]}
              </div>
              <div className="space-y-1">
                {catEntries.map((entry) => {
                  const inProject = isInProject(entry.id);
                  const hasLayers = !!entry.layers?.length;

                  return (
                    <div
                      key={entry.id}
                      className="rounded border border-stone-100 px-2 py-1.5"
                    >
                      <div className="flex items-start justify-between gap-1">
                        <div className="min-w-0 flex-1">
                          <div className="truncate text-[10px] font-medium text-stone-700">
                            {entry.name}
                          </div>
                          <div className="mt-0.5 text-[10px] text-stone-500">
                            U = {entry.uValue} W/(m{"\u00B2"}{"\u00B7"}K)
                            {entry.layers && (
                              <>
                                {" \u2022 "}
                                {entry.layers.length} lagen
                              </>
                            )}
                          </div>
                        </div>
                        <div className="shrink-0">
                          {inProject ? (
                            <span className="rounded bg-teal-50 px-1.5 py-0.5 text-[9px] font-medium text-teal-600">
                              In project
                            </span>
                          ) : hasLayers ? (
                            <button
                              onClick={() => copyFromCatalogue(entry)}
                              className="rounded bg-amber-50 px-1.5 py-0.5 text-[9px] font-medium text-amber-700 hover:bg-amber-100"
                            >
                              Toevoegen
                            </button>
                          ) : null}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}

        {byCategory.size === 0 && (
          <p className="text-[10px] text-stone-400">Geen resultaten.</p>
        )}
      </div>
    </div>
  );
}
