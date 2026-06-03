# TODO

## 🧪 Norm-conformiteit audit (02-06) — VOLLEDIGE LIJST

> Bron: 4 norm-audit-agents (ISSO 51/53 PDF regel-voor-regel) + UI-dekkingsaudit + Codex cross-check + PM-hardverificatie. Detail per item in `audit-reports/00-SAMENVATTING.md` (+ 01-06). Conform-beleid: **hybride** (norm leidend; Vabi-compat alleen achter gemarkeerd pad). Effort: [L]=laag [M]=middel [H]=hoog. ✅=hard geverifieerd.
> **ISSO 53 is voorgetrokken** (blokken A–C) vóór ISSO 51 (D–E).
> **Voortgang:** ronde 1 (D1, B1, A6) ✅ `f815c1f`. Ronde 2 (D3, B3) ✅ `bb70f7e`. **Ronde 3a (A5 Δθ₁ exterior + vide + datalaag Δθ_v) ✅ — datalaag compleet, Δθ₁ alleen op exterior (4.5/4.6); adjacent (4.11/4.12/4.19/4.20) bewust geparkeerd (tweezijdige Δθ₁/Δθ_a1 vereist per-element buur-heating_system).** A4/A7 ontgrendeld — exacte formules in `audit-reports/07-isso53-formules-ref.md` (form. 4.24+Tabel 4.3, 4.39 Δθ_v).

### 🌅 MORGENOCHTEND — START HIER (aanbevolen volgorde)

> Alle items hieronder staan met detail in blokken A–F. Baseline: `cargo test -p isso53-core` = 111 groen. Werk per ronde: general-purpose agent (NIET rust-developer — worktree-faalt), foreground, daarna `cargo test`, dan git-release commit. Formules: `audit-reports/07-isso53-formules-ref.md`.

1. ~~**Ronde 3a — A5 (ISSO 53 stratificatie Δθ₁ + vide).**~~ ✅ **GEDAAN.** Datalaag `delta_theta_1/_v/_corrected` + `vide_factor` in `tables/temperature_stratification.rs` (12 systemen, volledig getest). Δθ₁ toegepast op exterior horizontaal (4.5/4.6) in `transmission.rs` + `shell.rs` (wanden 1,0). **Adjacent (4.11/4.12 + 4.19/4.20) bewust NIET** — eenzijdige Δθ₁ overschat (+33% artefact op DR-buurplafond); tweezijdige `(θ_i+Δθ₁−(θ_adj+Δθ_a1))` vereist per-element buur-heating_system → A5-vervolg (zie open item onder). Onverwarmd-tak (4.15/4.16) ongemoeid: Δθ₁ hoort bij berekende f_k-route (auto-f_k TODO), niet bij forfaitaire Tabel 4.2. Golden-tests onveranderd groen (geen fixture heeft exterior-horizontaal + Δθ₁>0-systeem). 121 lib-tests groen (+10).
   - [ ] **A5-vervolg [M]** — tweezijdige stratificatie op aangrenzend-vertrek (4.11/4.12) + -gebouw (4.19/4.20): vereist `heating_system` per buur-element in het model. Nu geparkeerd met `// TODO A5-vervolg`-markers in `calculate_h_t_adjacent_rooms/_buildings`.
   - [ ] **U6-afhankelijk** — vide-correctie ×(h/4) is geïmplementeerd maar onbereikbaar zolang room-validatie `height>4m` weigert. Ontgrendelt bij U6 (height-validatie versoepelen + UI-veld).
2. **Ronde 3b — A4 + A7 (ISSO 53 grond + Δθ_v).** A4: ΔU_TB toevoegen aan U_k in `ground.rs:48` (form. 4.24, prioriteit als A6) + bestaande U_equiv-impl in `ground_params.rs` verifiëren tegen ref §Tabel 4.3. A7: Δθ_v toepassen in `ventilation.rs:154`/`infiltration.rs:75` (form. 4.39, `f_v=(θ_i+Δθ_v−θ_e)/(θ_i−θ_e)`) — vereist opp.-gewogen R_c per ruimte voor kolomkeuze.
3. **Ronde 4 — D2 + D4 (ISSO 53 common-case).** D2: bouwfase-veld in `VentilationConfig` + model + UI (U-blok), `ventilation.rs:116` ontkoppelen van hardcoded Nieuwbouw (+89% bestaande bouw). D4: `ground.rs:144` z=0-grondvloer (form. 4.24 nu bekend).
4. **Ronde 5 — ISSO 51 A1 + A2 (opwarmtoeslag 2023-rewrite).** `isso51 heating_up.rs` herschrijven naar `Φ_hu=P×A_g` (Form. 4.15 + Tabel 2.10, zie `00-SAMENVATTING.md`), Δt uit Ū (Afb 2.7, Ū≤0,5→1K), regeltype-branches §4.3.1/4.3.2/4.3.3. Verwijder de fout-codificerende test `test_isso51_example_room1_heating_up`. **+ V1: nieuwe fixture mét nachtverlaging** (anders blijft het ongetest).
5. **Ronde 6 — afronding.** K2 (gelijktijdigheid bronvermogen), UI-gaten U1-U6 (B-blok), test-aanscherping (C-blok V2 + split Φ_V/Φ_I), ISSO 51 K3 + vabi_import.rs (D-blok), twijfel-items A3-blok (PDF-verificatie), Vabi C1/C2-markering (F-blok).


