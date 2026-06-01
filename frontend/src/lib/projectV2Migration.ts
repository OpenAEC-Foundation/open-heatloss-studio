/**
 * V1 ↔ V2 project schema migratie (frontend).
 *
 * Spiegelt `openaec-project-shared::migration::from_legacy_v1` zodat
 * frontend en backend dezelfde mapping hanteren. Backend serveert
 * momenteel nog V1 JSON; bij ontvangst detecteren we het schema en
 * splitsen we de data over V1 store + V2 sidecar (`SharedExtra`).
 *
 * **Strategie:**
 *  - V1 binnen → V1 store, lege sidecar. Roep `migrateV1ToV2` aan voor
 *    UI's die V2 nodig hebben (i.t.t. naar backend sturen).
 *  - V2 binnen → splits in V1-store-shape + sidecar. V2-only velden
 *    landen in `SharedExtra`.
 *  - Save naar backend: voorlopig V1 (`Project`). Wanneer backend
 *    `/api/v2/projects` ondersteunt: call `buildV2Payload`.
 */

import type {
  BoundaryType,
  BuildingType,
  ConstructionElement,
  Project,
  ProjectInfo,
  Room,
  VerticalPosition,
  VentilationConfig,
} from "../types";
import {
  DEFAULT_SHARED_EXTRA,
  type BoundaryKind,
  type BuildingTypeShared,
  type Construction,
  type ConstructionKind,
  type HeatRecovery,
  type Isso53BuildingState,
  type Isso53RoomState,
  type ProjectV2,
  type ResidentialType,
  type SharedExtra,
  type SharedProject,
  type Space,
  type VentilationSystemKind,
  SCHEMA_VERSION_V2,
} from "../types/projectV2";
import { toIsso53LegacyProject } from "./isso53ProjectMapper";

/** Detecteer schema-versie uit raw JSON of geparste data. */
export function detectSchemaVersion(data: unknown): 1 | 2 {
  if (typeof data !== "object" || data === null) return 1;
  const obj = data as Record<string, unknown>;
  const v = obj.schema_version;
  if (typeof v === "number" && v >= 2) return 2;
  return 1;
}

/** Map V1 `Building.building_type` → V2 `BuildingTypeShared`. */
export function buildingTypeV1ToV2(bt: BuildingType): BuildingTypeShared {
  return { kind: "woning", subtype: bt as ResidentialType };
}

/** Wrap V1 project in a synthetic ProjectV2 (transitional). */
export function migrateV1ToV2(v1: Project): ProjectV2 {
  const info: ProjectInfo = v1.info;
  const shared: SharedProject = {
    name: info.name ?? "",
    project_number: info.project_number ?? null,
    address: info.address ?? null,
    postcode: null,
    location: null,
    client: info.client ?? null,
    date: info.date ?? null,
    engineer: info.engineer ?? null,
    notes: null,
    building_type: buildingTypeV1ToV2(v1.building.building_type),
    construction_year: null,
    gross_floor_area_m2: v1.building.total_floor_area ?? null,
    num_storeys: v1.building.num_floors ?? null,
  };

  return {
    schema_version: SCHEMA_VERSION_V2,
    shared,
    geometry: { spaces: [] },
    calcs: {
      isso51: { legacy_v1: v1 },
      tojuli: null,
    },
  };
}

/**
 * Resultaat van V2-detectie+split. `project` is altijd in V1-shape voor
 * `projectStore` compat. `sharedExtra` bevat V2-only velden.
 */
export interface ProjectSplit {
  project: Project;
  sharedExtra: SharedExtra;
}

/**
 * Splits een raw payload (V1 of V2) naar `{ project, sharedExtra }`.
 *
 * V1 → project = data, sharedExtra = defaults.
 * V2 → project = calcs.isso51.legacy_v1 (of een minimaal gereconstrueerd V1
 *      shell als die ontbreekt). sharedExtra = V2-only velden uit
 *      `shared`.
 */
export function splitV2ForStore(raw: unknown): ProjectSplit {
  const version = detectSchemaVersion(raw);

  if (version === 1) {
    return {
      project: raw as Project,
      sharedExtra: { ...DEFAULT_SHARED_EXTRA },
    };
  }

  const v2 = raw as ProjectV2;
  const legacy = v2.calcs?.isso51?.legacy_v1;
  const project = (legacy && typeof legacy === "object"
    ? (legacy as Project)
    : reconstructV1FromShared(v2)) as Project;

  // Backfill V1 info-velden vanuit `shared` als legacy ontbreekt of leeg is.
  if (!project.info || !project.info.name) {
    project.info = {
      ...(project.info ?? { name: "" }),
      name: v2.shared.name ?? project.info?.name ?? "",
      project_number: v2.shared.project_number ?? project.info?.project_number ?? null,
      address: v2.shared.address ?? project.info?.address ?? null,
      client: v2.shared.client ?? project.info?.client ?? null,
      date: v2.shared.date ?? project.info?.date ?? null,
      engineer: v2.shared.engineer ?? project.info?.engineer ?? null,
    };
  }

  const sharedExtra: SharedExtra = {
    postcode: v2.shared.postcode ?? null,
    location: v2.shared.location ?? null,
    notes: v2.shared.notes ?? null,
    construction_year: v2.shared.construction_year ?? null,
    num_storeys: v2.shared.num_storeys ?? null,
    building_type: v2.shared.building_type ?? null,
  };

  return { project, sharedExtra };
}

