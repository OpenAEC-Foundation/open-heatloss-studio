/**
 * Conversie-randgedrag van de eenheden-toggle dm³/s ↔ m³/h.
 *
 * De store rekent en bewaart ALTIJD dm³/s (header `types/ventilation.ts`);
 * de toggle is puur weergave. Deze tests dekken de UI-rand:
 *   - weergave: store-waarde → gekozen eenheid + afronding (dm³/s 1 decimaal,
 *     m³/h hele getallen) — alleen in de weergave, nooit in de store;
 *   - parse: invoer in de gekozen eenheid → exacte dm³/s store-waarde
 *     (round-trip zonder afrondingsdrift);
 *   - unit-capaciteit (opslag in hele m³/h, fabrikant-conventie): invoerveld
 *     in dm³/s-stand rond-tript exact naar dezelfde opgeslagen m³/h.
 */

import { describe, expect, it } from "vitest";

import {
  DM3S_TO_M3H,
  FLOW_UNIT_DECIMALS,
  FLOW_UNIT_LABELS,
  flowFromDisplay,
  flowToDisplay,
  otherFlowUnit,
} from "../../types/ventilation";
import {
  flowDisplayLabel,
  flowLabel,
  flowSecondaryLabel,
  m3hLabel,
} from "./shared";
import { capacityToFormValue, formToUnit } from "./UnitsCard";

// ---------------------------------------------------------------------------
// Pure conversie (types/ventilation.ts)
// ---------------------------------------------------------------------------

describe("flowToDisplay / flowFromDisplay", () => {
  it("dm³/s-stand is de identiteit (geen conversie, geen drift)", () => {
    for (const v of [0, 7, 12.5, 83.333333, 0.1]) {
      expect(flowToDisplay(v, "dm3s")).toBe(v);
      expect(flowFromDisplay(v, "dm3s")).toBe(v);
    }
  });

  it("m³/h-stand: weergave × 3,6, invoer ÷ 3,6", () => {
    expect(flowToDisplay(12.5, "m3h")).toBe(45);
    expect(flowFromDisplay(45, "m3h")).toBe(12.5);
    expect(flowFromDisplay(90, "m3h")).toBe(25);
  });

  it("invoer in m³/h → exacte dm³/s in de store; terugschakelen verandert niets", () => {
    // Gebruiker typt 45 m³/h → store = 12.5 dm³/s (exact). Terug naar
    // dm³/s-stand is identiteit op de store-waarde — geen drift.
    const stored = flowFromDisplay(45, "m3h");
    expect(stored).toBe(12.5);
    expect(flowToDisplay(stored, "dm3s")).toBe(stored);
    // En de m³/h-weergave reproduceert de getypte waarde.
    expect(flowToDisplay(stored, "m3h")).toBe(45);
  });

  it("weergave→parse round-trip blijft op de store-waarde (ook niet-ronde getallen)", () => {
    for (const dm3s of [7, 12.5, 6.944444444444445, 0.3, 21]) {
      const display = flowToDisplay(dm3s, "m3h");
      expect(flowFromDisplay(display, "m3h")).toBeCloseTo(dm3s, 12);
    }
  });

  it("otherFlowUnit wisselt tussen de twee standen", () => {
    expect(otherFlowUnit("dm3s")).toBe("m3h");
    expect(otherFlowUnit("m3h")).toBe("dm3s");
  });

  it("constantes: labels en weergave-decimalen per eenheid", () => {
    expect(DM3S_TO_M3H).toBe(3.6);
    expect(FLOW_UNIT_LABELS).toEqual({ dm3s: "dm³/s", m3h: "m³/h" });
    expect(FLOW_UNIT_DECIMALS).toEqual({ dm3s: 1, m3h: 0 });
  });
});

// ---------------------------------------------------------------------------
// Weergave-formatters (components/ventilation/shared.tsx)
// ---------------------------------------------------------------------------

