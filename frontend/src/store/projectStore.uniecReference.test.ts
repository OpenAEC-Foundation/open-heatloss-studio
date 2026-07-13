/**
 * Tests voor de Uniec 3 certified-referentie-sidecar op `projectStore` (F8).
 *
 * Geborgd gedrag:
 *  - `setUniecReference` zet/wist de referentie.
 *  - `reset()` en een projectwissel (`setProject`) zetten de referentie terug
 *    op `null` (geen lek naar een ander project).
 *  - Persist-migratie: state van vóór F8 (zonder `uniecReference`) rehydrateert
 *    naar `null`; een gepersisteerde referentie wordt hersteld.
 */
import { afterEach, describe, expect, it } from "vitest";

import { mergePersistedProjectStore, useProjectStore } from "./projectStore";
import type { Uniec3CertifiedResults } from "../types/uniec";

const SAMPLE: Uniec3CertifiedResults = {
  app_version: "3.4.0.0",
  beng1_kwh_m2_jr: 42.0,
  beng2_kwh_m2_jr: 18.5,
  beng3_pct: 61.0,
  energy_label: "A++",
};

afterEach(() => {
  useProjectStore.getState().reset();
});

describe("projectStore — uniecReference-sidecar", () => {
  it("setUniecReference zet en wist de referentie", () => {
    const store = useProjectStore.getState;
    store().setUniecReference(SAMPLE);
    expect(store().uniecReference?.energy_label).toBe("A++");

    store().setUniecReference(null);
    expect(store().uniecReference).toBeNull();
  });

  it("reset() zet uniecReference terug op null", () => {
    const store = useProjectStore.getState;
    store().setUniecReference(SAMPLE);
    store().reset();
    expect(store().uniecReference).toBeNull();
  });

  it("setProject (projectwissel) wist de referentie", () => {
    const store = useProjectStore.getState;
    store().setUniecReference(SAMPLE);
    // Herbruik het huidige project als 'nieuw' project — de sidecar moet weg.
    store().setProject(store().project);
    expect(store().uniecReference).toBeNull();
  });
});

describe("mergePersistedProjectStore — uniecReference migratie", () => {
  it("state van vóór F8 (zonder uniecReference) rehydrateert naar null", () => {
    const current = useProjectStore.getState();
    const persisted = { project: current.project, norm: "isso51" as const };

    const merged = mergePersistedProjectStore(persisted, current);

    expect(merged.uniecReference).toBeNull();
  });

  it("een gepersisteerde uniecReference wordt hersteld", () => {
    const current = useProjectStore.getState();
    const persisted = {
      project: current.project,
      norm: "isso51" as const,
      uniecReference: SAMPLE,
    };

    const merged = mergePersistedProjectStore(persisted, current);

    expect(merged.uniecReference?.beng1_kwh_m2_jr).toBe(42.0);
    expect(merged.uniecReference?.energy_label).toBe("A++");
  });
});