### A. ISSO 53 — calc-conformiteit (urgent eerst)
- [x] **D1 [L] LANDMINE** ✅ `f815c1f` (resolve_theta_i helper) — `tables/temperature.rs:21,93` sentinel `f64::MIN` voor `Garage` wordt door callers (`calc/transmission.rs:38`, `ventilation.rs:71`, `infiltration.rs:94`) NIET vervangen door θ_e → `H×(f64::MIN−θ_e)` = **oneindig/astronomisch verlies**. ✅ Fix: enum/Option of sentinel centraal resolven.
- [ ] **D2 [M]** — `calc/ventilation.rs:116` hardcodet `VentilatieBouwfase::Nieuwbouw` → bestaande bouw krijgt ~6,5 i.p.v. ~3,44 dm³/s pp ≈ **+89% Φ_V**. ✅ Fix: bouwfase in `VentilationConfig` + model/UI-veld (zie U-blok).
- [ ] **D4 [M]** — `calc/ground.rs:144-155` `U_equiv` weigert normale `z=0` grondvloer (`ground.rs:214` test bevestigt fout gedrag). Fix: formule 4.24 herafleiden + norm-voorbeelden z=0/0,5/5.
- [x] **D3 [L]** ✅ ronde 2 (resolve_building_dimensions helper) — `calc/infiltration.rs:117-119,134-136` `Unknown`/`UnknownVabiCompat` negeren `building_length/width/height` → f_wind=1,0 i.p.v. ~1,29 (~22% te laag). Fix: methode-dimensies gebruiken of verplicht maken.
- [x] **A6 [L]** ✅ `f815c1f` (shell.rs = transmission.rs) — `calc/shell.rs:52-56` ΔU_TB-prioriteit omgekeerd t.o.v. `transmission.rs` (forfaitair wint, custom genegeerd) → tot kW-orde voorontwerp.
- [ ] **A4 [M]** ✅ ONTGRENDELD (form. 4.24 + Tabel 4.3 in `07-isso53-formules-ref.md`) — `calc/ground.rs:48` geeft rauwe `u_value` als U_k aan `calculate_u_equivalent`, maar form. 4.24 vereist `(U_k + ΔU_TB)`. Voeg ΔU_TB toe met dezelfde forfaitair/custom-prioriteit als transmission.rs (A6). Verifieer meteen de bestaande `ground_params.rs` U_equiv-impl tegen de nu-bevestigde 4.24/Tabel 4.3.
- [ ] **A7 [M]** ✅ ONTGRENDELD (form. 4.39 in `07-isso53-formules-ref.md`: `f_v = (θ_i + Δθ_v − θ_e)/(θ_i − θ_e)`) — `calc/ventilation.rs:154` + `calc/infiltration.rs:75` hardcoden f_v=1,0, negeren Δθ_v → ~3% overschatting bij straling/vloer/wand-verwarming. Fix: nieuwe `delta_theta_v(system, rc≥3,5)` in `temperature_stratification.rs` (Tabel 2.3-kolom) + toepassen in 4.39/4.30. Vereist oppervlakte-gewogen R_c per ruimte voor de kolomkeuze.
- [ ] **A3 [M]** — `calc/heating_up.rs:106-110` §4.8.3-reductie `−H_v·Δθ` wordt via project-brede vlag óók op natuurlijk geventileerde ruimten toegepast → Φ_hu te laag/0.
- [ ] **K2 [M]** — `lib.rs:93` / `calc/source_capacity.rs:38,79` sommeren Σ Φ_hu onvoorwaardelijk; geen gelijktijdigheids-selectie (§4.1/§5.1) → overdimensionering Φ_source.
- [x] **A5 [H]** ✅ GEDAAN Ronde 3a (Δθ₁ exterior + vide-datalaag + Δθ_v-datalaag; adjacent geparkeerd) — PDF-bevestigd (tab 2.3 p.21-22 + voetnoot 2) — `tables/temperature_stratification.rs` had alléén Δθ₂ (1 call-site `ground.rs:189`, correct). Ontbreekt: **Δθ₁** (+4/+3/+2/+1/0/0,5 per systeem; nodig in form. 3.4/3.5, 4.5/4.6, 4.11/4.12, 4.15/4.16, 4.19/4.20 → ~+10% op dak/vloer-boven-buitenlucht), **Δθ_v** (=A7), Δθ_a1/Δθ_a2, en vide-correctie **Δθ₁×(h/4)** bij h>4m (voetnoot 2). Volledige tabel in `audit-reports/00-SAMENVATTING.md`. Mogelijk verklaart dit de verborgen +5,0% op dak-zwaar vertrek 3.10a.
- [ ] **D5 [H]** — `calc/shell.rs:88-94` voorontwerp-schil grove vaste aannames (0,5 ach + 0,00001 m³/s·m²) = niet norm-conform hfst 3. Fix: hfst 3 implementeren of API als niet-normatief labelen.

### A2. ISSO 53 — stille-fout defaults (fout antwoord zónder error)
- [x] **B1 [L]** ✅ `f815c1f` (InvalidHeatingUpParameters error) — `calc/heating_up.rs:97` `unwrap_or(0.0)` bij ongeldige setback-uren/graden → Φ_hu verdwijnt geruisloos.
- [ ] **B2 [L]** — `model/project.rs:27` `#[serde(default)]` → ontbrekend `heatingUp`-blok = Φ_hu=0 hele gebouw (third-party import ~10-28% te laag). Fix: expliciete waarschuwing/error.
- [x] **B3 [L]** ✅ ronde 2 (benoemde consts DEFAULT_OCCUPANCY_DENSITY/VENTILATION_RATE) — `calc/ventilation.rs:108,117` magic `unwrap_or(0.05/6.5)` zonder rapport-spoor.

### A3. ISSO 53 — twijfel (PDF-verificatie vóór fix)
- [ ] Formule 4.24 exacte `U_equiv`-machtsstructuur — `tables/ground_params.rs` geeft OCR-onzekerheid toe (verifieer tegen worked example p.65: U=2,43→U_equiv=0,177).
- [ ] Tabellen 4.13/4.14 dash-cellen — mag `tables/heating_up.rs:166-198` nearest-defined fallback gebruiken?
- [ ] Tabel 4.10 — behandeling afzuig/overstroomlucht in sanitair + keuken.
- [ ] Dode params: `material_type` (claimt ΔU_TB-invloed die niet bestaat — `DELTA_U_TB_DEFAULT` is constant) + `theta_b_adjacent_building` (hardcoded 15°C in `transmission.rs:178`).

