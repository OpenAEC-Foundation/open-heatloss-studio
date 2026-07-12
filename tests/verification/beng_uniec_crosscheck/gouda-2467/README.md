# Gouda 2467 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2467 Goejanverwelledijk 85 Gouda**. Certified met **Uniec 3.3.3.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2467-goejanverwelledijk-gouda.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 2 PV-velden, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).

| Indicator | Certified | Limiet |
|---|---|---|
| BENG 1 | 95,86 | ≤ 96,4 |
| BENG 2 | 27,48 | ≤ 30,0 |
| BENG 3 | 83,7% | ≥ 50 |
| Label | A+++ | — |

Sub-totalen (primair, kWh): verwarming 6506 · tapwater 4208 · koeling 244 · ventilatoren 822 · PV-opbrengst 8734 · koelbehoefte 504.

Toleranties: BENG 1 ±6%, BENG 2 ±8%, BENG 3 ±3 pp. Zie `../README.md` voor de gedeelde kanttekening (geometrie is benadering, regressie-golden, WTW-pad ongedekt).

## Meting F3d-3 (compute_beng vs certified) — 🔴 buiten tolerantie

De golden `uniec_gouda_2467` is end-to-end aangesloten (`oes_to_projectv2` → `compute_beng`) maar blijft `#[ignore]`: de engine haalt de tolerantie niet. Gemeten (A_g 133,1, A_ls 286,0, vormfactor 2,15):

| Indicator | Berekend | Certified | Δ | Tol | Binnen? |
|---|---|---|---|---|---|
| BENG 1 | 57,55 | 95,86 | −40,0% | ±6% | ✗ |
| BENG 2 | 53,41 | 27,48 | +94,3% | ±8% | ✗ |
| BENG 3 | 43,52% | 83,7% | −40,2 pp | ±3 pp | ✗ |
| Label | A++ | A+++ | −1 klasse | — | ✗ |

Sub-totalen (primair kWh, berekend vs certified): verwarming 2724 vs 6506 (−58%) · tapwater 2620 vs 4208 (−38%) · koeling 1507 vs 244 (+517%) · ventilatoren 1252 vs 822 (+52%) · PV −997 (salderend) vs 8734 opbrengst.

### Bekende engine-gaps (op gemeten impact)

1. **PV-west valt op ~0 — engine-bug (dominant voor BENG 2/3).** De bron heeft 7,2 kWp West + 1,2 kWp Oost. `map_pv` (`beng/mapping.rs`) normaliseert de DTO-azimuth naar −180..180 (west 270° → −90°), maar de yield-formule `cos((γ−180)/2)` in `nta8800-pv` is geschreven voor de 0-360-conventie: bij −90° wordt de factor negatief en door `.max(0.0)` op 0 geklemd. Gevolg: de west-string levert **niets** (de berekende ~688 kWh is de oost-string alleen). Dit is een **engine-inconsistentie** tussen `PvSystem::validate_azimuth` (±180) en `calculate_tilt_azimuth_factor` (0-360), niet een invoerfout — de input voedt de gedocumenteerde DTO-conventie (0=noord…270=west). Fixen buiten deze ronde (scope: gaps documenteren).
2. **Koeling +517%.** `Q_C;nd` met `F_sh = 1,0` overschat de koudebehoefte fors (F3d-benadering, whole-zone, geen zomerzonwering). Grootste per-dienst-fout in absolute én relatieve zin na PV.
3. **Verwarming −58% / tapwater −38%.** Koudebruggen (`thermalBridges`, 3 stuks in de bron) worden niet gepropageerd (`thermal_bridges_linear = []` in de nta8800-view) en de gemeten `qv10 = 0,98` is niet injecteerbaar (geen ProjectV2-veld) → H_T/H_ve te laag → Q_H;nd te laag. Tapwater volgt het A_g-forfait; het residu wijst op een SCOP_W-/distributie-verschil t.o.v. Uniec.

Verruiming van de tolerantie is verboden zonder normanalyse; activering volgt zodra de PV-azimuth-keten, F_sh-koeling en koudebrug-propagatie zijn geadresseerd.
