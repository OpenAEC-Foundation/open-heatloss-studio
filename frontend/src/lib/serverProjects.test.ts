import { afterEach, describe, expect, it, vi } from "vitest";

import {
  applyServerProjectResponse,
  buildServerProjectData,
  saveExistingServerProject,
} from "./serverProjects";
import { SessionExpiredError } from "./backend";
import { useProjectStore } from "../store/projectStore";
import { useSaveStatusStore } from "../store/saveStatusStore";
import { useModellerStore } from "../components/modeller/modellerStore";
import type {
  ModelDoor,
  ModelRoom,
  ModelWindow,
} from "../components/modeller/types";
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
 * Tests voor de envelope-pariteit van de server-save/-load flow
 * (`lib/serverProjects.ts`).
 *
 * Geborgd gedrag:
 *   (a) Round-trip "pc A → server → pc B": de save-payload
 *       ({@link buildServerProjectData}) bevat de volledige envelope en
 *       {@link applyServerProjectResponse} herstelt modeller-geometrie,
 *       ventilatie, ISSO 53-sidecars én sharedExtra — identieke staat op
 *       een andere machine.
 *   (b) Backward-compat: legacy kaal `project_data` (alleen een
 *       Project-object, van vóór de envelope-fix) laadt zonder crash met
 *       defaults.
 *   (c) De modeller-store wordt expliciet geleegd/gevuld bij wissel van
 *       serverproject — geen stale geometrie van het vorige project.
 *   (d) Result-keuze: `envelope.result` wint; `result_data` is fallback
 *       voor legacy rijen.
 *   (e) Race-guard: een stale save (id ≠ actief project) is een stille
 *       no-op — er gaat géén verkeerde payload over de lijn.
 *   (f) Save-status wordt gereset bij projectwissel/reset — geen stale
 *       "Conflict"/"Offline"/"Fout"-indicator op een ander project.
 */

afterEach(() => {
  vi.unstubAllGlobals();
  useProjectStore.getState().reset();
  useSaveStatusStore.getState().resetStatus();
  useModellerStore.getState().importModel([], [], []);
  useModellerStore.getState().importProjectConstructions([]);
});

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeProject(name = "Server roundtrip"): Project {
  return {
    info: { name },
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
      // fixture zodat de round-trip deep-equal sluit.
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
        constructions: [
          {
            id: "c1",
            description: "Buitenwand spouw",
            area: 12.5,
            u_value: 0.21,
            boundary_type: "exterior",
            material_type: "masonry",
            vertical_position: "wall",
          },
        ],
        heating_system: "radiator_ht",
      },
    ],
  };
}

function makeIsso53Project(): Project {
  const p = makeProject("Server roundtrip ISSO 53");
  p.building.default_heating_system = "radiatorenConvHtEnLuchtverwarming";
  p.rooms = p.rooms.map((r) => ({
    ...r,
    heating_system: "radiatorenConvHtEnLuchtverwarming",
  }));
  return p;
}

const MODELLER_ROOMS: ModelRoom[] = [
  {
    id: "r1",
    name: "Woonkamer",
    function: "living_room",
    polygon: [
      { x: 0, y: 0 },
      { x: 5000, y: 0 },
      { x: 5000, y: 4000 },
      { x: 0, y: 4000 },
    ],
    floor: 0,
    height: 2600,
  },
];

const MODELLER_WINDOWS: ModelWindow[] = [
  { roomId: "r1", wallIndex: 0, offset: 1200, width: 1800, height: 1400 },
];

const MODELLER_DOORS: ModelDoor[] = [
  { roomId: "r1", wallIndex: 2, offset: 800, width: 930, swing: "left" },
];

const VENTILATION: VentilationState = {
  terminals: [
    {
      id: "vent-1",
      roomId: "r1",
      type: "supply",
      source: "manual",
      wallIndex: 0,
      offsetMm: 2000,
      flowDm3s: 25,
    },
  ],
  rooms: {
    r1: {
      ventilationFunction: "verblijfsruimte",
      requiredSupplyDm3s: 18,
      requiredExhaustDm3s: 0,
      airSourceRoomId: null,
    },
  },
  system: "C",
};

const ISSO53_BUILDING: Isso53BuildingState = {
  ...DEFAULT_ISSO53_BUILDING,
  thermalMass: "zwaar",
};

const ISSO53_ROOMS: Record<string, Isso53RoomState> = {
  r1: { ...DEFAULT_ISSO53_ROOM, gebruiksFunctie: "kantoor", personen: 4 },
};

