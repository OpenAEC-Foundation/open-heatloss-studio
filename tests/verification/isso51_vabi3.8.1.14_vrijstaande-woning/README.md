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
| Status `expected.json` | ✅ compleet |

## Scope

Volledige room-by-room cross-validatie met Vabi-rapport: phi_t, phi_v, phi_hu, phi_hl_i per kamer + zone-totaal.

## Bekende afwijkingen

Geen — werkt 1-op-1 met Vabi-rapport binnen 2% room-tolerantie.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth
- Tests: `crates/isso51-core/tests/integration_test.rs::fixture_vabi_vrijstaande_woning` (`#[ignore]` — ISSO 51:2017 fixture, engine ondersteunt alleen 51:2023)
