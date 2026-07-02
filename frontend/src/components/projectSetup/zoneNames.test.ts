import { describe, expect, it } from "vitest";

import type { Zone } from "../../types";
import { normalizeZoneName, zoneNameExists } from "./zoneNames";

const ZONES: Zone[] = [
  { id: "zone-a", name: "Begane grond" },
  { id: "zone-b", name: "Verdieping" },
];

describe("normalizeZoneName", () => {
  it("trimt omringende whitespace", () => {
    expect(normalizeZoneName("  Zolder  ")).toBe("Zolder");
    expect(normalizeZoneName("Zolder")).toBe("Zolder");
    expect(normalizeZoneName("   ")).toBe("");
  });
});

describe("zoneNameExists", () => {
  it("herkent een exacte naam als duplicaat", () => {
    expect(zoneNameExists(ZONES, "Verdieping")).toBe(true);
  });

  it("is case-insensitief en trim-tolerant", () => {
    expect(zoneNameExists(ZONES, "  begane GROND ")).toBe(true);
    expect(zoneNameExists(ZONES, "VERDIEPING")).toBe(true);
  });

  it("geeft false voor een nieuwe naam", () => {
    expect(zoneNameExists(ZONES, "Zolder")).toBe(false);
  });

  it("telt een lege (na trim) naam nooit als duplicaat", () => {
    expect(zoneNameExists(ZONES, "   ")).toBe(false);
    expect(zoneNameExists(ZONES, "")).toBe(false);
  });

  it("sluit de eigen zone uit via exceptId (hernoem-pad)", () => {
    // Zone "zone-a" naar exact dezelfde naam → geen duplicaat (het is zichzelf).
    expect(zoneNameExists(ZONES, "Begane grond", "zone-a")).toBe(false);
    // Maar naar de naam van een ándere zone → wél duplicaat.
    expect(zoneNameExists(ZONES, "Verdieping", "zone-a")).toBe(true);
  });
});
