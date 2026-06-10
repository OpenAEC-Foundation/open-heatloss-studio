import { afterEach, describe, expect, it, vi } from "vitest";

import { useDocumentsStore } from "./documentsStore";
import { useProjectStore } from "./projectStore";
import { useSaveStatusStore } from "./saveStatusStore";
import { saveExistingServerProject } from "../lib/serverProjects";
import {
  DEFAULT_ISSO53_BUILDING,
  DEFAULT_ISSO53_ROOM,
} from "../types/projectV2";
import type { VentilationState } from "../types/ventilation";

/**
 * Tests voor de tab-wissel-isolatie van de documentsStore
 * (audit 09 §2.1 + §2.2).
 *
 * Geborgd gedrag:
 *   (a) Norm + sidecars (`norm`, `isso53Building`, `isso53Rooms`,
 *       `ventilation`) zijn per tab — geen rekenkern-routing op de norm
 *       van de vórige tab en geen sidecar-lekkage tussen tabs.
 *   (b) Server-binding (`activeProjectId`/`serverUpdatedAt`/`hasConflict`)
 *       is per tab — een auto-save op tab B schrijft nooit onder het
 *       project-id van tab A.
 *   (c) Legacy persisted snapshots (localStorage van vóór deze velden)
 *       laden met defaults i.p.v. te crashen of stale state te houden.
 *   (d) Samenspel met de race-guard in `saveExistingServerProject`: een
 *       stale debounce-save van tab A is na een tab-wissel een stille
 *       no-op (het tabs-pad omzeilde de guard vóór deze fix).
 */

const VENTILATION_A: VentilationState = {
  terminals: [
    { id: "t1", roomId: "r1", type: "supply", source: "manual", flowDm3s: 25 },
  ],
  rooms: {
    r1: {
      ventilationFunction: "verblijfsruimte",
      requiredSupplyDm3s: 18,
      requiredExhaustDm3s: 0,
      airSourceRoomId: null,
    },
  },
  system: "D",
};

/** Seed de actieve tab als "project A": ISSO 53 + sidecars + serverbinding. */
function seedTabA(): void {
  useProjectStore.setState({
    project: {
      ...useProjectStore.getState().project,
      info: { name: "Project A" },
    },
    norm: "isso53",
    isso53Building: { ...DEFAULT_ISSO53_BUILDING, thermalMass: "zwaar" },
    isso53Rooms: { r1: { ...DEFAULT_ISSO53_ROOM, gebruiksFunctie: "kantoor" } },
    ventilation: VENTILATION_A,
    activeProjectId: "proj-A",
    serverUpdatedAt: "2026-06-10 10:00:00",
    hasConflict: true,
    isDirty: true,
  });
}

