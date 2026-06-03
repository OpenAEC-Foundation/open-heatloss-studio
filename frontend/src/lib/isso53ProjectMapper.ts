/**
 * Pure transformatie V1 `Project` + ISSO 53-sidecars → ISSO 53 legacy
 * project-JSON (camelCase) zoals de Rust-rekenkern `isso53_core` verwacht.
 *
 * Fase 2 van de ISSO 53-calc-aansluiting. Géén dispatch / store-coupling /
 * calc-aanroep — uitsluitend de shape-transformatie. De resulterende blob
 * gaat (in een latere fase) inline onder `ProjectV2.calcs.isso53` en wordt
 * door `to_isso53_project()` direct gedeserialiseerd naar
 * `isso53_core::model::Project`.
 *
 * **Bindende ground-truth** voor veldnamen / enum-serde-reps:
 *   - `src-tauri/tests/calculate_v2_routing.rs` (ISSO 53 fixture)
 *   - `examples/projectV2-isso53-minimal.json`
 *   - `crates/isso53-core/src/model/*.rs` (alle `#[serde(rename_all="camelCase")]`)
 *   - `crates/isso53-core/src/calc/infiltration.rs` → `InfiltrationMethod`
 *
 * **Serde-quirks die hier 1:1 worden gerespecteerd:**
 *   - `InfiltrationMethod` heeft `rename_all="camelCase"` op variant-NIVEAU
 *     (de varianten heten `known`/`unknown`/`unknownVabiCompat`), maar de
 *     VELDEN binnen een variant worden daardoor NIET hernoemd. De `Known`-
 *     variant serialiseert dus als `{ "known": { "qv10_kar_class": ... } }`
 *     met snake_case veldnaam. Idem `Qv10Class` kent geen `rename_all`,
 *     dus PascalCase variant-strings (`"From040To060"`).
 *   - `Bezetting` is een struct mét `rename_all="camelCase"` → veld heet
 *     `personenPerM2Default` (de typo `persorenPerM2Default` in de fixture
 *     is een serde-onbekend veld dat genegeerd wordt; wij schrijven de
 *     correcte naam).
 *   - `Iso53Inputs.legacy` is `#[serde(flatten)]` in Rust; daardoor moeten de
 *     projectvelden INLINE onder `calcs.isso53` staan. `buildV2PayloadIsso53`
 *     plaatst de output van deze functie dus direct onder `isso53` (GEEN
 *     `legacy`-wrapper — die zou serde-flatten als de project-blob verzamelen
 *     en `to_isso53_project` op `missing field info` laten falen).
 */

import type {
  ConstructionElement,
  Project,
  ProjectInfo,
  Room,
} from "../types";
import {
  DEFAULT_ISSO53_ROOM,
  type Isso53BuildingState,
  type Isso53RoomState,
} from "../types/projectV2";
import {
  DEFAULT_UNHEATED_FACTOR,
  resolveUnheatedRoomIds,
} from "./isso53Unheated";
import { mapHeatingSystem } from "./normSwitch";

// ---------------------------------------------------------------------------
// Enum value-remaps (V1 snake_case → ISSO 53 camelCase serde-strings)
// ---------------------------------------------------------------------------

/**
 * V1 `BoundaryType` → ISSO 53 `BoundaryType` (camelCase). Let op:
 * `unheated_space` → `unheated` (de Rust-variant heet `Unheated`).
 */
const BOUNDARY_TYPE_MAP: Record<string, string> = {
  exterior: "exterior",
  adjacent_room: "adjacentRoom",
  adjacent_building: "adjacentBuilding",
  ground: "ground",
  unheated_space: "unheated",
  water: "water",
};

/** V1 `MaterialType` → ISSO 53 `MaterialType` (camelCase). */
const MATERIAL_TYPE_MAP: Record<string, string> = {
  masonry: "masonry",
  non_masonry: "nonMasonry",
};

/**
 * V1 `VerticalPosition` → ISSO 53 `VerticalPosition`. Identieke waarden
 * (`wall`/`floor`/`ceiling`), maar expliciet om robuust te zijn tegen
 * ontbrekend veld (V1 default = `wall`).
 */
function mapVerticalPosition(vp: string | undefined): string {
  if (vp === "floor") return "floor";
  if (vp === "ceiling") return "ceiling";
  return "wall";
}

// ---------------------------------------------------------------------------
// Sub-mappers
// ---------------------------------------------------------------------------

