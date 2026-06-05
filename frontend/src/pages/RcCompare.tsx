import { useEffect, useMemo, useState } from "react";

import { GlaserDiagram } from "../components/construction/GlaserDiagram";
import { MoistureYearTable } from "../components/construction/MoistureYearTable";
import { PageHeader } from "../components/layout/PageHeader";
import {
  CATALOGUE_CATEGORY_LABELS,
  type CatalogueCategory,
  type CatalogueLayer,
} from "../lib/constructionCatalogue";
import { calculateGlaser, GLASER_DEFAULTS } from "../lib/glaserCalculation";
import {
  calculateRc,
  RC_MIN_BOUWBESLUIT,
  roundUValue,
  type LayerInput,
} from "../lib/rcCalculation";
import { calculateYearlyMoisture } from "../lib/yearlyMoistureCalculation";
import {
  CLIMATE_DEFAULT_SELECTION,
  CLIMATE_DEFAULT_STATION,
  getMonthlyClimate,
  listAvailableYears,
  listStations,
  type YearSelection,
} from "../lib/climateData";
import { useCatalogueStore } from "../store/catalogueStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import type { VerticalPosition } from "../types";

// ---------- Constanten ----------

/** Map catalogus-categorie → constructiepositie (kozijnen vallen af: geen Rc). */
const CATEGORY_POSITION: Record<string, VerticalPosition> = {
  wanden: "wall",
  vloeren_plafonds: "floor",
  daken: "ceiling",
};

// ---------- Klimaatselectie helpers (gespiegeld van RcCalculator) ----------

function selectionToValue(selection: YearSelection): string {
  return typeof selection === "number" ? `year:${selection}` : `key:${selection}`;
}

function valueToSelection(value: string): YearSelection {
  if (value.startsWith("year:")) {
    return Number(value.slice(5));
  }
  return value.slice(4) as YearSelection;
}

function selectionLabel(selection: YearSelection): string {
  if (selection === "1991-2020") return "1991-2020 (normaal)";
  if (selection === "NEN5060") return "NEN5060 (referentie)";
  return String(selection);
}

// ---------- Constructie-bron ----------

/**
 * Eén kiesbare constructie voor de vergelijker. Verenigt bibliotheek-entries
 * (catalogus) en projectconstructies tot één lijst van pickers-opties met de
 * laag-opbouw + positie die de rekenkern nodig heeft.
 */
interface PickableConstruction {
  /** Stabiele key: `lib:{id}` of `proj:{id}` (id's kunnen overlappen). */
  key: string;
  /** Weergavenaam in de dropdown. */
  name: string;
  /** Bron-label voor de optgroup. */
  source: "library" | "project";
  category: CatalogueCategory;
  position: VerticalPosition;
  layers: CatalogueLayer[];
}

/** Map een CatalogueLayer[] → LayerInput[] voor de rekenkern (incl. stud). */
function toLayerInputs(layers: CatalogueLayer[]): LayerInput[] {
  return layers.map((l) => ({
    materialId: l.materialId,
    thickness: l.thickness,
    lambdaOverride: l.lambdaOverride,
    stud: l.stud,
  }));
}

// ---------- Per-kolom afgeleide berekening ----------

interface ColumnComputed {
  position: VerticalPosition;
  rc: number;
  uValue: number;
  rcMin: number;
  meetsRequirement: boolean;
  hasCondensation: boolean;
  glaserResult: ReturnType<typeof calculateGlaser>;
  moistureResult: ReturnType<typeof calculateYearlyMoisture>;
  maxMa: number;
  wetDays: number;
  hasMoistureRisk: boolean;
}

/**
 * Bereken alle vergelijkingsmetrieken voor één gekozen constructie.
 *
 * Hergebruikt 1:1 de rekenkern uit RcCalculator: `calculateRc`,
 * `calculateGlaser` (steady-state, norm-vast −10 °C) en
 * `calculateYearlyMoisture` (gevoed door de gedeelde KNMI-klimaatkeuze).
 */
