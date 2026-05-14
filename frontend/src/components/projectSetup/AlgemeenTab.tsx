/**
 * Algemeen-tab — SharedProject metadata, locatie, gebouwtype.
 *
 * Schrijft naar V1 `project.info` + `project.building.{total_floor_area,
 * num_floors}` plus V2-sidecar (`sharedExtra`) voor postcode/location/notes/
 * construction_year/building_type kind+subtype.
 */
import { useCallback, type ChangeEvent } from "react";
import { useTranslation } from "react-i18next";

import { Card } from "../ui/Card";
import { Input } from "../ui/Input";
import { Select } from "../ui/Select";
import { useToastStore } from "../../store/toastStore";
import { useProjectStore } from "../../store/projectStore";
import { BUILDING_TYPE_LABELS } from "../../lib/constants";
import { buildingTypeV1ToV2 } from "../../lib/projectV2Migration";
import type {
  BuildingType,
  CoverImage,
  ProjectInfo,
} from "../../types";
import type {
  BuildingTypeShared,
  ResidentialType,
  UtilityType,
} from "../../types/projectV2";

const MAX_COVER_IMAGE_BYTES = 2 * 1024 * 1024;
const ALLOWED_COVER_IMAGE_TYPES: ReadonlyArray<CoverImage["media_type"]> = [
  "image/png",
  "image/jpeg",
];

const RESIDENTIAL_SUBTYPES: ResidentialType[] = [
  "detached",
  "semi_detached",
  "terraced",
  "end_of_terrace",
  "porch",
  "gallery",
  "stacked",
];

const UTILITY_SUBTYPES: UtilityType[] = [
  "office",
  "education",
  "assembly",
  "healthcare",
  "lodging",
  "sport",
  "retail",
  "industrial",
  "other",
];

function toOptions(labels: Record<string, string>) {
  return Object.entries(labels).map(([value, label]) => ({ value, label }));
}

function numVal(v: string): number {
  return v === "" ? 0 : Number(v);
}

