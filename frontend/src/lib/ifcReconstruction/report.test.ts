/**
 * Tests voor `report.ts` — de oppervlaktenlijst-export en de vergelijking met
 * de bestaande (pyrevit) warmteverlies-import. Puur/synchroon, geen DOM.
 */
import { describe, expect, it } from "vitest";

import {
  compareWithPyrevit,
  flattenFaces,
  parsePyrevitJson,
  PyrevitParseError,
  resolveFaceColor,
  serializeFacesToCsv,
  spaceCategoryTotals,
  type PyrevitImportFile,
} from "./report";
import type { ReconstructedFace, ReconstructedSpace, ReconstructionResult } from "./types";

function face(overrides: Partial<ReconstructedFace> = {}): ReconstructedFace {
  return {
    zone: "wand",
    geometricOrientation: "wand",
    normal: [0, 0, 1],
    centroidMM: [0, 0, 0],
    grossAreaM2: 5,
    netAreaM2: 5,
    hostCategory: "opaak",
    classification: "exterieur",
    neighbourSpace: null,
    mixedSplit: null,
    hostElement: { id: 1, name: "Gevel-1", ifcType: "IFCWALL" },
    hostSource: "sb",
    qcFlag: "sb-raycast-match",
    qcReason: "SB-boundary en raycast wijzen naar dezelfde host.",
    materialLayers: null,
    measuredThicknessMM: null,
    storey: { id: 1, name: "Begane grond" },
    sbInternalOrExternal: "EXTERNAL",
    sbAreaM2: 5,
    raycastHostElement: null,
    sampleCount: 8,
    voteAgreement: 1,
    ...overrides,
  };
}

function space(overrides: Partial<ReconstructedSpace> = {}): ReconstructedSpace {
  return {
    id: 100,
    name: "0.01 Woonkamer",
    longName: null,
    storey: { id: 1, name: "Begane grond" },
    faces: [face()],
    floorAreaM2: 20,
    footprintEstimateM2: 21,
    zoneTotals: {},
    ...overrides,
  };
}

function result(spaces: ReconstructedSpace[]): ReconstructionResult {
  return {
    meta: {
      ifcSchema: "IFC4",
      generatedAt: "2026-01-01T00:00:00Z",
      maaiveldMM: 0,
      maaiveldSource: "test",
      hostMaxDistMM: 1500,
      totalMaxDistMM: 5000,
      gridCellMM: 500,
    },
    storeys: [{ id: 1, name: "Begane grond" }],
    outdoorPseudoSpaces: [],
    spaces,
  };
}

describe("flattenFaces + CSV export", () => {
  it("produceert één rij per vlak met stabiele rowKey en 1-based vlak-nummering in de CSV", () => {
    const r = result([
      space({
        faces: [
          face({ grossAreaM2: 3.5, netAreaM2: 3.5 }),
          face({ grossAreaM2: 1.25, netAreaM2: 0, hostCategory: "raam", hostElement: { id: 2, name: "Raam-1", ifcType: "IFCWINDOW" } }),
        ],
      }),
    ]);
    const rows = flattenFaces(r);
    expect(rows).toHaveLength(2);
    expect(rows[0]!.rowKey).toBe("0:0");
    expect(rows[1]!.rowKey).toBe("0:1");

    const csv = serializeFacesToCsv(rows);
    const lines = csv.split("\n");
    expect(lines[0]).toBe("ruimte;vlak;oriëntatie;classificatie;categorie;bruto_m2;netto_m2;host;bron;qc");
    expect(lines[1]).toBe("0.01 Woonkamer;1;wand;exterieur;opaak;3.50;3.50;Gevel-1;sb;");
    expect(lines[2]).toBe("0.01 Woonkamer;2;wand;exterieur;raam;1.25;0.00;Raam-1;sb;");
  });

  it("escapet velden met een `;` volgens CSV-quoting", () => {
    const r = result([space({ name: "Kamer; met puntkomma" })]);
    const csv = serializeFacesToCsv(flattenFaces(r));
    expect(csv).toContain('"Kamer; met puntkomma"');
  });

  it("markeert qc-vlag alleen wanneer isQcFlagged waar is (harde vlag of lage stem-overeenstemming)", () => {
    const r = result([
      space({
        faces: [
          face({ qcFlag: "sb-raycast-mismatch" }),
          face({ qcFlag: "sb-raycast-match", sampleCount: 6, voteAgreement: 0.3 }),
          face({ qcFlag: "sb-only-no-raycast-host", sampleCount: 0, voteAgreement: 0 }),
        ],
      }),
    ]);
    const rows = flattenFaces(r);
    expect(rows[0]!.qcFlagged).toBe(true); // hard flag
    expect(rows[1]!.qcFlagged).toBe(true); // low vote agreement with samples
    expect(rows[2]!.qcFlagged).toBe(false); // no raycast samples at all -> not penalised
  });
});

