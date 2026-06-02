# Audit 05 — UI-veld-dekking warmteverlies-invoer

**Scope:** ISSO 53 (utiliteit, `isso53-core`). De ISSO 51-keten (`isso51-core`) is
zijdelings meegenomen waar relevant. **Read-only audit** — geen wijzigingen.

**Vraag:** heeft elke invoer die de warmteverlies-berekening daadwerkelijk
*leest* ook een UI-control, zodat de gebruiker hem kan voeden i.p.v. stil terug
te vallen op een Rust-default?

**Methode:** Rust-model (`crates/isso53-core/src/model/*.rs`) gekruist met de
calc-modules (`calc/*.rs`, `lib.rs`) → lijst van échte calc-inputs. Daarna
gekruist met de frontend-mapper (`frontend/src/lib/isso53ProjectMapper.ts`, de
single bridge naar de kern) en de UI-componenten die de sidecar/V1-store voeden.

---

## Antwoord op de 5 prioriteitspunten

| # | Prioriteitspunt | UI-status | Conclusie |
|---|-----------------|-----------|-----------|
| 1 | **Opwarmtoeslag / nachtverlaging** (`heatingUp`-blok) | ✅ **Volledig gedekt** | `Isso53BuildingFields.tsx` rendert alle velden: `setbackActive`, regime (free/limited), `airChanges`, `warmupHoursWeekday/Weekend`, `setbackHoursWeekday/Weekend` (free) of `degreesWeekday/Weekend` (limited), `mechanicalSupplyOff`, `pWPerM2Override`. Mapt 1:1 op `HeatingUpConfig`. |
| 2 | **Vertrekhoogte / vide-hoogte (stratificatie)** | ⚠️ **Deels** | `room.height` is editable (`RoomHeaderRow.tsx`) en mapt naar `room.height`. **MAAR**: per-vertrek (hoofdstuk 4) leest de calc `room.height` niet — alleen `shell.rs` (voorontwerp-schilmethode) gebruikt het voor volume. Een aparte vide/stratificatie-hoogte >4 m bestaat niet in model of UI; ISSO 53 §2 vereist `height ≤ 4`. Geen veld voor verhoogde vertrekken. |
| 3 | **Koudebrug-toeslag ΔU_TB (forfaitair vs custom)** | ❌ **GAT** | `use_forfaitaire_thermal_bridge` + `custom_delta_u_tb` worden door de calc gelezen (`transmission.rs:70-77`), maar **geen UI-component schrijft ze**. Mapper hardcodet `useForfaitaireThermalBridge ?? true`, `customDeltaUTb ?? null`. Gebruiker kan forfaitair niet uitzetten of een eigen ΔU_TB opgeven. |
| 4 | **Ventilatiesysteem-parameters (WTW/systeemtype/debiet)** | ⚠️ **Grotendeels** | Systeemtype (A–E) ✅ via `Isso53BuildingFields`. WTW aan/uit + rendement ✅ via `VentilationPanel`. Per-vertrek q_v ✅ via `VentilationRow`. `supply_temperature` ✅ (gated achter WTW in `WarmteverliesInstellingen`). **GATEN:** `has_preheating`/`preheating_temperature` (voorverwarming, `ventilation.rs:144-153`) hebben **geen UI**; `frost_protection` heeft wél UI maar wordt door de isso53-kern **niet gelezen** (zie wezen). |
| 5 | **Verwarmingssysteemtype per ruimte** | ✅ **Gedekt** | `VentilationRow.tsx` heeft per-vertrek `heating_system`-dropdown. **Let op nuance:** de isso53-kern leest **`building.heating_system`** (gebouwbreed, voor grond-f_ig in `ground.rs`/`transmission.rs`), niet per-vertrek. De mapper distilleert dat uit `default_heating_system ?? rooms[0].heating_system` (`resolveHeatingSystem`). Per-vertrek instellen heeft dus alleen effect als het ook de eerste room/default beïnvloedt — Δθ-stratificatie per ruimte wordt in ISSO 53 niet per ruimte toegepast (anders dan ISSO 51). |

---

