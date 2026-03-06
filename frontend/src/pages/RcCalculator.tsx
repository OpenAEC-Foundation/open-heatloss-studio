import { useCallback, useMemo, useRef, useState } from "react";

import { PageHeader } from "../components/layout/PageHeader";
import { MaterialPicker } from "../components/construction/MaterialPicker";
import { Button } from "../components/ui/Button";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
} from "../lib/constructionCatalogue";
import { BOUNDARY_TYPE_LABELS } from "../lib/constants";
import { getMaterialById, type Material } from "../lib/materialsDatabase";
import {
  calculateRc,
  RC_MIN_BOUWBESLUIT,
  type LayerInput,
} from "../lib/rcCalculation";
import { useCatalogueStore } from "../store/catalogueStore";
import type { BoundaryType, MaterialType, VerticalPosition } from "../types";

// ---------- Constanten ----------

/** Categorieën die Rc-berekening ondersteunen (kozijnen niet). */
const RC_CATEGORIES: CatalogueCategory[] = [
  "wanden",
  "vloeren_plafonds",
  "daken",
];

const CATEGORY_POSITION: Record<string, VerticalPosition> = {
  wanden: "wall",
  vloeren_plafonds: "floor",
  daken: "ceiling",
};

const MATERIAL_TYPE_LABELS: Record<MaterialType, string> = {
  masonry: "Steenachtig",
  non_masonry: "Niet-steenachtig",
};

const BOUNDARY_OPTIONS: BoundaryType[] = [
  "exterior",
  "unheated_space",
  "adjacent_room",
  "adjacent_building",
  "ground",
];

// ---------- Component ----------

