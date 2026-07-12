/**
 * Tests voor de BENG `energy`-sidecar op `projectStore`.
 *
 * Geborgd gedrag:
 *  - `updateEnergy` is een deel-merge: een patch op één deelsysteem laat de
 *    andere deelsystemen intact.
 *  - Een `undefined`-waarde in de patch betekent "niet aanraken" en mag een
 *    bestaand deelsysteem NIET wissen (defensief tegen een spread-footgun).
 *  - Een expliciete `null` wist wél het betreffende deelsysteem (clear-
 *    conventie van dit blok).
 *  - `reset()` zet `energy` terug op `null` (geen lek naar een ander project).
 */
import { afterEach, describe, expect, it } from "vitest";

import { useProjectStore } from "./projectStore";

afterEach(() => {
  useProjectStore.getState().reset();
});

describe("projectStore — energy-sidecar", () => {
  it("deel-patch laat andere deelsystemen intact", () => {
    const store = useProjectStore.getState;
    store().setEnergy({
      heating: { generator: "heat_pump_ground", cop: 4.5 },
      dhw: { generator: "heat_pump", efficiency: 2.8 },
    });

    store().updateEnergy({
      ventilation: { system: "D", wtw_efficiency: 0.85 },
    });

    const energy = store().energy;
    expect(energy?.heating?.generator).toBe("heat_pump_ground");
    expect(energy?.dhw?.efficiency).toBe(2.8);
    expect(energy?.ventilation?.system).toBe("D");
  });

  it("undefined-waarde in de patch wist een bestaand deelsysteem NIET", () => {
    const store = useProjectStore.getState;
    store().setEnergy({
      heating: { generator: "hr_boiler", hr_class: "hr107" },
    });

    // Expliciet undefined meegeven — mag heating niet slopen.
    store().updateEnergy({ heating: undefined });

    expect(store().energy?.heating?.generator).toBe("hr_boiler");
  });

  it("expliciete null wist het betreffende deelsysteem", () => {
    const store = useProjectStore.getState;
    store().setEnergy({
      heating: { generator: "hr_boiler" },
      cooling: { generator: "compression", seer: 4.0 },
    });

    store().updateEnergy({ cooling: null });

    expect(store().energy?.cooling).toBeNull();
    expect(store().energy?.heating?.generator).toBe("hr_boiler");
  });

  it("reset() zet energy terug op null", () => {
    const store = useProjectStore.getState;
    store().setEnergy({ heating: { generator: "hr_boiler" } });
    store().reset();
    expect(store().energy).toBeNull();
  });
});