/** Maak een minimaal V1 Project shell uit een V2 zonder legacy_v1 blob. */
function reconstructV1FromShared(v2: ProjectV2): Project {
  const subtype =
    v2.shared.building_type.kind === "woning"
      ? (v2.shared.building_type.subtype as BuildingType)
      : ("terraced" as BuildingType);
  return {
    info: {
      name: v2.shared.name ?? "",
      project_number: v2.shared.project_number ?? null,
      address: v2.shared.address ?? null,
      client: v2.shared.client ?? null,
      date: v2.shared.date ?? null,
      engineer: v2.shared.engineer ?? null,
    },
    building: {
      building_type: subtype,
      qv10: 100,
      total_floor_area: v2.shared.gross_floor_area_m2 ?? 80,
      security_class: "b",
      num_floors: v2.shared.num_storeys ?? 1,
      aggregation_method: "vabi_compat",
    },
    climate: {
      theta_e: -10,
      theta_b_residential: 17,
      theta_b_non_residential: 14,
      wind_factor: 1.0,
      theta_water: 5,
    },
    ventilation: {
      system_type: "system_c",
      has_heat_recovery: false,
    },
    rooms: [],
  };
}

/** Map V1 BoundaryType → V2 BoundaryKind. Verschil: V1 "water" → V2 "open_water". */
function mapBoundary(v1: BoundaryType): BoundaryKind {
  if (v1 === "water") return "open_water";
  return v1; // overige waarden zijn 1:1 identiek
}

/** Map V1 VerticalPosition → V2 ConstructionKind. Default "wall". */
function mapConstructionKind(vp: VerticalPosition | undefined): ConstructionKind {
  if (vp === "floor") return "floor";
  if (vp === "ceiling") return "ceiling";
  return "wall";
}

/** Map één V1 ConstructionElement → V2 Construction. Layers + openings blijven leeg
 *  — V1-layers hebben andere shape (materialId i.p.v. material/thickness_mm); voor de
 *  TO-juli H_T berekening volstaan area_m2 + u_value + boundary. */
function mapV1ConstructionToV2(c: ConstructionElement): Construction {
  return {
    id: c.id,
    description: c.description,
    kind: mapConstructionKind(c.vertical_position),
    boundary: mapBoundary(c.boundary_type),
    area_m2: c.area,
    u_value: c.u_value,
    adjacent_space_id: c.adjacent_room_id ?? null,
  };
}

/** Map één V1 Room → V2 Space. Height fallback 2.7 m (NTA 8800 standaard). */
function mapV1RoomToSpace(room: Room): Space {
  return {
    id: room.id,
    name: room.name,
    function: room.function,
    floor_area_m2: room.floor_area,
    height_m: room.height ?? 2.7,
    theta_i_winter_c: room.custom_temperature ?? null,
    constructions: room.constructions.map(mapV1ConstructionToV2),
  };
}

/**
 * Map V1 `VentilationSystemType` → V2 `VentilationSystemKind`.
 *
 * V1 ISSO 51-systemen:
 *   system_a = natuurlijke toe- én afvoer          → "natural"
 *   system_b = mechanische toevoer, nat. afvoer    → "mech_supply"
 *   system_c = nat. toevoer, mechanische afvoer    → "mech_exhaust"
 *   system_d = mechanische toe- én afvoer          → "mech_balanced"
 *   system_e = decentraal (gebalanceerd per unit)  → "mech_balanced" (best fit)
 */
function mapV1SystemTypeToV2(systemType: string): VentilationSystemKind {
  switch (systemType) {
    case "system_a":
      return "natural";
    case "system_b":
      return "mech_supply";
    case "system_c":
      return "mech_exhaust";
    case "system_d":
      return "mech_balanced";
    case "system_e":
      return "mech_balanced"; // decentraal gebalanceerd → closest V2 match
    default:
      return "natural"; // defensieve fallback
  }
}

/**
 * Map V1 ventilation config → V2 SharedProject ventilation velden.
 *
 * - `ventilation_system` altijd gezet als v1Vent aanwezig is
 * - `heat_recovery` alleen als `has_heat_recovery === true` (efficiency verplicht)
 * - `infiltration_m3_per_h` niet beschikbaar in V1 VentilationConfig → altijd undefined
 * - `mechanical_supply_m3_per_h` / `mechanical_exhaust_m3_per_h` niet beschikbaar in
 *    V1 VentilationConfig → altijd undefined. Backend valt terug op `default_ach`
 *    in tojuli.rs als deze velden missen (NTA 8800 tab 11.23 default lookup).
 */
