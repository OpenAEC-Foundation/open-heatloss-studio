import { useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Input } from "../components/ui/Input";
import { Select } from "../components/ui/Select";
import { PageHeader } from "../components/layout/PageHeader";
import { useNormSwitch } from "../components/layout/NormSwitchContext";
import { VentilationPanel } from "../components/projectSetup/VentilationPanel";
import { useBackend } from "../hooks/useBackend";
import { useProjectStore } from "../store/projectStore";
import { formatArea } from "../lib/formatNumber";
import { prepareProjectForCalculation } from "../lib/frameOverride";
import { buildV2PayloadIsso53 } from "../lib/projectV2Migration";
import { useModellerStore } from "../components/modeller/modellerStore";
import { useToastStore } from "../store/toastStore";
import {
  AGGREGATION_METHOD_LABELS,
  BUILDING_TYPE_LABELS,
  DEFAULT_AGGREGATION_METHOD,
  DEFAULT_THETA_WATER,
  FROST_PROTECTION_LABELS,
  FROST_PROTECTION_SUPPLY_TEMP,
  getHeatingSystemLabels,
  SECURITY_CLASS_LABELS,
} from "../lib/constants";
import type {
  AggregationMethod,
  Building,
  DesignConditions,
  FrostProtectionType,
  HeatingSystem,
  VentilationConfig,
} from "../types";

const BULK_APPLY_CONFIRM_THRESHOLD = 5;
const DEFAULT_HEATING_SYSTEM_ISSO51: HeatingSystem = "radiator_ht";
const DEFAULT_HEATING_SYSTEM_ISSO53: HeatingSystem = "radiatorenConvHtEnLuchtverwarming";

function toOptions(labels: Record<string, string>) {
  return Object.entries(labels).map(([value, label]) => ({ value, label }));
}

