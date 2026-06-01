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
  Isso53BuildingPosition,
  Isso53BuildingShape,
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
    </Card>
  );
}
