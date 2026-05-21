# ISSO 53 — Implementatie-spec voor `isso53-core`

**Auteur:** Orchestrator / PM
**Datum:** 2026-05-16
**Status:** Concept voor rust-developer (eerste milestone)
**Bron-norm:** ISSO-publicatie 53 (2016, "Warmteverliesberekening voor utiliteitsgebouwen met vertrekhoogten tot 4 meter")
**PDF-pad:** `C:/DATA/3BM_projecten/50_projecten/7_3BM_bouwkunde/000_Documentatie/98_normen/ISSO-publicatie 53 Warmteverliesberekening voor utiliteitsgebouwen met vertrekhoogten tot 4 meter.pdf`

> **Doel van dit document:** een Rust-developer moet op basis hiervan `crates/isso53-core` kunnen bouwen (eerste milestone: rekenkern + CLI + JSON-fixtures) zonder de hele norm zelf te hoeven herlezen. Letterlijke formulenummers verwijzen naar de PDF; voor numerieke details die hier niet expliciet staan: lees de aangegeven paragraaf in de PDF.

---

## 1. Scope & milestone (vastgepind door PM)

| Aspect | Beslissing |
|---|---|
| Architectuur | **Parallelle nieuwe crate** `crates/isso53-core` naast bestaande `isso51-core`. Géén shared core, géén switch-in-bestaande-crate |
| MVP | **Alleen rekenkern + CLI binary + JSON test fixtures**. Geen UI, geen Tauri, geen IFCX, geen REST API in deze fase |
| Duplicatie | Acceptabel — gedeelde NEN-EN 12831-bouwstenen mogen worden gekopieerd van `isso51-core`. Latere extractie naar shared crate is een aparte beslissing |
| Conventies | Identiek aan `isso51-core` (pure Rust, geen I/O/async/unsafe, JSON in/uit, schemars voor JSON Schema) |
| Eenheden | mm voor afmetingen, m² voor oppervlakte, m³/s voor luchtvolumestroom, W voor vermogen, W/K voor H-waarden, °C voor temperaturen |
| Norm-conventie | Symbolen en indices volgens NEN-EN 12831-1 (zelfde als ISSO 51) |

**Out-of-scope voor deze milestone (later, aparte spec):**
- IFCX-namespace `isso53::*`
- Frontend / UI / norm-keuze in project-aanmaak
- REST API endpoints
- IFC-import voor utiliteit (verschilt van wonen)
- BCF/cloud-integratie
- Wrapper crates (Python/WASM/FFI)
- ISSO 57 (>4m vertrekhoogte) — die heeft eigen crate later

---

## 2. Verschillen ISSO 51 vs ISSO 53 (impact op core)

