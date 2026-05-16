import { useCallback, useState } from "react";

import RibbonButton from "./RibbonButton";
import RibbonGroup from "./RibbonGroup";
import { reportIcon, exportIcon } from "./icons";
import { useProjectStore } from "../../store/projectStore";
import { useToastStore } from "../../store/toastStore";
import { useReportStore } from "../../store/reportStore";
import { useModellerStore } from "../modeller/modellerStore";
import { buildReportData } from "../../lib/reportBuilder";
import { generateReportDirect } from "../../lib/reportClient";
import i18next from "../../i18n/config";

/**
 * Ribbon tab voor de Rapport-page. Bevat opties voor pagina-formaat en
 * oriëntatie (zoals OCS RapportageTab) plus een "Genereer rapport" knop die
 * de PDF bouwt en cached in `useReportStore` zodat de Rapport-page 'm in een
 * iframe kan tonen.
 */
export default function RapportTab() {
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const projectConstructions = useModellerStore((s) => s.projectConstructions);
  const addToast = useToastStore((s) => s.addToast);
  const sections = useReportStore((s) => s.sections);
  const setPdfBlobUrl = useReportStore((s) => s.setPdfBlobUrl);
  const [isGenerating, setIsGenerating] = useState(false);

  const handleGenerate = useCallback(async () => {
    if (!result) {
      addToast(
        "Voer eerst een berekening uit voordat je een rapport genereert.",
        "info",
      );
      return;
    }
    setIsGenerating(true);
    try {
      const reportData = await buildReportData(
        project,
        result,
        projectConstructions,
        sections,
      );
      const blob = await generateReportDirect(reportData);
      const url = URL.createObjectURL(blob);
      setPdfBlobUrl(url);
      addToast(i18next.t("reportGenerated"), "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : i18next.t("reportFailed");
      addToast(msg, "error");
    } finally {
      setIsGenerating(false);
    }
  }, [project, result, projectConstructions, sections, addToast, setPdfBlobUrl]);

  const handleDownload = useCallback(() => {
    const url = useReportStore.getState().pdfBlobUrl;
    if (!url) {
      addToast("Genereer eerst het rapport.", "info");
      return;
    }
    const a = document.createElement("a");
    a.href = url;
    a.download = `${project.info.name || "rapport"}.pdf`;
    a.click();
  }, [project.info.name, addToast]);

  return (
    <>
      <RibbonGroup label="Rapport">
        <RibbonButton
          icon={reportIcon}
          label={isGenerating ? "Bezig..." : "Genereren"}
          disabled={!result || isGenerating}
          onClick={handleGenerate}
        />
        <RibbonButton
          icon={exportIcon}
          label="Download PDF"
          onClick={handleDownload}
        />
      </RibbonGroup>

      {/* "Weergave" group (A4/A3 + Portret/Landschap) was hier — verwijderd
          omdat het identiek zit in de Rapport-page opties-sidebar. User
          feedback: knoppen stonden dubbel. */}
    </>
  );
}
