import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  exportProject,
  importProject,
  openProjectFile,
  type ImportResult,
} from "./importExport";
import {
  buildIfcEnergyDocument,
  emptyModellerSnapshot,
  serializeIfcEnergy,
} from "./ifcenergy";
import { useProjectStore } from "../store/projectStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import type { Project, ProjectResult } from "../types";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
  DEFAULT_SHARED_EXTRA,
  type Isso53BuildingState,
  type Isso53RoomState,
  type SharedExtra,
} from "../types/projectV2";
import type { VentilationState } from "../types/ventilation";

/**
 * Tests voor de norm-bewuste opslag/laad-cyclus in {@link exportProject} /
 * {@link importProject}.
 *
 * Geborgd gedrag:
 *   (a) ISSO 51-export bevat GEEN `norm`/`isso53`-velden (byte-compat) en
 *       round-trip levert identiek project/result.
 *   (b) ISSO 53-export schrijft norm + sidecars; import herstelt ze exact.
 *   (c) Een oud `.isso51.json`-envelope ZONDER norm/isso53 laadt nog steeds
 *       als ISSO 51 (geen crash, sidecars op default).
 */

// ---------------------------------------------------------------------------
// DOM-stubs — vitest draait in environment "node", dus Blob/URL/document
// ontbreken. We vangen de geserialiseerde JSON op via een Blob-stub.
// ---------------------------------------------------------------------------

let lastBlobContent = "";

class FakeBlob {
  constructor(parts: BlobPart[]) {
    lastBlobContent = parts.map((p) => String(p)).join("");
  }
}