| Aspect | ISSO 51 (woningen) | ISSO 53 (utiliteit ≤4m) | Impact op `isso53-core` |
|---|---|---|---|
| **Vertrekhoogte** | Geen expliciete grens (woningen zijn beperkt) | ≤ 4 m, daarboven verwijzen naar ISSO 57 | Validatie: error bij `height > 4.0` met verwijzing naar ISSO 57 |
| **Ontwerpbinnentemp.** | Per kamerfunctie (Woonkamer 20, Slaapkamer 18, Badkamer 22…) | Per **gebruiksfunctie × ruimtetype** (tabel 2.2 + bijlage G). Voorontwerp: 22 °C zorg / 20 °C overig (§3.1) | Nieuw enum `GebruiksFunctie` + `RuimteType`; nieuwe lookup-tabel |
| **Infiltratie** | `qv10` per woning, `PerExteriorArea` / `PerFloorArea` methodes | `q_is` per m² gevel via tabel 4.5 (als `q_v10,kar` bekend) **of** formule 4.31 met windcorrectie + gebouwtype + leeftijd + ventilatiesysteem | Nieuwe `InfiltrationMethod`-enum varianten; nieuwe tabellen 4.5–4.9 |
| **Windcorrectie infiltratie** | Niet expliciet | `f_wind`, `f_type`, `f_inf`, `f_jaar` (formules 4.32–4.34, tab 4.6–4.7) | Nieuwe `calc::infiltration` module — niet kopie van 51 |
| **Reductiefactor z (vertrek)** | n.v.t. | Tabel 4.4: 1.0 / 0.5 / 0.7 afhankelijk van gevelconfiguratie vertrek | Nieuw veld op `Room`: `infiltration_reduction_z` (of enum) |
| **Ventilatie-eis** | Per m² (Bouwbesluit-wonen, vaste rate per kamerfunctie) | **dm³/s per persoon × personen/m²** per gebruiksfunctie (tabel 4.10) | Nieuwe ventilatie-tabel + bezetting-veld op vertrek |
| **Bezettingsdichtheid** | n.v.t. | Tabel 4.11 (TODO: niet in mijn PDF-extract — Rust-dev: lees p51) personen/m² als default | Nieuw veld + lookup-tabel `tables::occupancy` |
| **f_k onverwarmde ruimte** | ISSO 51 eigen tabel | Tabel 4.2 (kelder, ruimte onder dak, gem. verkeersruimte, vloer boven kruipruimte) — andere indeling | Nieuwe tabel, vergelijkbare structuur |
| **Bedrijfsbeperking** | `warmup_time` + main-room percentage methode (kwadratisch erratum) | **P [W/m²] specifieke toeslag** per gebruiksfunctie + opwarmtijd (§4.8) | Andere formule en datamodel — TODO uitwerken |
| **Gebouwsommatie** | Erratum 2023: kwadratische sommatie `√(Φ_vent² + Φ_T,iaBE² + Φ_hu²)` op gebouwniveau | Eenvoudige optelling (formule 5.1) — **geen kwadratische sommatie** voor utiliteit | Simpelere `build_summary` — geen `quadratic_sum` module |
| **Schilmethode** | n.v.t. | **Expliciete vereenvoudigde voorontwerp-methode** (hoofdstuk 3) — gebouw als één vertrek | Aparte `calc::shell` module |
| **Aansluitvermogen** | Φ_basis + Φ_extra (kwadratisch) | Sommatie van componenten (5.1), géén kwadratisch verschil, wél aparte aftrek z·Σ(H_i·Δθ) (formule 5.2) | Aparte `calc::source_capacity` module |
| **Collectief vs individueel** | Eén `connection_capacity` + `collective_contribution` (woningscheidend uitsluiten) | Twee methodes (§5.1 individueel, §5.2 collectief — collectief sluit `Φ_T,iaBE` uit) | Vergelijkbare structuur, andere formule |
| **f_typ / f_jaar** | n.v.t. | Tabel 4.8 gebouwtype, formule 4.34 leeftijdsfactor | Nieuwe velden op `Building`: building_position, construction_year |
| **Adjacent building θ_b** | Standaard 15 °C | 15 °C kantoren/winkels, 5 °C vorstvrij, θ_e stallingsruimte (§4.5) | Idem ISSO 51-tabel, andere defaults |

---

## 3. Berekenmethodiek ISSO 53 — drie sub-methodes

### 3.1 Schilmethode (voorontwerp) — hoofdstuk 3

Gebouw beschouwen als één groot vertrek. Snel orde-grootte aansluitvermogen voor haalbaarheidsstudie.

```
Φ_HL,build = Φ_T,ie + Φ_T,iae + Φ_T,iaBE + Φ_T,ig + Φ_V,build
           + Σ Φ_hu - Σ Φ_gain + Σ Φ_systeem        (3.1)
```

- Ontwerpbinnentemperatuur eenvoudig: **22 °C zorg, 20 °C overig** (§3.1)
- Ventilatieverlies: `Φ_V,build = (H_i + H_v) · (θ_i − θ_e)` voor mechanische toevoer (3.18), `max(H_i, H_v) · Δθ` voor natuurlijke toevoer (3.19)

### 3.2 Per-vertrek (definitief ontwerp) — hoofdstuk 4

Voor het dimensioneren van afgiftesystemen per ruimte:

```
Φ_HL,i = Φ_T,i + Φ_V,i + Φ_hu,i − Φ_gain,i           (4.1)
Φ_T,i = (H_T,ie + H_T,ia + H_T,iae + H_T,iaBE + H_T,ig) · (θ_i − θ_e)   (4.2)
```

### 3.3 Aansluitvermogen warmteopwekker — hoofdstuk 5

**Individueel** (formule 5.1, één gebouw één opwekker):
```
Φ_source = Σ_i (Φ_T,ie + Φ_T,iae + Φ_T,iaBE + Φ_T,ig)
         + Φ_Ven                                       // (5.2)
         + Σ Φ_hu,i
         + Σ Φ_add                                     // (5.6)
         − Σ Φ_gain
```

**Collectief** (formule 5.9): **`Φ_T,iaBE` valt weg** (warmteverlies naar buren wordt door buren zelf gedragen).

**Belangrijke aftrek (5.2):** infiltratie wordt op gebouwniveau met fractie `z` (tabel 5.1) gereduceerd omdat wind nooit op alle gevels tegelijk staat:
```
Φ_Ven = z · Σ H_i · (θ_i − θ_e) + H_v,build · (θ_i − θ_e)
z = 1.0 voor systemen met volledig gescheiden warmteopwekkers per zone
z = 0.5 in overige gevallen
```