## Samenvattende tabel — alle calc-inputs

Legenda UI-status: ✅ gedekt · ⚠️ deels/indirect · ❌ gat · 🗑️ wees (UI zonder calc-effect)

| Veld (Rust-model) | Norm-relevantie | Leest calc het? | UI-status | Gevolg bij ontbreken |
|---|---|---|---|---|
| **Building** | | | | |
| `building_shape` | Infiltratie tabel 4.9 (Unknown-pad) | ja (Unknown) | ✅ `Isso53BuildingFields` | n.v.t. |
| `building_position` | Infiltratie tabel 4.8 | ja | ✅ `Isso53BuildingFields` | — |
| `wind_pressure_type` | Winddruk tabel 4.6 (Unknown) | ja (Unknown) | ✅ `Isso53BuildingFields` | — |
| `ventilation_system` | Tabel 4.7 / WTW-zichtbaarheid | ja (via VentPanel) | ✅ `Isso53BuildingFields` | — |
| `thermal_mass` | c_eff tabel 2.4 → opwarmtoeslag | ja (`heating_up.rs`) | ✅ `Isso53BuildingFields` | — |
| `construction_year` | Infiltratie formule 4.34 (Unknown) | ja (Unknown) | ✅ `Isso53BuildingFields` | Known-pad: irrelevant |
| `heating_system` | Δθ₂ grond-f_ig tabel 2.3 (`ground.rs`) | **ja** | ⚠️ indirect (afgeleid uit room/default) | Verkeerde f_ig vloer-op-grond bij vloerverwarming |
| `source_zone_config` | Infiltratie-fractie z tabel 5.1 (`lib.rs:75`) | **ja** | ❌ **GAT** | Φ_source altijd z=0.5 (Other); SeparatePerZone (z=1.0) onbereikbaar → bronvermogen onderschat |
| `building_height` | Infiltratie q_is tabel 4.5 + 4.32 | **ja** | ❌ **GAT** (mapper stuurt veld niet) | Valt op default 3.0 m → hoogbouw-infiltratie onderschat |
| `building_length` / `building_width` | f_wind formule 4.32 (Unknown) | ja (Unknown) | ❌ **GAT** (niet gemapt) | Known-pad: irrelevant; Unknown-pad: f_wind placeholder 1.0 |
| **Climate** | | | | |
| `theta_e` | Buitentemperatuur, overal | ja | ✅ `WarmteverliesInstellingen` | — |
| `theta_me` | Jaargemiddelde, grond-f_ig | ja (`ground.rs`) | ✅ `Isso53BuildingFields` (`thetaMe`) | — |
| `theta_b_adjacent_building` | θ_b buurpand (`transmission.rs:178`) | **ja** | ❌ **GAT** (mapper laat weg → serde-default 15) | Buurpand altijd 15 °C; vorstvrij/stalling (5 °C/θ_e) onbereikbaar |
| **Room** | | | | |
| `gebruiks_functie` / `ruimte_type` | θ_i tabel 2.2, vent-eis 4.10/4.11 | ja | ✅ `Isso53RoomFunctionCell` | — |
| `floor_area` | A_vl overal | ja | ✅ `RoomHeaderRow` | — |
| `height` | Volume (alleen shell-methode) | ja (shell) | ✅ `RoomHeaderRow` | Per-vertrek: niet gelezen |
| `custom_temperature` | θ_i override | ja | ✅ (room temp override) | — |
| `bezetting.personen` | Vent-eis personen | ja (`ventilation.rs`) | ✅ `Isso53RoomFunctionCell` | — |
| `bezetting.personen_per_m2_default` | Vent-dichtheid override | ja | ❌ **GAT** (mapper hardcodet `null`) | Dichtheid-override onbereikbaar → altijd tabel 4.11 |
| `infiltration_reduction_z` | Infiltratie tabel 4.4 | ja | ✅ `Isso53RoomFunctionCell` (z-dropdown) | — |
| `has_mechanical_supply` | Vent-gate q_v=0 | ja | ✅ `VentilationRow` (Toevoer-checkbox) | — |
| `ventilation_q_v_established` | Vastgestelde q_v (fase 3) | ja (override) | ✅ `VentilationRow` (q_v dm³/s) | — |
| **Construction** | | | | |
| `area` / `u_value` | H_T overal | ja | ✅ `ConstructionRow` | — |
| `boundary_type` | Routeert verliescomponent | ja | ✅ `ConstructionRow` | — |
| `adjacent_room_id` | Buurruimte-resolutie | ja | ✅ `ConstructionRow` | — |
| `adjacent_temperature` | Fallback θ buur (`transmission.rs:145`) | ja (fallback) | ❌ geen direct veld | Fallback alleen via import; UI gebruikt room-resolutie |
| `vertical_position` | Grond f_ig wall/floor (`ground.rs:182`) | ja | ✅ `ConstructionRow` | — |
| `temperature_factor` | f_k override onverwarmd | ja | ⚠️ indirect via `unheatedFactor` sidecar | Per-element override niet direct, wel per-doelruimte |
| `unheated_space` (enum) | f_k tabel 4.2 (`transmission.rs:96`) | ja | ❌ **GAT** | Enum-keuze (15 varianten) onbereikbaar; valt op `temperature_factor`/0.5 |
| `use_forfaitaire_thermal_bridge` | ΔU_TB aan/uit (`transmission.rs:72`) | ja | ❌ **GAT** | Altijd forfaitair aan voor exterior |
| `custom_delta_u_tb` | Custom ΔU_TB | ja | ❌ **GAT** | Eigen koudebrug-waarde onbereikbaar |
| `has_embedded_heating` | Vloerverwarming f_ig (`ground.rs:61`) | ja | ❌ **GAT** (alleen via import) | Embedded heating f_ig-correctie alleen via thermal-import |
| `ground_params.u_equivalent` | Grond-U formule 4.21 | ja | ❌ **GAT** (alleen import/fallback U) | Mapper fallbackt op construction-U |
| `ground_params.ground_water_factor` | f_gw | ja | ❌ **GAT** (default 1.0) | Grondwater ≥1m onbereikbaar → mist 1.15-factor |
| `ground_params.f_ig` / `perimeter` / `depth` | f_ig auto/override 4.22-4.24 | ja | ❌ **GAT** (weggelaten → auto) | Geen override; auto-pad altijd |
| `material_type` | (claim: ΔU_TB) | **nee** | 🗑️ wees-mapped | Gemapt maar nooit gelezen door isso53-calc |
| **Ventilation (project)** | | | | |
| `system_type` | — | ja | ✅ `Isso53BuildingFields` | — |
| `has_heat_recovery` | WTW f_v (`ventilation.rs:132`) | ja | ✅ `VentilationPanel` | — |
| `heat_recovery_efficiency` | η_WTW | ja | ✅ `VentilationPanel` (+ BCRG-selector) | — |
| `supply_temperature` | θ_t WTW (`ventilation.rs:135`) | ja | ✅ `WarmteverliesInstellingen` (achter WTW) | — |
| `has_preheating` | Voorverwarming f_v (`ventilation.rs:144`) | ja | ❌ **GAT** | Voorverwarming-pad onbereikbaar |
| `preheating_temperature` | θ_voorverwarming | ja | ❌ **GAT** | idem |
| `frost_protection` | — (isso53) | **nee** (isso53) | 🗑️ wees t.o.v. isso53 | UI bestaat (isso51-relevant) maar isso53-mapper stuurt `null` |
| **Heating-up** (alle velden) | §4.8 | ja | ✅ `Isso53BuildingFields` | — |
| `infiltration_method` (Known/Unknown) | Tabel 4.5 vs formule 4.31 | ja | ⚠️ deels (`qv10KarClass` ✅, methode-keuze hardcoded `known`) | Unknown-pad onbereikbaar; mapper forceert altijd `known` |

