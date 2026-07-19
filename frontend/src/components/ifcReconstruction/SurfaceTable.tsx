/**
 * Oppervlaktenlijst voor de "IFC-reconstructie (bèta)"-pagina — tabel per
 * ruimte, uitklapbaar naar vlakken. Puur presentational: leest alleen de
 * flat rows uit `lib/ifcReconstruction/report.ts`.
 */
import { Fragment, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { ReconstructionResult } from "../../lib/ifcReconstruction/types";
import {
  flattenFaces,
  spaceCategoryTotals,
  spaceClassificationTotals,
  type FlatFaceRow,
} from "../../lib/ifcReconstruction/report";

interface SurfaceTableProps {
  result: ReconstructionResult;
  selectedRowKey: string | null;
  onSelectRowKey: (key: string | null) => void;
  qcOnly: boolean;
}

const cellClass = "px-2 py-1 text-xs text-on-surface-secondary whitespace-nowrap";
const headClass = "px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-scaffold-gray text-left";

export function SurfaceTable({ result, selectedRowKey, onSelectRowKey, qcOnly }: SurfaceTableProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState<Set<number>>(() => new Set(result.spaces.map((_, i) => i)));

  const allRows = useMemo(() => flattenFaces(result), [result]);
  const rowsBySpace = useMemo(() => {
    const map = new Map<number, FlatFaceRow[]>();
    for (const row of allRows) {
      const list = map.get(row.spaceIndex) ?? [];
      list.push(row);
      map.set(row.spaceIndex, list);
    }
    return map;
  }, [allRows]);

  const toggleSpace = (idx: number) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) next.delete(idx);
      else next.add(idx);
      return next;
    });
  };

  return (
    <div className="overflow-auto">
      <table className="w-full border-collapse">
        <thead className="sticky top-0 bg-surface-alt">
          <tr>
            <th className={headClass}></th>
            <th className={headClass}>{t("ifcReconstruction.table.orientation")}</th>
            <th className={headClass}>{t("ifcReconstruction.table.classification")}</th>
            <th className={headClass}>{t("ifcReconstruction.table.category")}</th>
            <th className={`${headClass} text-right`}>{t("ifcReconstruction.table.gross")}</th>
            <th className={`${headClass} text-right`}>{t("ifcReconstruction.table.net")}</th>
            <th className={headClass}>{t("ifcReconstruction.table.host")}</th>
            <th className={headClass}>{t("ifcReconstruction.table.source")}</th>
            <th className={headClass}>{t("ifcReconstruction.table.qc")}</th>
          </tr>
        </thead>
        <tbody>
          {result.spaces.map((space, spaceIndex) => {
            const rows = (rowsBySpace.get(spaceIndex) ?? []).filter((r) => !qcOnly || r.qcFlagged);
            if (qcOnly && rows.length === 0) return null;
            const catTotals = spaceCategoryTotals(space);
            const classTotals = spaceClassificationTotals(space);
            const isExpanded = expanded.has(spaceIndex);
            return (
              <Fragment key={`space-${spaceIndex}`}>
                <tr
                  className="cursor-pointer border-t border-[var(--oaec-border-subtle)] bg-surface-alt/60 hover:bg-[var(--oaec-hover)]"
                  onClick={() => toggleSpace(spaceIndex)}
                >
                  <td className={`${cellClass} font-mono`}>{isExpanded ? "▾" : "▸"}</td>
                  <td className={`${cellClass} font-medium text-on-surface`} colSpan={4}>
                    {space.name ?? space.longName ?? `Ruimte ${space.id}`}{" "}
                    <span className="text-scaffold-gray">
                      ({t("ifcReconstruction.table.floorArea")}: {space.floorAreaM2.toFixed(1)} m²)
                    </span>
                  </td>
                  <td className={`${cellClass} text-right font-medium`}>
                    {(catTotals.opaakM2 + catTotals.raamM2 + catTotals.deurM2).toFixed(1)} m²
                  </td>
                  <td className={cellClass} colSpan={3}>
                    <span title={t("ifcReconstruction.table.categoryBreakdown")}>
                      opaak {catTotals.opaakM2.toFixed(1)} · raam {catTotals.raamM2.toFixed(1)} · deur{" "}
                      {catTotals.deurM2.toFixed(1)} m²
                    </span>
                  </td>
                </tr>
                {isExpanded && (
                  <tr key={`space-${spaceIndex}-totals`} className="bg-surface-alt/30">
                    <td className={cellClass}></td>
                    <td className={`${cellClass} text-scaffold-gray`} colSpan={8}>
                      {t("ifcReconstruction.table.classificationBreakdown")}: exterieur{" "}
                      {classTotals.exterieurM2.toFixed(1)} · grond {classTotals.grondM2.toFixed(1)} · buurruimte{" "}
                      {classTotals.buurruimteM2.toFixed(1)} · gemengd {classTotals.gemengdM2.toFixed(1)} · onbepaald{" "}
                      {classTotals.onbepaaldM2.toFixed(1)} m²
                    </td>
                  </tr>
                )}
                {isExpanded &&
                  rows.map((row, i) => (
                    <tr
                      key={row.rowKey}
                      onClick={(e) => {
                        e.stopPropagation();
                        onSelectRowKey(row.rowKey === selectedRowKey ? null : row.rowKey);
                      }}
                      className={`cursor-pointer border-t border-[var(--oaec-border-subtle)] ${
                        row.rowKey === selectedRowKey ? "bg-primary/15" : "hover:bg-[var(--oaec-hover)]"
                      }`}
                    >
                      <td className={`${cellClass} text-scaffold-gray`}>{i + 1}</td>
                      <td className={cellClass}>{row.zone}</td>
                      <td className={cellClass}>{row.classification}</td>
                      <td className={cellClass}>{row.hostCategory}</td>
                      <td className={`${cellClass} text-right tabular-nums`}>{row.grossAreaM2.toFixed(2)}</td>
                      <td className={`${cellClass} text-right tabular-nums`}>{row.netAreaM2.toFixed(2)}</td>
                      <td className={cellClass}>{row.hostName || "—"}</td>
                      <td className={cellClass}>{row.hostSource}</td>
                      <td className={cellClass}>
                        {row.qcFlagged ? (
                          <span className="rounded bg-red-100 px-1.5 py-0.5 text-[10px] font-medium text-red-700" title={row.qcReason}>
                            {row.qcFlag}
                          </span>
                        ) : (
                          "—"
                        )}
                      </td>
                    </tr>
                  ))}
              </Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