---

## 4. Concrete formules (eerste milestone — per-vertrek-methode)

> **Conventie voor Rust doc comments:** `/// ISSO 53 formule 4.3 — H_T,ie naar buitenlucht`. Houd PDF-paginanummer in commentaar bij elke tabel.

### 4.1 Transmissie

| H-waarde | Formule | Bron |
|---|---|---|
| `H_T,ie` (buitenlucht) | `Σ (A_k · (U_k + ΔU_TB) · f_k)` | 4.3, p38 |
| `H_T,ia` (verwarmd buurvertrek) | `Σ (A_k · U_k · f_ia,k)` met f volgens 4.10–4.12 | 4.9, p40 |
| `H_T,iae` (onverwarmde buurruimte) | `Σ (A_k · U_k · f_k)` met f uit tabel 4.2 of warmtebalans (bijlage F) | 4.13, p40 |
| `H_T,iaBE` (buurpand) | `Σ (A_k · U_k · f_ia,k)` met f volgens 4.18–4.20 en θ_b uit §4.5 | 4.17, p42 |
| `H_T,ig` (bodem) | `1,45 · Σ (A_k · U_equiv,k · f_gw · f_ig,k)` | 4.21, p43 |

**ΔU_TB tabel 3.1** (p28):
| Situatie | ΔU_TB [W/(m²·K)] |
|---|---|
| Reeds verrekend in U | 0 |
| Nieuw met speciale TB-voorzieningen | 0,02 |
| Nieuw, goed vakmanschap | 0,05 |
| Binnenisolatie doorbroken door plafonds | 0,15 |
| Overige | 0,10 |

**f_k tabel 4.2** (p41) — correctiefactor onverwarmde ruimte met onbekende binnentemperatuur:
| Ruimte | f_k |
|---|---|
| Vertrek 1 externe scheid. | 0,4 |
| Vertrek 2 externe, zonder buitendeur | 0,5 |
| Vertrek 2 externe, met buitendeur | 0,6 |
| Vertrek ≥3 externe | 0,8 |
| Kelder zonder ramen/deuren | 0,5 |
| Kelder met ramen/deuren | 0,8 |
| Ruimte onder pannendak zonder folie | 1,0 |
| Niet-geïsoleerd dak | 0,9 |
| Geïsoleerd dak | 0,7 |
| Interne verkeersruimte zonder buitenwand, n_v < 0,5 | 0,0 |
| Vrij geventileerd (A/V > 0,005) | 1,0 |
| Overig verkeersruimte | 0,5 |
| Vloer boven kruipruimte zwak geventileerd | 0,6 |
| Vloer boven kruipruimte matig geventileerd | 0,8 |
| Vloer boven kruipruimte sterk geventileerd | 1,0 |

**Bodemverlies — U_equiv** (formule 4.24, tabel 4.3 p44):
```
U_equiv,k = a · (B' + b)^n · (U_k + ΔU_TB)^c · (z + d)^…    [exact: parsen uit formule 4.24]
```
Parameters tabel 4.3:
| Vlak | a | b | c1 | c2 | c3 | n1 | n2 | n3 | d |
|---|---|---|---|---|---|---|---|---|---|
| Vloer | 0,9671 | -7,455 | 10,76 | 9,773 | 0,0265 | 0,5532 | 0,6027 | -0,9296 | -0,0203 |
| Wand | 0,799 | -6,7951 | 0¹ | 26,586 | 0,1523 | 0¹ | 0,5012 | -0,1406 | -1,074 |

¹ B' heeft geen invloed bij wanden, maar mag niet 0 zijn (rekenkundige integriteit).
- `B' = 2·A_vl / O`, geclamped `2 ≤ B' ≤ 50`
- `0 ≤ z ≤ 5`
- `f_gw = 1` (grondwaterspiegel ≥1 m onder vloer) of `1,15` overig
- Min: `U_equiv,k ≥ 0,1 W/(m²·K)`

> **TODO: rust-dev controleer letterlijke formule 4.24 in PDF p44** — exacte machtsstructuur niet 100% leesbaar uit OCR. Cross-check met identiek formule 3.17 voor schilmethode.

### 4.2 Infiltratie per vertrek (§4.7.1, p44-47)

```
Φ_i' = H_i · (θ_i − θ_e)                             (4.25)
H_i = z · q_i · 1200 · f_v                            (4.27)
q_i = q_is · A_u                                      (4.28, q_v10,kar bekend)
q_i = q_is · A_g                                      (4.29, q_v10,kar onbekend)
```