beforeEach(() => {
  lastBlobContent = "";
  vi.stubGlobal("Blob", FakeBlob as unknown as typeof Blob);
  vi.stubGlobal("URL", {
    createObjectURL: () => "blob:fake",
    revokeObjectURL: () => undefined,
  });
  vi.stubGlobal("document", {
    createElement: () => ({
      href: "",
      download: "",
      click: () => undefined,
    }),
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  useProjectStore.getState().reset();
  // Modeller store leegmaken zodat geen geometrie tussen tests lekt.
  useModellerStore.getState().importModel([], [], []);
  useModellerStore.getState().importProjectConstructions([]);
});

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeIsso51Project(): Project {
  return {
    info: { name: "Test ISSO 51" },
    building: {
      building_type: "terraced",
      qv10: 100,
      total_floor_area: 80,
      security_class: "b",
      has_night_setback: true,
      warmup_time: 2,
      num_floors: 1,
      default_heating_system: "radiator_ht",
      aggregation_method: "vabi_compat",
      // validateProject() backfilt dit veld op import; expliciet in de
      // fixture zodat de round-trip deep-equal sluit (anders verschijnt het
      // alleen in `imported.project`).
      infiltration_method: "per_exterior_area",
    },
    climate: {
      theta_e: -10,
      theta_b_residential: 17,
      theta_b_non_residential: 14,
      wind_factor: 1.0,
      theta_water: 5,
    },
    ventilation: { system_type: "system_c", has_heat_recovery: false },
    rooms: [
      {
        id: "r1",
        name: "Woonkamer",
        function: "living_room",
        floor_area: 20,
        constructions: [],
        heating_system: "radiator_ht",
      },
    ],
  };
}

function makeIsso53Project(): Project {
  const p = makeIsso51Project();
  p.info.name = "Test ISSO 53";
  p.building.default_heating_system = "radiatorenConvHtEnLuchtverwarming";
  p.rooms = p.rooms.map((r) => ({
    ...r,
    heating_system: "radiatorenConvHtEnLuchtverwarming",
  }));
  return p;
}

function makeResult(): ProjectResult {
  return {
    rooms: [],
    summary: { total_heat_loss: 1234 },
  } as unknown as ProjectResult;
}

function parseExported(): Record<string, unknown> {
  return JSON.parse(lastBlobContent) as Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// (a) ISSO 51 — byte-compat + round-trip
// ---------------------------------------------------------------------------

describe("exportProject — ISSO 51 byte-compat", () => {
  it("schrijft GEEN norm/isso53-velden bij norm === isso51", () => {
    const project = makeIsso51Project();
    const result = makeResult();
    useProjectStore.setState({ norm: "isso51" });

    exportProject(project, result);
    const env = parseExported();

    expect(env.schema).toBe("isso51-project-v1");
    expect("norm" in env).toBe(false);
    expect("isso53" in env).toBe(false);
    // Geen sharedExtra-veld bij volledig-default sidecar → byte-compat.
    expect("sharedExtra" in env).toBe(false);
  });

  it("round-trip levert identiek project + result", () => {
    const project = makeIsso51Project();
    const result = makeResult();
    useProjectStore.setState({ norm: "isso51" });

    exportProject(project, result);
    const imported = importProject(lastBlobContent) as ImportResult;

    expect(imported.type).toBe("project");
    expect(imported.project).toEqual(project);
    expect(imported.result).toEqual(result);
    // Geen sidecars in een ISSO 51-bestand.
    expect(imported.norm).toBeUndefined();
    expect(imported.isso53).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// (a2) Zones — optionele datalaag: legacy laadt schoon, round-trip behoudt
// ---------------------------------------------------------------------------

describe("importProject — zones (optionele datalaag)", () => {
  it("legacy JSON zonder zones/zoneId laadt schoon (geen backfill)", () => {
    const project = makeIsso51Project();
    useProjectStore.setState({ norm: "isso51" });

    exportProject(project, makeResult());
    const imported = importProject(lastBlobContent) as ImportResult;

    expect(imported.project.building.zones).toBeUndefined();
    expect(imported.project.rooms[0]!.zoneId).toBeUndefined();
  });

  it("round-trip behoudt building.zones + room.zoneId exact", () => {
    const project = makeIsso51Project();
    project.building.zones = [
      { id: "zone-a", name: "Zone 1" },
      { id: "zone-b", name: "Zone 2" },
    ];
    project.rooms[0]!.zoneId = "zone-a";
    useProjectStore.setState({ norm: "isso51" });

    exportProject(project, makeResult());
    const imported = importProject(lastBlobContent) as ImportResult;

    expect(imported.project.building.zones).toEqual([
      { id: "zone-a", name: "Zone 1" },
      { id: "zone-b", name: "Zone 2" },
    ]);
    expect(imported.project.rooms[0]!.zoneId).toBe("zone-a");
  });
});

// ---------------------------------------------------------------------------
// (b) ISSO 53 — sidecars round-trip
// ---------------------------------------------------------------------------

describe("exportProject — ISSO 53 sidecars", () => {
  it("export → import herstelt norm + building + rooms exact", () => {
    const project = makeIsso53Project();
    const result = makeResult();

    const building: Isso53BuildingState = {
      ...DEFAULT_ISSO53_BUILDING,
      thermalMass: "zwaar",
      heatingUp: {
        ...DEFAULT_ISSO53_BUILDING.heatingUp,
        setbackActive: true,
        regimeType: "limited",
        degreesWeekday: 4,
      },
    };
    const rooms: Record<string, Isso53RoomState> = {
      r1: {
        ...DEFAULT_ISSO53_ROOM,
        gebruiksFunctie: "onderwijs",
        ruimteType: "lesruimte",
        personen: 28,
      },
    };

    useProjectStore.setState({
      norm: "isso53",
      isso53Building: building,
      isso53Rooms: rooms,
    });

    exportProject(project, result);

    // Envelope bevat norm + isso53.
    const env = parseExported();
    expect(env.norm).toBe("isso53");
    expect(env.isso53).toBeTruthy();

    const imported = importProject(lastBlobContent) as ImportResult;
    expect(imported.norm).toBe("isso53");
    expect(imported.isso53?.building).toEqual(building);
    expect(imported.isso53?.rooms).toEqual(rooms);
  });

  it("setProject met opts herstelt de store-sidecars autoritatief", () => {
    const project = makeIsso53Project();
    const building: Isso53BuildingState = {
      ...DEFAULT_ISSO53_BUILDING,
      thermalMass: "licht",
    };
    const rooms: Record<string, Isso53RoomState> = {
      r1: { ...DEFAULT_ISSO53_ROOM, personen: 12 },
    };

    useProjectStore.getState().setProject(project, {
      norm: "isso53",
      isso53Building: building,
      isso53Rooms: rooms,
    });

    const state = useProjectStore.getState();
    expect(state.norm).toBe("isso53");
    expect(state.isso53Building).toEqual(building);
    expect(state.isso53Rooms).toEqual(rooms);
  });
});

// ---------------------------------------------------------------------------
// (b2) SharedExtra (bouwjaar etc.) — sidecar round-trip
// ---------------------------------------------------------------------------

describe("exportProject — sharedExtra sidecar", () => {
  it("round-trip behoudt construction_year + overige sharedExtra-velden", () => {
    const project = makeIsso51Project();
    const result = makeResult();

    const extra: SharedExtra = {
      ...DEFAULT_SHARED_EXTRA,
      construction_year: 1987,
      postcode: "1234 AB",
      location: "Amsterdam",
      num_storeys: 3,
    };

    useProjectStore.setState({ norm: "isso51", sharedExtra: extra });

    exportProject(project, result);

    // Envelope bevat het sharedExtra-veld (niet-default).
    const env = parseExported();
    expect(env.sharedExtra).toBeTruthy();
    expect((env.sharedExtra as SharedExtra).construction_year).toBe(1987);

    const imported = importProject(lastBlobContent) as ImportResult;
    expect(imported.sharedExtra).toBeTruthy();
    expect(imported.sharedExtra?.construction_year).toBe(1987);
    expect(imported.sharedExtra?.postcode).toBe("1234 AB");
    expect(imported.sharedExtra?.location).toBe("Amsterdam");
    expect(imported.sharedExtra?.num_storeys).toBe(3);

    // setProject herstelt de sidecar autoritatief in de store.
    useProjectStore.getState().setProject(imported.project, {
      sharedExtra: imported.sharedExtra,
    });
    expect(useProjectStore.getState().sharedExtra.construction_year).toBe(1987);
  });

  it("oud bestand zonder sharedExtra → store valt terug op defaults", () => {
    const project = makeIsso51Project();
    const legacyEnvelope = {
      version: "1.0.0",
      schema: "isso51-project-v1",
      exported_at: "2025-01-01T00:00:00.000Z",
      project,
      result: null,
      // NB: bewust GEEN sharedExtra — oud bestand.
    };

    const imported = importProject(
      JSON.stringify(legacyEnvelope),
    ) as ImportResult;
    expect(imported.sharedExtra).toBeUndefined();

    useProjectStore.getState().setProject(imported.project, {
      sharedExtra: imported.sharedExtra,
    });
    expect(useProjectStore.getState().sharedExtra).toEqual(DEFAULT_SHARED_EXTRA);
  });
});

// ---------------------------------------------------------------------------
// (c) Oud bestand zonder norm/isso53 — backward-compat
// ---------------------------------------------------------------------------

describe("importProject — legacy .isso51.json zonder sidecars", () => {
  it("laadt als ISSO 51 zonder crash; geen norm/isso53 in result", () => {
    const project = makeIsso51Project();
    const legacyEnvelope = {
      version: "1.0.0",
      schema: "isso51-project-v1",
      exported_at: "2025-01-01T00:00:00.000Z",
      project,
      result: null,
      // NB: bewust GEEN norm/isso53 — dit simuleert een oud bestand.
    };

    const imported = importProject(
      JSON.stringify(legacyEnvelope),
    ) as ImportResult;

    expect(imported.type).toBe("project");
    expect(imported.project.info.name).toBe("Test ISSO 51");
    expect(imported.norm).toBeUndefined();
    expect(imported.isso53).toBeUndefined();

    // setProject zonder opts valt terug op detectie + defaults.
    useProjectStore.getState().setProject(imported.project, {
      norm: imported.norm,
      isso53Building: imported.isso53?.building,
      isso53Rooms: imported.isso53?.rooms,
    });
    const state = useProjectStore.getState();
    expect(state.norm).toBe("isso51");
    expect(state.isso53Building).toEqual(DEFAULT_ISSO53_BUILDING);
    expect(state.isso53Rooms).toEqual({});
  });
});

// ---------------------------------------------------------------------------
// (d) Ventilatie-sidecar — save→reopen van `system` + `occupancy`
// ---------------------------------------------------------------------------

describe("exportProject — ventilatie-sidecar (system + occupancy)", () => {
  function makeVentilation(): VentilationState {
    return {
      system: "D",
      terminals: [
        {
          id: "vent-1",
          roomId: "r1",
          type: "supply",
          source: "manual",
          wallIndex: 0,
          offsetMm: 1200,
          flowDm3s: 25,
        },
      ],
      rooms: {
        r1: {
          ventilationFunction: "verblijfsruimte",
          requiredSupplyDm3s: 32,
          requiredExhaustDm3s: 0,
          airSourceRoomId: null,
          occupancy: 8,
        },
      },
    };
  }

  it("round-trip behoudt system, occupancy en terminals exact", () => {
    const project = makeIsso51Project();
    const ventilation = makeVentilation();
    useProjectStore.setState({ norm: "isso51", ventilation });

    exportProject(project, makeResult());

    const env = parseExported();
    expect((env.ventilation as VentilationState).system).toBe("D");

    const imported = importProject(lastBlobContent) as ImportResult;
    expect(imported.ventilation).toEqual(ventilation);
    expect(imported.ventilation?.rooms.r1?.occupancy).toBe(8);

    // setProject herstelt de sidecar (incl. system) autoritatief in de store.
    useProjectStore.getState().setProject(imported.project, {
      ventilation: imported.ventilation,
    });
    const state = useProjectStore.getState();
    expect(state.ventilation.system).toBe("D");
    expect(state.ventilation.rooms.r1?.occupancy).toBe(8);
    expect(state.ventilation.terminals).toHaveLength(1);
  });

  it("alleen een gekozen systeem (geen ventielen/rooms) overleeft de round-trip", () => {
    const project = makeIsso51Project();
    useProjectStore.setState({
      norm: "isso51",
      ventilation: { terminals: [], rooms: {}, system: "A" },
    });

    exportProject(project, makeResult());
    const imported = importProject(lastBlobContent) as ImportResult;
    expect(imported.ventilation?.system).toBe("A");
  });

  it("oud bestand zonder ventilatie/system → defaults (geen crash)", () => {
    const project = makeIsso51Project();
    const legacyEnvelope = {
      version: "1.0.0",
      schema: "isso51-project-v1",
      exported_at: "2025-01-01T00:00:00.000Z",
      project,
      result: null,
      // NB: bewust GEEN ventilation — oud bestand.
    };

    const imported = importProject(
      JSON.stringify(legacyEnvelope),
    ) as ImportResult;
    expect(imported.ventilation).toBeUndefined();

    useProjectStore.getState().setProject(imported.project, {
      ventilation: imported.ventilation,
    });
    const state = useProjectStore.getState();
    expect(state.ventilation.terminals).toEqual([]);
    expect(state.ventilation.rooms).toEqual({});
    // Geen expliciet systeem → undefined, downstream default (C).
    expect(state.ventilation.system).toBeUndefined();
  });

  it("ouder envelope met ventilation zonder system-veld laadt zonder system", () => {
    const project = makeIsso51Project();
    const ventilation = makeVentilation();
    const { system: _system, ...ventilationZonderSystem } = ventilation;
    const envelope = {
      version: "1.0.0",
      schema: "isso51-project-v1",
      exported_at: "2025-01-01T00:00:00.000Z",
      project,
      result: null,
      ventilation: ventilationZonderSystem,
    };

    const imported = importProject(JSON.stringify(envelope)) as ImportResult;
    expect(imported.ventilation?.system).toBeUndefined();
    expect(imported.ventilation?.rooms.r1?.occupancy).toBe(8);
  });
});

// ---------------------------------------------------------------------------
// (e) Ventilatie-sidecar — save→reopen van WTW/MV-units + toewijzingen
//     in BEIDE envelopes (.heatloss.json én .ifcenergy)
// ---------------------------------------------------------------------------

describe("ventilatie-units — save→reopen (beide envelopes)", () => {
  function makeVentilationWithUnits(): VentilationState {
    return {
      system: "D",
      terminals: [],
      rooms: {
        r1: {
          ventilationFunction: "verblijfsruimte",
          requiredSupplyDm3s: 32,
          requiredExhaustDm3s: 0,
          airSourceRoomId: null,
        },
      },
      units: [
        {
          id: "zehnder-comfoair-q450",
          type: "wtw",
          fabrikant: "Zehnder",
          model: "ComfoAir Q450",
          capaciteitM3h: 450,
          rendement: 0.9,
          source: "catalog",
        },
        {
          id: "unit-custom-1",
          type: "mv",
          fabrikant: "Eigen",
          model: "MV Box",
          capaciteitM3h: 250,
          geluidDb: 45,
          source: "custom",
        },
      ],
      unitAssignments: [
        { unitId: "zehnder-comfoair-q450", aantal: 2 },
        { unitId: "unit-custom-1", aantal: 1 },
      ],
    };
  }

  it(".heatloss.json: round-trip behoudt units + toewijzingen exact", () => {
    const project = makeIsso51Project();
    const ventilation = makeVentilationWithUnits();
    useProjectStore.setState({ norm: "isso51", ventilation });

    exportProject(project, makeResult());
    const imported = importProject(lastBlobContent) as ImportResult;
    expect(imported.ventilation).toEqual(ventilation);
    expect(imported.ventilation?.units).toHaveLength(2);
    expect(imported.ventilation?.unitAssignments).toEqual(
      ventilation.unitAssignments,
    );

    // setProject herstelt de sidecar (incl. units) autoritatief in de store.
    useProjectStore.getState().setProject(imported.project, {
      ventilation: imported.ventilation,
    });
    const state = useProjectStore.getState();
    expect(state.ventilation.units).toEqual(ventilation.units);
    expect(state.ventilation.unitAssignments).toEqual(
      ventilation.unitAssignments,
    );
  });

  it(".ifcenergy: round-trip behoudt units + toewijzingen exact", () => {
    const project = makeIsso51Project();
    const ventilation = makeVentilationWithUnits();

    const doc = buildIfcEnergyDocument({
      project,
      result: null,
      modeller: emptyModellerSnapshot(),
      ventilation,
    });
    const json = serializeIfcEnergy(doc);

    const imported = openProjectFile(json) as ImportResult & {
      format?: string;
    };
    expect(imported.format).toBe("ifcenergy");
    expect(imported.ventilation).toEqual(ventilation);

    useProjectStore.getState().setProject(imported.project, {
      ventilation: imported.ventilation,
    });
    const state = useProjectStore.getState();
    expect(state.ventilation.units).toEqual(ventilation.units);
    expect(state.ventilation.unitAssignments).toEqual(
      ventilation.unitAssignments,
    );
    expect(state.ventilation.system).toBe("D");
  });

  it("envelope zonder units-velden → undefined (oude bestanden, geen crash)", () => {
    const project = makeIsso51Project();
    const envelope = {
      version: "1.0.0",
      schema: "isso51-project-v1",
      exported_at: "2025-01-01T00:00:00.000Z",
      project,
      result: null,
      ventilation: { terminals: [], rooms: {}, system: "C" },
    };

    const imported = importProject(JSON.stringify(envelope)) as ImportResult;
    expect(imported.ventilation?.units).toBeUndefined();
    expect(imported.ventilation?.unitAssignments).toBeUndefined();

    useProjectStore.getState().setProject(imported.project, {
      ventilation: imported.ventilation,
    });
    const state = useProjectStore.getState();
    expect(state.ventilation.units).toBeUndefined();
    expect(state.ventilation.unitAssignments).toBeUndefined();
  });
});
