/**
 * Hellingbaan (parkeergarage) — losse tool (route `/tools/hellingbaan`).
 *
 * Dimensioneert een parkeergarage-hellingbaan volgens NEN 2443, via de
 * rekenkern in `lib/hellingbaanCalculation.ts` (frontend-only, 1-op-1
 * geport uit de bestaande pyRevit-tool, GEEN Rust/API/Revit-geometrie).
 * State leeft in `store/hellingbaanStore.ts` (persisted), dus de tool werkt
 * ook zonder geopend project — zelfde patroon als `HwaCalculator.tsx`.
 *
 * Structuur: invoer-kaart, resultaat-kaart (segmenten-tabel + totale
 * lengte + vergelijking zonder optimalisatie), een inline SVG-zijaanzicht,
 * en een bronvoetnoot die de `SourcedValue.reference`-teksten rechtstreeks
 * uit de rekenkern-constanten haalt.
 */
import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { formatDecimals } from "../lib/formatNumber";
import {
  GARAGE_TYPES,
  LEN_MAX_MM,
  LEN_MIN_MM,
  WIELBASIS_MM,
  calculateHellingbaan,
  calculateHellingbaanReferentie,
  getGarageType,
  isReferentieNormConform,
} from "../lib/hellingbaanCalculation";
import { useHellingbaanStore } from "../store/hellingbaanStore";
import type {
  HellingbaanGarageType,
  HellingbaanGarageTypeId,
  HellingbaanResult,
  HellingbaanSegmentType,
} from "../types/hellingbaan";

const inputClass =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

const GARAGE_TYPE_LABEL_KEYS: Record<HellingbaanGarageTypeId, string> = {
  openbaar: "hellingbaan.garageTypeOpenbaar",
  openbaar_dhumy: "hellingbaan.garageTypeOpenbaarDhumy",
  niet_openbaar: "hellingbaan.garageTypeNietOpenbaar",
  stalling: "hellingbaan.garageTypeStalling",
};

const SEGMENT_LABEL_KEYS: Record<HellingbaanSegmentType, string> = {
  overgang_onder: "hellingbaan.segmentOvergangOnder",
  hoofd: "hellingbaan.segmentHoofd",
  overgang_boven: "hellingbaan.segmentOvergangBoven",
  enkel: "hellingbaan.segmentEnkel",
};

/** Zone-label — "Handmatig" heeft voorrang op de zone-naam zodra er een geldige override actief is. */
function zoneLabel(t: TFunction, result: HellingbaanResult, garageType: HellingbaanGarageType): string {
  if (result.isOverride) return t("hellingbaan.zoneHandmatig");
  switch (result.zone) {
    case "vast":
      return t("hellingbaan.zoneVast", { value: garageType.maxHellingPercent });
    case "kort":
      return t("hellingbaan.zoneKort", { value: garageType.maxHellingPercent });
    case "lang":
      return t("hellingbaan.zoneLang", { value: garageType.minHellingPercent });
    case "simpel":
      return t("hellingbaan.zoneSimpel");
    default:
      return t("hellingbaan.zoneMidden");
  }
}