**Reductiefactor z (vertrek-niveau, tabel 4.4 p45):**
| Vertrek | z |
|---|---|
| 1 buitengevel of 2 niet-tegenoverliggend | 1,0 |
| 2 tegenover elkaar liggende gevels | 0,5 |
| Overige | 0,7 |

**q_is bij bekende q_v10,kar (tabel 4.5 p45):** matrix [q_v10,kar-klasse × gebouwhoogte]. 5 klassen × 5 hoogtes = 25 waarden. Letterlijke values:

| q_v10,kar [dm³/(s·m²gebr)] | ≤3m | 3-6m | 6-20m | 20-30m | >30m |
|---|---|---|---|---|---|
| < 0,20 | 0,00026 | 0,00034 | 0,00043 | 0,00051 | 0,00062 |
| 0,20-0,40 | 0,00039 | 0,00050 | 0,00063 | 0,00077 | 0,00092 |
| 0,40-0,60 | 0,00064 | 0,00082 | 0,00103 | 0,00126 | 0,00149 |
| 0,60-0,80 | 0,00088 | 0,00111 | 0,00140 | 0,00172 | 0,00200 |
| 0,80-1,00 | 0,00109 | 0,00138 | 0,00175 | 0,00213 | 0,00251 |
| > 1,0 | 0,00118 | 0,00151 | 0,00189 | 0,00232 | 0,00273 |

Eenheid output: `q_is` in m³/(s·m² gevelopp.).

**q_is bij onbekende q_v10,kar (formule 4.31 p46):**
```
q_is = f_wind · f_type · f_inf · (0,23 · q_i,spec,reken)
f_wind = …                                           (4.32, functie van L, B, H)
f_jaar = 0,4 + 0,033 · exp(0,05 · (2060 − J))        (4.34)
```

**f_type tabel 4.6 p46:**
| Gebouwtype | f_type |
|---|---|
| Eénlaags met kap | 1,0 |
| Eénlaags met plat dak | 0,77 |
| Meerlaags standaard | 0,51 |
| Meerlaags volgevel binnengalerij | 0,48 |
| Meerlaags dubbele huidgevel onderbroken | 0,46 |
| Meerlaags dubbele huidgevel doorlopend | 0,15 |

**f_inf tabel 4.7 p46:**
| Ventilatiesysteem | f_inf |
|---|---|
| A — natuurlijke toe- en afvoer | 0,80 |
| B — mech toevoer + nat afvoer | 0,85 |
| C — nat toevoer + mech afvoer | 1,0 |
| D — gebalanceerde mech | 1,15 |
| E — zone-mix met lokale WTW + CO₂ | 1,08 |

**f_typ tabel 4.8 p47** (let op: niet hetzelfde als f_type — `f_typ` is positie binnen gebouw):
| Situatie | f_typ |
|---|---|
| Enkellaags tussengelegen | 1,0 |
| Enkellaags kop/hoek | 1,2 |
| Enkellaags vrijstaand | 1,4 |
| Meerlaags gehele gebouw | 1,2 |
| Meerlaags topetage | 1,3 |
| Meerlaags tussenetages | 1,2 |
| Meerlaags onderste etage | 1,1 |

**q_i,spec,reken tabel 4.9 p47:**
| Gebouwtype | q_i,spec,reken [m³/(s·m²)] |
|---|---|
| Eén laag met kap | 0,0010 |
| Eén laag met half plat dak | 0,00085 |
| Eén laag met plat dak | 0,0007 |
| Meerlaags | 0,0005 |

### 4.3 Ventilatie per vertrek (§4.7.2, p47-50)

```
Φ_vent = H_v · (θ_i − θ_e)                           (4.35)
H_v = q_v · 1200 · f_v                                (4.37)
f_v = (θ_t − θ_e − Δθ_v) / (θ_i − θ_e)                voor WTW/voorverwarming  (4.38)
f_v = (θ_i − θ_e − Δθ_v) / (θ_i − θ_e)                voor natuurlijke/koude toevoer (4.39)
f_v = 0                                               voor luchtverwarming (θ_t > θ_i)
```

**Tabel 4.10 ventilatie-eisen Bouwbesluit (p49-50)** — selectie per gebruiksfunctie. Letterlijke kolommen: `dm³/s per pers` (nieuwbouw) | `pers/m²` | `dm³/s per pers` (bestaand). Belangrijkste:

