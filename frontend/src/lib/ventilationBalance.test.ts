import { describe, expect, it } from "vitest";

import type { ModelRoom } from "../components/modeller/types";
import {
  bblDemandDm3s,
  DEFAULT_OCCUPANCY_DM3S_PER_PERSON,
  isBblDemandIndicative,
  type VentilationRoomState,
  type VentilationTerminal,
} from "../types/ventilation";
import {
  aggregateVentilationBalance,
  computeOverflowDistribution,
  computeRoomVentilation,
  deriveOverflowRelations,
  DOOR_GAP_DELTA_P_OFFICE_PA,
  estimateDoorGapAreaCm2,
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
    // Zonder meegegeven overdruk-verdeling: afvoer-totaal = afvoer-eis van
    // de doel-ruimte (geen correctie), één binnenkomende relatie.
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

  it("werkt op het verdeelde debiet (afvoer-eis + overdruk-correctie) i.p.v. de volle afvoer-eis", () => {
    const rooms = twoAdjacentRooms();
    const vrooms = { "0.01": supply, "0.02": exhaust };
    // Overdruk = 25 − 21 = 4 → volledig naar de enige afvoerruimte (keuken):
    // afvoer-totaal = 21 + 4 = 25.
    const dist = computeOverflowDistribution(vrooms, { "0.01": 25, "0.02": 25 });
    const rels = deriveOverflowRelations(rooms, vrooms, dist);
    expect(rels).toHaveLength(1);
    expect(rels[0]!.flowDm3s).toBeCloseTo(25, 6);
  });

  it("verdeelt het afvoer-totaal gelijk over meerdere binnenkomende relaties", () => {
    // Drie ruimtes op een rij: toevoer | afvoer | toevoer — de middelste
    // afvoerruimte grenst aan twee toevoerruimtes (twee doorstroomopeningen).
    const [left, middle] = twoAdjacentRooms() as [ModelRoom, ModelRoom];
    middle.function = "bathroom";
    const right: ModelRoom = {
      ...left,
      id: "0.03",
      name: "Slaapkamer",
      polygon: [
        { x: 10000, y: 0 },
        { x: 15000, y: 0 },
        { x: 15000, y: 5000 },
        { x: 10000, y: 5000 },
      ],
    };
    const vrooms: Record<string, VentilationRoomState> = {
      "0.01": supply, // toevoer 25
      "0.02": exhaust, // afvoer 21
      "0.03": { ...supply }, // toevoer 25
    };
    // Overdruk = 50 − 21 = 29 → afvoer-totaal keuken = 50; twee
    // binnenkomende relaties → 25 per doorstroomopening.
    const dist = computeOverflowDistribution(vrooms, {
      "0.01": 25,
      "0.02": 25,
      "0.03": 25,
    });
    const rels = deriveOverflowRelations([left, middle, right], vrooms, dist);
    expect(rels).toHaveLength(2);
    for (const rel of rels) {
      expect(rel.targetRoomId).toBe("0.02");
      expect(rel.flowDm3s).toBeCloseTo(25, 6);
    }
  });
});

// ---------------------------------------------------------------------------
// Overdruk-verdeling — port van `_bereken_overdruk_verdeling`
// (VentilatieBalans.pushbutton/script.py:632-651): toevoer-overschot naar
// afvoerruimtes, naar rato van oppervlak, round(…, 1).
// ---------------------------------------------------------------------------

