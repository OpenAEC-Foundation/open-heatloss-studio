# Vabi infiltratie-keten — reproductie per fixture

**Datum:** 2026-05-12
**Modus:** READ-ONLY — geen code/test/tabel-wijzigingen.
**Bedoeld om:** Issue C definitief afsluiten + fix-richting voor Tabel 2.8 + 4.3 vaststellen.
**Hoofdbronnen:**
- ISSO 51:2023 PDF p.37–41 (`Z:/.../ISSO-51 ... 01-05-2023.pdf`).
- Vabi DR-Engineering rapport (`tests/references/dr-engineering-woningbouw-isso51-2024.pdf`, Vabi 3.12.0.127).
- Vabi Vrijstaande woning rapport 2017 (`tests/references/vrijstaande-woning-isso51-2017.pdf`, Vabi 3.8.1.14).

_Redactie 2026-07-02: letterlijke ISSO 51-tabeltranscripties (Tabel 2.5/2.6/2.7/2.8) vervangen door verwijzingen (auteursrecht ISSO); volledige transcripties lokaal bij 3BM. Vabi-rapportcijfers (eigen/derden-data) en eigen ketenafleidingen blijven staan._

---

## 1. ISSO 51:2023 formule 2.35 — volledig (letterlijk)

### 1.1 Twee bepalings-routes (p.37–38)

ISSO 51:2023 §2.5.6 onderscheidt twee paden om `q_i` (luchtvolumestroom infiltratie [dm³/s]) te bepalen:

| Route | Wanneer | Formule | Bron |
|---|---|---|---|
| **Meting** | `qv;10` gemeten (blower-door) | **(2.34)** `q_i = f_wind · f_inf · f_type2 · qv;10` | p.37 |
| **Forfaitair** | geen meting beschikbaar | **(2.35)** `q_i = f_wind · f_inf · f_type2 · qv,10,spec · A_g` | p.38 |

Definities (p.38) van de drie correctiefactoren:
- `f_wind` = correctie voor winddruk door **gebouwafmetingen** (zie formule 2.36). Voor lage gebouwen (H < 13 m) **veelal 1,0**.
- `f_inf` = correctie voor de **invloed van het ventilatiesysteem** op de infiltratie (tabel 2.5).
- `f_type2` = correctie voor **gebouwafhankelijke winddrukverdeling** en thermiek (tabel 2.6).

### 1.2 Tabel 2.5 — `f_inf` (p.39)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.5, p.39 (bron lokaal: `Z:\...\98_normen`)._ f_inf per ventilatiesysteem A t/m E: van 0,80 (A) via 1,0 (C) tot **1,10 (D)**, met 1,05 voor E. Voor de DR-fixture (systeem D) is f_inf = **1,10** — die waarde is nodig voor de keten-afleiding hieronder.

### 1.3 Tabel 2.6 — `f_type2` (p.39)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.6, p.39._ f_type2 (winddrukverdeling): grondgebonden en meerlaags-standaard = 1,0; meerlaagse varianten met (dubbele) huidgevel lopen af tot 0,30. Voor alle drie fixtures geldt f_type2 = 1,0.

### 1.4 Tabel 2.7 — `f_tp` (p.40, voor `qv,10,spec` via formule 2.37)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.7, p.40._ Liggings-/dakcorrectie f_tp per situatie, van 1,0 (tussenligging) tot **1,4** (vrijstaand puntdak / kop-hoek bovenste verdieping); schilberekening 1,2. Voor de DR-fixture (vrijstaand met kap) is f_tp = 1,4.

### 1.5 Tabel 2.8 — `q_i,spec` (p.41, bij NEN 8088-1)

_Geverifieerd tegen ISSO 51:2023 Tabel 2.8, p.41 (bron lokaal: `Z:\...\98_normen`; volledige tabel niet gereproduceerd)._ 3-rijige gebouwtype-lookup, per m² gebruiksoppervlak: eengezins kap/half platdak **1,0**, eengezins platdak **0,7**, etages flat/portiek **0,5** dm³/(s·m²). Deze drie waarden zijn nodig om de forfaitaire keten en de code-discrepantie te tonen.

### 1.6 Formule 2.37 — `qv,10,spec` (p.39–40)

