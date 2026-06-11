/**
 * Render-smoke-tests voor de Help-pagina.
 *
 * De vitest-omgeving is "node" (geen jsdom), dus we renderen server-side
 * via `react-dom/server` — voldoende om te garanderen dat de pagina en
 * alle vier content-secties zonder runtime-errors renderen en hun
 * kerncontent bevatten. De Help-pagina en de content-modules zijn bewust
 * router- en store-vrij, zodat dit zonder providers kan.
 */
import { renderToString } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { Help } from "./Help";
import { AFWIJKINGEN, HelpAfwijkingen } from "../content/help/HelpAfwijkingen";
import { HelpFormules } from "../content/help/HelpFormules";
import { HelpGebruik } from "../content/help/HelpGebruik";
import { HelpVerificatie } from "../content/help/HelpVerificatie";
import {
  buildIsso53VerifyPayload,
  ISSO53_VERIFICATION_PROJECTS,
  VERIFICATION_PROJECTS,
} from "../content/help/verificationData";

describe("Help-pagina", () => {
  it("rendert met alle vier sectie-tabs", () => {
    const html = renderToString(<Help />);
    expect(html).toContain("Gebruik");
    expect(html).toContain("Formules");
    expect(html).toContain("Afwijkingen");
    expect(html).toContain("Verificatie");
  });

  it("toont default de Gebruik-sectie met de werkflow", () => {
    const html = renderToString(<Help />);
    expect(html).toContain("Werkflow in zes stappen");
    expect(html).toContain("Drie invoerroutes");
  });

  it.each([
    ["formules", "Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)"],
    ["afwijkingen", "Aggregatiemethode"],
    ["verificatie", "Verifieer nu"],
  ] as const)("rendert sectie %s via initialSection", (section, expected) => {
    const html = renderToString(<Help initialSection={section} />);
    expect(html).toContain(expected);
  });
});

describe("Help content-secties (los)", () => {
  it("Gebruik rendert de eenheden-conventie", () => {
    const html = renderToString(<HelpGebruik />);
    expect(html).toContain("dm³/s");
    expect(html).toContain("m³/h");
  });

  it("Formules rendert hoofdformules met norm-referenties", () => {
    const html = renderToString(<HelpFormules />);
    expect(html).toContain("H_v = 1,2 × q_v × f_v");
    expect(html).toContain("Φ_hu = P × A_g");
    expect(html).toContain("ISSO 51:2023");
  });

  it("Afwijkingen rendert alle gedocumenteerde rijen", () => {
    const html = renderToString(<HelpAfwijkingen />);
    expect(AFWIJKINGEN.length).toBeGreaterThanOrEqual(5);
    for (const a of AFWIJKINGEN) {
      expect(html).toContain(a.onderwerp.replace(/&/g, "&amp;"));
    }
    // θ_w engineering-aanname moet expliciet gedocumenteerd zijn.
    expect(html).toContain("θ_w");
  });

  it("Verificatie rendert beide ISSO 51 Vabi-referentieprojecten met expected-tabel", () => {
    const html = renderToString(<HelpVerificatie />);
    // Beide ISSO 51-verificatieprojecten aanwezig.
    expect(html).toContain("Vrijstaande woning");
    expect(html).toContain("DR Engineering");
    // Vabi-gebouwtotalen uit expected.json staan al vóór de eerste run in de tabel.
    expect(html).toContain("9160");
    expect(html).toContain("6700");
    // Actieknop + gebouwtotaal-rij.
    expect(html).toContain("Verifieer nu");
    expect(html).toContain("Gebouwtotaal");
    // CI-spiegel-uitleg in de intro.
    expect(html).toContain("golden-tests");
  });

  it("Verificatie toont de vrijstaande woning als informatief met disclaimer", () => {
    const html = renderToString(<HelpVerificatie />);
    // Disclaimer-banner + neutrale badge.
    expect(html).toContain("Informatief — geen referentie");
    expect(html).toContain("informatief — geen referentie");
    expect(html).toContain("ISSO 51:2017");
    expect(html).toContain("normversie-verschillen, geen rekenfouten");
  });

  it("Verificatie rendert de drie ISSO 53-projecten met expected-waarden", () => {
    const html = renderToString(<HelpVerificatie />);
    expect(html).toContain("TR02 Houtfabriek — 3 verdiepingen");
    expect(html).toContain("TR02 Houtfabriek — Bedrijfsruimte 4");
    expect(html).toContain("Kantoor West");
    // Vabi-totalen per project staan al vóór de eerste run in de tabel.
    expect(html).toContain("3389"); // 3floors room 1.10a
    expect(html).toContain("8161"); // bedrijfsruimte4 room 0.14
    expect(html).toContain("3741"); // kantoorwest room 0.03
    // Metric-labels + tolerantie-kolom.
    expect(html).toContain("Φ_T (transmissie)");
    expect(html).toContain("Tolerantie");
  });
});

