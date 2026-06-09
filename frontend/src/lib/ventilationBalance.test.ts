import { describe, expect, it } from "vitest";

import type { ModelRoom } from "../components/modeller/types";
import {
  bblDemandDm3s,
  DEFAULT_OCCUPANCY_DM3S_PER_PERSON,
  type VentilationRoomState,
  type VentilationTerminal,
} from "../types/ventilation";
import {
  aggregateVentilationBalance,
  computeRoomVentilation,
  deriveOverflowRelations,
} from "./ventilationBalance";

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

// ---------------------------------------------------------------------------
// Personen-toeslag — max(opp × dm³/m², pers × pp-debiet, minimum)
// (port van `_bereken_ventilatie_eis`, VentilatieBalans.pushbutton/script.py)
// ---------------------------------------------------------------------------

describe("bblDemandDm3s — personen-toeslag", () => {
  it("zonder bezetting geldt max(opp-term, minimum) (ondergrens wint bij klein opp)", () => {
    // verblijfsruimte: 0,7 dm³/(s·m²), minimum 7 → 5 m² geeft 3,5 → ondergrens 7.
    expect(bblDemandDm3s(5, "verblijfsruimte")).toBe(7);
    // 20 m² → 14 > minimum.
    expect(bblDemandDm3s(20, "verblijfsruimte")).toBeCloseTo(14, 6);
  });

  it("personen-term wint wanneer pers × pp > opp-term", () => {
    // 20 m² verblijfsruimte → opp-term 14; 5 personen × 4,0 = 20 → eis 20.
    expect(bblDemandDm3s(20, "verblijfsruimte", 5)).toBeCloseTo(
      5 * DEFAULT_OCCUPANCY_DM3S_PER_PERSON,
      6,
    );
  });

  it("opp-term wint wanneer die groter is dan de personen-term", () => {
    // 50 m² verblijfsruimte → 35; 2 personen × 4,0 = 8 → eis 35.
    expect(bblDemandDm3s(50, "verblijfsruimte", 2)).toBeCloseTo(35, 6);
  });

  it("ondergrens blijft van kracht boven een kleine personen-term", () => {
    // keuken: opp-term 0, minimum 21; 1 persoon × 4 = 4 → eis 21.
    expect(bblDemandDm3s(8, "keuken", 1)).toBe(21);
    // 6 personen × 4 = 24 > 21 → eis 24.
    expect(bblDemandDm3s(8, "keuken", 6)).toBeCloseTo(24, 6);
  });

  it("bezetting 0 of undefined geeft geen toeslag (plugin: alleen personen > 0)", () => {
    expect(bblDemandDm3s(20, "verblijfsruimte", 0)).toBeCloseTo(14, 6);
    expect(bblDemandDm3s(20, "verblijfsruimte", undefined)).toBeCloseTo(14, 6);
  });
});

