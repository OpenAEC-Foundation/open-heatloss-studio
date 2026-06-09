# Vabi Elements — Reverse Engineering & Validatie-oracle

> Doel: Vabi Elements (demo, v0.9.x, build okt 2025) gebruiken als **referentie-oracle** om
> onze ISSO 51 warmteverliesberekening te valideren. Dit document legt het projectformaat,
> het datamodel en de gekozen validatiestrategie vast.
>
> Analyse uitgevoerd: 2026-06-09. Bron-installatie: `C:\Program Files\Vabi\Elements\`.

## 1. Samenvatting

| Aspect | Bevinding |
|---|---|
| App-type | .NET (WinForms + Ogre3D), NHibernate ORM, IDP-licentie via `idp.vabi.nl` |
| Projectformaat `.vp` | **ZIP-archief** met `Elements.sqlite3` (SQLite-DB) + `cfg` |
| Datamodel | **361 tabellen**, volledig leesbaar met standaard `sqlite3` |
| Berekende resultaten | **Niet** in `.vp` opgeslagen — `VariantResult.SerializedData = NULL`; Vabi herberekent bij openen |
| Rekenkern (binary) | `Vabi.Elements.ExternalSystems.CalculationEngines.HeatLossCalculation.dll` (decompileerbaar) |
| Referentieprojecten | `WV ISSO51 Portiekwoning.vp`, `WV ISSO51 Tuinkamerwoning.vp` (in `ProgramData\...\Examples\NL`) |
| Juridisch | Data-laag interoperability op zelf-bezeten bestanden — geen binary-cracking nodig |

## 2. Bestandsformaat `.vp`

```
WV ISSO51 Portiekwoning.vp  (ZIP, PK\x03\x04)
├── Elements.sqlite3   ← volledige projectdatabase (SQLite 3)
└── cfg                ← klein config-blob
```

Uitpakken + lezen:
```python
import zipfile, sqlite3, tempfile, os
with zipfile.ZipFile("project.vp") as z:
    z.extract("Elements.sqlite3", tmpdir)
con = sqlite3.connect(os.path.join(tmpdir, "Elements.sqlite3"))
```

## 3. Datamodel — WV-kritische tabellen

NHibernate-stijl: table-per-subclass. Base-class `ResultBase(ID)` met subclass-FK-tabellen
(`RoomResult`, `GroupResult`, `BuildingPartResult`, `ProjectResult`, `VariantResult`).

### Inputs (volledig extraheerbaar)

| Domein | Tabel(len) | Sleutelvelden |
|---|---|---|
| Gebouw | `Building` | `GrossBuildingVolume`, `GrossBuildingGroundArea`, `UsageArea`, `HabitableSpaceArea` |
| Ruimtes | `Room` (8 stuks) | `RoomNumber`, `Name`, `VolumeInfoID`, `VentilationID`, `CellID`, `UseInCalculations` |
| Klimaat | `ClimateHeatLossCalculation` | `DesignOutsideTemperatureWinter` (Portiek: **−10 °C**) |
| Ontwerptemp | `DesignTemperatures` | `TemperatureDay`/`TemperatureNight` (Woonkamer 20/15, badkamer-type 24/24) |
| Randvoorw. | `BoundaryConditions`, `BoundaryTemperatures`, `GroundHeatConduction` | `AdjacentUnheatedSpaceType`, `GroundHeatConductivity`, `GroundTemperature` |
| Constructies | `ConstructionData` → `OpaqueConstructionData` / `TransparentConstructionData` | `Type` (Floor/SlopingRoof/Wall/Window/Door) |
| Opaak Rc | `StandardConstruction` | **`RcValue`** (vloer 0.14, dak/wand 2.6), `Thickness`, `ThermalMass` |
| Transparant | `TransparentConstructionData`, `Glazing` (27), `Frame` (9) | `Psi` (0.08), `GFractionGlazing*`, `FrameID`, `GlazingID` |
| Geometrie | `BuildingPart` → `AreaInfo`/`PerimeterInfo` → `TypedAreaData` (416), `TypedPerimeterData` (324), `TypedVolumeData` | `AreaPriority`/`AreaSecondary`, `Volume` |
| 3D-mesh | `Cell`, `CellFace`, `Face`, `FaceGeometryEngine` (Area/Perimeter), `Vertex`, `VertexNode` | exacte vlakgeometrie |
| Ventilatie | `Ventilation`, `VentilationData`, `VentilationFlowRatesHeatLoss`, `AirExchange` | `TypeForHeatLoss`, `FlowRatesHeatLossID` |
| Infiltratie | `Infiltration`, `InfiltrationFlowRateHeatLoss`, `BuildingPart` | `TypeForHeatLoss`, `SpecificInfiltrationArea/Perimeter` |

**U-waarde-keten (opaak):** `ConstructionData.OpaqueConstructionDataID` → `OpaqueConstructionData.StandardConstructionID`
→ `StandardConstruction.RcValue` → U = 1/(Rc + Rsi + Rse).
**U-waarde-keten (raam):** `TransparentConstructionData` → `Frame` + `Glazing` (+ `Psi`-koudebrug).

### Outputs (NIET in bestand)

`VariantResult.SerializedData` is `NULL` in de meegeleverde voorbeelden. Result-tabellen zijn
lege FK-skeletten. **Conclusie: Φ/H_T per ruimte staan niet in de `.vp` — Vabi rekent on-open.**

## 4. Headless CLI — `ElementsConsole.exe`

Argument-grammatica (uit `Vabi.Elements.Presenters.Console.dll`):
```
ElementsConsole.exe -import <file> [-calculate <0|1>] [-export <file>] [-layers ...] [-names 0|1] [-doors 0|1]
```
Ondersteunde import/export-extensies: `vp` (project), `dxf/dwg` (CAD), `skp/skb` (Sketchup), `tst` (testdata).

| Test (2026-06-09) | Resultaat |
|---|---|
| `-import in.vp -export out.vp` (geen calc) | ✅ "Importing → Exporting → succes" |
| `-import in.vp -calculate 1 -export out.xml` | ⚠️ "Running project" → **geen output**, geen terugschrijf |
| Resultaat in bron-`.vp` na calc | `SerializedData` blijft NULL |

**Verdict:** rekenkern is **licentie-gated/no-op onder de demo** (`modules.lic` is versleuteld
ZIP-blob, niet leesbaar). Headless-oracle valt af → **PDF-fallback**.

## 5. Validatiestrategie

```
┌─ Vabi .vp ──────────┐        ┌─ onze tool ─────────┐
│ SQLite-inputs       │──(A)──▶│ gereconstrueerd     │
│ geom/U/klimaat/temp │        │ project (JSON)      │
└─────────────────────┘        └──────────┬──────────┘
        │                                  │ run ISSO 51
        │ (B) Vabi GUI → WV-rapport PDF    ▼
        ▼                          onze Φ_T, Φ_V, H_T
   ref Φ_T, Φ_V, H_T  ◀────(C) per-ruimte diff────▶
