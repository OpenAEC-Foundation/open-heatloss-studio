/**
 * Unit-tests voor de pure vergelijklogica van de verificatie-sectie.
 *
 * Toleranties moeten 1-op-1 sporen met de Rust-integratietests
 * (`crates/isso51-core/tests/integration_test.rs::close_enough`):
 * PASS bij `|Δ| ≤ max(2 W abs; 2 % rel)`.
 */
import { describe, expect, it } from "vitest";

import {
  BUILDING_ROW_NAME,
  closeEnough,
  compareResults,
  expectedOnlyRows,
  type VerificationExpected,
} from "./verificationCompare";
import type { ProjectResult, RoomResult } from "../types/result";

// Echte verificatiedata — zelfde bestanden als de Rust-tests en de Help-UI.
import vrijstaandeWoningExpected from "../../../tests/verification/isso51_vabi3.8.1.14_vrijstaande-woning/expected.json";
import drEngineeringExpected from "../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/expected.json";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const EXPECTED: VerificationExpected = {
  rooms: [
    { room_id: "0.1", room_name: "Hal", theta_i: 18.0, phi_hl_i: 245 },
    { room_id: "0.2", room_name: "Woonkamer", theta_i: 20.0, phi_hl_i: 2480 },
    { room_id: "0.3", room_name: "Toilet", theta_i: 18.0, phi_hl_i: 0 },
  ],
  building: { phi_hl_build: 9160 },
};

/** Minimale RoomResult — alleen de velden die de vergelijking raakt zijn echt. */
function roomResult(roomId: string, roomName: string, totalHeatLoss: number): RoomResult {
  return {
    room_id: roomId,
    room_name: roomName,
    theta_i: 20,
    transmission: {
      h_t_exterior: 0,
      h_t_adjacent_rooms: 0,
      h_t_unheated: 0,
      h_t_adjacent_buildings: 0,
      h_t_ground: 0,
      phi_t: 0,
    },
    infiltration: { h_i: 0, z_i: 0, phi_i: 0 },
    ventilation: { h_v: 0, f_v: 0, q_v: 0, phi_v: 0, phi_vent: 0 },
    heating_up: { phi_hu: 0, p: 0, a_g: 0 },
    system_losses: {
      phi_floor_loss: 0,
      phi_wall_loss: 0,
      phi_ceiling_loss: 0,
      phi_system_total: 0,
    },
    total_heat_loss: totalHeatLoss,
    basis_heat_loss: totalHeatLoss,
    extra_heat_loss: 0,
  };
}

function projectResult(
  rooms: RoomResult[],
  connectionCapacity: number,
): ProjectResult {
  return {
    rooms,
    summary: {
      total_envelope_loss: 0,
      total_neighbor_loss: 0,
      total_ventilation_loss: 0,
      total_heating_up: 0,
      total_system_losses: 0,
      connection_capacity: connectionCapacity,
      collective_contribution: 0,
    },
  };
}

// ---------------------------------------------------------------------------
// closeEnough — tolerantie-grenzen
// ---------------------------------------------------------------------------