| Functie | Ruimte | NB dm³/s·pp | pers/m² | Best. dm³/s·pp |
|---|---|---|---|---|
| Kantoor | Kantoorruimte | 6,5 | 0,05 | 3,44 |
| Kantoor | Receptie | 6,5 | 0,05 | 3,44 |
| Onderwijs | Lesruimte | 8,5 | 0,125 | 3,44 |
| Onderwijs | Collegezaal | 8,5 | 0,125 | 3,44 |
| Onderwijs | Werkplaats | 6,5 | 0,125 | 3,44 |
| Onderwijs | Bureauruimte | 6,5 | 0,05 | 3,44 |
| Gezondheid | Patiëntenkamer | 12 | 0,125 | 3,44 |
| Gezondheid | Operatiekamer | 12 | 0,05 | 3,44 |
| Gezondheid | Onderzoekruimte | 6,5 | 0,05 | 3,44 |
| Bijeenkomst | Eetruimte/restaurant/kantine | 4 | 0,125 | 2,12 |
| Bijeenkomst | Vergaderruimte | 6,5 | 0,05 | 3,44 |
| Logies | Hotelkamer | 12 | 0,05 | 6,4 |
| Sport | Sportzaal | 6,5 | n.v.t. | 3,44 |
| Winkel | Verkoopruimte/supermarkt/warenhuis | 4 | n.v.t. | 2,12 |
| Cel | Cel dag/nacht | 12 | 0,05 | 6,4 |

**Afvoereisen** (p50): toilet ≥7 dm³/s, douche ≥14 dm³/s per stuk.

> **TODO: rust-dev** — volledige tabel 4.10 + tabel 4.11 (bezettingsdichtheid defaults) staan op p49-51 van PDF. Lees zelf en zet om naar `tables::ventilation_requirements`.

### 4.4 Toeslag voor bedrijfsbeperking — §4.8

> **TODO: rust-dev — lees PDF p51-53.** ISSO 53 gebruikt een **specifieke toeslag P [W/m²]** afhankelijk van opwarmtijd en gebouw-tijdconstante (formule [TODO]). Tabel met P-waarden moet uit PDF. Dit verschilt fundamenteel van de ISSO 51 main-room-percentage-methode.

Indicatie voor model:
```rust
pub struct HeatingUpConfig {
    pub setback_active: bool,
    pub warmup_minutes: f64,          // opwarmtijd in minuten
    pub building_mass: ThermalMass,   // licht/gemiddeld/zwaar (tabel 2.4)
}
```

### 4.5 Aansluitvermogen — hoofdstuk 5

Zie §3.3 hierboven. Voor de eerste milestone: implementeer `Φ_source` (formule 5.1) en de aftrek met fractie `z` uit tabel 5.1.

---

## 5. Ontwerpbinnentemperatuur — tabel 2.2 (p20)

Letterlijk (per-vertrek-methode):

| Gebruiksfunctie | Ruimte | θ_i [°C] |
|---|---|---|
| Kantoor / onderwijs / cel | Verblijfsruimte / verblijfsgebied | 20 |
| Kantoor / onderwijs / cel | Badruimte | 22 |
| Kantoor / onderwijs / cel | Verkeersruimte | 18 of warmtebalans |
| Kantoor / onderwijs / cel | Toiletruimte | 18 of warmtebalans |
| Kantoor / onderwijs / cel | Technische ruimte | 10 of warmtebalans |
| Kantoor / onderwijs / cel | Onbenoemde ruimte | 10 of warmtebalans |
| Kantoor / onderwijs / cel | Bergruimte | 10 of warmtebalans |
| Gezondheidszorg | Verblijfsruimte / verblijfsgebied | **22** |
| Gezondheidszorg | Badruimte | 24 |
| Gezondheidszorg | Toilet/verkeer | 18 of warmtebalans |
| Gezondheidszorg | Technisch/onbenoemd | 10 of warmtebalans |
| Gezondheidszorg | Stallings/bergruimte (vorstvrij) | 5 |
| Buiten thermische schil | Niet-verwarmde stalling / garage | θ_e |

Voor combinaties die niet in tabel 2.2 staan: bijlage G (operatieve temperatuur via PMV — TODO als out-of-scope voor milestone, fallback naar 20°C).

**Voorontwerp (schilmethode) §3.1:** simpel — 22°C zorg, 20°C overig.

---

## 6. Voorgesteld Rust crate-skelet

