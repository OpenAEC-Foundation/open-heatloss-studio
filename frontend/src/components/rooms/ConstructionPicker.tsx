import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { useClickOutside } from "../../hooks/useClickOutside";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
  type CatalogueEntry,
} from "../../lib/constructionCatalogue";
import { useCatalogueStore } from "../../store/catalogueStore";
import { useModellerStore } from "../modeller/modellerStore";
import type { ProjectConstruction } from "../modeller/types";
import { calculateRc } from "../../lib/rcCalculation";

interface ConstructionPickerProps {
  onSelectCatalogue: (entry: CatalogueEntry) => void;
  onSelectProject?: (pc: ProjectConstruction) => void;
  onSelectBlank: () => void;
  onClose: () => void;
  anchorRect: DOMRect | null;
}

const CATEGORY_ORDER: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
  "kozijnen_vullingen",
];

export function ConstructionPicker({
  onSelectCatalogue,
  onSelectProject,
  onSelectBlank,
  onClose,
  anchorRect,
}: ConstructionPickerProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [search, setSearch] = useState("");

  useClickOutside(ref, onClose);

  const catalogueEntries = useCatalogueStore((s) => s.entries);
  const projectConstructions = useModellerStore((s) => s.projectConstructions);

  // Group catalogue by category
  const byCategory = useMemo(() => {
    const map = new Map<CatalogueCategory, CatalogueEntry[]>();
    for (const entry of catalogueEntries) {
      const list = map.get(entry.category) ?? [];
      list.push(entry);
      map.set(entry.category, list);
    }
    return map;
  }, [catalogueEntries]);

  // Filter catalogue
  const filteredCatalogue = useMemo(() => {
    if (!search.trim()) return byCategory;
    const q = search.toLowerCase();
    const result = new Map<CatalogueCategory, CatalogueEntry[]>();
    for (const [cat, entries] of byCategory) {
      const matches = entries.filter((e) => e.name.toLowerCase().includes(q));
      if (matches.length > 0) result.set(cat, matches);
    }
    return result;
  }, [byCategory, search]);

  // Filter project constructions
  const filteredProject = useMemo(() => {
    if (!search.trim()) return projectConstructions;
    const q = search.toLowerCase();
    return projectConstructions.filter((pc) =>
      pc.name.toLowerCase().includes(q),
    );
  }, [projectConstructions, search]);

  const handleSelectCatalogue = useCallback(
    (entry: CatalogueEntry) => {
      onSelectCatalogue(entry);
    },
    [onSelectCatalogue],
  );

  const handleSelectProject = useCallback(
    (pc: ProjectConstruction) => {
      if (onSelectProject) {
        onSelectProject(pc);
      } else {
        // Fallback: convert to catalogue-like entry for the existing handler
        const rcResult = pc.layers.length > 0
          ? calculateRc(pc.layers, pc.verticalPosition)
          : null;
        const asCatalogue: CatalogueEntry = {
          id: pc.id,
          name: pc.name,
          category: pc.category,
          uValue: rcResult ? Math.round(rcResult.uValue * 1000) / 1000 : 0,
          materialType: pc.materialType,
          verticalPosition: pc.verticalPosition,
          layers: pc.layers,
        };
        onSelectCatalogue(asCatalogue);
      }
    },
    [onSelectProject, onSelectCatalogue],
  );

  // Position the dropdown below the anchor, flip up if near viewport bottom.
  const [pos, setPos] = useState({ top: 0, left: 0, flipUp: false });

  useEffect(() => {
    if (!anchorRect) return;
    const PICKER_HEIGHT = 400;
    const spaceBelow = window.innerHeight - anchorRect.bottom;
    const flipUp = spaceBelow < PICKER_HEIGHT && anchorRect.top > PICKER_HEIGHT;

    setPos({
      top: flipUp ? anchorRect.top : anchorRect.bottom + 4,
      left: anchorRect.left,
      flipUp,
    });
  }, [anchorRect]);

  if (!anchorRect) return null;

  const picker = (
    <div
      ref={ref}
      onClick={(e) => e.stopPropagation()}
      style={{
        position: "fixed",
        top: pos.flipUp ? undefined : pos.top,
        bottom: pos.flipUp ? window.innerHeight - pos.top + 4 : undefined,
        left: pos.left,
        zIndex: 50,
      }}
      className="w-80 rounded-lg border border-stone-200 bg-white shadow-xl"
    >
      {/* Search */}
      <div className="border-b border-stone-200 p-2">
        <input
          type="text"
          placeholder="Zoek constructie..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded border border-stone-300 px-2 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
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

      <div className="max-h-80 overflow-y-auto">
        {/* Project constructions section */}
        {filteredProject.length > 0 && (
          <>
            <div className="sticky top-0 bg-teal-50 px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-teal-700">
              Project constructies
            </div>
            {filteredProject.map((pc) => {
              const rcResult = pc.layers.length > 0
                ? calculateRc(pc.layers, pc.verticalPosition)
                : null;
              const uVal = rcResult
                ? Math.round(rcResult.uValue * 1000) / 1000
                : 0;

              return (
                <button
                  key={pc.id}
                  type="button"
                  onClick={() => handleSelectProject(pc)}
                  className="flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-teal-50"
                >
                  <span className="flex items-center gap-1.5 text-stone-700">
                    <span className="rounded bg-teal-100 px-1 py-0.5 text-[10px] text-teal-700">
                      P
                    </span>
                    {pc.name}
                    {pc.layers.length > 0 && (
                      <span className="rounded bg-blue-50 px-1 py-0.5 text-[10px] text-blue-500">
                        {pc.layers.length} lagen
                      </span>
                    )}
                  </span>
                  <span className="ml-2 tabular-nums text-stone-400">
                    {uVal.toFixed(2)} W/m²K
                  </span>
                </button>
              );
            })}
            <div className="border-b border-stone-200" />
          </>
        )}

        {/* Catalogue entries by category */}
        {CATEGORY_ORDER.map((cat) => {
          const entries = filteredCatalogue.get(cat);
          if (!entries) return null;
          return (
            <div key={cat}>
              <div className="sticky top-0 bg-stone-100 px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-stone-500">
                {CATALOGUE_CATEGORY_LABELS[cat]}
              </div>
              {entries.map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  onClick={() => handleSelectCatalogue(entry)}
                  className="flex w-full items-center justify-between px-3 py-1.5 text-left text-sm hover:bg-blue-50"
                >
                  <span className="flex items-center gap-1.5 text-stone-700">
                    {entry.name}
                    {entry.layers && entry.layers.length > 0 && (
                      <span className="rounded bg-blue-50 px-1 py-0.5 text-[10px] text-blue-500">
                        {entry.layers.length} lagen
                      </span>
                    )}
                  </span>
                  <span className="ml-2 tabular-nums text-stone-400">
                    {entry.uValue.toFixed(2)} W/m²K
                  </span>
                </button>
              ))}
            </div>
          );
        })}
        {filteredCatalogue.size === 0 && filteredProject.length === 0 && (
          <div className="px-3 py-3 text-center text-sm text-stone-400">
            Geen resultaten
          </div>
        )}
      </div>
    </div>
  );

  return createPortal(picker, document.body);
}
