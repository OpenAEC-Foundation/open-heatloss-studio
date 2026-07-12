/**
 * Render-smoke-test voor de BENG-pagina, naar het patroon van
 * `DoorGapCalculator.test.tsx`: de vitest-omgeving is "node" (geen jsdom),
 * dus we renderen server-side via `react-dom/server`.
 *
 * De pagina leest de `projectStore` (zustand — werkt in SSR) en gebruikt
 * `useTranslation`; we initialiseren een synchrone i18next-instantie met de
 * echte NL-resources. Er worden GEEN backend-calls gedaan bij render, dus dit
 * verifieert alleen dat het invoerpaneel zonder runtime-errors rendert.
 */
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import { renderToString } from "react-dom/server";
import { beforeAll, describe, expect, it } from "vitest";

import nlCommon from "../i18n/locales/nl/common.json";
import { Beng } from "./Beng";
import { useProjectStore } from "../store/projectStore";

beforeAll(async () => {
  await i18next.use(initReactI18next).init({
    resources: { nl: { common: nlCommon } },
    lng: "nl",
    fallbackLng: "nl",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    initImmediate: false,
  });
  useProjectStore.getState().reset();
});

describe("BENG-pagina", () => {
  it("rendert titel + alle deelsysteem-kaarten (default: geen energy-blok)", () => {
    const html = renderToString(<Beng />);
    expect(html).toContain("BENG");
    expect(html).toContain("Verwarming (H.9)");
    expect(html).toContain("Warm tapwater (H.13)");
    expect(html).toContain("Ventilatie (H.11)");
    expect(html).toContain("Koeling (H.10)");
    expect(html).toContain("PV — zonnestroom (H.16)");
    expect(html).toContain("Gebouwautomatisering (H.15)");
    expect(html).toContain("Bereken BENG");
  });

  it("toont de read-only context + modeller-scope-hint en de PV-lege-staat", () => {
    // NB: de vitest-omgeving is SSR (node); zustand geeft in SSR de INITIALE
    // state terug (`getServerSnapshot === getInitialState`), dus store-mutaties
    // zijn hier niet zichtbaar. Interactieve panelen (actieve deelsystemen) zijn
    // pas met een DOM-runner (jsdom, niet in deze toolchain) te toetsen — F4c.
    const html = renderToString(<Beng />);
    // Context-paneel toont gebouwtype uit het default-project.
    expect(html).toContain("Project-context (read-only)");
    expect(html).toContain("woning / terraced");
    // Scope-afbakening: raam-zonwering loopt via de Modeller, niet hier.
    expect(html).toContain("Raam-zonwering/belemmering per raam loopt via de Modeller");
    // PV-kaart met lege staat + toevoeg-knop.
    expect(html).toContain("Geen PV-velden");
    expect(html).toContain("+ PV-veld toevoegen");
    // Elk optioneel deelsysteem staat default op "Niet aanwezig".
    expect(html).toContain("Niet aanwezig");
  });
});
