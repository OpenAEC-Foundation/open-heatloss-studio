import { describe, expect, it } from "vitest";

import type { ProjectInfo } from "../types/project";
import {
  VENTILATION_SYSTEMS,
  type VentilationRoomState,
  type VentilationSystemKey,
  type VentilationTerminal,
} from "../types/ventilation";
import {
  buildVentilationReportData,
  type VentilationReportInput,
  type VentilationReportRoom,
} from "./ventilationReportBuilder";

// ---------------------------------------------------------------------------
// Helpers — smal getypeerde toegang tot de report-JSON
// ---------------------------------------------------------------------------

interface ReportSection {
  title: string;
  level: number;
  content: Array<Record<string, unknown>>;
}

interface ReportTable {
  type: string;
  title?: string;
  headers: string[];
  rows: string[][];
  column_widths?: number[];
}

function sectionsOf(data: Record<string, unknown>): ReportSection[] {
  return data.sections as ReportSection[];
}

function tablesOf(section: ReportSection): ReportTable[] {
  return section.content.filter(
    (b) => b.type === "table",
  ) as unknown as ReportTable[];
}

function sectionByTitle(
  data: Record<string, unknown>,
  title: string,
): ReportSection {
  const section = sectionsOf(data).find((s) => s.title === title);
  expect(section, `sectie "${title}" ontbreekt`).toBeDefined();
  return section!;
}

// ---------------------------------------------------------------------------
// Fixture — woonkamer (toevoer + personen-toeslag), keuken (tekort),
// badkamer (ventiel zonder debiet), berging (geen eis)
// ---------------------------------------------------------------------------

const info: ProjectInfo = {
  name: "Testproject Hoogbouw",
  project_number: "2026-042",
  address: "Teststraat 1, Delft",
  client: "Opdrachtgever BV",
  date: "2026-06-10",
};

const rooms: VentilationReportRoom[] = [
  { id: "r1", name: "Woonkamer", floor_area: 20 },
  { id: "r2", name: "Keuken", floor_area: 8 },
  { id: "r3", name: "Badkamer", floor_area: 6 },
  { id: "r4", name: "Berging", floor_area: 4 },
];

/**
 * Per-room states zoals `deriveVentilationDemand` ze zou opleveren:
 * - Woonkamer: verblijfsruimte, 5 personen → eis 5 × 4,0 = 20 dm³/s
 *   (personen-term wint van opp-term 20 × 0,7 = 14).
 * - Keuken: afvoer-minimum 21 dm³/s.
 * - Badkamer: afvoer-minimum 14 dm³/s.
 * - Berging: geen eis.
 */
const ventilationRooms: Record<string, VentilationRoomState> = {
  r1: {
    ventilationFunction: "verblijfsruimte",
    requiredSupplyDm3s: 20,
    requiredExhaustDm3s: 0,
    airSourceRoomId: null,
    occupancy: 5,
  },
  r2: {
    ventilationFunction: "keuken",
    requiredSupplyDm3s: 0,
    requiredExhaustDm3s: 21,
    airSourceRoomId: null,
  },
  r3: {
    ventilationFunction: "badruimte",
    requiredSupplyDm3s: 0,
    requiredExhaustDm3s: 14,
    airSourceRoomId: null,
  },
  r4: {
    ventilationFunction: "bergruimte",
    requiredSupplyDm3s: 0,
    requiredExhaustDm3s: 0,
    airSourceRoomId: null,
  },
};

const terminals: VentilationTerminal[] = [
  // Woonkamer: toevoer volledig gedekt → ✔ voldoet (systeem D).
  { id: "t1", roomId: "r1", type: "supply", source: "manual", flowDm3s: 20 },
  // Keuken: 14 van 21 aanwezig → tekort 7.
  { id: "t2", roomId: "r2", type: "exhaust", source: "manual", flowDm3s: 14 },
  // Badkamer: ventiel zonder debiet → telt als 0, gemarkeerd.
  { id: "t3", roomId: "r3", type: "exhaust", source: "manual" },
];

function buildFixture(
  system: VentilationSystemKey = "D",
): Record<string, unknown> {
  const input: VentilationReportInput = {
    info,
    rooms,
    ventilationRooms,
    terminals,
    system,
  };
  return buildVentilationReportData(input);
}

// ---------------------------------------------------------------------------
// (a) Structuur — verplichte velden + envelope conform uw/rc-patroon
// ---------------------------------------------------------------------------

describe("buildVentilationReportData — structuur", () => {
  it("bevat de verplichte velden template + project en de uw/rc-envelope", () => {
    const data = buildFixture();

    expect(data.template).toBe("standaard_rapport");
    expect(data.project).toBe("Testproject Hoogbouw");
    expect(data.format).toBe("A4");
    expect(data.date).toBe("2026-06-10");
    expect(data.cover).toBeDefined();
    expect(data.colofon).toMatchObject({ enabled: true });
    expect(data.toc).toMatchObject({ enabled: true });
    expect(data.backcover).toMatchObject({ enabled: true });

    const sections = sectionsOf(data);
    expect(sections.map((s) => s.title)).toEqual([
      "Uitgangspunten",
      "Balans per vertrek",
      "Gebouwbalans",
    ]);
  });

  it("neemt de projectgegevens op in de uitgangspunten", () => {
    const data = buildFixture();
    const tables = tablesOf(sectionByTitle(data, "Uitgangspunten"));
    const projectTable = tables.find((t) => t.title === "Projectgegevens")!;
    expect(projectTable).toBeDefined();

    const flat = projectTable.rows.flat();
    expect(flat).toContain("Testproject Hoogbouw");
    expect(flat).toContain("2026-042");
    expect(flat).toContain("Teststraat 1, Delft");
    expect(flat).toContain("Opdrachtgever BV");
  });

  it("valt terug op een default-titel bij een leeg projectnaam-veld", () => {
    const data = buildVentilationReportData({
      info: { name: "" },
      rooms: [],
      ventilationRooms: {},
      terminals: [],
    });
    expect(data.project).toBe("Ventilatiebalans");
  });
});

