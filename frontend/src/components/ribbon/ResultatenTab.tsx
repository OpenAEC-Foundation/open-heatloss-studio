import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { reportIcon, exportIcon } from "./icons";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import { useModellerStore } from "../modeller/modellerStore";
import { exportIfcEnergy } from "../../lib/importExport";
import { buildReportData } from "../../lib/reportBuilder";
import { buildIsso53Report } from "../../lib/isso53ReportBuilder";
import type { Isso53ProjectResult } from "../../types/isso53Result";
import { generateReportDirect } from "../../lib/reportClient";
import i18next from "../../i18n/config";

export default function ResultatenTab() {
  const { t } = useTranslation("ribbon");
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const norm = useProjectStore((s) => s.norm);
  const isso53Building = useProjectStore((s) => s.isso53Building);
  const isso53Rooms = useProjectStore((s) => s.isso53Rooms);
  const projectConstructions = useModellerStore((s) => s.projectConstructions);
  const addToast = useToastStore((s) => s.addToast);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleReport = useCallback(async () => {
    if (!result) return;
    setIsGenerating(true);
    try {
      // Norm-routing — zie RapportTab.handleGenerate voor toelichting.
      const reportData =
        norm === "isso53"
          ? buildIsso53Report(
              project,
              result as unknown as Isso53ProjectResult,
              isso53Building,
              isso53Rooms,
            )
          : await buildReportData(project, result, projectConstructions);
      const blob = await generateReportDirect(reportData);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${project.info.name || "rapport"}.pdf`;
      a.click();
      URL.revokeObjectURL(url);
      addToast(i18next.t("reportGenerated"), "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : i18next.t("reportFailed");
      addToast(msg, "error");
    } finally {
      setIsGenerating(false);
    }
  }, [
    project,
    result,
    norm,
    isso53Building,
    isso53Rooms,
    projectConstructions,
    addToast,
  ]);

  const handleExport = useCallback(() => {
    exportIfcEnergy(project, result);
    addToast(i18next.t("projectExported"), "success");
  }, [project, result, addToast]);

  return (
    <>
      <RibbonGroup label={t("resultaten.report")}>
        <RibbonButton
          icon={reportIcon}
          label={t("resultaten.generateReport")}
          disabled={!result || isGenerating}
          onClick={handleReport}
        />
      </RibbonGroup>
      <RibbonGroup label={t("resultaten.export")}>
        <RibbonButton
          icon={exportIcon}
          label={t("resultaten.exportJson")}
          onClick={handleExport}
        />
      </RibbonGroup>
    </>
  );
}
