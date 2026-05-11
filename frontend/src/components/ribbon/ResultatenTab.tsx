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
import { generateReportDirect } from "../../lib/reportClient";
import i18next from "../../i18n/config";

export default function ResultatenTab() {
  const { t } = useTranslation("ribbon");
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const projectConstructions = useModellerStore((s) => s.projectConstructions);
  const addToast = useToastStore((s) => s.addToast);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleReport = useCallback(async () => {
    if (!result) return;
    setIsGenerating(true);
    try {
      const reportData = await buildReportData(project, result, projectConstructions);
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
  }, [project, result, projectConstructions, addToast]);

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
