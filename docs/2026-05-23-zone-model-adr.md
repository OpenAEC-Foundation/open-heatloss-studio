# ADR — Rekenzone als drager van norm-keuze (mixed-use)

**Status:** Draft / spike — niet geïmplementeerd
**Datum:** 2026-05-23
**Auteur:** PM (orchestrator) + Plan-agent
**Vervangt:** impliciete aanname dat `ActiveNorm` project-level is

---

## 1 Doel & scope

| Vraag | Antwoord |
|---|---|
| Welk probleem lost dit op? | Mixed-use gebouwen (bedrijfsverzamelgebouw met bovenwoning, zorginstelling met kantoorvleugel, school met conciërgewoning) krijgen één calc-run waarbij ISSO 51-defaults op woon-ruimtes en ISSO 53-defaults op utiliteits-ruimtes worden toegepast. |
| Wat lost het NIET op? | (1) Single-zone projecten worden niet duurder/complexer — de zone is dan onzichtbaar default. (2) ISSO 53 ≠ ISSO 51 in rekenkern: beide volgen NEN-EN 12831, verschillen zitten in defaults/tabellen/woordkeus. Dit ADR brengt geen rekenkundige unificatie — het routeert de juiste ruimte naar de juiste crate. (3) Tussen-zone warmtestromen via een tweede-orde correctie (zie §3). |
| Wat is een rekenzone? | Een verzameling `Room`s die (a) onder dezelfde norm worden gerekend, (b) dezelfde building-level parameters delen (qv10, building_shape, ventilation_system, …). 1 zone is de default. |
| Wat is GEEN zone? | Een verdieping. Een brandcompartiment. Een huurder. Een gebouwdeel met andere klimaatzone (gebruik aparte projecten). |

**Architectuur-statement:** een `ProjectV3` bevat 1..N rekenzones. Een zone heeft `norm`, building-params en een lijst rooms (via `zone_id`). NEN-EN 12831 rekent per zone; project-output is de som over zones. Geen `Zone`s gespecificeerd = stilzwijgend 1 default zone.

---

## 2 Datamodel

### 2.1 TypeScript

```ts
// types/projectV2.ts — voorstel additieven

export type ActiveNorm = "isso51" | "isso53";

export interface ZoneBuildingParams {
  // Norm-gemene velden (NEN-EN 12831):
  qv10?: number;                    // dm³/s — luchtdoorlatendheid van DEZE zone
  total_floor_area?: number;        // m² A_g van deze zone
  num_floors?: number;
  has_night_setback?: boolean;
  warmup_time?: number;             // uren

  // ISSO 51-only (alleen relevant als norm === "isso51"):
  isso51?: {
    building_type: BuildingType;
    security_class: SecurityClass;
    aggregation_method: AggregationMethod;
    dwelling_class?: DwellingClass | null;
    construction_variant?: ConstructionVariant | null;
    infiltration_method?: Isso51InfiltrationMethod;
  };

  // ISSO 53-only (alleen relevant als norm === "isso53"):
  isso53?: {
    building_shape: Isso53BuildingShape;
    building_position: Isso53BuildingPosition;
    wind_pressure_type: Isso53WindPressureType;
    thermal_mass: Isso53ThermalMass;
    ventilation_system: Isso53VentilationSystem;
    infiltration_method?: Isso53InfiltrationMethod;
  };
}

export interface Zone {
  id: string;                        // ULID
  name: string;                      // "Woningen", "Kantoorvleugel", default "Hoofdzone"
  norm: ActiveNorm;
  building: ZoneBuildingParams;
  ventilation: VentilationConfig;    // norm-gemene shape (system_type, has_heat_recovery, …)
  heating_up?: HeatingUpConfig;      // optioneel — overschrijft project-default
}

export interface ProjectV3 {
  schema_version: 3;
  info: ProjectInfo;
  site: {
    postcode?: string;
    location?: string;
    construction_year?: number | null;
    building_height_m?: number | null;
    building_length_m?: number | null;
    building_width_m?: number | null;
    latitude?: number | null;
    longitude?: number | null;
  };
  climate: DesignConditions;         // 1× θ_e/θ_b voor heel project (locatie-bepaald)
  zones: Zone[];                     // ≥ 1
  rooms: Room[];                     // GLOBAAL, elke room.zone_id wijst naar Zone
}

export interface Room {
  id: string;
  name: string;
  zone_id: string;                   // NIEUW in v3 (silent default in migratie)
  // ... overige velden ongewijzigd
}
```

### 2.2 Veld-verhuizing matrix

