# Gouda 2467 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2467 Goejanverwelledijk 85 Gouda**. Certified met **Uniec 3.3.3.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2467-goejanverwelledijk-gouda.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 2 PV-velden, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).
- **Gevel-georiënteerde BENG-geometrie (F6 fase 2b):** `beng_geometry.input.json` — buiten-oppervlak per gevel/dak op rekenzone-niveau (7 begrenzingsvlakken: vloer-op-kruipruimte + 4 gevels O/W/Z/N + 2 daken O/W), 1-op-1 uit de certified Uniec-capture. Bronnen: `uniec_fields_capture.json` (21 pagina's, alleen-lezen Playwright-walk) + `uniec_fields_capture_retry.json` (her-capture Achtergevel W + Gevel Rechts N mét loose-inputs/CONSTRD_OPP). Provenance + benaderingen (belemmering-V1, screens) staan in het `_meta`-blok van de fixture.

| Indicator | Certified | Limiet |
|---|---|---|
| BENG 1 | 95,86 | ≤ 96,4 |
| BENG 2 | 27,48 | ≤ 30,0 |
| BENG 3 | 83,7% | ≥ 50 |
| Label | A+++ | — |

Sub-totalen (primair, kWh): verwarming 6506 · tapwater 4208 · koeling 244 · ventilatoren 822 · PV-opbrengst 8734 · koelbehoefte 504.

Toleranties: BENG 1 ±6%, BENG 2 ±8%, BENG 3 ±3 pp. Zie `../README.md` voor de gedeelde kanttekening (geometrie is benadering, regressie-golden, WTW-pad ongedekt).

## Meting F3d-4 (compute_beng vs certified) — 🔴 buiten tolerantie

De golden `uniec_gouda_2467` is end-to-end aangesloten (`oes_to_projectv2` → `compute_beng`) maar blijft `#[ignore]`: de engine haalt de tolerantie niet. Gemeten na de F3d-4-fixes (PV-azimut via Tabel 17.2 + koudebrug-propagatie), A_g 133,1, A_ls 286,0, vormfactor 2,15:

| Indicator | F3d-3 | F3d-4 | Certified | Δ (F3d-4) | Tol | Binnen? |
|---|---|---|---|---|---|---|
| BENG 1 | 57,55 | 60,09 | 95,86 | −37,3% | ±6% | ✗ |
| BENG 2 | 53,41 | −8,20 | 27,48 | over-salderend | ±8% | ✗ |
| BENG 3 | 43,52% | 100,0% | 83,7% | +16,3 pp | ±3 pp | ✗ |
| Label | A++ | A++++ | A+++ | +1 klasse | — | ✗ |

Sub-totalen (primair kWh, F3d-4 vs certified): verwarming 2935 vs 6506 (−55%) · tapwater 2620 vs 4208 (−38%) · koeling 1479 vs 244 (+506%) · ventilatoren 1252 vs 822 (+52%) · PV −9377 (salderend) vs 8734 opbrengst.

### Gefixt in F3d-4

1. **PV-azimut (was dominant voor BENG 2/3).** De 7,2 kWp West + 1,2 kWp Oost leveren nu op. De oude `cos((γ−180)/2)`-benadering klemde west (−90° na `map_pv`) door `.max(0.0)` op 0; de norm kent hier géén correctiefactor maar leest `I_sol` per (β, γ) uit **Tabel 17.2** (zie `docs/2026-07-12-f3d4-norm-analyse-pv.md`). West-30°/oost-30° volgen nu de tabel.
2. **Koudebruggen.** De 3 `thermalBridges` (Σψ·L = 0,05·34 + 0,05·34 + 0,03·65 = 5,35 W/K) worden nu naar H_T gepropageerd (verwarming 2724 → 2935 kWh).

### Resterende gaps (op gemeten impact)

1. **PV-saldering = normversie-verschil (F3d-8), GEEN engine-bug.** BENG 2 → −8,2 en BENG 3 → 100% doordat NTA 8800:**2025+C1** (§5.5.2, formule 5.10) de PV-export **volledig** salderert tegen `fP;exp;el = 1,45` (EPTot mág negatief, OPMERKING 11 p. 87). De engine is hierin **norm-conform**. De certified Uniec 3.3.x crediteert daarentegen maar **~64 % van de PV** (5601/8734 kWh el = zelfgebruik via maand-directgebruik-fractie, ouder-norm/bijlage-AB-model) → +27,48 i.p.v. −6,6. Onder 2025+C1 zijn jaarbasis en maandmatching **identiek** (identiteit-bewijs in `docs/2026-07-12-f3d8-norm-analyse-saldering.md` §3), dus "maandmatching implementeren" lost dit **niet** op. Anti-fudge: EP-crate blijft ongewijzigd; deze BENG 2-golden is voor een all-electric hoog-PV-woning **niet** valideerbaar tegen 2025+C1.
2. **Koeling +506%.** `Q_C;nd` met `F_sh = 1,0` (whole-zone, geen zomerzonwering) overschat de koudebehoefte — bekende F3d-benadering, buiten scope.
3. **Q_H;nd structureel te laag (BENG 1 −37%).** Koudebruggen halveren de gap niet; het residu zit in het demand-model. De bron-`qv10 = 0,98` is sinds **F3d-9 wél injecteerbaar** (`q_v10_spec_dm3_s_m2`, NTA 8800 §11.2.5) — maar hier gelijk aan het forfait voor deze vrijstaande woning (bouwjaar 2020: f_type 1,4 · f_y 0,7 · q_spec 1,0 = 0,98), dus BENG 1 verandert niet. Conclusie: de infiltratie-invoer is **niet** de oorzaak van de Q_H;nd-onderschatting.

Verruiming van de tolerantie is verboden zonder normanalyse; activering volgt zodra de PV-saldering (maandmatching), F_sh-koeling en de Q_H;nd-onderschatting zijn geadresseerd.