function computeColumn(
  layers: CatalogueLayer[],
  position: VerticalPosition,
  climate: ReturnType<typeof getMonthlyClimate>,
): ColumnComputed {
  const inputs = toLayerInputs(layers);

  const rcResult = calculateRc(inputs, position);

  const glaserResult = calculateGlaser({
    layers: layers.map((l) => ({
      materialId: l.materialId,
      thickness: l.thickness,
      stud: l.stud,
    })),
    position,
    thetaI: GLASER_DEFAULTS.thetaI,
    thetaE: GLASER_DEFAULTS.thetaE,
    rhI: GLASER_DEFAULTS.rhI,
    rhE: GLASER_DEFAULTS.rhE,
  });

  const moistureResult = calculateYearlyMoisture(
    layers.map((l) => ({ materialId: l.materialId, thickness: l.thickness })),
    position,
    GLASER_DEFAULTS.thetaI,
    GLASER_DEFAULTS.rhI,
    climate ?? undefined,
  );

  const rcMin = RC_MIN_BOUWBESLUIT[position];

  return {
    position,
    rc: rcResult.rc,
    uValue: rcResult.uValue,
    rcMin,
    meetsRequirement: rcResult.rc >= rcMin,
    hasCondensation: glaserResult.hasCondensation,
    glaserResult,
    moistureResult,
    maxMa: moistureResult?.maxMa ?? 0,
    wetDays: moistureResult?.wetDays ?? 0,
    hasMoistureRisk: moistureResult?.hasRisk ?? false,
  };
}

// ---------- Statusbadge ----------