| Veld | V2 locatie | V3 locatie | Reden |
|---|---|---|---|
| `name`, `address`, `client`, `engineer` | `info` | `info` | project-niveau |
| `postcode`, `location`, `construction_year` | `sharedExtra` | `site` | 1 gebouw = 1 set |
| `building_height/length/width` | `building` | `site` | fysieke afmeting, niet norm-keuze |
| `latitude`/`longitude` | (nieuw) | `site` | klimaat-bepalend, niet zone-bepalend |
| `theta_e`, `theta_b_*`, `wind_factor` | `climate` | `climate` (project) | locatie-bepaald |
| `building_type` (51) | `building` | `zone.building.isso51` | per-zone |
| `building_shape/position/wind_pressure` (53) | `isso53Building` | `zone.building.isso53` | per-zone |
| `qv10`, `total_floor_area`, `num_floors` | `building` | `zone.building` | zones kunnen ver. luchtdichtheid hebben |
| `security_class`, `aggregation_method` | `building` | `zone.building.isso51` | 51-only |
| `thermal_mass`, `ventilation_system` (53) | `isso53Building` | `zone.building.isso53` | 53-only |
| `has_night_setback`, `warmup_time` | `building` | `zone.building` | per-zone (bedrijf 's nachts uit, woning niet) |
| `ventilation` (system_type, WTW) | project | `zone.ventilation` | per-zone (commerciële WTW vs woon-WTW) |
| `infiltration_method` | project | `zone.building.*` | infiltratie is zone-zaak |
| `rooms[]` | project | `rooms[]` + `room.zone_id` | flat lijst, koppeling via FK |
| `norm` (ActiveNorm) | project | `zone.norm` | **kern-verhuizing** |

**Argumentatie voor `rooms[]` flat + `zone_id` (i.p.v. `zone.rooms[]` nested):**
- Lijstweergave in UI blijft 1 array — geen `flatMap` overal
- Drag-drop tussen zones = pure `zone_id` mutatie, geen array-move
- IFC-import schrijft 1 lijst, zone-assignment is post-processing
- Adjacent-room lookup via `find(r => r.id === target)` blijft O(1) lookup-table-bouw

### 2.3 JSON — concrete voorbeelden

**Single-zone (default — 99% van projecten):**

```json
{
  "schema_version": 3,
  "info": { "name": "Voorbeeldwoning Tiel" },
  "site": { "construction_year": 2010, "building_height_m": 9.5 },
  "climate": { "theta_e": -10, "theta_b_residential": 17 },
  "zones": [{
    "id": "zone_default",
    "name": "Hoofdzone",
    "norm": "isso51",
    "building": {
      "qv10": 100, "total_floor_area": 120, "num_floors": 2,
      "isso51": { "building_type": "terraced", "security_class": "b",
                  "aggregation_method": "vabi_compat" }
    },
    "ventilation": { "system_type": "system_c", "has_heat_recovery": false }
  }],
  "rooms": [
    { "id": "r1", "name": "Woonkamer", "zone_id": "zone_default", "...": "..." }
  ]
}
```

**Mixed-use (bedrijfsverzamelgebouw + bovenwoning):**

```json
{
  "schema_version": 3,
  "info": { "name": "Hoofdstraat 5 - mixed" },
  "site": { "construction_year": 2018, "building_height_m": 12.0 },
  "climate": { "theta_e": -10, "theta_b_residential": 17 },
  "zones": [
    {
      "id": "z_kantoor",
      "name": "Kantoorvleugel",
      "norm": "isso53",
      "building": {
        "qv10": 420, "total_floor_area": 216, "num_floors": 2,
        "isso53": { "building_shape": "meerlaags", "building_position": "meerlaagsOnder",
                    "wind_pressure_type": "meerlaagsStandaard", "thermal_mass": "gemiddeld",
                    "ventilation_system": "systemD" }
      },
      "ventilation": { "system_type": "system_d", "has_heat_recovery": true,
                       "heat_recovery_efficiency": 0.85 }
    },
    {
      "id": "z_woning",
      "name": "Bovenwoning",
      "norm": "isso51",
      "building": {
        "qv10": 80, "total_floor_area": 95, "num_floors": 1,
        "isso51": { "building_type": "stacked", "security_class": "b",
                    "aggregation_method": "vabi_compat" }
      },
      "ventilation": { "system_type": "system_c", "has_heat_recovery": false }
    }
  ],
  "rooms": [
    { "id": "r_office_1", "name": "Kantoorruimte 1.01", "zone_id": "z_kantoor", "...": "..." },
    { "id": "r_living", "name": "Woonkamer", "zone_id": "z_woning", "...": "..." }
  ]
}
```

### 2.4 Backwards compat — migratie V2 → V3

| Scenario | Migratie-strategie |
|---|---|
| V2 project zonder `zones` array | `migrate_v2_to_v3()` creëert silent 1 zone uit `building` + `norm` + `ventilation`; alle rooms krijgen `zone_id = "zone_default"` |
| V2 project met `norm === "isso53"` + `isso53Building` sidecar | sidecar verhuist naar `zones[0].building.isso53`, `isso53Rooms` sidecar wordt geconsumeerd in default-temperatuur lookup (geen aparte map meer) |
| Schema-versie | Bump `schema_version: 2 → 3`. Frontend-only migratie bij load (voorkeur — backend ziet alleen v3) |
| Lege defaults | Nieuwe projecten krijgen direct 1 zone met `norm` uit `NormChoiceModal` (ongewijzigd UX) |

---

## 3 Rust calc-core impact

### 3.1 Crates-overzicht

| Crate | Wijziging | Omvang |
|---|---|---|
| `crates/isso51-core` | Niets aan rekenkern. Eigen `Project`/`Building` blijft entry-point. | Groen |
| `crates/isso53-core` | Idem — eigen `Project`/`Building` blijft. | Groen |
| `crates/openaec-project-shared` | **Hier landt het ProjectV3-model + zone-splitter.** Krijgt `split_into_zone_inputs(&ProjectV3) -> Vec<(ZoneRef, NormSpecificInput)>` waar `NormSpecificInput = Isso51Input \| Isso53Input`. | Nieuw |
| Nieuwe `isso-multi-zone` (of in `openaec-project-shared`) | Orchestrator die per zone de juiste crate aanroept en resultaten aggregeert. | Nieuw klein |

**Kernpunt:** **isso51-core en isso53-core blijven onaangetast.** Ze rekenen wat ze rekenen, op 1 input. De zone-loop zit erbuiten in de orchestrator-laag. Vabi-golden tests blijven 1-op-1 geldig (een single-zone v3-project wordt door de splitter omgezet naar exact dezelfde input als nu).

### 3.2 Berekeningsflow

```
ProjectV3
   │
   ▼
split_into_zone_inputs()  ── per zone één call ──┐
   │                                              ▼
   │                                    isso51_core::calculate(zone_input)
   │                                              │
   │                                    isso53_core::calculate(zone_input)
   │                                              │
   ▼                                              ▼
aggregate_project_result()  ◄────────  Vec<ZoneResult>
   │
   ▼
ProjectResult { zones: [...], project_totals: {...} }
```

### 3.3 Adjacent-room ΔT over zone-grens

| Optie | Voor | Tegen | Advies |
|---|---|---|---|
| A: zone-grens = `BoundaryType::AdjacentBuilding` (f_k uit andere-gebouw tabel) | Simpel, hergebruikt bestaande boundary | Fysisch fout — andere zone is wel verwarmd | Nee |
| B: zone-grens = `BoundaryType::AdjacentRoom` met `theta_target` van de andere zone's gemiddelde | Fysisch juist (NEN-EN 12831 §6.3.4) | Vereist coupling tussen zone-runs (2-pass) | **Ja** — start met statische coupling: ronde 1 bepaalt zone-gemiddelde θ_i, ronde 2 lost adjacent-room over zone-grens op |
| C: user vult expliciet `adjacent_room_temperature_override` in bij grens-constructies | Geen coupling nodig | UX-last, foutgevoelig | Backup voor edge-cases (formule 4.9/4.10 spoor) |

**Voorstel:** start v3 zonder zone-coupling — adjacent-room over zone-grens valt onder hetzelfde open spoor als binnen zone (formule 4.9/4.10). Voeg in fase 4 een 2-pass orchestrator toe wanneer mixed-use projecten in de hand komen.

### 3.4 Impact Vabi-golden tests

| Test | Risico | Mitigatie |
|---|---|---|
| `vabi_bedrijfsruimte4` (single-zone 53) | Geen — splitter geeft identieke input | Migratie-test toevoegen die fixture upgrade naar v3 en byte-identieke result eist |
| `dr_kantoorwest` (single-zone 53) | Idem | Idem |
| Toekomstige mixed-use fixtures | Niet beschikbaar bij Vabi (waarschijnlijk) | Maak eigen synthetic fixture: 2 zones identiek aan bestaande fixtures naast elkaar, verifieer dat totaal = som |

---

## 4 UI-impact

### 4.1 Default = onzichtbaar

| State | UI gedrag |
|---|---|
| 1 zone | Zone-tabs verborgen. AlgemeenTab toont zone-velden direct (alsof het project-niveau is). NormChoiceModal blijft bij New. |
| ≥ 2 zones | Zone-tabs verschijnen bovenaan AlgemeenTab. Elke ruimte krijgt zone-badge + dropdown in zijbalk. |

**Trigger voor meer-zones:** expliciete user-actie "Tweede zone toevoegen" in AlgemeenTab → toevoegen-modal vraagt naam + norm. Geen automatische multi-zone uit IFC (voorlopig).

### 4.2 Norm-toggle locatie

| Plek | Hoe |
|---|---|
| AlgemeenTab → zone-header | Pill-toggle "ISSO 51 / ISSO 53" per zone. Verandering binnen 1 zone triggert `normSwitch.ts`-conversie (zelfde logica als nu, maar gescoped op zone-rooms). |
| Backstage "Nieuw" | NormChoiceModal blijft — kiest norm voor `zones[0]`. |

### 4.3 Ruimte → zone assignment

| Mechanisme | UX | Implementatie |
|---|---|---|
| Default: nieuwe ruimte erft active zone | "Nieuwe ruimte" knop in zone-tab → `zone_id = activeZoneId` | Zustand-store krijgt `activeZoneId` |
| Verplaatsen via dropdown | Per ruimte in RoomEditor rechterpaneel: "Zone: [Hoofdzone ▾]" | `updateRoom(id, { zone_id })` |
| Batch-assign | In Modeller: selecteer N kamers → context-menu "Verplaats naar zone…" | Multi-select hook |
| Drag-drop | Optioneel later — sleep ruimte vanuit ruimtelijst naar zone-tab-header | Niet in MVP |

### 4.4 normSwitch.ts wijzigingen

| Huidige signature | Nieuwe signature |
|---|---|
| `deriveIsso53BuildingFromIsso51(project) → Isso53BuildingState` | `deriveIsso53BuildingFromIsso51(zone) → ZoneBuildingParams["isso53"]` |
| `deriveIsso53RoomsFromIsso51(project)` | `deriveIsso53RoomsFromZone(zone, allRooms)` — alleen rooms uit die zone |
| Globale norm-set | Per-zone norm-set |

Back-up envelope krijgt `zone_id` veld zodat undo per zone werkt.

---

## 5 Migratie / rollout

### 5.1 PR-volgorde (5 stuks, los mergeable)

| # | PR | Scope | Risico | Mergeable zonder volgende? |
|---|---|---|---|---|
| 1 | **schema-v3 read-side** | `ProjectV3` types + `migrate_v2_to_v3()` + `projectV2Migration.ts`-update. Store leest v2/v3, schrijft v2. Geen UI-zichtbare wijziging. | Laag | Ja |
| 2 | **store-shape rewrite** | Store interne representatie wordt `ProjectV3` (1 zone forced). Selectors `selectActiveZone()`, `selectZoneById()`. UI blijft tegen oude shape draaien via selectors. | Middel | Ja |
| 3 | **calc-core orchestrator** | `openaec-project-shared::zones::split_and_calc()` — accepteert v3, splitst per zone, roept isso51-core/isso53-core aan, aggregeert. Single-zone path = byte-identiek aan huidige flow (Vabi-golden tests blijven groen). | Middel | Ja — schrijft naar nieuw endpoint, oude blijft |
| 4 | **UI multi-zone** | AlgemeenTab krijgt zone-tabs (alleen zichtbaar bij ≥2), RoomEditor krijgt zone-dropdown, "Tweede zone toevoegen" knop. Norm-toggle verhuist naar zone-header. | Hoog (UX) | Nee — vereist PR 1-3 |
| 5 | **schema-v3 write-side** | Frontend schrijft v3 JSON, backend accepteert v3 native, schemas/v1 → schemas/v3 export. Verwijdert v2-write-pad. | Laag (alles werkt al) | Nee |

### 5.2 Schema-versie strategie

| Optie | Voor | Tegen | Advies |
|---|---|---|---|
| Additief op v2 (zones-veld optioneel toevoegen) | Backwards compat triviaal | Schema wordt diffuus, twee waarheden | Nee |
| Bump naar v3, migrate-on-load | Schoon model, één bron van waarheid | Migratie-code nodig (1 functie, klein) | **Ja** |
| Bump naar v3, schrijf v2 + v3 dual | Zekerheid bij rollback | Dubbele write-pad onderhouden | Nee — git is je rollback |

### 5.3 Sanity-checks

- Migratie-roundtrip-test: laad 10 bestaande Vabi-fixtures + .ifcenergy uit user-docs → migrate v2→v3 → calc → result byte-identiek aan v2-result
- Schema-versie in JSON-bestand zelf (`"schema_version": 3`) niet alleen impliciet
- `currentLocalPath` bestanden zonder schema_version → fallback "assume v1/v2", migrate on load

---

## 6 Open vragen / risico's

| # | Vraag | Voorstel |
|---|---|---|
| 6.1 | Rapport-output bij meerdere zones | **1 PDF, hoofdsectie per zone** (titel = `zone.name + norm`), eindsectie "Projectoverzicht" met som-tabel. `isso53ReportBuilder.ts` en `reportBuilder.ts` krijgen `zone`-parameter; root-builder loopt zones af. |
| 6.2 | Vabi-import bij mixed-use | Vabi-zones zijn ventilatie-zones, onze zones zijn norm-zones. Voorstel: import maakt 1 zone (kies norm via dialog), user splitst handmatig. Auto-detectie via Vabi-zone-mapping is open spoor. |
| 6.3 | IFC-energy export — heeft IFC zone-concept? | Ja, `IfcZone` koppelt `IfcSpace`s. Onze `Zone` mapt 1-op-1 op `IfcZone` met custom `PSet_HeatLossCalculation { Norm: "ISSO51" \| "ISSO53" }`. Export: 1 `IfcZone` per onze zone. Import: groepeer spaces per IfcZone, vraag user om norm-toewijzing als PSet ontbreekt. |
| 6.4 | Climate per zone of project? | **Project.** θ_e wordt bepaald door geografische locatie. Beide normen gebruiken identieke buitentemperatuur. θ_b_residential vs θ_b_non_residential: laat zone op basis van norm de juiste kiezen uit de project-`climate`. |
| 6.5 | Construction year per zone? | **Project (site).** 1 fysiek gebouw = 1 bouwjaar. Renovatie-deel met ander bouwjaar = apart project (edge case, accepteer dat). |
| 6.6 | Heating-system main-room logic (ISSO 51 §4.3 two-pass)? | Main-room-pass scoped binnen zone — elke ISSO 51-zone heeft eigen main room. Geen cross-zone main-room logic. |
| 6.7 | Single-source-of-truth voor zone-defaults | Move `DEFAULT_ISSO53_BUILDING` + ISSO 51-defaults naar `zone-defaults.ts` met `defaultZoneFor(norm: ActiveNorm): ZoneBuildingParams`. |
| 6.8 | Adjacent room over zone-grens (uitgesteld) | Risico: gebruiker maakt 2 zones met deelconstructie ertussen, krijgt onverwacht resultaat. **Mitigatie:** validator waarschuwt bij `BoundaryType::AdjacentRoom` waarbij target-room in andere zone zit; verzoek expliciete `theta_target_override`. |

---

## Aanbeveling

| Aspect | Advies |
|---|---|
| **Doorgaan?** | Ja — model is helder, incrementeel pad veilig, 99% van projecten merkt niks van de wissel. |
| **Volgorde** | PR 1-2-3 in willekeurige sprint, PR 4 als eerste mixed-use-klant zich meldt, PR 5 cleanup. |
| **Eerst doen** | Niets — eerst de 2 open sporen uit `warmteverlies_latest.md` (adjacent-room ΔT formule 4.9/4.10, Unknown-pad Vabi-compat) afronden. Zone-werk bouwt voort op stabiele calc-core. |
| **Niet doen** | Calc-core unificeren (isso51-core + isso53-core mergen). Te risicovol, geen winst — beide volgen 12831 maar verschillen in 30+ tabellen. |

---

### Critical Files for Implementation

- `frontend/src/types/projectV2.ts`
- `frontend/src/lib/projectV2Migration.ts`
- `frontend/src/store/projectStore.ts`
- `crates/openaec-project-shared/src/lib.rs` (nieuwe zone-splitter + orchestrator)
- `frontend/src/lib/normSwitch.ts`

---

### Eerste TitleBar-cleanup (samen gecommit met deze ADR)

De eerste UI-stap is in dezelfde commit meegenomen:

- Norm-badge weggehaald uit `TitleBar.tsx` + `TitleBar.css`
- Norm-switch entry verplaatst naar `Backstage.tsx` onder Voorkeuren als `MenuItem "Norm wisselen (ISSO 51 ↔ 53)"`
- Modal-state blijft op `AppShell` (Backstage sluit zichzelf → modal moet onafhankelijk leven)
- i18n-key `backstage:normSwitchEntry` toegevoegd in NL + EN

Geen calc-core wijzigingen, geen schema-impact. Effect: rust in chrome + minder uitnodigend om norm globaal te wisselen.
