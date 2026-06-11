/**
 * Pure vergelijklogica voor de Help-sectie "Verificatie".
 *
 * Vergelijkt de Vabi-referentiewaarden (truth uit
 * `tests/verification/<project>/expected.json`) met een live `ProjectResult`
 * van de actuele rekenkern. De toleranties zijn identiek aan de
 * Rust-integratietests (`crates/isso51-core/tests/integration_test.rs`):
 * PASS bij `|Δ| ≤ max(2 W absoluut; 2 % van verwacht)`.
 *
 * Bewust zonder React/store/fetch — node-env unit-testbaar
 * (`verificationCompare.test.ts`).
 */
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
