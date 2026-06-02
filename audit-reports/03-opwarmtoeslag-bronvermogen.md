# Audit 03 — Opwarmtoeslag (Φ_HU), bronvermogen & ruimte-aggregatie

**Scope:** ISSO 53 (2016) §4.8 (toeslag bedrijfsbeperking), §5.1/§5.2 (aansluitvermogen), §4.1 (per-vertrek aggregatie).
**Norm-PDF:** `ISSO-publicatie 53 ... vertrekhoogten tot 4 meter.pdf`
**Crate:** `isso53-core` — `calc/heating_up.rs`, `model/heating_up.rs`, `tables/heating_up.rs`, `tables/thermal_mass.rs`, `calc/source_capacity.rs`, `tables/source_fraction.rs`, `calc/room_load.rs`, `lib.rs` (aggregatie).
**Datum:** 2026-06-02 | **Mode:** read-only, geen broncode gewijzigd.

**Samenvattend oordeel:** de kern-rekenkern is norm-conform. Tabellen 4.13/4.14, formules 4.43/4.45/5.1/5.2/5.9 en het uitgewerkte voorbeeld p.66 reproduceren exact. De bevindingen draaien om **stille nul-uitkomsten** (serde-defaults + `unwrap_or(0.0)`), de **ontbrekende gelijktijdigheids-selectie** die de norm expliciet eist, en een **mis-toepassing van de §4.8.3-reductie bij natuurlijke ventilatie**.

---

## Kritieke conformiteitsfouten

Gesorteerd op numerieke impact. Geen enkele tabel- of formulewaarde is fout — de fouten zitten in randvoorwaarden, defaults en domein-gating.

### K1 — §4.8.3-reductie wordt toegepast bij ruimten zónder mechanische toevoer (over-reductie → Φ_hu te laag)

| Veld | Waarde |
|------|--------|
| Bestand:regel | `calc/heating_up.rs:106-110` |
| Norm-clausule | §4.8.3, p.54, formule 4.45 + definitie `a` |
| Norm-tekst | `a = 1 bij opwarmen zónder mechanische toevoer van buitenlucht`; `Φ_hu,i = Φ_op` bij *systemen zonder mechanische toevoer*. De reductie `−a·H_v·(θ_i−θ_e)` geldt alléén bij *mechanische* toevoer die in de nachtstand/uit gaat. |
| Code-waarde | `mechanical_supply_off` is één project-breed bool in `HeatingUpConfig`, uniform op elke ruimte toegepast. Voor een ruimte met natuurlijke toevoer (q_v > 0, f_v = 1, dus H_v > 0) trekt de code alsnog `H_v·(θ_i−θ_e)` af zodra de project-vlag aan staat. |
| Numerieke impact | Bij natuurlijk geventileerde ruimte met H_v = 10 W/K, Δθ = 30 K → −300 W ten onrechte afgetrokken van Φ_op; kan Φ_hu naar 0 clampen. **Onderschat het opgestelde vermogen** — risico op te kleine afgifte/opwarmtijd-overschrijding. |
| Root-cause | De §4.8.3-tak hoort gekoppeld te zijn aan de aanwezigheid van mechanische toevoer per ruimte (vgl. `Room.has_mechanical_supply`), niet aan een globale vlag. De norm spreekt over "systemen met mechanische toevoer". |
| Fix | Reductie alleen toepassen als de ruimte daadwerkelijk mechanische toevoer heeft die uitgeschakeld wordt: gate op `room.has_mechanical_supply != Some(false)` (én project-vlag). Voor natuurlijke ventilatie → `Φ_hu,i = Φ_op` (de tak zonder reductie). |

### K2 — Ontbrekende gelijktijdigheids-selectie van Σ Φ_hu (overdimensionering, norm-eis genegeerd)

| Veld | Waarde |
|------|--------|
| Bestand:regel | `lib.rs:93` (`total_heating_up += phi_hu`), `calc/source_capacity.rs:38,79` (`phi_hu_total += room.phi_hu`) |
| Norm-clausule | §4.1 p.38; §3.1 p.27; §5.1 p.55 — herhaald: *"alleen die toeslagen voor bedrijfsbeperking die gelijktijdig optreden"* |
| Norm-tekst | "Om overdimensionering te voorkomen moeten alleen die toeslagen voor bedrijfsbeperking in rekening gebracht worden die gelijktijdig optreden. Het is van belang hierover met de opdrachtgever afspraken te maken." |
| Code-waarde | Engine sommeert `Φ_hu` van **alle** ruimten onvoorwaardelijk, zonder gelijktijdigheids-factor of selectie-mechanisme. |
| Numerieke impact | Bij zones met verschillende opwarmregimes (bv. niet alle vleugels starten tegelijk) wordt het aansluitvermogen `Φ_source` structureel te hoog → overdimensionering opwekker, hogere investering. Grootteorde = de niet-gelijktijdige Φ_hu-bijdragen, kan tienprocenten van Σ Φ_hu zijn. |
| Fix | Mechanisme aanbieden om per zone/ruimte een gelijktijdigheidsvlag of -groep te zetten; minimaal documenteren dat de engine **100% gelijktijdigheid** aanneemt zodat de gebruiker dit bewust met de opdrachtgever afstemt. Idem voor systeemverliezen/warmtewinsten (§3.8/§3.9). |

