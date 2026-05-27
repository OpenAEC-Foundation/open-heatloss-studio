# Verification — voorbeeldprojecten voor norm-validatie

Eén centrale folder met **gevalideerde voorbeeldprojecten** voor alle 3 normen (ISSO 51, ISSO 53, TO-juli/NTA 8800 cooling). Elke subfolder bevat een complete case: input, verwachte uitkomst, bronrapport.

Doel:
- **Automatische verificatie** — Rust-tests in `crates/*/tests/` lezen `input.json` + `expected.json` en falen als de berekening afwijkt buiten tolerantie.
- **UI-controle** — `input.json` is direct te openen via "Project openen…" in heatloss-studio; resultaten in de UI moeten matchen met `expected.json`.

Bestaande regressie-only fixtures zonder norm-truth (`portiekwoning`, `woonboot`, placeholders in `crates/isso53-core/tests/fixtures/voorbeeld_6{1,2}`, `en12831_two_storey_house`) blijven in hun huidige locaties (`tests/fixtures/`, `crates/*/tests/fixtures/`) — die horen niet in deze folder thuis.

---

## Naamconventie

`{norm}_{software-versie}_{projectslug}/`

| Token | Voorbeelden |
|---|---|
| `norm` | `isso51`, `isso53`, `tojuli`, `koellast` |
| `software-versie` | `vabi3.8.1.14`, `vabi3.11.2.23`, `vabi3.12.0.127` (geen spaties, punten OK) |
| `projectslug` | kebab-case, kort, uniek binnen norm |

Voorbeeld: `isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4`

---

## Per-project structuur

| Bestand | Doel | Verplicht |
|---|---|---|
| `input.json` | Project-input in heatloss-studio schema (opent in UI én is fixture-input voor Rust-tests) | ✅ |
| `expected.json` | Verwachte uitkomsten per ruimte + gebouw (Vabi-rapport-truth of norm-PDF) | ✅ |
| `reference.pdf` | Bronrapport (Vabi, DR Engineering, norm-publicatie) | aanbevolen |
| `README.md` | Bron, versie, scope, bekende afwijkingen, status | ✅ |
| `reference.vp.zip` | Originele Vabi-projectfile indien beschikbaar | optioneel |

---

## Status-overzicht

| Subfolder | Norm | Software | Status `expected.json` | Cross-validatie | Migratie |
|---|---|---|---|---|---|
| `isso51_vabi3.8.1.14_vrijstaande-woning` | ISSO 51:2017 | Vabi 3.8.1.14 | ✅ compleet | ✅ rooms + zone 9160 W | ✅ gemigreerd |
| `isso51_vabi3.12.0.127_dr-engineering-woningbouw` | ISSO 51:2024 | Vabi 3.12.0.127 | ✅ compleet | ✅ erratum 2023 kwadratisch ~6700 W | ✅ gemigreerd |
| `isso51_vabi3.9.1.2_woonhuis-A` | ISSO 51:2017 | Vabi 3.9.1.2 | ❌ PDF aanwezig, fixture leeg | — (16 rooms, vloerverwarming) | — |
| `isso53_vabi3.11.2.23_houtfabriek-bedrijfsruimte4` | ISSO 53 | Vabi 3.11.2.23 | ✅ compleet | ✅ Δ +0.7% totaal | ✅ gemigreerd |
| `isso53_vabi3.11.2.23_houtfabriek-3floors` | ISSO 53 | Vabi 3.11.2.23 | ✅ compleet (3 rooms) | ✅ 2.10a +0.3%, 3.10a +5.0%, 1.10a `#[ignore]` | ✅ gemigreerd |
| `isso53_vabi3.12.0.127_dr-engineering-kantoorwest` | ISSO 53 | Vabi 3.12.0.127 | ✅ compleet | ✅ Φ_T +3.5%, Φ_I +1.8% | ✅ gemigreerd |
| `koellast_vabi3.12.0.127_dr-engineering-woningbouw` | Peak koellast (EN 12831 / NEN 5060 TO2) | Vabi 3.12.0.127 | ✅ peak W ingevuld (engine TBD) | — (engine ontbreekt nog) | ✅ gemigreerd uit tojuli folder |
| `tojuli_vabi3.12.0.127_dr-engineering-woningbouw` | TO-juli (NTA 8800 cooling) | Vabi 3.12.0.127 | 🟡 placeholder (wacht op Vabi BENG-PDF aanvraag bij installateur) | — | — |
| `tojuli_vabi3.12.0.127_dr-engineering-utiliteitsbouw` | TO-juli (NTA 8800 cooling) | Vabi 3.12.0.127 | ❌ nieuw, PDF aanwezig | — | — |

Toekomstige uitbreidingen (bron-referenties liggen lokaal in `tests/references/`, niet getrackt):
- `isso53_vabi3.12.0.127_utiliteit-A-all-air` — All Air ventilatie
- `isso53_vabi3.12.0.127_utiliteit-A-basis-ventilatie` — Basis ventilatie
- `isso53_vabi3.12.0.127_utiliteit-A-nachtkoeling` — Basis ventilatie + nachts actief koelen

---

## Workflow per project

1. **Input opbouwen** — vanuit Vabi-rapport (of UI) een `input.json` produceren in heatloss-studio schema.
2. **Verwachte waardes vastleggen** — `expected.json` invullen op basis van Vabi/PDF, met tolerantie per veld.
3. **Auto-test koppelen** — Rust-test in `crates/<engine>/tests/` wijst naar deze pad. Eén bron, geen duplicatie.
4. **UI-verificatie** — `input.json` openen, doorklikken per ruimte, getallen vergelijken met `expected.json`.

---

## Verbinding met Rust-tests

Tests in:
- `crates/isso51-core/tests/integration_test.rs` — `FixtureSource::Verification { subfolder }` voor de Vabi paren; `FixtureSource::Legacy` voor regressie-only (portiekwoning, woonboot)
- `crates/isso53-core/tests/{vabi_golden,vabi_dr_golden,vabi_houtfabriek_3floors_golden}.rs` — `include_str!("../../../tests/verification/<subfolder>/{input,expected}.json")`
- `crates/isso53-core/tests/golden.rs` — leest nog `crates/isso53-core/tests/fixtures/voorbeeld_6{1,2}_*.json` (placeholders zonder norm-truth, niet migreren)
- `crates/nta8800-cooling/tests/vabi_koellast_golden.rs` — verwijst naar `tests/verification/koellast_vabi3.12.0.127_dr-engineering-woningbouw/expected.json`; alle echte tests `#[ignore]` tot peak-cooling engine bestaat (huidige `nta8800-cooling` doet alleen NTA 8800 H.10 annual)

---

## Verbinding met UI

`input.json` is een normaal heatloss-studio projectbestand — direct te openen via **Project openen…** in de Tauri-app. Resultaten in de UI moeten matchen met `expected.json`.