export function RcCalculator() {
  // Metadata
  const [name, setName] = useState("");
  const [category, setCategory] = useState<CatalogueCategory>("wanden");
  const [materialType, setMaterialType] = useState<MaterialType>("masonry");
  const [boundaryType, setBoundaryType] = useState<BoundaryType>("exterior");

  // Lagen
  const [layers, setLayers] = useState<LayerInput[]>([
    { materialId: "", thickness: 0 },
  ]);

  // MaterialPicker state
  const [pickerIndex, setPickerIndex] = useState<number | null>(null);
  const [pickerRect, setPickerRect] = useState<DOMRect | null>(null);
  const materialBtnRefs = useRef<Map<number, HTMLButtonElement>>(new Map());

  // Opslaan feedback
  const [saved, setSaved] = useState(false);

  const addEntry = useCatalogueStore((s) => s.addEntry);

  // Afgeleide waarden
  const position = CATEGORY_POSITION[category] ?? "wall";

  const rcResult = useMemo(
    () => calculateRc(layers, position),
    [layers, position],
  );

  const rcMin = RC_MIN_BOUWBESLUIT[position];
  const meetsRequirement = rcResult.rc >= rcMin;

  // ---------- Laag-handlers ----------

  const handleAddLayer = useCallback(() => {
    setLayers((prev) => [...prev, { materialId: "", thickness: 0 }]);
  }, []);

  const handleRemoveLayer = useCallback((index: number) => {
    setLayers((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleMoveUp = useCallback((index: number) => {
    if (index === 0) return;
    setLayers((prev) => {
      const next = [...prev];
      const temp = next[index]!;
      next[index] = next[index - 1]!;
      next[index - 1] = temp;
      return next;
    });
  }, []);

  const handleMoveDown = useCallback((index: number) => {
    setLayers((prev) => {
      if (index >= prev.length - 1) return prev;
      const next = [...prev];
      const temp = next[index]!;
      next[index] = next[index + 1]!;
      next[index + 1] = temp;
      return next;
    });
  }, []);

  const handleThicknessChange = useCallback(
    (index: number, value: string) => {
      const thickness = Number(value) || 0;
      setLayers((prev) =>
        prev.map((l, i) => (i === index ? { ...l, thickness } : l)),
      );
    },
    [],
  );

  // ---------- MaterialPicker handlers ----------

  const handleOpenPicker = useCallback((index: number) => {
    const btn = materialBtnRefs.current.get(index);
    if (btn) {
      setPickerRect(btn.getBoundingClientRect());
    }
    setPickerIndex(index);
  }, []);

  const handleSelectMaterial = useCallback(
    (material: Material) => {
      if (pickerIndex === null) return;
      setLayers((prev) =>
        prev.map((l, i) =>
          i === pickerIndex ? { ...l, materialId: material.id } : l,
        ),
      );
      setPickerIndex(null);
      setPickerRect(null);
    },
    [pickerIndex],
  );

  const handleClosePicker = useCallback(() => {
    setPickerIndex(null);
    setPickerRect(null);
  }, []);

  // ---------- Opslaan ----------

  const handleSave = useCallback(() => {
    const validLayers = layers.filter((l) => l.materialId);
    if (!name.trim() || validLayers.length === 0) return;

    addEntry({
      name: name.trim(),
      category,
      uValue: Math.round(rcResult.uValue * 1000) / 1000,
      materialType,
      verticalPosition: position,
      boundaryType,
      layers: validLayers.map((l) => ({
        materialId: l.materialId,
        thickness: l.thickness,
      })),
    });

    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }, [name, category, materialType, position, boundaryType, layers, rcResult.uValue, addEntry]);

  const canSave =
    name.trim().length > 0 && layers.some((l) => l.materialId);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Rc-waarde"
        subtitle="Constructie-opbouw samenstellen en opslaan"
      />

      <div className="flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-3xl space-y-6">
          {/* Metadata */}
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            {/* Naam */}
            <div className="col-span-2 sm:col-span-1">
              <label className="mb-1 block text-xs font-medium text-stone-500">
                Naam
              </label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Bijv. Spouwmuur nieuwbouw"
                className="w-full rounded border border-stone-200 px-2.5 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
              />
            </div>

            {/* Categorie */}
            <div>
              <label className="mb-1 block text-xs font-medium text-stone-500">
                Categorie
              </label>
              <select
                value={category}
                onChange={(e) =>
                  setCategory(e.target.value as CatalogueCategory)
                }
                className="w-full rounded border border-stone-200 px-2.5 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
              >
                {RC_CATEGORIES.map((cat) => (
                  <option key={cat} value={cat}>
                    {CATALOGUE_CATEGORY_LABELS[cat]}
                  </option>
                ))}
              </select>
            </div>

            {/* Materiaaltype */}
            <div>
              <label className="mb-1 block text-xs font-medium text-stone-500">
                Materiaaltype
              </label>
              <select
                value={materialType}
                onChange={(e) =>
                  setMaterialType(e.target.value as MaterialType)
                }
                className="w-full rounded border border-stone-200 px-2.5 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
              >
                {(Object.entries(MATERIAL_TYPE_LABELS) as [MaterialType, string][]).map(
                  ([val, label]) => (
                    <option key={val} value={val}>
                      {label}
                    </option>
                  ),
                )}
              </select>
            </div>

            {/* Grenstype */}
            <div>
              <label className="mb-1 block text-xs font-medium text-stone-500">
                Grenstype
              </label>
              <select
                value={boundaryType}
                onChange={(e) =>
                  setBoundaryType(e.target.value as BoundaryType)
                }
                className="w-full rounded border border-stone-200 px-2.5 py-1.5 text-sm focus:border-blue-400 focus:outline-none"
              >
                {BOUNDARY_OPTIONS.map((bt) => (
                  <option key={bt} value={bt}>
                    {BOUNDARY_TYPE_LABELS[bt]}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Lagen tabel */}
          <div className="rounded-lg border border-stone-200 bg-white">
            <div className="border-b border-stone-200 px-4 py-2.5">
              <h3 className="text-sm font-semibold text-stone-700">
                Constructie-opbouw
              </h3>
            </div>

            <div className="px-4 py-3">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-stone-200 text-left text-xs font-semibold uppercase tracking-wider text-stone-500">
                    <th className="w-8 pb-2" />
                    <th className="pb-2">Materiaal</th>
                    <th className="w-24 pb-2 text-right">Dikte [mm]</th>
                    <th className="w-24 pb-2 text-right">
                      R [m{"\u00B2"}K/W]
                    </th>
                    <th className="w-16 pb-2" />
                  </tr>
                </thead>
                <tbody>
                  {/* Rsi */}
                  <tr className="text-stone-400">
                    <td />
                    <td className="py-1 text-xs italic">
                      Binnenoppervlakteweerstand (Rsi)
                    </td>
                    <td />
                    <td className="py-1 text-right tabular-nums">
                      {rcResult.rSi.toFixed(2)}
                    </td>
                    <td />
                  </tr>

                  {/* Lagen */}
                  {layers.map((layer, index) => {
                    const material = layer.materialId
                      ? getMaterialById(layer.materialId)
                      : undefined;
                    const layerResult = rcResult.layers[index];

                    return (
                      <tr
                        key={index}
                        className="border-b border-stone-100 hover:bg-stone-50/50"
                      >
                        {/* Volgorde knoppen */}
                        <td className="py-1">
                          <div className="flex flex-col gap-0.5">
                            <button
                              onClick={() => handleMoveUp(index)}
                              disabled={index === 0}
                              className="rounded p-0.5 text-stone-400 hover:text-stone-600 disabled:opacity-30"
                              title="Omhoog"
                            >
                              <svg
                                className="h-3 w-3"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                              >
                                <path
                                  fillRule="evenodd"
                                  d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z"
                                  clipRule="evenodd"
                                />
                              </svg>
                            </button>
                            <button
                              onClick={() => handleMoveDown(index)}
                              disabled={index === layers.length - 1}
                              className="rounded p-0.5 text-stone-400 hover:text-stone-600 disabled:opacity-30"
                              title="Omlaag"
                            >
                              <svg
                                className="h-3 w-3"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                              >
                                <path
                                  fillRule="evenodd"
                                  d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                                  clipRule="evenodd"
                                />
                              </svg>
                            </button>
                          </div>
                        </td>

                        {/* Materiaal */}
                        <td className="py-1">
                          <button
                            ref={(el) => {
                              if (el)
                                materialBtnRefs.current.set(index, el);
                              else materialBtnRefs.current.delete(index);
                            }}
                            onClick={() => handleOpenPicker(index)}
                            className="w-full rounded border border-stone-200 px-2 py-1 text-left text-sm hover:border-stone-300 hover:bg-stone-50"
                          >
                            {material ? (
                              <span className="text-stone-700">
                                {material.name}
                              </span>
                            ) : (
                              <span className="text-stone-400">
                                Kies materiaal...
                              </span>
                            )}
                          </button>
                        </td>

                        {/* Dikte */}
                        <td className="py-1 text-right">
                          {material?.rdFixed !== null &&
                          material?.rdFixed !== undefined ? (
                            <span className="text-xs text-stone-400">
                              n.v.t.
                            </span>
                          ) : (
                            <input
                              type="number"
                              step="any"
                              min="0"
                              value={layer.thickness || ""}
                              onChange={(e) =>
                                handleThicknessChange(index, e.target.value)
                              }
                              className="w-20 rounded border border-stone-200 px-2 py-1 text-right text-sm focus:border-blue-400 focus:outline-none"
                              placeholder="0"
                            />
                          )}
                        </td>

                        {/* R-waarde */}
                        <td className="py-1 text-right tabular-nums text-stone-600">
                          {layerResult ? layerResult.r.toFixed(3) : "\u2014"}
                        </td>

                        {/* Verwijderen */}
                        <td className="py-1 text-center">
                          <button
                            onClick={() => handleRemoveLayer(index)}
                            className="rounded p-0.5 text-stone-400 hover:bg-red-50 hover:text-red-600"
                            title="Verwijder laag"
                          >
                            <svg
                              className="h-3.5 w-3.5"
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
                        </td>
                      </tr>
                    );
                  })}

                  {/* Rse */}
                  <tr className="text-stone-400">
                    <td />
                    <td className="py-1 text-xs italic">
                      Buitenoppervlakteweerstand (Rse)
                    </td>
                    <td />
                    <td className="py-1 text-right tabular-nums">
                      {rcResult.rSe.toFixed(2)}
                    </td>
                    <td />
                  </tr>
                </tbody>
              </table>

              {/* Laag toevoegen */}
              <button
                onClick={handleAddLayer}
                className="mt-2 w-full rounded border border-dashed border-stone-300 px-3 py-1.5 text-sm text-stone-500 hover:border-stone-400 hover:bg-stone-50 hover:text-stone-700"
              >
                + Laag toevoegen
              </button>
            </div>
          </div>

          {/* Resultaten */}
          <div className="rounded-lg border border-stone-200 bg-white px-4 py-3">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <div className="flex items-center gap-6 text-sm">
                  <span className="text-stone-500">
                    Rc ={" "}
                    <strong className="text-stone-800">
                      {rcResult.rc.toFixed(2)}
                    </strong>{" "}
                    m{"\u00B2"}K/W
                  </span>
                  <span className="text-stone-500">
                    R<sub>totaal</sub> ={" "}
                    <strong className="text-stone-800">
                      {rcResult.rTotal.toFixed(2)}
                    </strong>{" "}
                    m{"\u00B2"}K/W
                  </span>
                  <span className="text-stone-500">
                    U ={" "}
                    <strong className="text-stone-800">
                      {rcResult.uValue.toFixed(3)}
                    </strong>{" "}
                    W/m{"\u00B2"}K
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-block h-2 w-2 rounded-full ${
                      meetsRequirement ? "bg-green-500" : "bg-red-500"
                    }`}
                  />
                  <span className="text-xs text-stone-500">
                    Bouwbesluit 2024: Rc {"\u2265"} {rcMin} m{"\u00B2"}K/W
                    {meetsRequirement ? " \u2714" : " \u2718"}
                  </span>
                </div>
              </div>

              <div className="flex items-center gap-2">
                {saved && (
                  <span className="text-xs text-green-600">
                    Opgeslagen!
                  </span>
                )}
                <Button
                  onClick={handleSave}
                  disabled={!canSave}
                  size="md"
                >
                  Opslaan in bibliotheek
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* MaterialPicker portal */}
      {pickerIndex !== null && (
        <MaterialPicker
          anchorRect={pickerRect}
          onSelect={handleSelectMaterial}
          onClose={handleClosePicker}
        />
      )}
    </div>
  );
}
