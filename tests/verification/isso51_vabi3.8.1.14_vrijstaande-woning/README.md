# ISSO 51:2017 — Vabi 3.8.1.14 — Vrijstaande woning

| Veld | Waarde |
|---|---|
| Norm | ISSO 51:2017 (incl. 53/57) |
| Software | Vabi Elements 3.8.1.14, rekenkern Warmteverlies 2.30 |
| Bron PDF | `tests/references/vrijstaande-woning-isso51-2017.pdf` |
| Gebouw | Vrijstaande woning |
| theta_e | -9.0 °C (basis -10 + 1K correctie zware bouw, tau=99.7h) |
| Ventilatie | Systeem C, continu bedrijf |
| Thermische bruggen | 0.05 W/(m²·K) (nieuw gebouw) |
| Zone aansluitvermogen | 9160 W (kwadratische sommatie) |
| Status `expected.json` | ⚠️ compleet, maar **informatief — geen referentie** (normversie-mismatch) |

## Scope

Volledige room-by-room cross-validatie met Vabi-rapport: phi_t, phi_v, phi_hu, phi_hl_i per kamer + zone-totaal.

## Status: informatief, geen referentie

Het Vabi-rapport is gerekend volgens **ISSO 51:2017**; de engine rekent
**ISSO 51:2023 incl. erratum**. Vastgesteld 2026-06-11:

- per vertrek: engine +4–8% t.o.v. Vabi (normversie-verschil);
- gebouwtotaal: engine −20% t.o.v. Vabi 9160 W (andere sommatiemethode —
  2023-erratum kwadratische sommatie vs. 2017-aanpak).

Beide afwijkingen zijn **normversie-verschillen, geen rekenfouten**. Het
project blijft in de repo (en in de Help → Verificatie-UI) ter illustratie,
zonder pass/fail-verdicts. De Rust-fixture staat op `#[ignore]`.

## Bekende afwijkingen

Zie hierboven — de eerdere claim "Geen — werkt 1-op-1 binnen 2%" gold voor
een oudere engine-versie en is achterhaald sinds de engine op ISSO 51:2023
(incl. erratum) rekent.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth
- Tests: `crates/isso51-core/tests/integration_test.rs::fixture_vabi_vrijstaande_woning` (`#[ignore]` — ISSO 51:2017 fixture, engine ondersteunt alleen 51:2023)