_Geverifieerd tegen ISSO 51:2023 §2.5.6, formule 2.37/2.38, p.39-40._ Zonder opgave geldt `qv,10,spec = f_tp · f_jaar · q_i,spec`, met f_jaar via formule 2.38 (grenzen 0,7 ≤ f_jaar ≤ 4,3).

Daarmee is de **volledige forfaitaire keten** (geen meting):

```
q_i = f_wind · f_inf · f_type2 · (f_tp · f_jaar · q_i,spec_Tab2.8) · A_g
```

Bij **gemeten** `qv;10` vervalt de bracket → **`q_i = f_wind · f_inf · f_type2 · qv;10`** (formule 2.34).

---

## 2. Onze huidige code-keten

`crates/isso51-core/src/calc/room_load.rs:103-123`:

```rust
let q_i = match building.infiltration_method {
    InfiltrationMethod::PerExteriorArea => {
        let qi_spec = tables::infiltration::qi_spec_per_exterior_area(building.qv10);
        // som ΣA_exterior van room
        infiltration::infiltration_flow_rate(qi_spec, total_exterior_area)
    }
    InfiltrationMethod::PerFloorArea => {
        let qi_spec = tables::infiltration::qi_spec_per_floor_area(building.qv10);
        qi_spec * room.floor_area
    }
};
let h_i = 1.2 * q_i;
let phi_i = 1.0 * h_i * (theta_i - theta_e);
```

`tables::infiltration` (volledig):

| Functie | Sleutel | Reeks |
|---|---|---|
| `qi_spec_per_exterior_area(qv10)` | qv10 ≤ 50 / ≤100 / ≤150 / >150 | 0,08 / 0,16 / 0,24 / 0,32 |
| `qi_spec_per_floor_area(qv10)` | idem | 0,04 / 0,08 / 0,12 / 0,16 |

**Vergelijking met norm:**

| Aspect | Onze code | ISSO 51:2023 forfaitair (2.35) | ISSO 51:2023 meting (2.34) |
|---|---|---|---|
| Sleutel `q_i,spec` | numerieke qv10-klassen | gebouwtype (3 rijen) | n.v.t. |
| `f_wind` | **niet toegepast** | × | × |
| `f_inf` | **niet toegepast** | × | × |
| `f_type2` | **niet toegepast** | × | × |
| `f_tp` | **niet toegepast** | × (via 2.37) | n.v.t. |
| `f_jaar` | **niet toegepast** | × (via 2.37) | n.v.t. |
| Basis-input | qv10-getal + Ag of A_ext | A_g | qv;10 |

**Verdict:** onze keten slaat **alle vijf correctiefactoren** over. We hebben effectief alleen het skelet `q_i = qi_spec_tabel · A` (met onbekende provenance van de tabel-getallen). Het pad is dus geen route uit ISSO 51:2023, het is een eigen vereenvoudiging.

---

## 3. DR Engineering — reconstructie per kamer (3 steekproeven)

### 3.1 Bouw-context (uit fixture + Vabi rapport p.3)

| Parameter | Waarde | Bron |
|---|---|---|
| Gebouwtype | detached / eengezins met kap | fixture `building_type` |
| `qv;10` (gemeten) | 152,0 dm³/s | fixture + p.3 |
| `A_g,totaal` | 243,2 m² | fixture + p.3 |
| `qv,10,spec` (= `qv;10 / A_g`) | **0,6250 dm³/(s·m² Ag)** | **p.3 letterlijk** |
| `f_wind` | 1,0 | **p.3 letterlijk** |
| `f_type2` | 1,0 | **p.3 letterlijk** |
| Vent. systeem D → `f_inf` | **1,10** | **per ruimte letterlijk** (p.7 etc: *"Correctiefactor invloed ventilatievoorziening [-] 1,10"*) |
| `infiltration_method` (fixture) | `per_floor_area` | fixture regel 18 |

**Belangrijk:** Vabi gebruikt de **meting-route** (formule 2.34) — niet de forfaitaire. p.3 zegt letterlijk *"Methode qv;10 Specifiek"*, en `0,6250` = `152 / 243,2` precies (gemeten qv;10 genormaliseerd per Ag, geen Tabel 2.8 waarde).

### 3.2 Per-kamer cijfers uit Vabi rapport

