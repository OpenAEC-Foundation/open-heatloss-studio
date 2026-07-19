/**
 * "IFC-reconstructie (bèta)" — fase 2b.
 *
 * Draait de fase-2a reconstructie-pipeline (`lib/ifcReconstruction/pipeline.ts`,
 * via de worker-client) op een lokaal gekozen .ifc-bestand (runtime-only,
 * niets wordt geüpload/opgeslagen), toont het resultaat als 3D-vlakken +
 * oppervlaktenlijst, en biedt een optionele vergelijking met een
 * pyrevit-warmteverlies-project-JSON van hetzelfde gebouw ("de bestaande
 * methode").
 *
 * Bewust NAAST de bestaande flows (rooms/results/modeller): niets hier
 * schrijft naar de project-store. Zie orchestrator-sessie "fase 2b" voor de
 * scope-afspraak.
 */
import { useCallback, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { PageHeader } from "../components/layout/PageHeader";
import { Card } from "../components/ui/Card";
import { ReconstructionViewer3D } from "../components/ifcReconstruction/ReconstructionViewer3D";
import { SurfaceTable } from "../components/ifcReconstruction/SurfaceTable";
import { ComparisonTable } from "../components/ifcReconstruction/ComparisonTable";
import { runIfcReconstructionInWorker } from "../workers/ifcReconstruction.client";
import type { ProgressEvent, ReconstructionResult } from "../lib/ifcReconstruction/types";
import {
  compareWithPyrevit,
  flattenFaces,
  parsePyrevitJson,
  serializeFacesToClipboardText,
  serializeFacesToCsv,
  type ComparisonResult,
} from "../lib/ifcReconstruction/report";
import { useToastStore } from "../store/toastStore";

const inputClass =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-2 py-1 text-sm text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

const buttonClass =
  "rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-on-accent hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50";

const secondaryButtonClass =
  "rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm font-medium text-on-surface-secondary hover:bg-[var(--oaec-hover)] disabled:cursor-not-allowed disabled:opacity-50";

function downloadTextFile(filename: string, content: string, mime: string): void {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export function IfcReconstruction() {
  const { t } = useTranslation();
  const addToast = useToastStore((s) => s.addToast);

  const [maaiveldInput, setMaaiveldInput] = useState("");
  const [ifcFile, setIfcFile] = useState<File | null>(null);
  const [pyrevitFile, setPyrevitFile] = useState<File | null>(null);

  const [isRunning, setIsRunning] = useState(false);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ReconstructionResult | null>(null);
  const [comparison, setComparison] = useState<ComparisonResult | null>(null);

  const [selectedRowKey, setSelectedRowKey] = useState<string | null>(null);
  const [qcOnly, setQcOnly] = useState(false);

  const ifcInputRef = useRef<HTMLInputElement>(null);
  const pyrevitInputRef = useRef<HTMLInputElement>(null);

  const handleRun = useCallback(async () => {
    if (!ifcFile) return;
    setIsRunning(true);
    setError(null);
    setProgress(null);
    setResult(null);
    setComparison(null);
    setSelectedRowKey(null);

    const maaiveldMM = maaiveldInput.trim() === "" ? undefined : Number(maaiveldInput.replace(",", "."));
    const opts = maaiveldMM !== undefined && Number.isFinite(maaiveldMM) ? { maaiveldMM } : {};

    try {
      const r = await runIfcReconstructionInWorker(ifcFile, opts, (event) => setProgress(event));
      setResult(r);
      addToast(t("ifcReconstruction.toast.done"), "success");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast(`${t("ifcReconstruction.toast.failed")}: ${message}`, "error", 6000);
    } finally {
      setIsRunning(false);
    }
  }, [ifcFile, maaiveldInput, addToast, t]);

  const handleCompare = useCallback(async () => {
    if (!result || !pyrevitFile) return;
    try {
      const text = await pyrevitFile.text();
      const parsed = parsePyrevitJson(JSON.parse(text));
      setComparison(compareWithPyrevit(result, parsed));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      addToast(`${t("ifcReconstruction.toast.compareFailed")}: ${message}`, "error", 6000);
    }
  }, [result, pyrevitFile, addToast, t]);

  const flatRows = useMemo(() => (result ? flattenFaces(result) : []), [result]);

  const handleExportCsv = useCallback(() => {
    if (flatRows.length === 0) return;
    downloadTextFile("ifc-reconstructie-oppervlakten.csv", serializeFacesToCsv(flatRows), "text/csv;charset=utf-8");
  }, [flatRows]);

  const handleCopyClipboard = useCallback(async () => {
    if (flatRows.length === 0) return;
    try {
      await navigator.clipboard.writeText(serializeFacesToClipboardText(flatRows));
      addToast(t("ifcReconstruction.toast.copied"), "success");
    } catch {
      addToast(t("ifcReconstruction.toast.copyFailed"), "error");
    }
  }, [flatRows, addToast, t]);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <PageHeader
        title={t("ifcReconstruction.title")}
        subtitle={t("ifcReconstruction.betaSubtitle")}
      />

      <div className="flex-1 overflow-auto p-6 space-y-4">
        {/* Beta banner */}
        <div className="flex items-center gap-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-800">
          <span className="rounded bg-amber-500 px-1.5 py-0.5 font-bold uppercase tracking-wide text-white">
            {t("ifcReconstruction.betaBadge")}
          </span>
          <span>{t("ifcReconstruction.betaNote")}</span>
        </div>

        {/* Input */}
        <Card title={t("ifcReconstruction.inputTitle")}>
          <div className="flex flex-wrap items-end gap-3">
            <div className="flex flex-col gap-1">
              <label className="text-xs text-on-surface-muted">{t("ifcReconstruction.ifcFile")}</label>
              <input
                ref={ifcInputRef}
                type="file"
                accept=".ifc"
                className={inputClass}
                onChange={(e) => setIfcFile(e.target.files?.[0] ?? null)}
              />
            </div>
            <div className="flex flex-col gap-1">
              <label className="text-xs text-on-surface-muted">{t("ifcReconstruction.maaiveldOverride")}</label>
              <input
                type="text"
                inputMode="decimal"
                placeholder="mm"
                value={maaiveldInput}
                onChange={(e) => setMaaiveldInput(e.target.value)}
                className={`${inputClass} w-28`}
              />
            </div>
            <button className={buttonClass} disabled={!ifcFile || isRunning} onClick={handleRun}>
              {isRunning ? t("ifcReconstruction.running") : t("ifcReconstruction.start")}
            </button>
          </div>

          {isRunning && progress && (
            <div className="mt-3">
              <div className="mb-1 flex justify-between text-xs text-on-surface-muted">
                <span>{progress.message ?? progress.phase}</span>
                <span>{progress.percent}%</span>
              </div>
              <div className="h-2 w-full overflow-hidden rounded bg-surface-alt">
                <div
                  className="h-full bg-primary transition-all"
                  style={{ width: `${Math.min(100, Math.max(0, progress.percent))}%` }}
                />
              </div>
            </div>
          )}

          {error && (
            <div className="mt-3 rounded-md border border-red-300 bg-red-50 px-3 py-2 text-xs text-red-700">
              {error}
            </div>
          )}
        </Card>

        {result && (
          <>
            {/* 3D + list */}
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <Card title={t("ifcReconstruction.view3dTitle")}>
                <div className="h-[480px]">
                  <ReconstructionViewer3D
                    result={result}
                    selectedRowKey={selectedRowKey}
                    onSelectRowKey={setSelectedRowKey}
                  />
                </div>
              </Card>

              <Card title={t("ifcReconstruction.listTitle")}>
                <div className="flex items-center justify-between gap-2 border-b border-[var(--oaec-border-subtle)] px-3 py-2">
                  <label className="flex items-center gap-1.5 text-xs text-on-surface-secondary">
                    <input type="checkbox" checked={qcOnly} onChange={(e) => setQcOnly(e.target.checked)} />
                    {t("ifcReconstruction.qcOnlyFilter")}
                  </label>
                  <div className="flex gap-2">
                    <button className={secondaryButtonClass} onClick={handleCopyClipboard}>
                      {t("ifcReconstruction.copyClipboard")}
                    </button>
                    <button className={secondaryButtonClass} onClick={handleExportCsv}>
                      {t("ifcReconstruction.exportCsv")}
                    </button>
                  </div>
                </div>
                <div className="h-[440px] overflow-auto">
                  <SurfaceTable
                    result={result}
                    selectedRowKey={selectedRowKey}
                    onSelectRowKey={setSelectedRowKey}
                    qcOnly={qcOnly}
                  />
                </div>
              </Card>
            </div>

            {/* Comparison */}
            <Card title={t("ifcReconstruction.compare.title")}>
              <p className="mb-3 text-xs text-on-surface-muted">{t("ifcReconstruction.compare.description")}</p>
              <div className="flex flex-wrap items-end gap-3">
                <div className="flex flex-col gap-1">
                  <label className="text-xs text-on-surface-muted">{t("ifcReconstruction.compare.pyrevitFile")}</label>
                  <input
                    ref={pyrevitInputRef}
                    type="file"
                    accept=".json"
                    className={inputClass}
                    onChange={(e) => setPyrevitFile(e.target.files?.[0] ?? null)}
                  />
                </div>
                <button className={buttonClass} disabled={!pyrevitFile} onClick={handleCompare}>
                  {t("ifcReconstruction.compare.run")}
                </button>
              </div>

              {comparison && (
                <div className="mt-4">
                  <ComparisonTable comparison={comparison} />
                </div>
              )}
            </Card>
          </>
        )}
      </div>
    </div>
  );
}