describe("closeEnough", () => {
  it("accepteert exact gelijke waarden", () => {
    expect(closeEnough(245, 245)).toBe(true);
  });

  it("hanteert ±2 W absoluut bij kleine waarden (waar 2% < 2 W)", () => {
    // 2% van 50 = 1 W → abs-tolerantie 2 W is de ruimste.
    expect(closeEnough(52, 50)).toBe(true);
    expect(closeEnough(52.01, 50)).toBe(false);
    expect(closeEnough(48, 50)).toBe(true);
    expect(closeEnough(47.99, 50)).toBe(false);
  });

  it("hanteert ±2% relatief bij grote waarden (waar 2% > 2 W)", () => {
    // 2% van 1000 = 20 W.
    expect(closeEnough(1020, 1000)).toBe(true);
    expect(closeEnough(1020.01, 1000)).toBe(false);
    expect(closeEnough(980, 1000)).toBe(true);
    expect(closeEnough(979.99, 1000)).toBe(false);
  });

  it("valt bij verwacht 0 terug op ±2 W absoluut", () => {
    expect(closeEnough(0, 0)).toBe(true);
    expect(closeEnough(2, 0)).toBe(true);
    expect(closeEnough(2.01, 0)).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// compareResults
// ---------------------------------------------------------------------------

describe("compareResults", () => {
  it("berekent delta's en verdicts per vertrek", () => {
    const result = projectResult(
      [
        roomResult("0.1", "Hal", 246), // Δ +1 W → pass (abs)
        roomResult("0.2", "Woonkamer", 2540), // Δ +60 W = +2.42% → fail
        roomResult("0.3", "Toilet", 0), // Δ 0 → pass
      ],
      9165,
    );
    const cmp = compareResults(EXPECTED, result);

    expect(cmp.rooms).toHaveLength(3);
    const [hal, woonkamer, toilet] = cmp.rooms;

    expect(hal?.deltaW).toBeCloseTo(1);
    expect(hal?.deltaPct).toBeCloseTo(100 / 245, 3);
    expect(hal?.pass).toBe(true);

    expect(woonkamer?.deltaW).toBeCloseTo(60);
    expect(woonkamer?.pass).toBe(false);

    // Verwacht 0 W → geen percentage, wel abs-verdict.
    expect(toilet?.deltaPct).toBeNull();
    expect(toilet?.pass).toBe(true);

    expect(cmp.passedRooms).toBe(2);
    expect(cmp.totalRooms).toBe(3);
    expect(cmp.allPass).toBe(false);
  });

  it("vergelijkt het gebouwtotaal tegen summary.connection_capacity", () => {
    const result = projectResult(
      EXPECTED.rooms.map((r) => roomResult(r.room_id, r.room_name, r.phi_hl_i)),
      9160 * 1.019, // +1,9% — binnen de 2%-grens → pass (exact op de grens is float-gevoelig)
    );
    const cmp = compareResults(EXPECTED, result);
    expect(cmp.building.roomName).toBe(BUILDING_ROW_NAME);
    expect(cmp.building.expectedW).toBe(9160);
    expect(cmp.building.pass).toBe(true);
    expect(cmp.buildingPass).toBe(true);
    expect(cmp.allPass).toBe(true);
  });

  it("markeert gebouwtotaal buiten tolerantie als fail (lineaire-som regressie)", () => {
    const result = projectResult(
      EXPECTED.rooms.map((r) => roomResult(r.room_id, r.room_name, r.phi_hl_i)),
      // Historische regressie-waarde: lineaire som gaf ~8121 i.p.v. ~6700.
      9160 * 1.21,
    );
    const cmp = compareResults(EXPECTED, result);
    expect(cmp.building.pass).toBe(false);
    expect(cmp.allPass).toBe(false);
  });

  it("matcht op room_id en valt terug op naam (case-insensitief)", () => {
    const result = projectResult(
      [
        roomResult("anders-id", "HAL ", 245), // id mismatcht → naam-match
        roomResult("0.2", "Hernoemd", 2480), // id-match wint
      ],
      9160,
    );
    const cmp = compareResults(EXPECTED, result);
    expect(cmp.rooms[0]?.actualW).toBe(245);
    expect(cmp.rooms[1]?.actualW).toBe(2480);
  });

  it("laat ontbrekende vertrekken leeg (geen crash, pass = null)", () => {
    const result = projectResult([roomResult("0.1", "Hal", 245)], 9160);
    const cmp = compareResults(EXPECTED, result);
    const missing = cmp.rooms[1];
    expect(missing?.actualW).toBeNull();
    expect(missing?.deltaW).toBeNull();
    expect(missing?.deltaPct).toBeNull();
    expect(missing?.pass).toBeNull();
    expect(cmp.totalRooms).toBe(3);
    // Alleen "Hal" is gevonden én binnen tolerantie; de twee ontbrekende
    // vertrekken tellen niet als pass.
    expect(cmp.passedRooms).toBe(1);
    expect(cmp.allPass).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// expectedOnlyRows — staat vóór de eerste run
// ---------------------------------------------------------------------------

describe("expectedOnlyRows", () => {
  it("levert rijen met alleen verwachte waarden", () => {
    const { rooms, building } = expectedOnlyRows(EXPECTED);
    expect(rooms).toHaveLength(3);
    for (const row of rooms) {
      expect(row.actualW).toBeNull();
      expect(row.deltaW).toBeNull();
      expect(row.pass).toBeNull();
    }
    expect(rooms[0]?.expectedW).toBe(245);
    expect(building.expectedW).toBe(9160);
    expect(building.thetaI).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Echte expected.json bestanden — shape-borging voor de Help-UI
// ---------------------------------------------------------------------------

describe("verificatiedata (tests/verification)", () => {
  it.each([
    ["vrijstaande-woning (Vabi 3.8.1.14)", vrijstaandeWoningExpected as VerificationExpected, 9160],
    ["dr-engineering (Vabi 3.12.0.127)", drEngineeringExpected as VerificationExpected, 6700],
  ])("%s heeft het verwachte shape", (_label, expected, phiHlBuild) => {
    expect(expected.rooms.length).toBeGreaterThan(0);
    expect(expected.building.phi_hl_build).toBe(phiHlBuild);
    for (const room of expected.rooms) {
      expect(typeof room.room_id).toBe("string");
      expect(typeof room.room_name).toBe("string");
      expect(Number.isFinite(room.theta_i)).toBe(true);
      expect(Number.isFinite(room.phi_hl_i)).toBe(true);
    }
    // De expected-only weergave (vóór eerste run) mag nooit crashen.
    const { rooms, building } = expectedOnlyRows(expected);
    expect(rooms).toHaveLength(expected.rooms.length);
    expect(building.expectedW).toBe(phiHlBuild);
  });
});
