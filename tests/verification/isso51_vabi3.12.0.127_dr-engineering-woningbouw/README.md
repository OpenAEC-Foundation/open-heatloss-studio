# ISSO 51:2024 — Vabi 3.12.0.127 — DR Engineering Woningbouw

| Veld | Waarde |
|---|---|
| Norm | ISSO 51:2024 (incl. erratum 2023) |
| Software | Vabi Elements 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-woningbouw-isso51-2024.pdf` |
| Gebouw | Vrijstaande woning met garage, 14 vertrekken |
| theta_e | -8.0 °C (basis -10 + 2K tijdconstantecorrectie) |
| Verwarming | Radiatoren LT |
| Ventilatie | Systeem D met WTW |
| Gebouwtotaal | 6700 W (kwadratische sommatie) |
| Status `expected.json` | ✅ compleet |

## Scope

Cross-validatie ISSO 51:2024 + erratum 2023 (kwadratische zone-sommatie i.p.v. lineair).

## Bekende afwijkingen

- `build_summary` in `lib.rs` gebruikt nog lineaire som op gebouwniveau — connection_capacity test verwacht ~6700 W maar engine geeft ~8121 W. Niet maskeren met tolerantie-verbreding.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth (Vabi 3.12.0.127 layout met transmission split)
- Tests: `crates/isso51-core/tests/integration_test.rs::fixture_dr_engineering_woningbouw`