export function HellingbaanCalculator() {
  const { t } = useTranslation();

  const hoogteMm = useHellingbaanStore((s) => s.hoogteMm);
  const garageTypeId = useHellingbaanStore((s) => s.garageTypeId);
  const metOvergang = useHellingbaanStore((s) => s.metOvergang);
  const breedteMm = useHellingbaanStore((s) => s.breedteMm);
  const hellingOverridePercent = useHellingbaanStore((s) => s.hellingOverridePercent);
  const setHoogteMm = useHellingbaanStore((s) => s.setHoogteMm);
  const setGarageTypeId = useHellingbaanStore((s) => s.setGarageTypeId);
  const setMetOvergang = useHellingbaanStore((s) => s.setMetOvergang);
  const setBreedteMm = useHellingbaanStore((s) => s.setBreedteMm);
  const setHellingOverridePercent = useHellingbaanStore((s) => s.setHellingOverridePercent);

  const garageType = useMemo(() => getGarageType(garageTypeId), [garageTypeId]);

  const result = useMemo(
    () =>
      calculateHellingbaan({
        hoogteMm,
        garageTypeId,
        metOvergang,
        breedteMm,
        hellingOverridePercent,
      }),
    [hoogteMm, garageTypeId, metOvergang, breedteMm, hellingOverridePercent],
  );

  const referentie = useMemo(
    () => calculateHellingbaanReferentie({ hoogteMm, garageTypeId, metOvergang }),
    [hoogteMm, garageTypeId, metOvergang],
  );

  const besparingMm = referentie.lengteTotaalMm - result.lengteTotaalMm;
  // De "zonder optimalisatie"-vergelijking (vaste max-helling) is alleen
  // norm-conform in de kort/vast/simpel-zone — in midden/lang verlangt de
  // norm juist een minder steile helling, zie isReferentieNormConform.
  const referentieNormConform = isReferentieNormConform(result.zone);

  return (
    <div>
      <PageHeader title={t("hellingbaan.title")} subtitle={t("hellingbaan.subtitle")} />

      <div className="mx-auto max-w-4xl space-y-4 p-6">
        {/* Invoer */}
        <Card title={t("hellingbaan.inputTitle")}>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("hellingbaan.hoogte")}{" "}
                <span className="font-normal text-on-surface-muted">[mm]</span>
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={hoogteMm}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  setHoogteMm(Number.isFinite(n) && n >= 0 ? n : 0);
                }}
                className={inputClass}
              />
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">{t("hellingbaan.garageType")}</span>
              <select
                value={garageTypeId}
                onChange={(e) => setGarageTypeId(e.target.value as HellingbaanGarageTypeId)}
                className={inputClass}
              >
                {GARAGE_TYPES.value.map((gt) => (
                  <option key={gt.id} value={gt.id}>
                    {t(GARAGE_TYPE_LABEL_KEYS[gt.id])}
                  </option>
                ))}
              </select>
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("hellingbaan.breedte")}{" "}
                <span className="font-normal text-on-surface-muted">[mm]</span>
              </span>
              <input
                type="number"
                min={0}
                step="any"
                value={breedteMm}
                onChange={(e) => {
                  const n = parseFloat(e.target.value);
                  setBreedteMm(Number.isFinite(n) && n >= 0 ? n : 0);
                }}
                className={inputClass}
              />
              <span
                className={`text-xs ${result.isBreedteOnderMinimum ? "oa-warning-text" : "text-on-surface-muted"}`}
              >
                {t("hellingbaan.breedteMinHint", { value: garageType.breedteMinMm })}
              </span>
            </label>

            <label className="flex flex-col gap-1 text-sm">
              <span className="font-medium text-on-surface">
                {t("hellingbaan.hellingOverride")}{" "}
                <span className="font-normal text-on-surface-muted">[%]</span>
              </span>
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min={0}
                  step="any"
                  value={hellingOverridePercent ?? ""}
                  placeholder={t("hellingbaan.hellingOverridePlaceholder")}
                  onChange={(e) => {
                    const raw = e.target.value;
                    if (raw === "") {
                      setHellingOverridePercent(undefined);
                      return;
                    }
                    const n = parseFloat(raw);
                    setHellingOverridePercent(Number.isFinite(n) ? n : undefined);
                  }}
                  className={inputClass}
                />
                {hellingOverridePercent !== undefined && (
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => setHellingOverridePercent(undefined)}
                  >
                    {t("hellingbaan.resetOverride")}
                  </Button>
                )}
              </div>
            </label>

            <label className="flex items-center gap-2 text-sm sm:col-span-2">
              <input
                type="checkbox"
                checked={metOvergang}
                onChange={(e) => setMetOvergang(e.target.checked)}
                className="accent-[var(--oaec-primary,#2563eb)]"
              />
              <span className="font-medium text-on-surface">{t("hellingbaan.metOvergang")}</span>
            </label>
          </div>
        </Card>

        {/* Resultaat */}
        <Card title={t("hellingbaan.resultTitle")}>
          <div className="space-y-3 text-sm">
            <ResultRow label={t("hellingbaan.zone")} value={zoneLabel(t, result, garageType)} />

            {/* Segmenten-tabel */}
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                    <th className="pb-2">{t("hellingbaan.segmentType")}</th>
                    <th className="pb-2 text-right">{t("hellingbaan.segmentLength")}</th>
                    <th className="pb-2 text-right">{t("hellingbaan.segmentSlope")}</th>
                    <th className="pb-2 text-right">{t("hellingbaan.segmentSlopeDeg")}</th>
                    <th className="pb-2 text-right">{t("hellingbaan.segmentHeight")}</th>
                  </tr>
                </thead>
                <tbody>
                  {result.segments.map((seg, i) => (
                    <tr key={i} className="border-b border-[var(--oaec-border-subtle)]">
                      <td className="py-1.5 text-on-surface-secondary">
                        {t(SEGMENT_LABEL_KEYS[seg.type])}
                      </td>
                      <td className="py-1.5 text-right tabular-nums">
                        {formatDecimals(seg.lengteMm, 0)} mm
                      </td>
                      <td className="py-1.5 text-right tabular-nums">
                        {formatDecimals(seg.hellingPercent, 1)}%
                      </td>
                      <td className="py-1.5 text-right tabular-nums">
                        {formatDecimals(seg.hellingGraden, 1)}°
                      </td>
                      <td className="py-1.5 text-right tabular-nums">
                        {formatDecimals(seg.hoogteMm, 0)} mm
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <ResultRow
              label={t("hellingbaan.totalLength")}
              value={`${formatDecimals(result.lengteTotaalMm / 1000, 2)} m (${formatDecimals(result.lengteTotaalMm, 0)} mm)`}
              emphasized
            />

            {/* Vergelijking zonder optimalisatie */}
            <div className="mt-2 rounded-md bg-surface-alt p-2.5">
              <ResultRow
                label={t("hellingbaan.comparisonLength", { value: referentie.hellingPercent })}
                value={`${formatDecimals(referentie.lengteTotaalMm / 1000, 2)} m`}
              />
              {/* Norm-context: de vaste max-helling is buiten de kort/vast/
                  simpel-zone GEEN toegestaan alternatief (zie
                  isReferentieNormConform) — expliciet markeren zodat deze
                  regel niet als geldig alternatief gelezen wordt. */}
              {!referentieNormConform && (
                <p className="mt-1 text-xs oa-warning-text">
                  ⚠ {t("hellingbaan.comparisonNotNormConform", { value: LEN_MIN_MM.value / 1000 })}
                </p>
              )}
              {/* Richting van het verschil is niet altijd "winst" — de
                  zone-optimalisatie kiest bij midden/lang-hoogtes bewust een
                  minder steile helling dan de type-max, wat MEER lengte
                  vraagt dan de (steilere) vaste max-helling. Alleen bij een
                  override die steiler is dan de type-max, of in de
                  kort-zone (gelijk), levert de vergelijking winst op. */}
              {besparingMm > 0.5 ? (
                <p className="mt-1 text-xs text-on-surface-muted">
                  {t("hellingbaan.comparisonSaving", { value: formatDecimals(besparingMm, 0) })}
                </p>
              ) : besparingMm < -0.5 ? (
                <p className="mt-1 text-xs text-on-surface-muted">
                  {t("hellingbaan.comparisonExtra", {
                    value: formatDecimals(Math.abs(besparingMm), 0),
                    helling: formatDecimals(result.hellingPercent, 1),
                    maxHelling: garageType.maxHellingPercent,
                  })}
                </p>
              ) : (
                <p className="mt-1 text-xs text-on-surface-muted">
                  {t("hellingbaan.comparisonNone")}
                </p>
              )}
            </div>

            {result.warnings.length > 0 && (
              <ul className="mt-2 space-y-1 text-xs oa-warning-text">
                {result.warnings.map((w, i) => (
                  <li key={i}>⚠ {w}</li>
                ))}
              </ul>
            )}
          </div>
        </Card>

        {/* Zijaanzicht */}
        <Card title={t("hellingbaan.diagramTitle")}>
          <HellingbaanDiagram result={result} hoogteMm={hoogteMm} t={t} />
        </Card>

        {/* Bronvoetnoot — teksten rechtstreeks uit de rekenkern-constanten */}
        <div className="space-y-1 text-xs text-on-surface-muted">
          <p>{t("hellingbaan.sourceIntro")}</p>
          <ul className="list-inside list-disc space-y-0.5">
            <li>
              {t("hellingbaan.sourceWielbasis")}: {WIELBASIS_MM.reference}
            </li>
            <li>
              {t("hellingbaan.sourceGarageTypes")}: {GARAGE_TYPES.reference}
            </li>
            <li>
              {t("hellingbaan.sourceLenMinMax")}: {LEN_MIN_MM.reference} / {LEN_MAX_MM.reference}
            </li>
          </ul>
          <p className="pt-1">{t("hellingbaan.sourceDiscrepancyNote")}</p>
        </div>
      </div>
    </div>
  );
}

/** Label + waarde-regel (tabular-nums voor uitlijning) — zelfde patroon als HwaCalculator/DoorGapCalculator. */
function ResultRow({
  label,
  value,
  emphasized = false,
}: {
  label: string;
  value: string;
  emphasized?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-on-surface-muted">{label}</span>
      <span
        className={`tabular-nums ${emphasized ? "font-semibold text-on-surface" : "text-on-surface-secondary"}`}
      >
        {value}
      </span>
    </div>
  );
}

/**
 * Inline SVG-zijaanzicht: schematisch profiel van de segmenten (voet-
 * overgang, hoofdhelling, top-overgang bij `metOvergang`, anders één rechte
 * lijn) met maatvoering voor totale lengte en hoogteverschil. Kleuren via
 * theme-vars (petrol/teal voor de segmenten, mint als vlak onder het
 * profiel) — geen hardcoded kleuren, zodat het diagram met het thema
 * meekleurt.
 */
function HellingbaanDiagram({
  result,
  hoogteMm,
  t,
}: {
  result: HellingbaanResult;
  hoogteMm: number;
  t: TFunction;
}) {
  const width = 640;
  const height = 220;
  const padding = 44;

  const totalLengthMm = Math.max(result.lengteTotaalMm, 1);
  const totalHeightMm = Math.max(hoogteMm, 1);

  const drawWidth = width - 2 * padding;
  const drawHeight = height - 2 * padding - 24;
  const scale = Math.min(drawWidth / totalLengthMm, drawHeight / totalHeightMm) * 0.95;

  const startX = padding;
  const startY = height - padding;

  const points: Array<{ x: number; y: number }> = [{ x: startX, y: startY }];
  let cumLengthMm = 0;
  let cumHeightMm = 0;
  for (const seg of result.segments) {
    cumLengthMm += seg.lengteMm;
    cumHeightMm += seg.hoogteMm;
    points.push({ x: startX + cumLengthMm * scale, y: startY - cumHeightMm * scale });
  }

  const topPoint = points[points.length - 1]!;
  const groundPolygon = [...points, { x: topPoint.x, y: startY }, { x: startX, y: startY }]
    .map((p) => `${p.x},${p.y}`)
    .join(" ");

  const segmentColor = (type: HellingbaanSegmentType) =>
    type === "hoofd" || type === "enkel" ? "var(--theme-accent)" : "var(--theme-btn-primary-bg)";

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      className="h-auto w-full"
      role="img"
      aria-label={t("hellingbaan.diagramTitle")}
    >
      {/* Vlak onder het profiel (mint-tint) */}
      <polygon points={groundPolygon} fill="var(--theme-hover-strong)" stroke="none" />

      {/* Vloerlijnen (stippel) */}
      <line
        x1={padding - 10}
        y1={startY}
        x2={width - padding + 10}
        y2={startY}
        stroke="var(--oaec-border)"
        strokeDasharray="4 3"
      />
      <line
        x1={padding - 10}
        y1={topPoint.y}
        x2={width - padding + 10}
        y2={topPoint.y}
        stroke="var(--oaec-border)"
        strokeDasharray="4 3"
      />

      {/* Segmenten */}
      {result.segments.map((seg, i) => (
        <line
          key={i}
          x1={points[i]!.x}
          y1={points[i]!.y}
          x2={points[i + 1]!.x}
          y2={points[i + 1]!.y}
          stroke={segmentColor(seg.type)}
          strokeWidth={5}
          strokeLinecap="round"
        />
      ))}

      {/* Maatlijn — lengte */}
      <line
        x1={points[0]!.x}
        y1={startY + 16}
        x2={topPoint.x}
        y2={startY + 16}
        stroke="var(--oaec-text-muted)"
        strokeWidth={1}
      />
      <text
        x={(points[0]!.x + topPoint.x) / 2}
        y={startY + 32}
        textAnchor="middle"
        fontSize={11}
        fill="var(--oaec-text-secondary)"
      >
        {formatDecimals(result.lengteTotaalMm / 1000, 2)} m
      </text>

      {/* Maatlijn — hoogte */}
      <line
        x1={width - padding + 20}
        y1={startY}
        x2={width - padding + 20}
        y2={topPoint.y}
        stroke="var(--oaec-text-muted)"
        strokeWidth={1}
      />
      <text
        x={width - padding + 26}
        y={(startY + topPoint.y) / 2}
        fontSize={11}
        fill="var(--oaec-text-secondary)"
      >
        {formatDecimals(hoogteMm / 1000, 2)} m
      </text>
    </svg>
  );
}
