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

## Meting F3d-4 (compute_beng vs certified) — 🔴 buiten tolerantie

De golden `uniec_aalten_2522` is end-to-end aangesloten (`oes_to_projectv2` → `compute_beng`) maar blijft `#[ignore]`: de engine haalt de tolerantie niet. Gemeten na de F3d-4-fixes (PV-azimut via Tabel 17.2 + koudebrug-propagatie), A_g 67,0, A_ls 177,6, vormfactor 2,65:

| Indicator | F3d-3 | F3d-4 | Certified | Δ (F3d-4) | Tol | Binnen? |
|---|---|---|---|---|---|---|
| BENG 1 | 73,33 | 77,11 | 103,69 | −25,6% | ±6% | ✗ |
| BENG 2 | 67,84 | 8,21 | 24,71 | −66,8% | ±10% | ✗ |
| BENG 3 | 42,32% | 93,31% | 85,0% | +8,3 pp | ±3 pp | ✗ |
| Label | A+ | A++++ | A+++ | +1 klasse | — | ✗ |

Sub-totalen (primair kWh, F3d-4 vs certified): verwarming 1456 vs 2551 (−43%) · tapwater 1683 vs 1813 (−7%) · koeling 859 vs 422 (+104%) · ventilatoren 644 vs 443 (+45%) · PV −4091 (salderend) vs 3811 opbrengst.

### Gefixt in F3d-4

1. **PV-azimut.** De noord-string (4,1 kWp, tilt 15°) levert nu **~2570 kWh** i.p.v. 0 — de norm leest `I_sol` voor (β=15°, γ=N) uit Tabel 17.2 met lineaire helling-interpolatie (noord > 0). Zie `docs/2026-07-12-f3d4-norm-analyse-pv.md`.
2. **Koudebruggen.** De 3 `thermalBridges` (Σψ·L = 0,05·26 + 0,05·26 + 0,03·42 = 3,86 W/K) gaan nu naar H_T (verwarming 1343 → 1456 kWh).

### Resterende gaps (op gemeten impact)

1. **PV-saldering = normversie-verschil (F3d-8), GEEN engine-bug.** Net als Gouda: certified Uniec crediteert **~64,6 % van de PV** (2464/3811 kWh el = zelfgebruik, ouder-norm/bijlage-AB-model) → BENG 2 = 24,71. NTA 8800:**2025+C1** salderert **volledig** (§5.5.2) → BENG 2 = −4,4 (engine 8,21, rest = demand-gaps). Twee onafhankelijke cases die beide op ~64 % zelfgebruik uitkomen bevestigen het partieel-salderen van de certified tool. Anti-fudge: EP-crate ongewijzigd. Zie `docs/2026-07-12-f3d8-norm-analyse-saldering.md`.
2. **PV-noord = bron-inconsistentie (secundair).** De bron zet `orientation = "N"`; noord haalt fysisch ~2570 kWh, terwijl certified 3811 kWh (~930 kWh/kWp, zuid-niveau) claimt. Het oes-`orientation`-veld strookt niet met de certified opbrengst. Invoer NIET aangepast (anti-fudge) — fixture-provenance-gap, op te lossen door de PV-oriëntatie tegen het originele certificaat te verifiëren.
2. **Koeling +104%.** `Q_C;nd` met `F_sh = 1,0` overschat de koudebehoefte; bekende F3d-benadering, buiten scope.
3. **Q_H;nd te laag (BENG 1 −26%).** Naast koudebruggen (nu gefixt) blijft de gemeten `airTightness.qv10 = 0,4` niet injecteerbaar (geen ProjectV2-veld) → tabel-11.13-forfait; H_ve te hoog t.o.v. de zeer luchtdichte werkelijkheid trekt Q_H;nd niet genoeg omhoog. Bij deze compacte woning weegt dat zwaar.

Verruiming van de tolerantie is verboden zonder normanalyse; activering volgt zodra de PV-oriëntatie-provenance, F_sh-koeling en de Q_H;nd-onderschatting zijn geadresseerd.
