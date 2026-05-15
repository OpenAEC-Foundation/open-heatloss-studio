# Diagnose drie open issues — isso51-core

**Datum:** 2026-05-12
**Scope:** statisch onderzoek + handmatige rekening, geen code wijzigen
**Test-output basis:** `cargo test --package isso51-core --test integration_test` op master `2dc144d`
**Modus:** READ-ONLY — geen formules verzonnen, geen tests aangepast, geen code gewijzigd.

---

## Korte samenvatting vooraf

| Issue | Werkelijkheid | Echte bug? |
|---|---|---|
| A | `connection_capacity` = lineaire optelsom van **W/K-componenten** (envelope + adj_buildings + Φ_v_full + Φ_hu + Φ_sys), **niet** Σ(per_room `total_heat_loss`) en **niet** kwadratische sommatie. Audit-claim "8121 W lineair" was *mis-geframed*: 8121 zou Σ(per_room phi_hl_i) zijn, maar engine produceert 6076 W — een derde aggregatieroute. | Ja, twee bugs (zie hieronder) |
| B | Portiekwoning room-niveau drift = gevolg van **één semantische wijziging**: `phi_t_adjacent` (binnen woning) telt nu mee in `phi_basis_no_sys`. Stale `portiekwoning_result.json` mist die bijdrage. r7 Entree clamp naar 0 = direct gevolg (basis wordt negatief). r3 Badkamer +123% = direct gevolg (5.82 × 32 = 186 W bijgeteld). | Genuanceerd: code-pad wijziging zonder fixture-update |
| C | `phi_i` exact 50% van Vabi. Vabi heeft `qv,10=152, Ag=243.2 → qv,10/Ag ≈ 0.625` en mapt blijkbaar naar de hoogste klasse `qi_spec = 0.32` dm³/(s·m²). Engine `qi_spec_per_floor_area(152.0)` retourneert `0.16` (klasse 100<qv≤150 omdat lookup absolute qv10 vergelijkt). | Ja, tabel-keuze fout |

---

## Issue A — `build_summary` werkelijke aggregatie-keten

### Wat de code doet (regel-voor-regel, `lib.rs:149-187`)

```rust
fn build_summary(rooms: &[result::RoomResult], theta_e: f64) -> BuildingSummary {
    let mut total_envelope_loss = 0.0;
    let mut total_neighbor_loss = 0.0;
    let mut total_ventilation_loss = 0.0;
    let mut total_heating_up = 0.0;
    let mut total_system_losses = 0.0;
    for r in rooms {
        let theta_diff = r.theta_i - theta_e;
        total_envelope_loss += r.transmission.h_t_exterior * theta_diff   // [lib.rs:159]
            + r.transmission.h_t_unheated * theta_diff
            + r.transmission.h_t_ground * theta_diff
            + r.transmission.h_t_water * theta_diff;
        total_neighbor_loss += r.transmission.h_t_adjacent_buildings * theta_diff;  // [lib.rs:164]
        total_ventilation_loss += r.ventilation.phi_v;                    // [lib.rs:166] — FULL φ_v, niet φ_vent
        total_heating_up += r.heating_up.phi_hu;
        total_system_losses += r.system_losses.phi_system_total;
    }
    let connection_capacity = total_envelope_loss + total_neighbor_loss
        + total_ventilation_loss + total_heating_up + total_system_losses; // [lib.rs:171-172]
    ...
}
```

Kerneigenschappen:
1. **Niet** een sommatie van `room.total_heat_loss`. Engine herberekent vanaf raw H_T's en raw Φ_v's.
2. `h_t_adjacent_rooms` (binnen woning) wordt **niet meegenomen** — terecht, want is internalflows, telt niet mee in gebouw-totaal.
3. `total_ventilation_loss` gebruikt `phi_v` (gross) en niet `phi_vent` (`= φ_v − φ_i` in erratum 3.3). Daardoor wordt infiltratie zowel in `phi_i` (via `phi_t_*` zou je verwachten — maar zit NIET in deze sommatie!) als impliciet in `phi_v` opgenomen. **Φ_i ontbreekt volledig uit het gebouw-totaal.**
4. Geen kwadratische sommatie.

### Wat dit voor DR Engineering oplevert (handmatig nageteld vanuit test-output)

Per-vertrek `phi_i` actual (test output):
```
0.01: 64.03  0.02: 5.89  0.03: 263.23  0.04: 234.72  0.05: 29.35
1.02: 53.28  1.03: 48.04  1.04: 141.47  1.05: 36.29  1.08: 137.09
Σ phi_i = 1013.4 W   (volledig genegeerd in connection_capacity)
```

