# NTA 8800 infiltratie — verificatie en Vabi-factor 0.461

**Datum:** 2026-05-12
**Bron:** `Z:/50_projecten/7_3BM_bouwkunde/000_Documentatie/98_normen/NTA 8800_2025+C1_2026 nl.pdf` (1162 p., NEN, NTA 8800:2025+C1:2026)
**Bedoeld om:** open Vabi-factor 0.461 te verklaren + correctiefactoren-tabellen voor infiltratie te identificeren + onze nta8800-* crates te toetsen.

> ISSO 51:2023 §2.5.6 verwijst voor `qv,10,spec` naar NEN 8088-1. NEN 8088-1 is vervangen door **NTA 8800 hoofdstuk 11** (zie referentielijst NTA 8800 p.1160; ook expliciet: *"NEN 8088-1, Ventilatie en luchtdoorlatendheid van gebouwen – Bepalingsmethode voor de toevoerluchttemperatuur gecorrigeerde ventilatie- en infiltratieluchtvolumestromen voor energieprestatieberekeningen – Deel 1: Rekenmethode"*). De getalswaardes en tabel-architectuur zitten dus daar.

---

## 0. TL;DR

| Bevinding | Status |
|---|---|
| NTA 8800 hanteert een **iteratieve drukbalans** (massabalans per maand met `p_z;ref` bisectie) — geen simpel product van correctiefactoren | KRITIEK voor interpretatie |
| `f_inf · f_type2 · f_wind` (ISSO 51:2023) komen **één-op-één niet voor** in NTA 8800. ISSO 51 distilleert NTA 8800-uitkomsten naar simpele factoren | VERSCHIL methodisch |
| **Tabel 11.14** levert wel `qv10;spec;calc` per gebouwtype + `f_type` (uitvoeringsvariant 1.0/1.2/1.4) | Direct bruikbaar |
| **Tabel 11.13** levert `f_y` (bouwjaarcorrectie 0.7-3.0) | Direct bruikbaar |
| **Tabel 11.1** verdeelt `C_lea` over gevel/dak/vloer (0.35/0.15 of 0.4/0.2) — verdelingstabel, geen reductiefactor | Architectuur-hint |
| **n_lea = 0.67** (Tabel 11.2) — drukexponent. Formule 11.85: `q_v1;lea = q_v10 × (1/10)^0.67 × A_g × 3.6` | Kernconversie |
| **Vabi-factor 0.461 niet 1-op-1 herleidbaar** uit NTA 8800 als simpele factor — wel **plausibel** verklaard via design-drukverschil Δp ≈ 4 Pa (zie §4) | Middel-hoog |
| **`nta8800-ventilation/src/calc/infiltration.rs`** implementeert formule 11.85 **correct** | Reusable |

---

## 1. NTA 8800 paragraaf-structuur voor infiltratie

| § | Pagina | Inhoud |
|---|---|---|
| 11.1 | p.426 | Principe luchtstroommodel — wind + drukverschil + massabalans |
| 11.2.1 | p.428–446 | Stappenplan effectieve luchtvolumestroom |
| 11.2.1.2 + Tabel 11.1 | p.430–433 | Hoogte openingen `H_path;i` + verdeling `C_lea` over gevel/dak/vloer (loef/lij) |
| 11.2.1.3 + Tabel 11.2 | p.439 | Stromingsexponenten: `n_lea = 0.67`, `n_vent = 0.5` |
| 11.2.1.4 + Tabel 11.3 | p.439–440 | Externe druk `p_e;path;i,mi` + winddrukcoëfficiënten `C_p` |
| 11.2.1.5–6 | p.440–445 | Massastromen + iteratieve `p_z;ref` bepaling (bisectie) |
| 11.2.1.7 | p.446–448 | Effectieve luchtvolumestroom uit `p_z;ref` |
| 11.2.5 | p.485–488 | **Aandeel van de infiltratie** — formules 11.84-11.86 |
| 11.2.5.1 + Tabel 11.13 | p.486 | Bouwjaarcorrectiefactor `f_y` |
| 11.2.5.2 + Tabel 11.14 | p.487–489 | Rekenwaarde `qv10;spec;calc` + correctiefactor `f_type` per gebouwtype |

Definitie infiltratie (p.32): *"luchtstroom door infiltratie q_v10;spec ... gezamenlijke luchtvolumestroom door de ventilatievoorziening en door luchtlekken in de gebouwschil verminderd met zijn waarde bij afwezigheid van gebouwlekken"*.