| Kamer | A_g [m²] | θ_i | Δθ | Vabi `q_i` [dm³/s] | Vabi `Φ_i` [W] | `q_i / A_g` | Vabi `f_inf` |
|---|---|---|---|---|---|---|---|
| 0.01 Entree | 11,91 | 20 | 28 | **3,8** | 127 | 0,3191 | 1,10 |
| 0.03 Woonkamer | 45,70 | 22 | 30 | **14,5** | 520 | 0,3173 | 1,10 |
| 1.04 Slaapkamer 1 | 24,56 | 22 | 30 | **7,8** | 280 | 0,3176 | 1,10 |

**Constante:** `q_i / A_g ≈ 0,317 dm³/(s·m² Ag)` voor elke verwarmde kamer.

### 3.3 Berekende keten — drie vergelijkingen

**Onze code-output (`qi_spec_per_floor_area(152)=0,16`):**

| Kamer | A_g | engine q_i = 0,16·A_g | engine Φ_i = 1,2·q_i·Δθ | Vabi Φ_i | ratio |
|---|---|---|---|---|---|
| 0.01 Entree | 11,91 | 1,91 | 64 | 127 | 0,50 |
| 0.03 Woonkamer | 45,70 | 7,31 | 263 | 520 | 0,51 |
| 1.04 Slaapkamer 1 | 24,56 | 3,93 | 141 | 280 | 0,50 |

Onze engine produceert exact **50%** van Vabi.

**Norm-route (formule 2.34 met directe pro-rata Ag-verdeling):**

```
q_i,gebouw = f_wind · f_inf · f_type2 · qv;10
           = 1,0 · 1,10 · 1,0 · 152
           = 167,2 dm³/s
q_i,kamer  = (q_i,gebouw / A_g,totaal) · A_g,kamer
           = (167,2 / 243,2) · A_g,kamer
           = 0,6875 · A_g,kamer
```

| Kamer | A_g | norm-route q_i | norm-route Φ_i | Vabi Φ_i | match? |
|---|---|---|---|---|---|
| 0.01 Entree | 11,91 | 8,19 | 275 | 127 | 2,17× te hoog |
| 0.03 Woonkamer | 45,70 | 31,42 | 1131 | 520 | 2,17× te hoog |
| 1.04 Slaapkamer 1 | 24,56 | 16,89 | 608 | 280 | 2,17× te hoog |

**Verschil:** norm-route geeft `q_i/A_g = 0,6875`, Vabi geeft `0,317`. Ratio Vabi/norm = **0,461**.

### 3.4 Verdict DR

| Hypothese | Bewijs voor | Bewijs tegen |
|---|---|---|
| H1 (Vabi = Tabel 2.8 puur, geen correctie) | — | Vabi gebruikt expliciet `qv;10` gemeten (p.3), niet Tabel 2.8. `q_i/A_g=0,317` zit niet in Tabel 2.8 (1,0/0,7/0,5). |
| H2 (Vabi = Tabel 2.8 × `f_type2`) | — | Idem H1. `f_type2 = 1,0` hier. |
| H3 (Vabi = NEN 8088-1 gevel-route) | — | Vabi rapporteert eenheid `Ag`, niet geveloppervlak. |
| **H4 (NIEUW)** Vabi past **een onbekende reductiefactor 0,461** toe bovenop formule 2.34 | Constant `q_i/A_g = 0,317` over alle 14 kamers. `0,317 / (1,0·1,10·1,0·0,625) = 0,461`. | Geen pagina in ISSO 51:2023 die deze factor noemt. Mogelijk Vabi-eigen "kierdichtheid" correctie of NEN 8088-1 sub-factor (`f_z` voor verdeling, of ventilatie-aftrek). |

**Hoofdverdict DR:** Vabi gebruikt **formule 2.34** (meting) met een **niet-genormeerde extra factor ~0,461**. De code is fundamenteel onderspecificeerd op twee niveaus: (a) `f_inf` mist (× 1,10), (b) onze tabel-keuze 0,16 ≠ qv;10/A_g·0,461 ≈ 0,288. **Zekerheid hoog op richting (formule 2.34 met `f_inf` is correct), middel op de exacte oorsprong van 0,461.**

---

## 4. Vabi vrijstaande woning — reconstructie (ISSO 51:2017)

### 4.1 Bouw-context