Σ per-room `total_heat_loss` (test output):
```
505 + 0 (clamped) + 2063 + 1791 + 293 + 255 + 234 + 536 + 228 + 1233 = 7138 W
```

Engine-produced `connection_capacity = 6076 W`. Verschil 7138 − 6076 = 1062 W ≈ Σphi_i (1013 W) — bevestigt dat de bijdrage van infiltratie via een ander aggregatie-pad is uitgevlakt.

Vabi expected `phi_hl_build = 6700 W` = `phi_basis(5931) + phi_extra_quadratic(770)` waarbij `phi_basis` Σ(phi_t_ie + phi_t_iae + phi_t_ig + phi_i) = 3601 + 0 + 326 + 2003 = 5930. **Vabi telt Σphi_i wél lineair in basis.**

### Verschil met audit-claim

| Bron | Beoogde route | Resultaat DR |
|---|---|---|
| Audit-tekst | "lineaire som per-room phi_hl_i" | 7138 W (Σ phi_hl_i actual) of 8123 W (Σ phi_hl_i expected) |
| Erratum 3.11 | `Σphi_basis + √(Σphi_vent² + Σphi_iaBE² + Σphi_hu²)` | 6700 W (Vabi) |
| Engine werkelijk | `Σ(envelope) + Σ(adj_b) + Σphi_v + Σphi_hu + Σphi_sys`, **excl. phi_i** | 6076 W |

De audit beschreef de bug correct (geen kwadratische som op gebouwniveau) maar mis-noemde de getallen-orde. De werkelijke aggregatie heeft **twee fouten**:

### Geherijkte audit-claim

> `build_summary` (`lib.rs:149-187`) produceert `connection_capacity` als een lineaire optelsom van W/K-bijdragen op gebouwniveau, **niet** als kwadratische som conform erratum formule 3.11. Bovendien ontbreekt `phi_i` (infiltratie) volledig uit de sommatie: `total_envelope_loss` somt alleen `h_t_ie + h_t_io + h_t_ig + h_t_iw` × Δθ, en `total_ventilation_loss` gebruikt `phi_v` (gross) i.p.v. `phi_vent` (= φ_v − φ_i conform formule 3.3). Voor de DR Engineering fixture levert dat een **dubbel falen**: kwadratische som ontbreekt (te hoog risico op overschatting) én infiltratie wordt niet meegerekend (~1013 W onderschatting). Toevallig combineren beide tot een onderschatting (−624 W) voor déze fixture, maar de mate van compensatie is fixture-afhankelijk en willekeurig.

---

## Issue B — Portiekwoning `phi_hl_i` drift

### Aggregatieketen per ruimte (`calc/room_load.rs:271-303`)

```rust
let phi_t_exterior   = h_t_ie * (theta_i - theta_e);   // [room_load.rs:272]
let phi_t_adjacent   = h_t_ia * (theta_i - theta_e);   // [room_load.rs:273] — *adjacent room*, binnen woning
let phi_t_unheated   = h_t_io * (theta_i - theta_e);
let phi_t_ground     = h_t_ig * (theta_i - theta_e);
let phi_t_water      = h_t_iw * (theta_i - theta_e);
let phi_basis_no_sys = phi_t_exterior + phi_t_adjacent + phi_t_unheated
                     + phi_t_ground + phi_t_water + phi_i;          // [room_load.rs:281-282]

let phi_t_adj_building = h_t_ib * (theta_i - theta_e);              // [room_load.rs:284]
let phi_extra = quadratic_sum(phi_vent, phi_t_adj_building, phi_hu);// [room_load.rs:285]

// (geen embedded heating in deze fixture → f_sys_total = 0)
let total = phi_basis_no_sys + phi_extra;                           // [room_load.rs:299]
let total = if room.clamp_positive { total.max(0.0) } else { total };// [room_load.rs:303]
```

**Cruciaal:** `phi_t_adjacent` (h_t_ia × Δθ, binnen woning) zit ín `phi_basis_no_sys`. Het opgeslagen `portiekwoning_result.json` is écht-stale — `basis_heat_loss` daar bevat *niet* deze adjacent-room bijdrage (zie r3 Badkamer: `basis_heat_loss = 0` terwijl `h_t_ia × 32 = 186 W` actief is). Dat is dus opgeslagen output van een **vroegere engine-versie waarin formule 4.5.3 §Φ_basis géén h_t_ia × Δθ-term had**.

### Handmatige reconstructie voor 3 vertrekken