### K3 — Stille nul bij ongeldige tabel-combinatie (`unwrap_or(0.0)`)

| Veld | Waarde |
|------|--------|
| Bestand:regel | `calc/heating_up.rs:97` (`specific_supplement(...).unwrap_or(0.0)`) |
| Norm-clausule | §4.8.1/§4.8.2, tabel 4.13/4.14, p.53 |
| Norm-waarde | Φ_hu,i moet uit de tabel komen; een combinatie buiten de gedefinieerde assen is een **invoerfout**, geen 0 W/m². |
| Code-waarde | `free_col`/`limited_col` geven `None` bij `setback_hours ∉ {8,14,62}` of `degrees ∉ {1..5}` → `specific_supplement` → `None` → `unwrap_or(0.0)` → **Φ_hu = 0 zonder waarschuwing**. |
| Numerieke impact | Een client die bv. `setbackHoursWeekday: 10` of `degreesWeekday: 6` stuurt krijgt geleidelijk Φ_hu = 0 i.p.v. een error → opwarmtoeslag verdwijnt geruisloos uit de berekening. |
| Fix | Bij `None` een `Err(Isso53Error::…)` retourneren (of validatie in `validate.rs`) i.p.v. stil naar 0 vallen. De legitieme "geen toeslag"-paden (adaptieve regelaar/continubedrijf, §4.8 p.34) lopen al via `setback_active=false` — die hoeven niet via de unwrap-tak. |

### K4 — `#[serde(default)]` op `Project.heating_up` → stille Φ_hu = 0 bij ontbrekend blok

| Veld | Waarde |
|------|--------|
| Bestand:regel | `model/project.rs:27-28` + `model/heating_up.rs:135-147` (`Default { setback_active: false }`) |
| Norm-clausule | §4.1 p.38 (Φ_hu is een verplichte bijdrage "indien van toepassing") |
| Code-waarde | Ontbreekt het `heatingUp`-blok in de JSON, dan default `setback_active=false` → Φ_hu = 0 voor het hele gebouw, zonder error of waarschuwing. |
| Numerieke impact | Third-party importers (cURL, andere mappers) die het blok weglaten krijgen een **systematisch te laag aansluitvermogen** — de hele opwarmtoeslag valt weg. In utiliteit met nachtverlaging is dit doorgaans 10-25% van Φ_source (vgl. voorbeeld p.66: 378 van 1.339 W ≈ 28%). |
| Fix | Overweeg het blok verplicht te maken, of bij `serde(default)` een waarschuwing/diagnostic in het resultaat opnemen ("heating-up niet opgegeven, aangenomen continubedrijf"). Consistent met de openstaande `heating_system`-default-discussie in MEMORY. |

---

## Risico in nieuwe heating_up-module

| # | Risico | Locatie | Beoordeling |
|---|--------|---------|-------------|
| R1 | §4.8.3-reductie ontkoppeld van werkelijke mechanische-toevoer-aanwezigheid per ruimte | `calc/heating_up.rs:106` | **Hoog** — zie K1. Over-reductie bij natuurlijke ventilatie. |
| R2 | `unwrap_or(0.0)` maskeert ongeldige invoer | `calc/heating_up.rs:97` | **Hoog** — zie K3. |
| R3 | `c_eff`-zwaarteklasse uit discrete `ThermalMass`-enum (15/50/75) i.p.v. werkelijk berekende c_eff (formule 4.44 = C_eff/V) | `calc/heating_up.rs:95` + `tables/thermal_mass.rs:17` | **Midden** — zie T1. Grensgevallen rond c_eff = 70 kunnen verkeerd in l/z vallen. |
| R4 | `interpolate_column` valt bij één-zijdig-`None` terug op de gedefinieerde cel i.p.v. te interpoleren naar de tabelgrens | `tables/heating_up.rs:184-188` | **Laag** — pragmatisch en conservatief; norm geeft geen interpolatie-voorschrift voorbij de `-`-grens. Reproduceert het p.66-voorbeeld correct. Documenteren als bewuste keuze. |
| R5 | `mechanical_supply_off` is project-breed, niet per ventilatiesysteem/zone | `model/heating_up.rs:123` | **Midden** — combineert met K1; in gemengde gebouwen (deels natuurlijk, deels mechanisch) is één vlag te grof. |
| R6 | Override `p_w_per_m2_override` ondergaat wél de §4.8.3-reductie | `calc/heating_up.rs:89-110` | **Laag/correct** — consistent met de norm (override vervangt alleen φ_hu,i [W/m²], de reductie blijft fysisch geldig). Bevestigd door test `test_manual_override` (reductie uit). |
| R7 | Σ Φ_hu zonder gelijktijdigheid | `lib.rs:93`, `source_capacity.rs:38/79` | **Hoog** — zie K2. |