| Parameter | Waarde | Bron |
|---|---|---|
| Gebouwtype | detached | fixture |
| `qv;10` (gemeten) | 110,2 dm³/s | fixture |
| `A_g,totaal` | 90,7 m² | fixture |
| Vent. systeem C → `f_inf` | 1,0 | tabel 2.5 |
| Vabi-versie | 3.8.1.14 (ISSO 51:**2017**) | rapport hdr |
| Fixture `infiltration_method` | `per_floor_area` | fixture regel 17 |
| Vabi-keying in rapport | **per m² geveloppervlak** | rapport p.10–34 |

### 4.2 Vabi-cijfers (3 steekproef-kamers uit rapport)

Het rapport noteert per ruimte: *"Infiltratie 0,000209 m³/s × X m² buitenopp"* of *"0,000000 m³/s × X m²"*.

| Kamer | A_g | Σ A_ext (rapport) | Vabi `qi_spec_ext` [m³/(s·m² A_ext)] | Vabi `q_i` [m³/s] | Vabi Φ_i [W] |
|---|---|---|---|---|---|
| 1.1 Hal | 8,40 | 8,40 | 0,000209 | 0,001756 | 57 (Φ_v totaal, infiltratie-deel) |
| 0.4 Keuken | 7,20 | 2,64 | **0,000000** | 0 | 0 |
| 1.3 Badkamer | 5,60 | 8,41 | 0,000209 | 0,001758 | 65 |
| 1.6 Slaapkamer 3 | 16,29 | 35,01 | 0,000000 | 0 | 0 |

**Twee waarden voor `qi_spec_ext`:** `0,000209` of `0,000000`. Selectie lijkt op basis van *type ruimte* (woonruimte=0; entree/badkamer/halsen=0,000209). Niet-genormeerd, Vabi-eigen logica.

`0,000209 m³/(s·m² A_ext) = 0,209 dm³/(s·m² A_ext)`. Ligt **tussen** onze `qi_spec_per_exterior_area` klassen 0,16 (≤100) en 0,24 (≤150). qv;10=110,2 valt in klasse `>100, ≤150` → onze code rekent **0,24**, Vabi rekent **0,209** (waar van toepassing) of **0**.

### 4.3 Drie-paden vergelijking

| Kamer | A_g | A_ext | Onze code (0,24·A_ext) | Onze code Φ_i [W] | Vabi q_i | Vabi Φ_i [W] |
|---|---|---|---|---|---|---|
| Hal | 8,40 | 8,40 | 2,02 dm³/s | 65 | 1,76 dm³/s | 57 |
| Keuken | 7,20 | 2,64 | 0,63 dm³/s | 22 | 0 | 0 |
| Slaapkamer 3 | 16,29 | 35,01 | 8,40 dm³/s | 292 | 0 | 0 |

Onze code rekent **alle** rooms infiltratie ≠ 0. Vabi rekent voor woonruimten (alle in expected.json met `phi_v`) **nul infiltratie** en stopt het in `phi_v` (ventilatie). Hal/badkamer/onverwarmde-ruimten krijgen wél een infiltratie-deel.

### 4.4 Verdict Vrijstaande woning

| Hypothese | Bewijs |
|---|---|
| H1 (Tabel 2.8 puur) | Niet — Vabi 2017 gebruikt `0,209 dm³/(s·m² A_ext)` waar wel infiltratie is, geen 1,0/0,7/0,5. |
| H2 (Tabel 2.8 × f_type2) | Niet — geen Tabel 2.8 waarde herkenbaar. |
| H3 (NEN 8088-1 exterior-keying) | **Sterk** — eenheid `m³/(s·m² A_ext)`, niet `A_g`. Waarde `0,209` is plausibel een **NEN 8088-1 forfaitaire** specifieke kierstroming uit oude editie. |
| H5 (NIEUW) **Vabi 2017 verdeelt infiltratie alleen over hal/badkamer/inpandig, en stopt het bij verwarmde verblijfsruimten in `phi_v`** | **Sterk** — `0,000000 m³/s × X m²` letterlijk in rapport voor woon-/slaap-/keuken-rooms. Consistent met norm-formule 3.3: `Φ_vent = Φ_v − Φ_i` (infiltratie afgetrokken van ventilatie, dus al onderdeel van Φ_v voor systeem A/C). |