Gebruikt: oude opgeslagen `_result.json` cijfers + nieuwe `total_heat_loss` uit test-run.

| Vertrek | oude basis | oude extra | oude total | h_t_ia | Δθ | h_t_ia·Δθ | nieuwe total (verwacht = oud + h_t_ia·Δθ, clamped) | actual test | match |
|---|---|---|---|---|---|---|---|---|---|
| r1 Woonkamer | 801.26 | 979.70 | 1780.96 | 1.510 | 30 | +45.30 | 1826.26 | 1896.27 | Δ +70 (zie noot 1) |
| r3 Badkamer | 0.00 | 151.62 | 151.62 | 5.822 | 32 | +186.32 | 337.94 | 337.93 | **exact** |
| r7 Entree | 27.86 | 132.23 | 160.09 | −11.021 | 25 | −275.53 | max(160.09−275.53,0) = 0 | 0.00 | **exact** |

**Noot 1 — r1 extra 70 W drift:** kan niet volledig uit één regel verklaard worden zonder ook `phi_hu`, `phi_vent` of `phi_t_adj_building` te herberekenen. Mogelijke factoren (niet bevestigd in code, hypothesen):
- `f_rh` waarde 1.7 in oud result, mogelijk nu door `tables/heating_up.rs::heating_up_factor` interpolatie iets anders. Geen veranderingen in code geconstateerd, maar testresultaat suggereert verschillen.
- `phi_t_adj_building = 16.06 × 30 = 481.8 W` constante. `phi_vent = 832.29 W` constante. `phi_hu` mogelijk hoger door iets vroeg in Pass 2.
- Voor r4, r5, r6 zijn de drifts ook +127/+55/+85 W, niet exact `h_t_ia × Δθ` (3.03·30=91, 0.15·30=4, 1.66·30=50). Reststaak wijst op een tweede, kleinere drift in Φ_hu Pass 2 (main-room percentage methode). Onbevestigd.

### r7 Entree clamp-pad

`room_load.rs:303`: `let total = if room.clamp_positive { total.max(0.0) } else { total };`
Voor Entree (r7): `phi_t_adjacent = -11.021 × 25 = -275.5` → `phi_basis_no_sys = -275.5 + 0 + 1.114·25 + 0 + 0 + 0 = -247.6`. Plus `phi_extra = √(132² + 0² + 0²) = 132`. Plus `f_sys_total = 0`. → `total = -115.6`. `clamp_positive = true` (default) → `0.0`. **De oude `_result.json` heeft basis_heat_loss=27.85 en total=160.09 omdat in de eerdere code-versie `h_t_ia × Δθ = -275.5` niet werd opgenomen in basis.**

### r3 Badkamer +123% drift

r3 heeft `h_t_exterior = 0` en `h_t_ground = 0`. Alleen `h_t_ia` (binnen woning, 5.822 W/K naar omliggende ruimten) en `h_t_ib` (adj_building, 2.795 W/K). Oude code: basis = phi_i + phi_t_exterior + … = 0. Nieuwe code: basis += `h_t_ia × Δθ = 186.32`. Resultaat: `0 + 186.32 + 151.62 = 337.94`. Identiek aan testoutput. Het +123% is **één-op-één afgeleid van** de toevoeging van phi_t_adjacent in basis.

### Conclusie per regressie

Drie van de zeven mismatches zijn 100% verklaard door één semantische wijziging: **`phi_t_adjacent` (binnen woning) opnemen in `phi_basis_no_sys`** (`room_load.rs:273+281`). Vier overige (r1/r4/r5/r6) bevatten een tweede, kleinere drift in Φ_hu (Pass 2) of phi_v die niet uit één-regel rekening volgt; deze verdienen aparte verificatie buiten dit rapport (regel-voor-regel `f_rh`-trace of een log van Pass 1 vs Pass 2 outputs).

Verdachte locaties (geen bug bevestigd, alleen kandidaten):
- `lib.rs:101-137` Pass 2 main-room selectie en hu_pct herberekening
- `tables/heating_up.rs::heating_up_factor` interpolatie tussen 1.5 en 2.0 h

---

## Issue C — Infiltratie −50% op DR fixture

### Code-pad

`crates/isso51-core/src/calc/room_load.rs:103-123`:

```rust
let q_i = match building.infiltration_method {
    InfiltrationMethod::PerExteriorArea => {
        let qi_spec = tables::infiltration::qi_spec_per_exterior_area(building.qv10);
        // ... sum exterior area
        infiltration::infiltration_flow_rate(qi_spec, total_exterior_area)
    }
    InfiltrationMethod::PerFloorArea => {
        let qi_spec = tables::infiltration::qi_spec_per_floor_area(building.qv10);  // [105:117]
        qi_spec * room.floor_area                                                    // [105:118]
    }
};
let h_i = infiltration::h_infiltration(q_i);                                         // [room_load.rs:121]
let z_i = 1.0;                                                                       // [room_load.rs:122]
let phi_i = infiltration::phi_infiltration(h_i, z_i, theta_i, theta_e);              // [room_load.rs:123]
```

DR fixture: `building.qv10 = 152.0`, `infiltration_method = "per_floor_area"`, `total_floor_area = 243.2 m²`.

`tables/infiltration.rs:34-45` (volledige tabel):
```rust
pub fn qi_spec_per_floor_area(qv10: f64) -> f64 {
    if qv10 <= 50.0  { 0.04 }
    else if qv10 <= 100.0 { 0.08 }
    else if qv10 <= 150.0 { 0.12 }
    else                  { 0.16 }
}
```

Met `qv10 = 152` valt het in laatste branche → `qi_spec = 0.16` dm³/(s·m²).

### Per-room handmatige berekening (3 steekproeven)

| Room | A_g | θ_i | Δθ | engine qi_spec | engine phi_i = 0.16·A_g·1.2·Δθ | actual test | match | expected (Vabi) | impliciete Vabi qi_spec |
|---|---|---|---|---|---|---|---|---|---|
| 0.01 Entree   | 11.91 | 20 | 28 | 0.16 | 64.03  | 64.03  | ✓ | 127 | **0.317** |
| 0.03 Woonkamer | 45.70 | 22 | 30 | 0.16 | 263.23 | 263.23 | ✓ | 520 | **0.316** |
| 1.04 Slaapkamer 1 | 24.56 | 22 | 30 | 0.16 | 141.47 | 141.47 | ✓ | 280 | **0.317** |

Vabi impliciete qi_spec ≈ **0.316–0.317 dm³/(s·m²)** = factor 2 × engine. Dat is precies de eerstvolgende klasse: `0.32` (de waarde uit `qi_spec_per_exterior_area` voor qv10 > 150, of een vermoede hogere Tabel 2.8 klasse).

### Hoofdoorzaak — drie hypothesen, één favoriet

| Hypothese | Bewijs voor | Bewijs tegen |
|---|---|---|
| (a) **Tabel 2.8 erratum heeft een vijfde klasse die we missen.** Bijv. `qv10 > 150 → 0.32`, niet `→ 0.16`. | Vabi resultaten matchen op 0.32 vrijwel exact. Onze code stopt bij 4 klassen, terwijl `qi_spec_per_exterior_area` 4 klassen heeft met 2× hogere absolute waardes (0.08/0.16/0.24/0.32). Symmetrie suggereert dat de echte tabel ook 0.04/0.08/0.12/0.16/0.32 of 0.04/0.08/0.16/0.24/0.32 heeft, of zelfs alle 5 verdubbeld bij `qv10 > 150`. | Niet bevestigd in code — geen norm-tekst in repo om Tabel 2.8 te verifiëren. |
| (b) **Vabi gebruikt `qv,10/A_g`-key i.p.v. absolute `qv10`.** `152/243.2 = 0.625` valt mogelijk in een hogere genormeerde klasse. | Verklaart de factor 2 zonder klasse-uitbreiding nodig te hebben. | Niet bevestigd in code. Geen hint in fixture-note. |
| (c) **Vabi past zijn 1.10 systeem-D correctie op een andere noemer toe.** | Note in `_result.json:8`: *"correctie 1.10 (systeem D) — verschilt van engine"* | 1.10 verklaart geen factor 2.0; 50% drift is veel groter dan 10%. **Onvoldoende verklaring.** |

**Favoriet:** (a) of (b). Beide zouden de drift volledig verklaren. (c) is een Vabi-eigen niet-genormeerde 10% correctie die de **resterende** 5–6 W per room verklaart, niet het 2× verschil.

**Niet bevestigd in code:** of `tables::infiltration::qi_spec_per_floor_area` werkelijk de juiste erratum Tabel 2.8 weergeeft of een onvolledige transcriptie is. Audit (regel 76, regel 114 in audit-doc) claimt "match Tabel 2.8 (erratum E.5)" maar zonder norm-citaat — dat is een onbevestigde assertie.

---

## Samenvattende conclusie