---

## Twijfelgevallen (norm-clausule te verifiëren)

### T1 — Zwaarteklasse l/z uit tabel-2.4-default i.p.v. werkelijke c_eff

`BuildingWeight::from_c_eff` (correct: `≤70 → l`, p.53) krijgt zijn c_eff uit `tables/thermal_mass.rs::c_eff(ThermalMass)`, dat slechts drie discrete waarden teruggeeft (Licht=15, Gemiddeld=50, Zwaar=75 — tabel 2.4, p.24). §4.8.1 schrijft echter de **werkelijk berekende** c_eff voor (formule 4.44 = C_eff/V uit §2.6.1). Voor de l/z-keuze is dit zelden kritisch (50 ≪ 70 → l; 75 > 70 → z), dus de discrete mapping landt nu altijd goed. **Maar:** een gebouw met werkelijke c_eff = 65 (zwaar-aandoende constructie) zou via de enum als `Gemiddeld` (50) óf `Zwaar` (75) worden geclassificeerd afhankelijk van de UI-keuze, terwijl 65 ≤ 70 → l is. Risico ontstaat pas als de UI een vierde "echte c_eff"-invoer toelaat. **Te verifiëren:** of de frontend ooit een vrije c_eff doorgeeft; zo niet, dan is dit puur een toekomst-risico.

### T2 — `H_v` mét f_v in de §4.8.3-reductie

De code gebruikt `ventilation_result.h_v`, dat de WTW/voorverwarming-factor `f_v` al bevat (`h_v = q_v·1200·f_v`, `ventilation.rs:38`). Ik heb geverifieerd tegen het **uitgewerkte voorbeeld p.66**: daar wordt H_v = 27,8·10·1200·0,2 = 6,672 W/K (mét f_v = 0,2) gebruikt in `Φ_hu,i = 568 − 1·6,672·28,5 = 378 W`. De norm refereert expliciet "H_v volgens paragraaf 4.7.2" = de f_v-gereduceerde waarde. **Conclusie: correct** — maar fysisch contra-intuïtief (bij uitgeschakelde toevoer is er geen WTW-werking), dus opgenomen als twijfelgeval ter borging. De norm rekent bewust met de operationele H_v; code volgt de norm letterlijk. Geen fix nodig.

### T3 — `total_building_heat_loss` (Σ Φ_HL,i) zónder z-reductie vs. `connection_capacity_*` mét z

`lib.rs:96` sommeert ruwe ruimte-totalen (infiltratie zonder z), terwijl `source_capacity` z = 0,5/1,0 toepast (formule 5.2). Dit zijn **twee verschillende grootheden** (Σ vertrek-vermogen vs. aansluitvermogen) en beide zijn norm-conform. **Te verifiëren in rapportage-laag:** dat het rapport deze twee niet als "hetzelfde gebouwtotaal" naast elkaar zet zonder uitleg — anders lijkt het een inconsistentie. Geen kern-bug.

### T4 — `f_v` op gebouwniveau (formule 5.3) vs. som van ruimte-H_v

Formule 5.3 (p.55) definieert `H_v,build = q_v,build·1200·f_v` met `q_v,build` mogelijk **kleiner** dan Σ q_v per ruimte bij vraagsturing (CO₂-regeling). `source_capacity.rs:43/83` sommeert echter `room.h_v` per ruimte. Voor systemen zónder vraagsturing is Σ H_v,room = H_v,build (identiek). **Te verifiëren:** of de engine vraaggestuurde ventilatie ondersteunt; zo ja, dan overschat de som het gebouw-ventilatieverlies licht. Buiten de directe heating_up-scope, maar raakt dezelfde `Φ_Ven`-term.

