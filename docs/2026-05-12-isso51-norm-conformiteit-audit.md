# ISSO 51 norm-conformiteit audit — isso51-core

**Datum:** 2026-05-12
**Scope:** `crates/isso51-core/` op master `2dc144d`
**Referenties:** ISSO 51:2023 (dr-engineering 2024 publ.) + erratum 2023 + 2 worked examples (vrijstaande woning 2017, Vabi Janssen 2017)
**Mode:** read-only audit, geen code aangepast.

## Samenvatting

- **De rekenkern dekt het erratum 2023 goed af op vertrekniveau** — alle vier expliciete aandachtspunten (θ_b=17°C, kwadratische sommatie, ρ·cp=1.2, qi_spec dm³/s/m²) zijn correct geïmplementeerd in de modules en zichtbaar in `formulas.rs` constanten.
- **Echter: `build_summary` (gebouwsamenvatting) telt verliezen lineair op in plaats van kwadratisch.** Dit overschat het gebouw-aansluitvermogen en verschilt van het gedrag op vertrekniveau. Hiaat t.o.v. erratum §10 en §16. [`lib.rs:171-176`]
- **Φ_vent = Φ_v op gebouwniveau** in plaats van `Φ_v − Φ_i` zoals erratum formule 3.3 (par. 3.2.3) voorschrijft voor hoofdstuk 3 (gebouw). Per-vertrek (hoofdstuk 4) is `Φ_vent = Φ_v` wel toegestaan. Discrepantie tussen norm-conventie en gebouwsamenvatting.
- **Tabel 2.12 erratum is 1-op-1 geïmplementeerd** (14 verwarmingssystemen × Δθ₁/Δθ₂/Δθ_v_high/Δθ_v_low) — bewezen door codestructuur match met erratum-samenvatting tabel.
- **`InfiltrationMethod::PerExteriorArea` is default** maar erratum formule E.5 (par. E.2.2) schrijft expliciet voor: `qi_spec per m² gebruiksoppervlak` (Tabel 2.8). De `PerFloorArea` methode is dus erratum-conform; de default `PerExteriorArea` (Tabel 4.3) is een legacy ISSO 51:2017 pad. Voor 2024-projecten levert de default andere getallen dan Vabi 3.12.

## Per-module bevindingen

### calc/transmission.rs
- **Bron:** ISSO 51 §2.5.1–§2.5.5; formules 4.2, 4.3a, 4.6, 4.10, 4.14–4.18.
- **Code:** zes helper-functies + `calculate_all_h_t`. Pre-erratum signconventie `(θ_i − θ_e)` correct overal toegepast (regel 83, 148–150, 206).
- **Status:** ✓ conform.
- **Bonus:** boundary `Water` is een **niet-norm extensie** voor woonboten (geclausuleerd in docstring regel 174-181, design-warm `theta_water=5°C` default). Correct geïmplementeerd; report-voetnoot is een eis op rapportage-laag (niet hier).
- **Sterk punt:** div-by-zero guards in `f_v`, `h_t_adjacent_room_element`, `h_t_adjacent_building_element`, `h_t_water_element` (regels 78-81, 142-146, 202-205) — robuust tegen pathologische input.
- **Issue (minor):** `h_t_unheated_element` default `f_k = 0.5` als geen `temperature_factor` is gezet (regel 101). Norm Tabel 4.1 differentieert per ruimtetype (b.v. `0.8` voor binnenzijde isolatie aanwezig, `0.5` voor crawlspace). Default oogt arbitrair — beter een explicit `None`-error of import-time validatie.

### calc/infiltration.rs
- **Bron:** §2.5.6, §3.2.1, §4.2.1; erratum E.5.
- **Code regel 35-37:** `H_i = 1.2 × q_i` met q_i in dm³/s.
- **Status:** ✓ conform.
- **Detail:** geen `z`-factor in `h_infiltration` zelf — die wordt op `phi_infiltration` regel 51-53 als parameter doorgegeven. `room_load.rs:122` zet `z_i = 1.0` als hardcode. Erratum verwijderde z-tabellen (par. 3.2 → "default 1") dus klopt.