describe("verificationData — ISSO 53 defs + payload", () => {
  it("vrijstaande woning is informatief, DR Engineering blijft referentie", () => {
    const vrijstaand = VERIFICATION_PROJECTS.find((p) =>
      p.id.includes("vrijstaande-woning"),
    );
    const dr = VERIFICATION_PROJECTS.find((p) => p.id.includes("dr-engineering"));
    expect(vrijstaand?.mode).toBe("informative");
    expect(vrijstaand?.disclaimer).toBeTruthy();
    expect(dr?.mode).toBe("reference");
    expect(dr?.disclaimer).toBeUndefined();
  });

  it("elke ISSO 53-def heeft consistente, eindige metric-rijen", () => {
    expect(ISSO53_VERIFICATION_PROJECTS).toHaveLength(3);
    for (const def of ISSO53_VERIFICATION_PROJECTS) {
      expect(def.metrics.length).toBeGreaterThanOrEqual(4);
      for (const m of def.metrics) {
        expect(Number.isFinite(m.expectedW)).toBe(true);
        expect(m.tolerancePct).toBeGreaterThanOrEqual(0);
        expect(m.roomId.length).toBeGreaterThan(0);
        expect(m.roomLabel.length).toBeGreaterThan(0);
      }
      // Elk project eindigt met een totaal-metric per vertrek.
      expect(def.metrics.some((m) => m.metric === "total")).toBe(true);
    }
  });

  it("normaliseert de heterogene expected-shapes naar de juiste Vabi-truth", () => {
    const byId = Object.fromEntries(ISSO53_VERIFICATION_PROJECTS.map((d) => [d.id, d]));

    const floors = byId["isso53_vabi3.11.2.23_houtfabriek-3floors"];
    expect(floors?.metrics.filter((m) => m.metric === "total").map((m) => m.expectedW))
      .toEqual([3389, 3369, 3446]);
    expect(
      floors?.metrics.find((m) => m.roomId === "1.10a" && m.metric === "phiT")?.tolerancePct,
    ).toBe(5.5);

    const bedrijfsruimte = byId["isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4"];
    // Vabi's phiV_plus_phiI (3080) is volledig infiltratie (Φ_V = 0).
    expect(bedrijfsruimte?.metrics.find((m) => m.metric === "phiI")?.expectedW).toBe(3080);
    expect(bedrijfsruimte?.metrics.find((m) => m.metric === "phiV")?.expectedW).toBe(0);
    expect(bedrijfsruimte?.metrics.find((m) => m.metric === "total")?.expectedW).toBe(8161);

    const kantoorwest = byId["isso53_vabi3.12.0.127_dr-engineering-kantoorwest"];
    expect(kantoorwest?.metrics.find((m) => m.metric === "phiT")?.expectedW).toBe(3059);
    // Golden-test V2-verstrakking: Φ_T 4%, Φ_I 2,5% (niet de 10/5 uit expected.json).
    expect(kantoorwest?.metrics.find((m) => m.metric === "phiT")?.tolerancePct).toBe(4.0);
    expect(kantoorwest?.metrics.find((m) => m.metric === "phiI")?.tolerancePct).toBe(2.5);
  });

  it("buildIsso53VerifyPayload routeert naar ISSO 53 met de input-blob inline", () => {
    for (const def of ISSO53_VERIFICATION_PROJECTS) {
      const payload = buildIsso53VerifyPayload(def);
      expect(payload.schema_version).toBe(2);
      // active_norm()-routing: isso51 null + isso53 gevuld → Isso53.
      expect(payload.calcs.isso51).toBeNull();
      expect(payload.calcs.isso53).toBe(def.input);
      expect(payload.calcs.tojuli).toBeNull();
      // De legacy blob bevat de projectvelden inline (serde-flatten contract).
      expect(payload.calcs.isso53).toHaveProperty("info");
      expect(payload.calcs.isso53).toHaveProperty("building");
      expect(payload.calcs.isso53).toHaveProperty("rooms");
    }
  });
});