const SHARED_EXTRA: SharedExtra = {
  ...DEFAULT_SHARED_EXTRA,
  construction_year: 1992,
  postcode: "1234 AB",
};

function makeResult(): ProjectResult {
  return {
    rooms: [],
    summary: { total_heat_loss: 4321 },
  } as unknown as ProjectResult;
}

/** Seed de stores zoals "pc A": volledig project met alle sidecars. */
function seedPcA(): Project {
  const project = makeIsso53Project();
  useProjectStore.setState({
    project,
    norm: "isso53",
    isso53Building: ISSO53_BUILDING,
    isso53Rooms: ISSO53_ROOMS,
    sharedExtra: SHARED_EXTRA,
    ventilation: VENTILATION,
    result: makeResult(),
  });
  useModellerStore
    .getState()
    .importModel(MODELLER_ROOMS, MODELLER_WINDOWS, MODELLER_DOORS);
  return project;
}

/** Wis de stores zoals een verse sessie op "pc B". */
function resetToPcB(): void {
  useProjectStore.getState().reset();
  useModellerStore.getState().importModel([], [], []);
  useModellerStore.getState().importProjectConstructions([]);
}

/** Simuleer de JSON-wire (serde Value → fetch-response). */
function overWire<T>(payload: T): unknown {
  return JSON.parse(JSON.stringify(payload));
}

/**
 * Strip de (random UUID) `project_construction_id`-links die
 * `extractAndLinkConstructions` toekent, zodat een deep-equal tussen
 * "pc A" en "pc B" op de inhoudelijke velden vergelijkt.
 */
function stripPcIds(p: Project): Project {
  return {
    ...p,
    rooms: p.rooms.map((r) => ({
      ...r,
      constructions: r.constructions.map(
        ({ project_construction_id: _omit, ...c }) => c,
      ),
    })),
  } as Project;
}

// ---------------------------------------------------------------------------
// (a) Envelope round-trip — pc A → server → pc B
// ---------------------------------------------------------------------------

describe("server envelope round-trip", () => {
  it("herstelt geometrie + ventilatie + isso53 + sharedExtra op 'pc B'", () => {
    const project = seedPcA();
    const payload = buildServerProjectData();

    // De payload is de volledige envelope, niet het kale project.
    expect(payload.schema).toBe("isso51-project-v1");
    expect(payload.modeller?.rooms).toHaveLength(1);
    expect(payload.norm).toBe("isso53");
    expect(payload.ventilation?.terminals).toHaveLength(1);
    expect(payload.sharedExtra?.construction_year).toBe(1992);

    resetToPcB();
    expect(useModellerStore.getState().rooms).toHaveLength(0);

    applyServerProjectResponse("proj-1", {
      project_data: overWire(payload),
      result_data: null,
      updated_at: "2026-06-10 10:00:00",
    });

    const state = useProjectStore.getState();
    expect(stripPcIds(state.project)).toEqual(stripPcIds(project));
    expect(state.activeProjectId).toBe("proj-1");
    expect(state.serverUpdatedAt).toBe("2026-06-10 10:00:00");
    expect(state.isDirty).toBe(false);

    // Sidecars hersteld — niet naar defaults gereset.
    expect(state.norm).toBe("isso53");
    expect(state.isso53Building).toEqual(ISSO53_BUILDING);
    expect(state.isso53Rooms).toEqual(ISSO53_ROOMS);
    expect(state.sharedExtra).toEqual(SHARED_EXTRA);
    expect(state.ventilation).toEqual(VENTILATION);

    // Modeller-geometrie hersteld.
    const modeller = useModellerStore.getState();
    expect(modeller.rooms).toEqual(MODELLER_ROOMS);
    expect(modeller.windows).toEqual(MODELLER_WINDOWS);
    expect(modeller.doors).toEqual(MODELLER_DOORS);
    // Project-constructies opnieuw gelinkt vanuit de room-elementen
    // (zelfde extractAndLinkConstructions-nabewerking als bestand-openen).
    expect(modeller.projectConstructions).toHaveLength(1);
    expect(modeller.projectConstructions[0]?.name).toBe("Buitenwand spouw");
  });

  it("envelope-result wint; result_data is fallback voor legacy rijen", () => {
    seedPcA();
    const payload = buildServerProjectData();
    resetToPcB();

    // Envelope draagt zijn eigen result mee — result_data wordt genegeerd.
    applyServerProjectResponse("proj-1", {
      project_data: overWire(payload),
      result_data: overWire({
        rooms: [],
        summary: { total_heat_loss: 9999 },
      }),
      updated_at: "2026-06-10 10:00:00",
    });
    expect(
      (useProjectStore.getState().result as ProjectResult).summary
        .total_heat_loss,
    ).toBe(4321);

    // Legacy kale rij zonder envelope → result_data is de enige bron.
    resetToPcB();
    applyServerProjectResponse("proj-2", {
      project_data: overWire(makeProject("Legacy")),
      result_data: overWire({
        rooms: [],
        summary: { total_heat_loss: 9999 },
      }),
      updated_at: "2026-06-10 11:00:00",
    });
    expect(
      (useProjectStore.getState().result as ProjectResult).summary
        .total_heat_loss,
    ).toBe(9999);
  });
});