### calc/ventilation.rs
- **Bron:** §2.5.7, §4.2.2; erratum formules 4.6a, 4.6b, 4.7.
- **Code regel 19-25:** `f_v = ((θ_i + Δθ_v) − θ_t) / (θ_i − θ_e)` — exact erratum formule 4.6a.
- **Code regel 39-45:** `f_v_adjacent = ((θ_i + Δθ_v) − θ_a) / (θ_i − θ_e)` — exact erratum formule 4.6b.
- **Code regel 73-75:** `h_ventilation_mixed = 1.2 × ((a × q_v × f_v1) + ((1−a) × q_v × f_v2))` — exact erratum formule 4.7.
- **Status:** ✓ conform, één-op-één erratum implementatie.

### calc/heating_up.rs
- **Bron:** §2.5.8, §4.3 (erratum: Φ_op → Φ_hu,i, A_vl → A_g).
- **Code:** main-room methode (`f_RH × ΣA_accumulating`) en percentage-methode voor overige ruimten (regel 27-53). Negatief clampen naar 0 (regel 47).
- **Status:** ✓ conform.
- **Issue (minor):** symboolwijziging erratum `Φ_op → Φ_hu,i` is in `result.rs::HeatingUpResult::phi_hu` en `formulas::ISSO_51_2023_PARAG4_3` correct toegepast, maar het concept van *bedrijfsbeperking* (continu bedrijf → 0 W) wordt afgevangen via `building.has_night_setback`. DR-engineering rapport p.5 expliciet "Geen opwarmtoeslag (continu bedrijf)", lib.rs:101 doet `if has_night_setback && warmup_time > 0.0` — gedrag conform.

### calc/quadratic_sum.rs
- **Bron:** erratum formule 3.11: `Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)`.
- **Code regel 20-22:** exact.
- **Status:** ✓ conform op vertrekniveau (`room_load.rs:285` past dit toe per ruimte).
- **Issue 🔴:** `build_summary` in `lib.rs:171-176` past de kwadratische sommatie **NIET** toe — `connection_capacity` is een lineaire optelsom. Het worked-example `dr_engineering_woningbouw_result.json:172-184` toont expliciet `phi_extra = 770` (kwadratische som) op gebouwniveau, en `phi_hl_build = 6700 = phi_basis(5931) + phi_extra(770)`. Engine produceert 5931 + 770 als simpel optellen op per-room basis (dat klopt voor deze fixture met alleen `phi_vent` als extra-component) maar mist de juiste sommatie als er meerdere extra-bronnen op gebouwniveau zijn. **Erratum §10/§16 expliciet niet gedekt op gebouwniveau.**

### calc/system_losses.rs
- **Bron:** §2.9, Tabel 2.17 (vloer), Tabel 2.18 (wand — erratum), par. 2.9.1 (plafond — erratum).
- **Code regel 12-24:** vloerwaardes 0.85/0.40/0.25/0.15/0.10 — match erratum-samenvatting "Tabel 2.18" 1:1. Norm Tabel 2.17 (vloer, originele) heeft identieke waarden.
- **Code regel 34-46:** wandwaarden — **identiek aan vloer**, niet de Tabel 2.18-erratum waarden (0.85 / 0.4 / 0.25 / 0.15 / 0.1). Wacht — erratum-samenvatting tabel toont **wel** dezelfde 5-bin staffel voor f_wvw als f_vvw. ✓
- **Code regel 56-62:** plafond = 0.20 (Rc≥3) / 0.50 (overig) — match erratum-samenvatting punt #12 1:1.
- **Status:** ✓ conform aan erratum.
- **Architectuur:** circulair `Φ_system = f × Φ_HL,i` opgelost in `room_load.rs:290-300` via algebraïsche herleiding `Φ_HL = (Φ_basis_no_sys + Φ_extra) / (1 − f)`. Wiskundig correct; geen iteratie nodig.