/** `ProjectInfo` (snake_case) → ISSO 53 `info` (camelCase, alleen tekst). */
function mapInfo(info: ProjectInfo): Record<string, unknown> {
  return {
    name: info.name ?? "",
    projectNumber: info.project_number ?? null,
    address: info.address ?? null,
    client: info.client ?? null,
    date: info.date ?? null,
    engineer: info.engineer ?? null,
    notes: info.notes ?? null,
  };
}

/**
 * Eén V1 `ConstructionElement` → ISSO 53 construction-element.
 *
 * `groundParams`: alleen voor `boundary_type === "ground"`. Mapt
 * `u_equivalent`→`uEquivalent`, `ground_water_factor`→`groundWaterFactor`.
 * Heeft het grondvlak-element geen (geldige) `ground_params.u_equivalent`,
 * dan valt `uEquivalent` terug op de construction-`u_value`. Zo levert een
 * grondvlak altijd een positieve equivalente U mee en heeft de ISSO 53-kern
 * geen `perimeter`/`depth` meer nodig (vermijdt "Ground element requires
 * perimeter and depth for U_equiv calculation").
 * `fg2` wordt GEDROPT (geen ISSO 53-veld). `fIg`/`perimeter`/`depth` worden
 * weggelaten zodat de kern ze auto-bepaalt zodra `uEquivalent > 0`.
 *
 * `temperatureFactor`: onverwarmd (`unheated_space`) → een expliciete
 * `c.temperature_factor` op de constructie wint altijd; anders de per-DOEL-
 * ruimte ingestelde f_k uit de sidecar van de aangrenzende onverwarmde ruimte
 * (`isso53Rooms[c.adjacent_room_id].unheatedFactor`); anders de norm-default
 * 0.5 (isso51-consistent, `h_t_unheated_element` unwrap_or(0.5)). Zo telt één
 * f_k per onverwarmde ruimte door op álle wanden die eraan grenzen. Andere
 * grensvlaktypes houden `null` — die vereisen geen f_k.
 *
 * **Onverwarmd-conversie:** een `adjacent_room`-constructie waarvan de
 * doel-ruimte (`adjacent_room_id`) in `unheatedRoomIds` zit (gemarkeerd via
 * `Isso53RoomState.isUnheated` of impliciet doel van een `unheated_space`),
 * wordt geëmit als `boundaryType: "unheated"` met dezelfde f_k-resolutie. Zo
 * lopen 10 °C-service-ruimtes die als `adjacent_room` zijn gemodelleerd alsnog
 * via de f_k-reductie i.p.v. de volle ΔT naar 10 °C.
 */
function mapConstruction(
  c: ConstructionElement,
  isso53Rooms: Record<string, Isso53RoomState>,
  unheatedRoomIds: Set<string>,
): Record<string, unknown> {
  // Behandel een adjacent_room naar een (expliciet/impliciet) onverwarmde
  // ruimte als unheated grensvlak. unheated_space blijft uiteraard unheated.
  const targetsUnheatedRoom =
    c.boundary_type === "adjacent_room" &&
    c.adjacent_room_id != null &&
    unheatedRoomIds.has(c.adjacent_room_id);
  const isUnheatedBoundary =
    c.boundary_type === "unheated_space" || targetsUnheatedRoom;

  const out: Record<string, unknown> = {
    id: c.id,
    description: c.description,
    area: c.area,
    uValue: c.u_value,
    boundaryType: isUnheatedBoundary
      ? "unheated"
      : BOUNDARY_TYPE_MAP[c.boundary_type] ?? "exterior",
    materialType: MATERIAL_TYPE_MAP[c.material_type] ?? "masonry",
    temperatureFactor:
      c.temperature_factor ??
      (isUnheatedBoundary
        ? (c.adjacent_room_id != null
            ? isso53Rooms[c.adjacent_room_id]?.unheatedFactor
            : undefined) ?? DEFAULT_UNHEATED_FACTOR
        : null),
    adjacentRoomId: c.adjacent_room_id ?? null,
    adjacentTemperature: c.adjacent_temperature ?? null,
    verticalPosition: mapVerticalPosition(c.vertical_position),
    useForfaitaireThermalBridge: c.use_forfaitaire_thermal_bridge ?? true,
    customDeltaUTb: c.custom_delta_u_tb ?? null,
    hasEmbeddedHeating: c.has_embedded_heating ?? false,
  };

  if (c.boundary_type === "ground") {
    const groundU =
      c.ground_params?.u_equivalent != null && c.ground_params.u_equivalent > 0
        ? c.ground_params.u_equivalent
        : c.u_value; // fallback: gebruik de construction-U als equivalente grond-U
    out.groundParams = {
      uEquivalent: groundU,
      groundWaterFactor: c.ground_params?.ground_water_factor ?? 1.0,
      // fg2 wordt bewust gedropt (geen ISSO 53-veld). fIg/perimeter/depth
      // weggelaten → kern auto-berekent f_ig zodra uEquivalent > 0.
    };
  } else {
    out.groundParams = null;
  }

  return out;
}

