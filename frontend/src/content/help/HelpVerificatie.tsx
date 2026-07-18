/**
 * Help — sectie "Verificatie".
 *
 * Toont per Vabi-referentieproject (bron: `tests/verification/`, zie
 * `verificationData.ts`) de verwachte resultaten in tabelvorm en kan die
 * met één klik live tegen de actuele rekenkern verifiëren.
 *
 * - **ISSO 51**: zelfde backend-route als de Resultaten-pagina
 *   (web: `POST /calculate`, desktop: Tauri `invoke("calculate")`).
 * - **ISSO 53**: de norm-routerende V2-route (web: `POST /calculate_v2`,
 *   desktop: `invoke("calculate_v2")`) met de legacy projectblob inline
 *   onder `calcs.isso53` — exact de input die de Rust golden-tests voeren.
 *
 * Vergelijklogica en toleranties staan in `lib/verificationCompare.ts`:
 * ISSO 51 ±2 W abs / ±2 % rel (identiek aan de Rust-integratietests),
 * ISSO 53 per-metric toleranties die de golden-tests spiegelen.
 *
 * Projecten met `mode: "informative"` (normversie-mismatch) tonen een
 * disclaimer-banner en Δ-kolommen zónder ✓/✗-verdicts.
 */
import { useState } from "react";

import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { createBackend } from "../../lib/backend";
import {
  compareIsso53Results,
  compareResults,
  expectedOnlyRows,
  isso53ExpectedOnlyRows,
  ISSO53_METRIC_LABELS,
  type ComparisonRow,
  type Isso53Comparison,
  type Isso53ComparisonRow,
  type VerificationComparison,
} from "../../lib/verificationCompare";
import type { BuildingType } from "../../types";
import {
  buildIsso53VerifyPayload,
  ISSO53_VERIFICATION_PROJECTS,
  VERIFICATION_PROJECTS,
  type Isso53VerificationProjectDef,
  type VerificationProjectDef,
} from "./verificationData";

// ---------------------------------------------------------------------------
// Weergave-helpers
// ---------------------------------------------------------------------------

const BUILDING_TYPE_LABELS: Record<BuildingType, string> = {
  detached: "Vrijstaand",
  semi_detached: "Twee-onder-een-kap",
  terraced: "Tussenwoning",
  end_of_terrace: "Hoekwoning",
  porch: "Portiekwoning",
  gallery: "Galerijwoning",
  stacked: "Gestapeld",
};

/** Vermogen in W, afgerond op hele watts; "—" wanneer (nog) onbekend. */
function fmtW(value: number | null): string {
  return value === null ? "—" : Math.round(value).toString();
}

/** Verschil in W, met expliciet teken. */
function fmtDeltaW(value: number | null): string {
  if (value === null) return "—";
  const rounded = Math.round(value);
  return `${rounded > 0 ? "+" : ""}${rounded}`;
}

/** Verschil in %, één decimaal, met expliciet teken. */
function fmtDeltaPct(value: number | null): string {
  if (value === null) return "—";
  return `${value > 0 ? "+" : ""}${value.toFixed(1)}%`;
}

/** ✓ / ✗ / — verdict-cel. */
function VerdictCell({ pass }: { pass: boolean | null }) {
  if (pass === null) {
    return <span className="text-on-surface-muted">—</span>;
  }
  return pass ? (
    <span className="font-semibold text-green-400">✓</span>
  ) : (
    <span className="font-semibold text-red-400">✗</span>
  );
}

function MetaItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="contents">
      <dt className="text-on-surface-muted">{label}</dt>
      <dd className="text-on-surface-secondary">{value}</dd>
    </div>
  );
}

/** Disclaimer-banner voor informatieve (niet-referentie) projecten. */
function InformativeBanner({ disclaimer }: { disclaimer: string }) {
  return (
    <div className="rounded-md border oa-warning-box px-4 py-3 text-sm leading-relaxed">
      <p className="mb-1 text-xs font-semibold uppercase tracking-wider">
        Informatief — geen referentie
      </p>
      <p>{disclaimer}</p>
    </div>
  );
}