```
crates/isso53-core/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # publieke API: calculate_from_json(), project_schema(), result_schema()
│   ├── error.rs                # Isso53Error + Result type
│   ├── formulas.rs             # const &str voor formulenummers ("ISSO_53_2016_formule4_3", ...)
│   ├── model/
│   │   ├── mod.rs              # re-exports
│   │   ├── project.rs          # Project (info, building, climate, ventilation, rooms)
│   │   ├── building.rs         # Building, gebouwtype, positie, leeftijd, ventilatiesysteem A-E
│   │   ├── climate.rs          # DesignConditions (θ_e, θ_me)
│   │   ├── ventilation.rs      # VentilationConfig (system_type A-E, wtw, voorverwarming)
│   │   ├── room.rs             # Room + RuimteType enum + bezetting
│   │   ├── construction.rs     # ConstructionElement (kopie isso51, evt. extra velden)
│   │   └── enums.rs            # GebruiksFunctie, RuimteType, GebouwTypePositie, ThermalMass, BoundaryType, VerticalPosition, VentilationSystemType (A/B/C/D/E)
│   ├── calc/
│   │   ├── mod.rs
│   │   ├── room_load.rs        # Φ_HL,i orkestratie per vertrek (formule 4.1, 4.2)
│   │   ├── transmission.rs     # H_T,ie/ia/iae/iaBE/ig (formules 4.3-4.23)
│   │   ├── ground.rs           # U_equiv via tabel 4.3 (formule 4.24) — apart vanwege complexiteit
│   │   ├── infiltration.rs     # H_i (formules 4.25-4.34) — bekende én onbekende q_v10,kar
│   │   ├── ventilation.rs      # H_v (formules 4.35-4.39)
│   │   ├── heating_up.rs       # Toeslag bedrijfsbeperking §4.8 — TODO uit PDF
│   │   ├── shell.rs            # Schilmethode hoofdstuk 3 (voorontwerp)
│   │   └── source_capacity.rs  # Aansluitvermogen hoofdstuk 5 (individueel + collectief)
│   ├── tables/
│   │   ├── mod.rs
│   │   ├── temperature.rs      # tabel 2.2 (θ_i per functie/ruimte)
│   │   ├── thermal_bridge.rs   # tabel 3.1 ΔU_TB
│   │   ├── thermal_mass.rs     # tabel 2.4 c_eff per zwaarte
│   │   ├── adjacent_unheated.rs # tabel 4.2 f_k onverwarmde ruimten
│   │   ├── ground_params.rs    # tabel 4.3 a/b/c/n/d voor U_equiv
│   │   ├── infiltration.rs     # tabel 4.5 q_is(q_v10,kar × hoogte), tabel 4.9 q_i,spec,reken
│   │   ├── building_type.rs    # tabel 4.6 f_type, tabel 4.8 f_typ
│   │   ├── ventilation_system.rs # tabel 4.7 f_inf
│   │   ├── ventilation_requirements.rs # tabel 4.10 dm³/s·pp + pers/m²
│   │   ├── occupancy.rs        # tabel 4.11 default bezetting pers/m² per functie
│   │   └── source_fraction.rs  # tabel 5.1 z (gebouwniveau infiltratie-fractie)
│   ├── validate.rs             # input-validatie incl. height ≤ 4.0
│   ├── result.rs               # RoomResult, BuildingSummary, ProjectResult (lijkt op isso51)
│   └── bin/
│       └── isso53-cli.rs       # `isso53-cli <input.json> [output.json]`
└── tests/
    └── fixtures/
        ├── kantoor_eenlaags.json       # Voorbeeld 6.1 schilmethode (TODO uit PDF H6)
        └── kantoor_per_vertrek.json    # Voorbeeld 6.2 gedetailleerd (TODO uit PDF H6)
```

**Module-verantwoordelijkheden:**
- `lib.rs` — `calculate_from_json(input) -> Result<String>` (kopieer patroon uit `isso51-core/src/lib.rs:49`)
- `calc/room_load.rs` — orkestreert per vertrek: transmissie + infiltratie + ventilatie + heating_up − gain
- `calc/source_capacity.rs` — nieuwe orkestratie voor §5.1/§5.2 met juiste fractie z toepassing
- `tables/*` — pure data, geen logica. Conventie: `pub fn lookup(...) -> f64` + `const TABLE: &[(...)]`
- `bin/isso53-cli.rs` — minimaal: stdin/file → calculate_from_json → stdout. Exit 0 op succes, 1 op fout, schrijf error naar stderr

---

## 7. Domeinmodel diff t.o.v. `isso51-core`

### Te kopiëren (1-op-1 uit `isso51-core/src/model/`)
- `BoundaryType` (Exterior, AdjacentRoom, AdjacentBuilding, Ground, Unheated, Water) — semantiek identiek
- `VerticalPosition` (Wall, Floor, Ceiling)
- `MaterialType` (Masonry, NonMasonry) — voor ΔU_TB-keuze
- `ConstructionElement`-veld (area, u_value, boundary_type, vertical_position, adjacent_temperature, ground_params) — vrijwel identieke structuur
- `DesignConditions` struct skelet (θ_e, θ_me=9°C) — andere defaults
- `quadratic_sum` module: **NIET kopiëren** — ISSO 53 doet geen kwadratische sommatie op gebouwniveau

