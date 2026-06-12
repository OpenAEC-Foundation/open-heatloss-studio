/**
 * Render-smoke-tests voor de deurspleet-calculator, naar het patroon van
 * `Help.test.tsx`: de vitest-omgeving is "node" (geen jsdom), dus we
 * renderen server-side via `react-dom/server`. De pagina is bewust router-
 * en projectstore-vrij (stateless tool), dus dit kan zonder providers.
 *
 * i18n: we initialiseren hier een synchrone i18next-instantie met de echte
 * NL-resources (`initImmediate: false`, geen async backends), zodat de
 * assertions op de werkelijke NL-teksten lopen i.p.v. op key-strings.
 * Varianten (advies, kantoor-Δp, meerdere deuren) gaan via de
 * `initial`-prop — zelfde testbaarheids-patroon als `Help.initialSection`.
 */
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import { renderToString } from "react-dom/server";
import { beforeAll, describe, expect, it } from "vitest";

import nlCommon from "../i18n/locales/nl/common.json";
import { DoorGapCalculator } from "./DoorGapCalculator";

beforeAll(async () => {
  await i18next.use(initReactI18next).init({
    resources: { nl: { common: nlCommon } },
    lng: "nl",
    fallbackLng: "nl",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    initImmediate: false,
  });
});

describe("DoorGapCalculator-pagina", () => {
  it("rendert titel, invoervelden en norm-referentie (zonder project/providers)", () => {
    const html = renderToString(<DoorGapCalculator />);
    expect(html).toContain("Deurspleet-calculator");
    expect(html).toContain("Overstroomdebiet");
    expect(html).toContain("Deurbreedte");
    expect(html).toContain("Aantal deuren");
    expect(html).toContain("NEN 1087:2001 §5.1.3.2");
    // Defaults: 880 mm deurbreedte, Δp-preset woonfunctie, geluidswerend-vinkje.
    expect(html).toContain("880");
    expect(html).toContain("1 Pa — woonfunctie");
    expect(html).toContain("2 Pa — kantoor");
    expect(html).toContain("Geluidswerend");
  });

  it("default (7 dm³/s, 880 mm, 1 Pa): 90,4 cm², 11 mm, vuistregel 84 cm², advies ok", () => {
    const html = renderToString(<DoorGapCalculator />);
    expect(html).toContain("90.4 cm²");
    expect(html).toContain("11 mm");
    // Vuistregel-vergelijking: 7 × 12 = 84 cm², met reconciliatie-noot.
    expect(html).toContain("Vuistregel (12 cm² per dm³/s)");
    expect(html).toContain("84 cm²");
    expect(html).toContain("12,9 cm² per dm³/s");
    expect(html).toContain("Uitvoerbaar als spleet onder de deur.");
    // Geen rooster-advies of -voorstel onder de 20 mm-drempel.
    expect(html).not.toContain("Voorstel deurrooster");
  });

  it("hoog debiet (25 dm³/s → 37 mm) toont rooster-advies + voorstel 2× 455×90", () => {
    const html = renderToString(<DoorGapCalculator initial={{ flowDm3s: 25 }} />);
    expect(html).toContain("322.7 cm²");
    expect(html).toContain("37 mm");
    expect(html).toContain("Pas een deurrooster toe");
    expect(html).not.toContain("Uitvoerbaar als spleet onder de deur.");
    // Voorstel: benodigde netto doorlaat (322,7 cm²) past niet in één
    // seed-rooster (max 273 cm² netto) → 2× 455×90 (2 × 163,8 = 327,6 cm²).
    expect(html).toContain("Voorstel deurrooster (indicatief)");
    expect(html).toContain("Benodigde netto doorlaat");
    expect(html).toContain("2× 455×90 mm");
    expect(html).toContain("163.8 cm² per rooster");
    expect(html).toContain("327.6 cm²");
    // Indicatief-disclaimer (geen fabrikantdata) altijd bij het voorstel.
    expect(html).toContain("Controleer altijd het productblad");
  });

  it("geluidswerend: altijd rooster-advies, groter rooster (25%-fractie) + noot", () => {
    // Default-debiet (7 dm³/s → 11 mm) zou zónder vinkje "ok" zijn.
    const html = renderToString(<DoorGapCalculator initial={{ acoustic: true }} />);
    expect(html).toContain("geluidswerend deurrooster");
    expect(html).not.toContain("Uitvoerbaar als spleet onder de deur.");
    // 90,4 cm² netto bij 25%-fractie → 1× 425×90 (netto 95,6 cm²).
    expect(html).toContain("Voorstel deurrooster (indicatief)");
    expect(html).toContain("1× 425×90 mm");
    expect(html).toContain("95.6 cm² per rooster");
    // Kanttekening geluidswerende uitvoering (conservatieve fractie).
    expect(html).toContain("25% i.p.v. 40%");
  });

  it("kantoor-preset (2 Pa) geeft een kleinere doorlaat dan woonfunctie", () => {
    const html = renderToString(
      <DoorGapCalculator initial={{ deltaPPreset: "office" }} />,
    );
    // 7 dm³/s bij 2 Pa: 90,4/√2 ≈ 63,9 cm² → 8 mm.
    expect(html).toContain("63.9 cm²");
    expect(html).toContain("8 mm");
  });

  it("meerdere deuren: debiet gelijk verdeeld, resultaat per deur", () => {
    const html = renderToString(
      <DoorGapCalculator initial={{ flowDm3s: 14, doorCount: 2 }} />,
    );
    // 14 dm³/s over 2 deuren → 7 dm³/s per deur → zelfde uitkomst als default.
    expect(html).toContain("Debiet per deur");
    expect(html).toContain("90.4 cm²");
    expect(html).toContain("11 mm");
  });
});
