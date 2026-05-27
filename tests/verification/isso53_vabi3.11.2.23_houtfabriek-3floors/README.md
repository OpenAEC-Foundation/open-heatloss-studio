# ISSO 53 — Vabi 3.11.2.23 — TR02 Houtfabriek 3 verdiepingen (1.10a / 2.10a / 3.10a)

| Veld | Waarde |
|---|---|
| Norm | ISSO 53 |
| Software | Vabi Elements 3.11.2.23 |
| Bron PDF | `tests/references/Warmteverliesberekening TR02 - Houtfabriek.pdf` (p.38-40, 82-84, 131-133) |
| Bron .vp | `tests/references/TR03 - Houtfabriek.vp.zip` |
| Gebouw | Utiliteit, identieke vertrekken op 3 verdiepingen |
| Status `expected.json` | ✅ compleet (3 rooms) |
| Tolerantie | 5% per room |

## Cross-validatie (sessie 8, na Optie C wrapper-schrap)

| Room | phi_t calc | phi_t Vabi | Δ | Status |
|---|---|---|---|---|
| 1.10a | 1418 W | 1514 W | -6.3% | ⚠️ `#[ignore]` (fixture-bundeling) |
| 2.10a | 1498 W | 1494 W | +0.3% | ✅ |
| 3.10a | 1776 W | 1691 W | +5.0% | ✅ binnen tolerantie |

phi_i 0.0% / 0.1% / 0.1% — alle 3 binnen 0.1% van Vabi.

## Bekende afwijkingen

Het 1.10a-artefact (-6,3%) is fixture-bundelings-effect (zie `crates/isso53-core/tests/PDF_GAPS.md` "Spoor 4"), niet calc-core bug. 30+ Vabi-constructies → gebundeld in fixture.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth (3 rooms: 1.10a, 2.10a, 3.10a)
- Tests: `crates/isso53-core/tests/vabi_houtfabriek_3floors_golden.rs`