**Hoofdverdict Vrijstaande woning:** ISSO 51:2017 + Vabi rekenkern 2.30 hanteert een **andere splitsing** tussen Φ_i en Φ_v dan onze code. Φ_i wordt per kamer **niet** universeel toegekend; verblijfsruimten met natuurlijke ventilatie (systeem C) krijgen `Φ_i = 0` omdat de infiltratie al in Φ_v zit (formule 3.3 conform). Onze code dubbel-telt potentieel. **Zekerheid middel — niet uit ISSO 51:2023 te verifiëren want fixture is 2017-norm.**

---

## 5. Portiekwoning — reconstructie (ISSO 51:2017)

### 5.1 Bouw-context

| Parameter | Waarde | Bron |
|---|---|---|
| Gebouwtype | porch (etage flat-/portiek) | fixture |
| `qv;10` | 100,0 dm³/s | fixture |
| `A_g,totaal` | 85 m² | fixture |
| Vent. systeem C → `f_inf` | 1,0 | tabel 2.5 |
| Fixture `infiltration_method` | **niet aanwezig** → default = `per_exterior_area` | fixture + enums.rs:199 |
| Tabel 2.8:2023 rij voor portiek | 0,5 dm³/(s·m² Ag) | norm |

### 5.2 Onze code-uitkomst per ruimte (uit `portiekwoning_result.json`)

| Vertrek | h_i [W/K] | z_i | Φ_i [W] | Δθ |
|---|---|---|---|---|
| r1 Woonkamer | 2,713 | 1,0 | 81,39 | 30 |
| r2 Keuken | 1,413 | 1,0 | 42,39 | 30 |
| r3 Badkamer | (niet getoond, A_ext≈0) | — | — | — |
| r7 Entree | 1,114 | 1,0 | 27,86 | 25 |

`qi_spec = qi_spec_per_exterior_area(100) = 0,16`. Voor r1 Woonkamer: `q_i = 0,16 × A_ext = 0,16 × 14,13 = 2,261 dm³/s`. `h_i = 1,2 × 2,261 = 2,713 W/K`. ✓ Match `_result.json`.

### 5.3 Norm-conforme verwachting (ISSO 51:2023)

| Route | Berekening | Resultaat |
|---|---|---|
| Forfaitair (2.35), portiek tussenligging | `q_i = 1·1·1·0,5·85 = 42,5 dm³/s` gebouw → pro rata A_g | 0,5 dm³/s per m² Ag |
| Meting (2.34) | `q_i = 1·1·1·100 = 100 dm³/s` gebouw → 100/85 = 1,176 per m² Ag | 1,176 dm³/(s·m² Ag) |

Onze code: `0,16 × A_ext = 2,261 dm³/s` voor r1. Vertaald per A_g: 2,261/28,2 = 0,080 dm³/(s·m² Ag). **Dat is ruim onder de norm-forfaitaire 0,5.**

### 5.4 Verdict Portiekwoning

Onze portiekwoning-pad gebruikt **`per_exterior_area`** met `0,16 dm³/(s·m² A_ext)`. Vergeleken met norm-Tabel 2.8 (0,5 dm³/(s·m² A_g) voor portiek) en gebouw-keten zonder factoren: **onze keten zit ~6× te laag**. Er is geen Vabi-rapport voor deze fixture beschikbaar om absolute waarheid te vergelijken; vergelijking is alleen tegen norm.

| Hypothese | Bewijs |
|---|---|
| H1 (Tabel 2.8 puur, gebouwtype-keying) | Best zekere richting voor portiek — Tabel 2.8 zegt expliciet 0,5 dm³/(s·m² Ag) voor "etages van flat- en portiekwoningen". |
| H2 (Tabel 2.8 × f_type2) | f_type2=1,0 (standaard) → identiek aan H1. |
| H3 (NEN 8088-1 exterior) | onbevestigd — onze 0,16 valt onder de plausibele NEN 8088-1 reeks. |

**Hoofdverdict Portiekwoning:** Issue C symptomen worden hier verklaard door (a) **default `infiltration_method = PerExteriorArea`** terwijl voor flat/portiek de ISSO 51:2023 forfaitaire route specifiek `A_g`-keying voorschrijft, (b) **Tabel 2.8 ontbreekt voor gebouwtype-keying** in onze code, (c) **alle vijf correctiefactoren ontbreken**. Voor portiek tussenligging is `q_i,kamer = 0,5 · A_g,kamer · f_jaar · f_tp · f_inf · f_type2 · f_wind` (forfaitair) of `qv;10 · f_corr / A_g,totaal · A_g,kamer` (meting). **Zekerheid hoog op norm-richting, geen referentie-rapport ter verificatie.**