/**
 * Eén V1 `Room` + sidecar → ISSO 53 room.
 *
 * `gebruiksFunctie`/`ruimteType` komen uit de sidecar (fallback
 * `DEFAULT_ISSO53_ROOM`). `bezetting.personen` uit `sidecar.personen`
 * (null = auto via tabel 4.11). `infiltrationReductionZ` uit de sidecar
 * (default 1.0).
 */
function mapRoom(
  room: Room,
  sidecar: Isso53RoomState | undefined,
  isso53Rooms: Record<string, Isso53RoomState>,
  unheatedRoomIds: Set<string>,
): Record<string, unknown> {
  const s = sidecar ?? DEFAULT_ISSO53_ROOM;
  return {
    id: room.id,
    name: room.name,
    gebruiksFunctie: s.gebruiksFunctie,
    ruimteType: s.ruimteType,
    floorArea: room.floor_area,
    height: room.height ?? 2.7,
    customTemperature: room.custom_temperature ?? null,
    constructions: room.constructions.map((c) =>
      mapConstruction(c, isso53Rooms, unheatedRoomIds),
    ),
    bezetting: {
      personen: s.personen ?? null,
      personenPerM2Default: null,
    },
    infiltrationReductionZ: s.infiltrationReductionZ ?? 1.0,
    // ISSO 53: alleen mechanische toevoer telt mee voor het
    // ventilatiewarmteverlies. `false` → kern gate't q_v op 0; `undefined`
    // (veld afwezig) → `null` → Rust `None` → geen gate, ongewijzigde berekening.
    hasMechanicalSupply: room.has_mechanical_supply ?? null,
    // Vastgestelde toevoer q_v uit het bestaande VentilationRow-veld
    // (`room.ventilation_rate`, in dm³/s) → kern verwacht m³/s
    // (Rust `ventilation_q_v_established: Option<f64>`). Leeg q_v → BBL-
    // placeholder (ISSO 53 verblijfsgebied: 0,9 dm³/s·m²). De kern gebruikt
    // altijd deze waarde.
    ventilationQVEstablished:
      (room.ventilation_rate ?? 0.9 * room.floor_area) / 1000,
  };
}

/**
 * Bepaal de bron-`HeatingSystem`-waarde voor de norm-mapping: project-brede
 * default, anders de eerste room, anders undefined. `mapHeatingSystem`
 * detecteert zelf of de bronwaarde al een ISSO 53-key is en mapt anders
 * vanuit ISSO 51.
 */
function resolveHeatingSystem(project: Project): string {
  const source =
    project.building.default_heating_system ??
    project.rooms[0]?.heating_system ??
    undefined;
  // fromNorm/toNorm: mapHeatingSystem inspecteert de waarde zelf; we geven
  // "isso51" als bron-hint zodat ISSO 51-keys correct worden geconverteerd
  // en reeds-ISSO-53-keys ongewijzigd blijven.
  return mapHeatingSystem(source, "isso51", "isso53");
}

// ---------------------------------------------------------------------------
// Hoofd-export
// ---------------------------------------------------------------------------

/**
 * Transformeer een V1 `Project` + ISSO 53-sidecars naar de ISSO 53 legacy
 * project-JSON (camelCase) die `isso53_core::model::Project` deserialiseert.
 *
 * Pure functie — geen store-import, geen side effects.
 */