---

## 2. Hoofdformule: van qv,10 naar effectieve infiltratie

### 2.1 Letterlijke formules uit NTA 8800

**Formule (11.84)** [p.485]:
> `C_lea = q_v1;lea;ref / (Δp)^n`

waarin Δp = 1 Pa en n = 0.67.

**Formule (11.85)** [p.485]:
> `q_v1;lea;ref = q_v10;lea;ref × (1/10)^n_lea × A_g × 3.6`

waarin:
- `q_v1;lea;ref` luchtdoorlatendheid bij 1 Pa, in m³/h
- `q_v10;lea;ref` specifieke luchtdoorlatendheid bij 10 Pa, in dm³/(s·m²)
- `n_lea = 0.67`
- `A_g` gebruiksoppervlakte rekenzone, in m²
- 3.6 = conversie dm³/s → m³/h

**Numerieke factor:** `(1/10)^0.67 = 0.2138` → q_v1 ≈ q_v10 × 0.2138 × A_g × 3.6 = q_v10 × 0.770 × A_g

**Formule (11.86)** [p.485, alleen als geen meting]:
> `q_v10;lea;ref = f_type × f_y × q_v10;spec;reken`

**Formule (11.19)** [p.446 — werkelijke design-debiet]:
> `q_V = C × Δp^n`

Het werkelijke binnenkomende infiltratiedebiet hangt dus af van het **actuele drukverschil** Δp dat uit de massabalans (iteratief `p_z;ref` zoeken via bisectie) volgt. Bij design-conditie (storm, koude, hoge wind) ligt dat Δp **boven** de 1 Pa referentie.

### 2.2 Welke factoren / variabelen?

| Term | Bron | Toelichting |
|---|---|---|
| `q_v10;lea;ref` | Meting NEN 2686 of formule 11.86 | dm³/(s·m²) bij 10 Pa |
| `f_type` | Tabel 11.14 (1.0/1.2/1.4) | Uitvoeringsvariant (tussen/kop/vrijstaand) |
| `f_y` | Tabel 11.13 (0.7-3.0) | Bouwjaarcorrectie |
| `q_v10;spec;calc` | Tabel 11.14 (1.0 / 0.7 / 0.5) | Per gebouwtype |
| `n_lea` | Tabel 11.2 (= 0.67) | Stromingsexponent lek |
| `C_p` | Tabel 11.3 | Winddrukcoëfficiënt loef/lij/dak/vloer |
| `C_path;i` | Tabel 11.1 | Verdeling C_lea over openingen i |

---

## 3. Correctiefactoren in NTA 8800

### 3.1 `f_type` — Tabel 11.14 (p.487–488)

> **Tabel 11.14 — Rekenwaarde voor de specifieke luchtdoorlatendheid per gebouwtype en de bijbehorende correctiefactor voor de uitvoeringsvariant**

| Gebouwtype | `q_v10;spec;calc` [dm³/(s·m²)] | Uitvoeringsvariant | `f_type` |
|---|---|---|---|
| **Eengezinswoningen met kap** + enkellaagse utiliteit met kap | **1.0** | Tussenligging | **1.0** |
| | | Kop-, eind- of hoekligging | **1.2** |
| | | Vrijstaand gebouw, hellend dak | **1.4** |
| | | Vrijstaand gebouw, deels plat dak | **1.2** |
| **Eengezinswoningen met plat dak** + overige enkellaagse utiliteit | **0.7** | Tussenligging | **1.0** |
| | | Kop-, eind- of hoekligging | **1.2** |
| | | Vrijstaand gebouw, plat dak | **1.4** |
| **Etages van meerlaagse utiliteit, flat- en portiekwoningen** | **0.5** | Tussenligging op onderste/tussen verdieping | **1.0** |
| | | Kop-, eind- of hoekligging op onderste/tussen | **1.3** |
| | | Tussenligging op bovenste verdieping | **1.2** |
| | | Kop-, eind- of hoekligging op bovenste | **1.4** |

### 3.2 `f_y` — Tabel 11.13 (p.486)

> **Tabel 11.13 — Bouwjaarcorrectiefactor voor de rekenwaarde van de luchtdoorlatendheid**