export function mapV1VentilationToV2(v1Vent: VentilationConfig | undefined): {
  ventilation_system?: VentilationSystemKind;
  heat_recovery?: HeatRecovery;
  infiltration_m3_per_h?: number;
  mechanical_supply_m3_per_h?: number;
  mechanical_exhaust_m3_per_h?: number;
} {
  if (!v1Vent) return {};

  const ventilation_system = mapV1SystemTypeToV2(v1Vent.system_type);

  let heat_recovery: HeatRecovery | undefined;
  if (v1Vent.has_heat_recovery === true && v1Vent.heat_recovery_efficiency != null) {
    heat_recovery = {
      efficiency: v1Vent.heat_recovery_efficiency,
      // V1 frost_protection is een enum-string (FrostProtectionType); aanwezigheid
      // (niet "unknown") betekent dat er vorstbeveiliging is.
      frost_protection:
        v1Vent.frost_protection != null && v1Vent.frost_protection !== "unknown",
      ...(v1Vent.supply_temperature != null
        ? { supply_temperature: v1Vent.supply_temperature }
        : {}),
    };
  }

  return {
    ventilation_system,
    ...(heat_recovery !== undefined ? { heat_recovery } : {}),
    // infiltratie is niet aanwezig in V1 VentilationConfig — veld weglaten
    // mechanical_supply_m3_per_h / mechanical_exhaust_m3_per_h ontbreken eveneens
    // in V1 — backend gebruikt default_ach fallback (NTA 8800 tab 11.23)
  };
}

/**
 * Bouw een V2-payload voor save (toekomstige V2 backend). Combineert
 * V1-store + sidecar tot één `ProjectV2`.
 */
export function buildV2Payload(
  project: Project,
  sharedExtra: SharedExtra,
): ProjectV2 {
  const buildingType =
    sharedExtra.building_type ?? buildingTypeV1ToV2(project.building.building_type);
  return {
    schema_version: SCHEMA_VERSION_V2,
    shared: {
      name: project.info.name ?? "",
      project_number: project.info.project_number ?? null,
      address: project.info.address ?? null,
      postcode: sharedExtra.postcode ?? null,
      location: sharedExtra.location ?? null,
      client: project.info.client ?? null,
      date: project.info.date ?? null,
      engineer: project.info.engineer ?? null,
      notes: sharedExtra.notes ?? null,
      building_type: buildingType,
      construction_year: sharedExtra.construction_year ?? null,
      gross_floor_area_m2: project.building.total_floor_area ?? null,
      num_storeys: sharedExtra.num_storeys ?? project.building.num_floors ?? null,
      ...mapV1VentilationToV2(project.ventilation),
      // V2-only ventilatieveld overlay vanuit sidecar — overschrijft mapV1*
      // velden wanneer expliciet gezet (anders blijft backend-default actief).
      ...(sharedExtra.infiltration_m3_per_h != null
        ? { infiltration_m3_per_h: sharedExtra.infiltration_m3_per_h }
        : {}),
      ...(sharedExtra.mechanical_supply_m3_per_h != null
        ? { mechanical_supply_m3_per_h: sharedExtra.mechanical_supply_m3_per_h }
        : {}),
      ...(sharedExtra.mechanical_exhaust_m3_per_h != null
        ? { mechanical_exhaust_m3_per_h: sharedExtra.mechanical_exhaust_m3_per_h }
        : {}),
    },
    geometry: { spaces: project.rooms.map(mapV1RoomToSpace) },
    calcs: {
      isso51: { legacy_v1: project },
      tojuli: null,
    },
  };
}

/**
 * Bouw een V2-payload voor een ISSO 53-project. Identiek aan
 * {@link buildV2Payload} qua `shared`/`geometry`, maar de `calcs`-sectie
 * activeert ISSO 53: `isso51` en `tojuli` zijn `null`, en `isso53` bevat de
 * door {@link toIsso53LegacyProject} getransformeerde legacy-blob.
 *
 * Omdat alleen `isso53` gevuld is, geeft de Rust `Calcs::active_norm()`
 * `ActiveNorm::Isso53` terug (zie
 * `crates/openaec-project-shared/src/calcs.rs`).
 *
 * De `legacy`-wrapper-key spiegelt de TS `Iso53Inputs`-shape
 * (`{ legacy: Record<string, unknown> }`). Aan de Rust-kant is
 * `Iso53Inputs.legacy` `#[serde(flatten)]`, dus de ISSO 53-velden
 * verschijnen op de wire inline onder `calcs.isso53` — de wrapper-key is
 * puur een TS-/serde-detail en verdwijnt bij (de)serialisatie.
 */
export function buildV2PayloadIsso53(
  project: Project,
  sharedExtra: SharedExtra,
  isso53Building: Isso53BuildingState,
  isso53Rooms: Record<string, Isso53RoomState>,
): ProjectV2 {
  const base = buildV2Payload(project, sharedExtra);
  return {
    ...base,
    calcs: {
      isso51: null,
      isso53: { legacy: toIsso53LegacyProject(project, isso53Building, isso53Rooms) },
      tojuli: null,
    },
  };
}
