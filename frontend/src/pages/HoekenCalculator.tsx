/**
 * Hoeken-omrekentool — losse tool (route `/tools/hoeken`).
 *
 * Rekent een hoek om tussen drie gangbare bouwkundige notaties: graden,
 * hellingspercentage en verhouding `1:n`. Pure wiskundige tan-relatie
 * (`lib/hoekenCalculation.ts`), GEEN norm nodig — anders dan de
 * HWA-/hellingbaan-tools is er hier geen `SourcedValue`-bronvoetnoot.
 *
 * State is bewust lokaal (`useState`, geen store): de invoer is vluchtig,
 * er is niets om tussen sessies te bewaren — zelfde overweging als
 * `DoorGapCalculator.tsx`. Vier tekst-inputs (graden, procent, verhouding,
 * afschot mm/m) houden hun eigen "ruwe" getypte string vast zodat
 * tussentijdse invoer (bv. `"4,"`) niet wordt teruggezet; bij een geldig
 * getal worden de andere drie velden herrekend vanuit dát veld (geen
 * circulaire updates, want de bron-waarde wordt zelf nooit herschreven
 * vanuit de afgeleiden).
 */
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Card } from "../components/ui/Card";
import { PageHeader } from "../components/layout/PageHeader";
import { formatDecimals } from "../lib/formatNumber";
import {
  gradenNaarMmPerM,
  gradenNaarProcent,
  gradenNaarVerhouding,
  mmPerMNaarGraden,
  mmPerMNaarProcent,
  mmPerMNaarVerhouding,
  procentNaarGraden,
  procentNaarMmPerM,
  procentNaarVerhouding,
  verhoudingNaarProcent,
} from "../lib/hoekenCalculation";

const inputClass =
  "rounded-md border border-[var(--oaec-border)] bg-[var(--oaec-bg-input)] px-3 py-1.5 text-sm text-on-surface focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary";

/** Eén rij van de overzichtstabel — canonieke waarde in graden óf procent, context puur informatief. */
interface OverzichtRij {
  key: string;
  contextKey: string;
  graden: number;
  procent: number;
  verhoudingN: number;
  mmPerM: number;
}

function rijVanProcent(key: string, contextKey: string, procent: number): OverzichtRij {
  return {
    key,
    contextKey,
    procent,
    graden: procentNaarGraden(procent),
    verhoudingN: procentNaarVerhouding(procent),
    mmPerM: procentNaarMmPerM(procent),
  };
}

function rijVanGraden(key: string, contextKey: string, graden: number): OverzichtRij {
  return {
    key,
    contextKey,
    graden,
    procent: gradenNaarProcent(graden),
    verhoudingN: gradenNaarVerhouding(graden),
    mmPerM: gradenNaarMmPerM(graden),
  };
}

const OVERZICHT_RIJEN: ReadonlyArray<OverzichtRij> = [
  rijVanProcent("verhouding1_20", "toegankelijkheid", 5), // 1:20
  rijVanProcent("verhouding1_16", "toegankelijkheid", 6.25), // 1:16
  rijVanProcent("verhouding1_12", "drempel", 100 / 12), // 1:12 = 8,333%
  rijVanProcent("procent10", "garage", 10),
  rijVanProcent("procent14", "garage", 14),
  rijVanProcent("procent15", "garage", 15),
  rijVanProcent("procent20", "garage", 20),
  rijVanProcent("procent24", "garage", 24),
  rijVanProcent("afschotPlatDak", "platdak", 1.6), // 16 mm/m
  rijVanGraden("dak15", "dak", 15),
  rijVanGraden("dak30", "dak", 30),
  rijVanGraden("dak45", "dak", 45),
  rijVanGraden("dak60", "dak", 60),
];

/** Formatteer `n` uit `1:n` — bij (bijna) 0% is n heel groot/Infinity, tonen als "∞". */
function formatVerhouding(n: number): string {
  if (!Number.isFinite(n)) return "∞";
  return formatDecimals(n, n < 10 ? 2 : 1);
}

/** Formatteer een afschot in mm/m — boven 100 mm/m (steile dakhellingen) zonder decimalen. */
function formatMmPerM(mmPerM: number): string {
  if (!Number.isFinite(mmPerM)) return "∞";
  return formatDecimals(mmPerM, mmPerM >= 100 ? 0 : 1);
}

type Bron = "graden" | "procent" | "verhouding" | "mmPerM";

