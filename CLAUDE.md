# ISSO 51 Warmteverliesberekening

Rekenbibliotheek voor warmteverliesberekeningen volgens ISSO 51:2023.

---

## Doel

Complete tool voor warmteverliesberekeningen volgens de ISSO 51 norm, bruikbaar als:
- Rust rekenbibliotheek (core engine)
- Python package (via PyO3)
- DLL (C ABI)
- WASM module (web browser)
- Web app (React + TypeScript)
- Desktop app (Tauri + React)
- REST API

---

## Architectuur

- **isso51-core** (Rust): Alle formules & tabellen, JSON in/uit, puur (geen I/O, geen async, geen unsafe)
- Wrapper crates: isso51-python (PyO3), isso51-wasm (wasm-bindgen), isso51-ffi (cbindgen)
- Frontend: React + TypeScript + Tailwind + Zustand
- Desktop: Tauri v2

---

## Belangrijke Bestanden

| Bestand | Doel |
|---------|------|
| `crates/isso51-core/src/lib.rs` | Public API: `calculate_from_json()` |
| `crates/isso51-core/src/model/` | Domeinmodel (structs/enums) |
| `crates/isso51-core/src/formulas.rs` | Formule-identifiers (29 constanten) |
| `crates/isso51-core/src/calc/` | Berekeningen per onderdeel |
| `crates/isso51-core/src/tables/` | ISSO 51 opzoektabellen |
| `schemas/v1/` | JSON schemas (gegenereerd uit Rust types) |
| `tests/fixtures/` | Test JSON bestanden |

---

## Conventies

- Rust: `cargo test` moet altijd slagen
- Eenheden: mm voor afmetingen, dm3/s voor luchtvolumestroom, W voor vermogen, W/K voor H-waarden
- Temperaturen in graden Celsius
- JSON schema first: types in Rust, schemas gegenereerd via schemars
- Doc comments verwijzen naar ISSO 51 formulenummers

## Referenties

- ISSO 51 voorbeeld portiekwoning: gebruikt theta_b = 15 graden C (oud), erratum 2023 zegt 17 graden C
- Erratum 2023: kwadratische sommatie voor niet-gelijktijdige verliezen
- Factor 1.2 kJ/(m3*K) = rho * c_p lucht
- qi_spec in dm3/s per m2 (niet m3/s)

---

## Orchestrator — Sessie afsluiting

**ALTIJD uitvoeren aan het einde van elke sessie** (of na een significante mijlpaal):

Schrijf een update naar:
`C:\Users\JochemK\.claude\orchestrator\sessions\warmteverlies_latest.md`

Gebruik dit formaat:
```markdown
# Warmteverlies — Sessie update
**Datum:** YYYY-MM-DD HH:MM
**Branch:** (git branch naam)

## Wat is gedaan
- (bullet per afgeronde taak)

## Huidige staat
(1-3 zinnen over de staat van het project)

## Gewijzigde bestanden
- (relevante paden)

## Openstaande issues / next steps
- (wat nog moet gebeuren)

## Cross-project notities
(iets wat relevant is voor pyrevit of report integratie)
```

**Orchestrator context:** `C:\Users\JochemK\.claude\orchestrator\context\warmteverlies.md`
**Project registry:** `C:\Users\JochemK\.claude\orchestrator\project-registry.json`
