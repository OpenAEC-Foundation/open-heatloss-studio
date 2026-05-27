# ISSO 53 — Vabi 3.11.2.23 — TR02 Houtfabriek Bedrijfsruimte 4

| Veld | Waarde |
|---|---|
| Norm | ISSO 53 |
| Software | Vabi Elements 3.11.2.23 |
| Bron PDF | `tests/references/Warmteverliesberekening TR02 - Houtfabriek.pdf` (p.18-20) |
| Bron .vp | `tests/references/TR03 - Houtfabriek.vp.zip` (TR03, niet TR02 — checken) |
| Gebouw | Utiliteit, industriefunctie (gemodelleerd als kantoor + verblijfsgebied) |
| Constructies | 30+ in Vabi, gebundeld tot ~6 in fixture |
| Verwarming | Luchtverwarming (toevoertemp 21°C) |
| Ventilatie | Systeem D + WTW + vorstbeveiliging |
| Status `expected.json` | ✅ compleet — Δ +0.7% totaal |

## Cross-validatie (na 4 fixes sessie 2)

| Component | Calc | Vabi | Δ |
|---|---|---|---|
| phi_t | 2918 W | 2919 W | -0.03% |
| phi_v | 0 W | 0 W | exact (luchtverwarming f_v=0) |
| phi_i | 3134 W | 3080 W | +1.8% |
| phi_hu | 2163 W | 2163 W | 0.0% |
| **Totaal** | **8215 W** | **8161 W** | **+0.7%** |

## Bekende afwijkingen

- 1.10a en 3.10a structureel −6,2% en +5,0% door fixture-bundeling (30+ Vabi-constructies → ~6 fixture-elementen). Documented in `crates/isso53-core/tests/PDF_GAPS.md`.

## Files

- `input.json` — heatloss-studio project (open in UI of via Rust-test)
- `expected.json` — Vabi-rapport truth
- Tests: `crates/isso53-core/tests/vabi_golden.rs`