### B. ISSO 53 — UI-veld-dekking (calc-input zónder invoerveld → stille default)
- [ ] **U1** — `source_zone_config` niet gemapt → Φ_source altijd z=0,5; gescheiden opwekker (z=1,0) onbereikbaar.
- [ ] **U2** — `unheated_space`-enum (15 norm-varianten tab 4.2) niet kiesbaar → reductiefactor altijd 0,5.
- [ ] **U3** — koudebrug-toggle + custom ΔU_TB geen UI → forfaitair altijd aan (raakt A6).
- [ ] **U4** — grond-params (u_equiv, f_gw, perimeter/diepte) alleen via thermal-import; f_gw altijd 1,0.
- [ ] **U5** — voorverwarming (`has_preheating`/temperatuur) geen UI.
- [ ] **U6** — vide/vertrekhoogte >4m: per-vertrek-calc leest `room.height` niet (raakt A5).

### C. ISSO 53 — testdekking
- [ ] **V2** — toleranties aanscherpen: `vabi_houtfabriek_3floors_golden.rs:48,54` (6% laat 3.10a +5% door); `vabi_dr_golden.rs:77,92` (10%, expected 3059 W vs snapshot 3165 W = +3,5%, nog ~190 W slack).
- [ ] Split `vabi_golden.rs:37` gecombineerde Φ_V+Φ_I-check → aparte Φ_V, Φ_I, q_v, H_v, q_i, H_i (fouten compenseren nu).
- [ ] Test bestaande-bouw ventilatiefase (dekt D2) + afzuig-only toilet/bad/keuken-eisen.
- [ ] End-to-end fixture met `source_fraction_z` (bronvermogen 5.1/5.9 heeft alleen synthetische units).
- [ ] Guard/test voor vertrekhoogte >4m (scope-grens, raakt A5).
- [ ] Fixture mét nachtverlaging die Φ_hu écht uitvoert.

### D. ISSO 51 — calc-conformiteit
- [ ] **A1 [H] GROOTSTE FOUT** — `calc/heating_up.rs:39-52` + `room_load.rs:222-242` gebruiken 2017-model `f_RH × ΣA_metselwerk` i.p.v. 2023 Formule 4.15 `Φ_hu = P × A_g` (vloeroppervlak). `f_RH` bestaat niet in 2023. ✅ PDF-bevestigd (§4.3.1 p.70 + Tabel 2.10 p.45). Rewrite + verwijder de fout-codificerende test `test_isso51_example_room1_heating_up`. Scope A_g (per-vertrek vs gebouwbreed verdeeld) exact uitwerken.
- [ ] **A2 [M]** — `tables/heating_up.rs:9-16` nacht-afkoeling Δt aan gebouwtype gekoppeld i.p.v. Ū (Afb 2.7 p.44); mist regel Ū≤0,50→1K + zwaarte-as (ZL+L+M/Z) van Tabel 2.10. ✅ PDF-bevestigd.
- [ ] **A1b [M]** — regeltype-branches ontbreken: §4.3.1 P×A_g / §4.3.2 zelflerend → Φ_hu=0 / §4.3.3 kamerthermostaat → 5 W/m².
- [ ] **K3 [M]** — `lib.rs:204,218-225,257` `connection_capacity` telt systeemverliezen mee (strijdig met Form. 3.12; horen alleen in 3.13). Alleen bij embedded heating.
- [ ] **vabi_import.rs [L]** — example compileert niet (`import_vabi_project` alleen onder `#[cfg(feature="vabi-import")]`). Fix: `[[example]]` met `required-features = ["vabi-import"]` in `Cargo.toml` (geen code-wijziging).

### E. ISSO 51 — testdekking
- [ ] **V1 KRITIEK** — beide Vabi-fixtures hebben `night_setback=false` → alle `phi_hu=0`; A1/A2 worden NOOIT getest. Voeg fixture mét nachtverlaging toe.
- [ ] **V3** — `integration_test.rs:5-11` comment claimt dat DR moet falen op linear-sum; achterhaald (`lib.rs:257` doet quadratic). Opschonen.
- [ ] `integration_test.rs:323-334` slaat per-veld-checks over voor ruimten <1 W → kan teken-/componentfouten verbergen vóór clamp.

### F. Cross-cutting / Vabi-keuzes (hybride: markeren + dubbel testen)
- [ ] **C1** — `tables/nen8088.rs` infiltratie power-law (Δp=3,14) = Vabi-reproductie, niet ISSO 53 → expliciet markeren in rapport-output.
- [ ] **C2** — `isso51 lib.rs:218-225` `VabiCompat`-aggregatie sluit Φ_T,iae uit (afwijkend van Form. 3.10). Verifieer tegen §3.5.1; zet ISSO-conforme variant naast de Vabi-variant.
- [ ] **frost_protection** — orphan in isso53-mapper (stuurt altijd null), wél isso51-relevant → opruimen of wiren.

---

## 🔍 ISSO 53 warmteverlies — ventilatie + onverwarmd (02-06, Reddingspost Kijkduin, 256 m² utiliteit)

> Context: gebruiker valideerde een ISSO 53-utiliteitsproject (reddingspost, kleedkamers/techniek/berging). 02-06 zijn 10 commits gemaakt (zie `sessions/warmteverlies_latest.md` in de orchestrator). Onderstaande items staan nog open; de oorspronkelijke 4 meldingen van 01-06 zijn opgelost of doorontwikkeld.

### ✅ Opgelost 02-06
- Berekenen crashte (serde regime `9c2bb2b`); opslaan verloor ISSO 53-config (`3e29bf4`, nu `.heatloss.json` met norm+sidecars); ruimte zonder ventilatie-eis crashte (`d32d497`).
- Ventilatie-rij: **vastgestelde toevoer-q_v** stuurt de calc (leeg=BBL-placeholder 0,9 dm³/s·m²), met **BBL-min / personen-min / gekozen** in de rij + snelknoppen (`5e9834d`/`365556b`/`ac62b4b`). Vervangt #2 "ventilatie te laag" + #4 "personen-ventilatie tonen".
- Chart transmissie: **onverwarmd eigen categorie** + f_k=0,5 i.p.v. volle ΔT + ISSO 53-temps (`95873cf`). Het "8000W naar binnenwanden" was puur deze weergavebug — echte binnenwanden = netto −772W.
- **f_k per onverwarmde ruimte instelbaar** (`5584384`), default 0,5, override per ruimte.