describe("computeOverflowDistribution", () => {
  /** Plugin-fixture: woonkamer toevoer 100; badkamer 8 m² (eis 14) en keuken
   *  12 m² (eis 21) als afvoer. Overdruk = 100 − 35 = 65 → naar rato opp:
   *  badkamer 65 × 8/20 = 26,0 en keuken 65 × 12/20 = 39,0. */
  const vrooms: Record<string, VentilationRoomState> = {
    woonkamer: {
      ventilationFunction: "verblijfsruimte",
      requiredSupplyDm3s: 100,
      requiredExhaustDm3s: 0,
      airSourceRoomId: null,
    },
    badkamer: {
      ventilationFunction: "badruimte",
      requiredSupplyDm3s: 0,
      requiredExhaustDm3s: 14,
      airSourceRoomId: null,
    },
    keuken: {
      ventilationFunction: "keuken",
      requiredSupplyDm3s: 0,
      requiredExhaustDm3s: 21,
      airSourceRoomId: null,
    },
  };
  const areas = { woonkamer: 40, badkamer: 8, keuken: 12 };

  it("verdeelt de overdruk naar rato van oppervlak over de afvoerruimtes (plugin-getallen)", () => {
    const dist = computeOverflowDistribution(vrooms, areas);
    expect(dist.surplusDm3s).toBeCloseTo(65, 6);
    expect(dist.exhaustCorrectionDm3s.badkamer).toBeCloseTo(26.0, 6);
    expect(dist.exhaustCorrectionDm3s.keuken).toBeCloseTo(39.0, 6);
    expect(dist.exhaustCorrectionDm3s.woonkamer).toBe(0);
    expect(dist.exhaustTotalDm3s.badkamer).toBeCloseTo(40.0, 6);
    expect(dist.exhaustTotalDm3s.keuken).toBeCloseTo(60.0, 6);
    expect(dist.exhaustTotalDm3s.woonkamer).toBe(0);
  });

  it("rondt de correctie af op 1 decimaal (plugin: round(…, 1))", () => {
    // Overdruk 10 over twee gelijke afvoerruimtes met opp 3 en 6 m²:
    // 10 × 3/9 = 3,333… → 3,3 en 10 × 6/9 = 6,666… → 6,7.
    const v: Record<string, VentilationRoomState> = {
      s: { ...vrooms.woonkamer!, requiredSupplyDm3s: 45 },
      a: { ...vrooms.badkamer! }, // 14
      b: { ...vrooms.keuken! }, // 21
    };
    const dist = computeOverflowDistribution(v, { s: 30, a: 3, b: 6 });
    expect(dist.surplusDm3s).toBeCloseTo(10, 6);
    expect(dist.exhaustCorrectionDm3s.a).toBeCloseTo(3.3, 6);
    expect(dist.exhaustCorrectionDm3s.b).toBeCloseTo(6.7, 6);
  });

  it("geen verdeling bij balans of onderdruk (overdruk ≤ 0)", () => {
    const v: Record<string, VentilationRoomState> = {
      s: { ...vrooms.woonkamer!, requiredSupplyDm3s: 20 },
      a: { ...vrooms.badkamer! }, // 14
      b: { ...vrooms.keuken! }, // 21 → afvoer 35 > toevoer 20
    };
    const dist = computeOverflowDistribution(v, { s: 20, a: 8, b: 12 });
    expect(dist.surplusDm3s).toBe(0);
    expect(dist.exhaustCorrectionDm3s.a).toBe(0);
    expect(dist.exhaustCorrectionDm3s.b).toBe(0);
    // Afvoer-totaal blijft de kale eis.
    expect(dist.exhaustTotalDm3s.a).toBe(14);
    expect(dist.exhaustTotalDm3s.b).toBe(21);
  });

  it("ontbrekende oppervlakken: ruimte zonder opp krijgt correctie 0", () => {
    const dist = computeOverflowDistribution(vrooms, { badkamer: 8 });
    // Alleen badkamer heeft opp → krijgt de volle overdruk.
    expect(dist.exhaustCorrectionDm3s.badkamer).toBeCloseTo(65.0, 6);
    expect(dist.exhaustCorrectionDm3s.keuken).toBe(0);
  });
});

// ---------------------------------------------------------------------------
// Spleet onder de deur — NEN 1087:2001-verankering (Δp-parameter)
// ---------------------------------------------------------------------------

