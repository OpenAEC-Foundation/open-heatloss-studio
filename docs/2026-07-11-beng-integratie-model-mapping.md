# BENG-integratie — gebouwmodel-mapping (ontwerp)

**Datum:** 2026-07-11 · **Status:** ontwerp, nog geen code · **Auteur:** PM/architectuur

## Doel

Open-heatloss-studio wordt **de nieuwe "energy studio"**: één Rust-tool die zowel
de ISSO 51/53-warmteverliesberekening als de **NTA 8800-BENG-berekening** doet.
De BENG-engine van collega John Heikens (`derden/open-energy-studio`, TypeScript,
~1-5% van gecertificeerde Uniec op woningbouw) wordt **geport naar Rust**
(`crates/nta8800-core`) en beide engines lezen uit **één uitgebreid gebouwmodel**.

Dit doc beschrijft dat gedeelde model — het fundament vóór er code komt.

## Wat de twee modellen nu hebben

| Aspect | Heatloss (Rust, bestemming) | Energy (TS, bron) |
|---|---|---|
| Hiërarchie | `Project→Building→Room→ConstructionElement` | `Project→Zone→Surface→Window` |
| Geometrie | 2D-roompolygonen in de frontend (`modeller/types.ts`) + `deriveRoomGeometry.ts` | oppervlakken direct als m² |
| Constructie | `u_value` direct (+ afgeleiden) | laag-opbouw → `rcValue`/`uValue` |
| Randvoorwaarde | rijk: `boundary_type`, `temperature_factor`, `adjacent_room_id`, `ground_params`, `vertical_position` | simpel: `SurfaceType` |
| **Oriëntatie** | — (niet expliciet; **afleidbaar** uit 2D-wandgeometrie + noordhoek) | **compas N/O/Z/W op vlak + raam** |
| **g-waarde (ZTA)** | — | **op raam** (nodig voor zonwinst) |
| **Volume** | afleidbaar (`floor_area × height`) | expliciet op zone |
| Systemen | `HeatingSystem` (warmteverlies-oriëntatie) | heating COP, ventilatie WTW+SFP, koeling EER, tapwater η + zonneboiler |
| Hernieuwbaar | — | PV (kWp/oriëntatie/tilt), zonthermisch |
| Gebruik/klimaat | ontwerpcondities (θ_e, etc.) | NTA 8800 maandklimaat (Tabel 17.1/17.2) + gebouwfunctie/gebruiksuren |

**Overlap ~60%** (geometrie, oppervlakken, U-waarden, zones). Elk mist wat de ander
domein-eigen heeft. De warmteverliescalc heeft géén zonwinst nodig (vandaar geen
oriëntatie/g-waarde); BENG heeft die juist essentieel nodig.

## Ontwerp: één uitgebreid Rust-model, twee calc-ingangen

**Principe:** het heatloss `Project/Building/Room/ConstructionElement` blijft de
**ruimtelijke basis**. We voegen de BENG-benodigde velden **additief** toe
(`#[serde(default)]` — bestaande projecten/fixtures blijven geldig, geen regressie
op de isso51/53-goldens). Twee entry-points lezen hetzelfde model:

```
model (uitgebreid, gedeeld)
  ├── isso5x-calc  → warmteverlies (bestaand, ongewijzigd gedrag)
  └── nta8800-calc → BENG 1/2/3 + TO-juli + label (nieuw, geport)
```

### Toe te voegen velden (additief, `serde(default)`)

1. **Op `ConstructionElement`** (exterieure vlakken + ramen):
   - `orientation: Option<Orientation>` — enum N/NO/O/ZO/Z/ZW/W/NW/horizontaal.
     **Bij voorkeur afgeleid** uit de 2D-wandgeometrie + een gebouw-noordhoek
     (frontend `deriveRoomGeometry`), met dit veld als expliciete override.
   - `g_value: Option<f64>` — ZTA, alleen zinvol op transparante elementen.
2. **Op `Building`**: `north_angle: Option<f64>` (graden) — referentie voor de
   oriëntatie-afleiding.
