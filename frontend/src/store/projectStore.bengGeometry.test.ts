/**
 * Tests voor de F6 `beng_geometry`-sidecar op `projectStore`.
 *
 * Geborgd gedrag:
 *  - `updateBengGeometry` is een deel-merge: een patch op één sleutel laat de
 *    andere sleutels (bibliotheken/zones) intact.
 *  - Een `undefined`-waarde in de patch betekent "niet aanraken" en mag een
 *    bestaande lijst NIET wissen (defensief tegen een spread-footgun).
 *  - `reset()` zet `bengGeometry` terug op `null` (geen lek naar een ander
 *    project).
 *  - Persist-migratie: een gepersisteerde state van vóór F6 (zonder
 *    `bengGeometry`) rehydrateert naar `null` — bestaande projecten laden
 *    ongewijzigd.
 */
import { afterEach, describe, expect, it } from "vitest";

import { mergePersistedProjectStore, useProjectStore } from "./projectStore";
import type { BengGeometry } from "../types/bengGeometry";

const SAMPLE: BengGeometry = {
  opaque_defs: [
    { id: "def-wand", omschrijving: "Wand", kind: "gevel", thermal: { rc: 4.7 } },
  ],
  window_defs: [
    {
      id: "merk-a",
      omschrijving: "A",
      kind: "raam",
      u_w_per_m2k: 1.3,
      ggl: 0.4,
      area_m2: 4.12,
    },
  ],
  zones: [
    {
      id: "rz-woning",
      naam: "woning",
      a_g_m2: 67.0,
      gevels: [
        {
          id: "gevel-o",
          omschrijving: "Wand",
          vlak_type: "gevel",
          grenst_aan: { buitenlucht: { orientatie: "oost" } },
          bruto_buiten_opp_m2: 23.81,
          helling_deg: 90,
          constructie_ref: "def-wand",
          ramen: [
            {
              kozijn_ref: "merk-a",
              aantal: 1,
              belemmering: "minimal",
              zomernachtventilatie: false,
            },
          ],
        },
      ],
    },
  ],
};

afterEach(() => {
  useProjectStore.getState().reset();
});

describe("projectStore — beng_geometry-sidecar", () => {
  it("deel-patch laat andere sleutels intact", () => {
    const store = useProjectStore.getState;
    store().setBengGeometry({ opaque_defs: SAMPLE.opaque_defs });

    store().updateBengGeometry({ window_defs: SAMPLE.window_defs });

    const geo = store().bengGeometry;
    expect(geo?.opaque_defs).toHaveLength(1);
    expect(geo?.opaque_defs?.[0]?.id).toBe("def-wand");
    expect(geo?.window_defs?.[0]?.id).toBe("merk-a");
  });

  it("undefined-waarde in de patch wist een bestaande lijst NIET", () => {
    const store = useProjectStore.getState;
    store().setBengGeometry({ opaque_defs: SAMPLE.opaque_defs });

    // Expliciet undefined meegeven — mag opaque_defs niet slopen.
    store().updateBengGeometry({ opaque_defs: undefined });

    expect(store().bengGeometry?.opaque_defs).toHaveLength(1);
  });

  it("bootstrapt {} wanneer er nog niets is", () => {
    const store = useProjectStore.getState;
    expect(store().bengGeometry).toBeNull();

    store().updateBengGeometry({ zones: SAMPLE.zones });

    expect(store().bengGeometry?.zones?.[0]?.id).toBe("rz-woning");
  });

  it("reset() zet beng_geometry terug op null", () => {
    const store = useProjectStore.getState;
    store().setBengGeometry(SAMPLE);
    store().reset();
    expect(store().bengGeometry).toBeNull();
  });

  it("setProject wist beng_geometry (geen lek naar het volgende project)", () => {
    const store = useProjectStore.getState;
    store().setBengGeometry(SAMPLE);
    // Herbruik het huidige project als 'nieuw' project — de sidecar moet weg.
    store().setProject(store().project);
    expect(store().bengGeometry).toBeNull();
  });
});

describe("mergePersistedProjectStore — beng_geometry migratie", () => {
  it("state van vóór F6 (zonder bengGeometry) rehydrateert naar null", () => {
    const current = useProjectStore.getState();
    // Persisted blob zonder de bengGeometry-sleutel (legacy).
    const persisted = { project: current.project, norm: "isso51" as const };

    const merged = mergePersistedProjectStore(persisted, current);

    expect(merged.bengGeometry).toBeNull();
  });

  it("een gepersisteerde bengGeometry wordt hersteld", () => {
    const current = useProjectStore.getState();
    const persisted = {
      project: current.project,
      norm: "isso51" as const,
      bengGeometry: SAMPLE,
    };

    const merged = mergePersistedProjectStore(persisted, current);

    expect(merged.bengGeometry?.zones?.[0]?.a_g_m2).toBe(67.0);
  });
});
