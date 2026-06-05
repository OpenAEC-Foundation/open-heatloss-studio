import { useCallback, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { AlgemeenTab } from "../components/projectSetup/AlgemeenTab";

import { Button } from "../components/ui/Button";
import { PageHeader } from "../components/layout/PageHeader";
import { useAuth } from "../hooks/useAuth";
import { useProjectStore } from "../store/projectStore";
import { createProject, updateProject as updateProjectApi, ConflictError } from "../lib/backend";
import { exportIfcEnergy, openProjectFile, extractAndLinkConstructions } from "../lib/importExport";
import type { ProjectResult } from "../types";
import { useRunCalculation } from "../hooks/useRunCalculation";
import { useToastStore } from "../store/toastStore";

export function ProjectSetup() {
  const navigate = useNavigate();
  const auth = useAuth();
  const {
    project, isCalculating,
    setError, activeProjectId, setActiveProjectId,
    serverUpdatedAt,
  } = useProjectStore();
  const addToast = useToastStore((s) => s.addToast);
  const runCalculation = useRunCalculation();
  const [isSaving, setIsSaving] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleCalculate = useCallback(async () => {
    const ok = await runCalculation();
    if (ok) {
      navigate("/results");
    }
  }, [runCalculation, navigate]);

  const handleSave = useCallback(async () => {
    setIsSaving(true);
    try {
      if (activeProjectId) {
        const response = await updateProjectApi(activeProjectId, {
          name: project.info.name || undefined,
          project_data: project,
          expected_updated_at: serverUpdatedAt ?? undefined,
        });
        useProjectStore.setState({ isDirty: false, serverUpdatedAt: response.updated_at });
      } else {
        const name = project.info.name || "Naamloos project";
        const result = await createProject(name, project);
        setActiveProjectId(result.id);
        useProjectStore.setState({ isDirty: false });
      }
      addToast("Project opgeslagen", "success", 2000);
    } catch (err) {
      if (err instanceof ConflictError) {
        useProjectStore.setState({ hasConflict: true });
      } else {
        addToast(err instanceof Error ? err.message : "Opslaan mislukt", "error");
      }
    } finally {
      setIsSaving(false);
    }
  }, [project, activeProjectId, serverUpdatedAt, setActiveProjectId, addToast]);

  const handleExport = useCallback(() => {
    const { result } = useProjectStore.getState();
    exportIfcEnergy(project, result as ProjectResult | null);
  }, [project]);

  const handleImportFile = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = () => {
        try {
          const imported = openProjectFile(reader.result as string);

          // Thermal import detected — redirect to wizard
          if (imported.type === "thermal") {
            sessionStorage.setItem("thermalImportJson", imported.rawJson);
            navigate("/import/thermal");
            addToast("Thermal import gedetecteerd — wizard geopend", "info");
            return;
          }

          // Regular project import — thread norm + ISSO 53 sidecars uit de
          // envelope mee zodat ISSO 53-config behouden blijft.
          extractAndLinkConstructions(imported.project);
          const { setProject, setResult } = useProjectStore.getState();
          setProject(imported.project, {
            norm: imported.norm,
            isso53Building: imported.isso53?.building,
            isso53Rooms: imported.isso53?.rooms,
            sharedExtra: imported.sharedExtra,
          });
          if (imported.result) {
            setResult(imported.result);
          }
        } catch (err) {
          setError(err instanceof Error ? err.message : "Import mislukt");
        }
      };
      reader.readAsText(file);

      // Reset input so the same file can be re-imported.
      e.target.value = "";
    },
    [setError, navigate, addToast],
  );

  const { t } = useTranslation();

  return (
    <div>
      <PageHeader
        title={t("projectSetup.title", "Project")}
        subtitle={t("projectSetup.subtitle", "Gebouw- en installatie-instellingen")}
        actions={
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => fileInputRef.current?.click()}>
              Importeren
            </Button>
            <Button variant="ghost" onClick={handleExport}>
              Exporteren
            </Button>
            {auth.isLoggedIn && (
              <Button variant="secondary" onClick={handleSave} disabled={isSaving}>
                {isSaving
                  ? "Opslaan..."
                  : activeProjectId
                    ? "Opslaan"
                    : "Opslaan naar server"}
              </Button>
            )}
            <Button onClick={handleCalculate} disabled={isCalculating || project.rooms.length === 0}>
              {isCalculating ? "Berekenen..." : "Berekenen"}
            </Button>
          </div>
        }
      />

      <div className="p-6">
        <AlgemeenTab />
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".ifcenergy,.json,.isso51.json"
        className="hidden"
        onChange={handleImportFile}
      />
    </div>
  );
}