### ⬜ Open — calc/feature
- [ ] **Auto-f_k voor onverwarmde ruimtes** = `H_ue / (H_iu + H_ue)` uit de geometrie van de onverwarmde ruimte (ISSO 53 §4.4 / tabel 4.2). Goed geïsoleerde, "meeverwarmende" ruimtes → f_k≈0 → verlies ~0. **Geverifieerd op dit project: Berging 0,030 · Meterkast 0,026** (i.p.v. 0,5 → 16× lager, verlies 3843W→~230W). Handmatige `unheatedFactor` (`5584384`) blijft als override. Plek: `lib/isso53Unheated.ts` (helper aanwezig: `collectUnheatedTargetIds`) + `isso53ProjectMapper.ts` + chart `deltaT.ts`.
- [x] **Per-ruimte "Onverwarmd"-toggle** — checkbox + f_k-veld per ruimte (`Isso53RoomState.isUnheated`). Aanvinken → wanden van buren naar die ruimte worden als `unheated` geëmit met de f_k van de ruimte. Lost de inconsistente import-markering op (Techniek/afval als 10°C adjacent_room → nu handmatig op onverwarmd te zetten, f_k≈0,03 → ~0 verlies).
- [ ] **Onverwarmde ruimte uit gebouwtotaal halen.** Een als onverwarmd gemarkeerde ruimte telt nog steeds als eigen (10/15°C) ruimte mee in het totaal → kleine dubbeltelling met de buren-f_k-route. Flagged-unheated rooms zouden geen eigen verwarmingsvraag moeten produceren (hun schilverlies loopt via de buren-f_k).
- [ ] **Auto z-factor infiltratie (tabel 5.1) uit kompasrichtingen.** De z (1,0 / 0,7 / 0,5) hangt af van de gevel-configuratie per vertrek: 1 buitengevel of 2 niet-tegenover → 1,0; 2 tegenover elkaar → 0,5; overig → 0,7. Nu handmatig per ruimte, default 1,0 (max/conservatief → infiltratie hoog). De import heeft per wand een `compass` (N/O/Z/W) → z automatisch afleiden: heeft een vertrek exterior-wanden op tegenoverliggende richtingen → 0,5; één richting → 1,0. Analoog aan auto-f_k. `crates/isso51-core/src/import/thermal.rs` (kompas aanwezig) + `isso53Ventilation`/sidecar + UI z-dropdown (`Isso53RoomFunctionCell.tsx`).
- [ ] **Opwarmtoeslag §4.8 valideren tegen Vabi** — formule matcht PDF p.66 (test `regression_isso53_example_p66`), maar nog geen Vabi-ijkpunt voor dit project. In de huidige config staat `setbackActive=false` → φ_hu=0, dus alleen relevant zodra setback aan gaat. `crates/isso53-core/src/calc/heating_up.rs`.
- [ ] **Onverwarmde ruimtes lichte dubbeltelling** — Meterkast/Bergingen tellen óók als 15°C-ruimte mee in het gebouwtotaal (+365W netto). Conceptueel dubbel (onverwarmd-buur én 15°C-ruimte).