describe("spaceCategoryTotals", () => {
  it("telt bruto m² per hostcategorie op", () => {
    const s = space({
      faces: [
        face({ hostCategory: "opaak", grossAreaM2: 10 }),
        face({ hostCategory: "raam", grossAreaM2: 2 }),
        face({ hostCategory: "deur", grossAreaM2: 1.5 }),
        face({ hostCategory: "opaak", grossAreaM2: 4 }),
      ],
    });
    expect(spaceCategoryTotals(s)).toEqual({ opaakM2: 14, raamM2: 2, deurM2: 1.5 });
  });
});

describe("resolveFaceColor — kleurmapping", () => {
  it("geeft raam/deur hun eigen tint, onafhankelijk van classificatie", () => {
    const raam = resolveFaceColor(face({ hostCategory: "raam", classification: "exterieur" }));
    const deur = resolveFaceColor(face({ hostCategory: "deur", classification: "buurruimte" }));
    expect(raam.fill).not.toBe(deur.fill);
    expect(raam.fillSecondary).toBeUndefined();
  });

  it("geeft gemengd een tweede kleur (fillSecondary) voor een gestreept/tweekleurig vlak", () => {
    const gemengd = resolveFaceColor(face({ hostCategory: "opaak", classification: "gemengd" }));
    expect(gemengd.fillSecondary).toBeDefined();
    expect(gemengd.fill).not.toBe(gemengd.fillSecondary);
  });

  it("zet qcFlagged onafhankelijk van de fill-kleur (zelfde classificatie, andere qc-status)", () => {
    const clean = resolveFaceColor(face({ qcFlag: "sb-raycast-match", sampleCount: 8, voteAgreement: 1 }));
    const flagged = resolveFaceColor(face({ qcFlag: "sb-raycast-mismatch" }));
    expect(clean.fill).toBe(flagged.fill); // same classification -> same base colour
    expect(clean.qcFlagged).toBe(false);
    expect(flagged.qcFlagged).toBe(true);
  });

  it("geeft exterieur en grond verschillende kleuren", () => {
    const ext = resolveFaceColor(face({ classification: "exterieur" }));
    const grond = resolveFaceColor(face({ classification: "grond" }));
    expect(ext.fill).not.toBe(grond.fill);
  });
});

describe("parsePyrevitJson", () => {
  it("accepteert een minimale geldige rooms[]-JSON", () => {
    const parsed = parsePyrevitJson({
      rooms: [{ id: "r1", name: "0.01 Woonkamer", constructions: [{ boundary_type: "exterior", area: 12.3 }] }],
    });
    expect(parsed.rooms).toHaveLength(1);
    expect(parsed.rooms[0]!.constructions[0]!.area).toBe(12.3);
  });

  it("gooit PyrevitParseError wanneer 'rooms' ontbreekt", () => {
    expect(() => parsePyrevitJson({})).toThrow(PyrevitParseError);
  });

  it("gooit PyrevitParseError wanneer een construction geen boundary_type heeft", () => {
    expect(() =>
      parsePyrevitJson({ rooms: [{ id: "r1", name: "x", constructions: [{ area: 1 }] }] }),
    ).toThrow(PyrevitParseError);
  });
});

