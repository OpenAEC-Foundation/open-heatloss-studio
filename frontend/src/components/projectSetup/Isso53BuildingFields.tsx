/**
 * ISSO 53 gebouw-niveau invoervelden (fase 3).
 *
 * Vervangt de V1 `BuildingType`-selector in `AlgemeenTab` wanneer de
 * actieve norm `"isso53"` is. Schrijft naar `isso53Building` sidecar
 * in de store; raakt V1 `project.building` niet aan.
 *
 * Bron-tabellen:
 * - `BuildingShape` — ISSO 53 tabel 4.9 (infiltratie vormfactor)
 * - `BuildingPosition` — ISSO 53 tabel 4.8 (positie in complex)
 * - `WindPressureType` — ISSO 53 tabel 4.6 (winddruk f_type)
 */
import { useTranslation } from "react-i18next";

import { Card } from "../ui/Card";
import { Input } from "../ui/Input";
import { Select } from "../ui/Select";
import { useProjectStore } from "../../store/projectStore";
import type {
  Isso53AirChangeRate,
  Isso53BuildingPosition,
  Isso53BuildingShape,
  Isso53CoolingRegimeType,
  Isso53HeatingUpState,
  Isso53ThermalMass,
  Isso53VentilationSystem,
  Isso53WindPressureType,
  Qv10Class,
} from "../../types/projectV2";

const BUILDING_SHAPES: Isso53BuildingShape[] = [
  "meerlaags",
  "eenLaagMetKap",
  "eenLaagMetPlatDak",
  "eenLaagMetHalfPlatDak",
];

const BUILDING_POSITIONS: Isso53BuildingPosition[] = [
  "enkellaagsTussen",
  "enkellaagsKop",
  "enkellaagsVrijstaand",
  "meerlaagsGeheel",
  "meerlaagsTop",
  "meerlaagsTussen",
  "meerlaagsOnder",
];

const WIND_PRESSURE_TYPES: Isso53WindPressureType[] = [
  "eenlaagsMetKap",
  "eenlaagsMetPlatDak",
  "meerlaagsStandaard",
  "meerlaagsVolgevelBinnengalerij",
  "meerlaagsDubbeleHuidOnderbroken",
  "meerlaagsDubbeleHuidDoorlopend",
];

const THERMAL_MASSES: Isso53ThermalMass[] = ["licht", "gemiddeld", "zwaar"];

const VENTILATION_SYSTEMS: Isso53VentilationSystem[] = [
  "systemA",
  "systemB",
  "systemC",
  "systemD",
  "systemE",
];

const COOLING_REGIMES: Isso53CoolingRegimeType[] = ["free", "limited"];

const AIR_CHANGE_RATES: Isso53AirChangeRate[] = ["low", "high"];

const QV10_CLASSES: Qv10Class[] = [
  "LessThan020",
  "From020To040",
  "From040To060",
  "From060To080",
  "From080To100",
  "GreaterThan100",
];