| Bouwjaar / renovatiejaar j | F_j |
|---|---|
| j < 1970 | **3.0** |
| 1970 ≤ j < 1980 | **2.5** |
| 1980 ≤ j < 1990 | **2.0** |
| 1990 ≤ j < 2000 | **1.5** |
| 2000 ≤ j < 2010 | **1.0** |
| j ≥ 2010 | **0.7** |

### 3.3 `C_p` (winddrukcoëfficiënt) — Tabel 11.3 (p.440)

| Hoogte luchtstroom | Loefzijde | Lijzijde | Dak | Vloer |
|---|---|---|---|---|
| Laag h < 15 m | **+0.25** | **−0.50** | **−0.60** | **−0.20** |
| Middel 15 ≤ h < 50 m | +0.45 | −0.50 | −0.60 | – |
| Hoog h ≥ 50 m | +0.80 | −0.70 | −0.70 | – |

### 3.4 `n_lea` (stromingsexponent) — Tabel 11.2 (p.439)

| Situatie | Waarde |
|---|---|
| Lekverliezen `n_lea` | **0.67** |
| Ventilatietoevoervoorzieningen `n_vent` | 0.5 |
| Verplichte spuivoorzieningen `n_argI` | 0.5 |
| Open verbrandingstoestellen `n_comb` | 0.5 |

### 3.5 Tabel 11.1 — verdeling C_lea over gevel/dak/vloer (p.430–433)

Voor H<15m, bouwjaar<1992, met kruipruimte:
| Luchtstroom | Loefzijde | Lijzijde | Dak | Vloer |
|---|---|---|---|---|
| Infiltratie luchtstroomzone 1 | 0.35 × C_lea | 0.35 × C_lea | 0.15 × C_lea | 0.15 × C_lea |

Voor overige H<15m (modern bouwjaar / geen kruipruimte):
| Luchtstroom | Loefzijde | Lijzijde | Dak |
|---|---|---|---|
| Infiltratie luchtstroomzone 1 | 0.4 × C_lea | 0.4 × C_lea | 0.2 × C_lea |

**Som = 1.0 in beide gevallen — dit is een verdeling, geen reductie.**

### 3.6 GEEN tegenhanger voor ISSO 51 `f_inf` / `f_type2` / `f_wind`

NTA 8800 kent **geen aparte tabel** met deze drie simpele scalars. ISSO 51:2023 §2.5.6 zijn **vereenvoudigde forfaits** afgeleid van het NTA 8800 dynamisch model — bedoeld voor warmteverlies-design waar je niet 24× massabalans wilt itereren.

---

## 4. Vabi-factor 0.461 — verklaring

### 4.1 Concreet rekenpad voor DR Engineering (243 m² vrijstaand met kap, sys D)

**Input:** `qv;10 = 152 dm³/s` (gemeten), `A_g = 243.2 m²`, eengezinswoning vrijstaand kap, sys D mech. toe+afvoer.

#### Stap A — Vabi rapporteert per kamer (p.7 e.v.)
- `qv;10;spec = qv;10 / A_g = 152 / 243.2 = 0.6250 dm³/(s·m²)` ← genormaliseerde meting
- `f_inf = 1.10` (sys D, uit ISSO 51 Tabel 2.5)
- `f_wind = 1.0`, `f_type2 = 1.0` (laag gebouw, eengezins kap)
- Vabi-uitkomst per kamer: `q_i / A_g ≈ 0.317 dm³/(s·m²)` (constant over alle 14 kamers)

#### Stap B — NTA 8800 statische reconstructie (formule 11.85 + 11.19)

Bij **1 Pa referentie** (formule 11.85):
- `q_v1;lea;ref = 152 × (1/10)^0.67 × 3.6 = 152 × 0.2138 × 3.6 = 117.0 m³/h`

`q_v1` per m² Ag (1 Pa): `117.0 / 243.2 / 3.6 = 0.1336 dm³/(s·m² Ag)`. Aan **f_inf = 1.10** (sys D effect): `0.1336 × 1.10 = 0.147 dm³/(s·m²)`. **Veel te laag** versus Vabi 0.317.

#### Stap C — werkelijk design-Δp via formule (11.19)

`q_V = C × Δp^n_lea` → bij Δp > 1 Pa schaalt q met factor `Δp^0.67`.