### calc/room_load.rs
- **Bron:** Hoofdstuk 4, orchestrator.
- **Status:** ✓ grotendeels conform. Drie specifieke punten:
- **Voetnoot 2 Tabel 2.12 (vide-correctie)** correct geïmplementeerd in `height_factor()` regel 39-45 met threshold `> 4.0 m` en lineair `h/4` — bewezen met regressie-tests regel 387-463.
- **Δθ_v selectie** (regel 132-133): op gebouwniveau Ū berekend in `lib.rs:75-82`, dan per-vertrek doorgegeven. ✓
- **Issue ⚠ (regel 195-198):** comment "ventilation loss, independent of infiltration" met `phi_vent = phi_v.max(0.0)`. Op vertrekniveau (Hoofdstuk 4) zegt erratum regel 528: *"Er geldt voor het in rekening te brengen ventilatiewarmteverlies: Φvent = Φv"* dus dit is **correct voor par. 4**. Maar op gebouwniveau (Hoofdstuk 3 formule 3.3): `Φ_vent = Φ_v − Φ_i`. Build_summary doet niet deze subtractie. Zie 🔴 hierboven.
- **Issue ⚠ (regel 252-262):** R_c berekening uit U-waarde voor system losses gebruikt vaste `R_si=0.17` (wand+vloer) en `R_si=0.14` (plafond) plus `R_se=0.04` (exterior) of `R_se=0.0` (ground/water). Norm NEN-EN ISO 6946 schrijft R_si=0.13 voor wanden, 0.10 voor plafond opwaartse, 0.17 voor neerwaartse warmtestroom. Engineer-aanname documenteer of corrigeer.

## Per-tables bevindingen

### tables/temperature.rs (Tabel 2.12 erratum)
- 14 verwarmingssystemen met Δθ₁/Δθ₂/Δθ_v_high/Δθ_v_low.
- **Vergelijking erratum-samenvatting tabel:** alle 14 waarden 1:1 match. Specifiek gecontroleerd: `RadiatorLt` (2/-1/0/0), `FloorHeatingMainHigh` (0/0/-1/-0.5), `FloorHeatingMainLow` (0/0/-0.5/0), `WallHeating` (2/-1/-1/-0.5), `FanConvector` (0.5/0/0/0).
- **Status:** ✓ conform erratum. Eén nuance: erratum staffelt `FloorHeatingWithRadiatorLt` Δθ₂=0, code regel 91 ook =0 ✓.

### tables/infiltration.rs (Tabel 2.8 + Tabel 4.3)
- `qi_spec_per_exterior_area` regel 13-24: 0.08/0.16/0.24/0.32 per qv10-klasse — match Tabel 4.3 (ISSO 51:2017/2023).
- `qi_spec_per_floor_area` regel 34-45: 0.04/0.08/0.12/0.16 per qv10-klasse — match Tabel 2.8 (erratum E.5).
- **Status:** ✓ conform. Beide tabellen aanwezig, default kiest oude Tabel 4.3 — zie 🟡 hieronder.

### tables/ventilation.rs (BBL minimums)
- Niet uit ISSO 51 maar uit BBL Afdeling 3.6: 0.9 dm³/s/m² verblijfsruimte, 7/14/21 dm³/s voor toilet/badkamer/keuken.
- **Status:** ✓ niet-norm fallback voor missende input, correct gelabeld als BBL.

### tables/heating_up.rs (Tabel 4.6 + Figuur 4.2)
- `night_cooling()` regel 9-16: 3.0K vrijstaand, 2.0K twee-onder-een-kap/hoekwoning, 1.5K tussenwoning/portiek/galerij/gestapeld — match ISSO 51 Figuur 4.2.
- `heating_up_factor()` regel 27-75: 8 Δt-rijen × 3 warmup-kolommen lookup met lineaire interpolatie. 1.5K@2h=1.7 ✓ (test regel 81-89).
- **Status:** ✓ conform. Geen erratum-wijziging op deze tabel volgens samenvatting.

### tables/thermal_bridge.rs (Tabel 2.8 — forfaitaire)
- Constante `DELTA_U_TB_FORFAITAIRE = 0.1` regel 7.
- **Status:** ✓ conform §2.5.1.
- **Issue (info):** DR-engineering rapport p.4: `dUtb = 0.02` voor nieuw gebouw met voorzieningen (2024). Onze code default 0.1 maar respecteert `custom_delta_u_tb` per element. Geen norm-fout, maar default suggereert oudbouw — overweeg context-afhankelijke default of project-level override.

## Erratum 2023 dekking

