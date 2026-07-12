# Aalten 2522 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2522 Woning Aalten**. Certified met **Uniec 3.3.2.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2522-woning-aalten.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 1 PV, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).

| Indicator | Certified | Limiet |
|---|---|---|
| BENG 1 | 103,69 | ≤ 123,39 |
| BENG 2 | 24,71 | ≤ 30,0 |
| BENG 3 | 85,0% | ≥ 50 |
| Label | A+++ | — |

Sub-totalen (primair, kWh): verwarming 2551 · tapwater 1813 · koeling 422 · ventilatoren 443 · PV-opbrengst 3811 · koelbehoefte 873.

Toleranties: BENG 1 ±6%, BENG 2 ±10% (ruimer dan Gouda: lagere absolute BENG 2 ⇒ hogere relatieve gevoeligheid), BENG 3 ±3 pp. Zie `../README.md` voor de gedeelde kanttekening.

## Meting F3d-3 (compute_beng vs certified) — 🔴 buiten tolerantie

De golden `uniec_aalten_2522` is end-to-end aangesloten (`oes_to_projectv2` → `compute_beng`) maar blijft `#[ignore]`: de engine haalt de tolerantie niet. Gemeten (A_g 67,0, A_ls 177,6, vormfactor 2,65):

| Indicator | Berekend | Certified | Δ | Tol | Binnen? |
|---|---|---|---|---|---|
| BENG 1 | 73,33 | 103,69 | −29,3% | ±6% | ✗ |
| BENG 2 | 67,84 | 24,71 | +174,6% | ±10% | ✗ |
| BENG 3 | 42,32% | 85,0% | −42,7 pp | ±3 pp | ✗ |
| Label | A+ | A+++ | −2 klassen | — | ✗ |

Sub-totalen (primair kWh, berekend vs certified): verwarming 1343 vs 2551 (−47%) · tapwater 1683 vs 1813 (−7%) · koeling 876 vs 422 (+108%) · ventilatoren 644 vs 443 (+45%) · PV 0 vs 3811.

### Bekende engine-gaps (op gemeten impact)

1. **PV ≈ 0 (dominant voor BENG 2/3).** De bron zet `solarPV[0].orientation = "N"` (noord, 4,1 kWp, tilt 15°); de forfaitaire azimuth-factor `cos((γ−180)/2)` levert daar ~0, terwijl de certified PV-opbrengst 3811 kWh (≈ 930 kWh/kWp, zuid-niveau) is. **Bron-inconsistentie**: het oes-`orientation`-veld strookt niet met de certified opbrengst. Zonder PV mist BENG 2 de volledige −1,45×3811 MJ saldering en blijft BENG 3 op ~42%. Invoer NIET aangepast (anti-fudge); dit is een fixture-provenance-gap, op te lossen door de PV-oriëntatie tegen het originele certificaat te verifiëren.
2. **Koeling +108%.** `Q_C;nd` met `F_sh = 1,0` (whole-zone, geen zomerzonwering-reductie) overschat de koudebehoefte; bekende F3d-benadering (zie `beng/mod.rs` module-doc + `no_active_cooling`-note).
3. **Verwarming −47%.** De nta8800-view propageert de `thermalBridges` (3 lineaire bruggen in de bron) niet naar `thermal_bridges_linear`, én de gemeten `airTightness.qv10 = 0,4` (zeer luchtdicht — maar níet injecteerbaar: geen ProjectV2-veld) valt terug op het tabel-11.13-leakage-forfait. Beide verlagen H_T/H_ve → lagere Q_H;nd. Bij Aalten (kleine, compacte woning) weegt dit relatief zwaar.

Verruiming van de tolerantie is verboden zonder normanalyse; activering volgt zodra de PV-azimuth-keten + F_sh-koeling + koudebrug-propagatie zijn geadresseerd.