---

## 6. Welke correctie hebben we werkelijk nodig?

### 6.1 Code-wijzigingen — overzicht

| Wijziging | Bestand | Wat | Waarom |
|---|---|---|---|
| **A** — Tabel 2.8 vervangen | `tables/infiltration.rs` | Reeks 0,04..0,16 → 3-rijige gebouwtype-tabel (1,0 / 0,7 / 0,5) | Norm-conform met Tabel 2.8:2023 |
| **B** — Methode-router | `model/building.rs` + `calc/room_load.rs` | Nieuwe `InfiltrationMethod::ForfaitairTabel28` + `Measured` (op qv;10) | Twee paden formule 2.34 vs 2.35 |
| **C** — Correctiefactoren laag | `calc/infiltration.rs` | Functies `f_wind(L,B,H)`, `f_inf(ventilatie_systeem)` (Tabel 2.5), `f_type2(gebouw_subtype)` (Tabel 2.6), `f_tp(ligging)` (Tabel 2.7), `f_jaar(bouwjaar)` (formule 2.38) | Vermenigvuldiging in `q_i`-berekening |
| **D** — Tabel 4.3 verwijderen | `tables/infiltration.rs` | `qi_spec_per_exterior_area` deprecated of behouden onder NEN 8088-1 vlag | Tabel 4.3 bestaat niet in 2023-norm |
| **E** — Per-kamer pro-rata | `calc/room_load.rs:103-119` | Vermenigvuldig gebouw-q_i × `A_g,kamer / A_g,totaal,verwarmd` | Vabi-conform: gebouw-keten → kamer-toedeling |
| **F** — Φ_v − Φ_i splitsing (formule 3.3) | `calc/ventilation.rs` of `lib.rs` | Bij systeem A/C: per-kamer `Φ_vent = max(0, Φ_v − Φ_i)` ipv `Φ_v` | Voorkomt dubbeltelling; matcht Vabi 2017-gedrag |

### 6.2 Voorgestelde nieuwe `tables/infiltration.rs` (schets — niet implementeren)

```rust
//! Infiltratie-tabellen ISSO 51:2023 §2.5.6, p.39-41.

use crate::model::enums::{BuildingType, VentilationSystem, GeveltypeMeerlaags, Ligging};

/// Tabel 2.5 — invloed ventilatiesysteem.
pub fn f_inf(systeem: VentilationSystem) -> f64 {
    match systeem {
        VentilationSystem::A => 0.80,
        VentilationSystem::B => 0.85,
        VentilationSystem::C => 1.00,
        VentilationSystem::D => 1.10,
        VentilationSystem::E => 1.05,
    }
}

/// Tabel 2.6 — winddrukverdeling/thermiek (gebouwtype × geveltype).
pub fn f_type2(bt: BuildingType, geveltype: GeveltypeMeerlaags) -> f64 {
    // Voor grondgebonden (detached/semi/terraced/end): altijd 1,0.
    // Voor meer lagen (porch/flat): standaard 1,0,
    //  binnengalerij 0,94, dubbele huidgevel onderbroken 0,90, doorlopend 0,30.
    ...
}

/// Tabel 2.7 — invloedfactor ligging (gebruikt in formule 2.37 → qv,10,spec).
pub fn f_tp(bt: BuildingType, ligging: Ligging) -> f64 {
    // Eengezinswoningen met kap: tussenligging 1,0, kop/eind/hoek 1,2,
    //  vrijstaand puntdak 1,4, vrijstaand half platdak 1,2.
    // Etage van woongebouw met meer lagen: tussenligging 1,0,
    //  kop/eind/hoek onderste/tussen 1,3, tussen op bovenste 1,2,
    //  kop/eind/hoek op bovenste 1,4.
    // Schilberekening 1,2.
    ...
}

/// Formule 2.38 — leeftijdsfactor grondgebonden woning.
pub fn f_jaar(bouwjaar: u16) -> f64 {
    let f = ...; // exponentiële formule uit p.40, niet uit ext-tekst extraheerbaar
    f.clamp(0.7, 4.3)
}

/// Tabel 2.8 (ISSO 51:2023 p.41) — `q_i,spec` per m² A_g.
/// Forfaitaire fallback wanneer qv;10 niet gemeten is.
pub fn q_i_spec_tabel_2_8(bt: BuildingType) -> f64 {
    match bt {
        BuildingType::Detached | BuildingType::SemiDetached
        | BuildingType::Terraced | BuildingType::EndOfTerrace => {
            // Eengezinswoning — kap of half platdak → 1,0
            // Eengezinswoning — platdak → 0,7
            // Onderscheid op basis van dak-type uit Building (nu nog niet beschikbaar)
            1.0
        }
        BuildingType::Porch /* | Flat */ => 0.5,
    }
}

/// Formule 2.36 — winddruk-correctie door gebouwafmetingen.
pub fn f_wind(l: f64, b: f64, h: f64) -> f64 {
    let f = ...; // formule 2.36, niet uit ext-tekst extraheerbaar
    f.max(1.0)
}
```