3. **Op `Room`/`Zone`**: `volume` is al afleidbaar (`floor_area × height`); expliciet
   veld alleen als override nodig blijkt.
4. **Nieuw `Project.energy: Option<EnergyInput>`** — het BENG-invoerblok dat
   warmteverlies niet kent:
   - `building_function` (woning/utiliteit-subtypes → NTA 8800 gebruiksprofielen).
   - `systems`: heating (COP/dekking), ventilation (WTW-rendement + SFP), cooling
     (EER), hot_water (η + zonneboilerfractie).
   - `renewables`: PV (kWp/oriëntatie/tilt), solar_thermal (opp/oriëntatie/tilt).
   - Klimaat/gebruik komt grotendeels uit NTA 8800-tabellen (in de engine, niet de
     invoer) — vgl. hoe isso53 de klimaattabellen intern heeft.

De meeste van deze systeem-/hernieuwbaar-structs bestaan al 1-op-1 in
`open-energy-studio/src/core/energy/types.ts` — overneembaar als Rust-structs.

## Frontend-consequenties (de nieuwe energy studio)

- **Oriëntatie**: primair afleiden uit de bestaande 2D-modeller + een noordhoek-
  instelling; geen dubbele invoer. g-waarde: één veld erbij op het raam-dialog.
- **BENG-invoerpaneel**: systemen + hernieuwbaar. De UI hiervoor bestaat al vrijwel
  compleet in open-energy-studio (dialogs voor heating/cooling/hotwater/ventilatie/
  PV/zonthermisch) — porten/mappen i.p.v. opnieuw bouwen.
- **Resultaten**: BENG 1/2/3 + TO-juli + label naast de bestaande warmteverlies-
  output.

## Grondwaarheid & vangrail (migratie)

De 3 gecertificeerde referentieprojecten (`training-data/*.oes.json` met
`meta.uniecReference` uit Uniec 3.3.7.0) zijn de golden-fixtures voor `nta8800-core`
— exact de discipline van de isso53 §6.2-golden.

- **Nu (gedaan):** vitest-vangrail in open-energy-studio (`bengValidation.test.ts`)
  meet de TS-engine tegen deze referenties. Woningen ~1-5%, utiliteit-gaten benoemd.
- **Port-fase:** transcribeer de 3 project-inputs naar het uitgebreide Rust-model
  (of schrijf een `.oes.json → model`-mapper) en pin de `uniecReference` als expected
  in `crates/nta8800-core/tests/golden.rs`. De Rust-engine is pas "af" als hij dezelfde
  referenties binnen dezelfde toleranties haalt.

## Fasering

| Fase | Wat | Afhankelijk van |
|---|---|---|
| **P1** (dit doc) | model-mapping-ontwerp | — |
| **P2** | Rust-model additief uitbreiden (oriëntatie/g-waarde/north/energy-blok) + oriëntatie-afleiding; `nta8800-core` scaffolden met de 3 golden-fixtures (rood) | P1 |
| **P3** | engine module-voor-module porten (transmissie→ventilatie→winsten→vraag→primair→BENG-indicatoren→TO-juli), fixtures groen | P2 |
| **P4** | frontend: g-waarde/oriëntatie exposen + BENG-systemenpaneel porten + resultaten tonen | P3 |
| **P5** | 5 benoemde gaten dichten (utiliteit-verlichting, koelvraag, tapwater-klein, TO-juli, pass/fail-vlag), toleranties aanscherpen; rebrand → "energy studio" | P3/P4 |

## Open beslissingen

- **`.oes.json`-input → Rust-model:** eenmalige transcriptie per referentie, of een
  herbruikbare mapper? Mapper is meer werk maar houdt de import-route (UNIEC3/VABI)
  in beeld voor later.
- **Oriëntatie-afleiding:** volledig geometrisch (2D-wand → azimut) vs. handmatig
  per vlak. Geometrisch is eleganter en sluit aan op het bestaande modeller-model,
  maar vereist een betrouwbare noordhoek + omgang met interne wanden.
- **Governance bron:** John's LGPL-code wordt geport; auteurschap/licentie
  respecteren in `nta8800-core` (LGPL-herkomst vermelden).
