import { useCallback, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Input } from "../components/ui/Input";
import { Select } from "../components/ui/Select";
import { PageHeader } from "../components/layout/PageHeader";
import { useAuth } from "../hooks/useAuth";
import { useBackend } from "../hooks/useBackend";
import { useProjectStore } from "../store/projectStore";
import { createProject, updateProject as updateProjectApi, ConflictError } from "../lib/backend";
import { exportProject, importProject, extractAndLinkConstructions } from "../lib/importExport";
import { formatArea } from "../lib/formatNumber";
import { prepareProjectForCalculation } from "../lib/frameOverride";
import { useModellerStore } from "../components/modeller/modellerStore";
import { useToastStore } from "../store/toastStore";
import {
  BUILDING_TYPE_LABELS,
  DEFAULT_THETA_WATER,
  SECURITY_CLASS_LABELS,
  VENTILATION_SYSTEM_LABELS,
} from "../lib/constants";
import type { Building, DesignConditions, ProjectInfo, VentilationConfig } from "../types";

function toOptions(labels: Record<string, string>) {
  return Object.entries(labels).map(([value, label]) => ({ value, label }));
}

export function ProjectSetup() {
  const navigate = useNavigate();
  const backend = useBackend();
  const auth = useAuth();
  const {
    project, updateProject, isCalculating, setCalculating,
    setResult, setError, activeProjectId, setActiveProjectId,
    serverUpdatedAt, setFrameUValueOverride,
  } = useProjectStore();
  const projectConstructions = useModellerStore((s) => s.projectConstructions);
  const addToast = useToastStore((s) => s.addToast);
  const [isSaving, setIsSaving] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { info, building, climate, ventilation } = project;

  const updateInfo = useCallback(
    (partial: Partial<ProjectInfo>) => {
      updateProject({ info: { ...project.info, ...partial } });
    },
    [project.info, updateProject],
  );

  const updateBuilding = useCallback(
    (partial: Partial<Building>) => {
      updateProject({ building: { ...project.building, ...partial } });
    },
    [project.building, updateProject],
  );

  const updateClimate = useCallback(
    (partial: Partial<DesignConditions>) => {
      updateProject({ climate: { ...project.climate, ...partial } });
    },
    [project.climate, updateProject],
  );

  const updateVentilation = useCallback(
    (partial: Partial<VentilationConfig>) => {
      updateProject({ ventilation: { ...project.ventilation, ...partial } });
    },
    [project.ventilation, updateProject],
  );

  const handleCalculate = useCallback(async () => {
    setCalculating(true);
    try {
      const payload = prepareProjectForCalculation(project, projectConstructions);
      const result = await backend.calculate(payload);
      setResult(result);
      navigate("/results");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Berekening mislukt");
    }
  }, [
    backend,
    project,
    projectConstructions,
    setCalculating,
    setResult,
    setError,
    navigate,
  ]);

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
    exportProject(project, result);
  }, [project]);

  const handleImportFile = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = () => {
        try {
          const imported = importProject(reader.result as string);

          // Thermal import detected — redirect to wizard
          if (imported.type === "thermal") {
            sessionStorage.setItem("thermalImportJson", imported.rawJson);
            navigate("/import/thermal");
            addToast("Thermal import gedetecteerd — wizard geopend", "info");
            return;
          }

          // Regular project import
          extractAndLinkConstructions(imported.project);
          const { setProject, setResult } = useProjectStore.getState();
          setProject(imported.project);
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

  const numVal = (v: string) => (v === "" ? 0 : Number(v));

  return (
    <div>
      <PageHeader
        title="Project"
        subtitle="Gebouw- en installatie-instellingen"
        breadcrumbs={[{ label: "Project" }]}
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

      <div className="space-y-6 p-6">
        {/* Project info */}
        <Card title="Projectgegevens">
          <div className="grid grid-cols-2 gap-4">
            <Input
              id="name"
              label="Projectnaam"
              value={info.name}
              onChange={(e) => updateInfo({ name: e.target.value })}
            />
            <Input
              id="project_number"
              label="Projectnummer"
              value={info.project_number ?? ""}
              onChange={(e) => updateInfo({ project_number: e.target.value || null })}
            />
            <Input
              id="address"
              label="Adres"
              value={info.address ?? ""}
              onChange={(e) => updateInfo({ address: e.target.value || null })}
            />
            <Input
              id="client"
              label="Opdrachtgever"
              value={info.client ?? ""}
              onChange={(e) => updateInfo({ client: e.target.value || null })}
            />
            <Input
              id="engineer"
              label="Berekend door"
              value={info.engineer ?? ""}
              onChange={(e) => updateInfo({ engineer: e.target.value || null })}
            />
            <Input
              id="date"
              label="Datum"
              type="date"
              value={info.date ?? ""}
              onChange={(e) => updateInfo({ date: e.target.value || null })}
            />
          </div>
        </Card>

        {/* Building */}
        <Card title="Gebouw">
          <div className="grid grid-cols-3 gap-4">
            <Select
              id="building_type"
              label="Gebouwtype"
              value={building.building_type}
              options={toOptions(BUILDING_TYPE_LABELS)}
              onChange={(e) =>
                updateBuilding({ building_type: e.target.value as Building["building_type"] })
              }
            />
            <Input
              id="qv10"
              label="Luchtdichtheid qv10"
              type="number"
              unit="dm³/s"
              value={building.qv10}
              onChange={(e) => updateBuilding({ qv10: numVal(e.target.value) })}
            />
            <Input
              id="total_floor_area"
              label="Gebruiksoppervlak"
              type="number"
              unit="m²"
              value={building.total_floor_area}
              onChange={(e) => updateBuilding({ total_floor_area: numVal(e.target.value) })}
            />
            <Select
              id="security_class"
              label="Zekerheidsklasse"
              value={building.security_class}
              options={toOptions(SECURITY_CLASS_LABELS)}
              onChange={(e) =>
                updateBuilding({ security_class: e.target.value as Building["security_class"] })
              }
            />
            <Input
              id="num_floors"
              label="Aantal verdiepingen"
              type="number"
              value={building.num_floors ?? 1}
              onChange={(e) => updateBuilding({ num_floors: Math.max(1, numVal(e.target.value)) })}
            />
            <Input
              id="warmup_time"
              label="Opwarmtijd"
              type="number"
              unit="uur"
              value={building.warmup_time ?? 2}
              onChange={(e) => updateBuilding({ warmup_time: numVal(e.target.value) })}
            />
          </div>
          <div className="mt-3">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={building.has_night_setback ?? false}
                onChange={(e) => updateBuilding({ has_night_setback: e.target.checked })}
                className="rounded border-[var(--oaec-border)] accent-primary"
              />
              Nachtreductie
            </label>
          </div>
          <div className="mt-4 grid grid-cols-3 gap-4 border-t border-[var(--oaec-border-subtle)] pt-4">
            <div>
              <Input
                id="frame_u_override"
                label="U-waarde kozijnen (override)"
                type="number"
                unit="W/(m²·K)"
                step={0.1}
                min={0}
                max={10}
                value={project.frameUValueOverride ?? ""}
                onChange={(e) => {
                  const raw = e.target.value;
                  if (raw === "") {
                    setFrameUValueOverride(undefined);
                  } else {
                    setFrameUValueOverride(Number(raw));
                  }
                }}
              />
              <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                Leeg laten voor individuele waarden per element. Vervangt
                in de berekening alle U-waarden van kozijnen en vullingen
                (categorie kozijnen_vullingen) in één keer.
              </p>
            </div>
          </div>
        </Card>

        {/* Climate */}
        <Card title="Klimaat (ontwerpcondities)">
          <div className="grid grid-cols-4 gap-4">
            <Input
              id="theta_e"
              label="Buitentemperatuur θ_e"
              type="number"
              unit="°C"
              value={climate.theta_e ?? -10}
              onChange={(e) => updateClimate({ theta_e: numVal(e.target.value) })}
            />
            <Input
              id="theta_b_res"
              label="Buurwoning θ_b (wonen)"
              type="number"
              unit="°C"
              value={climate.theta_b_residential ?? 17}
              onChange={(e) => updateClimate({ theta_b_residential: numVal(e.target.value) })}
            />
            <Input
              id="theta_b_nonres"
              label="Buurwoning θ_b (overig)"
              type="number"
              unit="°C"
              value={climate.theta_b_non_residential ?? 14}
              onChange={(e) => updateClimate({ theta_b_non_residential: numVal(e.target.value) })}
            />
            <Input
              id="wind_factor"
              label="Windfactor"
              type="number"
              value={climate.wind_factor ?? 1.0}
              onChange={(e) => updateClimate({ wind_factor: numVal(e.target.value) })}
            />
            <Input
              id="theta_water"
              label="Watertemperatuur θ_w"
              type="number"
              unit="°C"
              value={climate.theta_water ?? DEFAULT_THETA_WATER}
              onChange={(e) => updateClimate({ theta_water: numVal(e.target.value) })}
            />
          </div>
          <p className="mt-2 text-xs text-on-surface-muted">
            Watertemperatuur is een engineering-aanname voor grensvlakken aan water
            (bv. woonboten). Geen norm-waarde; default {DEFAULT_THETA_WATER} °C is
            conservatief voor Nederlandse binnenwateren in winterconditie. Komt
            automatisch terug in het PDF-rapport als er water-grensvlakken in het
            project zitten.
          </p>
        </Card>

        {/* Ventilation */}
        <Card title="Ventilatie">
          <div className="grid grid-cols-3 gap-4">
            <Select
              id="system_type"
              label="Ventilatiesysteem"
              value={ventilation.system_type}
              options={toOptions(VENTILATION_SYSTEM_LABELS)}
              onChange={(e) =>
                updateVentilation({
                  system_type: e.target.value as VentilationConfig["system_type"],
                })
              }
            />
            <div className="flex items-end">
              <label className="flex items-center gap-2 pb-1.5 text-sm">
                <input
                  type="checkbox"
                  checked={ventilation.has_heat_recovery ?? false}
                  onChange={(e) => updateVentilation({ has_heat_recovery: e.target.checked })}
                  className="rounded border-[var(--oaec-border)] accent-primary"
                />
                Warmteterugwinning (WTW)
              </label>
            </div>
            {ventilation.has_heat_recovery && (
              <Input
                id="heat_recovery_efficiency"
                label="WTW-rendement"
                type="number"
                unit="%"
                value={
                  ventilation.heat_recovery_efficiency != null
                    ? ventilation.heat_recovery_efficiency * 100
                    : 85
                }
                onChange={(e) =>
                  updateVentilation({
                    heat_recovery_efficiency: numVal(e.target.value) / 100,
                  })
                }
              />
            )}
          </div>
        </Card>

        {/* Rooms hint */}
        {project.rooms.length === 0 && (
          <Card>
            <div className="flex flex-col items-center gap-2 py-2">
              <p className="text-sm text-on-surface-muted">
                Voeg vertrekken toe om de berekening te kunnen starten.
              </p>
              <Button variant="secondary" size="sm" onClick={() => navigate("/rooms")}>
                Vertrekken invoeren
              </Button>
            </div>
          </Card>
        )}

        {/* Room count summary */}
        {project.rooms.length > 0 && (
          <Card title={`Vertrekken (${project.rooms.length})`}>
            <ul className="space-y-1">
              {project.rooms.map((room) => (
                <li
                  key={room.id}
                  className="flex items-center justify-between rounded px-2 py-1 text-sm hover:bg-[var(--oaec-hover)]"
                >
                  <span className="font-medium">{room.name}</span>
                  <span className="font-mono text-xs text-on-surface-muted">
                    {formatArea(room.floor_area)} m²
                  </span>
                </li>
              ))}
            </ul>
          </Card>
        )}
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".json"
        className="hidden"
        onChange={handleImportFile}
      />
    </div>
  );
}