export function WarmteverliesInstellingen() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const backend = useBackend();
  const {
    project,
    norm,
    sharedExtra,
    isso53Building,
    isso53Rooms,
    updateProject,
    isCalculating,
    setCalculating,
    setResult,
    setError,
    setFrameUValueOverride,
    applyHeatingSystemToAllRooms,
    setAggregationMethod,
  } = useProjectStore();
  const { openNormSwitch } = useNormSwitch();
  const projectConstructions = useModellerStore((s) => s.projectConstructions);
  const addToast = useToastStore((s) => s.addToast);

  const { building, climate, ventilation } = project;

  // Norm-aware afgeleide constanten — bepalen welke verwarmingssysteem-
  // set en welke labels we tonen, en welke tabel-tooltip de norm-referentie
  // krijgt. Default verwarmingssysteem volgt de norm-specifieke fallback.
  const isIsso53 = norm === "isso53";
  const heatingLabels = getHeatingSystemLabels(isIsso53 ? "isso53" : "isso51");
  const defaultHeatingSystem = isIsso53
    ? DEFAULT_HEATING_SYSTEM_ISSO53
    : DEFAULT_HEATING_SYSTEM_ISSO51;
  const heatingTableRef = isIsso53 ? "ISSO 53 Tabel 2.3" : "ISSO 51 Tabel 2.12";
  const neighbourResidentialLabel = isIsso53
    ? "Buurgebouw θ_b (verwarmd)"
    : "Buurwoning θ_b (wonen)";
  const neighbourNonResidentialLabel = isIsso53
    ? "Buurgebouw θ_b (onverwarmd)"
    : "Buurwoning θ_b (overig)";

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
      if (norm === "isso53") {
        // ISSO 53 routeert via de V2-payload (active_norm → Isso53) naar
        // de calculate_v2-kern. De isso51-route crasht op de camelCase
        // verwarmingssysteem-enum van ISSO 53.
        const payload = buildV2PayloadIsso53(
          project,
          sharedExtra,
          isso53Building,
          isso53Rooms,
        );
        const result = await backend.calculateV2(payload);
        setResult(result);
      } else {
        const payload = prepareProjectForCalculation(project, projectConstructions);
        const result = await backend.calculate(payload);
        setResult(result);
      }
      navigate("/results");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Berekening mislukt");
    }
  }, [
    backend,
    norm,
    project,
    sharedExtra,
    isso53Building,
    isso53Rooms,
    projectConstructions,
    setCalculating,
    setResult,
    setError,
    navigate,
  ]);

  const numVal = (v: string) => (v === "" ? 0 : Number(v));

  const title = t("warmteverliesInstellingen.title", "Warmteverlies-instellingen");
  const subtitle = t(
    "warmteverliesInstellingen.subtitle",
    isIsso53 ? "ISSO 53:2024 parameters" : "ISSO 51:2023 parameters",
  );

  return (
    <div>
      <PageHeader
        title={title}
        subtitle={subtitle}
        actions={
          <Button
            onClick={handleCalculate}
            disabled={isCalculating || project.rooms.length === 0}
          >
            {isCalculating ? "Berekenen..." : "Berekenen"}
          </Button>
        }
      />

      <div className="space-y-6 p-6">
        {/* Actieve rekennorm — wissel-trigger (ISSO 51 ↔ 53).
            Voorheen verstopt in Backstage onder Voorkeuren; verhuisd naar
            de instellingen-pagina omdat reken-instellingen hier thuis-
            horen. Zie sessie 2026-05-26. */}
        <Card>
          <div className="flex items-center justify-between gap-4">
            <div>
              <div className="text-xs uppercase tracking-wider text-on-surface-muted">
                Actieve rekennorm
              </div>
              <div className="mt-0.5 text-lg font-semibold text-on-surface">
                {norm === "isso53"
                  ? "ISSO 53 — Utiliteit"
                  : "ISSO 51 — Woningen"}
              </div>
              <p className="mt-1 text-[11px] leading-tight text-on-surface-muted">
                Wisselen converteert je projectdata en maakt een backup.
                Eén project kan slechts één rekennorm hanteren.
              </p>
            </div>
            <Button variant="secondary" onClick={openNormSwitch}>
              Norm wisselen…
            </Button>
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
              id="total_floor_area"
              label="Gebruiksoppervlak Ag"
              type="number"
              unit="m²"
              value={building.total_floor_area}
              onChange={(e) => updateBuilding({ total_floor_area: numVal(e.target.value) })}
            />
            <Input
              id="qv10"
              label="Luchtdichtheid qv10 (totaal)"
              type="number"
              unit="dm³/s"
              value={building.qv10}
              onChange={(e) => updateBuilding({ qv10: numVal(e.target.value) })}
            />
            <Input
              id="qv10_spec"
              label="qv10;spec (BENG)"
              type="number"
              step={0.01}
              unit="dm³/(s·m²)"
              value={
                building.total_floor_area > 0
                  ? Number((building.qv10 / building.total_floor_area).toFixed(3))
                  : ""
              }
              onChange={(e) => {
                const spec = numVal(e.target.value);
                if (building.total_floor_area > 0) {
                  updateBuilding({ qv10: spec * building.total_floor_area });
                }
              }}
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
            <div>
              <Select
                id="default_heating_system"
                label="Standaard verwarmingssysteem"
                value={building.default_heating_system ?? defaultHeatingSystem}
                options={toOptions(heatingLabels)}
                onChange={(e) =>
                  updateBuilding({
                    default_heating_system: e.target.value as HeatingSystem,
                  })
                }
              />
              <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                Wordt gebruikt bij nieuwe vertrekken. Gebruik de knop hieronder
                om dit systeem op alle bestaande vertrekken toe te passen.
                Bepaalt Δθ₁/Δθ₂/Δθᵥ correcties ({heatingTableRef}).
              </p>
            </div>
          </div>
          <div className="mt-3 flex items-center gap-4">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={building.has_night_setback ?? false}
                onChange={(e) => updateBuilding({ has_night_setback: e.target.checked })}
                className="rounded border-[var(--oaec-border)] accent-primary"
              />
              Nachtreductie
            </label>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                const system =
                  building.default_heating_system ?? defaultHeatingSystem;
                const count = project.rooms.length;
                if (count === 0) {
                  addToast("Geen vertrekken om aan te passen", "info", 2000);
                  return;
                }
                if (count > BULK_APPLY_CONFIRM_THRESHOLD) {
                  const label = heatingLabels[system] ?? system;
                  if (
                    !window.confirm(
                      `Weet je zeker dat je "${label}" wilt toepassen op alle ${count} vertrekken? Dit overschrijft eventuele per-vertrek afwijkingen.`,
                    )
                  ) {
                    return;
                  }
                }
                applyHeatingSystemToAllRooms(system);
                addToast(
                  `Verwarmingssysteem toegepast op ${count} vertrekken`,
                  "success",
                  2500,
                );
              }}
            >
              Toepassen op alle vertrekken
            </Button>
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
            <div className="col-span-2">
              <Select
                id="aggregation_method"
                label="Aggregatiemethode"
                value={
                  building.aggregation_method ?? DEFAULT_AGGREGATION_METHOD
                }
                options={toOptions(AGGREGATION_METHOD_LABELS)}
                onChange={(e) =>
                  setAggregationMethod(e.target.value as AggregationMethod)
                }
              />
              <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                Bepaalt hoe Φ_T,iae op gebouwniveau wordt geaggregeerd.
                Vabi-conform sluit transmissie via onverwarmde ruimtes uit
                van het basis-verlies op gebouwniveau (markt-conventie).
                Norm-strict volgt §3.5.1 letterlijk en geeft ~17% hoger
                aansluitvermogen.
              </p>
            </div>
          </div>
        </Card>

        {/* V2 Ventilation (TO-juli / NTA 8800) — sidecar + V1 spiegel */}
        <VentilationPanel />

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
              label={neighbourResidentialLabel}
              type="number"
              unit="°C"
              value={climate.theta_b_residential ?? 17}
              onChange={(e) => updateClimate({ theta_b_residential: numVal(e.target.value) })}
            />
            <Input
              id="theta_b_nonres"
              label={neighbourNonResidentialLabel}
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

        {/* WTW vorstbeveiliging — alleen tonen als heat recovery actief */}
        {ventilation.has_heat_recovery && (
          <Card title="WTW vorstbeveiliging">
            <div className="grid grid-cols-3 gap-4">
              <Select
                id="frost_protection"
                label="Vorstbeveiliging"
                value={ventilation.frost_protection ?? "unknown"}
                options={toOptions(FROST_PROTECTION_LABELS)}
                onChange={(e) =>
                  updateVentilation({
                    frost_protection: e.target.value as FrostProtectionType,
                  })
                }
              />
              <div>
                <Input
                  id="supply_temperature"
                  label="Toevoertemperatuur θ_t"
                  type="number"
                  unit="°C"
                  value={
                    ventilation.supply_temperature ??
                    FROST_PROTECTION_SUPPLY_TEMP[ventilation.frost_protection ?? "unknown"] ??
                    10
                  }
                  onChange={(e) =>
                    updateVentilation({
                      supply_temperature: e.target.value === "" ? null : numVal(e.target.value),
                    })
                  }
                />
                <p className="mt-1 text-[10px] leading-tight text-on-surface-muted">
                  ISSO 51 Tabel 2.14 (erratum). Wordt automatisch bepaald op basis
                  van vorstbeveiliging. Handmatig aanpassen overschrijft de tabelwaarde.
                </p>
              </div>
              <div className="flex items-center">
                <div className="rounded-md bg-[var(--oaec-accent-soft)] px-3 py-2 text-sm">
                  <span className="text-on-surface-muted">ΔT ventilatie: </span>
                  <span className="font-semibold tabular-nums">
                    {20 - (ventilation.supply_temperature ?? FROST_PROTECTION_SUPPLY_TEMP[ventilation.frost_protection ?? "unknown"] ?? 10)}
                  </span>
                  <span className="text-on-surface-muted"> K</span>
                  <span className="ml-2 text-xs text-on-surface-muted">(bij θ_i = 20°C)</span>
                </div>
              </div>
            </div>
          </Card>
        )}

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
    </div>
  );
}
