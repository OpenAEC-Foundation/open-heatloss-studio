# TO-juli (NTA 8800 cooling) — Vabi 3.12.0.127 — DR Engineering Woningbouw

| Veld | Waarde |
|---|---|
| Norm | TO-juli / NTA 8800 cooling |
| Software | Vabi Elements 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-koellast-woningbouw-2024.pdf` (432 KB) |
| Gebouw | Vrijstaande woning, Ag = 243.2 m² (volgens sessie-handoff; checken in PDF) |
| Ventilatie | Systeem D + WTW |
| Status `expected.json` | ❌ **placeholder — _TODO velden** |

## Open werk

Bestaande `vabi_tojuli_woning_120m2_expected.json` is volledig placeholder met `_TODO` markers en dummy-waardes (2500 MJ etc.). Stappen:

1. **PDF extraheren** — handmatig de Vabi TO-juli output uit `dr-engineering-koellast-woningbouw-2024.pdf` halen:
   - jaar-KPIs: Q_C;use [MJ + kWh], H_T [W/K], H_V [W/K], τ [h]
   - maandwaardes: Q_C;nd, Q_C;use, Q_H;nd, θ_e per maand (12 × 4 arrays)
   - tolerances per veld
2. `input.json` afleiden — vermoedelijk cross-link met de bestaande heating-fixture (zelfde gebouw, andere norm-pad)
3. Auto-test heractiveren — `crates/nta8800-cooling/tests/vabi_tojuli_golden.rs`

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — placeholder (`_TODO`-markers + dummy waardes), te vullen uit `dr-engineering-koellast-woningbouw-2024.pdf`
- Tests: `crates/nta8800-cooling/tests/vabi_tojuli_golden.rs` — `vabi_tojuli_woning_120m2_matches` op `#[ignore]` tot expected.json gevuld

## Naamgeving open vraag

Bestaande fixture heet `vabi_tojuli_woning_120m2_expected.json`, maar volgens sessie 11 is `Ag = 243.2 m²` — naam klopt mogelijk niet meer. Bij migratie hernoemen naar `dr-engineering-woningbouw` voor consistentie.
