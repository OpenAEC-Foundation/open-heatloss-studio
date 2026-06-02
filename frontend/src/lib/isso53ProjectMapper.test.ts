import { describe, expect, it } from "vitest";
import { toIsso53LegacyProject } from "./isso53ProjectMapper";
import { buildV2PayloadIsso53 } from "./projectV2Migration";
import type { Project } from "../types";
import {
  DEFAULT_SHARED_EXTRA,
  type Isso53BuildingState,
  type Isso53RoomState,
} from "../types/projectV2";

/**
 * Verifieert dat {@link toIsso53LegacyProject} de V1 `Project` + ISSO 53-
 * sidecars transformeert naar de exacte camelCase shape die de Rust-
 * rekenkern verwacht. De verwachte veldnamen/enum-strings zijn gespiegeld
 * aan de bindende fixture `src-tauri/tests/calculate_v2_routing.rs` +
 * `crates/isso53-core/src/model/*.rs`.
 *
 * Aandachtspunten in deze suite:
 *   - enum value-remaps: `unheated_space`→`unheated`, `non_masonry`→`nonMasonry`
 *   - `groundParams.fg2` wordt gedropt; `u_equivalent`→`uEquivalent`
 *   - InfiltrationMethod serde-quirk: `{ known: { qv10_kar_class } }`
 *     (snake_case veld, PascalCase Qv10Class-string)
 *   - `bezetting.personenPerM2Default` (correcte naam, niet de fixture-typo)
 */

function makeProject(): Project {
  return {
    info: {
      name: "Test ISSO 53",
      project_number: "P-53",
      address: "Teststraat 1",
      client: "Klant BV",
      date: "2026-06-01",
      engineer: "OHS",
      notes: "test notes",
    },
    building: {
      building_type: "stacked",
      qv10: 0.4,
      total_floor_area: 50,
      security_class: "b",
      num_floors: 2,
      default_heating_system: "radiator_ht",
    },
    climate: {
      theta_e: -10,
    },
    ventilation: {
      system_type: "system_d",
      has_heat_recovery: true,
      heat_recovery_efficiency: 0.85,
      frost_protection: "central_preheating",
      supply_temperature: 18,
      has_preheating: false,
    },
    rooms: [
      {
        id: "K01",
        name: "Kantoor 1.01",
        function: "living_room",
        floor_area: 25,
        height: 2.8,
        custom_temperature: null,
        heating_system: "radiator_ht",
        constructions: [
          {
            id: "wall-n",
            description: "Buitenwand noord",
            area: 14,
            u_value: 0.21,
            boundary_type: "exterior",
            material_type: "masonry",
            vertical_position: "wall",
            use_forfaitaire_thermal_bridge: true,
          },
          {
            id: "window-n",
            description: "Raam noord (kozijn)",
            area: 6,
            u_value: 1.1,
            boundary_type: "exterior",
            material_type: "non_masonry",
            vertical_position: "wall",
            use_forfaitaire_thermal_bridge: true,
          },
          {
            id: "wall-corridor",
            description: "Wand naar gang (onverwarmd)",
            area: 10,
            u_value: 0.5,
            boundary_type: "unheated_space",
            material_type: "masonry",
            vertical_position: "wall",
          },
          {
            id: "wall-adj",
            description: "Wand naar buurvertrek",
            area: 8,
            u_value: 0.4,
            boundary_type: "adjacent_room",
            material_type: "masonry",
            vertical_position: "wall",
            adjacent_room_id: "K02",
          },
          {
            id: "floor",
            description: "Vloer op grond",
            area: 25,
            u_value: 0.22,
            boundary_type: "ground",
            material_type: "masonry",
            vertical_position: "floor",
            ground_params: {
              u_equivalent: 0.18,
              ground_water_factor: 1.0,
              fg2: 0.9,
            },
          },
        ],
      },
    ],
  };
}

const building53: Isso53BuildingState = {
  buildingShape: "meerlaags",
  buildingPosition: "meerlaagsTussen",
  windPressureType: "meerlaagsStandaard",
  thermalMass: "gemiddeld",
  ventilationSystem: "systemD",
  constructionYear: 2020,
  thetaMe: 9.0,
  qv10KarClass: "From040To060",
  heatingUp: {
    setbackActive: true,
    regimeType: "free",
    setbackHoursWeekday: 14,
    setbackHoursWeekend: 62,
    degreesWeekday: 3,
    degreesWeekend: 3,
    airChanges: "low",
    warmupHoursWeekday: 2,
    warmupHoursWeekend: 4,
    mechanicalSupplyOff: true,
    pWPerM2Override: null,
  },
};

