/**
 * Pure vergelijklogica voor de Help-sectie "Verificatie".
 *
 * Vergelijkt de Vabi-referentiewaarden (truth uit
 * `tests/verification/<project>/expected.json`) met een live resultaat van
 * de actuele rekenkern. Twee smaken:
 *
 * - **ISSO 51** (`compareResults`): per vertrek Φ_HL + gebouwtotaal, met de
 *   toleranties van de Rust-integratietests
 *   (`crates/isso51-core/tests/integration_test.rs`):
 *   PASS bij `|Δ| ≤ max(2 W absoluut; 2 % van verwacht)`.
 * - **ISSO 53** (`compareIsso53Results`): per (vertrek, grootheid)-metric
 *   met een **eigen relatieve tolerantie per metric** — 1-op-1 spiegel van
 *   de `close()`-asserts in de Rust golden-tests
 *   (`crates/isso53-core/tests/vabi_*_golden.rs`). De ISSO 53
 *   verificatiebestanden hebben heterogene expected-shapes én heterogene
 *   toleranties (fixture-bundeling, Vabi-anomalies), dus geen gedeelde
 *   ±2W/±2%-grens zoals bij ISSO 51.
 *
 * Bewust zonder React/store/fetch — node-env unit-testbaar
 * (`verificationCompare.test.ts`).
 */
import type { Isso53ProjectResult, Isso53RoomResult } from "../types/isso53Result";
import type { ProjectResult } from "../types/result";

/** Absolute tolerantie in W — spiegel van `ABS_TOLERANCE_W` in de Rust-test. */
export const ABS_TOLERANCE_W = 2.0;

/** Relatieve tolerantie (fractie van verwacht) — spiegel van `REL_TOLERANCE`. */
export const REL_TOLERANCE = 0.02;

/** Eén vertrek uit `expected.json` (alleen de velden die wij vergelijken). */
export interface ExpectedRoom {
  room_id: string;
  room_name: string;
  theta_i: number;
  /** Φ_HL,i — totaal warmteverlies van het vertrek volgens Vabi (W). */
  phi_hl_i: number;
}

/** Gebouwtotaal uit `expected.json`. */
export interface ExpectedBuilding {
  /** Φ_HL,build — aansluitvermogen volgens Vabi (W, kwadratische sommatie). */
  phi_hl_build: number;
}

/** Minimale shape van een `expected.json` verificatiebestand. */
export interface VerificationExpected {
  rooms: ReadonlyArray<ExpectedRoom>;
  building: ExpectedBuilding;
}

/** Eén rij in de vergelijkingstabel (vertrek óf gebouwtotaal). */
export interface ComparisonRow {
  roomId: string;
  roomName: string;
  /** Ontwerpbinnentemperatuur (°C); null voor de gebouwtotaal-rij. */
  thetaI: number | null;
  /** Φ_HL verwacht volgens Vabi (W). */
  expectedW: number;
  /** Φ_HL berekend door de actuele engine (W); null = nog niet berekend of vertrek niet gevonden. */
  actualW: number | null;
  /** actual − expected (W); null wanneer actualW null is. */
  deltaW: number | null;
  /** Δ in % van verwacht; null wanneer actualW null is of verwacht ≈ 0. */
  deltaPct: number | null;
  /** Binnen tolerantie? null wanneer actualW null is. */
  pass: boolean | null;
}

/** Volledig vergelijkresultaat voor één verificatieproject. */
export interface VerificationComparison {
  rooms: ComparisonRow[];
  /** Gebouwtotaal: Vabi `phi_hl_build` vs engine `summary.connection_capacity`. */
  building: ComparisonRow;
  /** Aantal vertrekken met pass === true. */
  passedRooms: number;
  /** Aantal vertrek-rijen met een berekende waarde. */
  totalRooms: number;
  buildingPass: boolean;
  /** Alle vertrekken én het gebouwtotaal binnen tolerantie. */
  allPass: boolean;
}

/**
 * Tolerantie-check — 1-op-1 met `close_enough` in de Rust-integratietest:
 * de ruimste van ±2 W absoluut en ±2 % relatief geldt.
 */
export function closeEnough(actual: number, expected: number): boolean {
  const tol = Math.max(REL_TOLERANCE * Math.abs(expected), ABS_TOLERANCE_W);
  return Math.abs(actual - expected) <= tol;
}