### Aan te passen / vervangen
- `RoomFunction` (isso51: LivingRoom, Bedroom, …) → vervangen door **(`GebruiksFunctie`, `RuimteType`)** tuple
- `BuildingType` (isso51: Detached, SemiDetached, Porch, …) → `GebouwTypePositie` (Enkellaags{tussen/kop/vrijstaand}, Meerlaags{geheel/top/tussen/onder}) — voor f_typ en f_type
- `InfiltrationMethod` (isso51: PerExteriorArea/PerFloorArea) → `InfiltrationInput` enum:
  - `KnownQv10 { value_dm3_s_m2_gebruiksopp: f64 }`
  - `UnknownQv10 { construction_year: u32, building_shape: BuildingShape }`
- `VentilationSystemType` enum: ISSO 53 gebruikt A/B/C/D/**E** (E is lokale WTW met CO₂-sturing) — isso51 heeft geen E
- `HeatingSystem` (isso51 heeft RadiatorLt, RadiatorHt, FloorHeating, …) — kopieer, maar `ventilation_rate` per vertrek wordt vervangen door bezetting-gebaseerde berekening

### Nieuw (alleen in ISSO 53)
```rust
pub enum GebruiksFunctie {
    Kantoor, Onderwijs, Gezondheidszorg, Bijeenkomst,
    Logies, Sport, Winkel, Cel, Industrie,
}

pub enum RuimteType {
    Verblijfsruimte, Verblijfsgebied,
    Badruimte, Toiletruimte, Verkeersruimte,
    TechnischeRuimte, Bergruimte, OnbenoemdeRuimte,
    Stallingsruimte, Garage,
    // domeinspecifiek
    Kantoorruimte, Patientenkamer, Lesruimte, Vergaderruimte,
    Hotelkamer, Restaurant, Verkoopruimte, // ...
}

pub struct Bezetting {
    /// Override van tabel 4.11 default; None = gebruik default per gebruiksfunctie
    pub personen: Option<f64>,
    pub personen_per_m2_default: Option<f64>,
}

pub enum BuildingShape {
    EenLaagMetKap,
    EenLaagMetHalfPlatDak,
    EenLaagMetPlatDak,
    Meerlaags,
}

pub enum ThermalMass { Licht, Gemiddeld, Zwaar } // tabel 2.4: c_eff = 15/50/75 Wh/(m³·K)

pub enum CalculationMethod {
    Shell,        // hoofdstuk 3 — voorontwerp
    PerRoom,      // hoofdstuk 4 — definitief ontwerp
    SourceIndividual, // hoofdstuk 5.1
    SourceCollective, // hoofdstuk 5.2
}
```

---

## 8. Test fixtures

> **TODO — belangrijk:** ISSO 53 hoofdstuk 6 bevat **twee uitgewerkte rekenvoorbeelden** (6.1 schilberekening, 6.2 gedetailleerd per vertrek) — dat is de gouden bron voor regressie-fixtures. Ik heb p55-58 (H5) gelezen maar nog niet H6. **Rust-dev: lees PDF p59-75 voor de exacte input-data en verwachte output-getallen per vertrek.**

### Fixture-aanpak
Per voorbeeld twee bestanden:
- `tests/fixtures/{voorbeeld}_input.json` — handmatig samengesteld uit de geometrie/U-waarden uit de PDF
- `tests/fixtures/{voorbeeld}_expected.json` — verwachte H-waarden, Φ-componenten en totaalverlies per vertrek + aansluitvermogen

### Minimaal voor MVP
1. **Voorbeeld 6.1 — schilberekening** (volledig gebouw als één vertrek). Verwacht: `connection_capacity ± 5%` van PDF-waarde
2. **Voorbeeld 6.2 — per vertrek gedetailleerd**. Per vertrek: `Φ_T, Φ_V, Φ_hu, Φ_HL_i`. Tolerantie ±2% (zelfde als ISSO 51-fixtures)
3. **Min. één synthetisch kantoor** (1 vertrek 25 m², 1 buitengevel 10 m² muur + 5 m² raam, vloer op grond, plafond verwarmd buurvertrek) — sanity check of de pipeline überhaupt eindigt. Verwachte waardes met de hand uitrekenen volgens H4

### Conventie
Volg `tests/fixtures/dr_engineering_woningbouw.json` patroon uit isso51-core: input-JSON checkt in, test in `src/lib.rs` `#[test]` doet `include_str!` + asserts per kamer per Φ-component.

---

## 9. Effort-inschatting

| Onderdeel | LOC indicatie | Tijd indicatie | Risico |
|---|---|---|---|
| Crate-skelet + Cargo.toml | 50 | 0,5u | Laag |
| `model/` (10 files) | 600 | 1,5d | Laag (kopiëren + uitbreiden) |
| `tables/` (12 files, veel data) | 800 | 2d | Middel (data-overdracht uit PDF foutgevoelig) |
| `calc/transmission.rs` + `ground.rs` | 400 | 1,5d | Middel (U_equiv-formule complex) |
| `calc/infiltration.rs` | 350 | 1,5d | **Hoog** (twee methodes + 5 factoren — fout in formule 4.31 propageert ver) |
| `calc/ventilation.rs` | 200 | 1d | Laag |
| `calc/shell.rs` | 250 | 1d | Laag |
| `calc/source_capacity.rs` | 300 | 1d | Middel (z-fractie correct toepassen) |
| `calc/heating_up.rs` | 200 | 1d | **TODO: na PDF p51-53 lezen** |
| `bin/isso53-cli.rs` | 50 | 0,5u | Laag |
| Fixtures + tests | 600 | 2d (incl. PDF H6 inlezen) | **Hoog** (alleen waarde van regressietest staat of valt met juiste PDF-extract) |
| **Totaal** | **~3800 LOC** | **~12 werkdagen** | |

**Belangrijkste risico's:**
1. **Tabel-overdracht uit PDF** — handmatige overdracht van tabel 4.5 (30 waarden) en 4.10 (>40 ventilatie-eisen) — eenmalig grondig + dubbele review verplicht
2. **Formule 4.24 U_equiv** — OCR uit PDF was niet 100% leesbaar — Rust-dev moet zelf de exacte machtsstructuur uit p44 halen
3. **Hoofdstuk 6 fixtures** — zonder uitgewerkte PDF-voorbeelden geen regressietest mogelijk
4. **Toeslag bedrijfsbeperking** — §4.8 niet in mijn extract, **rust-dev moet zelf p51-53 lezen** voor de P-tabel
5. **Bijlage G operatieve temperatuur** — buiten MVP, maar Bezetting/luchtsnelheid-gebaseerde θ_i is wel referentie voor edge cases — schrijf TODO-comment naar `bijlage_g.rs` stub

---

## 10. Out-of-scope expliciet (voor deze milestone)

- IFCX namespace `isso53::*` en mapper
- Bijlage A vraagspecificatie-template
- Bijlage B uitgebreide raam-U-waarde berekening (gebruik U-waarde als input)
- Bijlage C WTW-rendement berekening (gebruik `heat_recovery_efficiency` als input, kopie uit isso51)
- Bijlage D leiding/kanaal-verliezen in onverwarmde ruimten (`Φ_leid = 0` in MVP)
- Bijlage E afkoeling/zwaarte (alleen `ThermalMass`-enum overnemen voor heating_up)
- Bijlage F warmtebalans aangrenzende ruimten (gebruik tabel 4.2 forfaitair)
- Bijlage G operatieve temperatuur via PMV (val terug op tabel 2.2)
- Klimaatgevels/klimaatramen (formules 4.4, 4.7, 4.8 — out of scope; throw `NotSupported` als input dit aangeeft)
- Voorverwarmer-vermogen Φ_vv (formules 3.31/3.32, 5.7/5.8 — return 0 in MVP)
- Frontend / Tauri / REST API / IFC-import

---

## 11. Volgende stappen (na deze spec)

1. **PM:** voorlegt deze spec ter review aan user
2. **User:** review + akkoord
3. **`rust-developer` agent** wordt gespawned met deze spec + `crates/isso51-core` als blueprint. Output: schone PR met `crates/isso53-core` + groene `cargo test`
4. **`qc-reviewer`** op de PR voor compile / lint / formule-cross-check
5. **`git-release`** voor commit + push
6. **Apart spoor (toekomst):** UI-norm-keuze, IFCX namespace, REST API

---

## Bijlage A — Bron-pagina's PDF (voor toekomstige verfijning)

| Sectie | PDF pagina's |
|---|---|
| TOC + samenvatting | 2-4 |
| Symbolenlijst | 6-7 |
| H1 inleiding + toepassingsgebied | 12-14 |
| H2 uitgangspunten + tabel 2.2 θ_i | 15-22 |
| H3 schilmethode (formules 3.1-3.22) | 27-37 |
| H4 per vertrek (formules 4.1-4.39) | 38-54 |
| H5 aansluitvermogen | 55-58 |
| H6 rekenvoorbeelden (FIXTURE-BRON) | 59-75 |
| Bijlage A-G | 76-95 |
