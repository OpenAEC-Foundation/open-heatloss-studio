import { useCallback, useMemo, useRef, useState } from "react";

import { useClickOutside } from "../../hooks/useClickOutside";
import {
  CATALOGUE_CATEGORY_LABELS,
  getCatalogueByCategory,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../../lib/constructionCatalogue";

interface ConstructionPickerProps {
  onSelectCatalogue: (entry: CatalogueEntry) => void;
  onSelectBlank: () => void;
  onClose: () => void;
}

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "kozijnen_vullingen",
];

export function ConstructionPicker({
  onSelectCatalogue,
  onSelectBlank,
  onClose,
}: ConstructionPickerProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [search, setSearch] = useState("");

  useClickOutside(ref, onClose);

  const byCategory = useMemo(() => getCatalogueByCategory(), []);

  const filtered = useMemo(() => {
    if (!search.trim()) return byCategory;
    const q = search.toLowerCase();
    const result = new Map<CatalogueCategory, CatalogueEntry[]>();
    for (const [cat, entries] of byCategory) {
      const matches = entries.filter((e) => e.name.toLowerCase().includes(q));
      if (matches.length > 0) result.set(cat, matches);
    }
    return result;
  }, [byCategory, search]);

  const handleSelect = useCallback(
    (entry: CatalogueEntry) => {
      onSelectCatalogue(entry);
    },
    [onSelectCatalogue],
  );

  return (
    <div
      ref={ref}
      onClick={(e) => e.stopPropagation()}
      className="absolute left-0 z-20 mt-1 w-72 rounded-lg border border-stone-200 bg-white shadow-lg"
    >
      {/* Search */}
      <div className="border-b border-stone-200 p-2">
        <input
          type="text"
          placeholder="Zoek constructie..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded border border-stone-300 px-2 py-1 text-sm focus:border-blue-400 focus:outline-none"
          autoFocus
        />
      </div>

      {/* Blank option */}
      <button
        type="button"
        onClick={onSelectBlank}
        className="w-full border-b border-stone-200 px-3 py-2 text-left text-sm text-stone-600 hover:bg-stone-50"
      >
        Leeg grensvlak
      </button>

      {/* Category groups */}
      <div className="max-h-64 overflow-y-auto">
        {CATEGORY_ORDER.map((cat) => {
          const entries = filtered.get(cat);
          if (!entries) return null;
          return (
            <div key={cat}>
              <div className="sticky top-0 bg-stone-100 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-stone-500">
                {CATALOGUE_CATEGORY_LABELS[cat]}
              </div>
              {entries.map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  onClick={() => handleSelect(entry)}
                  className="flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-blue-50"
                >
                  <span className="text-stone-700">{entry.name}</span>
                  <span className="ml-2 tabular-nums text-stone-400">
                    {entry.uValue.toFixed(2)}
                  </span>
                </button>
              ))}
            </div>
          );
        })}
        {filtered.size === 0 && (
          <div className="px-3 py-3 text-center text-sm text-stone-400">
            Geen resultaten
          </div>
        )}
      </div>
    </div>
  );
}