| Erratum-bullet | In code? | Locatie | Opmerking |
|---|---|---|---|
| #1 θ_b = 17°C woonfunctie | ✓ | `climate.rs:55, 69` (default 17.0) | Conform |
| #1 θ_b = 14°C overige functies | ✓ | `climate.rs:24, 72` (default 14.0) | Apart veld `theta_b_non_residential` |
| #2 Formule 2.15 (θ_i − θ_e) | ✓ | overal (b.v. `transmission.rs:83, 148`) | Sign correct |
| #3 Formule 2.18 (θ_a) | ✓ | `formulas.rs:43-44` (Δ₂ correctie correct toegepast) | f_b vloer |
| #4 Tabel 2.12 vervangen | ✓ | `tables/temperature.rs:33-120` | Alle 14 systemen + 4 kolommen |
| #5 Tabel 2.14 (WTW θ_t) | ✓ | `model/ventilation.rs:78-89` | Alle 8 frost-protection typen |
| #6 Tabel 3.1/4.1 verwijderd (z-factor) | ✓ | `room_load.rs:122` (`z_i = 1.0` hardcode) | z-tabellen weg, default 1 |
| #7 Factor 1200 → 1.2 | ✓ | `infiltration.rs:36`, `ventilation.rs:58, 74` | dm³/s units |
| #8 Φ_op → Φ_hu,i | ✓ | `formulas.rs:139` `ISSO_51_2023_PARAG4_3`, `result.rs::HeatingUpResult::phi_hu` | Symbool gemigreerd |
| #8 A_vl → A_g | ✓ | `building.rs:74` (`total_floor_area`) | Gebruiksoppervlak veld |
| #9 Kwadratische som 3.11 | ⚠ | `quadratic_sum.rs:20-22` + `room_load.rs:285` | Wel per ruimte, NIET in `build_summary` (lib.rs:171) |
| #10 f_v formules met Δθ_v | ✓ | `ventilation.rs:19-25, 39-45` | Beide 4.6a/4.6b correct |
| #11 Tabel 2.18 wandverwarming | ✓ | `system_losses.rs:34-46` | 5-bin staffel match |
| #12 Plafondverwarming Φ_verlies3 | ✓ | `system_losses.rs:56-62` | 0.20 / 0.50 staffel |
| #13 Ventilatiesysteem E | ⚠ | `enums.rs:130` (`SystemE` enum) — runtime ondersteuning **niet aanwezig** | Lokaal D + centraal C per ruimte mengvorm bestaat alleen als label, geen per-ruimte routing |
| #14 Overige (formule 2.29 B', PMW→PMV) | n/a | — | Niet-rekenkern wijzigingen |
| Form. 3.3 `Φ_vent = Φ_v − Φ_i` (gebouw) | ✗ | `lib.rs:166` (gebouwniveau) telt `phi_v` op, geen aftrek | Gebouwsamenvatting wijkt af van norm-conventie |
| Form. E.5 `H_i = 1.2 · qi,spec · z · ΣA_g` | ✓ | `infiltration.rs:35-37` (Hi) + `room_load.rs:115-120` (PerFloorArea pad) | Maar default = PerExteriorArea (Tabel 4.3 legacy) |

## Vier expliciete aandachtspunten