---

## Geverifieerd correct (kort)

| Onderdeel | Bestand:regel | Norm-ref | Bevinding |
|-----------|---------------|----------|-----------|
| Tabel 4.13 (vrije afkoeling, 7×12) | `tables/heating_up.rs:74-89` | tabel 4.13, p.53 | **Bit-perfect** — alle 84 cellen + `-`/`None` matchen de PDF. |
| Tabel 4.14 (beperkte afkoeling, 5×20) | `tables/heating_up.rs:104-115` | tabel 4.14, p.53 | **Bit-perfect** — alle 100 cellen + `-`/`None` matchen. |
| Kolom-ordening (uren×luchtw×zwaarte) | `tables/heating_up.rs:119-148` | header p.53 | Layout `8/0,1/l … 62/0,5/z` correct gemapt; `(idx*4 + air*2 + weight)`. |
| Formule 4.43 (Φ_op = A_vl · φ_hu,i) | `calc/heating_up.rs:102` | §4.8, p.52 | Correcte vermenigvuldiging W/m² × m² → W. Geen eenheidsfout. |
| Formule 4.45 (§4.8.3, clamp ≥0) | `calc/heating_up.rs:106-110` | §4.8.3, p.54 | Structuur + `.max(0.0)` correct; tak "zonder mechanische toevoer" → Φ_op correct. (Toepassings-gating: zie K1.) |
| max(doordeweeks, weekend) | `calc/heating_up.rs:48-53` | p.66 "hoogste van de twee is maatgevend" | Correct, incl. `None`-tolerantie. |
| Drempel c_eff ≤ 70 → l | `tables/heating_up.rs:40-46` | §4.8.1, p.53 | Grens correct; `Gemiddeld`(50)→l, `Zwaar`(75)→z. |
| Uitgewerkt voorbeeld p.66 | `calc/heating_up.rs:183-223` (regressie-test) | §6.2, p.66 | φ = 28 W/m², Φ_op = 568 W, Φ_hu = 378 W — **reproduceert exact**. Verplichte gate aanwezig. |
| Formule 5.1 (individueel) | `calc/source_capacity.rs:16-54` | §5.1, p.55 | Σ Φ_T,ie + Φ_T,iae + Φ_T,iaBE + Φ_T,ig + Φ_Ven + Σ Φ_hu − Σ Φ_gain; Φ_T,ia (verwarmde buur) correct **uitgesloten**. |
| Formule 5.9 (collectief) | `calc/source_capacity.rs:59-95` | §5.2, p.58 | Identiek aan 5.1 maar **zonder** Φ_T,iaBE — correct geïmplementeerd; test `test_individual_vs_collective_difference` borgt het verschil. |
| Formule 5.2 (z alleen op infiltratie) | `calc/source_capacity.rs:47,88` | §5.1.1, p.55 | `z·Σ(H_i·Δθ) + Σ(H_v·Δθ)` — z **alleen** op H_i, niet op H_v. Correct. |
| Tabel 5.1 (z = 1,0 / 0,5) | `tables/source_fraction.rs:33-38` | tabel 5.1, p.57 | `SeparatePerZone → 1,0`, `Other → 0,5` — exact. |
| Formule 4.1 (per-vertrek aggregatie) | `calc/room_load.rs:60-62` | §4.1, p.38 | `Φ_T + Φ_V + Φ_I + Φ_hu − Φ_gain`; Φ_V correct gesplitst in vent + infiltratie (formule 4.42). Geen dubbeltelling met §4.8.3-reductie (die haalt juist het vrijgekomen ventilatiedeel eraf). |
| serde camelCase roundtrip | `model/heating_up.rs:159-275` | — | `CoolingRegime`/`HeatingUpConfig` deserialisatie van mapper-shape geborgd; bekende `rename_all`-valkuil op enum-velden expliciet afgevangen met `#[serde(rename=...)]`. |

---

## Aanbevolen prioriteit

1. **K1** (§4.8.3-gating per ruimte) — fysiek onjuiste over-reductie, raakt elke gemengd-geventileerd of natuurlijk-geventileerd project met de vlag aan.
2. **K3 + K4** (stille nul via `unwrap_or` + serde-default) — onderschat het vermogen zonder enige waarschuwing; gevaarlijkste klasse fouten (geen error, fout antwoord).
3. **K2** (gelijktijdigheid) — overdimensionering; minder gevaarlijk dan onderschatting maar expliciete norm-eis.
4. T1/T4 zijn toekomst-risico's afhankelijk van wat de frontend doorgeeft — verifiëren vóór ze relevant worden.
