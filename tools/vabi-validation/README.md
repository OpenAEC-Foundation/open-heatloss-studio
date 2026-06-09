# Vabi Elements validation tooling

Standalone Python tooling to use **Vabi Elements** as a reference oracle for our
ISSO 51 warmteverliesberekening. Two scripts:

| Script | Doel |
|---|---|
| `extract_vp.py` | `.vp` (Vabi project) → ons project-JSON (`schemas/v1/project.schema.json`) |
| `compare.py` | onze ISSO 51-uitkomst ⟷ een handmatig gevulde Vabi-referentie, per ruimte |

Pure Python **standaardbibliotheek** (`zipfile`, `sqlite3`, `json`, `argparse`,
`pathlib`, `logging`). Geen pip-dependencies. Getest met Python 3.14.

Achtergrond + datamodel: `docs/vabi-elements-reverse-engineering.md`. De SQL-joins
spiegelen de Rust-importer in `crates/isso51-core/src/import/vabi/mapper.rs`.

---

## 1. Extractor — `extract_vp.py`

```bash
# Naar stdout:
python extract_vp.py "C:\ProgramData\Vabi\Elements\Examples\NL\WV ISSO51 Portiekwoning.vp"

# Naar bestand (+ WARNINGs op stderr):
python extract_vp.py "<pad>\WV ISSO51 Portiekwoning.vp" -o output/portiekwoning_vabi.json -v
```

Elke run print een `[summary]`-regel op stderr met dekking:
`rooms`, `faces`, `explicit_u` (face had een eigen `ConstructionID`),
`palette_fallback` (U afgeleid via type-gebaseerde palette-lookup),
`no_u` (geen U herleidbaar → sentinel 2.5), `warnings`.

### Wat de mapping dekt

| Domein | Bron (SQLite) | Betrouwbaarheid |
|---|---|---|
| Projectinfo | `Project` + `ProjectData` | ✅ betrouwbaar (naam, referentienr., omschrijving) |
| Klimaat θ_e | `ClimateHeatLossCalculation.DesignOutsideTemperatureWinter` | ✅ betrouwbaar (−10 °C) |
| Ontwerptemp θ_i | `Room → …Aspect/Template… → DesignTemperatures.TemperatureDay` | ✅ betrouwbaar |
| Ruimtes | `Room` (`UseInCalculations=1`) | ✅ betrouwbaar (8 / 10 ruimtes) |
| Ruimte-functie | afgeleid uit ruimtenaam + θ_i | ⚠️ heuristisch (naam-keyword → enum) |
| Geometrie (vlak-opp., helling) | `Room → MainFace → CellFace → BuildingPart → Face → FaceGeometryEngine.Area/Slope` | ✅ betrouwbaar |
| Grenstype | `BoundaryConditions.Type` | ✅ betrouwbaar (OutsideAir/OtherBuilding/Unconditioned) |
| Vloeropp. + hoogte | som vloer-faces; hoogte = `TypedVolumeData.Volume / vloeropp.` | ✅ betrouwbaar |
| Opaak Rc → U | `Construction → …StandardConstruction.RcValue`, U=1/(R_si+Rc+R_se) | ⚠️ palette-fallback (zie hieronder) |
| Raam/deur U | `TransparentConstructionData + Frame + Glazing` (frame-% weging) | ⚠️ palette-fallback |
| Constructie-koudebrug | `BuildingPart.PsiThermalBridge` → `custom_delta_u_tb` | ✅ wanneer >0, anders forfaitair |

### Wat geschat / overgeslagen wordt (per WARNING gelogd)

1. **U-waarde per vlak is de zwakke schakel.** In de ISSO 51-voorbeeld-`.vp`
   staat een palette van 9 constructies, maar `BuildingPart.ConstructionID` is
   **NULL voor ±94 % van de vlakken** — Vabi resolveert de constructie per vlak
   pas tijdens de berekening uit de architectural template. Daarom:
   - Heeft een vlak een eigen `ConstructionID` → die wordt gebruikt (`explicit_u`).
   - Anders → **type-gebaseerde palette-lookup** (Floor/Wall/Roof/Window/Door),
     met disambiguatie op grenstype voor wanden (buitenwand → hoogste Rc,
     woningscheidende wand → laagste Rc). Elke fallback geeft een WARNING.
   - Geen herleidbare U → sentinel `2.5 W/m²K` + WARNING.
2. **Geen adjacent-room-koppeling.** Vlakken met een binnen-grens worden als
   `adjacent_building` of `unheated_space` gemapt; er wordt **geen**
   `adjacent_room_id` gezet (de cel-naar-cel-buur-relatie is nog niet
   uitgelezen). Dit is de grootste bron van afwijking t.o.v. de handmatige
   fixture (interne wanden worden verliesvlakken i.p.v. neutrale binnenwanden).
3. **Ventilatie/infiltratie per ruimte** wordt niet uit Vabi gehaald — de
   rekenkern leidt `q_v` af uit BBL-minima. Gebouw-niveau ventilatiesysteem +
   WTW worden wél gemapt.
4. **`qv10` / luchtdichtheid**: de voorbeelden hebben `Qv10Type='FlatRate'` met
   `SpecificQv10=0` → geen bruikbare waarde opgeslagen → fallback `qv10=100` +
   WARNING.
5. **Ruimte-functie** is een naam-heuristiek; controleer bij afwijkende
   ruimtenamen.

### Bekende afwijkingen t.o.v. `tests/fixtures/portiekwoning.json`