describe("compareWithPyrevit — matching + Δ%-berekening", () => {
  const pyrevit: PyrevitImportFile = {
    rooms: [
      {
        id: "pr1",
        name: "0.01 Woonkamer",
        constructions: [
          { boundary_type: "exterior", vertical_position: "wall", area: 10 },
          { boundary_type: "ground", vertical_position: "floor", area: 20 },
        ],
      },
      { id: "pr2", name: "Onbekende ruimte", constructions: [{ boundary_type: "exterior", area: 3 }] },
    ],
  };

  it("matcht op genormaliseerde naam en berekent Δ% per (vertical_position, boundary_type)", () => {
    const r = result([
      space({
        name: "0.01 woonkamer", // andere casing/whitespace dan pyrevit -> normalize moet matchen
        faces: [
          face({ zone: "wand", classification: "exterieur", grossAreaM2: 11 }), // pyrevit 10 -> +10%
          face({ zone: "vloer", geometricOrientation: "vloer", classification: "grond", grossAreaM2: 20 }), // exact match
        ],
      }),
    ]);
    const cmp = compareWithPyrevit(r, pyrevit);
    expect(cmp.matched).toHaveLength(1);
    const m = cmp.matched[0]!;
    expect(m.pyrevitRoomId).toBe("pr1");

    const wallExterior = m.cells.find((c) => c.verticalPosition === "wall" && c.boundaryType === "exterior");
    expect(wallExterior).toBeDefined();
    expect(wallExterior!.reconM2).toBe(11);
    expect(wallExterior!.pyrevitM2).toBe(10);
    expect(wallExterior!.deltaPercent).toBe(10);
    expect(wallExterior!.flagged).toBe(false); // exactly at threshold, not > threshold

    const floorGround = m.cells.find((c) => c.verticalPosition === "floor" && c.boundaryType === "ground");
    expect(floorGround!.deltaPercent).toBe(0);
    expect(floorGround!.flagged).toBe(false);

    // "Onbekende ruimte" (pr2) has no recon counterpart -> unmatched pyrevit.
    expect(cmp.unmatchedPyrevit.map((r) => r.id)).toEqual(["pr2"]);
    expect(cmp.unmatchedRecon).toHaveLength(0);
  });

  it("matcht op leidend ruimtenummer wanneer de volledige naam afwijkt", () => {
    const r = result([space({ name: "0.01 - Living room (EN)", faces: [face({ grossAreaM2: 10 })] })]);
    const cmp = compareWithPyrevit(r, pyrevit);
    expect(cmp.matched).toHaveLength(1);
    expect(cmp.matched[0]!.pyrevitRoomId).toBe("pr1");
  });

  it("vlagt een cel met |Δ%| > 10 en zet unmatched recon-ruimtes apart wanneer er geen match is", () => {
    const r = result([
      space({ name: "0.01 Woonkamer", faces: [face({ zone: "wand", classification: "exterieur", grossAreaM2: 15 })] }), // +50%
      space({ id: 999, name: "9.99 Zolder (nieuw)", faces: [face({ grossAreaM2: 4 })] }), // geen pyrevit-match
    ]);
    const cmp = compareWithPyrevit(r, pyrevit);
    const woonkamer = cmp.matched.find((m) => m.pyrevitRoomId === "pr1")!;
    const cell = woonkamer.cells.find((c) => c.verticalPosition === "wall" && c.boundaryType === "exterior")!;
    expect(cell.deltaPercent).toBe(50);
    expect(cell.flagged).toBe(true);

    expect(cmp.unmatchedRecon).toHaveLength(1);
    expect(cmp.unmatchedRecon[0]!.id).toBe(999);
  });

  it("splitst 'gemengd' recon-vlakken via mixedSplit in ground+exterior buckets", () => {
    const r = result([
      space({
        name: "0.01 Woonkamer",
        faces: [
          face({
            zone: "wand",
            classification: "gemengd",
            grossAreaM2: 10,
            mixedSplit: { groundM2: 4, exteriorM2: 6 },
          }),
        ],
      }),
    ]);
    const cmp = compareWithPyrevit(r, pyrevit);
    const m = cmp.matched[0]!;
    const wallGround = m.cells.find((c) => c.verticalPosition === "wall" && c.boundaryType === "ground");
    const wallExterior = m.cells.find((c) => c.verticalPosition === "wall" && c.boundaryType === "exterior");
    expect(wallGround!.reconM2).toBe(4);
    expect(wallExterior!.reconM2).toBe(6);
  });

  it("sluit 'onbepaald' recon-vlakken uit van de cellen maar rapporteert de m² apart", () => {
    const r = result([
      space({
        name: "0.01 Woonkamer",
        faces: [face({ classification: "onbepaald", grossAreaM2: 7 })],
      }),
    ]);
    const cmp = compareWithPyrevit(r, pyrevit);
    const m = cmp.matched[0]!;
    // Geen enkel recon-vlak levert een bucket (alles "onbepaald"), maar de
    // pyrevit-zijde van de match heeft nog steeds haar eigen constructies ->
    // die cellen blijven bestaan met reconM2 = 0, zodat de UI kan tonen dat de
    // bestaande methode wél iets rapporteert waar de reconstructie niets vindt.
    expect(m.cells.every((c) => c.reconM2 === 0)).toBe(true);
    expect(m.cells.length).toBeGreaterThan(0);
    expect(m.reconExcludedOnbepaaldM2).toBe(7);
  });
});