function StatusBadge({
  tone,
  label,
}: {
  tone: "ok" | "warn" | "bad";
  label: string;
}) {
  const cls =
    tone === "bad"
      ? "bg-red-600/20 text-red-400"
      : tone === "warn"
        ? "bg-amber-600/15 text-amber-400"
        : "bg-green-600/15 text-green-400";
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${cls}`}
    >
      <span className="inline-block h-1.5 w-1.5 rounded-full bg-current" />
      {label}
    </span>
  );
}

// ---------- Kolom ----------

function CompareColumn({
  title,
  options,
  selectedKey,
  onSelect,
  computed,
}: {
  title: string;
  options: PickableConstruction[];
  selectedKey: string;
  onSelect: (key: string) => void;
  computed: ColumnComputed | null;
}) {
  const libraryOptions = options.filter((o) => o.source === "library");
  const projectOptions = options.filter((o) => o.source === "project");

  return (
    <div className="flex flex-col gap-4">
      {/* Picker */}
      <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
        <div className="mb-2 flex items-center justify-between">
          <h3 className="text-sm font-semibold text-on-surface-secondary">
            {title}
          </h3>
        </div>
        <select
          value={selectedKey}
          onChange={(e) => onSelect(e.target.value)}
          className="w-full rounded border border-[var(--oaec-border)] px-2.5 py-1.5 text-sm focus:border-primary focus:outline-none"
        >
          <option value="">Kies constructie…</option>
          {projectOptions.length > 0 && (
            <optgroup label="Projectconstructies">
              {projectOptions.map((o) => (
                <option key={o.key} value={o.key}>
                  {o.name}
                </option>
              ))}
            </optgroup>
          )}
          {libraryOptions.length > 0 && (
            <optgroup label="Bibliotheek">
              {libraryOptions.map((o) => (
                <option key={o.key} value={o.key}>
                  {CATALOGUE_CATEGORY_LABELS[o.category]} — {o.name}
                </option>
              ))}
            </optgroup>
          )}
        </select>
      </div>

      {/* Lege staat */}
      {!computed && (
        <div className="rounded-lg border border-dashed border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-10 text-center text-sm text-on-surface-muted">
          Kies een constructie om de Rc-, condensatie- en vochtanalyse te tonen.
        </div>
      )}

      {computed && (
        <>
          {/* Rc / U + Bouwbesluit */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
            <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm">
              <span className="text-on-surface-muted">
                Rc ={" "}
                <strong className="text-on-surface">
                  {computed.rc.toFixed(2)}
                </strong>{" "}
                m{"²"}K/W
              </span>
              <span className="text-on-surface-muted">
                U ={" "}
                <strong className="text-on-surface">
                  {roundUValue(computed.uValue).toFixed(3)}
                </strong>{" "}
                W/m{"²"}K
              </span>
            </div>
            <div className="mt-2 flex items-center gap-2">
              <span
                className={`inline-block h-2 w-2 rounded-full ${
                  computed.meetsRequirement ? "bg-green-600/100" : "bg-red-600/150"
                }`}
              />
              <span className="text-xs text-on-surface-muted">
                Bouwbesluit 2024: Rc {"≥"} {computed.rcMin} m{"²"}K/W
                {computed.meetsRequirement ? " ✔" : " ✘"}
              </span>
            </div>
          </div>

          {/* Glaser-oordeel + diagram */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                Glaser (steady-state, {GLASER_DEFAULTS.thetaE}°C)
              </h3>
              {computed.hasCondensation ? (
                <StatusBadge tone="bad" label="Condensatierisico" />
              ) : (
                <StatusBadge tone="ok" label="Geen condensatie" />
              )}
            </div>
            <div className="px-4 py-3">
              <GlaserDiagram
                result={computed.glaserResult}
                thetaI={GLASER_DEFAULTS.thetaI}
                thetaE={GLASER_DEFAULTS.thetaE}
              />
            </div>
          </div>

          {/* Jaarlijkse vochtbalans */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)]">
            <div className="flex items-center justify-between border-b border-[var(--oaec-border)] px-4 py-2.5">
              <h3 className="text-sm font-semibold text-on-surface-secondary">
                Jaarlijkse vochtbalans
              </h3>
              {computed.hasMoistureRisk ? (
                <StatusBadge tone="bad" label="Schimmelrisico" />
              ) : computed.maxMa > 0.1 ? (
                <StatusBadge tone="warn" label="Tijdelijk vocht" />
              ) : (
                <StatusBadge tone="ok" label="Geen vocht" />
              )}
            </div>
            <div className="px-4 py-3">
              <div className="mb-3 flex flex-wrap gap-x-6 gap-y-1 text-xs text-on-surface-muted">
                <span>
                  Max. ophoping:{" "}
                  <strong className="text-on-surface">
                    {computed.maxMa.toFixed(1)}
                  </strong>{" "}
                  g/m{"²"}
                </span>
                <span>
                  Natte dagen:{" "}
                  <strong className="text-on-surface">{computed.wetDays}</strong>
                </span>
              </div>
              {computed.moistureResult ? (
                <MoistureYearTable result={computed.moistureResult} />
              ) : (
                <p className="text-xs italic text-on-surface-muted">
                  Geen vochtbalans beschikbaar (onbekende materialen of geen
                  dampweerstand).
                </p>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// ---------- Vergelijk-samenvatting ----------

function DeltaSummary({
  a,
  b,
}: {
  a: ColumnComputed;
  b: ColumnComputed;
}) {
  const rcDelta = a.rc - b.rc;
  const rcHigher =
    Math.abs(rcDelta) < 0.005 ? "gelijk" : rcDelta > 0 ? "A" : "B";

  const condDiffers = a.hasCondensation !== b.hasCondensation;

  const maDelta = a.maxMa - b.maxMa;
  const maLower =
    Math.abs(maDelta) < 0.05 ? "gelijk" : maDelta < 0 ? "A" : "B";

  return (
    <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
      <h3 className="mb-2 text-sm font-semibold text-on-surface-secondary">
        Vergelijking A ↔ B
      </h3>
      <div className="grid gap-2 text-xs sm:grid-cols-3">
        <div className="rounded border border-[var(--oaec-border-subtle)] px-3 py-2">
          <div className="text-on-surface-muted">Hoogste Rc</div>
          <div className="mt-0.5 text-on-surface">
            {rcHigher === "gelijk"
              ? "Gelijk"
              : `Constructie ${rcHigher}`}{" "}
            <span className="text-on-surface-muted">
              ({"Δ"} {Math.abs(rcDelta).toFixed(2)} m{"²"}K/W)
            </span>
          </div>
        </div>
        <div className="rounded border border-[var(--oaec-border-subtle)] px-3 py-2">
          <div className="text-on-surface-muted">Condensatie (Glaser)</div>
          <div className="mt-0.5 text-on-surface">
            {!condDiffers
              ? a.hasCondensation
                ? "Beide: risico"
                : "Beide: vrij"
              : `A: ${a.hasCondensation ? "risico" : "vrij"} · B: ${
                  b.hasCondensation ? "risico" : "vrij"
                }`}
          </div>
        </div>
        <div className="rounded border border-[var(--oaec-border-subtle)] px-3 py-2">
          <div className="text-on-surface-muted">Laagste vochtophoping</div>
          <div className="mt-0.5 text-on-surface">
            {maLower === "gelijk"
              ? "Gelijk"
              : `Constructie ${maLower}`}{" "}
            <span className="text-on-surface-muted">
              ({"Δ"} {Math.abs(maDelta).toFixed(1)} g/m{"²"})
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

// ---------- Component ----------

export function RcCompare() {
  // Gekozen constructie per kolom (key uit PickableConstruction).
  const [keyA, setKeyA] = useState<string>("");
  const [keyB, setKeyB] = useState<string>("");

  // Gedeelde KNMI-klimaatkeuze (voedt de jaarbalans van BEIDE kolommen).
  // Default De Bilt / 1991-2020-normaal reproduceert het forfaitaire klimaat.
  const [climateStationId, setClimateStationId] = useState<string>(
    CLIMATE_DEFAULT_STATION,
  );
  const [climateSelection, setClimateSelection] = useState<YearSelection>(
    CLIMATE_DEFAULT_SELECTION,
  );

  const catalogueEntries = useCatalogueStore((s) => s.entries);
  const projectConstructions = useModellerStore((s) => s.projectConstructions);

  // Verenig bibliotheek + projectconstructies tot één pickerlijst. Alleen
  // entries mét laag-opbouw én een Rc-positie (wand/vloer/dak) zijn bruikbaar;
  // kozijnen/vullingen (geen layers) vallen af.
  const options = useMemo<PickableConstruction[]>(() => {
    const out: PickableConstruction[] = [];

    for (const pc of projectConstructions) {
      const position = CATEGORY_POSITION[pc.category];
      if (!position || pc.layers.length === 0) continue;
      out.push({
        key: `proj:${pc.id}`,
        name: pc.name,
        source: "project",
        category: pc.category,
        position,
        layers: pc.layers,
      });
    }

    for (const entry of catalogueEntries) {
      const position = CATEGORY_POSITION[entry.category];
      if (!position || !entry.layers || entry.layers.length === 0) continue;
      out.push({
        key: `lib:${entry.id}`,
        name: entry.name,
        source: "library",
        category: entry.category,
        position,
        layers: entry.layers,
      });
    }

    return out;
  }, [catalogueEntries, projectConstructions]);

  const selectedA = useMemo(
    () => options.find((o) => o.key === keyA) ?? null,
    [options, keyA],
  );
  const selectedB = useMemo(
    () => options.find((o) => o.key === keyB) ?? null,
    [options, keyB],
  );

  // KNMI-afgeleiden (gedeeld). Spiegelt RcCalculator's WP2-patroon.
  const climateStations = useMemo(() => listStations(), []);
  const climateYears = useMemo(
    () => listAvailableYears(climateStationId),
    [climateStationId],
  );
  const selectedClimate = useMemo(
    () => getMonthlyClimate(climateStationId, climateSelection),
    [climateStationId, climateSelection],
  );
  const climateUnavailable = selectedClimate === null;

  // Val terug op default-selectie wanneer het gekozen station de huidige
  // selectie niet kent (voorkomt een ongeldige dropdown-waarde).
  useEffect(() => {
    if (
      climateYears.length > 0 &&
      !climateYears.some(
        (y) => selectionToValue(y) === selectionToValue(climateSelection),
      )
    ) {
      setClimateSelection(
        climateYears.includes(CLIMATE_DEFAULT_SELECTION)
          ? CLIMATE_DEFAULT_SELECTION
          : climateYears[0]!,
      );
    }
  }, [climateYears, climateSelection]);

  const computedA = useMemo(
    () =>
      selectedA
        ? computeColumn(selectedA.layers, selectedA.position, selectedClimate)
        : null,
    [selectedA, selectedClimate],
  );
  const computedB = useMemo(
    () =>
      selectedB
        ? computeColumn(selectedB.layers, selectedB.position, selectedClimate)
        : null,
    [selectedB, selectedClimate],
  );

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        title="Rc-vergelijker"
        subtitle="Twee constructies naast elkaar — thermisch + hygrothermisch"
      />

      <div className="flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-6xl space-y-6">
          {/* Gedeelde KNMI-klimaatkeuze (voedt beide jaarbalansen) */}
          <div className="rounded-lg border border-[var(--oaec-border)] bg-[var(--oaec-bg-lighter)] px-4 py-3">
            <h3 className="mb-2 text-sm font-semibold text-on-surface-secondary">
              Klimaat (jaarbalans, beide kolommen)
            </h3>
            <div className="grid grid-cols-2 gap-3">
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>KNMI-station</span>
                <select
                  value={climateStationId}
                  onChange={(e) => setClimateStationId(e.target.value)}
                  className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm focus:border-primary focus:outline-none"
                >
                  {climateStations.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </label>
              <label className="flex flex-col gap-1 text-xs font-medium text-on-surface-muted">
                <span>Klimaatjaar / -periode</span>
                <select
                  value={selectionToValue(climateSelection)}
                  onChange={(e) =>
                    setClimateSelection(valueToSelection(e.target.value))
                  }
                  className="rounded border border-[var(--oaec-border)] px-2 py-1 text-sm focus:border-primary focus:outline-none"
                >
                  {climateYears.map((y) => (
                    <option key={selectionToValue(y)} value={selectionToValue(y)}>
                      {selectionLabel(y)}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            {climateUnavailable && (
              <div className="mt-3 rounded border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-300">
                Voor deze selectie is nog geen klimaatdata beschikbaar
                (NEN 5060-tabel volgt); 1991-2020-normaal gebruikt.
              </div>
            )}
            <p className="mt-2 text-2xs text-on-surface-muted">
              De steady-state Glaser blijft norm-vast op {GLASER_DEFAULTS.thetaE}
              °C en wordt niet door deze keuze beïnvloed.
            </p>
          </div>

          {/* Vergelijk-samenvatting (alleen als beide kolommen gevuld) */}
          {computedA && computedB && (
            <DeltaSummary a={computedA} b={computedB} />
          )}

          {/* Twee kolommen */}
          <div className="grid gap-6 lg:grid-cols-2">
            <CompareColumn
              title="Constructie A"
              options={options}
              selectedKey={keyA}
              onSelect={setKeyA}
              computed={computedA}
            />
            <CompareColumn
              title="Constructie B"
              options={options}
              selectedKey={keyB}
              onSelect={setKeyB}
              computed={computedB}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