/** Neutrale badge voor informatieve projecten (vervangt het rood/groen-verdict). */
function InformativeBadge() {
  return (
    <span className="rounded bg-surface-alt px-2 py-0.5 text-xs font-semibold text-on-surface-secondary">
      informatief — geen referentie
    </span>
  );
}

/** Bekende-afwijkingen-blok (gedeeld door ISSO 51- en ISSO 53-kaarten). */
function KnownDeviations({ items }: { items: ReadonlyArray<string> }) {
  if (items.length === 0) return null;
  return (
    <div className="rounded-md border border-[var(--oaec-border-subtle)] bg-surface-alt px-4 py-3">
      <h4 className="mb-1 text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
        Bekende afwijkingen
      </h4>
      <ul className="list-disc pl-5 text-sm leading-relaxed text-on-surface-secondary">
        {items.map((d) => (
          <li key={d}>{d}</li>
        ))}
      </ul>
    </div>
  );
}

/** Gedeelde foutmelding-banner voor mislukte verificaties. */
function VerifyError({ message }: { message: string }) {
  return (
    <div className="rounded-md border border-red-600/30 bg-red-600/10 px-4 py-3 text-sm text-red-400">
      <p className="font-medium">Verificatie mislukt: {message}</p>
      <p className="mt-1 text-xs text-red-400/80">
        Controleer of de rekenkern bereikbaar is (web: backend-API, desktop:
        ingebouwde engine) en probeer het opnieuw.
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// ISSO 51 — per-project kaart
// ---------------------------------------------------------------------------

type RunState =
  | { status: "idle" }
  | { status: "running" }
  | { status: "done"; comparison: VerificationComparison }
  | { status: "error"; message: string };

function ResultTable({
  rooms,
  building,
  hasRun,
  showVerdicts,
}: {
  rooms: ComparisonRow[];
  building: ComparisonRow;
  hasRun: boolean;
  /** false = informatief project: géén ✓/✗-kolom. */
  showVerdicts: boolean;
}) {
  return (
    <div className="overflow-x-auto rounded-lg border border-[var(--oaec-border)]">
      <table className="w-full border-collapse text-sm tabular-nums">
        <thead>
          <tr className="border-b-2 border-[var(--oaec-border)] bg-surface-alt text-left text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
            <th className="px-3 py-2">Vertrek</th>
            <th className="px-3 py-2 text-right">θ_i (°C)</th>
            <th className="px-3 py-2 text-right">Φ_HL Vabi (W)</th>
            <th className="px-3 py-2 text-right">Φ_HL berekend (W)</th>
            <th className="px-3 py-2 text-right">Δ (W)</th>
            <th className="px-3 py-2 text-right">Δ (%)</th>
            {showVerdicts && (
              <th className="w-[60px] px-3 py-2 text-center">{hasRun ? "✓/✗" : ""}</th>
            )}
          </tr>
        </thead>
        <tbody>
          {rooms.map((row) => (
            <tr
              key={row.roomId}
              className="border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]/50"
            >
              <td className="px-3 py-1.5 text-on-surface">
                <span className="font-mono text-xs text-on-surface-muted">{row.roomId}</span>{" "}
                {row.roomName}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {row.thetaI === null ? "—" : row.thetaI.toFixed(1)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtW(row.expectedW)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface">{fmtW(row.actualW)}</td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtDeltaW(row.deltaW)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtDeltaPct(row.deltaPct)}
              </td>
              {showVerdicts && (
                <td className="px-3 py-1.5 text-center">
                  <VerdictCell pass={row.pass} />
                </td>
              )}
            </tr>
          ))}
          <tr className="border-t-2 border-[var(--oaec-border)] bg-surface-alt font-semibold">
            <td className="px-3 py-2 text-on-surface" colSpan={2}>
              {building.roomName}
            </td>
            <td className="px-3 py-2 text-right text-on-surface">{fmtW(building.expectedW)}</td>
            <td className="px-3 py-2 text-right text-on-surface">{fmtW(building.actualW)}</td>
            <td className="px-3 py-2 text-right text-on-surface-secondary">
              {fmtDeltaW(building.deltaW)}
            </td>
            <td className="px-3 py-2 text-right text-on-surface-secondary">
              {fmtDeltaPct(building.deltaPct)}
            </td>
            {showVerdicts && (
              <td className="px-3 py-2 text-center">
                <VerdictCell pass={building.pass} />
              </td>
            )}
          </tr>
        </tbody>
      </table>
    </div>
  );
}

function VerificationProjectCard({ def }: { def: VerificationProjectDef }) {
  const [run, setRun] = useState<RunState>({ status: "idle" });
  const informative = def.mode === "informative";

  const handleVerify = async () => {
    setRun({ status: "running" });
    try {
      // Zelfde route als de Resultaten-pagina: web → POST /calculate,
      // Tauri → invoke("calculate"). Geen frame-override of project-
      // constructies nodig — input.json is al het kale engine-formaat
      // (identiek aan wat de Rust-integratietests voeren).
      const result = await createBackend().calculate(def.input);
      setRun({ status: "done", comparison: compareResults(def.expected, result) });
    } catch (err) {
      setRun({
        status: "error",
        message: err instanceof Error ? err.message : "Onbekende fout bij berekenen",
      });
    }
  };

  const comparison = run.status === "done" ? run.comparison : null;
  const rows = comparison ?? expectedOnlyRows(def.expected);
  const thetaE = def.input.climate.theta_e;

  return (
    <Card title={`${def.title} — ${def.software}`}>
      <div className="flex flex-col gap-4">
        {/* Informatief project: prominente disclaimer bovenaan de kaart */}
        {informative && def.disclaimer && <InformativeBanner disclaimer={def.disclaimer} />}

        {/* Bron-metadata + input-samenvatting */}
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <MetaItem label="Norm" value={def.norm} />
          <MetaItem label="Software" value={def.software} />
          <MetaItem
            label="Gebouw"
            value={`${BUILDING_TYPE_LABELS[def.input.building.building_type]}, ${def.input.rooms.length} vertrekken (invoer)`}
          />
          <MetaItem
            label="θ_e"
            value={thetaE === undefined ? "—" : `${thetaE.toFixed(1)} °C`}
          />
          <MetaItem label="q_v10" value={`${def.input.building.qv10.toFixed(1)} dm³/s`} />
          <MetaItem label="Ventilatie" value={def.ventilation} />
          <MetaItem
            label="Tolerantie"
            value={
              informative
                ? "n.v.t. — informatief project, geen pass/fail"
                : "±2 W absoluut óf ±2 % relatief (ruimste geldt)"
            }
          />
          <MetaItem label="Bron" value={def.sourcePath} />
        </dl>

        <KnownDeviations items={def.knownDeviations} />

        {/* Actie + status */}
        <div className="flex flex-wrap items-center gap-3">
          <Button
            size="sm"
            onClick={handleVerify}
            disabled={run.status === "running"}
          >
            {run.status === "running" ? "Bezig met verifiëren…" : "Verifieer nu"}
          </Button>

          {informative ? (
            <InformativeBadge />
          ) : (
            comparison && (
              <span
                className={`rounded px-2 py-0.5 text-xs font-semibold ${
                  comparison.allPass
                    ? "bg-green-600/15 text-green-400"
                    : "bg-red-600/15 text-red-400"
                }`}
              >
                {comparison.passedRooms}/{comparison.totalRooms} vertrekken binnen tolerantie ·
                gebouwtotaal {comparison.buildingPass ? "✓" : "✗"}
              </span>
            )
          )}
        </div>

        {/* Foutmelding — engine niet bereikbaar of berekening geweigerd */}
        {run.status === "error" && <VerifyError message={run.message} />}

        {/* Resultaat-tabel: vóór de eerste run alleen de Vabi-verwachting */}
        <ResultTable
          rooms={rows.rooms}
          building={rows.building}
          hasRun={comparison !== null}
          showVerdicts={!informative}
        />
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// ISSO 53 — per-project kaart (metric-rijen, per-metric CI-toleranties)
// ---------------------------------------------------------------------------

type Isso53RunState =
  | { status: "idle" }
  | { status: "running" }
  | { status: "done"; comparison: Isso53Comparison }
  | { status: "error"; message: string };

/** Tolerantie-weergave: absolute 1 W-grens bij verwacht 0, anders ±x%. */
function fmtTolerance(row: Isso53ComparisonRow): string {
  if (Math.abs(row.expectedW) < Number.EPSILON) return "±1 W";
  return `±${row.tolerancePct.toLocaleString("nl-NL", { maximumFractionDigits: 1 })}%`;
}

function Isso53ResultTable({
  rows,
  hasRun,
}: {
  rows: Isso53ComparisonRow[];
  hasRun: boolean;
}) {
  return (
    <div className="overflow-x-auto rounded-lg border border-[var(--oaec-border)]">
      <table className="w-full border-collapse text-sm tabular-nums">
        <thead>
          <tr className="border-b-2 border-[var(--oaec-border)] bg-surface-alt text-left text-xs font-semibold uppercase tracking-wider text-on-surface-secondary">
            <th className="px-3 py-2">Vertrek</th>
            <th className="px-3 py-2">Grootheid</th>
            <th className="px-3 py-2 text-right">Vabi (W)</th>
            <th className="px-3 py-2 text-right">Berekend (W)</th>
            <th className="px-3 py-2 text-right">Δ (W)</th>
            <th className="px-3 py-2 text-right">Δ (%)</th>
            <th className="px-3 py-2 text-right">Tolerantie</th>
            <th className="w-[60px] px-3 py-2 text-center">{hasRun ? "✓/✗" : ""}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr
              key={row.rowKey}
              className={`border-b border-[var(--oaec-border-subtle)] hover:bg-[var(--oaec-hover)]/50 ${
                row.metric === "total" ? "bg-surface-alt font-semibold" : ""
              }`}
            >
              <td className="px-3 py-1.5 text-on-surface">
                <span className="font-mono text-xs text-on-surface-muted">{row.roomId}</span>{" "}
                {row.roomLabel}
              </td>
              <td className="px-3 py-1.5 text-on-surface-secondary">
                {ISSO53_METRIC_LABELS[row.metric]}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtW(row.expectedW)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface">{fmtW(row.actualW)}</td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtDeltaW(row.deltaW)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtDeltaPct(row.deltaPct)}
              </td>
              <td className="px-3 py-1.5 text-right text-on-surface-secondary">
                {fmtTolerance(row)}
              </td>
              <td className="px-3 py-1.5 text-center">
                <VerdictCell pass={row.pass} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Isso53VerificationProjectCard({ def }: { def: Isso53VerificationProjectDef }) {
  const [run, setRun] = useState<Isso53RunState>({ status: "idle" });

  const handleVerify = async () => {
    setRun({ status: "running" });
    try {
      // Norm-routerende V2-route (web: POST /calculate_v2, Tauri:
      // invoke("calculate_v2")). De legacy ISSO 53-projectblob gaat inline
      // onder calcs.isso53 — exact wat de Rust golden-tests via
      // `calculate_from_json` voeren.
      const result = await createBackend().calculateV2(buildIsso53VerifyPayload(def));
      setRun({ status: "done", comparison: compareIsso53Results(def.metrics, result) });
    } catch (err) {
      setRun({
        status: "error",
        message: err instanceof Error ? err.message : "Onbekende fout bij berekenen",
      });
    }
  };

  const comparison = run.status === "done" ? run.comparison : null;
  const rows = comparison ? comparison.rows : isso53ExpectedOnlyRows(def.metrics);

  return (
    <Card title={`${def.title} — ${def.software}`}>
      <div className="flex flex-col gap-4">
        {/* Bron-metadata + input-samenvatting */}
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <MetaItem label="Norm" value={def.norm} />
          <MetaItem label="Software" value={def.software} />
          <MetaItem label="Gebouw" value={def.roomsSummary} />
          <MetaItem label="θ_e" value={`${def.thetaE.toFixed(1)} °C`} />
          <MetaItem label="Ventilatie" value={def.ventilation} />
          <MetaItem label="Verwarming" value={def.heating} />
          <MetaItem
            label="Tolerantie"
            value="per grootheid — zelfde grenzen als de Rust golden-tests in CI (zie tabel)"
          />
          <MetaItem label="Bron" value={def.sourcePath} />
          <MetaItem label="CI-test" value={def.goldenTest} />
        </dl>

        <KnownDeviations items={def.knownDeviations} />

        {/* Actie + status */}
        <div className="flex flex-wrap items-center gap-3">
          <Button
            size="sm"
            onClick={handleVerify}
            disabled={run.status === "running"}
          >
            {run.status === "running" ? "Bezig met verifiëren…" : "Verifieer nu"}
          </Button>

          {comparison && (
            <span
              className={`rounded px-2 py-0.5 text-xs font-semibold ${
                comparison.allPass
                  ? "bg-green-600/15 text-green-400"
                  : "bg-red-600/15 text-red-400"
              }`}
            >
              {comparison.passed}/{comparison.total} grootheden binnen tolerantie
            </span>
          )}
        </div>

        {run.status === "error" && <VerifyError message={run.message} />}

        {/* Resultaat-tabel: vóór de eerste run alleen de Vabi-verwachting */}
        <Isso53ResultTable rows={rows} hasRun={comparison !== null} />
      </div>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Sectie
// ---------------------------------------------------------------------------

export function HelpVerificatie() {
  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm leading-relaxed text-on-surface-secondary">
        De rekenkern wordt cross-gevalideerd tegen referentieberekeningen uit Vabi
        Elements. Dezelfde projecten worden bij élke codewijziging automatisch
        bewaakt door de Rust golden-tests in CI
        (<code className="font-mono text-xs">crates/isso51-core/tests/integration_test.rs</code>,{" "}
        <code className="font-mono text-xs">crates/isso53-core/tests/vabi_*_golden.rs</code>);
        deze pagina is daarvan de on-demand spiegel. Onderstaande tabellen tonen
        per verificatieproject de verwachte Vabi-waarden; met{" "}
        <em>Verifieer nu</em> draait dezelfde invoer door de actuele engine en
        worden de afwijkingen live berekend.
      </p>

      <h3 className="mt-2 font-heading text-sm font-semibold uppercase tracking-wider text-on-surface-secondary">
        ISSO 51 — woningbouw
      </h3>
      <p className="text-sm leading-relaxed text-on-surface-secondary">
        Een vertrek is PASS bij{" "}
        <code className="font-mono text-xs">|Δ| ≤ max(2 W; 2 % van verwacht)</code>{" "}
        — identiek aan de criteria van de Rust-integratietests.
      </p>

      {VERIFICATION_PROJECTS.map((def) => (
        <VerificationProjectCard key={def.id} def={def} />
      ))}

      <h3 className="mt-2 font-heading text-sm font-semibold uppercase tracking-wider text-on-surface-secondary">
        ISSO 53 — utiliteitsbouw
      </h3>
      <p className="text-sm leading-relaxed text-on-surface-secondary">
        De ISSO 53-projecten worden per grootheid (Φ_T, Φ_V, Φ_I, Φ_hu, totaal)
        vergeleken, elk met de tolerantie die de bijbehorende Rust golden-test
        in CI hanteert. Die toleranties verschillen per project en grootheid —
        fixture-bundeling en gedocumenteerde Vabi-anomalies maken een uniforme
        2%-grens hier niet realistisch (zie de bekende afwijkingen per kaart).
      </p>

      {ISSO53_VERIFICATION_PROJECTS.map((def) => (
        <Isso53VerificationProjectCard key={def.id} def={def} />
      ))}

      <p className="text-xs leading-relaxed text-on-surface-muted">
        Bron-data: <code className="font-mono">tests/verification/</code> in de
        repository — dezelfde bestanden die de geautomatiseerde Rust-tests voeren.
        Het ISSO 51-gebouwtotaal vergelijkt het Vabi-aansluitvermogen (Φ_HL,build,
        kwadratische sommatie conform erratum 2023) met{" "}
        <code className="font-mono">connection_capacity</code> uit de engine; het
        ISSO 53-totaal per vertrek volgt de Vabi-conventie Φ_T + Φ_V + Φ_I + Φ_hu.
      </p>
    </div>
  );
}