describe("estimateDoorGapAreaCm2 — Δp-criteria NEN 1087 §5.1.3.2.7", () => {
  it("default Δp = 1,0 Pa (woonfunctie-dwarsventilatie)", () => {
    // A = q / (0,6 · √(2·1,0/1,2)) met q = 0,025 m³/s → ≈ 322,7 cm².
    const a = estimateDoorGapAreaCm2(25);
    expect(a).toBeCloseTo(322.7, 0);
  });

  it("kantoor-dwarsventilatie (2 Pa) geeft een √2 kleinere doorlaat", () => {
    const a1 = estimateDoorGapAreaCm2(25);
    const a2 = estimateDoorGapAreaCm2(25, DOOR_GAP_DELTA_P_OFFICE_PA);
    expect(a2).toBeCloseTo(a1 / Math.SQRT2, 6);
  });

  it("debiet ≤ 0 → 0 cm²", () => {
    expect(estimateDoorGapAreaCm2(0)).toBe(0);
    expect(estimateDoorGapAreaCm2(-5)).toBe(0);
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

// ---------------------------------------------------------------------------
// Persoon-gebaseerde utiliteitsfuncties — Bbl artikel 4.122 lid 2
// (eis = personen × personDm3s; NIET de vlakke 4,0 dm³/s pp-toeslag)
// ---------------------------------------------------------------------------

describe("bblDemandDm3s — per-persoon utiliteit (Bbl 4.122 lid 2)", () => {
  it("onderwijsfunctie: 30 personen → 30 × 8,5 = 255 dm³/s (niet 30 × 4 = 120)", () => {
    expect(bblDemandDm3s(60, "onderwijsfunctie", 30)).toBeCloseTo(255, 6);
  });

  it("bijeenkomstfunctie (kinderopvang): 6,5 dm³/s p.p.", () => {
    expect(
      bblDemandDm3s(40, "bijeenkomstfunctie (kinderopvang)", 10),
    ).toBeCloseTo(65, 6);
  });

  it("bijeenkomstfunctie (niet-kinderopvang) en winkel: 4 dm³/s p.p.", () => {
    expect(bblDemandDm3s(40, "bijeenkomstfunctie", 10)).toBeCloseTo(40, 6);
    expect(bblDemandDm3s(40, "winkelfunctie", 10)).toBeCloseTo(40, 6);
  });

  it("kantoor/industrie/sport: 6,5 dm³/s p.p.", () => {
    expect(bblDemandDm3s(40, "kantoorfunctie", 8)).toBeCloseTo(52, 6);
    expect(bblDemandDm3s(40, "industriefunctie", 8)).toBeCloseTo(52, 6);
    expect(bblDemandDm3s(40, "sportfunctie", 8)).toBeCloseTo(52, 6);
  });

  it("gezondheidszorg: bedgebied 12, overig 8,5 dm³/s p.p.", () => {
    expect(
      bblDemandDm3s(30, "gezondheidszorgfunctie (bedgebied)", 4),
    ).toBeCloseTo(48, 6);
    expect(bblDemandDm3s(30, "gezondheidszorgfunctie", 4)).toBeCloseTo(34, 6);
  });

  it("minimum blijft de ondergrens bij een kleine bezetting", () => {
    // 1 persoon winkel → 4 dm³/s < minimum 7 → 7.
    expect(bblDemandDm3s(2, "winkelfunctie", 1)).toBe(7);
  });

  it("zonder bezetting: indicatieve m²-fallback (max(opp × 0,9, minimum))", () => {
    expect(bblDemandDm3s(60, "onderwijsfunctie")).toBeCloseTo(54, 6);
    expect(bblDemandDm3s(60, "onderwijsfunctie", 0)).toBeCloseTo(54, 6);
    // Kleine ruimte → ondergrens 7.
    expect(bblDemandDm3s(5, "kantoorfunctie")).toBe(7);
  });

  it("woonfunctie blijft ongewijzigd: 4,0-toeslag, nooit indicatief", () => {
    // 20 m² woonfunctie → max(20 × 0,7; 5 × 4,0; 7) = 20 — identiek aan
    // het oude gedrag (de vlakke pp-toeslag geldt alléén woonfunctie-achtig).
    expect(bblDemandDm3s(20, "woonfunctie", 5)).toBeCloseTo(20, 6);
    expect(bblDemandDm3s(20, "woonfunctie")).toBeCloseTo(14, 6);
  });
});

describe("isBblDemandIndicative", () => {
  it("persoon-gebaseerde functie zonder bezetting → indicatief", () => {
    expect(isBblDemandIndicative("onderwijsfunctie")).toBe(true);
    expect(isBblDemandIndicative("onderwijsfunctie", 0)).toBe(true);
    expect(isBblDemandIndicative("kantoorfunctie", undefined)).toBe(true);
  });

  it("met bezetting → niet indicatief", () => {
    expect(isBblDemandIndicative("onderwijsfunctie", 30)).toBe(false);
  });

  it("oppervlakte-gebaseerde functies zijn nooit indicatief", () => {
    expect(isBblDemandIndicative("woonfunctie")).toBe(false);
    expect(isBblDemandIndicative("verblijfsruimte")).toBe(false);
    expect(isBblDemandIndicative("keuken")).toBe(false);
    expect(isBblDemandIndicative("badruimte", undefined)).toBe(false);
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
