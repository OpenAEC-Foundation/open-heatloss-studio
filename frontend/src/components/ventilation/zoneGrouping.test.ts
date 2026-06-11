import { describe, expect, it } from "vitest";

import type { Zone } from "../../types";
import type { RoomVentilationBalance } from "../../lib/ventilationBalance";
import { groupRoomsByZone, sumZoneBalance } from "./zoneGrouping";

const ZONES: Zone[] = [
  { id: "zone-a", name: "Begane grond" },
  { id: "zone-b", name: "Verdieping" },
];

interface TestRoom {
  id: string;
  zoneId?: string;
}

describe("groupRoomsByZone", () => {
  it("groepeert rooms per zone in de zone-volgorde, restgroep achteraan", () => {
    const rooms: TestRoom[] = [
      { id: "r1", zoneId: "zone-b" },
      { id: "r2", zoneId: "zone-a" },
      { id: "r3" },
      { id: "r4", zoneId: "zone-a" },
    ];

    const groups = groupRoomsByZone(rooms, ZONES);

    expect(groups).toHaveLength(3);
    expect(groups[0]!.zone?.id).toBe("zone-a");
    expect(groups[0]!.rooms.map((r) => r.id)).toEqual(["r2", "r4"]);
    expect(groups[1]!.zone?.id).toBe("zone-b");
    expect(groups[1]!.rooms.map((r) => r.id)).toEqual(["r1"]);
    // Restgroep "Niet ingedeeld" altijd als laatste.
    expect(groups[2]!.zone).toBeUndefined();
    expect(groups[2]!.rooms.map((r) => r.id)).toEqual(["r3"]);
  });

  it("laat lege zones weg (geen lege kopjes)", () => {
    const rooms: TestRoom[] = [{ id: "r1", zoneId: "zone-b" }];

    const groups = groupRoomsByZone(rooms, ZONES);

    expect(groups).toHaveLength(1);
    expect(groups[0]!.zone?.id).toBe("zone-b");
  });

  it("zet rooms met een dangling zoneId in de restgroep", () => {
    const rooms: TestRoom[] = [
      { id: "r1", zoneId: "zone-verwijderd" },
      { id: "r2", zoneId: "zone-a" },
    ];

    const groups = groupRoomsByZone(rooms, ZONES);

    expect(groups).toHaveLength(2);
    expect(groups[0]!.zone?.id).toBe("zone-a");
    expect(groups[1]!.zone).toBeUndefined();
    expect(groups[1]!.rooms.map((r) => r.id)).toEqual(["r1"]);
  });

  it("geeft geen groepen terug zonder rooms", () => {
    expect(groupRoomsByZone([], ZONES)).toEqual([]);
  });

  it("zonder zones landt alles in de restgroep", () => {
    const rooms: TestRoom[] = [{ id: "r1", zoneId: "zone-a" }, { id: "r2" }];

    const groups = groupRoomsByZone(rooms, []);

    expect(groups).toHaveLength(1);
    expect(groups[0]!.zone).toBeUndefined();
    expect(groups[0]!.rooms.map((r) => r.id)).toEqual(["r1", "r2"]);
  });
});

describe("sumZoneBalance", () => {
  const balanceRow = (
    id: string,
    partial: Partial<RoomVentilationBalance>,
  ): RoomVentilationBalance => ({
    roomId: id,
    requiredSupplyDm3s: 0,
    requiredExhaustDm3s: 0,
    presentSupplyDm3s: 0,
    presentExhaustDm3s: 0,
    missingFlowCount: 0,
    supplyDeficitDm3s: 0,
    exhaustDeficitDm3s: 0,
    ...partial,
  });

  it("sommeert eis + aanwezig per richting over de room-ids", () => {
    const rows = {
      r1: balanceRow("r1", {
        requiredSupplyDm3s: 25,
        presentSupplyDm3s: 20,
      }),
      r2: balanceRow("r2", {
        requiredExhaustDm3s: 21,
        presentExhaustDm3s: 14,
      }),
      r3: balanceRow("r3", {
        requiredSupplyDm3s: 7,
        presentSupplyDm3s: 7,
      }),
    };

    const sub = sumZoneBalance(["r1", "r2"], rows);

    expect(sub.requiredSupplyDm3s).toBe(25);
    expect(sub.requiredExhaustDm3s).toBe(21);
    expect(sub.presentSupplyDm3s).toBe(20);
    expect(sub.presentExhaustDm3s).toBe(14);
  });

  it("negeert ids zonder balans-regel (tellen als 0)", () => {
    const rows = {
      r1: balanceRow("r1", { requiredSupplyDm3s: 10 }),
    };

    const sub = sumZoneBalance(["r1", "onbekend"], rows);

    expect(sub.requiredSupplyDm3s).toBe(10);
    expect(sub.requiredExhaustDm3s).toBe(0);
  });
});
