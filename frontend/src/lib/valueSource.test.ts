/**
 * Unit-tests voor de bronregistratie-helpers (F4c). Puur logica; geen DOM —
 * de vitest-omgeving is "node" (SSR), dus interactieve select-invoer valt buiten
 * deze toolchain (zie Beng.test.tsx). De select-opties + de resultaat-formattering
 * zijn hier volledig te dekken.
 */
import { describe, expect, it } from "vitest";

import type { ValueSourceReport } from "../types/beng";
import {
  VALUE_SOURCE_KINDS,
  bengSubsystemLabel,
  formatValueSourceReport,
  valueSourceKindLabel,
} from "./valueSource";

describe("VALUE_SOURCE_KINDS", () => {
  it("bevat de vijf serde-waarden met forfait vooraan", () => {
    expect(VALUE_SOURCE_KINDS.map((k) => k.value)).toEqual([
      "forfait",
      "kwaliteitsverklaring",
      "gelijkwaardigheidsverklaring",
      "meting",
      "overig",
    ]);
  });
});

describe("valueSourceKindLabel", () => {
  it("mapt een bekende soort naar het NL-label", () => {
    expect(valueSourceKindLabel("kwaliteitsverklaring")).toBe(
      "Kwaliteitsverklaring (BCRG)",
    );
  });
});

describe("bengSubsystemLabel", () => {
  it("mapt elk deelsysteem naar een NL-label", () => {
    expect(bengSubsystemLabel("heating")).toBe("Verwarming");
    expect(bengSubsystemLabel("dwtw")).toBe("Douchewater-WTW");
    expect(bengSubsystemLabel("pv")).toBe("PV");
  });
});

describe("formatValueSourceReport", () => {
  it("toont systeem, soort en (getrimde) referentie", () => {
    const r: ValueSourceReport = {
      system: "heating",
      kind: "kwaliteitsverklaring",
      reference: "  BCRG-20231234  ",
    };
    expect(formatValueSourceReport(r)).toBe(
      "Verwarming: Kwaliteitsverklaring (BCRG), ref. BCRG-20231234",
    );
  });

  it("neemt een PV-veldlabel op tussen haakjes", () => {
    const r: ValueSourceReport = {
      system: "pv",
      label: "dak-zuid",
      kind: "meting",
      reference: null,
    };
    expect(formatValueSourceReport(r)).toBe("PV (dak-zuid): Meting");
  });

  it("laat een lege referentie weg", () => {
    const r: ValueSourceReport = {
      system: "cooling",
      kind: "overig",
      reference: "   ",
    };
    expect(formatValueSourceReport(r)).toBe("Koeling: Overig");
  });
});
