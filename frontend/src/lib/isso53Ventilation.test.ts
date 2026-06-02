import { describe, expect, it } from "vitest";

import { isso53BblMinimumDm3s } from "./isso53Ventilation";

describe("isso53BblMinimumDm3s — 0,9 dm³/s per m²", () => {
  it("10,27 m² → 9,243 dm³/s", () => {
    expect(isso53BblMinimumDm3s(10.27)).toBeCloseTo(9.243, 12);
  });

  it("100 m² → 90 dm³/s", () => {
    expect(isso53BblMinimumDm3s(100)).toBe(90);
  });

  it("0 m² → 0 dm³/s", () => {
    expect(isso53BblMinimumDm3s(0)).toBe(0);
  });
});