/** Bouw één vergelijkingsrij uit verwacht + (optioneel) berekend. */
function buildRow(
  roomId: string,
  roomName: string,
  thetaI: number | null,
  expectedW: number,
  actualW: number | null,
): ComparisonRow {
  if (actualW === null || !Number.isFinite(actualW)) {
    return { roomId, roomName, thetaI, expectedW, actualW: null, deltaW: null, deltaPct: null, pass: null };
  }
  const deltaW = actualW - expectedW;
  const deltaPct = Math.abs(expectedW) < 1e-9 ? null : (100 * deltaW) / expectedW;
  return {
    roomId,
    roomName,
    thetaI,
    expectedW,
    actualW,
    deltaW,
    deltaPct,
    pass: closeEnough(actualW, expectedW),
  };
}

/** Naam van de gebouwtotaal-rij. */
export const BUILDING_ROW_NAME = "Gebouwtotaal (Φ_HL,build)";

/**
 * Rijen vóór de eerste verificatie-run: alleen de verwachte Vabi-waarden,
 * berekend/Δ/verdict leeg.
 */
export function expectedOnlyRows(expected: VerificationExpected): {
  rooms: ComparisonRow[];
  building: ComparisonRow;
} {
  return {
    rooms: expected.rooms.map((r) =>
      buildRow(r.room_id, r.room_name, r.theta_i, r.phi_hl_i, null),
    ),
    building: buildRow("__building__", BUILDING_ROW_NAME, null, expected.building.phi_hl_build, null),
  };
}

/**
 * Vergelijk een live engine-resultaat met de Vabi-verwachting.
 *
 * Vertrek-matching primair op `room_id`, fallback op naam
 * (case-insensitief) voor het geval id's afwijken tussen invoer en
 * verwachting. Gebouwtotaal: `summary.connection_capacity` — hetzelfde
 * veld dat de Rust-test tegen `building.phi_hl_build` legt.
 */
export function compareResults(
  expected: VerificationExpected,
  result: ProjectResult,
): VerificationComparison {
  const rooms = expected.rooms.map((exp) => {
    const actual =
      result.rooms.find((r) => r.room_id === exp.room_id) ??
      result.rooms.find(
        (r) => r.room_name.trim().toLowerCase() === exp.room_name.trim().toLowerCase(),
      );
    return buildRow(
      exp.room_id,
      exp.room_name,
      exp.theta_i,
      exp.phi_hl_i,
      actual ? actual.total_heat_loss : null,
    );
  });

  const building = buildRow(
    "__building__",
    BUILDING_ROW_NAME,
    null,
    expected.building.phi_hl_build,
    result.summary.connection_capacity,
  );

  const passedRooms = rooms.filter((r) => r.pass === true).length;
  const buildingPass = building.pass === true;

  return {
    rooms,
    building,
    passedRooms,
    totalRooms: rooms.length,
    buildingPass,
    allPass: buildingPass && passedRooms === rooms.length,
  };
}

// ---------------------------------------------------------------------------
// ISSO 53 — metric-gebaseerde vergelijking (spiegel van de Rust golden-tests)
// ---------------------------------------------------------------------------

/**
 * Grootheden die de golden-tests per vertrek tegen het Vabi-rapport leggen.
 * `total` = Φ_T + Φ_V + Φ_I + Φ_hu — exact zoals
 * `vabi_3floors_total_matches` het Vabi-"Totaal warmteverlies" reconstrueert
 * (NIET het engine-veld `totalHeatLoss`, dat ook Φ_system/Φ_gain kan bevatten).
 */
export type Isso53MetricKey = "phiT" | "phiV" | "phiI" | "phiHu" | "total";

/** Weergavelabels voor de metric-kolom in de Help-UI. */
export const ISSO53_METRIC_LABELS: Record<Isso53MetricKey, string> = {
  phiT: "Φ_T (transmissie)",
  phiV: "Φ_V (ventilatie)",
  phiI: "Φ_I (infiltratie)",
  phiHu: "Φ_hu (opwarmtoeslag)",
  total: "Totaal warmteverlies",
};

/** Eén verwachte (vertrek, grootheid)-waarde uit een ISSO 53 expected.json. */
export interface Isso53ExpectedMetric {
  /** Vertrek-id zoals in `input.json` / het calc-resultaat. */
  roomId: string;
  /** Weergavenaam van het vertrek. */
  roomLabel: string;
  metric: Isso53MetricKey;
  /** Vabi-rapportwaarde in W. */
  expectedW: number;
  /**
   * Relatieve tolerantie in % — de waarde die de bijbehorende Rust
   * golden-test hanteert. Bij `expectedW === 0` geldt in plaats daarvan
   * de absolute 1 W-grens (zie {@link closeEnoughPct}).
   */
  tolerancePct: number;
}