| Issue | Status | Echte bug? | Locatie | Fix-richting |
|---|---|---|---|---|
| A | bevestigd | **Ja, twee bugs** | `lib.rs:149-187` | (1) Voeg `Σphi_i` toe aan `total_envelope_loss` of als apart veld. (2) Vervang lineaire som door kwadratische sommatie op gebouwniveau conform erratum 3.11 + `phi_vent = phi_v − phi_i` conform formule 3.3. Aanbeveling: één PR met beide fixes plus een uitbreiding van `BuildingSummary` met `phi_extra` decompositie (`phi_vent_total`, `phi_t_iaBE_total`, `phi_hu_total`). |
| B | grotendeels bevestigd | **Genuanceerd** | `room_load.rs:273+281` | Stale `portiekwoning_result.json`: regenereer met huidige engine of besluit dat oude semantiek (h_t_ia ∉ basis) terug moet. **Eerst norm-uitspraak nodig:** hoort `phi_t,ia` (binnen woning) in `phi_basis` van een vertrek thuis? Formule 4.2 (`Φ_T = H_T·Δθ` waar `H_T = ΣH_T,ie + …`) suggereert van wel, maar Vabi-formaat in DR-fixture splitst `phi_t_ie + phi_t_ia + …` waarbij `phi_t_ia` wél in basis valt (zie DR Woonkamer: basis=2101 = 878+381+174+148+520, dus h_t_ia → phi_t_ia=381 zit erin). **Engine-gedrag lijkt dus norm-conform; fixture is stale.** |
| C | bevestigd | **Ja, tabel-keuze fout** | `tables/infiltration.rs:34-45` | Verifieer Tabel 2.8 erratum-tekst tegen ISSO 51:2024 print. Verwachte uitkomst: vijfde klasse of `qv,10/A_g`-keying. Voorlopige fix-hypothese: voeg `if qv10/A_g > 0.6 → 0.32` of vergelijkbaar toe. Niet implementeren voordat norm-citaat is bevestigd. |

---

## Onbeantwoorde vragen

1. **Restdrift r1/r4/r5/r6 portiekwoning (10–18 W bovenop `h_t_ia·Δθ` shift).** Niet kunnen herleiden uit één regelwijziging. Vermoeden: tweede-orde drift in `phi_hu` Pass 2 of `phi_v` met gewijzigde `f_v` of `delta_v` selectie. Vereist instrumentatie/log-trace van een vertrek door beide passes om vast te stellen.
2. **Is de huidige `phi_t_adjacent ∈ basis` semantiek norm-conform?** Engine-gedrag lijkt te matchen met Vabi (DR fixture splitst `phi_t_ie + phi_t_ia + phi_t_iae + phi_t_ig` als basis-componenten), maar het ISSO 51 erratum §4.5.3 zelf is niet in deze repo aanwezig om vers te citeren. Audit zegt impliciet "✓ grotendeels conform" zonder dit punt expliciet te benoemen.
3. **Tabel 2.8 erratum exacte waardes.** Geen norm-PDF in repo om te dubbelchecken. Audit zegt "match Tabel 2.8 (erratum E.5)" maar de Vabi-resultaten tonen 2× hogere `qi_spec` dan onze tabel — een van beide is fout. Externe norm-verificatie nodig.
4. **`phi_t_adj_building` (h_t_ib) in `phi_extra` (kwadratisch) i.p.v. `phi_basis` (lineair):** Het opnemen van `phi_t_adj_building` in de kwadratische som (`quadratic_sum.rs:21`, conform erratum formule 3.11) is logisch op gebouwniveau (woningscheidende wand is niet-simultaan). Maar op vertrekniveau wordt het *ook* in de kwadratische som van die ene ruimte gestopt. Of dat op vertrekniveau klopt is niet zelfstandig na te lezen; geen vergelijking met norm in deze diagnose.
5. **DR fixture phi_v expected = 770 W, gebruikt in `phi_extra`.** Engine `phi_vent` per ruimte matcht goed maar Vabi `phi_vent` gebouwsom = 770 (lineaire som over rooms van phi_vent). Engine reproduceert het niet als output veld op gebouwniveau (zit alleen in connection_capacity). Geen mismatch in test (test extract gebruikt `phi_vent` per room voor `phi_v` mapping voor DR — werkt) maar audit-aanbeveling #5 (`phi_vent_total` veld toevoegen) blijft nuttig.
6. **Vabi 1.10 correctie systeem D.** Note in result-json suggereert het is een Vabi-eigen factor buiten norm. Of dat klopt is niet onafhankelijk te verifiëren binnen deze diagnose.