export function HoekenCalculator() {
  const { t } = useTranslation();

  // Elk veld heeft zijn eigen "ruwe" getypte string zodat je bv. "4," kunt
  // intypen zonder dat het veld meteen wordt teruggezet naar de laatst
  // geldige waarde.
  const [gradenInput, setGradenInput] = useState("30");
  const [procentInput, setProcentInput] = useState(formatDecimals(gradenNaarProcent(30), 3));
  const [verhoudingInput, setVerhoudingInput] = useState(
    formatVerhouding(gradenNaarVerhouding(30)),
  );
  const [mmPerMInput, setMmPerMInput] = useState(formatMmPerM(gradenNaarMmPerM(30)));
  const [gradenGetekend, setGradenGetekend] = useState(30);
  const [foutmelding, setFoutmelding] = useState<string | null>(null);

  /** Vul alle vier de velden vanuit een consistente hoek in graden (klik op tabelrij, of geldige invoer). */
  function zetAlleVeldenVanuit(
    bron: Bron,
    graden: number,
    procent: number,
    verhoudingN: number,
    mmPerM: number,
  ) {
    if (bron !== "graden") setGradenInput(formatDecimals(graden, 4));
    if (bron !== "procent") setProcentInput(formatDecimals(procent, 3));
    if (bron !== "verhouding") setVerhoudingInput(formatVerhouding(verhoudingN));
    if (bron !== "mmPerM") setMmPerMInput(formatMmPerM(mmPerM));
    setGradenGetekend(graden);
    setFoutmelding(null);
  }

  function handleGradenChange(raw: string) {
    setGradenInput(raw);
    const n = parseFloat(raw.replace(",", "."));
    if (!Number.isFinite(n)) return;
    try {
      const procent = gradenNaarProcent(n);
      const verhoudingN = procentNaarVerhouding(procent);
      const mmPerM = procentNaarMmPerM(procent);
      zetAlleVeldenVanuit("graden", n, procent, verhoudingN, mmPerM);
    } catch {
      setFoutmelding(t("hoeken.errorRange"));
    }
  }

  function handleProcentChange(raw: string) {
    setProcentInput(raw);
    const n = parseFloat(raw.replace(",", "."));
    if (!Number.isFinite(n)) return;
    try {
      const graden = procentNaarGraden(n);
      const verhoudingN = procentNaarVerhouding(n);
      const mmPerM = procentNaarMmPerM(n);
      zetAlleVeldenVanuit("procent", graden, n, verhoudingN, mmPerM);
    } catch {
      setFoutmelding(t("hoeken.errorRange"));
    }
  }

  function handleVerhoudingChange(raw: string) {
    setVerhoudingInput(raw);
    const n = parseFloat(raw.replace(",", "."));
    if (!Number.isFinite(n)) return;
    try {
      const procent = verhoudingNaarProcent(n);
      const graden = procentNaarGraden(procent);
      const mmPerM = procentNaarMmPerM(procent);
      zetAlleVeldenVanuit("verhouding", graden, procent, n, mmPerM);
    } catch {
      setFoutmelding(t("hoeken.errorRange"));
    }
  }

  function handleMmPerMChange(raw: string) {
    setMmPerMInput(raw);
    const n = parseFloat(raw.replace(",", "."));
    if (!Number.isFinite(n)) return;
    try {
      const procent = mmPerMNaarProcent(n);
      const graden = mmPerMNaarGraden(n);
      const verhoudingN = mmPerMNaarVerhouding(n);
      zetAlleVeldenVanuit("mmPerM", graden, procent, verhoudingN, n);
    } catch {
      setFoutmelding(t("hoeken.errorRange"));
    }
  }

  function handleRijKlik(rij: OverzichtRij) {
    setGradenInput(formatDecimals(rij.graden, 4));
    setProcentInput(formatDecimals(rij.procent, 3));
    setVerhoudingInput(formatVerhouding(rij.verhoudingN));
    setMmPerMInput(formatMmPerM(rij.mmPerM));
    setGradenGetekend(rij.graden);
    setFoutmelding(null);
  }

  return (
    <div>
      <PageHeader title={t("hoeken.title")} subtitle={t("hoeken.subtitle")} />

      <div className="mx-auto max-w-4xl space-y-4 p-6">
        {/* Converter */}
        <Card title={t("hoeken.converterTitle")}>
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
            <div className="grid grid-cols-1 gap-4">
              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("hoeken.graden")} <span className="font-normal text-on-surface-muted">[°]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={gradenInput}
                  onChange={(e) => handleGradenChange(e.target.value)}
                  className={inputClass}
                />
              </label>

              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("hoeken.procent")} <span className="font-normal text-on-surface-muted">[%]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={procentInput}
                  onChange={(e) => handleProcentChange(e.target.value)}
                  className={inputClass}
                />
              </label>

              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">{t("hoeken.verhouding")}</span>
                <div className="flex items-center gap-2">
                  <span className="text-on-surface-muted">1&nbsp;:</span>
                  <input
                    type="text"
                    inputMode="decimal"
                    value={verhoudingInput}
                    onChange={(e) => handleVerhoudingChange(e.target.value)}
                    className={`${inputClass} flex-1`}
                  />
                </div>
              </label>

              <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-on-surface">
                  {t("hoeken.mmPerM")} <span className="font-normal text-on-surface-muted">[mm/m]</span>
                </span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={mmPerMInput}
                  onChange={(e) => handleMmPerMChange(e.target.value)}
                  className={inputClass}
                />
              </label>

              {foutmelding && <p className="text-xs oa-warning-text">⚠ {foutmelding}</p>}
            </div>

            <div className="flex items-center justify-center">
              <HoekDiagram graden={gradenGetekend} t={t} />
            </div>
          </div>
        </Card>

        {/* Overzichtstabel */}
        <Card title={t("hoeken.overzichtTitle")}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--oaec-border)] text-left text-xs font-semibold uppercase tracking-wider text-on-surface-muted">
                  <th className="w-24 pb-2">{t("hoeken.colGraden")}</th>
                  <th className="w-24 pb-2 text-right">{t("hoeken.colProcent")}</th>
                  <th className="w-24 pb-2 pl-6 text-right">{t("hoeken.colVerhouding")}</th>
                  <th className="w-24 pb-2 pl-6 text-right">{t("hoeken.colMmPerM")}</th>
                  <th className="pb-2 pl-6">{t("hoeken.colContext")}</th>
                </tr>
              </thead>
              <tbody>
                {OVERZICHT_RIJEN.map((rij) => (
                  <tr
                    key={rij.key}
                    onClick={() => handleRijKlik(rij)}
                    className="cursor-pointer border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]"
                    title={t("hoeken.rowClickHint")}
                  >
                    <td className="w-24 py-1.5 tabular-nums text-on-surface-secondary">
                      {formatDecimals(rij.graden, 2)}°
                    </td>
                    <td className="w-24 py-1.5 text-right tabular-nums text-on-surface-secondary">
                      {formatDecimals(rij.procent, 2)}%
                    </td>
                    <td className="w-24 py-1.5 pl-6 text-right tabular-nums text-on-surface-secondary">
                      1:{formatVerhouding(rij.verhoudingN)}
                    </td>
                    <td className="w-24 py-1.5 pl-6 text-right tabular-nums text-on-surface-secondary">
                      {formatMmPerM(rij.mmPerM)}
                    </td>
                    <td className="py-1.5 pl-6 text-xs text-on-surface-muted">
                      {t(`hoeken.context.${rij.contextKey}`)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        <p className="text-xs text-on-surface-muted">{t("hoeken.disclaimer")}</p>
      </div>
    </div>
  );
}

/**
 * Inline SVG-hoekdiagram: rechthoekige driehoek die met de hoek meedraait,
 * met een boog die de hoek markeert. Kleuren via theme-vars — geen
 * hardcoded amber/orange, zelfde stijl-conventie als
 * `HellingbaanCalculator.tsx`.
 */
function HoekDiagram({ graden, t }: { graden: number; t: (key: string) => string }) {
  const width = 220;
  const height = 180;
  const padding = 20;

  // Clamp voor de tekening zelf (visuele leesbaarheid) — de rekenkern kent
  // deze clamp niet, dit is puur om de SVG binnen het kader te houden.
  const gradenGetekend = Math.min(Math.max(graden, 0), 85);
  const rad = (gradenGetekend * Math.PI) / 180;

  const baseLength = width - 2 * padding;
  const startX = padding;
  const startY = height - padding;
  const topX = startX + baseLength;
  const topY = startY - baseLength * Math.tan(rad);

  // Boog bij de hoek (linksonder), straal klein en vast.
  const arcRadius = 28;
  const arcEndX = startX + arcRadius * Math.cos(rad);
  const arcEndY = startY - arcRadius * Math.sin(rad);

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="h-auto w-full max-w-[220px]" role="img" aria-label={t("hoeken.diagramTitle")}>
      {/* Driehoek-vlak */}
      <polygon
        points={`${startX},${startY} ${topX},${startY} ${topX},${topY}`}
        fill="var(--theme-hover-strong)"
        stroke="none"
      />

      {/* Basis (horizontaal) */}
      <line x1={startX} y1={startY} x2={topX} y2={startY} stroke="var(--oaec-border)" strokeWidth={1.5} />
      {/* Verticale zijde */}
      <line x1={topX} y1={startY} x2={topX} y2={topY} stroke="var(--oaec-border)" strokeWidth={1.5} strokeDasharray="4 3" />
      {/* Hellingzijde (schuine kant) — de hoek zelf */}
      <line
        x1={startX}
        y1={startY}
        x2={topX}
        y2={topY}
        stroke="var(--theme-accent)"
        strokeWidth={4}
        strokeLinecap="round"
      />

      {/* Hoek-boog */}
      <path
        d={`M ${startX + arcRadius} ${startY} A ${arcRadius} ${arcRadius} 0 0 0 ${arcEndX} ${arcEndY}`}
        fill="none"
        stroke="var(--theme-btn-primary-bg)"
        strokeWidth={2}
      />
      <text x={startX + arcRadius + 8} y={startY - 8} fontSize={11} fill="var(--oaec-text-secondary)">
        {formatDecimals(graden, 1)}°
      </text>
    </svg>
  );
}