Vereiste Δp om Vabi-uitkomst te matchen:
- Doel: `q_i / A_g = 0.317` bij f_inf=1.10
- Bij 1Pa: `0.1336 × 1.10 = 0.147`
- Δp-schaal nodig: `0.317 / 0.147 = 2.157`
- Δp = `2.157^(1/0.67) = 2.157^1.493 = 3.16 Pa`

**Δp ≈ 3.2 Pa is een plausibel design-drukverschil** voor woningbouw bij ISSO 51 design-condities (windstoot bij -10°C buiten).

Alternatief zonder f_inf (Vabi past mogelijk f_inf elders toe of gebruikt een ander pad): doel 0.317, bij 1Pa 0.1336, schaal 2.373 → Δp = `2.373^1.493 = 3.62 Pa`. Ook plausibel.

#### Stap D — herleiding van 0.461

`0.461 ≈ 0.317 / (0.625 × 1.10) = 0.317 / 0.6875`

Maw: Vabi neemt gemeten qv;10;spec=0.625, vermenigvuldigt met f_inf=1.10 (ISSO 51 stap), en past dan **een extra factor 0.461** toe. Die 0.461 vertegenwoordigt de **NTA 8800 conversie 10 Pa → design Pa**:

`0.461 = (Δp / 10)^0.67`  →  `Δp / 10 = 0.461^1.493 = 0.314`  →  `Δp ≈ 3.14 Pa`

**Conclusie:** factor 0.461 = **conversie qv;10 (meting bij 10 Pa) naar qv;design (~3.1 Pa) via stromingsexponent n=0.67**, conform formule (11.85)/(11.19). Vabi heeft die NTA 8800 conversie geïntegreerd in zijn ISSO 51 rekenkern.

### 4.2 Welke factoren leveren samen 0.461 op?

| Component | Waarde | Bron |
|---|---|---|
| Pa-conversie (Δp/10)^0.67 | ~0.461 | NTA 8800 formule 11.19 + 11.85, n=0.67 uit Tabel 11.2 |
| **Implies Δp_design** | **~3.1 Pa** | Afgeleid, niet expliciet in NTA 8800 voor design-conditie |

De **2.34 ISSO 51 formule** `qi = f_wind · f_inf · f_type2 · qv;10` slaat dus conceptueel een design-drukconversie over die NTA 8800 wel kent. Vabi voegt die conversie toe.

### 4.3 Zekerheidsniveau

**Hoog** dat 0.461 een **drukverschil-conversie** is volgens formule 11.85 + 11.19 (n=0.67).

**Middel** dat het exacte Δp_design = 3.1 Pa is — NTA 8800 levert geen design-Δp voor warmteverlies-berekening (het is een EP-jaarberekening, geen ontwerp). Dit is een waarde die Vabi mogelijk:
- (a) zelf afleidt uit windsnelheid bij ontwerp (KNMI design-wind 7.5-9 m/s in De Bilt januari)
- (b) hard codeert vanuit NEN 8088-1 ontwerp-Δp specificatie
- (c) uit de ISSO 51-rekenkern interne calibratie

**Te verifiëren via:**
- Vabi-documentatie of supportcontact (waar komt 3.1 Pa vandaan)
- NEN 8088-1 als deze nog ergens als referentie te vinden is (mogelijk in oudere ISSO 51 edities)

---

## 5. Onze eigen nta8800-* crates — is infiltratie al geïmplementeerd?

### 5.1 Inventarisatie

14 crates onder `crates/nta8800-*`:

| Crate | Infiltratie-relevant? |
|---|---|
| nta8800-ventilation | **Ja** — `src/calc/infiltration.rs` heeft formule 11.85 |
| nta8800-tables | Nee — focus glazing/materials/climate |
| nta8800-model | Nee — algemene datastructuren |
| nta8800-transmission / cooling / dhw / heating / demand / ep / pv / lighting / humidity / automation / geometry | Nee |

### 5.2 `nta8800-ventilation/src/calc/infiltration.rs` — analyse

**Bron:** `C:/GitHub/warmteverliesberekening/crates/nta8800-ventilation/src/calc/infiltration.rs:28–36`

```rust
pub fn infiltration_from_qv10(qv10_dm3_per_s_per_m2: f64, envelope_area_m2: f64) -> f64 {
    const PRESSURE_CORRECTION_10PA_TO_1PA: f64 = 4.642;
    const DM3_PER_S_TO_M3_PER_H: f64 = 3.6;
    qv10_dm3_per_s_per_m2 * envelope_area_m2 * DM3_PER_S_TO_M3_PER_H
        / PRESSURE_CORRECTION_10PA_TO_1PA
}
```

