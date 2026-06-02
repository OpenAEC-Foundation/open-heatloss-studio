import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  exportProject,
  importProject,
  type ImportResult,
} from "./importExport";
import { useProjectStore } from "../store/projectStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import type { Project, ProjectResult } from "../types";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
  type Isso53BuildingState,
  type Isso53RoomState,
} from "../types/projectV2";

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