describe("computeRoomVentilation — occupancy", () => {
  it("verhoogt de eis via de sidecar-occupancy en bewaart het veld", () => {
    const existing: VentilationRoomState = {
      ventilationFunction: "verblijfsruimte",
      requiredSupplyDm3s: 0,
      requiredExhaustDm3s: 0,
      airSourceRoomId: null,
      occupancy: 8,
    };
    const vr = computeRoomVentilation(20, "living_room", existing);
    // 8 × 4,0 = 32 > opp-term 14.
    expect(vr.requiredSupplyDm3s).toBeCloseTo(32, 6);
    expect(vr.occupancy).toBe(8);
  });

  it("laat occupancy weg wanneer niet gezet (geen toeslag, geen veld)", () => {
    const vr = computeRoomVentilation(20, "living_room");
    expect(vr.requiredSupplyDm3s).toBeCloseTo(14, 6);
    expect("occupancy" in vr).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Gebouwbalans-aggregatie
// ---------------------------------------------------------------------------

describe("aggregateVentilationBalance", () => {
  const vrooms: Record<string, VentilationRoomState> = {
    "0.01": supply, // toevoer-eis 25
    "0.02": exhaust, // afvoer-eis 21
  };

  function terminal(
    partial: Partial<VentilationTerminal> & Pick<VentilationTerminal, "id" | "roomId" | "type">,
  ): VentilationTerminal {
    return { source: "manual", ...partial };
  }

  it("sommeert eisen + aanwezig per ruimte en gebouwbreed (systeem D)", () => {
    const terminals: VentilationTerminal[] = [
      terminal({ id: "t1", roomId: "0.01", type: "supply", flowDm3s: 15 }),
      terminal({ id: "t2", roomId: "0.01", type: "supply", flowDm3s: 10 }),
      terminal({ id: "t3", roomId: "0.02", type: "exhaust", flowDm3s: 14 }),
    ];
    const b = aggregateVentilationBalance(vrooms, terminals, "D");

    expect(b.totalRequiredSupplyDm3s).toBe(25);
    expect(b.totalRequiredExhaustDm3s).toBe(21);
    expect(b.totalPresentSupplyDm3s).toBe(25);
    expect(b.totalPresentExhaustDm3s).toBe(14);
    expect(b.rooms["0.01"]!.supplyDeficitDm3s).toBe(0);
    expect(b.rooms["0.02"]!.exhaustDeficitDm3s).toBeCloseTo(7, 6);
    // Eis-onbalans 25 − 21 = 4 > tolerantie 1 → niet in balans.
    expect(b.balanced).toBe(false);
    expect(b.imbalanceDm3s).toBeCloseTo(4, 6);
  });

  it("telt terminals zonder flowDm3s als 0 maar markeert ze", () => {
    const terminals: VentilationTerminal[] = [
      terminal({ id: "t1", roomId: "0.01", type: "supply" }), // geen debiet
      terminal({ id: "t2", roomId: "0.01", type: "supply", flowDm3s: 10 }),
    ];
    const b = aggregateVentilationBalance(vrooms, terminals, "D");

    expect(b.rooms["0.01"]!.presentSupplyDm3s).toBe(10);
    expect(b.rooms["0.01"]!.missingFlowCount).toBe(1);
    expect(b.rooms["0.02"]!.missingFlowCount).toBe(0);
  });

  it("rapporteert geen toevoer-tekort bij natuurlijke toevoer (systeem C, default)", () => {
    // Geen terminals: bij systeem D zou er een toevoer-tekort van 25 zijn.
    const bD = aggregateVentilationBalance(vrooms, [], "D");
    expect(bD.rooms["0.01"]!.supplyDeficitDm3s).toBe(25);

    // Systeem C (natuurlijke toevoer via gevelroosters) → toevoer-tekort 0,
    // afvoer (mechanisch) houdt zijn tekort.
    const bC = aggregateVentilationBalance(vrooms, [], "C");
    expect(bC.rooms["0.01"]!.supplyDeficitDm3s).toBe(0);
    expect(bC.rooms["0.02"]!.exhaustDeficitDm3s).toBe(21);

    // Default (undefined) = systeem C.
    const bDefault = aggregateVentilationBalance(vrooms, [], undefined);
    expect(bDefault.system.key).toBe("C");
    expect(bDefault.rooms["0.01"]!.supplyDeficitDm3s).toBe(0);
  });

  it("is in balans wanneer |toevoer-eis − afvoer-eis| < 1 dm³/s", () => {
    const nearBalanced: Record<string, VentilationRoomState> = {
      a: { ...supply, requiredSupplyDm3s: 21.5 },
      b: exhaust, // 21
    };
    const b = aggregateVentilationBalance(nearBalanced, [], "D");
    expect(b.balanced).toBe(true);
    expect(b.imbalanceDm3s).toBeCloseTo(0.5, 6);
  });

  it("negeert terminals van onbekende/verwijderde ruimtes", () => {
    const terminals: VentilationTerminal[] = [
      terminal({ id: "t1", roomId: "weg", type: "supply", flowDm3s: 99 }),
    ];
    const b = aggregateVentilationBalance(vrooms, terminals, "D");
    expect(b.totalPresentSupplyDm3s).toBe(0);
  });
});