### 6.3 Impact-analyse — numeriek per fixture

| Fixture | Huidige Φ_i (engine) | Vabi/expected Φ_i | Na voorgestelde fix (formule 2.34 + `f_inf`) | Resterende drift |
|---|---|---|---|---|
| **DR Engineering** (gemeten qv;10=152, sys D, eengezins kap) | 1013 W gebouw / 64 W kamer 0.01 | 2003 W gebouw / 127 W kamer 0.01 | `q_i,gebouw = 1·1,10·1·152 = 167,2 dm³/s` → Φ_i,gebouw ≈ `1,2·167,2·Δθ_gem` ≈ 3500 W. Per kamer 0.01 ≈ 275 W. | **2× te hoog**. Vabi past blijkbaar **een extra factor 0,46** toe; bron onbekend (mogelijk NEN 8088-1 `f_z` of verdeel-factor). |
| **Vrijstaande woning** (gemeten qv;10=110,2, sys C, ISSO 51:2017) | 85 W gebouw (expected) | 85 W gebouw expected, maar **0 voor verblijfsruimten** | Na fix systeem C → `f_inf=1,0`: q_i,gebouw = 110,2 dm³/s → te hoog tov 85. Voor verblijfsruimten formule 3.3 zou `Φ_vent = max(0, Φ_v − Φ_i)` toepassen — eerst Φ_v meten dan splitten. | Vereist **fix F** (Φ_v − Φ_i splitsing) bovenop. |
| **Portiekwoning** (gemeten qv;10=100, sys C, ISSO 51:2017) | Σ Φ_i ≈ 250 W | onbekend (geen Vabi-ref) | Forfaitair Tabel 2.8: `0,5·85 = 42,5 dm³/s` gebouw → Φ_i,gebouw ≈ 1100 W. Meting: `100 dm³/s` → 2600 W. | Onbekend zonder Vabi-rapport voor deze fixture. |

### 6.4 Onbekende factor ~0,461 — uitdiepen

DR Vabi heeft `q_i/A_g = 0,317` ipv de norm-formule `0,6875 (= f_inf·qv;10/A_g)`. Constante ratio `0,461`. Hypothesen:
1. **NEN 8088-1 fractieverdeling `f_z`** — *"Correctie op het gesommeerd infiltratie-warmteverlies doordat de wind niet tegelijk op alle buitengevels zal staan"* (zie Vabi 2017 rapport voetnoot, niet gegeven exacte waarde, maar Vabi documenteert het concept).
2. **`A_g,verwarmd / A_g,totaal`** scaling: gebouw heeft onverwarmde rooms (meterkast, toilet BG, garage, etc., niet duidelijk hoeveel in DR). Niet getest.
3. **Vabi rekenkern proprietary** — `0,461` matcht geen ISSO-tabelwaarde, geen `f_v`, geen `f_type2`. Mogelijk een eigen Vabi-correctie buiten norm.

**Te onderzoeken vóór code-implementatie:** vraag bij Vabi naar de invul van **`f_z`** of equivalent. Of haal NEN 8088-1 erbij voor verificatie van `0,461`. Zonder dit zal onze fix `formule 2.34 + f_inf` per kamer een factor 2 te hoog uitkomen versus Vabi.