const rooms53: Record<string, Isso53RoomState> = {
  K01: {
    gebruiksFunctie: "kantoor",
    ruimteType: "verblijfsruimte",
    personen: 3,
    infiltrationReductionZ: 1.0,
  },
};

describe("toIsso53LegacyProject", () => {
  const out = toIsso53LegacyProject(makeProject(), building53, rooms53);

  it("mapt info naar camelCase tekstvelden", () => {
    expect(out.info).toEqual({
      name: "Test ISSO 53",
      projectNumber: "P-53",
      address: "Teststraat 1",
      client: "Klant BV",
      date: "2026-06-01",
      engineer: "OHS",
      notes: "test notes",
    });
  });

  it("bouwt building met sidecar-velden + gemapt verwarmingssysteem", () => {
    const b = out.building as Record<string, unknown>;
    expect(b.buildingShape).toBe("meerlaags");
    expect(b.buildingPosition).toBe("meerlaagsTussen");
    expect(b.windPressureType).toBe("meerlaagsStandaard");
    expect(b.thermalMass).toBe("gemiddeld");
    expect(b.ventilationSystem).toBe("systemD");
    expect(b.constructionYear).toBe(2020);
    // radiator_ht (ISSO 51) → radiatorenConvHtEnLuchtverwarming (ISSO 53)
    expect(b.heatingSystem).toBe("radiatorenConvHtEnLuchtverwarming");
  });

  it("mapt climate (thetaE/thetaMe), laat thetaBAdjacentBuilding weg", () => {
    expect(out.climate).toEqual({ thetaE: -10, thetaMe: 9.0 });
  });

  it("zet ventilation.systemType uit de sidecar en frostProtection op null", () => {
    const v = out.ventilation as Record<string, unknown>;
    expect(v.systemType).toBe("systemD");
    expect(v.hasHeatRecovery).toBe(true);
    expect(v.heatRecoveryEfficiency).toBe(0.85);
    expect(v.frostProtection).toBeNull();
    expect(v.supplyTemperature).toBe(18);
    expect(v.hasPreheating).toBe(false);
  });

  it("mapt heatingUp naar de HeatingUpConfig-shape (tagged CoolingRegime, override null)", () => {
    expect(out.heatingUp).toEqual({
      setbackActive: true,
      pWPerM2Override: null,
      regime: {
        type: "free",
        setbackHoursWeekday: 14,
        setbackHoursWeekend: 62,
      },
      airChanges: "low",
      warmupHoursWeekday: 2,
      warmupHoursWeekend: 4,
      mechanicalSupplyOff: true,
    });
  });

  it("wikkelt infiltrationMethod in de Known-serde-shape (snake_case veld)", () => {
    expect(out.infiltrationMethod).toEqual({
      known: { qv10_kar_class: "From040To060" },
    });
  });

  it("mapt room-velden + bezetting + z-factor", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    expect(rooms).toHaveLength(1);
    const r = rooms[0];
    expect(r.id).toBe("K01");
    expect(r.name).toBe("Kantoor 1.01");
    expect(r.gebruiksFunctie).toBe("kantoor");
    expect(r.ruimteType).toBe("verblijfsruimte");
    expect(r.floorArea).toBe(25);
    expect(r.height).toBe(2.8);
    expect(r.customTemperature).toBeNull();
    expect(r.bezetting).toEqual({ personen: 3, personenPerM2Default: null });
    expect(r.infiltrationReductionZ).toBe(1.0);
    // has_mechanical_supply afwezig in de V1-room → null (Rust None, geen gate).
    expect(r.hasMechanicalSupply).toBeNull();
    // Geen ventilation_rate op de room → BBL-placeholder 0,9 × 25 m² = 22.5
    // dm³/s → 0.0225 m³/s (key matcht Rust `ventilation_q_v_established` via
    // rename_all=camelCase).
    expect(r.ventilationQvEstablished).toBeCloseTo(0.0225, 12);
  });

  it("leidt ventilationQvEstablished af uit room.ventilation_rate (dm³/s → m³/s)", () => {
    // Ingevuld q_v op de room → die waarde /1000, BBL-placeholder genegeerd.
    const base = makeProject();
    const projWithQv: Project = {
      ...base,
      rooms: [{ ...base.rooms[0], ventilation_rate: 50 }],
    };
    const outQv = toIsso53LegacyProject(projWithQv, building53, rooms53);
    expect(
      (outQv.rooms as Array<Record<string, unknown>>)[0].ventilationQvEstablished,
    ).toBeCloseTo(0.05, 12);

    // Leeg/undefined q_v → BBL-placeholder 0,9 × floor_area (25 m²) = 22.5
    // dm³/s → 0.0225 m³/s.
    const outEmpty = toIsso53LegacyProject(makeProject(), building53, rooms53);
    expect(
      (outEmpty.rooms as Array<Record<string, unknown>>)[0]
        .ventilationQvEstablished,
    ).toBeCloseTo(0.0225, 12);

    // ventilation_rate expliciet null → ook BBL-placeholder.
    const projNull: Project = {
      ...base,
      rooms: [{ ...base.rooms[0], ventilation_rate: null }],
    };
    const outNull = toIsso53LegacyProject(projNull, building53, rooms53);
    expect(
      (outNull.rooms as Array<Record<string, unknown>>)[0]
        .ventilationQvEstablished,
    ).toBeCloseTo(0.0225, 12);
  });

  it("propageert has_mechanical_supply (true/false/afwezig → true/false/null)", () => {
    const base = makeProject();

    const projTrue: Project = {
      ...base,
      rooms: [{ ...base.rooms[0], has_mechanical_supply: true }],
    };
    const outTrue = toIsso53LegacyProject(projTrue, building53, rooms53);
    expect((outTrue.rooms as Array<Record<string, unknown>>)[0].hasMechanicalSupply).toBe(
      true,
    );

    const projFalse: Project = {
      ...base,
      rooms: [{ ...base.rooms[0], has_mechanical_supply: false }],
    };
    const outFalse = toIsso53LegacyProject(projFalse, building53, rooms53);
    expect((outFalse.rooms as Array<Record<string, unknown>>)[0].hasMechanicalSupply).toBe(
      false,
    );

    // afwezig → null
    const outAbsent = toIsso53LegacyProject(base, building53, rooms53);
    expect(
      (outAbsent.rooms as Array<Record<string, unknown>>)[0].hasMechanicalSupply,
    ).toBeNull();
  });

  it("remapt boundaryType-waarden correct (incl. unheated_space→unheated)", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    const byId = Object.fromEntries(cons.map((c) => [c.id as string, c]));

    expect(byId["wall-n"].boundaryType).toBe("exterior");
    expect(byId["window-n"].boundaryType).toBe("exterior");
    expect(byId["wall-corridor"].boundaryType).toBe("unheated");
    expect(byId["wall-adj"].boundaryType).toBe("adjacentRoom");
    expect(byId["floor"].boundaryType).toBe("ground");
  });

  it("valt onverwarmd grensvlak zonder factor terug op temperatureFactor 0.5", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    const byId = Object.fromEntries(cons.map((c) => [c.id as string, c]));

    // unheated_space zonder expliciete temperature_factor → 0.5 (isso51-consistent)
    expect(byId["wall-corridor"].temperatureFactor).toBe(0.5);
    // andere grensvlaktypes zonder factor blijven null
    expect(byId["wall-n"].temperatureFactor).toBeNull();
    expect(byId["wall-adj"].temperatureFactor).toBeNull();
  });

  it("gebruikt de f_k uit de sidecar van de onverwarmde DOEL-ruimte", () => {
    // Onverwarmd grensvlak naar K02; K02-sidecar zet unheatedFactor = 0.17.
    const base = makeProject();
    const proj: Project = {
      ...base,
      rooms: [
        {
          ...base.rooms[0],
          constructions: [
            {
              id: "wall-to-meterkast",
              description: "Wand naar meterkast (onverwarmd)",
              area: 5,
              u_value: 0.5,
              boundary_type: "unheated_space",
              material_type: "masonry",
              vertical_position: "wall",
              adjacent_room_id: "K02",
            },
          ],
        },
      ],
    };
    const rooms53WithFactor: Record<string, Isso53RoomState> = {
      ...rooms53,
      K02: {
        gebruiksFunctie: "kantoor",
        ruimteType: "bergruimte",
        infiltrationReductionZ: 1.0,
        unheatedFactor: 0.17,
      },
    };
    const o = toIsso53LegacyProject(proj, building53, rooms53WithFactor);
    const cons = (o.rooms as Array<Record<string, unknown>>)[0]
      .constructions as Array<Record<string, unknown>>;
    expect(cons[0].temperatureFactor).toBe(0.17);
  });

  it("valt terug op 0.5 als de DOEL-ruimte-sidecar geen unheatedFactor heeft", () => {
    const base = makeProject();
    const proj: Project = {
      ...base,
      rooms: [
        {
          ...base.rooms[0],
          constructions: [
            {
              id: "wall-to-berging",
              description: "Wand naar berging",
              area: 5,
              u_value: 0.5,
              boundary_type: "unheated_space",
              material_type: "masonry",
              vertical_position: "wall",
              adjacent_room_id: "K02",
            },
          ],
        },
      ],
    };
    // K02-sidecar bestaat maar zonder unheatedFactor → norm-default 0.5.
    const rooms53NoFactor: Record<string, Isso53RoomState> = {
      ...rooms53,
      K02: {
        gebruiksFunctie: "kantoor",
        ruimteType: "bergruimte",
        infiltrationReductionZ: 1.0,
      },
    };
    const o = toIsso53LegacyProject(proj, building53, rooms53NoFactor);
    const cons = (o.rooms as Array<Record<string, unknown>>)[0]
      .constructions as Array<Record<string, unknown>>;
    expect(cons[0].temperatureFactor).toBe(0.5);
  });

  it("expliciete temperature_factor op de constructie wint van de sidecar-f_k", () => {
    const base = makeProject();
    const proj: Project = {
      ...base,
      rooms: [
        {
          ...base.rooms[0],
          constructions: [
            {
              id: "wall-explicit",
              description: "Wand naar onverwarmd (expliciete f_k)",
              area: 5,
              u_value: 0.5,
              boundary_type: "unheated_space",
              material_type: "masonry",
              vertical_position: "wall",
              adjacent_room_id: "K02",
              temperature_factor: 0.8,
            },
          ],
        },
      ],
    };
    const rooms53WithFactor: Record<string, Isso53RoomState> = {
      ...rooms53,
      K02: {
        gebruiksFunctie: "kantoor",
        ruimteType: "bergruimte",
        infiltrationReductionZ: 1.0,
        unheatedFactor: 0.17,
      },
    };
    const o = toIsso53LegacyProject(proj, building53, rooms53WithFactor);
    const cons = (o.rooms as Array<Record<string, unknown>>)[0]
      .constructions as Array<Record<string, unknown>>;
    expect(cons[0].temperatureFactor).toBe(0.8);
  });

  it("emit adjacent_room naar een isUnheated-room als boundary 'unheated' met f_k uit sidecar", () => {
    const base = makeProject();
    // Een verwarmd vertrek met een adjacent_room-wand naar TECH (techniek).
    const proj: Project = {
      ...base,
      rooms: [
        {
          ...base.rooms[0],
          constructions: [
            {
              id: "wall-to-tech",
              description: "Wand naar techniekruimte",
              area: 6,
              u_value: 0.5,
              boundary_type: "adjacent_room",
              material_type: "masonry",
              vertical_position: "wall",
              adjacent_room_id: "TECH",
            },
          ],
        },
      ],
    };
    // TECH is een aparte room die de gebruiker als onverwarmd markeert + f_k 0.03.
    const rooms53WithUnheated: Record<string, Isso53RoomState> = {
      ...rooms53,
      TECH: {
        gebruiksFunctie: "kantoor",
        ruimteType: "technischeRuimte",
        infiltrationReductionZ: 1.0,
        isUnheated: true,
        unheatedFactor: 0.03,
      },
    };
    const o = toIsso53LegacyProject(proj, building53, rooms53WithUnheated);
    const cons = (o.rooms as Array<Record<string, unknown>>)[0]
      .constructions as Array<Record<string, unknown>>;
    expect(cons[0].boundaryType).toBe("unheated");
    expect(cons[0].temperatureFactor).toBe(0.03);
    // adjacentRoomId blijft behouden, groundParams null (geen ground-element)
    expect(cons[0].adjacentRoomId).toBe("TECH");
    expect(cons[0].groundParams).toBeNull();
  });

  it("laat adjacent_room naar een NIET-onverwarmde room ongewijzigd (adjacentRoom)", () => {
    const base = makeProject();
    const proj: Project = {
      ...base,
      rooms: [
        {
          ...base.rooms[0],
          constructions: [
            {
              id: "wall-to-k02",
              description: "Wand naar buurvertrek",
              area: 8,
              u_value: 0.4,
              boundary_type: "adjacent_room",
              material_type: "masonry",
              vertical_position: "wall",
              adjacent_room_id: "K02",
            },
          ],
        },
      ],
    };
    // K02 niet gemarkeerd als onverwarmd.
    const o = toIsso53LegacyProject(proj, building53, {
      ...rooms53,
      K02: {
        gebruiksFunctie: "kantoor",
        ruimteType: "verblijfsruimte",
        infiltrationReductionZ: 1.0,
      },
    });
    const cons = (o.rooms as Array<Record<string, unknown>>)[0]
      .constructions as Array<Record<string, unknown>>;
    expect(cons[0].boundaryType).toBe("adjacentRoom");
    expect(cons[0].temperatureFactor).toBeNull();
  });

  it("remapt materialType (non_masonry→nonMasonry) en zet uValue", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    const window = cons.find((c) => c.id === "window-n")!;
    expect(window.materialType).toBe("nonMasonry");
    expect(window.uValue).toBe(1.1);
    expect(window.verticalPosition).toBe("wall");

    const wall = cons.find((c) => c.id === "wall-n")!;
    expect(wall.materialType).toBe("masonry");
  });

  it("propageert adjacentRoomId voor adjacent_room-elementen", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    const adj = cons.find((c) => c.id === "wall-adj")!;
    expect(adj.adjacentRoomId).toBe("K02");
  });

  it("mapt groundParams (uEquivalent/groundWaterFactor) en dropt fg2/fIg", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    const floor = cons.find((c) => c.id === "floor")!;
    expect(floor.groundParams).toEqual({
      uEquivalent: 0.18,
      groundWaterFactor: 1.0,
    });
    // fg2 mag NIET in de output zitten
    expect((floor.groundParams as Record<string, unknown>).fg2).toBeUndefined();
    // fIg / perimeter / depth weggelaten → auto-bepaling door de kern
    expect((floor.groundParams as Record<string, unknown>).fIg).toBeUndefined();
  });

  it("zet groundParams null voor niet-ground elementen", () => {
    const rooms = out.rooms as Array<Record<string, unknown>>;
    const cons = rooms[0].constructions as Array<Record<string, unknown>>;
    expect(cons.find((c) => c.id === "wall-n")!.groundParams).toBeNull();
  });

  it("valt terug op DEFAULT_ISSO53_ROOM als sidecar ontbreekt", () => {
    const out2 = toIsso53LegacyProject(makeProject(), building53, {});
    const r = (out2.rooms as Array<Record<string, unknown>>)[0];
    expect(r.gebruiksFunctie).toBe("kantoor");
    expect(r.ruimteType).toBe("verblijfsruimte");
    expect(r.bezetting).toEqual({ personen: null, personenPerM2Default: null });
    expect(r.infiltrationReductionZ).toBe(1.0);
    // Geen ventilation_rate → BBL-placeholder 0,9 × 25 m² = 0.0225 m³/s.
    expect(r.ventilationQvEstablished).toBeCloseTo(0.0225, 12);
  });
});

describe("buildV2PayloadIsso53", () => {
  const v2 = buildV2PayloadIsso53(
    makeProject(),
    { ...DEFAULT_SHARED_EXTRA },
    building53,
    rooms53,
  );

  it("activeert alleen de isso53-calc (isso51/tojuli null)", () => {
    expect(v2.calcs.isso51).toBeNull();
    expect(v2.calcs.tojuli).toBeNull();
    expect(v2.calcs.isso53).not.toBeNull();
  });

  it("plaatst de ISSO 53-projectvelden direct onder calcs.isso53", () => {
    const isso53 = v2.calcs.isso53 as Record<string, unknown>;
    expect(isso53.info).toBeDefined();
    expect(isso53.building).toBeDefined();
    expect(isso53.infiltrationMethod).toEqual({
      known: { qv10_kar_class: "From040To060" },
    });
  });

  it("behoudt shared + geometry uit buildV2Payload", () => {
    expect(v2.schema_version).toBe(2);
    expect(v2.shared.name).toBe("Test ISSO 53");
    expect(v2.geometry.spaces).toHaveLength(1);
  });
});