---

## GAT — calc-input zonder UI-veld

Gerangschikt op impact. Elk item: veld → norm-parameter → gevolg → thuishorend component.

### Hoog (stille reken-afwijking op realistische projecten)

1. **`building.source_zone_config`** — tabel 5.1 infiltratie-fractie z.
   *Gevolg:* mapper stuurt het veld niet → kern gebruikt altijd `Other` (z=0.5).
   Gebouwen met gescheiden opwekkers per zone (z=1.0) krijgen een **onderschat
   aansluitvermogen Φ_source** (individueel + collectief, `lib.rs:75-77`).
   *Thuishorend:* `Isso53BuildingFields.tsx` (gebouwniveau), nieuw veld + mapper-regel in `building`-blok.

2. **Koudebrug: `use_forfaitaire_thermal_bridge` + `custom_delta_u_tb`** — ΔU_TB
   forfaitair (`transmission.rs:73 DELTA_U_TB_DEFAULT`) vs eigen waarde.
   *Gevolg:* gebruiker kan de forfaitaire opslag niet uitschakelen of een
   gedetailleerd berekende ΔU_TB invoeren — koudebruggen worden voor elk
   exterieur-element forfaitair opgeslagen, ook waar dat onjuist is.
   *Thuishorend:* `ConstructionRow.tsx` (per grensvlak), extra cel/expand.

