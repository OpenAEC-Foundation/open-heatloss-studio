import { useCallback, useMemo, useRef, useState } from "react";

import { getMaterialById, type Material } from "../../lib/materialsDatabase";
import {
  calculateRc,
  RC_MIN_BOUWBESLUIT,
  type LayerInput,
} from "../../lib/rcCalculation";
import type {
  ConstructionElementLayer,
  VerticalPosition,
} from "../../types";
import { MaterialPicker } from "./MaterialPicker";

interface LayerEditorProps {
  layers: ConstructionElementLayer[];
  position: VerticalPosition;
  onApply: (layers: ConstructionElementLayer[], uValue: number) => void;
  onClose: () => void;
}

export function LayerEditor({
  layers: initialLayers,
  position,
  onApply,
  onClose,
}: LayerEditorProps) {
  const [layers, setLayers] = useState<ConstructionElementLayer[]>(
    () => initialLayers.map((l) => ({ ...l })),
  );

  // MaterialPicker state
  const [pickerIndex, setPickerIndex] = useState<number | null>(null);
  const [pickerRect, setPickerRect] = useState<DOMRect | null>(null);
  const materialBtnRefs = useRef<Map<number, HTMLButtonElement>>(new Map());

  // Rc berekening
  const layerInputs: LayerInput[] = useMemo(
    () => layers.map((l) => ({ materialId: l.materialId, thickness: l.thickness })),
    [layers],
  );
  const rcResult = useMemo(
    () => calculateRc(layerInputs, position),
    [layerInputs, position],
  );

  const rcMin = RC_MIN_BOUWBESLUIT[position];
  const meetsRequirement = rcResult.rc >= rcMin;

  // Laag toevoegen
  const handleAddLayer = useCallback(() => {
    setLayers((prev) => [
      ...prev,
      { materialId: "", thickness: 0 },
    ]);
  }, []);

  // Laag verwijderen
  const handleRemoveLayer = useCallback((index: number) => {
    setLayers((prev) => prev.filter((_, i) => i !== index));
  }, []);

  // Laag omhoog
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

  // Laag omlaag
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

  // Dikte wijzigen
  const handleThicknessChange = useCallback((index: number, value: string) => {
    const thickness = Number(value) || 0;
    setLayers((prev) =>
      prev.map((l, i) => (i === index ? { ...l, thickness } : l)),
    );
  }, []);

  // Materiaal selecteren via picker
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

  // Toepassen
  const handleApply = useCallback(() => {
    // Filter lege lagen
    const validLayers = layers.filter((l) => l.materialId);
    onApply(validLayers, rcResult.uValue);
  }, [layers, rcResult.uValue, onApply]);

  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/30"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-3">
          <h3 className="font-heading text-lg font-semibold text-on-surface">
            Constructie-opbouw
          </h3>
          <button
            onClick={onClose}
            className="rounded p-1 text-on-surface-muted hover:bg-surface-alt hover:text-on-surface-secondary"
          >
            <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path
                fillRule="evenodd"
                d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
        </div>

        {/* Lagen tabel */}
        <div className="max-h-96 overflow-y-auto px-4 py-3">
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
                          <svg className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
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
                          <svg className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
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
                          if (el) materialBtnRefs.current.set(index, el);
                          else materialBtnRefs.current.delete(index);
                        }}
                        onClick={() => handleOpenPicker(index)}
                        className="w-full rounded border border-[var(--oaec-border)] px-2 py-1 text-left text-sm hover:border-[var(--oaec-border)] hover:bg-[var(--oaec-hover)]"
                      >
                        {material ? (
                          <span className="text-on-surface-secondary">{material.name}</span>
                        ) : (
                          <span className="text-on-surface-muted">Kies materiaal...</span>
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
                      {layerResult ? layerResult.r.toFixed(3) : "—"}
                    </td>

                    {/* Verwijderen */}
                    <td className="py-1 text-center">
                      <button
                        onClick={() => handleRemoveLayer(index)}
                        className="rounded p-0.5 text-on-surface-muted hover:bg-red-600/15 hover:text-red-400"
                        title="Verwijder laag"
                      >
                        <svg className="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
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

        {/* Resultaten footer */}
        <div className="border-t border-[var(--oaec-border)] px-4 py-3">
          <div className="flex items-center justify-between text-sm">
            <div className="space-y-1">
              <div className="flex items-center gap-4">
                <span className="text-on-surface-muted">
                  Rc = <strong className="text-on-surface">{rcResult.rc.toFixed(2)}</strong> m{"²"}K/W
                </span>
                <span className="text-on-surface-muted">
                  U = <strong className="text-on-surface">{rcResult.uValue.toFixed(3)}</strong> W/m{"²"}K
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

            <div className="flex gap-2">
              <button
                onClick={onClose}
                className="rounded border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
              >
                Annuleren
              </button>
              <button
                onClick={handleApply}
                className="rounded bg-primary px-3 py-1.5 text-sm font-medium text-white hover:bg-primary-hover"
              >
                Toepassen
              </button>
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
