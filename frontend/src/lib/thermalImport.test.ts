import { describe, expect, it } from "vitest";

import { applyEditsToProject } from "./thermalImport";
import type { ThermalRoom } from "./thermalImport";
import type { Project, Room } from "../types";

/**
 * Tests voor de zone-mapping in {@link applyEditsToProject}:
 *   - find-or-create `Zone` op naam (case-sensitive exact match);
 *   - meerdere rooms met dezelfde zonenaam delen één `Zone`;
 *   - rooms zonder `zone`-veld blijven `zoneId: undefined`;
 *   - bestaande zones op het project worden hergebruikt, niet gedupliceerd;
 *   - thermal rooms die niet als project-room bestaan (pseudo-rooms) maken
 *     geen lege zones aan.
 */

function makeProjectRoom(id: string): Room {
  return {
    id,
    name: `Ruimte ${id}`,
    function: "living_room",
    floor_area: 20,
    constructions: [],
    heating_system: "radiator_ht",
  };
}

function makeProject(rooms: Room[]): Project {
  return {
    info: { name: "Test" },
    building: {
      building_type: "terraced",
      qv10: 100,
      total_floor_area: 80,
      security_class: "b",
    },
    climate: { theta_e: -10 },
    ventilation: { system_type: "system_c" },
    rooms,
  };
}

function makeThermalRoom(
  id: string,
  zone?: string,
): ThermalRoom {
  return { id, name: `Ruimte ${id}`, type: "heated", zone };
}

describe("applyEditsToProject — zone-mapping", () => {
  it("find-or-create: rooms met dezelfde zonenaam delen één Zone", () => {
    const project = makeProject([
      makeProjectRoom("r1"),
      makeProjectRoom("r2"),
      makeProjectRoom("r3"),
    ]);
    const thermalRooms = [
      makeThermalRoom("r1", "Zone 1"),
      makeThermalRoom("r2", "Zone 1"),
      makeThermalRoom("r3", "Zone 2"),
    ];

    const result = applyEditsToProject(project, thermalRooms, []);

    const zones = result.building.zones!;
    expect(zones).toHaveLength(2);
    expect(zones.map((z) => z.name)).toEqual(["Zone 1", "Zone 2"]);

    const byId = new Map(result.rooms.map((r) => [r.id, r.zoneId]));
    const zone1 = zones.find((z) => z.name === "Zone 1")!.id;
    const zone2 = zones.find((z) => z.name === "Zone 2")!.id;
    expect(byId.get("r1")).toBe(zone1);
    expect(byId.get("r2")).toBe(zone1);
    expect(byId.get("r3")).toBe(zone2);
  });

  it("rooms zonder zone-veld blijven zoneId: undefined", () => {
    const project = makeProject([makeProjectRoom("r1"), makeProjectRoom("r2")]);
    const thermalRooms = [
      makeThermalRoom("r1", "Zone 1"),
      makeThermalRoom("r2"), // geen zone
    ];

    const result = applyEditsToProject(project, thermalRooms, []);

    expect(result.rooms.find((r) => r.id === "r1")?.zoneId).toBeDefined();
    expect(result.rooms.find((r) => r.id === "r2")?.zoneId).toBeUndefined();
  });

  it("zonder zones in de export blijft building.zones afwezig", () => {
    const project = makeProject([makeProjectRoom("r1")]);

    const result = applyEditsToProject(project, [makeThermalRoom("r1")], []);

    expect(result.building.zones).toBeUndefined();
    expect(result.rooms[0]!.zoneId).toBeUndefined();
  });

  it("hergebruikt een bestaande zone op het project (exact match op naam)", () => {
    const project = makeProject([makeProjectRoom("r1")]);
    project.building.zones = [{ id: "zone-bestaand", name: "Zone 1" }];

    const result = applyEditsToProject(
      project,
      [makeThermalRoom("r1", "Zone 1")],
      [],
    );

    expect(result.building.zones).toEqual([
      { id: "zone-bestaand", name: "Zone 1" },
    ]);
    expect(result.rooms[0]!.zoneId).toBe("zone-bestaand");
  });

  it("matcht case-sensitive: 'Zone 1' en 'zone 1' zijn verschillende zones", () => {
    const project = makeProject([makeProjectRoom("r1"), makeProjectRoom("r2")]);
    const thermalRooms = [
      makeThermalRoom("r1", "Zone 1"),
      makeThermalRoom("r2", "zone 1"),
    ];

    const result = applyEditsToProject(project, thermalRooms, []);

    const zones = result.building.zones!;
    expect(zones).toHaveLength(2);
    expect(result.rooms[0]!.zoneId).not.toBe(result.rooms[1]!.zoneId);
  });

  it("pseudo-rooms (niet in project.rooms) maken geen lege zones aan", () => {
    const project = makeProject([makeProjectRoom("r1")]);
    const thermalRooms: ThermalRoom[] = [
      makeThermalRoom("r1"),
      // Outside-pseudoroom met (onrealistische) zone — door de backend niet
      // als project-room gemapt; mag geen orphan-zone achterlaten.
      { id: "outside", name: "Buiten", type: "outside", zone: "Zone X" },
    ];

    const result = applyEditsToProject(project, thermalRooms, []);

    expect(result.building.zones).toBeUndefined();
  });

  it("lege zone-string telt als geen zone", () => {
    const project = makeProject([makeProjectRoom("r1")]);

    const result = applyEditsToProject(
      project,
      [makeThermalRoom("r1", "")],
      [],
    );

    expect(result.building.zones).toBeUndefined();
    expect(result.rooms[0]!.zoneId).toBeUndefined();
  });
});
