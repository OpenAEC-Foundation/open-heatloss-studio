# Codex onafhankelijke cross-check (gpt-5, read-only)

**Datum:** 2026-06-02. Codex draaide read-only (kon geen tests draaien, statische analyse). Bevindingen zijn grotendeels **orthogonaal** aan de 4 norm-audit-agents — Codex vond andere, deels zwaardere bugs. Twee criticals (D1, D2) zijn door de PM hard geverifieerd aan de bron.

## Kritieke conformiteitsfouten (Codex)

1. **D4 — `ground.rs:144-155`** — `U_equiv` voor grondvloeren weigert normale ondiepe gevallen via `depth_sum = z + d`; met `d≈-0,02` voor vloeren faalt een vloer op `z=0`. `ground.rs:214` test dit zelfs als verwacht gedrag. Norm 4.24 moet werken voor `0 ≤ z ≤ 5`. Impact: normale begane-grondvloeren falen tenzij `u_equivalent` vooraf is ingevuld. (Agent flagde dit als twijfelgeval T2; Codex verheft tot critical.)

2. **D1 — `temperature.rs:21,93` → `transmission.rs:38`/`ventilation.rs:71`/`infiltration.rs:94`** — `TEMPERATURE_IS_EXTERIOR = f64::MIN` wordt voor `RuimteType::Garage` teruggegeven maar callers vervangen hem NIET door θ_e. `H × (f64::MIN − θ_e)` → astronomisch/oneindig warmteverlies. ✅ **PM-geverifieerd:** `transmission.rs:37-49` gebruikt de sentinel rauw. Fix: enum/Option of sentinel centraal resolven.

3. **D2 — `ventilation.rs:116`** — ventilatie gebruikt altijd `VentilatieBouwfase::Nieuwbouw`. Tabel 4.10 heeft aparte bestaande-bouw-debieten. Impact: bestaande kantoren krijgen ~6,5 i.p.v. ~3,44 dm³/s pp ≈ **+89% Φ_V**. ✅ **PM-geverifieerd:** hardcoded enum-variant. Fix: bouwfase toevoegen/gebruiken in `VentilationConfig` (+ UI-veld).

4. **D3 — `infiltration.rs:117-119,134-136`** — `Unknown`/`UnknownVabiCompat` negeren `building_length/width/height`, vallen stil terug op `0,0,3`. Voor 50×30×20 m gebouw zou `f_wind≈1,29` moeten zijn; fallback geeft `1,00` ≈ 22% te lage infiltratie. Fix: methode-dimensies gebruiken of verplicht maken.

5. **D5 — `shell.rs:88-94`** — schilmethode gebruikt grove vaste aannames `0,5 ach` + `0,00001 m³/s·m²`. Niet norm-conform ISSO 53 hoofdstuk 3. Impact tientallen % in voorontwerp-bronvermogen. Fix: hoofdstuk 3 implementeren of API als niet-normatief labelen.

## Verborgen afwijkingen in tests (Codex)

- `houtfabriek-3floors/expected.json` + golden `:48,54` staan 6% Φ_T toe → laat `3.10a` op +5,0% door. Brede tolerantie maskeert ook echte regressies. (= V2)
- `vabi_dr_golden.rs:77,92` accepteert Φ_T +3,5% met 10% tolerantie. Expected 3059 W, snapshot 3165 W. Nog ~190 W extra regressie zou slagen.
- `vabi_golden.rs:37` checkt Φ_V+Φ_I gecombineerd op 10% → ventilatie- en infiltratiefouten kunnen elkaar compenseren. Splits Φ_V, Φ_I, q_v, H_v, q_i, H_i.
- `isso51 integration_test.rs:323-334` slaat per-veld-checks over voor ruimten met verwacht totaal <1 W → kan teken-/componentfouten verbergen vóór clamp.

## Testdekking-gaten (Codex)

- ISSO 53 auto-`U_equiv`: geen norm-voorbeelden voor normale `z=0` vloer, kelderwand, grondwaterfactor 1,15, B'-clamp-grenzen.
- ISSO 53 ventilatie: geen tests voor bestaande-bouw-fase en afzuig-only toilet/bad/keuken-eisen.
- ISSO 53 heating-up `unwrap_or(0.0)` (`heating_up.rs:97`) voor ongedefinieerde tabel-combinaties; geen test dwingt falen af. `tables/heating_up.rs:166-198` gebruikt nearest-defined fallback voor dash-cellen zonder PDF-bevestiging.
- Bronvermogen 5.1/5.9: alleen synthetische unit-tests, geen end-to-end fixture met `source_fraction_z`.
- ISSO 53-scope is tot 4 m vertrekhoogte, maar geen guard weigert/routeert hogere ruimten. (raakt A5)

## Twijfelgevallen (Codex — PDF nodig)

- ISSO 53 formule 4.24 exacte `U_equiv` machtsstructuur (`ground_params.rs` geeft OCR-onzekerheid toe).
- ISSO 53 tabellen 4.13/4.14: mogen dash-cellen nearest-defined fallback gebruiken?
- ISSO 53 tabel 4.10: behandeling afzuig/overstroomlucht in sanitair en keuken.
- ISSO 51 `VabiCompat` sluit Φ_T,iae uit op gebouwniveau; bevestig tegen ISSO 51:2023 §3.5.1 (= C2).

## Schoon (Codex bevestigt)

- ISSO 53 hoofdruimte `Φ_T = H_T,total × (θ_i − θ_e)` en exterieur/onverwarmd/aangrenzend-factorisatie zijn eenheidsconsistent.
- Luchtwarmtecapaciteit consistent: ISSO 53 `1200 J/(m³·K)` met m³/s; ISSO 51 `1,2` met dm³/s. (= agent-bevinding)
- ISSO 53 bekende-infiltratie `A_u` sluit terecht buitenvloeren/-plafonds uit. (= agent-bevinding)
- ISSO 51 gebouw-aggregatie gebruikt nu kwadratische extra-last (`lib.rs:257`); oude integration-test-comment over lineaire sommatie is achterhaald. (= agent V3-comment)

> **Niet door Codex aangeraakt:** A1 (ISSO 51 opwarmtoeslag 2017-model). Codex sprak het niet tegen; het staat op de PM-hardverificatie tegen ISSO 51:2023 Formule 4.15 + Tabel 2.10. Codex bevestigt wél dat de kwadratische gebouw-aggregatie correct is (los van de Φ_hu-component-fout).
