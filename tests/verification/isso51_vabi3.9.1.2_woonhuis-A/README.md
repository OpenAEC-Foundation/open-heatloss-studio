# ISSO 51:2017 — Vabi 3.9.1.2 — Woonhuis A

| Veld | Waarde |
|---|---|
| Norm | ISSO 51:2017 (incl. 53/57) |
| Software | Vabi Elements 3.9.1.2 |
| Bron PDF | `tests/references/vabi-woonhuis-A-isso51-2017.pdf` (gitignored, lokaal) |
| Samenvatting | `tests/references/vabi-woonhuis-A-samenvatting.md` |
| Gebouw | Vrijstaande woning, 16 vertrekken |
| Verwarming | Vloerverwarming |
| Ventilatie | Systeem C |
| Vertrekken-totaal | 10784 W |
| Aansluitvermogen | 12564 W |
| Status `expected.json` | ❌ **nog niet ingevuld** |

## Scope

Goede uitbreiding op de andere ISSO 51-fixtures: 16 rooms (vs 7 in vrijstaande_woning, 14 in DR), én vloerverwarming activeert §4.6 embedded heating path die de andere niet raken.

## Open werk

1. `input.json` opbouwen uit Vabi-rapport (room-geometrie, constructies, vloerverwarming-flag, ventilatie-systeem C)
2. `expected.json` invullen vanuit PDF (per kamer phi_t, phi_v, phi_hu, phi_hl_i + zone 10784/12564)
3. Auto-test toevoegen in `crates/isso51-core/tests/integration_test.rs`
4. UI-handcheck: openen in heatloss-studio, doorklikken alle 16 rooms

## Referentiemateriaal aanwezig

- `tests/references/vabi-woonhuis-A-isso51-2017.pdf` (gitignored, lokaal)
- `tests/references/vabi-woonhuis-A-samenvatting.md` (getrackt — anoniem)