3. **`ground_params` (u_equivalent, ground_water_factor, f_ig, perimeter, depth)** —
   grond formule 4.21-4.24.
   *Gevolg:* mapper fallbackt `uEquivalent` op de construction-U en dropt
   perimeter/depth/f_ig → de kern auto-berekent met grove aanname; `f_gw` altijd
   1.0 (hoge grondwaterstand 1.15 onbereikbaar). Grondvloer-verliezen
   structureel onnauwkeurig tenzij via thermal-import gevoed.
   *Thuishorend:* `ConstructionRow.tsx` detail-panel bij `boundary_type === "ground"`.

4. **`construction.unheated_space` (enum, 15 varianten)** — f_k tabel 4.2.
   *Gevolg:* de norm-correcte f_k per onverwarmd-type (kelder, kruipruimte,
   ruimte onder dak, interne verkeersruimte…) is niet kiesbaar; alles valt op
   een handmatige `temperature_factor` of de default 0.5.
   *Thuishorend:* `ConstructionRow.tsx` / `Isso53RoomFunctionCell.tsx`.

### Midden

5. **`ventilation.has_preheating` + `preheating_temperature`** — voorverwarming
   f_v (`ventilation.rs:144-153`). *Gevolg:* het voorverwarmings-/luchtverwarmings-
   pad is volledig onbereikbaar; alleen WTW of natuurlijk. *Thuishorend:* `VentilationPanel.tsx`.

6. **`building.heating_system` (direct, gebouwbreed)** — Δθ₂ grond-f_ig tabel 2.3.
   *Gevolg:* alleen indirect afgeleid uit `default_heating_system ?? rooms[0]`.
   Geen gebouwbreed veld in de ISSO 53-UI → de per-vertrek-dropdown in
   `VentilationRow` stuurt de kern alleen als de eerste room/default toevallig
   meebeweegt. *Thuishorend:* `Isso53BuildingFields.tsx`.

7. **`climate.theta_b_adjacent_building`** — θ_b buurpand (`transmission.rs:178`).
   *Gevolg:* mapper laat het weg → serde-default 15 °C. Vorstvrij (5 °C) of
   onverwarmde stalling (θ_e) buurpanden onbereikbaar → verlies naar buurpand fout.
   *Thuishorend:* `WarmteverliesInstellingen.tsx` klimaat-card (isso53-tak).

8. **`infiltration_method` Unknown-pad** — mapper forceert altijd
   `{ known: { qv10_kar_class } }`. *Gevolg:* projecten zonder gemeten q_v10;kar
   kunnen de formule 4.31-route (Unknown) niet kiezen; de hele Unknown-keten
   (`building_length/width/height`, `building_shape`, `wind_pressure_type`) is
   daardoor dode invoer. *Thuishorend:* `Isso53BuildingFields.tsx` (methode-toggle).

### Laag

9. **`bezetting.personen_per_m2_default`** — dichtheid-override (`ventilation.rs:106`).
   Mapper hardcodet `null`. *Gevolg:* altijd tabel 4.11-dichtheid; afwijkende
   bezettingsgraad alleen via absolute `personen`. *Thuishorend:* `Isso53RoomFunctionCell.tsx`.

