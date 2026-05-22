import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useSearchParams } from "react-router-dom";

import { PageHeader } from "../components/layout/PageHeader";
import { Button } from "../components/ui/Button";
import { generateReportDirect } from "../lib/reportClient";
import {
  calculateUw,
  computeGeometry,
  fromUwBreakdown,
  resolvePsiG,
  toUwBreakdown,
  validateUwInput,
  type UwInput,
} from "../lib/uwCalculation";
import { buildUwReportData } from "../lib/uwReportBuilder";
import { SPACER_LABELS_NL, SPACER_ORDER, spacerPsiG } from "../lib/spacerTable";
import { useProjectStore } from "../store/projectStore";
import { useToastStore } from "../store/toastStore";
import type { Spacer } from "../types/project";

// ---------- Constanten ----------

/** Startwaarden — worked example 1 uit het werkpakket. */
const DEFAULT_INPUT: UwInput = {
  width_mm: 1200,
  height_mm: 1500,
  frame_width_mm: 80,
  pane_columns: 1,
  pane_rows: 1,
  u_g: 1.1,
  u_f: 1.4,
  spacer: "aluminium",
  psi_g: 0.08,
  psi_g_is_manual: false,
};

// ---------- Component ----------

export function UwCalculator() {
  const { t } = useTranslation();
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  // Edit-modus: kozijn-element bewerken vanuit de constructies-pagina.
  const editRoomId = searchParams.get("room");
  const editConstructionId = searchParams.get("element");
  const isEditMode = !!editRoomId && !!editConstructionId;

  const projectRooms = useProjectStore((s) => s.project.rooms);
  const updateConstruction = useProjectStore((s) => s.updateConstruction);
  const addToast = useToastStore((s) => s.addToast);

  // Geometrie + materiaal
  const [widthMm, setWidthMm] = useState<number>(DEFAULT_INPUT.width_mm);
  const [heightMm, setHeightMm] = useState<number>(DEFAULT_INPUT.height_mm);
  const [frameWidthMm, setFrameWidthMm] = useState<number>(
    DEFAULT_INPUT.frame_width_mm,
  );
  const [paneColumns, setPaneColumns] = useState<number>(
    DEFAULT_INPUT.pane_columns,
  );
  const [paneRows, setPaneRows] = useState<number>(DEFAULT_INPUT.pane_rows);
  const [uG, setUG] = useState<number>(DEFAULT_INPUT.u_g);
  const [uF, setUF] = useState<number>(DEFAULT_INPUT.u_f);

  // Ψ_g — spacer-keuze + handmatige override
  const [spacer, setSpacer] = useState<Spacer>("aluminium");
  const [psiGIsManual, setPsiGIsManual] = useState<boolean>(false);
  const [psiGManual, setPsiGManual] = useState<number>(
    spacerPsiG("aluminium") ?? 0.08,
  );

  // Effectieve invoer
  const input = useMemo<UwInput>(
    () => ({
      width_mm: widthMm,
      height_mm: heightMm,
      frame_width_mm: frameWidthMm,
      pane_columns: paneColumns,
      pane_rows: paneRows,
      u_g: uG,
      u_f: uF,
      spacer: psiGIsManual ? null : spacer,
      psi_g: psiGIsManual ? psiGManual : (spacerPsiG(spacer) ?? psiGManual),
      psi_g_is_manual: psiGIsManual,
    }),
    [
      widthMm,
      heightMm,
      frameWidthMm,
      paneColumns,
      paneRows,
      uG,
      uF,
      spacer,
      psiGIsManual,
      psiGManual,
    ],
  );

  const errors = useMemo(() => validateUwInput(input), [input]);
  const isValid = errors.length === 0;

  const geometry = useMemo(() => computeGeometry(input), [input]);
  const effectivePsiG = useMemo(() => resolvePsiG(input), [input]);
  const result = useMemo(
    () => (isValid ? calculateUw(input) : null),
    [input, isValid],
  );

  /** Fout-melding voor een specifiek veld, of undefined. */
  const errorFor = (field: string): string | undefined =>
    errors.find((e) => e.field === field)?.message;

  // Opslaan-feedback + rapport-status
  const [saved, setSaved] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);

  // ---------- Opslaan op het kozijn-element ----------

  const handleSave = useCallback(() => {
    if (!isEditMode || !result) return;
    const breakdown = toUwBreakdown(input, result);
    updateConstruction(editRoomId!, editConstructionId!, {
      uw_breakdown: breakdown,
      u_value: result.u_w,
    });
    setSaved(true);
    setTimeout(() => {
      setSaved(false);
      navigate("/constructies");
    }, 1000);
  }, [
    isEditMode,
    result,
    input,
    updateConstruction,
    editRoomId,
    editConstructionId,
    navigate,
  ]);

  // ---------- Zelfstandig U_w-rapport ----------

  const handleGenerateReport = useCallback(async () => {
    if (!result) return;
    setIsGenerating(true);
    const reportName = t("uw.reportName");
    try {
      const reportData = await buildUwReportData({
        name: reportName,
        input,
        result,
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

      addToast(t("uw.reportSuccess"), "success");
    } catch (err) {
      const message = err instanceof Error ? err.message : t("uw.reportError");
      addToast(`${t("uw.reportFailed")}: ${message}`, "error", 5000);
    } finally {
      setIsGenerating(false);
    }
  }, [result, input, addToast, t]);

  // Spacer-keuze vult Ψ_g (en kantelt terug naar tabel-modus).
  const handleSpacerChange = (next: Spacer) => {
    setSpacer(next);
    setPsiGManual(spacerPsiG(next) ?? psiGManual);
  };

  // Override aan/uit: bij inschakelen Ψ_g voorvullen met de huidige tabelwaarde.
  const handleToggleManual = (manual: boolean) => {
    setPsiGIsManual(manual);
    if (manual) setPsiGManual(spacerPsiG(spacer) ?? psiGManual);
  };

  // ---------- Edit-modus: bestaande uw_breakdown inladen ----------

  // Bewaakt dat de edit-load eenmalig draait. Re-runt bij projectRooms-
  // updates tot het element gevonden is (Zustand persist hydrateert
  // asynchroon); daarna ref-guard zodat verdere store-updates de lopende
  // edits van de gebruiker niet overschrijven.
  const hasInitializedEditRef = useRef<boolean>(false);

  useEffect(() => {
    if (!isEditMode || hasInitializedEditRef.current) return;
    const room = projectRooms.find((r) => r.id === editRoomId);
    const element = room?.constructions.find(
      (c) => c.id === editConstructionId,
    );
    if (!element) return;
    // Markeer als geïnitialiseerd: ook zonder bestaande uw_breakdown is dit
    // een geldige edit-load (kozijn dat nog geen U_w-opbouw heeft → defaults).
    hasInitializedEditRef.current = true;
    const b = element.uw_breakdown;
    if (!b) return;
    const loaded = fromUwBreakdown(b);
    setWidthMm(loaded.width_mm);
    setHeightMm(loaded.height_mm);
    setFrameWidthMm(loaded.frame_width_mm);
    setPaneColumns(loaded.pane_columns);
    setPaneRows(loaded.pane_rows);
    setUG(loaded.u_g);
    setUF(loaded.u_f);
    setPsiGIsManual(loaded.psi_g_is_manual);
    if (loaded.spacer) setSpacer(loaded.spacer);
    setPsiGManual(loaded.psi_g);
  }, [isEditMode, editRoomId, editConstructionId, projectRooms]);

  const inputClass =
    "w-full rounded border border-[var(--oaec-border)] px-2.5 py-1.5 text-sm tabular-nums focus:border-primary focus:outline-none";

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title={t("uw.title")}
        subtitle={isEditMode ? t("uw.editSubtitle") : t("uw.subtitle")}
        breadcrumbs={
          isEditMode
            ? [
                { label: t("uw.breadcrumbConstructions"), to: "/constructies" },
                { label: t("uw.editTitle") },
              ]
            : [{ label: t("uw.title") }]
        }
        actions={
          isEditMode ? (
            <button
              type="button"
              onClick={() => navigate("/constructies")}
              className="rounded-md border border-[var(--oaec-border)] px-3 py-1.5 text-sm text-on-surface-secondary hover:bg-surface-alt"
            >
              {t("uw.backToConstructions")}
            </button>
          ) : undefined
        }
      />

      <div className="flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-3xl space-y-6">
          {/* Afmetingen & indeling */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                {t("uw.groupGeometry")}
              </h3>
            </div>
            <div className="grid grid-cols-3 gap-3 px-4 py-3">
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.width")}</span>
                <input
                  type="number"
                  min="1"
                  step="10"
                  value={widthMm || ""}
                  onChange={(e) => setWidthMm(Number(e.target.value) || 0)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.height")}</span>
                <input
                  type="number"
                  min="1"
                  step="10"
                  value={heightMm || ""}
                  onChange={(e) => setHeightMm(Number(e.target.value) || 0)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.frameWidth")}</span>
                <input
                  type="number"
                  min="1"
                  step="5"
                  value={frameWidthMm || ""}
                  onChange={(e) => setFrameWidthMm(Number(e.target.value) || 0)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.paneColumns")}</span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={paneColumns || ""}
                  onChange={(e) => setPaneColumns(Number(e.target.value) || 0)}
                  className={inputClass}
                />
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.paneRows")}</span>
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={paneRows || ""}
                  onChange={(e) => setPaneRows(Number(e.target.value) || 0)}
                  className={inputClass}
                />
              </label>
            </div>
            {(errorFor("width_mm") ||
              errorFor("height_mm") ||
              errorFor("frame_width_mm") ||
              errorFor("pane_columns") ||
              errorFor("pane_rows")) && (
              <div className="border-t border-[var(--oaec-border)] px-4 py-2 text-xs text-red-400">
                {errorFor("width_mm") ??
                  errorFor("height_mm") ??
                  errorFor("frame_width_mm") ??
                  errorFor("pane_columns") ??
                  errorFor("pane_rows")}
              </div>
            )}
          </div>

          {/* U-waarden (glas + profiel) */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                {t("uw.groupMaterials")}
              </h3>
            </div>
            <div className="grid grid-cols-2 gap-3 px-4 py-3">
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.uG")}</span>
                <input
                  type="number"
                  min="0"
                  step="0.1"
                  value={uG || ""}
                  onChange={(e) => setUG(Number(e.target.value) || 0)}
                  className={inputClass}
                />
                <span className="text-2xs text-scaffold-gray">
                  {t("uw.hints.uG")}
                </span>
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.uF")}</span>
                <input
                  type="number"
                  min="0"
                  step="0.1"
                  value={uF || ""}
                  onChange={(e) => setUF(Number(e.target.value) || 0)}
                  className={inputClass}
                />
                <span className="text-2xs text-scaffold-gray">
                  {t("uw.hints.uF")}
                </span>
              </label>
            </div>
          </div>

          {/* Beglazingsrand Ψ_g */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                {t("uw.groupSpacer")}
              </h3>
              <label className="flex items-center gap-2 text-xs text-on-surface-muted">
                <input
                  type="checkbox"
                  checked={psiGIsManual}
                  onChange={(e) => handleToggleManual(e.target.checked)}
                  className="rounded border-[var(--oaec-border)]"
                />
                {t("uw.manualPsiG")}
              </label>
            </div>
            <div className="grid grid-cols-2 gap-3 px-4 py-3">
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.spacer")}</span>
                <select
                  value={spacer}
                  disabled={psiGIsManual}
                  onChange={(e) => handleSpacerChange(e.target.value as Spacer)}
                  className={`${inputClass} disabled:opacity-50`}
                >
                  {SPACER_ORDER.map((s) => (
                    <option key={s} value={s}>
                      {SPACER_LABELS_NL[s]} ({"Ψ_g"}={spacerPsiG(s)})
                    </option>
                  ))}
                </select>
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>{t("uw.fields.psiG")}</span>
                <input
                  type="number"
                  min="0"
                  step="0.01"
                  value={
                    psiGIsManual
                      ? psiGManual || ""
                      : (spacerPsiG(spacer) ?? 0)
                  }
                  disabled={!psiGIsManual}
                  onChange={(e) => setPsiGManual(Number(e.target.value) || 0)}
                  className={`${inputClass} disabled:opacity-50`}
                />
                <span className="text-2xs text-scaffold-gray">
                  {psiGIsManual
                    ? t("uw.hints.psiGManual")
                    : t("uw.hints.psiGTable")}
                </span>
              </label>
            </div>
          </div>

          {/* Resultaat — live U_w + geometrie */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
            {result ? (
              <div className="space-y-2">
                <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm">
                  <span className="text-on-surface-muted">
                    A<sub>g</sub> ={" "}
                    <strong className="text-on-surface tabular-nums">
                      {geometry.a_g_m2.toFixed(4)}
                    </strong>{" "}
                    m{"²"}
                  </span>
                  <span className="text-on-surface-muted">
                    A<sub>f</sub> ={" "}
                    <strong className="text-on-surface tabular-nums">
                      {geometry.a_f_m2.toFixed(4)}
                    </strong>{" "}
                    m{"²"}
                  </span>
                  <span className="text-on-surface-muted">
                    l<sub>g</sub> ={" "}
                    <strong className="text-on-surface tabular-nums">
                      {geometry.l_g_m.toFixed(3)}
                    </strong>{" "}
                    m
                  </span>
                  <span className="text-on-surface-muted">
                    {"Ψ"}
                    <sub>g</sub> ={" "}
                    <strong className="text-on-surface tabular-nums">
                      {effectivePsiG.toFixed(3)}
                    </strong>{" "}
                    W/(m{"·"}K)
                  </span>
                </div>
                <div className="flex items-baseline gap-2 text-base">
                  <span className="text-on-surface-muted">
                    U<sub>w</sub> =
                  </span>
                  <strong className="text-xl text-primary tabular-nums">
                    {result.u_w.toFixed(3)}
                  </strong>
                  <span className="text-on-surface-muted text-sm">
                    W/(m{"²"}
                    {"·"}K)
                  </span>
                </div>
                <p className="text-2xs text-scaffold-gray">
                  {t("uw.normRef")}
                </p>
              </div>
            ) : (
              <div className="text-sm text-red-400">
                {errors[0]?.message ?? t("uw.invalidInput")}
              </div>
            )}

            {/* Acties — rapport + (in edit-modus) opslaan */}
            <div className="mt-3 flex items-center justify-end gap-2 border-t border-[var(--oaec-border)] pt-3">
              {saved && (
                <span className="text-xs text-green-400">
                  {t("uw.savedFeedback")}
                </span>
              )}
              <Button
                variant="secondary"
                onClick={handleGenerateReport}
                disabled={isGenerating || !result}
                size="md"
              >
                {isGenerating ? t("uw.generating") : t("uw.report")}
              </Button>
              {isEditMode && (
                <Button onClick={handleSave} disabled={!result} size="md">
                  {t("uw.save")}
                </Button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
