import { describe, expect, it } from "vitest";

import type { ModelRoom } from "../components/modeller/types";
import type { VentilationRoomState } from "../types/ventilation";
import { deriveOverflowRelations } from "./ventilationBalance";

/** Twee 5×5 m ruimtes die een verticale wand op x = 5000 delen. */
function twoAdjacentRooms(): ModelRoom[] {
  const left: ModelRoom = {
    id: "0.01",
    name: "Woonkamer",
    function: "living_room",
    floor: 0,
    height: 2600,
    polygon: [
      { x: 0, y: 0 },
      { x: 5000, y: 0 },
      { x: 5000, y: 5000 },
      { x: 0, y: 5000 },
    ],
  };
  const right: ModelRoom = {
    id: "0.02",
    name: "Keuken",
    function: "kitchen",
    floor: 0,
    height: 2600,
    polygon: [
      { x: 5000, y: 0 },
      { x: 10000, y: 0 },
      { x: 10000, y: 5000 },
      { x: 5000, y: 5000 },
    ],
  };
  return [left, right];
}

const supply: VentilationRoomState = {
  ventilationFunction: "verblijfsruimte",
  requiredSupplyDm3s: 25,
  requiredExhaustDm3s: 0,
  airSourceRoomId: null,
};
const exhaust: VentilationRoomState = {
  ventilationFunction: "keuken",
  requiredSupplyDm3s: 0,
  requiredExhaustDm3s: 21,
  airSourceRoomId: null,
};

describe("deriveOverflowRelations", () => {
  it("leidt één overstroom-relatie af van toevoer- naar afvoer-ruimte over de gedeelde wand", () => {
    const rooms = twoAdjacentRooms();
    const rels = deriveOverflowRelations(rooms, { "0.01": supply, "0.02": exhaust });

    expect(rels).toHaveLength(1);
    const rel = rels[0]!;
    expect(rel.sourceRoomId).toBe("0.01"); // toevoer (droog)
    expect(rel.targetRoomId).toBe("0.02"); // afvoer (nat)
    // Gedeelde wand zit op x = 5000, midden op y = 2500.
    expect(rel.mid.x).toBeCloseTo(5000, 0);
    expect(rel.mid.y).toBeCloseTo(2500, 0);
    // Normaal wijst bron → doel (naar rechts, +x).
    expect(rel.nx).toBeCloseTo(1, 2);
    expect(rel.ny).toBeCloseTo(0, 2);
    // Overlap = volledige 5 m wand.
    expect(rel.overlapMm).toBeCloseTo(5000, 0);
    // Debiet = afvoer-eis van de doel-ruimte.
    expect(rel.flowDm3s).toBe(21);
  });

  it("respecteert airSourceRoomId boven de type-heuristiek (richting omgekeerd)", () => {
    const rooms = twoAdjacentRooms();
    // De afvoer-ruimte betrekt haar lucht uit de toevoer-ruimte → 0.01 → 0.02.
    // Zet airSourceRoomId op de TOEVOER-ruimte naar de afvoer-ruimte, wat de
    // type-heuristiek zou tegenspreken, om de prioriteit te bewijzen.
    const a: VentilationRoomState = { ...supply, airSourceRoomId: "0.02" };
    const rels = deriveOverflowRelations(rooms, { "0.01": a, "0.02": exhaust });

    expect(rels).toHaveLength(1);
    // air_source op 0.01 = "0.02" → lucht stroomt 0.02 → 0.01.
    expect(rels[0]!.sourceRoomId).toBe("0.02");
    expect(rels[0]!.targetRoomId).toBe("0.01");
  });

  it("geeft geen relatie wanneer beide ruimtes hetzelfde type hebben", () => {
    const rooms = twoAdjacentRooms();
    const rels = deriveOverflowRelations(rooms, { "0.01": supply, "0.02": supply });
    expect(rels).toHaveLength(0);
  });

  it("dedupliceert op ruimte-paar (max één relatie per paar)", () => {
    const rooms = twoAdjacentRooms();
    const rels = deriveOverflowRelations(rooms, { "0.01": supply, "0.02": exhaust });
    const keys = new Set(rels.map((r) => r.key));
    expect(keys.size).toBe(rels.length);
  });

  it("geeft geen relatie voor niet-aangrenzende ruimtes (geen gedeelde wand)", () => {
    const rooms = twoAdjacentRooms();
    // Verschuif de rechter ruimte zodat er een gat zit (geen gedeelde wand).
    rooms[1]!.polygon = rooms[1]!.polygon.map((p) => ({ x: p.x + 2000, y: p.y }));
    const rels = deriveOverflowRelations(rooms, { "0.01": supply, "0.02": exhaust });
    expect(rels).toHaveLength(0);
  });
});
