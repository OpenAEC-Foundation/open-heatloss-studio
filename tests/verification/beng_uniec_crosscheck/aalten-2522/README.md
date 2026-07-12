# Aalten 2522 — certified Uniec BENG-crosscheck

Grondgebonden woning, **2522 Woning Aalten**. Certified met **Uniec 3.3.2.1 / BengCert**.

- **Invoer:** `input.oes.json` (kopie van `open-energy-studio/training-data/2522-woning-aalten.oes.json`, John Heikens, LGPL-3.0). Bevat het volledige `project{}`-blok (1 rekenzone, 1 verwarming, 1 ventilatie, 1 koeling, 1 tapwater, 1 PV, 3 constructies) + `meta`.
- **Expected:** `expected.json`, waarden EXACT uit `meta.uniecReference` (JSON-paden in `provenance`).
- **Uniec-invoercapture:** `uniec_fields_capture.json` — volledige velden-dump van alle 20 Uniec 3-invoerpagina's van deze case (alleen-lezen Playwright-walk). Geïnventariseerd + geanalyseerd in `docs/2026-07-12-uniec-velden-inventarisatie.md`.

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
3. **Q_H;nd te laag (BENG 1 −26%).** De gemeten `airTightness.qv10 = 0,40` is sinds **F3d-9 injecteerbaar** (`q_v10_spec_dm3_s_m2`, NTA 8800 §11.2.5). Ze ligt echter *onder* het forfait (0,98 voor deze vrijstaande woning) → minder lek, dus injectie **verlaagt** Q_H;nd juist licht (BENG 1 −25,6% → −26,0%; verwarming 1456 → 1446 kWh). De onderschatting komt dus **niet** van de infiltratie-invoer maar van het demand-model; de zeer luchtdichte schil maakt de residu-gap eerder groter dan kleiner.

Verruiming van de tolerantie is verboden zonder normanalyse; activering volgt zodra de PV-oriëntatie-provenance, F_sh-koeling en de Q_H;nd-onderschatting zijn geadresseerd.

## Meting F6 (compute_beng via de gevel-georiënteerde geometrie-brug) — 🟢 binnen tolerantie

De F6-brug (`beng/geometry_bridge.rs`) hangt de certified gevel-geometrie (`beng_geometry.input.json`, buiten-oppervlak per gevel) op hetzelfde oes-project — zelfde installaties, koudebruggen en luchtdichtheid, **alleen** de geometrie-bron wisselt van binnen- naar buiten-oppervlakten. Daarmee landen **BENG 1/2/3 binnen de certified tolerantie**; het ruimte-georiënteerde oes-pad bleef op −26 %/−67 %. Dit bevestigt de F6-hoofdthese: de Q_H;nd-onderschatting kwam van de binnen- i.p.v. buiten-oppervlakte-bron, niet van het rekenpad.

| Indicator | oes-binnen (F3d-9) | BENG-buiten (F6-brug) | Certified | Δ (F6) | Tol | Binnen? |
|---|---|---|---|---|---|---|
| BENG 1 | 76,73 | 102,84 | 103,69 | −0,8% | ±6% | ✓ |
| BENG 2 | 8,06 | 22,61 | 24,71 | −8,5% | ±10% | ✓ |
| BENG 3 | 93,40 | 83,57 | 85,0 | −1,4 pp | ±3 pp | ✓ |
| Label | A++++ | A++++ | A+++ | +1 klasse | — | ✗ |

Geometrie-kentallen: A_g 67,0 (ongewijzigd); A_ls 177,6 → **245,7** (buiten-schil: 4 gevels + dak + vloer op grond); vormfactor 2,65 → 3,67. Verwarming primair 1446 → 1474 kWh (nauwelijks — de warmtepomp-SCOP dempt; de winst zit in de **demand** Q_H;nd/Q_C;nd, niet het primair verbruik).

**Groene golden:** `aalten_beng_geometry_within_certified_tolerance` (draait mee in `cargo test`) toetst BENG 1/2/3 tegen deze tolerantie. Diagnostiek: `uniec_measure_bridged` (`--ignored --nocapture`).

**Label blijft A++++** (vs certified A+++): dat is de gedocumenteerde PV-saldering-normversie-delta (F3d-8) die BENG 2 licht onder certified houdt en één labelklasse tipt — een EP-crate-kwestie los van de geometrie. De bestaande `uniec_aalten_2522`-golden (niet-bridged pad + exacte label-assertie) blijft daarom `#[ignore]`.

**Ketenbeperkingen (nog niet benut door het rekenpad, gedocumenteerd in `geometry_bridge`):** de vloer-op-grond P/A-omtrek (grond-conductantie is forfait `h_g;an = 10 W/K`, §8.3.1) en de raam-U in de demand-transmissie (ramen op opake U; de raam-U voedt wél de TOjuli-noemer). Dat het resultaat ondanks deze twee vereenvoudigingen binnen tolerantie valt, wijst erop dat de buiten-oppervlakte-bron de dominante driver was.
