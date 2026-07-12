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

🔴 **Rood/`#[ignore]`** — de goldens zijn in F3d-3 **end-to-end aangesloten** (`oes_to_projectv2` → `compute_beng` in `crates/openaec-project-shared/tests/beng_golden.rs`) met de tolerantie-asserts uit `expected.json`, maar blijven `#[ignore]`: beide cases vallen ver buiten tolerantie. Per-case meting + gap-analyse staan in `gouda-2467/README.md` en `aalten-2522/README.md`.

**Top-3 engine-gaps (op gemeten impact, beide cases):**
1. **PV** — Gouda: west-string op ~0 door de `map_pv`-azimuthnormalisatie (270°→−90°) i.c.m. de `cos((γ−180)/2)`-clamp in `nta8800-pv` (engine-inconsistentie ±180 vs 0-360). Aalten: bron-`orientation="N"` strookt niet met de certified opbrengst (3811 kWh). Dominant voor BENG 2/3.
2. **Koeling** — `Q_C;nd` met `F_sh = 1,0` (whole-zone) overschat de koudebehoefte (+108% Aalten, +517% Gouda).
3. **Verwarming** — koudebruggen (`thermalBridges`) niet gepropageerd + gemeten `qv10` niet injecteerbaar → H_T/H_ve te laag → Q_H;nd −47…−58%.

De sub-totalen (`heating_primary_kwh` etc. in `expected.json.expected`) waren vooraf al vastgelegd; ze maakten deze per-dienst-diagnose mogelijk (welke crate afwijkt, niet alleen de eindindicator). De diagnostische meting draai je met `cargo test -p openaec-project-shared --test beng_golden uniec_measure -- --ignored --nocapture`. Anti-fudge: `expected.json` en de toleranties zijn niet aangepast.