function resetDocumentsStore(): void {
  useDocumentsStore.setState({
    tabs: [],
    snapshots: {},
    activeId: null,
    nextNamelessIndex: 1,
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
  resetDocumentsStore();
  useProjectStore.getState().reset();
  useSaveStatusStore.getState().resetStatus();
});

// ---------------------------------------------------------------------------
// (a) + (b) Tab-wissel — norm/sidecars/serverbinding per tab
// ---------------------------------------------------------------------------

describe("tab-wissel — norm, sidecars en serverbinding zijn per tab", () => {
  it("isoleert isso53-/ventilatie-sidecars en de serverbinding per tab", () => {
    const tabA = useDocumentsStore.getState().newTab();
    seedTabA();

    // Nieuwe (verse) tab B — snapshot van A wordt eerst vastgelegd.
    const tabB = useDocumentsStore.getState().newTab();
    expect(useDocumentsStore.getState().activeId).toBe(tabB);

    // Tab B is vers: isso51, lege sidecars, géén serverbinding van A.
    let s = useProjectStore.getState();
    expect(s.norm).toBe("isso51");
    expect(s.isso53Rooms).toEqual({});
    expect(s.ventilation).toEqual({ terminals: [], rooms: {} });
    expect(s.activeProjectId).toBeNull();
    expect(s.serverUpdatedAt).toBeNull();
    expect(s.hasConflict).toBe(false);

    // Terug naar A: norm + sidecars + serverbinding volledig hersteld.
    useDocumentsStore.getState().switchTab(tabA);
    s = useProjectStore.getState();
    expect(s.norm).toBe("isso53");
    expect(s.isso53Building.thermalMass).toBe("zwaar");
    expect(s.isso53Rooms.r1?.gebruiksFunctie).toBe("kantoor");
    expect(s.ventilation).toEqual(VENTILATION_A);
    expect(s.activeProjectId).toBe("proj-A");
    expect(s.serverUpdatedAt).toBe("2026-06-10 10:00:00");
    expect(s.hasConflict).toBe(true);

    // En weer naar B: geen lekkage van A's state.
    useDocumentsStore.getState().switchTab(tabB);
    s = useProjectStore.getState();
    expect(s.norm).toBe("isso51");
    expect(s.isso53Rooms).toEqual({});
    expect(s.activeProjectId).toBeNull();
    expect(s.hasConflict).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// (c) Legacy persisted snapshot — defaults i.p.v. stale state
// ---------------------------------------------------------------------------

describe("legacy snapshot zonder sidecar-/serverbinding-velden", () => {
  it("laadt met defaults + norm-detectie (geen crash, geen stale binding)", () => {
    const tabA = useDocumentsStore.getState().newTab();
    const tabB = useDocumentsStore.getState().newTab();
    expect(useDocumentsStore.getState().activeId).toBe(tabB);

    // Bouw een legacy snapshot na: alleen de velden van vóór audit 09.
    const snaps = useDocumentsStore.getState().snapshots;
    const full = snaps[tabA]!;
    const legacyProject = { ...full.project };
    delete legacyProject.norm;
    delete legacyProject.isso53Building;
    delete legacyProject.isso53Rooms;
    delete legacyProject.ventilation;
    delete legacyProject.activeProjectId;
    delete legacyProject.serverUpdatedAt;
    delete legacyProject.hasConflict;
    useDocumentsStore.setState({
      snapshots: { ...snaps, [tabA]: { ...full, project: legacyProject } },
    });

    useDocumentsStore.getState().switchTab(tabA);

    const s = useProjectStore.getState();
    expect(s.norm).toBe("isso51"); // detectie op default heating-shape
    expect(s.isso53Building).toEqual(DEFAULT_ISSO53_BUILDING);
    expect(s.isso53Rooms).toEqual({});
    expect(s.ventilation).toEqual({ terminals: [], rooms: {} });
    expect(s.activeProjectId).toBeNull();
    expect(s.serverUpdatedAt).toBeNull();
    expect(s.hasConflict).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// (d) Samenspel met de race-guard van saveExistingServerProject
// ---------------------------------------------------------------------------

describe("tabs-pad × race-guard saveExistingServerProject", () => {
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

  it("stale auto-save van tab A is een stille no-op na wissel naar tab B", async () => {
    const fetchSpy = vi.fn(async () => {
      throw new Error("fetch hoort niet aangeroepen te worden");
    });
    vi.stubGlobal("fetch", fetchSpy);

    useDocumentsStore.getState().newTab();
    seedTabA(); // activeProjectId = proj-A
    useDocumentsStore.getState().newTab(); // verse tab B, geen binding

    // De debounce-timer van tab A vuurt te laat — guard moet afbreken.
    const result = await saveExistingServerProject("proj-A");

    expect(result).toBeNull();
    expect(fetchSpy).not.toHaveBeenCalled();
    expect(useProjectStore.getState().activeProjectId).toBeNull();
  });

  it("na terugwissel naar tab A loopt de save weer onder A's binding", async () => {
    const fetchSpy = vi.fn(async () =>
      fakeOkResponse({ ok: true, updated_at: "2026-06-10 11:00:00" }),
    );
    vi.stubGlobal("fetch", fetchSpy);

    const tabA = useDocumentsStore.getState().newTab();
    seedTabA();
    // Conflict-vlag zou de save-flow vertroebelen — voor deze case schoon.
    useProjectStore.setState({ hasConflict: false });
    useDocumentsStore.getState().snapshotActive();
    useDocumentsStore.getState().newTab();
    useDocumentsStore.getState().switchTab(tabA);

    const result = await saveExistingServerProject("proj-A");

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    expect(result?.updated_at).toBe("2026-06-10 11:00:00");
    const s = useProjectStore.getState();
    expect(s.serverUpdatedAt).toBe("2026-06-10 11:00:00");
    expect(s.isDirty).toBe(false);
  });
});
