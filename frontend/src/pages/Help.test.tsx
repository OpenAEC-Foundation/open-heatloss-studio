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

  it("Verificatie rendert beide Vabi-referentieprojecten met expected-tabel", () => {
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
  });
});
