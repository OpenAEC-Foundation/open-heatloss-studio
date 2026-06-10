import { afterEach, describe, expect, it } from "vitest";

import { useProjectStore } from "./projectStore";
import type { ConstructionElement, Project, Room } from "../types";
import type { VentilationState } from "../types/ventilation";

/**
 * Tests voor {@link useProjectStore.syncProjectConstruction} — de propagatie
 * van een bewerkte ProjectConstruction naar álle gekoppelde room-elementen.
 *
 * Gedrag dat geborgd wordt:
 *   - Alleen elementen met `project_construction_id === pcId` worden geraakt.
 *   - Uitsluitend de type-definiërende velden (`description`, `u_value`,
 *     `material_type`, `vertical_position`, `layers`) worden overschreven.
 *   - Element-specifieke velden (`id`, `area`, `boundary_type`,
 *     `adjacent_room_id`, `uw_breakdown`) blijven ongemoeid.
 *   - De mutatie is undo-aware (één `undo()` herstelt alle elementen).
 */

function makeElement(
  overrides: Partial<ConstructionElement> & Pick<ConstructionElement, "id">,
): ConstructionElement {
  return {
    description: "Oude wand",
    area: 12.5,
    u_value: 0.3,
    boundary_type: "exterior",
    material_type: "masonry",
    vertical_position: "wall",
    ...overrides,
  };
}

function makeRoom(
  id: string,
  constructions: ConstructionElement[],
): Room {
  return {
    id,
    name: `Ruimte ${id}`,
    function: "living_room",
    floor_area: 20,
    constructions,
    heating_system: "radiator_ht",
  };
}

function seedProject(rooms: Room[]): void {
  const base: Project = {
    info: { name: "Test" },
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
    rooms,
  };
  useProjectStore.setState({ project: base, _past: [], _future: [] });
}

afterEach(() => {
  useProjectStore.getState().reset();
});

describe("syncProjectConstruction", () => {
  it("overschrijft type-velden op alle gekoppelde elementen en laat element-velden ongemoeid", () => {
    const linkedA = makeElement({
      id: "el-a",
      area: 10,
      boundary_type: "exterior",
      adjacent_room_id: null,
      project_construction_id: "proj-1",
      u_value: 0.3,
      uw_breakdown: undefined,
    });
    const linkedB = makeElement({
      id: "el-b",
      area: 25,
      boundary_type: "ground",
      adjacent_room_id: "room-x",
      project_construction_id: "proj-1",
      u_value: 0.3,
    });
    const unlinked = makeElement({
      id: "el-c",
      area: 5,
      project_construction_id: "proj-2",
      u_value: 0.9,
    });

    seedProject([
      makeRoom("r1", [linkedA, unlinked]),
      makeRoom("r2", [linkedB]),
    ]);

    useProjectStore.getState().syncProjectConstruction("proj-1", {
      description: "Nieuwe wand",
      u_value: 0.18,
      material_type: "non_masonry",
      vertical_position: "ceiling",
      layers: [{ materialId: "pir", thickness: 110 }],
    });

    const rooms = useProjectStore.getState().project.rooms;
    const a = rooms[0]!.constructions.find((c) => c.id === "el-a")!;
    const b = rooms[1]!.constructions.find((c) => c.id === "el-b")!;
    const c = rooms[0]!.constructions.find((c) => c.id === "el-c")!;

    // Type-velden overschreven op gekoppelde elementen.
    for (const el of [a, b]) {
      expect(el.description).toBe("Nieuwe wand");
      expect(el.u_value).toBe(0.18);
      expect(el.material_type).toBe("non_masonry");
      expect(el.vertical_position).toBe("ceiling");
      expect(el.layers).toEqual([{ materialId: "pir", thickness: 110 }]);
    }

    // Element-specifieke velden ongemoeid.
    expect(a.area).toBe(10);
    expect(a.boundary_type).toBe("exterior");
    expect(a.adjacent_room_id).toBe(null);
    expect(b.area).toBe(25);
    expect(b.boundary_type).toBe("ground");
    expect(b.adjacent_room_id).toBe("room-x");

    // Niet-gekoppeld element volledig ongemoeid.
    expect(c.description).toBe("Oude wand");
    expect(c.u_value).toBe(0.9);
    expect(c.material_type).toBe("masonry");
  });

  it("zet layers op undefined wanneer een lege laag-lijst wordt doorgegeven", () => {
    const linked = makeElement({
      id: "el-a",
      project_construction_id: "proj-1",
      layers: [{ materialId: "steen", thickness: 100 }],
    });
    seedProject([makeRoom("r1", [linked])]);

    useProjectStore.getState().syncProjectConstruction("proj-1", {
      description: "Triple glas",
      u_value: 0.8,
      material_type: "non_masonry",
      vertical_position: "wall",
      layers: [],
    });

    const el = useProjectStore.getState().project.rooms[0]!.constructions[0]!;
    expect(el.layers).toBeUndefined();
    expect(el.u_value).toBe(0.8);
  });

  it("is undo-aware: één undo herstelt alle elementen", () => {
    const linked = makeElement({
      id: "el-a",
      project_construction_id: "proj-1",
      u_value: 0.3,
      description: "Oude wand",
    });
    seedProject([makeRoom("r1", [linked])]);

    useProjectStore.getState().syncProjectConstruction("proj-1", {
      description: "Nieuwe wand",
      u_value: 0.18,
      material_type: "masonry",
      vertical_position: "wall",
      layers: [{ materialId: "pir", thickness: 110 }],
    });

    expect(
      useProjectStore.getState().project.rooms[0]!.constructions[0]!.u_value,
    ).toBe(0.18);

    useProjectStore.getState().undo();

    const restored =
      useProjectStore.getState().project.rooms[0]!.constructions[0]!;
    expect(restored.u_value).toBe(0.3);
    expect(restored.description).toBe("Oude wand");
  });
});