10. **`construction.adjacent_temperature`** — fallback θ buur. Geen direct UI-veld;
    alleen room-resolutie via `adjacent_room_id`. Lage prioriteit: room-resolutie
    dekt de normale flow, fallback is import-only.

---

## Wel gedekt (calc-input mét UI)

`heatingUp.*` (volledig blok), `building_shape`, `building_position`,
`wind_pressure_type`, `ventilation_system`, `thermal_mass`, `construction_year`,
`theta_e`, `theta_me`, `qv10KarClass`, room `gebruiks_functie`/`ruimte_type`,
`floor_area`, `height`, `custom_temperature`, `bezetting.personen`,
`infiltration_reduction_z`, `has_mechanical_supply`, `ventilation_q_v_established`,
construction `area`/`u_value`/`boundary_type`/`adjacent_room_id`/`vertical_position`,
onverwarmd-`temperature_factor` (indirect via `unheatedFactor`-sidecar),
ventilation `has_heat_recovery`/`heat_recovery_efficiency`/`supply_temperature`.

---

## Wees (orphan) — UI/mapping zonder calc-effect

1. **`material_type`** — gemapt door `mapConstruction` (`MATERIAL_TYPE_MAP`) en
   editbaar via project-constructie-koppeling, maar **geen niet-test calc-code
   leest het** in isso53-core. Het docstring-commentaar claimt invloed op ΔU_TB,
   maar `transmission.rs` gebruikt de constante `DELTA_U_TB_DEFAULT` ongeacht
   materiaal. → verwarrende invoer (suggereert effect dat er niet is). *Aanbeveling:*
   óf de calc material-afhankelijk maken, óf het veld als puur-documentatie markeren.

2. **`frost_protection`** (t.o.v. isso53) — heeft volledige UI
   (`WarmteverliesInstellingen`, achter WTW) en is relevant voor **ISSO 51**, maar
   de isso53-mapper stuurt altijd `frostProtection: null` (regel 293, met
   commentaar dat het V1-enum niet op `Option<f64>` mapt). In ISSO 53-modus is dit
   dus dode invoer.

---

## Aanbevolen prioritering (PM)

| Prio | Fix | Aard | Component |
|------|-----|------|-----------|
| P1 | `source_zone_config` toevoegen + mappen | 1 dropdown + 1 mapper-regel | `Isso53BuildingFields` |
| P1 | Koudebrug-toeslag (forfaitair toggle + custom ΔU_TB) | per-grensvlak control | `ConstructionRow` |
| P1 | Grond-params (u_equiv, f_gw, perimeter/depth) | detail-panel bij ground | `ConstructionRow` |
| P2 | `unheated_space`-enum keuze | dropdown bij Unheated-grensvlak | `ConstructionRow`/`Isso53RoomFunctionCell` |
| P2 | Voorverwarming (`has_preheating` + temp) | 2 velden | `VentilationPanel` |
| P2 | Gebouwbreed `heating_system` expliciet | dropdown | `Isso53BuildingFields` |
| P2 | `theta_b_adjacent_building` + mapping | 1 veld + mapper | klimaat-card isso53 |
| P3 | Known/Unknown infiltratie-toggle | toggle (ontsluit hele Unknown-keten) | `Isso53BuildingFields` |
| P3 | `personen_per_m2_default` override | 1 veld | `Isso53RoomFunctionCell` |
| — | `material_type` wees opruimen of calc-aansluiten | besluit | — |

**Kernrisico:** de meeste gaten zijn **fout-stil** — de Rust-kern heeft `#[serde(default)]`
op vrijwel elk veld, dus een ontbrekend UI-veld levert geen fout maar een
ongemarkeerde default-waarde (z=0.5, f_gw=1.0, θ_b=15, forfaitair ΔU_TB aan,
Known-pad). Dat is exact het scenario uit de lessons-learned "engineering-aannames
structureel zichtbaar maken": de gebruiker ziet niet dat de aanname bestaat.