export function toIsso53LegacyProject(
  project: Project,
  isso53Building: Isso53BuildingState,
  isso53Rooms: Record<string, Isso53RoomState>,
): Record<string, unknown> {
  const building: Record<string, unknown> = {
    buildingShape: isso53Building.buildingShape,
    buildingPosition: isso53Building.buildingPosition,
    windPressureType: isso53Building.windPressureType,
    thermalMass: isso53Building.thermalMass,
    ventilationSystem: isso53Building.ventilationSystem,
    // constructionYear is non-optional in de Rust `Building` (u32). Sidecar
    // mag null zijn → val terug op project-bouwjaar is niet beschikbaar in
    // V1, dus 0 als laatste redmiddel (kern gebruikt dit alleen op het
    // Unknown-infiltratiepad; wij draaien Known, dus irrelevant voor de calc).
    constructionYear: isso53Building.constructionYear ?? 0,
    heatingSystem: resolveHeatingSystem(project),
  };

  const climate: Record<string, unknown> = {
    thetaE: project.climate.theta_e ?? -10.0,
    thetaMe: isso53Building.thetaMe,
    // thetaBAdjacentBuilding: geen V1-equivalent → weglaten, serde default 15.
  };

  // ventilation: systemType uit de sidecar (al camelCase ISSO 53-vorm).
  // Overige velden uit V1 VentilationConfig (snake→camel). frost_protection
  // is in V1 een enum-string (FrostProtectionType); de ISSO 53-kern verwacht
  // `Option<f64>` → niet 1:1 mapbaar, dus weglaten (serde skip = None).
  const v = project.ventilation;
  const ventilation: Record<string, unknown> = {
    systemType: isso53Building.ventilationSystem,
    bouwfase: isso53Building.bouwfase,
    hasHeatRecovery: v?.has_heat_recovery ?? false,
    heatRecoveryEfficiency: v?.heat_recovery_efficiency ?? null,
    frostProtection: null,
    supplyTemperature: v?.supply_temperature ?? null,
    hasPreheating: v?.has_preheating ?? false,
    preheatingTemperature: v?.preheating_temperature ?? null,
  };

  // heatingUp → Rust `HeatingUpConfig` (serde camelCase). `regime` is een
  // intern-tagged enum (`#[serde(tag="type")]`): de variant-tag staat in het
  // veld `type` ("free"/"limited"), de regime-specifieke velden (uren bij
  // vrije, graden bij beperkte afkoeling) staan op hetzelfde niveau. De
  // tegenovergestelde set velden wordt weggelaten zodat serde de juiste
  // variant matcht. `pWPerM2Override` is `Option<f64>` → null laat de kern de
  // §4.8-tabel-lookup doen, een getal overschrijft.
  const hu = isso53Building.heatingUp;
  const regime: Record<string, unknown> =
    hu.regimeType === "limited"
      ? {
          type: "limited",
          degreesWeekday: hu.degreesWeekday,
          degreesWeekend: hu.degreesWeekend,
        }
      : {
          type: "free",
          setbackHoursWeekday: hu.setbackHoursWeekday,
          setbackHoursWeekend: hu.setbackHoursWeekend,
        };
  const heatingUp: Record<string, unknown> = {
    setbackActive: hu.setbackActive,
    pWPerM2Override: hu.pWPerM2Override,
    regime,
    airChanges: hu.airChanges,
    warmupHoursWeekday: hu.warmupHoursWeekday,
    warmupHoursWeekend: hu.warmupHoursWeekend,
    mechanicalSupplyOff: hu.mechanicalSupplyOff,
    simultaneityFactor: hu.simultaneityFactor,
  };

  // InfiltrationMethod::Known — variant-key "known", VELD snake_case
  // (qv10_kar_class) want rename_all werkt op variant-niveau, niet op de
  // inner struct-velden. Qv10Class kent geen rename → PascalCase string.
  const infiltrationMethod: Record<string, unknown> = {
    known: {
      qv10_kar_class: isso53Building.qv10KarClass,
    },
  };

  // Gecombineerde set onverwarmde room-ids: impliciete unheated_space-doelen
  // ∪ expliciet via sidecar `isUnheated` gemarkeerde vertrekken. Bepaalt welke
  // adjacent_room-wanden als unheated-grensvlak worden geëmit.
  const unheatedRoomIds = resolveUnheatedRoomIds(project.rooms, isso53Rooms);

  const rooms = project.rooms.map((room) =>
    mapRoom(room, isso53Rooms[room.id], isso53Rooms, unheatedRoomIds),
  );

  return {
    info: mapInfo(project.info),
    building,
    climate,
    ventilation,
    heatingUp,
    infiltrationMethod,
    rooms,
  };
}