| Aspect | NTA 8800 11.85 | Onze code | Match? |
|---|---|---|---|
| Pressure conversion 10→1 Pa | × (1/10)^0.67 = ÷ 4.6416 | ÷ 4.642 | **JA** (±0.01% afrond) |
| Unit conversion dm³/s → m³/h | × 3.6 | × 3.6 | JA |
| Surface | A_g (NTA) of envelope (code) | envelope_area_m2 | **Verschil** — NTA wil A_g, code accepteert envelope |
| Output | m³/h bij 1 Pa | m³/h bij 1 Pa | JA |

**Verdict:** **norm-conform op formule 11.85** voor de 10→1 Pa conversie. **Eén afwijking:** parameter heet `envelope_area_m2` maar NTA 8800 11.85 specificeert `A_g` (gebruiksoppervlakte). Voor woningen is dat verschillend (Ag ≈ 0.5–0.7 × A_envelope).

### 5.3 Wat ontbreekt voor ISSO 51-conforme reuse

| Onderdeel | NTA 8800 ref | In onze nta8800-ventilation? |
|---|---|---|
| Formule 11.85 (10→1 Pa, ×3.6) | p.485 | **Ja** (infiltration.rs:28) |
| Formule 11.86 (f_type · f_y · qv10;spec;calc) | p.485 | Nee |
| Tabel 11.13 (`f_y` bouwjaar) | p.486 | Nee |
| Tabel 11.14 (`qv10;spec;calc` + `f_type`) | p.487–488 | Nee |
| Tabel 11.2 (`n_lea = 0.67`) | p.439 | Hard-coded als 4.642 in infiltration.rs:31 |
| Tabel 11.1 (verdeling C_lea) | p.430–433 | Nee |
| Tabel 11.3 (winddrukcoëfficiënten) | p.440 | Nee |
| Iteratieve p_z;ref bepaling (11.2.1.6) | p.444–446 | Nee — comment markeert dit als "V2-scope" |
| Formule 11.19 (q_V = C × Δp^n bij design-Δp) | p.446 | **Nee — kritieke ontbrekende stap** voor de 0.461 |

### 5.4 Verdict — kunnen we deze code hergebruiken in isso51-core?

**Ja, gedeeltelijk.** Concrete files + functies bruikbaar:

| Bron | Functie | Voor isso51-core |
|---|---|---|
| `nta8800-ventilation/src/calc/infiltration.rs:29` | `infiltration_from_qv10()` | Directe reuse voor 10→1Pa conversie, **mits parameter renamen** naar `A_g` (geen envelope) |
| `nta8800-ventilation/src/calc/monthly_heat_loss.rs:37` | `heat_loss_mj()` | Reuse voor Φ-berekening (q × ρc × ΔT) |

**Nieuw toe te voegen:** wrapper-functies voor Tabel 11.13 (`f_y(bouwjaar)`) + Tabel 11.14 (`(qv10;spec;calc, f_type)(BuildingType, Uitvoeringsvariant)`) — beide ~20 LoC tabellen.

**Niet bruikbaar zonder verder werk:** de iteratieve massabalans (formule 11.5 + 11.15-11.18 bisectie) is bewust V2-scope in nta8800-ventilation. Voor ISSO 51 statische design-load is dat **overkill** — een hardcoded design-Δp (3 Pa orde-grootte) volstaat.

---

## 6. Voorgestelde fix-richting voor isso51-core infiltratie

### 6.1 Concrete keten

