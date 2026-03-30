/**
 * Jaarlijkse vochtbalans-tabel voor NEN-EN-ISO 13788 analyse.
 *
 * Toont per maand de condensatie- of droogsnelheid en het opgebouwde
 * vocht op het kritieke condensatievlak. Waarschuwt bij schimmelrisico.
 */

import type { YearlyMoistureResult } from "../../lib/yearlyMoistureCalculation";

interface MoistureYearTableProps {
  result: YearlyMoistureResult;
}

// ---------- Helpers ----------

function formatGc(gc: number): string {
  if (Math.abs(gc) < 0.01) return "0";
  return gc > 0
    ? `+${gc.toFixed(1)}`
    : gc.toFixed(1);
}

function statusColor(status: string): string {
  switch (status) {
    case "condensation":
      return "bg-red-100 text-red-800";
    case "drying":
      return "bg-blue-100 text-blue-800";
    default:
      return "bg-green-100 text-green-800";
  }
}

function statusLabel(status: string): string {
  switch (status) {
    case "condensation":
      return "Condensatie";
    case "drying":
      return "Droging";
    default:
      return "Droog";
  }
}

function statusDot(status: string): string {
  switch (status) {
    case "condensation":
      return "bg-red-600/150";
    case "drying":
      return "bg-blue-500";
    default:
      return "bg-green-500";
  }
}

// ---------- Component ----------

export function MoistureYearTable({ result }: MoistureYearTableProps) {
  const { months, maxMa, wetDays, hasRisk, driesOut, planeInnerLayer, planeOuterLayer } = result;

  const hasAnyMoisture = maxMa > 0.1;

  return (
    <div className="space-y-3">
      {/* Condensatievlak info */}
      <div className="flex items-center gap-2 text-xs text-on-surface-muted">
        <span className="font-medium text-on-surface-secondary">Condensatievlak:</span>
        <span>
          tussen {planeInnerLayer} en {planeOuterLayer}
        </span>
      </div>

      {/* Waarschuwingen */}
      {hasRisk && (
        <div className="flex items-start gap-2 rounded-md border border-red-600/30 bg-red-600/15 px-3 py-2">
          <span className="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full bg-red-600/150" />
          <div className="text-xs text-red-800">
            <strong>Schimmelrisico:</strong> vocht aanwezig gedurende ca.{" "}
            {wetDays} dagen ({Math.ceil(wetDays / 30)} maanden).
            Bij houtachtige materialen ontstaat schimmelgroei na &gt; 90 dagen.
          </div>
        </div>
      )}

      {!driesOut && hasAnyMoisture && (
        <div className="flex items-start gap-2 rounded-md border border-amber-600/30 bg-amber-600/15 px-3 py-2">
          <span className="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full bg-amber-600/150" />
          <div className="text-xs text-amber-400">
            <strong>Onvoldoende droging:</strong> de constructie droogt niet
            volledig uit binnen een jaar. Vochtophoping neemt jaarlijks toe.
          </div>
        </div>
      )}

      {hasAnyMoisture && !hasRisk && driesOut && (
        <div className="flex items-start gap-2 rounded-md border border-green-200 bg-green-50 px-3 py-2">
          <span className="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full bg-green-500" />
          <div className="text-xs text-green-800">
            <strong>Acceptabel:</strong> er treedt tijdelijk condensatie op
            ({wetDays} dagen), maar de constructie droogt volledig uit.
          </div>
        </div>
      )}

      {!hasAnyMoisture && (
        <div className="flex items-start gap-2 rounded-md border border-green-200 bg-green-50 px-3 py-2">
          <span className="mt-0.5 inline-block h-2 w-2 shrink-0 rounded-full bg-green-500" />
          <div className="text-xs text-green-800">
            <strong>Geen condensatie:</strong> er treedt het hele jaar geen
            inwendige condensatie op.
          </div>
        </div>
      )}

      {/* Tabel */}
      <div className="overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b border-[var(--oaec-border)] text-left text-[10px] font-semibold uppercase tracking-wider text-on-surface-muted">
              <th className="pb-1.5 pr-2">Maand</th>
              <th className="pb-1.5 pr-2 text-right">{"θ"}e [°C]</th>
              <th className="pb-1.5 pr-2 text-right">RV [%]</th>
              <th className="pb-1.5 pr-2 text-right">{"θ"}c [°C]</th>
              <th className="pb-1.5 pr-2 text-right">
                p<sub>sat</sub> [Pa]
              </th>
              <th className="pb-1.5 pr-2 text-right">
                g<sub>c</sub> [g/m{"²"}]
              </th>
              <th className="pb-1.5 pr-2 text-right">
                M<sub>a</sub> [g/m{"²"}]
              </th>
              <th className="pb-1.5 pl-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {months.map((row) => {
              const isMax = row.ma >= maxMa - 0.01 && row.ma > 0.1;

              return (
                <tr
                  key={row.month}
                  className={`border-b border-[var(--oaec-border-subtle)] ${
                    isMax ? "bg-red-600/10" : ""
                  }`}
                >
                  <td className="py-1 pr-2 font-medium text-on-surface-secondary">
                    {row.month}
                  </td>
                  <td className="py-1 pr-2 text-right tabular-nums text-on-surface-secondary">
                    {row.thetaE.toFixed(1)}
                  </td>
                  <td className="py-1 pr-2 text-right tabular-nums text-on-surface-secondary">
                    {row.rhE}
                  </td>
                  <td className="py-1 pr-2 text-right tabular-nums text-on-surface-secondary">
                    {row.thetaC.toFixed(1)}
                  </td>
                  <td className="py-1 pr-2 text-right tabular-nums text-on-surface-secondary">
                    {Math.round(row.pSatC)}
                  </td>
                  <td
                    className={`py-1 pr-2 text-right tabular-nums font-medium ${
                      row.gc > 0.1
                        ? "text-red-700"
                        : row.gc < -0.1
                          ? "text-blue-700"
                          : "text-on-surface-muted"
                    }`}
                  >
                    {formatGc(row.gc)}
                  </td>
                  <td
                    className={`py-1 pr-2 text-right tabular-nums font-medium ${
                      row.ma > 0.1
                        ? isMax
                          ? "text-red-800"
                          : "text-on-surface-secondary"
                        : "text-on-surface-muted"
                    }`}
                  >
                    {row.ma > 0.1 ? row.ma.toFixed(1) : "0"}
                  </td>
                  <td className="py-1 pl-2">
                    <span
                      className={`inline-flex items-center gap-1 rounded-full px-1.5 py-0.5 text-[10px] font-medium ${statusColor(row.status)}`}
                    >
                      <span
                        className={`inline-block h-1.5 w-1.5 rounded-full ${statusDot(row.status)}`}
                      />
                      {statusLabel(row.status)}
                    </span>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Samenvatting */}
      {hasAnyMoisture && (
        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 border-t border-[var(--oaec-border)] pt-2 text-xs text-on-surface-muted">
          <span>
            Max. vocht:{" "}
            <strong className="text-on-surface">
              {maxMa.toFixed(1)} g/m{"²"}
            </strong>
          </span>
          <span>
            Vochtperiode:{" "}
            <strong
              className={hasRisk ? "text-red-700" : "text-on-surface"}
            >
              ca. {wetDays} dagen
            </strong>
          </span>
          <span>
            Jaarbalans:{" "}
            <strong
              className={driesOut ? "text-green-700" : "text-red-700"}
            >
              {driesOut ? "droogt uit" : "ophoping"}
            </strong>
          </span>
        </div>
      )}
    </div>
  );
}