De handgebouwde fixture en de Vabi-extractie beschrijven hetzelfde gebouw maar
verschillen substantieel — dit zijn validatie-bevindingen, geen bugs in de tool:

| Aspect | Fixture (handmatig) | Vabi-extractie | Oorzaak |
|---|---|---|---|
| `building_type` | `porch` | `terraced` | Vabi `BuildingShapeType='FlatRate'` ≠ Porch; mapping valt terug op terraced |
| `qv10` | 100 | 100 (fallback) | Vabi `Qv10Type=FlatRate`, geen waarde opgeslagen |
| Ventilatie | `system_c`, geen WTW | `system_d`, WTW 0.5 | Vabi-DB heeft een `LocalHeatRecoverySystemX`-koppeling |
| Interne wanden | `adjacent_room` met θ-buur | `adjacent_building`/`unheated` | geen buur-cel-koppeling in extractor |
| Buitenwand U | 0.36 | 0.361 | **match** (Rc 2.6) ✅ |
| Raam U | 3.2 | 3.2 | **match** (glas-U) ✅ |
| Vloer U | 2.5 | 2.857 | Rc 0.14 + vloer-R_si 0.17; fixture rondt anders |

---

## 2. Harness — `compare.py`

Vergelijkt per ruimte: Φ_transmissie, Φ_ventilatie+infiltratie, Φ_totaal, H_T.

```bash
# A) laat onze engine het project doorrekenen (cargo) en vergelijk:
python compare.py reference_portiekwoning_example.json --project output/portiekwoning_vabi.json

# B) vergelijk tegen een vooraf-berekend result-JSON (geen cargo-build):
python compare.py reference_portiekwoning_example.json --our-result output/portiekwoning_fixture_result.json

# tolerantie overschrijven (default 5% of de waarde in de referentie):
python compare.py reference_portiekwoning_example.json --project <p.json> --tolerance 2.5
```

### Hoe onze tool een project doorrekent (entry point)

`compare.py --project` roept aan:

```bash
cargo run --example calc_from_file -- <project.json>
```

Dat is de canonieke debug-entry (`crates/isso51-core/examples/calc_from_file.rs`)
die het volledige result-JSON naar stdout print. De rekenkern zelf
(`isso51_core::calculate_from_json`) is **niet** aangepast. Per-ruimte velden in
het result:
`rooms[].transmission.phi_t`, `.ventilation.phi_v`, `.infiltration.phi_i`,
`.total_heat_loss`, en `transmission.h_t_*` (gesommeerd tot H_T).

> Voor ISSO 53-projecten bestaat een aparte CLI: `crates/isso53-core/src/bin/isso53-cli.rs`.
> Deze harness richt zich op ISSO 51 (de Vabi-voorbeelden zijn ISSO 51).

### Referentie-formaat

Een `reference_*.json` wordt **handmatig** gevuld uit een Vabi WV-rapport-PDF.
Zie `reference_template.json` (leeg) en `reference_portiekwoning_example.json`
(gevuld). Per ruimte vier grootheden (vermogens in W, H_T in W/K):

```json
{
  "project": "...",
  "source": "Vabi WV-rapport PDF, <bestand>",
  "tolerance_pct": 5.0,
  "rooms": {
    "Woonkamer": {
      "phi_transmission": 783.7,
      "phi_ventilation": 667.7,
      "phi_total": 1391.8,
      "h_t": 30.1
    }
  }
}
```

Ruimte-matching: eerst op id, dan op naam (case-insensitive; numeriek prefix als
`01:` wordt genegeerd). De `_example`-referentie bevat **placeholder**-getallen
(onze eigen engine-output op de fixture) zodat de harness aantoonbaar draait —
vervang deze door echte Vabi-PDF-waarden.

---

## 3. Output

- `output/portiekwoning_vabi.json` — extractie van WV ISSO51 Portiekwoning.vp
- `output/tuinkamerwoning_vabi.json` — extractie van WV ISSO51 Tuinkamerwoning.vp
- `output/portiekwoning_fixture_result.json` — onze engine-output op de
  hand-fixture (voor de snelle `--our-result` demo)

---

## Open punten — naar een volledige importer (Fase 2)

1. **Adjacent-room-koppeling** uit de cel-buur-relatie afleiden, zodat interne
   wanden `adjacent_room_id` + θ-buur krijgen i.p.v. als verliesvlak te tellen.
   Dit is veruit de grootste foutbron.
2. **Per-vlak constructie-resolutie** via de architectural template
   (`ArchitecturalTemplate` → constructie per `BuildingPartType`) i.p.v. de
   huidige type-gebaseerde palette-heuristiek.
3. **Echte Vabi-PDF-referenties** voor beide voorbeelden vullen (Vabi GUI →
   WV-rapport → `pdf_tools` → `reference_*.json`), dan de harness draaien om de
   werkelijke afwijking van onze rekenkern t.o.v. Vabi te kwantificeren.
4. **`qv10`/luchtdichtheid + ventilatie per ruimte** uit Vabi halen wanneer een
   project deze wél opslaat (`Qv10Type='Measured'/'Specific'`).
5. **Gebouwtype-mapping** verfijnen (`BuildingShapeType`/`MiddleRow`/WithHood →
   onze 7 `BuildingType`-varianten); nu valt veel terug op `terraced`.
6. Consolideren met de bestaande Rust-importer (`crates/vabi-importer`): deze
   Python-tool dekt de envelop vollediger (filtert niet op `HasConstruction=1`)
   en kan als referentie dienen om de Rust-importer bij te trekken.