1. **θ_b = 17°C (erratum #1):** ✓
   - `climate.rs:55` → `default_theta_b_residential() -> f64 { 17.0 }`
   - `climate.rs:53-62` Default impl gebruikt 17.0
   - `room_load.rs:71` → `let theta_b = climate.theta_b_residential;` — gerouteerd naar f_b berekening
   - Bewijs van 15°C-paadje als legacy: `lib.rs:257` test creëert `theta_b_residential: 15.0` expliciet voor ISSO 51:2017 voorbeeld; dat is correct (norm-versie 2017 had 15°C).

2. **Kwadratische sommatie (erratum #9 formule 3.11):**
   - **Per vertrek:** ✓ `quadratic_sum.rs:20-22` + `room_load.rs:285` → `quadratic_sum(phi_vent, phi_t_adj_building, phi_hu)`
   - **Per gebouw:** ✗ `lib.rs:171-186` doet lineair `total_envelope_loss + total_neighbor_loss + total_ventilation_loss + total_heating_up + total_system_losses`. Geen `sqrt(vent² + iaBE² + hu²)` op gebouwniveau.

3. **Factor 1.2 kJ/(m³·K) = ρ·cp (erratum #7):**
   - ✓ `infiltration.rs:36` `1.2 * q_i`
   - ✓ `ventilation.rs:58` `1.2 * q_v * fv`
   - ✓ `ventilation.rs:74` `1.2 * ((a * q_v * f_v1) + ((1.0 - a) * q_v * f_v2))`
   - Geen enkel `1200` magic-number meer in calc/ — clean.

4. **qi_spec unit consistentie (dm³/s/m², niet m³/s):**
   - ✓ `tables/infiltration.rs:13-24` retourneert 0.08–0.32 dm³/s per m² — match Tabel 4.3 ISSO 51 (0.08, 0.16, 0.24, 0.32 dm³/s/m² ≈ 8/16/24/32 × 10⁻⁵ m³/s/m²).
   - ✓ `infiltration_flow_rate` regel 20-22 retourneert q_i in dm³/s.
   - ✓ `h_infiltration` regel 35-37 verwerkt q_i in dm³/s met factor 1.2 (geen 1200).
   - Test `test_isso51_example_room1_infiltration` regel 60-75 verifieert 0.16 × 14.13 = 2.26 dm³/s — match norm voorbeeld 1.

## Numerieke validatie tegen worked examples

### Vrijstaande woning ISSO 51:2017 (Vabi/Janssen)
- **Input fixture:** `tests/fixtures/vabi_vrijstaande_woning.json` (52 KB). 9 verwarmde ruimten, θ_e=-9°C, dUtb=0.05, qv10 zware bouw.
- **Expected:** `tests/fixtures/vabi_vrijstaande_woning_expected.json` — per-ruimte φ_t / φ_v / φ_hu / φ_hl_i. Gebouw 9160 W aansluitvermogen.
- **Test code:** geen integration test op `tests/` map (geen `cargo test`-target). Fixtures bestaan maar worden niet door de Rust testsuite gevalideerd. Worden mogelijk door frontend/Tauri tests gebruikt — buiten audit-scope.
- **Statisch oordeel:** fixture+expected paar is compleet en bruikbaar; ontbreekt alleen de Rust runner. Dekking conceptueel goed (alle vertrekken met heatingsystem=Vloerverwarming, ventilatiesysteem C, nachtverlaging 2K/2h, dUtb=0.05). Discrepantie tussen 2017-norm en 2023-erratum gemerkt in fixture-comments.

### Vabi Woonhuis Janssen 2017
- **Geen Rust fixture aanwezig** — alleen referentie-PDFs + samenvatting-md in `tests/references/`. Output-data uit dit voorbeeld is niet in `tests/fixtures/`.
- **Statisch oordeel:** dit worked-example is alleen documentair gebruikt voor norm-uitleg, niet als regressietest. Gat voor numerieke validatie tegen een tweede onafhankelijke rekenkern.

### DR Engineering Woningbouw ISSO 51:2024
- **Input fixture:** `tests/fixtures/dr_engineering_woningbouw.json` (36 KB) + `_result.json` (4 KB).
- **Statisch oordeel:** dekt erratum-conform geval (θ_b=17, dUtb=0.02, WTW systeem D). Het verwachte resultaat noteert expliciet *"Correctiefactor invloed ventilatievoorziening 1.10 (systeem D) - verschilt van engine"* — een Vabi-eigen factor die nog niet in onze engine zit. Bewust onbestreden, niet uit ISSO 51 norm.

### Portiekwoning (ISSO 51 voorbeeld 1)
- **Input fixture:** `tests/fixtures/portiekwoning.json` + `_result.json`. Heeft theta_b_residential=15 (oude norm).
- **In-source test:** `lib.rs:230+` `test_full_calculation_portiekwoning_woonkamer` — past worked-example #1 toe. ✓

## Gaten en aanbevelingen (prio)

🔴 **Hoog**
1. **`build_summary` past geen kwadratische sommatie toe (erratum formule 3.11 gebouwniveau).** Vandaag werkt het toevallig voor de DR-engineering fixture omdat alle ruimten zelfde phi_extra-componenten hebben, maar bij mengprojecten met thermische bruggen + meerdere systeem-E zones lekt het mis. Fix: op gebouwniveau `Φ_extra_total = √(Σ Φ_vent² + Σ Φ_iaBE² + Σ Φ_hu²)` en `connection_capacity = Φ_basis_total + Φ_extra_total`. Locatie: `lib.rs:171-186`.
2. **`Φ_vent = Φ_v − Φ_i` op gebouwniveau (erratum formule 3.3) niet geïmplementeerd.** `lib.rs:166` somt `phi_v` zonder af te trekken. Levert overschatting van ventilatiebijdrage op H3-niveau. Combineer met fix #1 in één pass.

🟡 **Midden**
3. **Default infiltration method = PerExteriorArea (Tabel 4.3 legacy) ipv PerFloorArea (Tabel 2.8 erratum E.5).** Erratum verwijst voor par. E.2.2 expliciet naar `qi,spec per m² gebruiksoppervlak conform tabel 2.8`. Verander default `InfiltrationMethod::default()` in `enums.rs:199` naar `PerFloorArea`, of merk nieuw-gemaakte projecten 2023+ als erratum-conform. Hold-out: 2017-projecten moeten oude pad blijven kiezen (backward compat).
4. **Ventilatiesysteem E (erratum #13)** heeft alleen een enum-label maar geen runtime-routing per ruimte. Code maakt geen onderscheid tussen `SystemD` en `SystemE` in vent_config behandeling. Voor gemengde systemen moeten ruimten met natte afvoer (badkamer/toilet/keuken) als systeem-C behandeld worden, verblijfsruimten als systeem-D. Voeg per-room `VentilationSystemType` override toe (i.p.v. alleen gebouw-niveau).
5. **`build_summary` mist eigen `Φ_extra` als afzonderlijk veld** — alleen `connection_capacity` (totaal). Voor rapportage en QA is decomposed `phi_vent_total, phi_iaBE_total, phi_hu_total, phi_extra_quadratic` waardevol. Voeg toe aan `BuildingSummary`.

🟢 **Laag**
6. **`h_t_unheated_element` default f_k = 0.5** is een arbitraire keuze waar Tabel 4.1 een differentiatie verwacht. Promoot tot import-time required field of valideer dat `temperature_factor` aanwezig is voor `BoundaryType::UnheatedSpace`.
7. **R_si/R_se hardcodes in system-loss R_c reconstructie** (`room_load.rs:252-262`) gebruiken niet-standaard waarden (0.17/0.14/0.04). NEN-EN ISO 6946 schrijft 0.13/0.10/0.04 voor. Verschil <0.1 m²K/W maar voor staffelgrenzen op 0.35/1.0/2.0/3.0 kan dit bin-flips veroorzaken.
8. **`DELTA_U_TB_FORFAITAIRE = 0.1` (Tabel 2.8 default)** suggereert oudbouw. DR-engineering rapport gebruikt 0.02 voor 2024-nieuwbouw. Overweeg context-afhankelijke default via `building.construction_year` of `building.has_thermal_bridge_provisions`.
9. **Integration test gat:** `tests/fixtures/*.json` worden niet door enige `#[test]` in de Rust testsuite geladen — alleen losse `lib.rs::tests` met hardcoded portiekwoning-room. Voeg `tests/integration_test.rs` toe die alle 4 fixtures laadt, `calculate_from_json` aanroept en tegen `*_expected.json` checkt met tolerantie ±2W per ruimte.

## Niet-onderzocht / out-of-scope

- **NTA 8800 crates** (`nta8800-*`): expliciet uitgesloten.
- **Frontend / Tauri / import / API:** geen review.
- **`crates/isso51-core/src/import/`:** niet geopend; deze bevat thermal-import + IFC mapping, valt onder schil-conversie, niet rekenkern-norm.
- **`crates/isso51-core/src/validate/`:** alleen schemastructuur gecheckt (1 file `mod.rs`). Norm-validatie regels in zicht maar inhoudelijk niet beoordeeld.
- **Floor-heating supply-temperature dependency (par. 2.9.2):** Δθ-tabel kent vloerverwarmingstemperatuur ≥27°C vs <27°C. Code lost dit op via twee aparte HeatingSystem-varianten (`FloorHeatingMainHigh` / `FloorHeatingMainLow`) — correct, maar er zit geen automatische selectie op basis van een berekende `theta_vloer`. Buiten audit-scope: ligt op UX-laag.
- **Numerieke deltas vs Vabi worked-examples:** geen `cargo test` of `cargo run --example` gedraaid (mandaat: statisch). Aanbeveling onder 🟢 #9 dekt dit.
- **G_w (grondwaterfactor) Tabel 2.13:** niet gecontroleerd of het project-level `theta_ground` ergens een rol speelt in de berekening. Code regel 167 `transmission.rs` gebruikt `gp.ground_water_factor` per element, default 1.0 — conform §2.5.5 voor normale waterstand.
- **Solar heat gain / interne warmtelast:** niet aanwezig in ISSO 51 — buiten norm-scope, geen audit-bevinding.