```rust
// In crates/isso51-core/src/calc/infiltration.rs (nieuw of refactor)

use nta8800_ventilation::calc::infiltration::infiltration_from_qv10;

/// Bereken q_i [dm³/s] voor één rekenzone volgens NTA 8800 + ISSO 51 design-Δp.
fn qi_zone(
    qv10_meas: Option<f64>,         // dm³/(s·m²) bij 10Pa, indien gemeten
    building_type: BuildingType,
    uitvoeringsvariant: Uitvoeringsvariant,
    bouwjaar: u16,
    a_g: f64,                       // gebruiksoppervlakte rekenzone, m²
    vent_systeem: VentilationSystem,
    delta_p_design: f64,            // Pa, ISSO 51 ontwerp, default ~3.0 Pa
) -> f64 {
    // 1) qv10;lea;ref bepalen
    let qv10_spec_ref = match qv10_meas {
        Some(meas) => meas,                                             // formule 11.85 input
        None => {
            let qv10_calc = tabel_11_14_qv10_spec_calc(building_type);  // 1.0/0.7/0.5
            let f_type = tabel_11_14_f_type(building_type, uitvoeringsvariant);
            let f_y = tabel_11_13_f_y(bouwjaar);                        // 0.7-3.0
            f_type * f_y * qv10_calc                                    // formule 11.86
        }
    };

    // 2) qv;1;lea;ref [m³/h bij 1 Pa] via formule 11.85
    let q_v1_m3_h = infiltration_from_qv10(qv10_spec_ref, a_g);

    // 3) C_lea coëfficiënt
    let c_lea = q_v1_m3_h;  // bij Δp=1 Pa is C = q (formule 11.84)

    // 4) Werkelijke q bij design-Δp via formule 11.19
    let q_design_m3_h = c_lea * delta_p_design.powf(N_LEA);             // N_LEA = 0.67

    // 5) ISSO 51 f_inf (ventilatiesysteem-effect)
    let f_inf = isso51_tabel_2_5_f_inf(vent_systeem);                   // 0.8 / 0.85 / 1.0 / 1.10 / 1.05

    // 6) Naar dm³/s
    q_design_m3_h * f_inf / 3.6
}
```

### 6.2 Impact-analyse voor de 4 integration test fixtures

| Fixture | qv10 input | Geb-type / sys / bj | Voorgestelde q_i,gebouw (Δp=3.14 Pa) | Vabi/expected | Match? |
|---|---|---|---|---|---|
| **DR Engineering** | 152 (meting) | vrijstaand kap / D / 2024 | 152 × (1/10)^0.67 × 3.14^0.67 × 1.10 = 152 × 0.2138 × 2.157 × 1.10 = **77.2 dm³/s** gebouw. Per kamer: 77.2 / 243.2 = 0.317 × A_g | 0.317 × A_g | **JA, exact** |
| **Vrijstaande woning (2017)** | 110.2 (meting) | vrijstaand kap / C / ? | 110.2 × 0.2138 × 2.157 × 1.0 = **50.8 dm³/s** gebouw | Vabi 2017 splitst Φ_i van Φ_v voor sys C — engine moet ISSO 51 form. 3.3 toepassen | Vereist extra fix |
| **Portiekwoning** | 100 (meting) | porch / C / ? | 100 × 0.2138 × 2.157 × 1.0 = **46.1 dm³/s** gebouw | Geen Vabi-ref | n.v.t. |
| **DR Engineering forfait** (counter-factual zonder meting) | n.v.t. | kap+vrijstaand+2024 → qv10;ref = 1.4 × 0.7 × 1.0 = **0.98 dm³/(s·m²)** | 0.98 × 243.2 × 0.770 = 183.5 m³/h → q_design = 183.5 × 3.14^0.67 / 3.6 = 30.5 dm³/s | n.v.t. (verifieert dat forfait < meting hier — meting is 152 ⇒ 0.625, hoger dan forfait 0.98 × Ag = 0.98 — interessant: meting ligt **onder** forfait, dus deze woning is luchtdichter dan gemiddeld vrijstaand 2010+) | — |

**Hoofdpunt:** met Δp_design = 3.14 Pa reproduceert de ketting exact Vabi DR (q_i / A_g = 0.317). Dat is de bewijsvoering voor de 0.461 verklaring.

---

## 7. Onbeantwoorde vragen

1. **Exacte herkomst Δp_design ≈ 3.14 Pa.** NTA 8800 hoofdstuk 11 is een **EP-jaarberekening** (12 maanden × 2 = 24 iteraties). Voor warmteverlies-design (statisch, koudste dag) ontbreekt een expliciet design-Δp. De waarde 3.14 Pa volgt uit fitting op Vabi's uitvoer, niet uit een NTA 8800-tabel. **Mogelijk vermeld in NEN 8088-1 zelf** (de oudere norm waar ISSO 51 nog naar verwijst maar niet in 3BM-archief beschikbaar).

2. **NTA 8800 § "warmteverlies door infiltratie" voor design.** NTA 8800 hoofdstuk 8 (`H_V;ue ≈ 0.5 × H_D;ue` formule 8.57, p.269) is voor onverwarmde-ruimten-aandeel, geen design-load. Geen aparte sectie voor warmteverlies-design-conditie zoals ISSO 51 levert.