// ---------------------------------------------------------------------------
// (b) Backward-compat — legacy kaal project_data
// ---------------------------------------------------------------------------

describe("legacy kaal project_data", () => {
  it("laadt zonder crash met default sidecars", () => {
    const legacy = makeProject("Legacy project");

    applyServerProjectResponse("proj-legacy", {
      project_data: overWire(legacy),
      result_data: null,
      updated_at: "2026-06-10 09:00:00",
    });

    const state = useProjectStore.getState();
    expect(state.project.info.name).toBe("Legacy project");
    expect(state.activeProjectId).toBe("proj-legacy");
    expect(state.norm).toBe("isso51");
    expect(state.isso53Building).toEqual(DEFAULT_ISSO53_BUILDING);
    expect(state.isso53Rooms).toEqual({});
    expect(state.sharedExtra).toEqual(DEFAULT_SHARED_EXTRA);
    expect(state.ventilation).toEqual({ terminals: [], rooms: {} });
    expect(state.result).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// (c) Modeller-store geleegd/gevuld bij serverproject-wissel
// ---------------------------------------------------------------------------

describe("modeller-store bij serverproject-wissel", () => {
  it("leegt stale geometrie wanneer een legacy project zonder envelope opent", () => {
    // Eerst een envelope-project mét geometrie laden.
    seedPcA();
    const payload = buildServerProjectData();
    resetToPcB();
    applyServerProjectResponse("proj-1", {
      project_data: overWire(payload),
      result_data: null,
      updated_at: "2026-06-10 10:00:00",
    });
    expect(useModellerStore.getState().rooms).toHaveLength(1);

    // Daarna een legacy kaal project openen → geometrie moet weg.
    applyServerProjectResponse("proj-2", {
      project_data: overWire(makeProject("Legacy zonder geometrie")),
      result_data: null,
      updated_at: "2026-06-10 11:00:00",
    });
    const modeller = useModellerStore.getState();
    expect(modeller.rooms).toHaveLength(0);
    expect(modeller.windows).toHaveLength(0);
    expect(modeller.doors).toHaveLength(0);
  });
});

// ---------------------------------------------------------------------------
// (e) Race-guard — stale save na projectwissel
// ---------------------------------------------------------------------------

describe("saveExistingServerProject — race-guard bij projectwissel", () => {
  /** Response-stub die parseResponse als geldige JSON-200 accepteert. */
  function fakeOkResponse(body: unknown) {
    return {
      ok: true,
      redirected: false,
      status: 200,
      headers: { get: () => "application/json" },
      json: async () => body,
    };
  }

  it("breekt stil af (null, geen API-call) wanneer het id niet meer actief is", async () => {
    const fetchSpy = vi.fn(async () => {
      throw new Error("fetch hoort niet aangeroepen te worden");
    });
    vi.stubGlobal("fetch", fetchSpy);

    // Project A was geladen, daarna is project B actief geworden — de
    // debounce-timer van A vuurt te laat.
    applyServerProjectResponse("proj-A", {
      project_data: overWire(makeProject("Project A")),
      result_data: null,
      updated_at: "2026-06-10 10:00:00",
    });
    applyServerProjectResponse("proj-B", {
      project_data: overWire(makeProject("Project B")),
      result_data: null,
      updated_at: "2026-06-10 10:05:00",
    });

    const result = await saveExistingServerProject("proj-A");

    expect(result).toBeNull();
    expect(fetchSpy).not.toHaveBeenCalled();
    // Stille no-op: ook de statusindicator blijft onaangeroerd (idle na load).
    expect(useSaveStatusStore.getState().status).toBe("idle");
    // Project B-data is niet als "opgeslagen" gemarkeerd onder A's vlag.
    expect(useProjectStore.getState().project.info.name).toBe("Project B");
  });

  it("slaat wél op wanneer het id het actieve project is (guard blokkeert niet te veel)", async () => {
    const fetchSpy = vi.fn(async () =>
      fakeOkResponse({ ok: true, updated_at: "2026-06-10 10:10:00" }),
    );
    vi.stubGlobal("fetch", fetchSpy);

    applyServerProjectResponse("proj-B", {
      project_data: overWire(makeProject("Project B")),
      result_data: null,
      updated_at: "2026-06-10 10:05:00",
    });

    const result = await saveExistingServerProject("proj-B");

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    expect(result?.updated_at).toBe("2026-06-10 10:10:00");
    const state = useProjectStore.getState();
    expect(state.serverUpdatedAt).toBe("2026-06-10 10:10:00");
    expect(state.isDirty).toBe(false);
    expect(useSaveStatusStore.getState().status).toBe("saved");
  });
});

// ---------------------------------------------------------------------------
// (e2) Verlopen sessie — serverbinding loskoppelen (R1)
// ---------------------------------------------------------------------------

describe("saveExistingServerProject — definitief verlopen sessie (R1)", () => {
  it("koppelt de serverbinding los maar laat het project + dirty-vlag staan", async () => {
    // 401 van de forward-auth proxy → parseResponse gooit SessionExpiredError.
    const fetchSpy = vi.fn(async () => ({
      ok: false,
      redirected: false,
      status: 401,
      headers: { get: () => "application/json" },
      json: async () => ({}),
    }));
    vi.stubGlobal("fetch", fetchSpy);

    applyServerProjectResponse("proj-A", {
      project_data: overWire(makeProject("Project van user A")),
      result_data: null,
      updated_at: "2026-06-10 10:00:00",
    });
    // Onopgeslagen wijziging vóór de mislukte save.
    useProjectStore.setState({ isDirty: true });

    await expect(saveExistingServerProject("proj-A")).rejects.toThrowError(
      SessionExpiredError,
    );

    const s = useProjectStore.getState();
    // Binding los — een volgende (andere) gebruiker op deze browser erft
    // geen activeProjectId/serverUpdatedAt van user A via localStorage.
    expect(s.activeProjectId).toBeNull();
    expect(s.serverUpdatedAt).toBeNull();
    expect(s.hasConflict).toBe(false);
    // Werk niet weggegooid: project + dirty-vlag blijven staan.
    expect(s.project.info.name).toBe("Project van user A");
    expect(s.isDirty).toBe(true);
    // Geen hangende "Opslaan…"/"Fout"-indicator voor een losgekoppeld project.
    expect(useSaveStatusStore.getState().status).toBe("idle");
  });
});

// ---------------------------------------------------------------------------
// (f) Save-status gereset bij projectwissel/reset
// ---------------------------------------------------------------------------

describe("save-status reset bij projectwissel", () => {
  it("projectStore.reset() zet een stale status terug naar idle", () => {
    useSaveStatusStore.getState().setConflict();
    expect(useSaveStatusStore.getState().status).toBe("conflict");

    useProjectStore.getState().reset();

    expect(useSaveStatusStore.getState().status).toBe("idle");
  });

  it("documentsStore tab-wissel zet een stale status terug naar idle", async () => {
    // Lazy import: documentsStore persist't naar localStorage; in de
    // node-testomgeving valt dat stil terug (zustand-warning, geen error).
    const { useDocumentsStore } = await import("../store/documentsStore");

    const tabA = useDocumentsStore.getState().newTab();
    const tabB = useDocumentsStore.getState().newTab();
    expect(useDocumentsStore.getState().activeId).toBe(tabB);

    useSaveStatusStore.getState().setError("save mislukt op tab B");
    expect(useSaveStatusStore.getState().status).toBe("error");

    useDocumentsStore.getState().switchTab(tabA);

    expect(useSaveStatusStore.getState().status).toBe("idle");
  });

  it("applyServerProjectResponse reset een stale status (consistent met reset())", () => {
    useSaveStatusStore.getState().setOffline();

    applyServerProjectResponse("proj-fresh", {
      project_data: overWire(makeProject("Vers project")),
      result_data: null,
      updated_at: "2026-06-10 12:00:00",
    });

    expect(useSaveStatusStore.getState().status).toBe("idle");
  });
});