describe("flowDisplayLabel / flowSecondaryLabel", () => {
  it("dm³/s op 1 decimaal, m³/h op hele getallen", () => {
    expect(flowDisplayLabel(12.5, "dm3s")).toBe("12.5 dm³/s");
    expect(flowDisplayLabel(12.5, "m3h")).toBe("45 m³/h");
    expect(flowDisplayLabel(12.34, "dm3s")).toBe("12.3 dm³/s");
    expect(flowDisplayLabel(6.944444444444445, "m3h")).toBe("25 m³/h");
  });

  it("afronding alleen in de weergave — de doorgegeven store-waarde blijft onaangetast", () => {
    const stored = flowFromDisplay(25, "m3h"); // 6.9444... dm³/s
    expect(flowDisplayLabel(stored, "m3h")).toBe("25 m³/h");
    // Idempotent: opnieuw formatteren vanaf dezelfde store-waarde geeft
    // hetzelfde label (er is geen tussentijdse afgeronde herschrijving).
    expect(flowDisplayLabel(stored, "m3h")).toBe(flowDisplayLabel(stored, "m3h"));
    expect(stored).toBe(25 / 3.6);
  });

  it("flowSecondaryLabel toont de andere eenheid (tussen haakjes-weergave)", () => {
    expect(flowSecondaryLabel(12.5, "dm3s")).toBe("45 m³/h");
    expect(flowSecondaryLabel(12.5, "m3h")).toBe("12.5 dm³/s");
  });

  it("backward-compat: flowLabel/m3hLabel (rapport + zijpaneel) ongewijzigd", () => {
    expect(flowLabel(12.5)).toBe("12.5 dm³/s");
    expect(m3hLabel(12.5)).toBe("45 m³/h");
  });
});

// ---------------------------------------------------------------------------
// Unit-capaciteit invoerveld (UnitsCard) — opslag in hele m³/h
// ---------------------------------------------------------------------------

describe("capacityToFormValue / formToUnit — capaciteit-invoer per eenheid", () => {
  const baseForm = {
    type: "wtw" as const,
    fabrikant: "Test",
    model: "X",
    capaciteit: "",
    rendementPct: "",
    geluidDb: "",
  };

  it("m³/h-stand: invoer 1:1 naar opslag (afgerond op hele m³/h)", () => {
    const unit = formToUnit({ ...baseForm, capaciteit: "300" }, "m3h");
    expect(unit?.capaciteitM3h).toBe(300);
  });

  it("dm³/s-stand: invoer × 3,6 naar opslag", () => {
    // 25 dm³/s = 90 m³/h.
    const unit = formToUnit({ ...baseForm, capaciteit: "25" }, "dm3s");
    expect(unit?.capaciteitM3h).toBe(90);
  });

  it("opslag → invoerveld → opslag rond-tript exact (alle hele m³/h 1..2000)", () => {
    // capacityToFormValue toont dm³/s op 2 decimalen; gedocumenteerde
    // garantie: ×3,6 + afronden op hele m³/h komt exact terug op de opslag.
    for (let m3h = 1; m3h <= 2000; m3h++) {
      const formValue = capacityToFormValue(m3h, "dm3s");
      const unit = formToUnit({ ...baseForm, capaciteit: formValue }, "dm3s");
      expect(unit?.capaciteitM3h).toBe(m3h);
    }
  });

  it("m³/h-stand: opslagwaarde verschijnt onveranderd in het invoerveld", () => {
    expect(capacityToFormValue(300, "m3h")).toBe("300");
    expect(capacityToFormValue(90, "m3h")).toBe("90");
  });

  it("ongeldige of niet-positieve capaciteit → null (submit disabled)", () => {
    expect(formToUnit({ ...baseForm, capaciteit: "" }, "m3h")).toBeNull();
    expect(formToUnit({ ...baseForm, capaciteit: "abc" }, "dm3s")).toBeNull();
    expect(formToUnit({ ...baseForm, capaciteit: "0" }, "m3h")).toBeNull();
    // 0.1 dm³/s → 0.36 m³/h → rondt af naar 0 → ongeldig.
    expect(formToUnit({ ...baseForm, capaciteit: "0.1" }, "dm3s")).toBeNull();
  });
});
