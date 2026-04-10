/**
 * Step 3 — Construction catalog review with LayerEditor integration.
 *
 * Shows the unique construction catalog returned by the thermal import backend.
 * Each entry represents one layer fingerprint, optionally shared between
 * multiple room surfaces. Users can open the LayerEditor to match materials,
 * adjust thicknesses, and calculate Rc/U-values per catalog entry. Edits
 * automatically propagate to every room that references that entry via
 * `catalog_ref`.
 */
import { useCallback, useMemo, useState } from "react";
import {
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Layers,
  Pencil,
  Check,
} from "lucide-react";

import type {
  CatalogEntry,
  ThermalImportConstructionLayer,
  ThermalRoom,
} from "../../lib/thermalImport";
import type { Project } from "../../types";
import { searchMaterials } from "../../lib/materialsDatabase";
import { matchIfcMaterial } from "../../lib/ifcMaterialMatcher";
import type { ConstructionElementLayer, VerticalPosition } from "../../types";
import { formatArea } from "../../lib/formatNumber";
import { LayerEditor } from "../construction/LayerEditor";

interface ConstructionImportStepProps {
  rooms: ThermalRoom[];
  project: Project;
  catalog: CatalogEntry[];
  /**
   * Called when user applies LayerEditor results for a catalog entry.
   * The U-value is shared between every ConstructionElement with that
   * `catalog_ref`.
   */
  onCatalogUValue?: (catalogId: string, uValue: number) => void;
}

/** Get a room's display name by id, falling back to the id. */
function getRoomName(rooms: ThermalRoom[], id: string): string {
  const room = rooms.find((r) => r.id === id);
  return room?.name ?? id;
}

/** Display label for orientation. */
function orientationLabel(o: string): string {
  const labels: Record<string, string> = {
    wall: "Wand",
    floor: "Vloer",
    ceiling: "Plafond",
    roof: "Dak",
  };
  return labels[o] ?? o;
}

/** Display label for boundary type. */
function boundaryLabel(b: string): string {
  const labels: Record<string, string> = {
    exterior: "Buiten",
    ground: "Grond",
    unheated_space: "Onverwarmd",
    adjacent_room: "Aangrenzend vertrek",
    adjacent_building: "Buurpand",
  };
  return labels[b] ?? b;
}

/** Map orientation to VerticalPosition for Rc calculation. */
function orientationToPosition(o: string): VerticalPosition {
  if (o === "floor") return "floor";
  if (o === "ceiling" || o === "roof") return "ceiling";
  return "wall";
}

/**
 * Convert backend catalog layers to ConstructionElementLayer format that the
 * LayerEditor understands. Uses the IFC material matcher to map Revit
 * material names (e.g. `f2_C25/30`, `i1_hout_bamboe`) to database materials
 * via exact / keyword / category-heuristic strategies. Air gaps are mapped
 * to the first `spouw` category entry. When nothing matches, the materialId
 * is left empty so the LayerEditor shows the raw Revit material name as a
 * fallback label via `layerDisplayOverrides` (see `catalogLayerDisplayNames`)
 * and the user can pick a proper database material manually.
 *
 * Guarantees: the returned array has exactly the same length as `layers`,
 * and the `thickness` of each entry comes straight from `thickness_mm`.
 */
function catalogLayersToEditorLayers(
  layers: ThermalImportConstructionLayer[] | undefined | null,
): ConstructionElementLayer[] {
  if (!Array.isArray(layers)) {
    return [];
  }
  return layers.map((tl) => {
    let materialId = "";

    if (tl.type === "air_gap") {
      // Prefer the IFC matcher's "spouw" heuristic so we land on the same
      // cavity material as other import paths. Fall back to a direct
      // category lookup, then to the legacy searchMaterials call.
      const spouwMatch = matchIfcMaterial("spouw");
      if (spouwMatch.material) {
        materialId = spouwMatch.material.id;
      } else {
        const airMatches = searchMaterials("spouw");
        materialId = airMatches.length > 0 ? airMatches[0]!.id : "";
      }
    } else if (tl.material && tl.material.trim()) {
      const match = matchIfcMaterial(tl.material);
      if (match.material) {
        materialId = match.material.id;
      }
    }

    // Carry the exporter's lambda through as an override so the Rc-
    // calculator can still produce a sensible R-value when the material
    // could not be matched against the database.
    const lambdaOverride =
      typeof tl.lambda === "number" && tl.lambda > 0 ? tl.lambda : undefined;

    return {
      materialId,
      thickness: tl.thickness_mm ?? 0,
      lambdaOverride,
    };
  });
}

