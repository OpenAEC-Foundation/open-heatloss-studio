import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { GlaserDiagram } from "../components/construction/GlaserDiagram";
import { MoistureYearTable } from "../components/construction/MoistureYearTable";
import { PageHeader } from "../components/layout/PageHeader";
import { MaterialPicker } from "../components/construction/MaterialPicker";
import { Button } from "../components/ui/Button";
import {
  buildLayerName,
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
} from "../lib/constructionCatalogue";
import {
  calculateGlaser,
  GLASER_DEFAULTS,
} from "../lib/glaserCalculation";
import { getMaterialById, type Material } from "../lib/materialsDatabase";
import { buildRcReportData } from "../lib/rcReportBuilder";
import {
  calculateRc,
  RC_MIN_BOUWBESLUIT,
  type LayerInput,
} from "../lib/rcCalculation";
import { generateReportDirect } from "../lib/reportClient";
import { calculateYearlyMoisture } from "../lib/yearlyMoistureCalculation";
import { useCatalogueStore } from "../store/catalogueStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import { useToastStore } from "../store/toastStore";
import type { MaterialType, VerticalPosition } from "../types";

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

// ---------- Component ----------

export function RcCalculator() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const editId = searchParams.get("edit");

  // Metadata
  const [name, setName] = useState("");
  const [category, setCategory] = useState<CatalogueCategory>("wanden");
  const [materialType, setMaterialType] = useState<MaterialType>("masonry");

  // Lagen
  const [layers, setLayers] = useState<LayerInput[]>([
    { materialId: "", thickness: 0 },
  ]);

  // Klimaatcondities (Glaser)
  const [thetaI, setThetaI] = useState<number>(GLASER_DEFAULTS.thetaI);
  const [thetaE, setThetaE] = useState<number>(GLASER_DEFAULTS.thetaE);
  const [rhI, setRhI] = useState<number>(GLASER_DEFAULTS.rhI);
  const [rhE, setRhE] = useState<number>(GLASER_DEFAULTS.rhE);

  // MaterialPicker state
  const [pickerIndex, setPickerIndex] = useState<number | null>(null);
  const [pickerRect, setPickerRect] = useState<DOMRect | null>(null);
  const materialBtnRefs = useRef<Map<number, HTMLButtonElement>>(new Map());

  // Opslaan feedback
  const [saved, setSaved] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);

  const addEntry = useCatalogueStore((s) => s.addEntry);
  const updateEntry = useCatalogueStore((s) => s.updateEntry);
  const allEntries = useCatalogueStore((s) => s.entries);
  const addProjectConstruction = useModellerStore(
    (s) => s.addProjectConstruction,
  );
  const addToast = useToastStore((s) => s.addToast);

  // Load entry when editing
  useEffect(() => {
    if (!editId) return;
    const entry = allEntries.find((e) => e.id === editId);
    if (!entry) return;
    setName(entry.name);
    setCategory(entry.category);
    setMaterialType(entry.materialType);
    if (entry.layers?.length) {
      setLayers(entry.layers.map((l) => ({ materialId: l.materialId, thickness: l.thickness })));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [editId]);

  // Afgeleide waarden
  const position = CATEGORY_POSITION[category] ?? "wall";

  const rcResult = useMemo(
    () => calculateRc(layers, position),
    [layers, position],
  );

  const glaserResult = useMemo(
    () =>
      calculateGlaser({
        layers,
        position,
        thetaI,
        thetaE,
        rhI,
        rhE,
      }),
    [layers, position, thetaI, thetaE, rhI, rhE],
  );

  const moistureResult = useMemo(
    () => calculateYearlyMoisture(layers, position, thetaI, rhI),
    [layers, position, thetaI, rhI],
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
    if (validLayers.length === 0) return;

    const layerName = buildLayerName(validLayers);
    const entryData = {
      name: layerName,
      category,
      uValue: Math.round(rcResult.uValue * 1000) / 1000,
      materialType,
      verticalPosition: position,
      layers: validLayers.map((l) => ({
        materialId: l.materialId,
        thickness: l.thickness,
      })),
    };

    if (editId) {
      updateEntry(editId, entryData);
      setName(layerName);
    } else {
      addEntry(entryData);
    }

    setSaved(true);
    setTimeout(() => {
      setSaved(false);
      if (editId) navigate("/library");
    }, 1000);
  }, [category, materialType, position, layers, rcResult.uValue, addEntry, updateEntry, editId, navigate]);

  const handleSaveToProject = useCallback(() => {
    const validLayers = layers.filter((l) => l.materialId);
    if (validLayers.length === 0) return;

    const layerName = buildLayerName(validLayers);
    addProjectConstruction({
      name: layerName,
      category,
      materialType,
      verticalPosition: position,
      layers: validLayers.map((l) => ({
        materialId: l.materialId,
        thickness: l.thickness,
      })),
    });

    addToast("Opgeslagen als projectconstructie", "success");
  }, [category, materialType, position, layers, addProjectConstruction, addToast]);

  const handleGenerateReport = useCallback(async () => {
    setIsGenerating(true);
    const validLayers = layers.filter((l) => l.materialId);
    const reportName = validLayers.length > 0 ? buildLayerName(validLayers) : "constructie";
    try {
      const reportData = await buildRcReportData({
        name: reportName,
        category,
        materialType,
        position,
        layers,
        rcResult,
        glaserResult,
        moistureResult,
        thetaI,
        thetaE,
        rhI,
        rhE,
      });
      const blob = await generateReportDirect(reportData);

      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${reportName}.pdf`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      addToast("Rapport gegenereerd", "success");
    } catch (err) {
      const message = err instanceof Error ? err.message : "Onbekende fout";
      addToast(`Rapport mislukt: ${message}`, "error", 5000);
    } finally {
      setIsGenerating(false);
    }
  }, [category, materialType, position, layers, rcResult, glaserResult, moistureResult, thetaI, thetaE, rhI, rhE, addToast]);

  const canSave = layers.some((l) => l.materialId);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title={editId ? "Constructie bewerken" : "Rc-waarde"}
        subtitle={editId ? name : "Constructie-opbouw samenstellen en opslaan"}
        breadcrumbs={editId ? [{ label: "Bibliotheek", to: "/library" }, { label: "Constructie bewerken" }] : [{ label: "Rc-waarde" }]}
        actions={
          editId ? (
            <button
              type="button"
              onClick={() => navigate("/library")}
              className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
            >
              Terug naar bibliotheek
            </button>
          ) : undefined
        }
      />

      <div className="flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-3xl space-y-6">
          {/* Metadata */}
          <div className="grid grid-cols-2 gap-4">
            {/* Categorie */}
            <div>
              <label className="mb-1 block text-xs font-medium text-on-surface-muted">
                Categorie
              </label>
              <select
                value={category}
                onChange={(e) =>
                  setCategory(e.target.value as CatalogueCategory)
                }
                className="w-full rounded border border-[var(--oaec-border)] px-2.5 py-1.5 text-sm focus:border-primary focus:outline-none"
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
              <label className="mb-1 block text-xs font-medium text-on-surface-muted">
                Materiaaltype
              </label>
              <select
                value={materialType}
                onChange={(e) =>
                  setMaterialType(e.target.value as MaterialType)
                }
                className="w-full rounded border border-[var(--oaec-border)] px-2.5 py-1.5 text-sm focus:border-primary focus:outline-none"
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
          </div>

          {/* Lagen tabel */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                Constructie-opbouw
              </h3>
            </div>

            <div className="px-4 py-3">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                    <th className="w-8 pb-2" />
                    <th className="pb-2">Materiaal</th>
                    <th className="w-24 pb-2 text-right">Dikte [mm]</th>
                    <th className="w-24 pb-2 text-right">
                      R [m{"²"}K/W]
                    </th>
                    <th className="w-16 pb-2" />
                  </tr>
                </thead>
                <tbody>
                  {/* Rsi */}
                  <tr className="text-on-surface-muted">
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
                        className="border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]/50"
                      >
                        {/* Volgorde knoppen */}
                        <td className="py-1">
                          <div className="flex flex-col gap-0.5">
                            <button
                              onClick={() => handleMoveUp(index)}
                              disabled={index === 0}
                              className="rounded p-0.5 text-on-surface-muted hover:text-on-surface-secondary disabled:opacity-30"
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
                              className="rounded p-0.5 text-on-surface-muted hover:text-on-surface-secondary disabled:opacity-30"
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
                            className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-left text-sm hover:border-[var(--oaec-border)] hover:bg-[var(--oaec-hover)]"
                          >
                            {material ? (
                              <span className="text-on-surface-secondary">
                                {material.name}
                              </span>
                            ) : (
                              <span className="text-on-surface-muted">
                                Kies materiaal...
                              </span>
                            )}
                          </button>
                        </td>

                        {/* Dikte */}
                        <td className="py-1 text-right">
                          {material?.sdFixed !== null &&
                          material?.sdFixed !== undefined ? (
                            <span
                              className="text-xs tabular-nums text-on-surface-muted"
                              title="Vaste sd-waarde (productspecificatie)"
                            >
                              {"sd=" + material.sdFixed + " m"}
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
                              className="w-20 rounded border border-[var(--oaec-border)] px-2 py-1 text-right text-sm focus:border-primary focus:outline-none"
                              placeholder="0"
                            />
                          )}
                        </td>

                        {/* R-waarde */}
                        <td className="py-1 text-right tabular-nums text-on-surface-secondary">
                          {layerResult ? layerResult.r.toFixed(3) : "\u2014"}
                        </td>

                        {/* Verwijderen */}
                        <td className="py-1 text-center">
                          <button
                            onClick={() => handleRemoveLayer(index)}
                            className="rounded p-0.5 text-on-surface-muted hover:bg-red-600/15 hover:text-red-400"
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
                  <tr className="text-on-surface-muted">
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
                className="mt-2 w-full rounded border border-dashed border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-muted hover:border-[var(--oaec-border)] hover:bg-[var(--oaec-hover)] hover:text-on-surface-secondary"
              >
                + Laag toevoegen
              </button>
            </div>
          </div>

          {/* Dampspanning (Glaser) */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                Dampspanning (Glaser-methode)
              </h3>
              {glaserResult.hasCondensation ? (
                <span className="flex items-center gap-1.5 rounded-full bg-red-600/20 px-2.5 py-0.5 text-xs font-medium text-red-400">
                  <span className="inline-block h-1.5 w-1.5 rounded-full bg-red-600/150" />
                  Condensatierisico
                </span>
              ) : (
                glaserResult.totalThickness > 0 && (
                  <span className="flex items-center gap-1.5 rounded-full bg-green-600/15 px-2.5 py-0.5 text-xs font-medium text-green-400">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-green-600/100" />
                    Geen condensatie
                  </span>
                )
              )}
            </div>

            <div className="px-4 py-3">
              {/* Klimaatcondities */}
              <div className="mb-4 grid grid-cols-4 gap-3">
                <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                  <span>
                    Temp. binnen [°C]
                  </span>
                  <input
                    type="number"
                    step="1"
                    value={thetaI}
                    onChange={(e) => setThetaI(Number(e.target.value) || 0)}
                    className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm tabular-nums focus:border-primary focus:outline-none"
                  />
                </label>
                <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                  <span>
                    RV binnen [%]
                  </span>
                  <input
                    type="number"
                    step="5"
                    min="0"
                    max="100"
                    value={rhI}
                    onChange={(e) => setRhI(Number(e.target.value) || 0)}
                    className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm tabular-nums focus:border-primary focus:outline-none"
                  />
                </label>
                <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                  <span>
                    Temp. buiten [°C]
                  </span>
                  <input
                    type="number"
                    step="1"
                    value={thetaE}
                    onChange={(e) => setThetaE(Number(e.target.value) || 0)}
                    className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm tabular-nums focus:border-primary focus:outline-none"
                  />
                </label>
                <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                  <span>
                    RV buiten [%]
                  </span>
                  <input
                    type="number"
                    step="5"
                    min="0"
                    max="100"
                    value={rhE}
                    onChange={(e) => setRhE(Number(e.target.value) || 0)}
                    className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm tabular-nums focus:border-primary focus:outline-none"
                  />
                </label>
              </div>

              {/* Diagram */}
              <GlaserDiagram
                result={glaserResult}
                thetaI={thetaI}
                thetaE={thetaE}
              />
            </div>
          </div>

          {/* Jaarlijkse vochtbalans */}
          {moistureResult && (
            <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
              <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-2.5">
                <h3 className="text-sm font-semibold text-on-surface-secondary">
                  Jaarlijkse vochtbalans (NEN-EN-ISO 13788)
                </h3>
                {moistureResult.hasRisk ? (
                  <span className="flex items-center gap-1.5 rounded-full bg-red-600/20 px-2.5 py-0.5 text-xs font-medium text-red-400">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-red-600/150" />
                    Schimmelrisico
                  </span>
                ) : moistureResult.maxMa > 0.1 ? (
                  <span className="flex items-center gap-1.5 rounded-full bg-amber-600/15 px-2.5 py-0.5 text-xs font-medium text-amber-400">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-amber-600/150" />
                    Tijdelijk vocht
                  </span>
                ) : (
                  <span className="flex items-center gap-1.5 rounded-full bg-green-600/15 px-2.5 py-0.5 text-xs font-medium text-green-400">
                    <span className="inline-block h-1.5 w-1.5 rounded-full bg-green-600/100" />
                    Geen vocht
                  </span>
                )}
              </div>
              <div className="px-4 py-3">
                <MoistureYearTable result={moistureResult} />
              </div>
            </div>
          )}

          {/* Resultaten */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <div className="flex items-center gap-6 text-sm">
                  <span className="text-on-surface-muted">
                    Rc ={" "}
                    <strong className="text-on-surface">
                      {rcResult.rc.toFixed(2)}
                    </strong>{" "}
                    m{"²"}K/W
                  </span>
                  <span className="text-on-surface-muted">
                    R<sub>totaal</sub> ={" "}
                    <strong className="text-on-surface">
                      {rcResult.rTotal.toFixed(2)}
                    </strong>{" "}
                    m{"²"}K/W
                  </span>
                  <span className="text-on-surface-muted">
                    U ={" "}
                    <strong className="text-on-surface">
                      {rcResult.uValue.toFixed(3)}
                    </strong>{" "}
                    W/m{"²"}K
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-block h-2 w-2 rounded-full ${
                      meetsRequirement ? "bg-green-600/100" : "bg-red-600/150"
                    }`}
                  />
                  <span className="text-xs text-on-surface-muted">
                    Bouwbesluit 2024: Rc {"≥"} {rcMin} m{"²"}K/W
                    {meetsRequirement ? " \u2714" : " \u2718"}
                  </span>
                </div>
              </div>

              <div className="flex items-center gap-2">
                {saved && (
                  <span className="text-xs text-green-400">
                    Opgeslagen!
                  </span>
                )}
                <Button
                  variant="secondary"
                  onClick={handleGenerateReport}
                  disabled={isGenerating || !layers.some((l) => l.materialId)}
                  size="md"
                >
                  {isGenerating ? "Genereren..." : "Genereer rapport"}
                </Button>
                {!editId && (
                  <Button
                    variant="secondary"
                    onClick={handleSaveToProject}
                    disabled={!canSave}
                    size="md"
                  >
                    Opslaan in project
                  </Button>
                )}
                <Button
                  onClick={handleSave}
                  disabled={!canSave}
                  size="md"
                >
                  {editId ? "Opslaan" : "Opslaan in bibliotheek"}
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