// ---------------------------------------------------------------------------
// (b) Per-vertrek rij-inhoud — tekort + personen-toeslag + ontbrekend debiet
// ---------------------------------------------------------------------------

describe("buildVentilationReportData — per-vertrek tabel", () => {
  function roomTable(data: Record<string, unknown>): ReportTable {
    const tables = tablesOf(sectionByTitle(data, "Balans per vertrek"));
    expect(tables).toHaveLength(1);
    return tables[0]!;
  }

  it("zet kolomkoppen + proportionele kolombreedtes op de brede tabel", () => {
    const table = roomTable(buildFixture());
    expect(table.headers).toEqual([
      "Vertrek",
      "Gebruiksfunctie (BBL)",
      "Opp.",
      "Pers.",
      "Type",
      "Eis",
      "Aanwezig",
      "Status",
    ]);
    expect(table.column_widths).toHaveLength(table.headers.length);
    expect(table.rows).toHaveLength(4);
  });

  it("woonkamer: personen-toeslag (5 pers → 20 dm³/s) en ✔ voldoet", () => {
    const row = roomTable(buildFixture()).rows[0]!;
    expect(row[0]).toBe("Woonkamer");
    expect(row[1]).toBe("verblijfsruimte");
    expect(row[2]).toBe("20.00 m²");
    expect(row[3]).toBe("5"); // personen
    expect(row[4]).toBe("toevoer");
    expect(row[5]).toBe("20 dm³/s (72 m³/h)"); // 5 × 4,0; m³/h = × 3,6
    expect(row[6]).toBe("20 dm³/s (72 m³/h)");
    expect(row[7]).toBe("✔ voldoet");
  });

  it("keuken: tekort (21 eis − 14 aanwezig = 7 dm³/s)", () => {
    const row = roomTable(buildFixture()).rows[1]!;
    expect(row[0]).toBe("Keuken");
    expect(row[3]).toBe("—"); // geen bezetting opgegeven
    expect(row[4]).toBe("afvoer");
    expect(row[5]).toBe("21 dm³/s (76 m³/h)");
    expect(row[6]).toBe("14 dm³/s (50 m³/h)");
    expect(row[7]).toBe("tekort 7 dm³/s");
  });

  it("badkamer: ventiel zonder debiet herkenbaar gemarkeerd in Aanwezig", () => {
    const row = roomTable(buildFixture()).rows[2]!;
    expect(row[0]).toBe("Badkamer");
    expect(row[6]).toContain("0 dm³/s");
    expect(row[6]).toContain("1 ventiel zonder debiet");
    expect(row[7]).toBe("tekort 14 dm³/s");
  });

  it("berging: geen eis → em-dashes + status 'geen eis'", () => {
    const row = roomTable(buildFixture()).rows[3]!;
    expect(row[0]).toBe("Berging");
    expect(row[4]).toBe("geen");
    expect(row[5]).toBe("—");
    expect(row[6]).toBe("—");
    expect(row[7]).toBe("geen eis");
  });

  it("natuurlijke kant (systeem C): toevoer via gevelroosters, status natuurlijk", () => {
    const row = roomTable(buildFixture("C")).rows[0]!;
    expect(row[6]).toBe("via gevelroosters");
    expect(row[7]).toBe("natuurlijk");
  });
});

// ---------------------------------------------------------------------------
// (c) Systeemlabel-mapping A–D + gebouwbalans
// ---------------------------------------------------------------------------

describe("buildVentilationReportData — systeem & gebouwbalans", () => {
  it.each(["A", "B", "C", "D"] as VentilationSystemKey[])(
    "neemt het leesbare NL-label van systeem %s op in de uitgangspunten",
    (key) => {
      const data = buildFixture(key);
      const tables = tablesOf(sectionByTitle(data, "Uitgangspunten"));
      const basisTable = tables.find(
        (t) => t.title === "Ventilatiesysteem & norm-grondslag",
      )!;
      expect(basisTable).toBeDefined();
      const flat = basisTable.rows.flat();
      expect(flat).toContain(VENTILATION_SYSTEMS[key].label);
    },
  );

  it("default (system undefined) valt terug op systeem C", () => {
    const data = buildVentilationReportData({
      info,
      rooms,
      ventilationRooms,
      terminals,
    });
    const tables = tablesOf(sectionByTitle(data, "Uitgangspunten"));
    const flat = tables
      .find((t) => t.title === "Ventilatiesysteem & norm-grondslag")!
      .rows.flat();
    expect(flat).toContain(VENTILATION_SYSTEMS.C.label);
  });

  it("gebouwbalans: totalen + onbalans-oordeel zoals het zijpaneel", () => {
    const data = buildFixture();
    const section = sectionByTitle(data, "Gebouwbalans");
    const totals = tablesOf(section)[0]!;
    const flat = totals.rows.flat();
    // Eis-totalen: toevoer 20, afvoer 21 + 14 = 35.
    expect(flat).toContain("20 dm³/s (72 m³/h)");
    expect(flat).toContain("35 dm³/s (126 m³/h)");

    // Onbalans 20 − 35 = −15 → onderdruk, niet in balans.
    const calc = section.content.find((b) => b.type === "calculation")!;
    expect(calc.result).toBe("-15");
    expect(calc.unit).toBe("dm³/s");

    const verdict = section.content.find((b) => b.type === "paragraph")!;
    expect(String(verdict.text)).toContain("Onderdruk");
  });
});
