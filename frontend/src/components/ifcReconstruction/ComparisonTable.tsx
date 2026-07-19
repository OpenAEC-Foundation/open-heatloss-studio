/**
 * Vergelijkingstabel: reconstructie vs. de bestaande (pyrevit-warmteverlies)
 * methode, per ruimte-match. Puur presentational — de matching/Δ%-logica zit
 * in `lib/ifcReconstruction/report.ts::compareWithPyrevit`.
 */
import { useTranslation } from "react-i18next";

import type { ComparisonResult } from "../../lib/ifcReconstruction/report";

interface ComparisonTableProps {
  comparison: ComparisonResult;
}

const cellClass = "px-2 py-1 text-xs text-on-surface-secondary whitespace-nowrap";
const headClass = "px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-scaffold-gray text-left";

function formatDelta(deltaPercent: number | null): string {
  if (deltaPercent === null) return "—";
  if (!Number.isFinite(deltaPercent)) return "∞";
  const sign = deltaPercent > 0 ? "+" : "";
  return `${sign}${deltaPercent.toFixed(1)}%`;
}

export function ComparisonTable({ comparison }: ComparisonTableProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-4">
      {comparison.matched.map((room) => (
        <div key={`${room.reconRoomId}-${room.pyrevitRoomId}`} className="overflow-hidden rounded-lg border border-[var(--oaec-border-subtle)]">
          <div className="flex items-center justify-between bg-surface-alt/60 px-3 py-2">
            <span className="text-sm font-medium text-on-surface">
              {room.reconRoomName} <span className="text-scaffold-gray">↔</span> {room.pyrevitRoomName}
            </span>
            <span className="text-xs text-scaffold-gray">
              Σ {room.reconTotalM2.toFixed(1)} / {room.pyrevitTotalM2.toFixed(1)} m²
              {room.reconExcludedOnbepaaldM2 > 0 && (
                <>
                  {" "}
                  · {t("ifcReconstruction.compare.excluded")}: {room.reconExcludedOnbepaaldM2.toFixed(1)} m²
                </>
              )}
            </span>
          </div>
          <table className="w-full border-collapse">
            <thead>
              <tr>
                <th className={headClass}>{t("ifcReconstruction.table.orientation")}</th>
                <th className={headClass}>boundary_type</th>
                <th className={`${headClass} text-right`}>{t("ifcReconstruction.compare.recon")}</th>
                <th className={`${headClass} text-right`}>{t("ifcReconstruction.compare.pyrevit")}</th>
                <th className={`${headClass} text-right`}>Δ%</th>
              </tr>
            </thead>
            <tbody>
              {room.cells.map((cell) => (
                <tr
                  key={`${cell.verticalPosition}-${cell.boundaryType}`}
                  className={`border-t border-[var(--oaec-border-subtle)] ${cell.flagged ? "bg-red-50" : ""}`}
                >
                  <td className={cellClass}>{cell.verticalPosition}</td>
                  <td className={cellClass}>{cell.boundaryType}</td>
                  <td className={`${cellClass} text-right tabular-nums`}>{cell.reconM2.toFixed(2)}</td>
                  <td className={`${cellClass} text-right tabular-nums`}>{cell.pyrevitM2.toFixed(2)}</td>
                  <td className={`${cellClass} text-right tabular-nums font-medium ${cell.flagged ? "text-red-700" : ""}`}>
                    {formatDelta(cell.deltaPercent)}
                  </td>
                </tr>
              ))}
              {room.cells.length === 0 && (
                <tr>
                  <td className={cellClass} colSpan={5}>
                    {t("ifcReconstruction.compare.noCells")}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      ))}

      {(comparison.unmatchedRecon.length > 0 || comparison.unmatchedPyrevit.length > 0) && (
        <div className="rounded-lg border border-dashed border-[var(--oaec-border-subtle)] p-3 text-xs text-scaffold-gray">
          {comparison.unmatchedRecon.length > 0 && (
            <div>
              {t("ifcReconstruction.compare.unmatchedRecon")}:{" "}
              {comparison.unmatchedRecon.map((s) => s.name ?? s.longName ?? `#${s.id}`).join(", ")}
            </div>
          )}
          {comparison.unmatchedPyrevit.length > 0 && (
            <div>
              {t("ifcReconstruction.compare.unmatchedPyrevit")}:{" "}
              {comparison.unmatchedPyrevit.map((r) => r.name).join(", ")}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