### ⬜ Open — opschoning/weergave
- [ ] **supply-toggle opruimen** (`514bbf9`, `has_mechanical_supply`-gate) — overbodig geworden nu de vastgestelde q_v leidend is (leeg/0 = geen toevoer). Verwarrend in de UI voor ISSO 53.
- [ ] **Chart adjacent_room: bruto-positief vs netto** — de chart sommeert alleen positieve bijdragen (1662W) terwijl de calc netto −772W oplevert (koude ruimtes winnen terug). Overweeg netto tonen of het label verduidelijken.
- [ ] **`.ifcenergy`-export draagt ISSO 53-sidecars niet** — alleen `.heatloss.json` persisteert norm+sidecars. Bij opslaan als `.ifcenergy` gaat ISSO 53-config verloren.
- [ ] **Infiltratie z-reporting inconsistentie** — `result.summary.infiltrationReductionFactorZ` toont `0.5` (oud ISSO 51-gebouwveld) terwijl de ISSO 53-calc de **per-ruimte** z gebruikt (default 1,0). Verwarrend in de samenvatting. Laat de gerapporteerde z matchen met wat de calc gebruikt (of verberg 'm bij isso53). 02-06 verifieerd op Reddingspost: infiltratie 5248W = q_is(0,00064)×A_u(231,6)×1200 met z=1,0 (impliciete factor exact 1,000 per ruimte) — rekenkundig correct, maar z=1,0 overal = conservatief.
- [ ] **Ventilatie-feedthrough — GEDIAGNOSEERD 03-06: stale result, geen calc-bug.** Op `Reddingspost_kijkduin.heatloss.json` (03-06) phiV per ruimte exact terug te rekenen op de **personen-fallback** (q_v=None-pad: `floor_area×0,05×6,5/1000×1200×f_v×ΔT`) i.p.v. de ingevulde q_v (Instructie 125→35W, Ieeftuimte 150→77W, Politiepost 75→0W via supply-gate). Mapper (`isso53ProjectMapper.ts:227` `ventilation_rate/1000`, 0 blijft 0) én Rust (`calc/ventilation.rs:96` vastgestelde q_v overruled gate, getest) zijn **correct**; het opgeslagen result dateert van vóór de q_v-invoer. Verse Berekenen → verwacht Instructie ~900W / Ieeftuimte ~1080W / Politiepost ~540W, totaal ~2520W (systeem D + WTW 80%). **Open vraag:** waarom blijft het result stale terwijl transmissie wél vers is — onderzoek de recompute-trigger (`/calculate_v2`-aanroep vanuit Results/save): wordt ventilatie bij élke Berekenen herrekend, of mist er een invalidatie na een q_v-edit? Zo niet → echte trigger-bug.
- [ ] **Rust `temperature_factor` `#[serde(default)]`** ontbreekt (`room.rs`); third-party clients zonder dit veld falen. Mapper vult het nu altijd, dus geen blocker.

---

## 🎯 Sprint v1.0 — BENG/TO-juli/koellast strategie (mei-juni 2026)

### Beschikbaar lokaal (`tests/references/`, gitignored)

- [x] **RVO Rekentool Bijlage AA NTA 8800 2025.04** (`rekentool-bijlage-aa-nta8800-2025.04.xlsm`) — officiële golden master voor BENG-koelbehoefte
- [x] **RVO BENG-voorbeeldconcepten woningbouw 2021** (`rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf`) — DGMR-rapport met 93 doorgerekende cases incl. TO-juli per concept
- [x] **DR Engineering Koellast woningbouw** (`dr-engineering-koellast-woningbouw-2024.pdf`) — Vabi 3.12.0.127, Ag 191.7 m², peak 6420 W
- [x] **Koellastberekeningen.nl Woning B** (`vabi-koellastberekeningen-woning-B-2024.pdf`) — Vabi 3.11.2.23, Ag 182.6 m², peak 8894 W, 17 pp gedetailleerd
- [x] **Vabi statistieken-export Woning C** (`vabi-koellast-statistieken-woning-C.xls`) — 3 ruimtes, 5260 W totaal voelbaar
- [x] **DR Engineering Koellast utiliteitsbouw** (`dr-engineering-koellast-utiliteitsbouw-2024.pdf`)
- [x] **Leever Utiliteit Horeca 2015** (`vabi-koellast-utiliteit-leever-2015.pdf` + `.xls`) — historisch NEN 5067:1985, structurele referentie

### Strategie — Bijlage AA Rekentool als golden master

Met de officiële RVO-rekentool kunnen we **onbeperkt fixtures genereren** zonder externe afhankelijkheden. Workflow:
1. Bijlage AA module implementeren in `crates/nta8800-cooling/src/bijlage_aa.rs` (formules AA.1-AA.13 + Tabel AA.3 lookup)
2. Per fixture-case: invoer in `rekentool-bijlage-aa-nta8800-2025.04.xlsm` → Rekentool output → `expected.json`
3. Onze engine runt met identieke input → vergelijk

DGMR-aanvraag is hiermee **niet meer nodig**.

### Implementatie

- [x] **Bijlage AA module in nta8800-cooling** (Bijlage AA NTA 8800:2025 concept, ~1300 LOC Rust)
  - [x] Formules AA.1 (P_int) t/m AA.13 (capaciteits-toets)
  - [x] Tabel AA.1 (θ_e per uur), AA.2 (f_iso per bouwjaar), AA.3 (I_sol 240 waarden)
  - [x] Per-room max-zoek over 9-18h × 8 oriëntaties × 5 hellingshoeken
  - [x] F_F (kozijnfactor, default 0.9) toegevoegd na cross-val (2026-05-28)
  - [x] **Cross-validatie tegen RVO-rekentool xlsm sample case 1** — groen binnen 0.07% (max 0.26 W op 377 W). Test: `golden_master_xlsm_cross_validatie`. Zie `tests/verification/INSTRUCTIES-bijlage-aa-cross-validatie.md` voor reproductie.
- [ ] **Peak-koellast engine** (separaat, EN 12831/NEN 5060 TO2) voor de Vabi Koellast cases
  - Twee fixture-cases met expected.json klaar: DR Engineering (6420W) + Koellastberekeningen.nl Woning B (8894W)
  - Statistieken-export Woning C als 3e fixture indien gewenst (kleinere case)
- [ ] **3 BENG-fixtures uit RVO voorbeeldconcepten** (Tussenwoning M, Hoekwoning M, Vrijstaande M)
  - Eindwaardes (BENG-1/2/3, TO-juli) staan in PDF
  - Volledige invoer-reconstructie via Rekentool xlsm
- [ ] **Utiliteitsbouw peak-koellast fixture** — folder + expected.json klaar (2026-05-28), wacht op peak-cooling engine

### Optioneel later

- [ ] ISSO 54 testset (BRL 9501 attestering, ~€1500 BouwZo trial) — alleen relevant voor formele software-attestering
- [ ] Uniec voorbeeldproject — Uniec is cloud-only SaaS, geen lokale bestanden mogelijk zonder DGMR-samenwerking

## 🎯 v1.0 Release Criteria

**Vastgelegd 2026-05-26.** v1.0 wordt uitgegeven wanneer onderstaande punten allemaal afgevinkt zijn. v0.2.0 (huidige tag) markeerde ISSO 51 feature-complete; v1.0 markeert het volledige platform (ISSO 51 + 53 + TO-juli) als productie-klaar.

### Blokkades

- [ ] **Alle test-fixtures aanwezig**
  - [x] Spoor 4 fixture-bundeling completeren — Bedrijfsruimte4 en 1.10a gedecomposeerd naar 1-op-1 Vabi-mapping, beide `#[ignore]` weg (sessie 14, 2026-05-29)
  - [ ] ISSO 53 batch 2d norm-verificatie afronden (infrastructuur klaar, verificatie pending)
  - _TO-juli Vabi-cross-validatie fixtures verschoven naar v1.1 (geen Vabi TO-juli PDF beschikbaar, sessie 14)_

- [ ] **Alle tests groen**
  - [ ] `cargo test` workspace — alle crates passend (isso51-core, isso53-core, nta8800-cooling, vabi-importer, ifcx)
  - [ ] `cd frontend && npm run build` slaagt
  - [ ] `cd frontend && npm test` slaagt (indien aanwezig)
  - [ ] CI groen op de release-commit

- [ ] **ISSO 53 productie-klaar**
  - [x] Vabi end-to-end verificatie op minimaal 2 reëele projecten binnen norm-tolerantie — 5 fixtures binnen ≤6% tol: Bedrijfsruimte4 (+3.6%), DR Kantoor West (+3.5%), 1.10a (+0.1%), 2.10a (+0.3%), 3.10a (+5.0%) (sessie 14, 2026-05-29)
  - [ ] Alle ISSO 53-specifieke UI-flows getest (norm-switch, utiliteit-velden, rapport)
  - [x] Geen `TODO:` of `FIXME:` in `crates/isso53-core/` en isso53-gerelateerde frontend code (commit `40b905c`, 2026-05-28)

- [ ] **TO-juli productie-klaar**
  - [ ] UI-flow `/tojuli` + `/tojuli-full` getest door user
  - _Vabi-cross-validatie groen op referentie-project — verschoven naar v1.1 (geen Vabi TO-juli PDF beschikbaar, sessie 14)_
  - _PDF-rapport TO-juli verifieerbaar tegen Vabi-uitvoer — verschoven naar v1.1 (sessie 14)_

### v1.1 doelen (post-v1.0)

- [ ] TO-juli Vabi-cross-validatie fixture vullen wanneer Vabi BENG/TO-juli PDF beschikbaar is (folder `tests/verification/tojuli_vabi3.12.0.127_dr-engineering-woningbouw/`)
- [ ] TO-juli PDF-rapport cross-val tegen Vabi-uitvoer
- [ ] Utiliteitsbouw peak-koellast fixture invullen wanneer peak-cooling engine af is
- [ ] 3 BENG-fixtures uit RVO voorbeeldconcepten (Tussenwoning M, Hoekwoning M, Vrijstaande M)
- [ ] ISSO 54 testset (optioneel, BRL 9501 attestering)

### Release-actie wanneer alles ✅
1. Versie bump → `1.0.0` in `Cargo.toml` workspace + `frontend/package.json` + `src-tauri/tauri.conf.json`
2. CHANGELOG sectie `[1.0.0]` met milestone-statement
3. Tag `v1.0.0` (annotated)
4. Tauri Windows-installer build via CI (`build-installer.yml`)
5. GitHub Release met installer als artifact + release notes

---

## Huidige focus: IFCX als universeel formaat + web-app IFC integratie

Zie `docs/ifc-herontwerp-verslag.md` sectie 10-11 voor het volledige implementatieplan.

---

## Fase 1: IFC Parser (Python sidecar) — GROTENDEELS KLAAR
- [x] Python project opzetten (`tools/ifc-tool/`) met IfcOpenShell
- [x] Import: IfcSpace → polygonen, verdiepingen
- [x] Storey clustering (nabije bouwlagen samenvoegen)
- [x] Polygon simplificatie pipeline
- [x] Shared edge detectie (binnenwanden herkennen)
- [x] Gap closing (polygonen uitbreiden naar wandhartlijn)
- [x] IfcWindow/IfcDoor extractie (hoogte, borstwering)
- [x] IfcWallType + materiaallagen extractie
- [x] PyInstaller bundeling
- [x] Tauri sidecar integratie
- [ ] Output converteren naar IFCX (i.p.v. bare JSON)
- [ ] Export command: IFCX → IFC4 SPF

## Fase 2: IFCX als universeel formaat — KLAAR
- [x] IFCX parser/writer crate in Rust (`crates/isso51-ifcx/`)
- [x] isso51:: namespace definitie (welke properties)
- [x] Mapper: bestaande Project types ↔ IFCX isso51:: namespace
- [x] isso51-core accepteert IFCX input, produceert IFCX output
- [x] REST API endpoint voor IFCX berekening (`POST /api/v1/calculate/ifcx`)
- [x] IFCX JSON schema in schema-endpoint (`GET /api/v1/schemas/ifcx`)
- [x] Adjacent room resolving (second pass, bidirectioneel)
- [x] Ground parameters mapping (`isso51::construction::ground`)
- [x] ProjectInfo metadata mapping (`isso51::project_info`)
- [ ] IFC parser output converteren naar IFCX (→ verplaatst naar Fase 3)

## Fase 3: Web-app IFC integratie
- [x] IFC parser als server-side service (Docker)
- [x] REST endpoint: `POST /api/v1/ifc/import` (file upload → JSON)
- [x] Frontend: IFC upload → server → modeller store (met web-ifc fallback)
- [ ] Modeller toont geïmporteerde ruimtes in 2D/3D
- [ ] Modeller → IFCX → isso51-core → resultaten

## Fase 4: Space Boundaries & Export
- [ ] 2nd level boundary lezer in IFC parser
- [ ] 1st level → 2nd level splitter
- [ ] Geometrie-based boundary calculator (Vabi-aanpak)
- [ ] Boundary UI in modeller
- [ ] IFC4 SPF export (met thermal psets)
- [ ] IFCX export met isso51::calc:: resultaten

## Fase 5: Herbruikbaarheid & distributie
- [ ] isso51-core als DLL (C ABI via cbindgen)
- [ ] isso51-core als WASM module
- [ ] isso51-core als Python package (PyO3)
- [ ] Modeller als standalone npm package
- [ ] API documentatie + IFCX namespace specificatie

---

## Bugs & correctheid
- [x] **PerFloorArea infiltratie bug** — gefixed (commit 7464e78)
- [x] **BBL ventilatie magic numbers** — gefixed, gebruikt nu `BBL_QV_*` constanten
- [x] **Runtime validatie server-responses** — `validateProjectResult()` toegevoegd, blinde casts vervangen in Projects.tsx, ConflictDialog.tsx, importExport.ts
- [x] **NTA 8800 drukmodel integratie (C2.3)** — gefixed, norm-exacte massabalans (§11.2.1) gewired in TO-juli rekenketen
- [x] #20 foutmelding server-opslag verbeterd (sessie-verlopen-detectie) — root-cause nog open
- [x] **Jaarverbruik schatting (graaddagen-methode)** — nieuwe Results-veld toont geschat netto jaarverbruik via H_extern × HDD_NL × 24/1000 met expliciete disclaimer (commit 8458a5a)

## Thermal-import — Revit-exporter audit follow-ups (2026-05-22)

> Uit de read-only audit van de PyRevit warmteverlies-exporter. Deze items vereisen éérst een schema-uitbreiding aan deze kant; daarna kan de exporter ze vullen. Exporter-zijdige items staan in de pyRevit-repo `TODO.md`.
- [ ] D3 — optioneel `u_value`/`rc` per construction in `schemas/v1/thermal-import.schema.json` + deserialisatie in `crates/isso51-core/src/import/thermal.rs` → Rc-calculatorstap voor-ingevuld i.p.v. U=0 placeholder
- [ ] D4 — `sfb_code` per construction in schema + `thermal.rs` → betere catalog-groepering; NLRS/SfB-parameter komt uit het Revit-type
- [x] Construction-catalog refactor (`docs/thermal-import-construction-catalog-spec.md`) — geverifieerd volledig geïmplementeerd in `thermal.rs` + frontend; spec-status mag van "Approved" naar "Implemented"

## Verificatie & testing
- [x] Vabi vrijstaande woning test fixture (9 kamers, 110 constructies, verwachte resultaten)
- [x] DR Engineering woningbouw test fixture
- [x] ISSO 51 portiekwoning test fixture
- [ ] Referentieberekeningen cross-valideren met python-hvac (EN 12831)
- [ ] Kwadratische sommatie unit test: sqrt(101² + 651²) = 659 W

## Code kwaliteit — Rust
- [ ] Constanten definiëren: `RHO_CP_AIR = 1.2`, `GROUND_CORRECTION_FACTOR = 1.45`, `R_SI_*`, `R_SE_*`
- [ ] DRY: `default_one()`/`default_true()` naar gedeeld module
- [ ] DRY: SQL upsert user naar gedeelde functie (handlers/user.rs + handlers/projects.rs)
- [ ] Dead code opruimen: `ventilation_requirement_living()`, `ventilation_requirement_wet_room()`, ongebruikte error varianten
- [ ] Infiltratie tabelnotatie vereenvoudigen (`0.08` ipv `0.08e-3 * 1000.0`)
- [ ] VentilationConfig validatie toevoegen (bijv. heat_recovery_efficiency > 1.0)

## UI / Theming — light theme afmaken
**Status:** Echte light theme staat sinds 2026-05-16 op master (`a88999e`); 3 themes via Settings → Uiterlijk werken via `var(--theme-*)`.
- **2026-05-17 (`12de603`):** `--oaec-*` tokens binnen `[data-theme="light"]` in `themes.css` overschreven (17 vars, gemapt naar `--theme-*`). Lost de `#44444C` cards en `#2E2E36` inputs op voor `/project` (ProjectSetup → AlgemeenTab) en bij Vertrekken (RoomTable). Upstream PR: `OpenAEC-Foundation/openaec-ui#1` (token-split + v0.2.0) — bij merge `package.json` bumpen en het lokale override-blok kan dan verdwijnen.
- Resterend: import-wizard files gebruiken hardcoded Tailwind dark-utility classes (`bg-gray-800/*`, `border-gray-*`) en negeren daardoor zowel `--theme-*` als `--oaec-*`. Zichtbaar in `/import/thermal` flow.
- [ ] `components/import/ConstructionImportStep.tsx` — vervang `bg-gray-800/50`, `border-gray-700`, `bg-gray-700/60` door theme-aware (`var(--theme-surface)`, `var(--theme-border)`, `var(--theme-bg-lighter)`)
- [ ] `components/import/FileUploadStep.tsx` — idem (`bg-gray-800/50`, `border-gray-600`, `bg-gray-700`, `border-gray-700`)
- [ ] `components/import/ImportSummary.tsx` — idem (`bg-gray-800/50`, `border-gray-700`)
- [ ] `components/import/OpeningImportStep.tsx` — idem (`bg-gray-800/{30,40,80}`, `border-gray-{600,700}`, `text-gray-{400,500,600}`, `placeholder-gray-600`)
- [ ] `components/import/RoomImportStep.tsx` — idem (`bg-gray-800/{40,80}`, `border-gray-{600,700}`, `text-gray-{400,500}`)
- [ ] `components/import/ThermalImportWizard.tsx` — idem (`bg-gray-{700,800}`, `border-gray-{500,600,700}`, `text-gray-{300,400}`)
- [ ] `components/layout/Topbar.tsx` — `bg-[#27272A]` hover-states (regels 70/103/112/119) → `var(--theme-hover-strong)`. **Eerst checken of Topbar nog actief is** — volgens CLAUDE.md UI-migratie is hij vervangen door TitleBar+Ribbon; mogelijk dead code (verwijderen i.p.v. fixen).
- [ ] Sweep-strategie: per file beoordelen of theme-aware classes (via `:where([data-theme="light"]) .X { ... }` in component.css) of inline CSS-vars (`style={{ background: "var(--theme-surface)" }}`) de schoonste route is. Inline vars zijn pragmatischer voor de import-wizard (Tailwind utility-overflow).
- [ ] Acceptance: in light mode geen `bg-gray-*` zichtbaar; switch tussen 3 themes verandert alle wizard-screens.

## Code kwaliteit — Frontend
- [ ] `MATERIAL_TYPE_LABELS` centraliseren naar `constants.ts` (nu 3x gedupliceerd)
- [ ] `niceMax()` utility centraliseren (nu 4x gedupliceerd in chart/svg bestanden)
- [ ] `FUNCTION_COLORS` centraliseren (nu 3x gedupliceerd in modeller)
- [ ] `Library.tsx` (1052 regels) splitsen in component-bestanden
- [ ] `FloorCanvas.tsx` (1729 regels) splitsen: shapes, room rendering, drawing, utils
- [ ] Dead code verwijderen: `ModellerToolbar.tsx`, `DrawingToolsPanel.tsx` (vervangen door Ribbon)
- [ ] Store snapshot mist constructie-assignments (undo/redo verliest wall/floor/roof toewijzingen)

## Cloud integratie — BACKEND KLAAR
- [x] `openaec-cloud` dependency (gedeelde Nextcloud cloud crate)
- [x] Multi-tenant config (`TENANTS_CONFIG`, `DEFAULT_TENANT` env vars)
- [x] `GET /api/v1/cloud/status` — cloud storage beschikbaarheid
- [x] `GET /api/v1/cloud/projects` — projecten uit Nextcloud
- [x] `GET /api/v1/cloud/projects/{project}/models` — IFC bestanden
- [x] `GET /api/v1/cloud/projects/{project}/calculations` — berekeningen
- [x] `POST /api/v1/cloud/projects/{project}/save` — berekening opslaan + manifest update
- [ ] Server-side deployment: volume mount + env vars in docker-compose
- [ ] Frontend: cloud storage browser in de UI
- [ ] Frontend: "Opslaan naar cloud" knop in Backstage/resultaten

## App features
- [x] OIDC login/logout op productie
- [x] Projecten opslaan/laden
- [x] Vertrekken invoer + bewerken
- [x] Resultaten weergave + grafieken
- [x] JSON import/export
- [x] Rc-calculator met laag-editor
- [x] Rc-calculator: inhomogene lagen (ISO 6946 combined method) + bevestigingsmiddelencorrectie (Annex F)
- [x] Glaser-analyse + diagram
- [x] Constructiebibliotheek + materialendatabase
- [x] PDF rapportgeneratie
- [x] Conflict detectie (optimistic locking)
- [x] Auto-save + dark/light theme
- [ ] Materialen: inline bewerken, lambda nat, zoekwoorden
- [x] U_w kozijn-calculator Fase 1: `uw_breakdown`-datamodel + `Spacer`-enum (`7727e79`)
- [x] U_w kozijn-calculator Fase 2: `uwCalculation.ts` + spacer-tabel + `/uw`-calculatorpagina
- [x] U_w kozijn-calculator Fase 3: opslaan op kozijn-element + opbouw in project-rapport + zelfstandig U_w-rapport
- [x] U_w kozijn-calculator: fabrikant-catalogus (profiel/glas) + Ψ_g-correctie naar EN-ISO 10077-1 Annex E-richtwaarde
- [x] U_w kozijn-calculator: afronding — setTimeout-cleanup, edit-param-feedback, catalogus-herkomst persistent in rapport
- [x] #21 rekenexpressies (=1,5*2,6) in numerieke tabelcellen

## Modeller features
- [x] 2D/3D modeller met pan/zoom, grid, polygonen, wanden, ramen, deuren
- [x] Ribbon toolbar, teken-tools, snap, meten
- [x] Room splitsen/samenvoegen/verplaatsen
- [x] Constructiebibliotheek koppelen, boundary override
- [x] Onderlegger import, undo/redo, verdiepingen, context menu
- [x] IFC import (IfcSpace → ModelRoom)
- [x] IFC Phase 2: window/door hoogte extractie
- [x] IFC Phase 3: storey clustering, polygon simplificatie, shared edges, gap closing
- [ ] Modeller data ↔ IFCX synchronisatie
- [ ] PDF/DWG onderlegger
- [ ] Schuine daken en dakkapellen

## Architectuur / open ontwerpen
- [ ] **Zone-model ADR** — `docs/2026-05-23-zone-model-adr.md` — ontwerp voor mixed-use support via norm-keuze per rekenzone (spike/draft)

## Roadmap — toekomst
- [ ] BAG-data import (postcode + huisnummer)
- [ ] Quick-calc wizard (5-10 min berekening)
- [ ] ISSO 53 (utiliteitsgebouwen)
  - [x] Batch 1: skelet + model-setup (`crates/isso53-core/`)
  - [x] Batch 2a: opzoektabellen (11 tabel-modules in `tables/`)
  - [x] Batch 2b: calc-kern (theta_i, q_h,nd)
  - [x] Batch 2c: orkestratie + CLI werkend
  - [x] Batch 2d: test fixtures + verificatie — infrastructuur klaar, norm-verificatie pending
  - [x] **ISSO 53 UI-spoor** — dual-calc support in bestaande web-app (COMPLEET)
    - [x] Fase 1: backend dual-pipeline (KLAAR — commit 86e8ab6)
    - [x] Fase 2: norm-keuze UI + topbar-badge (KLAAR — commit 8ffa728)
    - [x] Fase 3: conditional rendering bestaande screens (KLAAR — commit 28c429f)
    - [x] Fase 4: wissel-flow met waarschuwing (KLAAR — commit e697c97)
    - [x] Fase 5: isso53-report-builder (KLAAR — commit 7d8a307)
  - [x] **ISSO 53 - calc-core warmteverlies sporen** — AFGESLOTEN sessie 8 (2026-05-25)
    - [x] **§4.6 embedded heating clause geïmplementeerd** (commit 0f4293a)
      - phiT: 4385→2918 W vs Vabi 2919 W (<0.1% afwijking) ✅
      - f_ig = 0.0 voor elementen met has_embedded_heating = true
    - [x] **Adjacent-room transmissie sporen 1/2/3** — OPGELOST via Optie C wrapper-schrap (sessie 8)
      - Dubbeltelling adjacent-room-bijdrage weg (5-7% overschatting gefixed)
      - Tests: 92 passed / 0 failed / 4 ignored
    - [x] **Spoor 4 fixture-artefact** — GEDIAGNOSEERD en GEDOCUMENTEERD (PDF_GAPS.md)
      - Plan-agent bewijs: gap zit in fixture-bundeling, niet calc-core algoritme
      - Norm-conforme implementatie formule 4.18 bevestigd
  - [x] **ISSO 53 - "toekomstige sporen" geverifieerd norm-conform** (2026-05-26)
    - [x] **WTW ventilatie** — implementatie was al norm-conform (ISSO 53 §4.7.2 formule 4.38)
      - Verificatie: f_v ≈ 0.15 bij η_wtw=85% → ~85% reductie van Φ_V (test `test_wtw_ventilation_efficiency_applied` in `calc/ventilation.rs`)
      - "phiV = 3076 W" was absolute waarde bij groot debiet, niet bewijs van bug
    - [x] **Infiltratie systeem-D** — ISSO 53 tabel 4.7 schrijft f_inf=1.15 voor SystemD vs 0.80 voor SystemA
      - Hogere infiltratie bij balanced ventilation is fysisch correct (ventiel-drukverschillen)
      - Regressie-test: `test_systemd_infiltration_norm_compliant` in `calc/infiltration.rs`
- [ ] ISSO 57 (vloerverwarming)
- [ ] Radiatorselectie + hydraulische balancering
- [ ] R3F viewer migratie (ThatOpen → React Three Fiber)
- [ ] Multi-user: projecten delen, rollen
- [ ] Template-projecten: veelvoorkomende woningtypes
