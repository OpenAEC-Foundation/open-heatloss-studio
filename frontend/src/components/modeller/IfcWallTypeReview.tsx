/**
 * IFC wall type review dialog.
 *
 * Shows extracted wall types + material layer matches after IFC import,
 * allowing the user to confirm/override matches and import selected types
 * as project constructions.
 */
import { useCallback, useState } from "react";

import type { IfcWallTypeInfo, IfcWallTypeLayer } from "./ifc-wall-types";
import type { ProjectConstruction } from "./types";
import { buildLayerName } from "../../lib/constructionCatalogue";
import {
  MATERIALS_DATABASE,
  MATERIAL_CATEGORY_LABELS,
  MATERIAL_CATEGORY_ORDER,
  type Material,
  type MaterialCategory,
} from "../../lib/materialsDatabase";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface IfcWallTypeReviewProps {
  wallTypes: IfcWallTypeInfo[];
  onImport: (constructions: Omit<ProjectConstruction, "id">[]) => void;
  onCancel: () => void;
}

// ---------------------------------------------------------------------------
// Confidence indicator
// ---------------------------------------------------------------------------

const CONFIDENCE_COLORS: Record<string, string> = {
  exact: "bg-green-500",
  keyword: "bg-green-400",
  heuristic: "bg-amber-400",
  none: "bg-red-400",
};

const CONFIDENCE_LABELS: Record<string, string> = {
  exact: "Exact",
  keyword: "Keyword",
  heuristic: "Heuristiek",
  none: "Geen match",
};

function ConfidenceDot({ confidence }: { confidence: string }) {
  return (
    <span
      className={`inline-block h-2 w-2 rounded-full ${CONFIDENCE_COLORS[confidence] ?? "bg-[var(--oaec-hover)]"}`}
      title={CONFIDENCE_LABELS[confidence] ?? confidence}
    />
  );
}

// ---------------------------------------------------------------------------
// Compact material selector (for overrides)
// ---------------------------------------------------------------------------