// ---------------------------------------------------------------------------
// Ventilatie-sidecar — regressietests delegatie 6 (WTW/MV-units)
// ---------------------------------------------------------------------------

/**
 * Geborgd gedrag:
 *   (a) `removeRoom` behoudt de gebouw-niveau ventilatievelden (`system`,
 *       `units`, `unitAssignments`) — vóór de spread-fix herbouwde
 *       `removeRoom` het ventilation-object met alleen terminals+rooms en
 *       gingen die velden verloren.
 *   (b) `removeVentilationUnit` verwijdert cascade óók de toewijzingen die
 *       naar de unit wijzen (geen dangling `unitId`-referenties).
 */

function makeVentilationWithUnits(): VentilationState {
  return {
    system: "D",
    terminals: [
      { id: "t1", roomId: "r1", type: "supply", source: "manual", flowDm3s: 25 },
      { id: "t2", roomId: "r2", type: "exhaust", source: "manual", flowDm3s: 14 },
    ],
    rooms: {
      r1: {
        ventilationFunction: "verblijfsruimte",
        requiredSupplyDm3s: 20,
        requiredExhaustDm3s: 0,
        airSourceRoomId: null,
      },
      r2: {
        ventilationFunction: "badruimte",
        requiredSupplyDm3s: 0,
        requiredExhaustDm3s: 14,
        airSourceRoomId: null,
      },
    },
    units: [
      {
        id: "u-wtw",
        type: "wtw",
        fabrikant: "Zehnder",
        model: "ComfoAir Q450",
        capaciteitM3h: 450,
        rendement: 0.9,
        source: "catalog",
      },
    ],
    unitAssignments: [{ unitId: "u-wtw", aantal: 2 }],
  };
}

describe("removeRoom — gebouw-niveau ventilatievelden blijven behouden", () => {
  it("behoudt system, units en unitAssignments; ruimt alleen room-data op", () => {
    seedProject([makeRoom("r1", []), makeRoom("r2", [])]);
    useProjectStore.getState().setVentilation(makeVentilationWithUnits());

    useProjectStore.getState().removeRoom("r1");

    const v = useProjectStore.getState().ventilation;
    // Gebouw-niveau velden onaangetast (regressie: spread-fix in removeRoom).
    expect(v.system).toBe("D");
    expect(v.units).toHaveLength(1);
    expect(v.unitAssignments).toEqual([{ unitId: "u-wtw", aantal: 2 }]);
    // Room-gebonden data van r1 wél opgeschoond; r2 blijft.
    expect(v.terminals.map((t) => t.id)).toEqual(["t2"]);
    expect(v.rooms.r1).toBeUndefined();
    expect(v.rooms.r2).toBeDefined();
  });
});

describe("removeVentilationUnit — cascade naar toewijzingen", () => {
  it("verwijdert de unit én de toewijzingen die ernaar wijzen", () => {
    const id = useProjectStore.getState().addVentilationUnit({
      type: "mv",
      fabrikant: "Orcon",
      model: "MVS-15",
      capaciteitM3h: 500,
      source: "catalog",
    });
    useProjectStore.getState().setVentilationUnitAssignment(id, 2);

    let v = useProjectStore.getState().ventilation;
    expect(v.units).toHaveLength(1);
    expect(v.unitAssignments).toEqual([{ unitId: id, aantal: 2 }]);

    useProjectStore.getState().removeVentilationUnit(id);

    v = useProjectStore.getState().ventilation;
    expect(v.units).toEqual([]);
    expect(v.unitAssignments).toEqual([]);
  });
});