/**
 * Build a parallel array of display-only labels for the LayerEditor. Each
 * entry is the raw Revit material name (or `"Spouw"` for air gaps). The
 * LayerEditor shows this next to the material picker so the user can still
 * identify the imported layer even when no database match was found.
 */
function catalogLayerDisplayNames(
  layers: ThermalImportConstructionLayer[] | undefined | null,
): (string | null)[] {
  if (!Array.isArray(layers)) {
    return [];
  }
  return layers.map((tl) => {
    if (tl.type === "air_gap") {
      return "Spouw";
    }
    const raw = tl.material?.trim() ?? "";
    return raw.length > 0 ? raw : null;
  });
}

export function ConstructionImportStep({
  rooms,
  project,
  catalog,
  onCatalogUValue,
}: ConstructionImportStepProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  // Track which catalog entry is being edited in LayerEditor
  const [editingId, setEditingId] = useState<string | null>(null);

  // Store calculated U-values per catalog entry id
  const [calculatedUValues, setCalculatedUValues] = useState<Map<string, number>>(
    new Map(),
  );

  // Build a per-catalog-entry list of room references for the "gebruikt in" view.
  // Each element: { roomId, roomName, area }
  const usageByCatalog = useMemo(() => {
    const map = new Map<
      string,
      { roomId: string; roomName: string; area: number }[]
    >();
    for (const room of project.rooms) {
      for (const ce of room.constructions) {
        if (!ce.catalog_ref) continue;
        const list = map.get(ce.catalog_ref) ?? [];
        list.push({
          roomId: room.id,
          roomName: room.name ?? room.id,
          area: ce.area,
        });
        map.set(ce.catalog_ref, list);
      }
    }
    return map;
  }, [project]);

  const handleToggle = useCallback((id: string) => {
    setExpandedId((prev) => (prev === id ? null : id));
  }, []);

  // Open LayerEditor for a catalog entry
  const handleEdit = useCallback((entry: CatalogEntry) => {
    setEditingId(entry.id);
  }, []);

  // LayerEditor apply: save U-value
  const handleLayerApply = useCallback(
    (_layers: ConstructionElementLayer[], uValue: number) => {
      if (editingId) {
        setCalculatedUValues((prev) => {
          const next = new Map(prev);
          next.set(editingId, uValue);
          return next;
        });
        onCatalogUValue?.(editingId, uValue);
      }
      setEditingId(null);
    },
    [editingId, onCatalogUValue],
  );

  const handleLayerClose = useCallback(() => {
    setEditingId(null);
  }, []);

  const withoutLayers = catalog.filter((c) => c.layers.length === 0);

  // Find the catalog entry being edited (for LayerEditor)
  const editingEntry = editingId
    ? catalog.find((e) => e.id === editingId) ?? null
    : null;
  const editingLayers = editingEntry
    ? catalogLayersToEditorLayers(editingEntry.layers)
    : [];
  const editingDisplayOverrides = editingEntry
    ? catalogLayerDisplayNames(editingEntry.layers)
    : [];
  // Pick the most representative position from the entry's `used_for` list,
  // defaulting to wall when nothing useful is recorded.
  const editingPosition: VerticalPosition = editingEntry?.used_for[0]
    ? orientationToPosition(editingEntry.used_for[0][1])
    : "wall";

  return (
    <div>
      <h2 className="mb-2 text-lg font-semibold text-gray-100">
        Constructie catalogus
      </h2>
      <p className="mb-6 text-sm text-gray-400">
        Eén regel per unieke laag-opbouw. Wijzigingen via{" "}
        <Pencil className="inline h-3 w-3" /> gelden automatisch voor alle
        ruimtes die deze constructie gebruiken.
      </p>

      {/* Warning for entries without layers */}
      {withoutLayers.length > 0 && (
        <div className="mb-4 flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-3">
          <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0 text-amber-400" />
          <div>
            <p className="text-sm text-amber-300">
              {withoutLayers.length} catalogus-entrie(s) zonder laag-opbouw
            </p>
            <p className="mt-0.5 text-xs text-amber-400/70">
              Deze constructies hebben geen materiaallagen uit Revit. De
              U-waarde moet handmatig worden ingevoerd na import.
            </p>
          </div>
        </div>
      )}

      {/* Catalog list */}
      <div className="space-y-1">
        {catalog.map((entry) => {
          const isExpanded = expandedId === entry.id;
          const hasLayers = entry.layers.length > 0;
          const calculatedU = calculatedUValues.get(entry.id);
          const hasCalculatedU = calculatedU != null && calculatedU > 0;
          const usage = usageByCatalog.get(entry.id) ?? [];

          return (
            <div
              key={entry.id}
              className="rounded-lg border border-gray-700 bg-gray-800/50"
            >
              {/* Catalog entry header */}
              <button
                onClick={() => handleToggle(entry.id)}
                className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-gray-800"
              >
                {isExpanded ? (
                  <ChevronDown className="h-4 w-4 flex-shrink-0 text-gray-500" />
                ) : (
                  <ChevronRight className="h-4 w-4 flex-shrink-0 text-gray-500" />
                )}

                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-gray-200">
                      {entry.description}
                    </span>
                    {!hasLayers && (
                      <AlertTriangle className="h-3.5 w-3.5 text-amber-400" />
                    )}
                    {hasCalculatedU && (
                      <Check className="h-3.5 w-3.5 text-green-400" />
                    )}
                  </div>
                  <div className="mt-0.5 flex items-center gap-3 text-xs text-gray-500">
                    {entry.used_for.slice(0, 3).map(([bt, or], i) => (
                      <span
                        key={i}
                        className="rounded bg-gray-700/60 px-1.5 py-0.5"
                      >
                        {boundaryLabel(bt)} · {orientationLabel(or)}
                      </span>
                    ))}
                    {entry.used_for.length > 3 && (
                      <span>+{entry.used_for.length - 3}</span>
                    )}
                  </div>
                </div>

                <div className="flex items-center gap-4 text-xs">
                  <span className="tabular-nums text-gray-400">
                    {formatArea(entry.total_area_m2)} m²
                  </span>
                  <span className="flex items-center gap-1 text-gray-500">
                    <Layers className="h-3 w-3" />
                    {entry.layers.length}
                  </span>
                  <span className="text-gray-500">
                    {entry.surface_count}× gebruikt
                  </span>
                  {hasCalculatedU && (
                    <span className="rounded bg-green-900/50 px-1.5 py-0.5 tabular-nums text-green-300">
                      U={calculatedU.toFixed(3)}
                    </span>
                  )}
                </div>
              </button>

              {/* Expanded: layer detail + usage list + edit button */}
              {isExpanded && (
                <div className="border-t border-gray-700 px-4 py-3">
                  {hasLayers ? (
                    <>
                      <CatalogLayerTable layers={entry.layers} />

                      {/* Usage list */}
                      {usage.length > 0 && (
                        <div className="mt-3 border-t border-gray-700/50 pt-3">
                          <p className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-gray-500">
                            Gebruikt in
                          </p>
                          <ul className="space-y-0.5">
                            {usage.map((u, i) => (
                              <li
                                key={`${u.roomId}-${i}`}
                                className="flex items-center justify-between text-xs text-gray-400"
                              >
                                <span>{getRoomName(rooms, u.roomId)}</span>
                                <span className="tabular-nums text-gray-500">
                                  {formatArea(u.area)} m²
                                </span>
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}

                      {/* Edit button */}
                      <div className="mt-3 flex items-center justify-between border-t border-gray-700/50 pt-3">
                        <div className="text-xs text-gray-500">
                          {hasCalculatedU ? (
                            <span className="text-green-400">
                              U = {calculatedU.toFixed(3)} W/m²K (berekend)
                            </span>
                          ) : (
                            <span>U-waarde nog niet berekend</span>
                          )}
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleEdit(entry);
                          }}
                          className="flex items-center gap-1.5 rounded bg-[#45B6A8] px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-[#3da396]"
                        >
                          <Pencil className="h-3 w-3" />
                          Bewerken in Rc-calculator
                        </button>
                      </div>
                    </>
                  ) : (
                    <p className="text-xs italic text-gray-500">
                      Geen laag-opbouw beschikbaar uit Revit. Voeg na import
                      handmatig een constructie toe.
                    </p>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Stats */}
      <div className="mt-4 flex items-center gap-4 text-xs text-gray-500">
        <span>
          {catalog.length} unieke constructies ·{" "}
          {catalog.length - withoutLayers.length} met laag-opbouw
        </span>
        {calculatedUValues.size > 0 && (
          <span className="text-green-400">
            {calculatedUValues.size} berekend
          </span>
        )}
      </div>

      {/* LayerEditor modal */}
      {editingId && editingEntry && (
        <LayerEditor
          key={editingId}
          layers={editingLayers}
          position={editingPosition}
          onApply={handleLayerApply}
          onClose={handleLayerClose}
          layerDisplayOverrides={editingDisplayOverrides}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-component: read-only layer table for preview
// ---------------------------------------------------------------------------

interface CatalogLayerTableProps {
  layers: ThermalImportConstructionLayer[];
}

function CatalogLayerTable({ layers }: CatalogLayerTableProps) {
  return (
    <table className="w-full text-xs">
      <thead>
        <tr className="border-b border-gray-700/50 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-500">
          <th className="pb-1.5 pr-3">Materiaal (Revit)</th>
          <th className="pb-1.5 pr-3">Database match</th>
          <th className="pb-1.5 pr-3 text-right">Dikte [mm]</th>
          <th className="pb-1.5 text-right">Lambda [W/mK]</th>
        </tr>
      </thead>
      <tbody>
        {layers.map((layer, i) => {
          // Use the IFC material matcher for consistency with the LayerEditor
          // conversion — so the preview shows the exact same match the user
          // will see after clicking "Bewerken in Rc-calculator".
          let matchName: string | null = null;
          if (layer.type === "air_gap") {
            const spouwMatch = matchIfcMaterial("spouw");
            if (spouwMatch.material) {
              matchName = spouwMatch.material.name;
            } else {
              const airMatches = searchMaterials("spouw");
              matchName = airMatches.length > 0 ? airMatches[0]!.name : null;
            }
          } else if (layer.material.trim()) {
            const match = matchIfcMaterial(layer.material);
            matchName = match.material ? match.material.name : null;
          }

          return (
            <tr key={i} className="border-b border-gray-700/30">
              <td className="py-1.5 pr-3 text-gray-300">
                {layer.material || (
                  <span className="italic text-gray-600">(leeg)</span>
                )}
                {layer.type === "air_gap" && (
                  <span className="ml-1.5 text-[10px] text-gray-500">
                    (spouw)
                  </span>
                )}
              </td>
              <td className="py-1.5 pr-3">
                {matchName ? (
                  <span className="text-[#45B6A8]">{matchName}</span>
                ) : (
                  <span className="italic text-gray-600">geen match</span>
                )}
              </td>
              <td className="py-1.5 pr-3 text-right tabular-nums text-gray-400">
                {layer.thickness_mm.toFixed(1)}
              </td>
              <td className="py-1.5 text-right tabular-nums text-gray-400">
                {layer.lambda != null ? layer.lambda.toFixed(3) : "—"}
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