function MaterialOverrideSelect({
  currentId,
  onChange,
}: {
  currentId: string | null;
  onChange: (materialId: string | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="rounded border border-[var(--oaec-border)] px-1.5 py-0.5 text-[10px] text-amber-400 hover:border-amber-500/30 hover:bg-amber-600/15"
      >
        Wijzig
      </button>
    );
  }

  const grouped = new Map<MaterialCategory, Material[]>();
  const lowerSearch = search.toLowerCase();
  for (const m of MATERIALS_DATABASE) {
    if (
      lowerSearch &&
      !m.name.toLowerCase().includes(lowerSearch) &&
      !m.keywords.some((k) => k.toLowerCase().includes(lowerSearch))
    ) {
      continue;
    }
    const list = grouped.get(m.category) ?? [];
    list.push(m);
    grouped.set(m.category, list);
  }

  return (
    <div className="absolute right-0 top-6 z-50 w-64 rounded-lg border border-[var(--oaec-border)] bg-surface-alt shadow-lg">
      <div className="border-b border-[var(--oaec-border-subtle)] p-2">
        <input
          autoFocus
          placeholder="Zoek materiaal..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-[11px] outline-none focus:border-amber-400"
        />
      </div>
      <div className="max-h-48 overflow-y-auto p-1">
        <button
          onClick={() => {
            onChange(null);
            setOpen(false);
          }}
          className="block w-full rounded px-2 py-1 text-left text-[10px] text-on-surface-muted hover:bg-[var(--oaec-hover)]"
        >
          Geen match (overslaan)
        </button>
        {MATERIAL_CATEGORY_ORDER.filter((cat) => grouped.has(cat)).map(
          (cat) => (
            <div key={cat}>
              <div className="mt-1 px-2 text-[9px] font-semibold uppercase tracking-wider text-on-surface-muted">
                {MATERIAL_CATEGORY_LABELS[cat]}
              </div>
              {grouped.get(cat)!.map((m) => (
                <button
                  key={m.id}
                  onClick={() => {
                    onChange(m.id);
                    setOpen(false);
                  }}
                  className={`block w-full rounded px-2 py-1 text-left text-[10px] hover:bg-amber-600/15 ${
                    m.id === currentId
                      ? "bg-amber-600/15 font-medium text-amber-400"
                      : "text-on-surface-secondary"
                  }`}
                >
                  {m.name}
                </button>
              ))}
            </div>
          ),
        )}
      </div>
      <div className="border-t border-[var(--oaec-border-subtle)] p-1">
        <button
          onClick={() => setOpen(false)}
          className="w-full rounded px-2 py-1 text-[10px] text-on-surface-muted hover:bg-[var(--oaec-hover)]"
        >
          Sluiten
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function IfcWallTypeReview({
  wallTypes,
  onImport,
  onCancel,
}: IfcWallTypeReviewProps) {
  // Track which wall types are selected for import
  const [selected, setSelected] = useState<Set<string>>(
    () => new Set(wallTypes.map((wt) => wt.globalId)),
  );

  // Track material overrides per wallType+layerIndex
  // Key: "globalId:layerIndex" → materialId
  const [overrides, setOverrides] = useState<Record<string, string | null>>(
    {},
  );

  const toggleSelect = useCallback((globalId: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(globalId)) next.delete(globalId);
      else next.add(globalId);
      return next;
    });
  }, []);

  const toggleAll = useCallback(() => {
    setSelected((prev) => {
      if (prev.size === wallTypes.length) return new Set();
      return new Set(wallTypes.map((wt) => wt.globalId));
    });
  }, [wallTypes]);

  const setOverride = useCallback(
    (globalId: string, layerIndex: number, materialId: string | null) => {
      const key = `${globalId}:${layerIndex}`;
      setOverrides((prev) => ({ ...prev, [key]: materialId }));
    },
    [],
  );

  const getEffectiveMaterialId = useCallback(
    (
      layer: IfcWallTypeLayer,
      globalId: string,
      layerIndex: number,
    ): string | null => {
      const key = `${globalId}:${layerIndex}`;
      if (key in overrides) return overrides[key] ?? null;
      return layer.match.material?.id ?? null;
    },
    [overrides],
  );

  const handleImport = useCallback(() => {
    const constructions: Omit<ProjectConstruction, "id">[] = [];

    for (const wt of wallTypes) {
      if (!selected.has(wt.globalId)) continue;

      const layers = wt.layers
        .map((layer, i) => {
          const materialId = getEffectiveMaterialId(
            layer,
            wt.globalId,
            i,
          );
          if (!materialId) return null;
          return {
            materialId,
            thickness: layer.thickness,
          };
        })
        .filter(
          (l): l is { materialId: string; thickness: number } =>
            l !== null,
        );

      if (layers.length === 0) continue;

      constructions.push({
        name: buildLayerName(layers),
        category: "wanden",
        materialType: "masonry",
        verticalPosition: "wall",
        layers,
        ifcSource: {
          wallTypeName: wt.name,
          globalId: wt.globalId,
          originalMaterialNames: wt.originalMaterialNames,
        },
      });
    }

    onImport(constructions);
  }, [wallTypes, selected, getEffectiveMaterialId, onImport]);

  const selectedCount = selected.size;
  const importableCount = wallTypes.filter((wt) => {
    if (!selected.has(wt.globalId)) return false;
    return wt.layers.some((layer, i) =>
      getEffectiveMaterialId(layer, wt.globalId, i),
    );
  }).length;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="max-h-[80vh] w-[700px] overflow-hidden rounded-lg bg-surface-alt shadow-xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-5 py-3">
          <div>
            <h2 className="text-sm font-semibold text-on-surface">
              IFC wandtypen importeren
            </h2>
            <p className="mt-0.5 text-xs text-on-surface-muted">
              {wallTypes.length} wandtype(n) gevonden — controleer de
              materiaalmatching
            </p>
          </div>
          <button
            onClick={onCancel}
            className="rounded p-1 text-on-surface-muted hover:bg-surface-alt hover:text-on-surface-secondary"
          >
            <svg
              className="h-4 w-4"
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
        </div>

        {/* Wall type list */}
        <div className="max-h-[55vh] overflow-y-auto px-5 py-3">
          {/* Select all toggle */}
          <div className="mb-3 flex items-center gap-2">
            <label className="flex cursor-pointer items-center gap-1.5 text-xs text-on-surface-secondary">
              <input
                type="checkbox"
                checked={selectedCount === wallTypes.length}
                onChange={toggleAll}
                className="rounded border-[var(--oaec-border)]"
              />
              Alles selecteren
            </label>
            <span className="text-[10px] text-on-surface-muted">
              {selectedCount} van {wallTypes.length} geselecteerd
            </span>
          </div>

          <div className="space-y-3">
            {wallTypes.map((wt) => {
              const isSelected = selected.has(wt.globalId);

              return (
                <div
                  key={wt.globalId}
                  className={`rounded-lg border p-3 transition-colors ${
                    isSelected
                      ? "border-teal-200 bg-teal-50/30"
                      : "border-[var(--oaec-border)] opacity-60"
                  }`}
                >
                  {/* Wall type header */}
                  <div className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleSelect(wt.globalId)}
                      className="rounded border-[var(--oaec-border)]"
                    />
                    <span className="text-xs font-medium text-on-surface">
                      {wt.name}
                    </span>
                    <span className="text-[10px] text-on-surface-muted">
                      {wt.layers.length} lagen
                    </span>
                  </div>

                  {/* Layers table */}
                  {isSelected && (
                    <table className="mt-2 w-full text-[10px]">
                      <thead>
                        <tr className="border-b border-[var(--oaec-border)] text-left text-[9px] font-semibold uppercase tracking-wider text-on-surface-muted">
                          <th className="w-5 pb-1" />
                          <th className="pb-1">IFC materiaal</th>
                          <th className="w-16 pb-1 text-right">
                            Dikte
                          </th>
                          <th className="pb-1 pl-2">Match</th>
                          <th className="w-12 pb-1" />
                        </tr>
                      </thead>
                      <tbody>
                        {wt.layers.map((layer, li) => {
                          const effectiveId = getEffectiveMaterialId(
                            layer,
                            wt.globalId,
                            li,
                          );
                          const overrideKey = `${wt.globalId}:${li}`;
                          const hasOverride =
                            overrideKey in overrides;
                          const confidence = hasOverride
                            ? effectiveId
                              ? "exact"
                              : "none"
                            : layer.match.confidence;

                          // Find material name for display
                          const matchedMaterial = effectiveId
                            ? MATERIALS_DATABASE.find(
                                (m) => m.id === effectiveId,
                              )
                            : null;

                          return (
                            <tr
                              key={li}
                              className="border-b border-[var(--oaec-border-subtle)]"
                            >
                              <td className="py-1">
                                <ConfidenceDot
                                  confidence={confidence}
                                />
                              </td>
                              <td className="py-1 text-on-surface-secondary">
                                {layer.ifcMaterialName}
                              </td>
                              <td className="py-1 text-right tabular-nums text-on-surface-muted">
                                {layer.thickness > 0
                                  ? `${layer.thickness} mm`
                                  : "\u2014"}
                              </td>
                              <td className="relative py-1 pl-2">
                                {matchedMaterial ? (
                                  <span className="text-on-surface-secondary">
                                    {matchedMaterial.name}
                                  </span>
                                ) : (
                                  <span className="italic text-on-surface-muted">
                                    Geen match
                                  </span>
                                )}
                              </td>
                              <td className="relative py-1">
                                <MaterialOverrideSelect
                                  currentId={effectiveId}
                                  onChange={(id) =>
                                    setOverride(
                                      wt.globalId,
                                      li,
                                      id,
                                    )
                                  }
                                />
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between border-t border-[var(--oaec-border)] px-5 py-3">
          <span className="text-xs text-on-surface-muted">
            {importableCount} constructie(s) importeerbaar
          </span>
          <div className="flex gap-2">
            <button
              onClick={onCancel}
              className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-xs text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
            >
              Annuleren
            </button>
            <button
              onClick={handleImport}
              disabled={importableCount === 0}
              className="rounded-md bg-teal-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-teal-700 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Importeer {importableCount > 0 ? `(${importableCount})` : ""}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