---

## 7. Open vragen die deze diagnose niet kon oplossen

1. **Oorsprong factor `0,461` in Vabi DR.** ISSO 51:2023 PDF p.37–41 vermeldt geen zodanige sub-factor. Vabi-rapport p.3 toont `f_wind=1`, `f_type2=1`, en per-room `f_inf=1,10`. Het product `1·1·1,10 = 1,10`, niet `0,506`. Verschil moet ergens anders zitten — vermoedelijk NEN 8088-1 `f_z` (windrichtingsverdeling). NEN 8088-1 is niet in repo aanwezig.
2. **Exacte formule 2.36 voor `f_wind`.** PDF heeft de formule als afbeelding (geen tekstlaag). Voor lage gebouwen (H<13m) is `f_wind=1,0` (p.38), wat alle drie fixtures dekt, maar voor flat/portiek met H>13m zouden we de formule moeten extraheren of OCR'en.
3. **Exacte formule 2.38 voor `f_jaar`.** Idem afbeelding. Tekst zegt `0,7 ≤ f_jaar ≤ 4,3` en variabelen `e` (=2,718) en `J` (bouwjaar). Implementeerbaar maar tekst niet automatiseerbaar geëxtraheerd.
4. **Onderscheid eengezins-kap vs eengezins-platdak in Tabel 2.8.** Onze `Building` heeft geen `dak_type`-veld. Vereist data-model uitbreiding.
5. **Gedrag systeem A/C in Vabi 2017 (Φ_i=0 voor verblijfsruimten).** Lijkt formule 3.3 (`Φ_vent = Φ_v − Φ_i`) — maar Vabi past dat **op kamerniveau** toe en zet `Φ_i = 0` waar `Φ_v > 0`. ISSO 51:2023 (p.58) formuleert 3.3 op vertrekniveau. Bij DR (systeem D mech.) telt 3.3 niet en wordt `Φ_i` apart gerapporteerd. **Onze code mist deze splitsing volledig.**
6. **Is Vabi 2017 = ISSO 51:2017 == norm?** Vrijstaande woning fixture matcht Vabi rapport exact (expected = Vabi). Maar 2017-norm is potentieel een andere tabelnummering. Op deze fixture niet 1-op-1 met ISSO 51:2023 te toetsen.

---

## 8. Conclusie — fix-richting

**Drie genuanceerde verdicten:**

| Fixture | Vabi-keten verklaard? | Aanbevolen code-fix richting | Zekerheid |
|---|---|---|---|
| DR Engineering | Deels — formule 2.34 + `f_inf=1,10` + onbekende factor ~0,461 | Implementeer formule 2.34 + Tabel 2.5 (`f_inf`). Onderzoek `0,461` (waarschijnlijk NEN 8088-1 `f_z`). | Middel — richting hoog, magnitude pending NEN 8088-1 inzage |
| Vrijstaande woning | Ja — Vabi 2017 gebruikt formule 3.3 (`Φ_vent = Φ_v − Φ_i`) op kamerniveau + exterior-keying voor inpandige ruimten | Implementeer Φ_v − Φ_i splitsing voor systeem A/C, behoud `per_exterior_area` voor sub-pad inpandig | Middel |
| Portiekwoning | Niet vergeleken met Vabi (geen referentie) | Implementeer Tabel 2.8:2023 (gebouwtype-keying, 3 rijen) + correctiefactoren | Hoog op norm-conformiteit, geen Vabi-cross-check |

**Hoofdactie voor volgende sessie:**
1. Bouw `Building` data-model uit met `dak_type` + `ligging` + `bouwjaar` + `vent_systeem` (zo nodig sub-typeren).
2. Vervang `qi_spec_per_floor_area` / `qi_spec_per_exterior_area` door norm-conforme `q_i_spec_tabel_2_8(bt, dak_type)` + correctiefactoren-laag.
3. Voeg `InfiltrationMethod::MeasuredQv10` toe (formule 2.34) naast `Forfaitair` (formule 2.35).
4. Splits `Φ_v − Φ_i` voor systeem A/C op kamerniveau (formule 3.3).
5. **Voorafgaand aan punt 1–4: NEN 8088-1 inzage** om factor `0,461` op te lossen — anders blijft DR-fixture 2× te hoog.

---

**Einde rapport.**