/** Eén rij in de ISSO 53-vergelijkingstabel. */
export interface Isso53ComparisonRow {
  /** Stabiele key: `${roomId}:${metric}`. */
  rowKey: string;
  roomId: string;
  roomLabel: string;
  metric: Isso53MetricKey;
  expectedW: number;
  tolerancePct: number;
  /** Berekend door de actuele engine (W); null = nog niet berekend of vertrek niet gevonden. */
  actualW: number | null;
  deltaW: number | null;
  /** Δ in % van verwacht; null wanneer actualW null is of verwacht ≈ 0. */
  deltaPct: number | null;
  pass: boolean | null;
}

/** Volledig ISSO 53-vergelijkresultaat voor één verificatieproject. */
export interface Isso53Comparison {
  rows: Isso53ComparisonRow[];
  /** Aantal rijen met pass === true. */
  passed: number;
  /** Totaal aantal metric-rijen. */
  total: number;
  allPass: boolean;
}

/**
 * Tolerantie-check — 1-op-1 met `close()` in de Rust golden-tests
 * (`crates/isso53-core/tests/vabi_*_golden.rs`): relatieve grens in %,
 * met als speciale case verwacht ≈ 0 → absolute 1 W-grens (geen deling
 * door nul).
 */
export function closeEnoughPct(actual: number, expected: number, tolerancePct: number): boolean {
  if (Math.abs(expected) < Number.EPSILON) {
    return Math.abs(actual) < 1.0;
  }
  return (Math.abs(actual - expected) / Math.abs(expected)) * 100 < tolerancePct;
}

/** Lees de metric-waarde uit een engine-vertrekresultaat. */
function isso53MetricValue(room: Isso53RoomResult, metric: Isso53MetricKey): number {
  switch (metric) {
    case "phiT":
      return room.phiT;
    case "phiV":
      return room.phiV;
    case "phiI":
      return room.phiI;
    case "phiHu":
      return room.phiHu;
    case "total":
      // Vabi-conventie: Totaal warmteverlies = Φ_T + Φ_V + Φ_I + Φ_hu
      // (spiegel van vabi_3floors_total_matches).
      return room.phiT + room.phiV + room.phiI + room.phiHu;
  }
}

/** Bouw één ISSO 53-vergelijkingsrij uit verwacht + (optioneel) berekend. */
function buildIsso53Row(
  exp: Isso53ExpectedMetric,
  actualW: number | null,
): Isso53ComparisonRow {
  const base = {
    rowKey: `${exp.roomId}:${exp.metric}`,
    roomId: exp.roomId,
    roomLabel: exp.roomLabel,
    metric: exp.metric,
    expectedW: exp.expectedW,
    tolerancePct: exp.tolerancePct,
  };
  if (actualW === null || !Number.isFinite(actualW)) {
    return { ...base, actualW: null, deltaW: null, deltaPct: null, pass: null };
  }
  const deltaW = actualW - exp.expectedW;
  const deltaPct = Math.abs(exp.expectedW) < 1e-9 ? null : (100 * deltaW) / exp.expectedW;
  return {
    ...base,
    actualW,
    deltaW,
    deltaPct,
    pass: closeEnoughPct(actualW, exp.expectedW, exp.tolerancePct),
  };
}

/** Rijen vóór de eerste verificatie-run: alleen de verwachte Vabi-waarden. */
export function isso53ExpectedOnlyRows(
  metrics: ReadonlyArray<Isso53ExpectedMetric>,
): Isso53ComparisonRow[] {
  return metrics.map((m) => buildIsso53Row(m, null));
}

/**
 * Vergelijk een live ISSO 53 engine-resultaat met de Vabi-verwachting.
 * Vertrek-matching op `roomId` (camelCase resultaat-veld).
 */
export function compareIsso53Results(
  metrics: ReadonlyArray<Isso53ExpectedMetric>,
  result: Isso53ProjectResult,
): Isso53Comparison {
  const rows = metrics.map((exp) => {
    const room = result.rooms.find((r) => r.roomId === exp.roomId);
    return buildIsso53Row(exp, room ? isso53MetricValue(room, exp.metric) : null);
  });
  const passed = rows.filter((r) => r.pass === true).length;
  return {
    rows,
    passed,
    total: rows.length,
    allPass: passed === rows.length,
  };
}