export function AlgemeenTab() {
  const { t } = useTranslation();
  const { project, updateProject, sharedExtra, updateSharedExtra } = useProjectStore();
  const addToast = useToastStore((s) => s.addToast);

  const info = project.info;
  const building = project.building;

  const updateInfo = useCallback(
    (partial: Partial<ProjectInfo>) => {
      updateProject({ info: { ...project.info, ...partial } });
    },
    [project.info, updateProject],
  );

  // Building type — kind (woning/utiliteit) is V2-only via sidecar; subtype
  // schrijft naar V1 building.building_type voor woning, en blijft V2-only
  // voor utiliteit (V1 model heeft daar geen begrip van).
  const buildingType: BuildingTypeShared =
    sharedExtra.building_type ?? buildingTypeV1ToV2(building.building_type);

  const onKindChange = (kind: "woning" | "utiliteit") => {
    if (kind === "woning") {
      const subtype: ResidentialType =
        buildingType.kind === "woning" ? buildingType.subtype : "terraced";
      updateSharedExtra({ building_type: { kind, subtype } });
      updateProject({
        building: { ...building, building_type: subtype as BuildingType },
      });
    } else {
      const subtype: UtilityType =
        buildingType.kind === "utiliteit" ? buildingType.subtype : "office";
      updateSharedExtra({ building_type: { kind, subtype } });
      // V1 building.building_type heeft geen utiliteit-variant — laat staan
      // op huidige woning-type voor backward-compat ISSO 51 berekening.
    }
  };

  const onSubtypeChange = (subtype: string) => {
    if (buildingType.kind === "woning") {
      updateSharedExtra({
        building_type: { kind: "woning", subtype: subtype as ResidentialType },
      });
      updateProject({
        building: { ...building, building_type: subtype as BuildingType },
      });
    } else {
      updateSharedExtra({
        building_type: { kind: "utiliteit", subtype: subtype as UtilityType },
      });
    }
  };

  const handleCoverImageChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const input = e.target;
      const file = input.files?.[0];
      if (!file) return;
      if (file.size > MAX_COVER_IMAGE_BYTES) {
        addToast("Afbeelding mag maximaal 2 MB zijn", "error");
        input.value = "";
        return;
      }
      const isAllowedType = ALLOWED_COVER_IMAGE_TYPES.some((tp) => tp === file.type);
      if (!isAllowedType) {
        addToast("Alleen PNG of JPEG toegestaan", "error");
        input.value = "";
        return;
      }
      const reader = new FileReader();
      reader.onload = () => {
        const dataUrl = reader.result as string;
        const base64 = dataUrl.split(",")[1] ?? "";
        if (!base64) {
          addToast("Kon de afbeelding niet lezen", "error");
          return;
        }
        updateInfo({
          cover_image: {
            data: base64,
            media_type: file.type as CoverImage["media_type"],
            filename: file.name,
          },
        });
      };
      reader.readAsDataURL(file);
      input.value = "";
    },
    [addToast, updateInfo],
  );

  const handleCoverImageClear = useCallback(() => {
    updateInfo({ cover_image: null });
  }, [updateInfo]);

  const subtypeLabels: Record<string, string> =
    buildingType.kind === "woning"
      ? Object.fromEntries(
          RESIDENTIAL_SUBTYPES.map((s) => [
            s,
            BUILDING_TYPE_LABELS[s] ?? s,
          ]),
        )
      : Object.fromEntries(
          UTILITY_SUBTYPES.map((s) => [s, t(`projectSetup.utilitySubtype.${s}`, s)]),
        );

  return (
    <div className="space-y-6">
      <Card title={t("projectSetup.sections.projectInfo", "Projectgegevens")}>
        <div className="grid grid-cols-2 gap-4">
          <Input
            id="name"
            label={t("projectSetup.fields.name", "Projectnaam")}
            value={info.name}
            onChange={(e) => updateInfo({ name: e.target.value })}
          />
          <Input
            id="project_number"
            label={t("projectSetup.fields.project_number", "Projectnummer")}
            value={info.project_number ?? ""}
            onChange={(e) => updateInfo({ project_number: e.target.value || null })}
          />
          <Input
            id="address"
            label={t("projectSetup.fields.address", "Adres")}
            value={info.address ?? ""}
            onChange={(e) => updateInfo({ address: e.target.value || null })}
          />
          <Input
            id="postcode"
            label={t("projectSetup.fields.postcode", "Postcode")}
            value={sharedExtra.postcode ?? ""}
            onChange={(e) =>
              updateSharedExtra({ postcode: e.target.value || null })
            }
          />
          <Input
            id="location"
            label={t("projectSetup.fields.location", "Plaats")}
            value={sharedExtra.location ?? ""}
            onChange={(e) =>
              updateSharedExtra({ location: e.target.value || null })
            }
          />
          <Input
            id="client"
            label={t("projectSetup.fields.client", "Opdrachtgever")}
            value={info.client ?? ""}
            onChange={(e) => updateInfo({ client: e.target.value || null })}
          />
          <Input
            id="engineer"
            label={t("projectSetup.fields.engineer", "Berekend door")}
            value={info.engineer ?? ""}
            onChange={(e) => updateInfo({ engineer: e.target.value || null })}
          />
          <Input
            id="date"
            label={t("projectSetup.fields.date", "Datum")}
            type="date"
            value={info.date ?? ""}
            onChange={(e) => updateInfo({ date: e.target.value || null })}
          />
        </div>
        <div className="mt-4 border-t border-[var(--oaec-border-subtle)] pt-4">
          <label
            className="block text-xs font-medium text-on-surface-secondary"
            htmlFor="notes"
          >
            {t("projectSetup.fields.notes", "Notities")}
          </label>
          <textarea
            id="notes"
            value={sharedExtra.notes ?? ""}
            onChange={(e) =>
              updateSharedExtra({ notes: e.target.value || null })
            }
            rows={3}
            className="mt-1 w-full rounded border border-[var(--oaec-border)] bg-[var(--oaec-surface)] px-2 py-1.5 text-sm"
          />
        </div>
      </Card>

      <Card title={t("projectSetup.sections.buildingType", "Gebouwtype")}>
        <div className="grid grid-cols-2 gap-4">
          <Select
            id="bt_kind"
            label={t("projectSetup.fields.building_kind", "Categorie")}
            value={buildingType.kind}
            options={[
              { value: "woning", label: t("projectSetup.buildingKind.woning", "Woning") },
              {
                value: "utiliteit",
                label: t("projectSetup.buildingKind.utiliteit", "Utiliteit"),
              },
            ]}
            onChange={(e) => onKindChange(e.target.value as "woning" | "utiliteit")}
          />
          <Select
            id="bt_subtype"
            label={t("projectSetup.fields.building_subtype", "Subtype")}
            value={buildingType.subtype}
            options={toOptions(subtypeLabels)}
            onChange={(e) => onSubtypeChange(e.target.value)}
          />
          <Input
            id="construction_year"
            label={t("projectSetup.fields.construction_year", "Bouwjaar")}
            type="number"
            value={sharedExtra.construction_year ?? ""}
            onChange={(e) =>
              updateSharedExtra({
                construction_year: e.target.value === "" ? null : numVal(e.target.value),
              })
            }
          />
          <Input
            id="total_floor_area_general"
            label={t("projectSetup.fields.gross_floor_area", "Bruto gebruiksoppervlak A_g")}
            type="number"
            unit="m²"
            value={building.total_floor_area}
            onChange={(e) =>
              updateProject({
                building: { ...building, total_floor_area: numVal(e.target.value) },
              })
            }
          />
          <Input
            id="num_storeys"
            label={t("projectSetup.fields.num_storeys", "Aantal bouwlagen")}
            type="number"
            value={building.num_floors ?? sharedExtra.num_storeys ?? 1}
            onChange={(e) => {
              const v = Math.max(1, numVal(e.target.value));
              updateProject({
                building: { ...building, num_floors: v },
              });
              updateSharedExtra({ num_storeys: v });
            }}
          />
        </div>
      </Card>

      <Card title={t("projectSetup.sections.coverImage", "Voorblad-afbeelding")}>
        <p className="text-[10px] text-on-surface-muted">
          {t(
            "projectSetup.fields.coverImageHint",
            "PNG of JPEG, max 2 MB. Komt op het voorblad van het rapport.",
          )}
        </p>
        {info.cover_image ? (
          <div className="mt-2 flex items-center gap-3">
            <img
              src={`data:${info.cover_image.media_type};base64,${info.cover_image.data}`}
              alt="Voorblad afbeelding"
              className="h-20 w-20 rounded border border-[var(--oaec-border)] object-cover"
            />
            <div className="flex-1 text-xs text-on-surface-muted">
              {info.cover_image.filename ?? "afbeelding"}
            </div>
            <button
              type="button"
              onClick={handleCoverImageClear}
              className="rounded border border-[var(--oaec-border)] px-2 py-1 text-xs text-on-surface-secondary hover:bg-[var(--oaec-hover)]"
            >
              {t("delete", "Verwijderen")}
            </button>
          </div>
        ) : (
          <label className="mt-2 inline-block cursor-pointer rounded border border-[var(--oaec-border)] px-3 py-1.5 text-xs text-on-surface-secondary hover:bg-[var(--oaec-hover)]">
            {t("projectSetup.fields.chooseImage", "Afbeelding kiezen…")}
            <input
              type="file"
              accept="image/png,image/jpeg"
              onChange={handleCoverImageChange}
              className="hidden"
            />
          </label>
        )}
      </Card>
    </div>
  );
}