```

- **(A) Input-extractor** — `.vp` → onze project-JSON. Volledig haalbaar uit SQLite.
- **(B) Referentie-output** — Vabi GUI openen → warmteverlies-rapport (PDF) van de 2 ISSO 51-voorbeelden
  → getallen extraheren via `pdf_tools`. (Console-route afgevallen, zie §4.)
- **(C) Vergelijking** — per ruimte: Φ_transmissie, Φ_ventilatie/infiltratie, Φ_totaal, H_T;
  tolerantie + afwijkingsrapport. Afwijkingen = bug-kandidaten in onze ISSO 51-implementatie.

## 6. Gefaseerde aanpak

| Fase | Inhoud | Status |
|---|---|---|
| 0 | Format/datamodel-analyse + console-feasibility (dit document) | ✅ Klaar |
| 1 | Input-extractor (`.vp`→JSON) + vergelijkings-harness voor 2 ISSO 51-refs | ⬜ Te delegeren |
| 2 | Extractor uitbouwen → volwaardige `.vp`-importer (gebruikers migreren Vabi-projecten) | ⬜ |
| 3 | `HeatLossCalculation.dll` decompileren (ILSpy/dotPeek) → exacte ISSO 51-formules toetsen | ⬜ Diepe RE |
| — | Selectieve schema-documentatie als ISSO 51/53-modelleer-referentie | ⬜ Doorlopend |

## 7. Concrete waarden — Portiekwoning (referentie)

- 8 ruimtes: WC, Woonkamer, Entree, Slaapkamer 1–3, Keuken, Badkamer
- Winter-ontwerp buitentemp: **−10 °C** (default)
- Constructies: vloer Rc 0.14, schuin dak Rc 2.6 (350 mm), wand Rc 2.6 (300 mm), beglazing Ψ 0.08, g 0.7
- Ontwerptemp woonruimte 20 °C dag / 15 °C nacht; natte ruimtes 24 °C

---
*Scratch-analyse: `/tmp/vabi-re/`. Voorbeelden: `C:\ProgramData\Vabi\Elements\Examples\NL\`.*