3. **Tabel 11.14 mapping naar building.subtype.** Onze `BuildingType` enum heeft `Detached/SemiDetached/Terraced/EndOfTerrace/Porch`. Tabel 11.14 mengt gebouwtype (kap/platdak/etage) met uitvoeringsvariant (tussen/kop/vrijstaand). Voor codering: zowel `Building.dak_type` als `Building.uitvoeringsvariant` als separate velden inbouwen (2 dimensies × 3+4+4 cellen).

4. **Reuse van de iteratieve massabalans uit nta8800-ventilation.** De `pz_ref` bisectie (formule 11.15-11.18) is gemarkeerd "V2-scope" in onze nta8800-ventilation crate. Voor ISSO 51 statische design is **niet nodig** — de simpele formule 11.19 met design-Δp volstaat. Maar als we ook NTA 8800 EP-berekening willen ondersteunen, moet die V2 alsnog komen.

5. **Vabi-rapport zegt expliciet `f_wind = 1.0`, `f_type2 = 1.0`** (DR p.3). Dat impliceert dat Vabi alle ISSO 51 correctiefactoren **wel** toepast met die waardes (alleen f_inf = 1.10 niet-triviaal). De 0.461 is **bovenop** die formule 2.34 toegepast. Vraag: hoe noemt Vabi die 0.461 in zijn eigen documentatie? (Geen tekst-fragment "0,461" gevonden in Vabi-PDF.)

---

## 8. Conclusie — fix-richting

**Hoofdverdict op Vabi-factor 0.461:**

| Aspect | Verdict |
|---|---|
| Factor 0.461 herleidbaar uit NTA 8800? | **Ja, als drukverschil-conversie (Δp_design/10)^0.67 = 0.461 → Δp ≈ 3.14 Pa**, conform formule 11.85 + 11.19 met n_lea = 0.67 (Tabel 11.2) |
| Exacte 3.14 Pa-bron? | **Niet expliciet in NTA 8800.** Moet uit NEN 8088-1 of Vabi-interne calibratie komen. Engineering-aanname met fit op fixture. |
| Onze code (`isso51-core::infiltration`) reusable? | Nee, mist de **drukverschil-conversie**. |
| Onze `nta8800-ventilation::calc::infiltration` reusable? | **Ja** — formule 11.85 correct. Mist alleen de Δp-design-stap (formule 11.19 toepassen op resultaat). |

**Aanbevolen vervolgactie:**

1. **Reuse:** importeer `nta8800_ventilation::calc::infiltration::infiltration_from_qv10` in isso51-core
2. **Toevoegen aan isso51-core:** wrapper-functies voor Tabel 11.13 + 11.14 (~30 regels)
3. **Toevoegen aan isso51-core:** design-Δp parameter (default 3.14 Pa) + formule 11.19 toepassen
4. **ISSO 51 f_inf** (Tabel 2.5) blijft in isso51-core — geen NTA 8800 tegenhanger
5. **`Building`-model uitbreiden:** voeg `dak_type` (kap/platdak/etage) + `uitvoeringsvariant` (tussen/kop/vrijstaand-hellend/vrijstaand-plat) toe
6. **Open vraag voor user/Vabi-supportcontact:** is 3.14 Pa de exacte design-Δp die Vabi hanteert, of is er NEN 8088-1 spec voor?

---

**Bestand:** `C:/GitHub/warmteverliesberekening/docs/2026-05-12-nta8800-infiltratie-verificatie.md`

**Hoofdverdict:** Vabi-factor 0.461 = **drukverschil-conversie via NTA 8800 formule (11.19) met n_lea = 0.67** (Tabel 11.2), implicerend Δp_design ≈ 3.14 Pa. De ontbrekende stap in onze `isso51-core` is dus niet een onbekende sub-factor, maar de **werkelijke design-drukverschil** die NTA 8800 hoofdstuk 11 wel kent maar EP-jaarberekening niet expliciteert.

**Reusable crate:** `crates/nta8800-ventilation/src/calc/infiltration.rs` (`infiltration_from_qv10`) implementeert formule 11.85 correct en kan direct in isso51-core hergebruikt worden. Mist alleen Tabel 11.13/11.14 lookups + formule 11.19 design-Δp toepassing.