export function Isso53BuildingFields() {
  const { t } = useTranslation();
  const isso53Building = useProjectStore((s) => s.isso53Building);
  const updateIsso53Building = useProjectStore((s) => s.updateIsso53Building);

  const shapeOptions = BUILDING_SHAPES.map((v) => ({
    value: v,
    label: t(`isso53.building.shapeOptions.${v}`),
  }));
  const positionOptions = BUILDING_POSITIONS.map((v) => ({
    value: v,
    label: t(`isso53.building.positionOptions.${v}`),
  }));
  const windOptions = WIND_PRESSURE_TYPES.map((v) => ({
    value: v,
    label: t(`isso53.building.windPressureOptions.${v}`),
  }));
  const thermalOptions = THERMAL_MASSES.map((v) => ({
    value: v,
    label: t(`isso53.building.thermalMassOptions.${v}`),
  }));
  const ventOptions = VENTILATION_SYSTEMS.map((v) => ({
    value: v,
    label: t(`isso53.building.ventilationSystemOptions.${v}`),
  }));
  const qv10Options = QV10_CLASSES.map((v) => ({
    value: v,
    label: t(`isso53.building.qv10ClassOptions.${v}`),
  }));
  const regimeOptions = COOLING_REGIMES.map((v) => ({
    value: v,
    label: t(`isso53.building.coolingRegimeOptions.${v}`),
  }));
  const airChangeOptions = AIR_CHANGE_RATES.map((v) => ({
    value: v,
    label: t(`isso53.building.airChangeOptions.${v}`),
  }));

  /** Partial merge op de heatingUp-sidecar (behoudt overige velden). */
  const updateHeatingUp = (partial: Partial<Isso53HeatingUpState>) =>
    updateIsso53Building({
      heatingUp: { ...isso53Building.heatingUp, ...partial },
    });

  return (
    <Card title={t("isso53.building.sectionTitle")}>
      <div className="grid grid-cols-2 gap-4">
        <Select
          id="isso53_building_shape"
          label={t("isso53.building.shape")}
          value={isso53Building.buildingShape}
          options={shapeOptions}
          onChange={(e) =>
            updateIsso53Building({
              buildingShape: e.target.value as Isso53BuildingShape,
            })
          }
        />
        <Select
          id="isso53_building_position"
          label={t("isso53.building.position")}
          value={isso53Building.buildingPosition}
          options={positionOptions}
          onChange={(e) =>
            updateIsso53Building({
              buildingPosition: e.target.value as Isso53BuildingPosition,
            })
          }
        />
        <Select
          id="isso53_wind_pressure"
          label={t("isso53.building.windPressureType")}
          value={isso53Building.windPressureType}
          options={windOptions}
          onChange={(e) =>
            updateIsso53Building({
              windPressureType: e.target.value as Isso53WindPressureType,
            })
          }
        />
        <Select
          id="isso53_thermal_mass"
          label={t("isso53.building.thermalMass")}
          value={isso53Building.thermalMass}
          options={thermalOptions}
          onChange={(e) =>
            updateIsso53Building({
              thermalMass: e.target.value as Isso53ThermalMass,
            })
          }
        />
        <Select
          id="isso53_ventilation_system"
          label={t("isso53.building.ventilationSystem")}
          value={isso53Building.ventilationSystem}
          options={ventOptions}
          onChange={(e) =>
            updateIsso53Building({
              ventilationSystem: e.target.value as Isso53VentilationSystem,
            })
          }
        />
        <Input
          id="isso53_construction_year"
          label={t("isso53.building.constructionYear")}
          type="number"
          value={isso53Building.constructionYear ?? ""}
          onChange={(e) =>
            updateIsso53Building({
              constructionYear:
                e.target.value === "" ? null : Number(e.target.value),
            })
          }
        />
        <div>
          <Input
            id="isso53_theta_me"
            label={t("isso53.building.thetaMe")}
            type="number"
            step="0.1"
            unit="°C"
            value={isso53Building.thetaMe}
            onChange={(e) =>
              updateIsso53Building({ thetaMe: Number(e.target.value) })
            }
          />
          <p className="mt-1 text-xs text-on-surface-muted">
            {t("isso53.building.thetaMeHint")}
          </p>
        </div>
        <Select
          id="isso53_qv10_class"
          label={t("isso53.building.qv10Class")}
          value={isso53Building.qv10KarClass}
          options={qv10Options}
          onChange={(e) =>
            updateIsso53Building({
              qv10KarClass: e.target.value as Qv10Class,
            })
          }
        />
      </div>

      <div className="mt-6 border-t border-[var(--oaec-border)] pt-4">
        <h4 className="mb-3 text-sm font-semibold text-on-surface-secondary">
          {t("isso53.building.heatingUpTitle")}
        </h4>
        <label
          htmlFor="isso53_setback_active"
          className="flex items-center gap-2 text-sm text-on-surface"
        >
          <input
            id="isso53_setback_active"
            type="checkbox"
            checked={isso53Building.heatingUp.setbackActive}
            onChange={(e) => updateHeatingUp({ setbackActive: e.target.checked })}
            className="h-4 w-4 rounded border-[1.5px] border-[var(--oaec-border)]
              accent-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
          />
          {t("isso53.building.setbackActive")}
        </label>

        {isso53Building.heatingUp.setbackActive && (
          <div className="mt-4 grid grid-cols-2 gap-4">
            <Select
              id="isso53_cooling_regime"
              label={t("isso53.building.coolingRegime")}
              value={isso53Building.heatingUp.regimeType}
              options={regimeOptions}
              onChange={(e) =>
                updateHeatingUp({
                  regimeType: e.target.value as Isso53CoolingRegimeType,
                })
              }
            />
            <Select
              id="isso53_air_changes"
              label={t("isso53.building.airChanges")}
              value={isso53Building.heatingUp.airChanges}
              options={airChangeOptions}
              onChange={(e) =>
                updateHeatingUp({
                  airChanges: e.target.value as Isso53AirChangeRate,
                })
              }
            />
            <Input
              id="isso53_warmup_weekday"
              label={t("isso53.building.warmupHoursWeekday")}
              type="number"
              step="0.5"
              min="0"
              unit="h"
              value={isso53Building.heatingUp.warmupHoursWeekday}
              onChange={(e) =>
                updateHeatingUp({ warmupHoursWeekday: Number(e.target.value) })
              }
            />
            <Input
              id="isso53_warmup_weekend"
              label={t("isso53.building.warmupHoursWeekend")}
              type="number"
              step="0.5"
              min="0"
              unit="h"
              value={isso53Building.heatingUp.warmupHoursWeekend}
              onChange={(e) =>
                updateHeatingUp({ warmupHoursWeekend: Number(e.target.value) })
              }
            />

            {isso53Building.heatingUp.regimeType === "free" ? (
              <>
                <Input
                  id="isso53_setback_hours_weekday"
                  label={t("isso53.building.setbackHoursWeekday")}
                  type="number"
                  step="1"
                  min="0"
                  unit="h"
                  value={isso53Building.heatingUp.setbackHoursWeekday}
                  onChange={(e) =>
                    updateHeatingUp({
                      setbackHoursWeekday: Number(e.target.value),
                    })
                  }
                />
                <Input
                  id="isso53_setback_hours_weekend"
                  label={t("isso53.building.setbackHoursWeekend")}
                  type="number"
                  step="1"
                  min="0"
                  unit="h"
                  value={isso53Building.heatingUp.setbackHoursWeekend}
                  onChange={(e) =>
                    updateHeatingUp({
                      setbackHoursWeekend: Number(e.target.value),
                    })
                  }
                />
              </>
            ) : (
              <>
                <Input
                  id="isso53_degrees_weekday"
                  label={t("isso53.building.degreesWeekday")}
                  type="number"
                  step="1"
                  min="1"
                  max="5"
                  unit="K"
                  value={isso53Building.heatingUp.degreesWeekday}
                  onChange={(e) =>
                    updateHeatingUp({ degreesWeekday: Number(e.target.value) })
                  }
                />
                <Input
                  id="isso53_degrees_weekend"
                  label={t("isso53.building.degreesWeekend")}
                  type="number"
                  step="1"
                  min="1"
                  max="5"
                  unit="K"
                  value={isso53Building.heatingUp.degreesWeekend}
                  onChange={(e) =>
                    updateHeatingUp({ degreesWeekend: Number(e.target.value) })
                  }
                />
              </>
            )}

            <label
              htmlFor="isso53_mech_supply_off"
              className="col-span-2 flex items-center gap-2 text-sm text-on-surface"
            >
              <input
                id="isso53_mech_supply_off"
                type="checkbox"
                checked={isso53Building.heatingUp.mechanicalSupplyOff}
                onChange={(e) =>
                  updateHeatingUp({ mechanicalSupplyOff: e.target.checked })
                }
                className="h-4 w-4 rounded border-[1.5px] border-[var(--oaec-border)]
                  accent-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              />
              {t("isso53.building.mechanicalSupplyOff")}
            </label>

            <div className="col-span-2">
              <Input
                id="isso53_heating_up_override"
                label={t("isso53.building.heatingUpOverride")}
                type="number"
                step="0.1"
                min="0"
                unit="W/m²"
                value={isso53Building.heatingUp.pWPerM2Override ?? ""}
                onChange={(e) =>
                  updateHeatingUp({
                    pWPerM2Override:
                      e.target.value === "" ? null : Number(e.target.value),
                  })
                }
              />
              <p className="mt-1 text-xs text-on-surface-muted">
                {t("isso53.building.heatingUpOverrideHint")}
              </p>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
