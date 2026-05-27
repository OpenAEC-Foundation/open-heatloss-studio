# ISSO 53 — Vabi 3.12.0.127 — DR Engineering Kantoor West 0.03

| Veld | Waarde |
|---|---|
| Norm | ISSO 53 |
| Software | Vabi Elements 3.12.0.127 |
| Bron PDF | `tests/references/dr-engineering-woningbouw-isso51-2024.pdf` (DR Engineering, "Voorbeeld Warmteverliesberekening Utiliteitsbouw", 27-2-2025) |
| Gebouw | Kantoor utiliteit, ruimte 0.03 Kantoor West |
| Verwarming | Luchtverwarming (toevoertemp 21.5 °C) |
| Bijzonderheden | Unknown-pad infiltratie (Vabi-compat via NEN 8088-1) |
| Status `expected.json` | ✅ compleet |

## Cross-validatie (sessie 8)

| Component | Calc | Vabi | Δ | Status |
|---|---|---|---|---|
| phi_v | 0 W | 0 W | exact | ✅ |
| phi_t | 3165 W | 3059 W | +3.5% | ✅ heractiveerd na Optie C |
| phi_i | 693 W | 681 W | +1.8% | ✅ Unknown-pad Vabi-compat |
| Totaal | 3858 W | 3741 W | +3.1% | ✅ |

## Bekende afwijkingen

- f_ig auto-berekening (§4.6) was historische bug — opgelost sessie 2026-05-24.
- Unknown-pad infiltratie vereist `InfiltrationMethod::UnknownVabiCompat` (NEN 8088-1 Tabel 9/10 + NTA 8800 Tabel 11.13). Norm-strikt pad geeft Φ_I = 177 W.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth voor room 0.03 + `_calc_snapshot` voor regressie-detectie
- Tests: `crates/isso53-core/tests/vabi_dr_golden.rs`
