# BENG Uniec-crosscheck — certified replay-goldens (rood/`#[ignore]`)

Onafhankelijke BENG-cross-check tegen **certified Uniec 3.3.x**-berekeningen, als tweede validatielaag naast de RVO-eindwaarden. Deterministische invoer (het volledige `project{}`-blok is engine-compleet) → diagnostisch sterker dan alleen eindwaarden, want ook sub-totalen per dienst (verwarming/tapwater/koeling/ventilatoren/PV) zijn beschikbaar.

## Bron

- **Repo:** open-energy-studio van **John Heikens**, licentie **LGPL-3.0**.
- **Bestanden:** `training-data/2467-goejanverwelledijk-gouda.oes.json` en `training-data/2522-woning-aalten.oes.json`, elk gekopieerd naar `{case}/input.oes.json`.
- **Certified referentie:** `meta.uniecReference` per bestand (Uniec 3.3.3.1 resp. 3.3.2.1 / BengCert).

## Cases

| Case | Certified tool | BENG 1 | BENG 2 | BENG 3 | Label | Toleranties (B1/B2/B3) |
|---|---|---|---|---|---|---|
| `gouda-2467` | Uniec 3.3.3.1 | 95,86 (≤96,4) | 27,48 (≤30) | 83,7% (≥50) | A+++ | ±6% / ±8% / ±3pp |
| `aalten-2522` | Uniec 3.3.2.1 | 103,69 (≤123,39) | 24,71 (≤30) | 85,0% (≥50) | A+++ | ±6% / ±10% / ±3pp |

Certified waarden zijn EXACT overgenomen uit `meta.uniecReference` (JSON-pad in `expected.json.provenance`). Toleranties zijn gekalibreerd op Johns eigen vitest-banden (open-energy-studio `validate-beng.ts`).

## Kanttekening

De geometrie in het `project{}`-blok is een **benadering** van de originele certificaten (residueel 1-5% t.o.v. certified). Dit is daarom een **regressie-golden, geen certificerings-referentie**. Het WTW-pad is in alle drie Johns projecten ongedekt (η_wtw = 0). Kijkduin (2786, utiliteit) is bewust nog niet als golden opgenomen — pas na het utiliteit/verlichting-pad (plan §F5).

## Status

🔴 **Rood/`#[ignore]`** — activeren in fase F3 zodra `compute_beng(ProjectV2)` bestaat. Harnas: `crates/openaec-project-shared/tests/beng_golden.rs`.

De sub-totalen (`heating_primary_kwh` etc. in `expected.json.expected`) zijn nu al vastgelegd zodat F3 per-dienst kan diagnosticeren welke crate afwijkt, niet alleen de eindindicator.
