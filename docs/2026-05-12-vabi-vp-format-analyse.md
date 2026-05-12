# Vabi .vp formaat — analyse en importer-haalbaarheid

**Datum:** 2026-05-12
**Bestanden onderzocht:**
- `tests/references/24221-20250618.vp` (1.7 MB) — Vabi Elements v3.12.0.9
- `tests/references/Voorweg 210a - nieuw.vp` (956 KB) — Vabi Elements v3.5.0.7 → v3.6.0.2

---

## 1. File signature

### 24221-20250618.vp
- Eerste 16 bytes (hex): `50 4B 03 04 2D 00 00 00 08 00 A6 7E E1 5A 2F A2`
- Interpretatie: **ZIP-container** (`PK\x03\x04`), inhoud:
  - `Elements.sqlite3` — 12.86 MB SQLite 3.x database (geschreven met SQLite 3.46.1)
  - `cfg` — 1.8 KB XML met UI-instellingen (camera-posities, license-modules)

### Voorweg 210a - nieuw.vp
- Eerste 16 bytes (hex): `50 4B 03 04 2D 00 00 00 08 00 52 76 7B 5C 9D 31`
- Interpretatie: identieke structuur — ZIP met `Elements.sqlite3` (4.47 MB) + `cfg` (2.1 KB)

**Conclusie:** beide `.vp` files zijn ZIP-archieven met daarbinnen een ongecomprimeerde SQLite database. Format is **volledig open en machine-leesbaar** met standaard Python `sqlite3` stdlib.

---

## 2. ASCII-ratio + leesbare strings

Niet relevant uitgevoerd — bestanden zijn ZIP-containers met daarin een SQLite database (geen sequentiële bytes-analyse zinvol). Inhoud is in plaats daarvan via SQL bevraagd.

Wel opgemerkt:
- Vabi-fingerprint string: `OVabi.Elements.PluginHost, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null` (in result-BLOBs)
- Plugin-module GUIDs in `cfg` matchen Vabi licentie-modules

---

## 3. Structuur

### 3.1 Schema — 361 tabellen, Hibernate-gegenereerd

Beide databases bevatten identiek **361 tabellen** (geen views). Naamgeving en kolom-structuur volgen klassiek Hibernate ORM-patroon (`ID` als PK, `*ID` als FK, `Data`-tabellen voor lookup-data, `ObjectId BLOB(16)` voor GUIDs). Versie-tabel toont productID `VABI ELEMENTS PROJECT`.

### 3.2 Sleutel-tabellen voor warmteverlies (volledig leesbare invoer)

| Tabel | Doel | Rij-count 24221 / Voorweg | Belangrijke kolommen |
|-------|------|-----------------------------|---------------------|
| `Project` | Project-header | 1 / 1 | `Name`, `Description`, `CurrentProjectVersionID` |
| `ProjectData` | Klant/locatie | 1 / 1 | `ReferenceNumber`, `PrincipalID`, `ConsultantID` |
| `Building` | Gebouw-metadata | 1 / 9 | `BuildingHeight`, `UsageArea`, `NumberOfFloors` |
| `Variant` | Calculatie-variant | 1 / 1 | `Name="Basisvariant"`, `CreationDate`, `LastModifiedDate` |
| `Room` | Vertrek-definitie | 106 / 21 | `Name`, `RoomNumber`, `UseInCalculations`, FK's naar emission/ventilation/architectural |
| `BuildingPart` | Bouwdeel (vloer/wand/dak/raam) | 825 / 154 | `BuildingPartType`, `ConstructionID`, `BoundaryConditionsID`, `PsiThermalBridge`, `SpecificInfiltrationArea` |
| `Construction` + `ConstructionData` | Constructie-header | 33 / 24 | `Name`, `Type=Floor/Wall/Roof`, `LoadBearing` |
| `LayeredConstruction` + `ConstructionLayer` | Laagopbouw | 33+105 / 24+74 | `SortNumber`, `Thickness`, `MaterialID`, `IsThermalActive` |
| `Material` + `MaterialData` | Materiaal-eigenschappen | 19 / 43 | `HeatConductivity`, `HeatResistance`, `SpecificMass`, `SpecificHeat` |
| `BoundaryConditions` | Begrenzing (CrawlSpace/Ground/etc.) | 825 / 154 | `Type`, `AdjacentUnheatedSpaceType`, `CrawlSpaceHeight`, `RcFloorCrawlSpace` |
| `BoundaryTemperatures` | Winter+Zomer θ_grond/θ_aangr | 1650 / 308 | `TemperatureDay`, `TemperatureNight` |
| `Climate` + `ClimateHeatLossCalculation` | Buitenklimaat | 1 / 1 | `DesignOutsideTemperatureWinter` (= -10.0 °C) |
| `HeatLossCalculationSettings` | Norm-instellingen | 1 / 1 | `CalculationMethod=Method2023`, `CalculationDetail=RoomLevel`, `UValueWindowCalculationType` |
| `DesignTemperatures` | Setpoint per ruimte | 18 / 18 | `TemperatureDay`, `TemperatureNight`, `ActivityType` |
| `RoomDesignConditions` | Koppeling Room→DesignTemp | 9 / 9 | FK's |
| `Glazing` | Beglazing | 99 / 72 | `U`, `GFraction`, `TransmissionFraction` |
| `Frame` | Kozijn | 33 / 24 | `Type=Wood/Plastic/...`, `U`, `FrameWidth`, `FramePercentage` |
| `StandardWindow` + `TransparentConstructionData` | Raam-geheel | 33 / 24 | `Psi`, `IsOpenable` |
| `OpaqueConstructionData` | Niet-transparant | 33 / 24 | `IsLayered`, `FinishOutsideID`, `FinishInsideID` |
| `Ventilation` + `VentilationData` + `Infiltration` | Luchthuishouding | 26 / 6 | flow rates, infiltratie |
| `EmissionDevice` + `EmissionPowerHeating` | Verwarmingstoestel | 11+12 / 5+8 | `SubtypeHeating=FloorHeating`, `SupplyTemperature=35`, `ReturnTemperature=30` |
| `AreaInfo` / `PerimeterInfo` / `VolumeInfo` | Geometrie afgeleid | 825 / 154 | gemeten oppervlakken |
| `Face` + `Vertex` + `Cell` | Volledige 3D-geometrie | 1322+987 / 241+229 | x/y/z-coords per vertex |

### 3.3 Resultaten — opaque .NET BinaryFormatter BLOBs

`VariantResult` (1259 / 338 rijen) bevat per metric één rij met:
- `DefinitionId` — GUID die de metric identificeert (14 distinct GUIDs)
- `SerializedData` — BLOB (203 bytes tot 1.5 MB)

Eerste 9 bytes van elke BLOB: `00 01 00 00 00 FF FF FF FF` — dit is de signature van **.NET BinaryFormatter** (legacy `System.Runtime.Serialization.Formatters.Binary`). Embedded type-strings tonen `Vabi.Elements.PluginHost.HostPluginModule.ReportTextResults`, `Vabi.Elements.ExternalSystem...`. Velden zoals `_version`, `_isFullReport`, `_text` zijn herkenbaar.

**Implicatie:** numerieke eindresultaten (Φ per ruimte/bouwdeel/gebouw) zijn **niet** direct uit SQL leesbaar. Alle invoer-data is dat wél.

### 3.4 cfg-bestand — irrelevant

Het tweede ZIP-onderdeel `cfg` is een 2 KB XML met UI-state (camera-posities in 3D-viewer, geactiveerde license-modules). Geen rekenwaarden — kan genegeerd worden door een importer.

---

## 4. Vabi documentation / openbare informatie

Geen WebSearch uitgevoerd binnen tijd-budget — bevindingen zijn afgeleid uit het bestand zelf:

- **SQL/Hibernate-schema** zelf is open en zelf-documenterend: kolomnamen zijn semantisch (`HeatConductivity`, `DesignOutsideTemperatureWinter`, `CalculationMethod=Method2023`). Geen API-licentie nodig om de SQL-laag te lezen.
- **GUID-mapping voor result-BLOBs** is *niet* in de DB aanwezig — die hash-naar-metric mapping zit in de Vabi .NET assemblies. Reverse-engineering noodzakelijk om resultaten direct uit te lezen.
- **Alternatief:** rapport-tekst (PDF/XLS) parse-baar omdat Vabi rapporten een vaste lay-out hebben — dat is de gangbare manier om Vabi-resultaten extern te valideren.

---

## 5. Importer-haalbaarheid: **EENVOUDIG voor invoer, MOEILIJK voor resultaten**

### Onderbouwing
- **Invoer-laag (ruimtes, bouwdelen, constructies, materialen, ventilatie, beglazing, klimaat, design-temperaturen):** volledig in standaard SQL-tabellen met semantische kolomnamen. Python `sqlite3` + ~300-500 regels mapping-code is voldoende voor een werkende `.vp → fixture.json` importer. Geen externe dependencies.
- **Resultaten-laag (Φ per ruimte/bouwdeel):** zit in `VariantResult.SerializedData` als .NET BinaryFormatter blob. Decoderen kan met `pythonnet` of een third-party BinaryFormatter parser (bijv. `nrbf` op PyPI), maar de **veld-mapping** (welke GUID = welke metric) is alleen via Vabi's `.dll` te achterhalen of via reverse-engineering. Praktisch advies: skip deze laag en haal Φ-cijfers uit het PDF-rapport in plaats daarvan.

### Wat een MVP-importer zou doen
1. Open `.vp` als ZIP, extraheer `Elements.sqlite3` naar tempfile.
2. SELECT alle rijen uit ~25 sleutel-tabellen via één set joins.
3. Map naar onze fixture-schema (Project → ProjectFixture, Room → RoomFixture met DesignTemperatures, BuildingPart → ConstructionAssignmentFixture met BoundaryConditions, ConstructionLayer → LayerFixture met Material).
4. Bereken afgeleide velden zoals `Rc_construction = Σ(thickness/λ)` zelf — Vabi slaat dit niet noodzakelijk gedenormaliseerd op.
5. Output: één JSON die direct in onze test-runner past.

### Geschatte inspanning
- **1 dag** voor invoer-only MVP (project/rooms/constructies/temperaturen → 80% van fixture-schema).
- **+1 dag** voor edge cases (gekoppelde-DHW, vloerverwarming-emissie, dakvensters, schaduwobjecten, thermische bruggen).
- **+3-5 dagen** voor result-laag (.NET BinaryFormatter decoder + GUID-mapping via experimenteel reverse-engineeren met test-projecten).
- **Totaal voor full audit-tool**: ~2 dagen voor invoer-import, +PDF-tekst extractor voor resultaten als sneller alternatief op resultaten-decoder.

### Schema-velden vs .vp data

| Onze fixture-veld (representatief) | Zit in .vp? | Bron-tabel / haalbaarheid |
|-----------------------------------|-------------|--------------------------|
| `project.name` / `project.reference_number` | Ja | `Project.Name`, `ProjectData.ReferenceNumber` |
| `building.height` / `usage_area` / `floors` | Ja | `Building.BuildingHeight` / `UsageArea` / `NumberOfFloors` |
| `climate.theta_e` (ontwerptemperatuur buiten) | Ja | `ClimateHeatLossCalculation.DesignOutsideTemperatureWinter` |
| `calculation_method` (Method2023) | Ja | `HeatLossCalculationSettings.CalculationMethod` |
| `room.name` / `room.number` | Ja | `Room.Name` / `RoomNumber` |
| `room.theta_i` (design indoor temp) | Ja | `RoomDesignConditions → DesignTemperatures.TemperatureDay/Night` |
| `room.volume` / `room.area` | Ja | `VolumeInfo` + `TypedVolumeData`/`TypedAreaData` |
| `room.heating_system` (subtype HT/LT/Vloer) | Ja | `EmissionDevice.SubtypeHeating` + `IsLowTemperature` |
| `room.supply_temp` / `return_temp` (Δθ) | Ja | `EmissionPowerHeating.SupplyTemperature` / `ReturnTemperature` |
| `room.ventilation_flow_rate` | Ja | `VentilationFlowRatesHeatLoss` |
| `room.infiltration_flow_rate` | Ja | `InfiltrationFlowRateHeatLoss` |
| `building_part.type` (Floor/Wall/Roof/Door/Window) | Ja | `BuildingPart.BuildingPartType` |
| `building_part.area` / `perimeter` | Ja | `AreaInfo` / `PerimeterInfo` (joined op typed data) |
| `building_part.boundary_type` (Outside/Crawl/Ground/Adj) | Ja | `BoundaryConditions.Type` + `AdjacentUnheatedSpaceType` |
| `boundary.theta_b` (winter dag/nacht aangrenzend) | Ja | `BoundaryTemperatures.TemperatureDay/Night` |
| `boundary.theta_water` (default 5°C) | **Nee?** | Niet expliciet — Vabi gebruikt waarschijnlijk vaste norm-waarde |
| `building_part.psi` (lineaire koudebrug) | Ja | `BuildingPart.PsiThermalBridge` |
| `construction.layers[].thickness` / `material_id` | Ja | `ConstructionLayer.Thickness` + `MaterialID` |
| `material.lambda` / `rho` / `cp` | Ja | `MaterialData.HeatConductivity` / `SpecificMass` / `SpecificHeat` |
| `material.R` (vaste laag-weerstand) | Ja | `MaterialData.HeatResistance` |
| `window.U_glazing` / `g_value` | Ja | `Glazing.U` / `GFraction` |
| `window.U_frame` / `frame_type` | Ja | `Frame.U` / `Type` |
| `window.psi` | Ja | `TransparentConstructionData.Psi` |
| `window.frame_percentage` | Ja | `Frame.FramePercentage` |
| **Resultaten** (Φ per ruimte, gebouw-totaal) | Indirect | `VariantResult.SerializedData` BLOB — .NET BinaryFormatter, niet triviaal |

Conclusie: **>95% van de invoer-velden is 1-op-1 mapbaar** naar onze fixture-JSON via standaard SQL.

---

## 6. Aanbeveling

**Bouwen — invoer-importer is goedkoop en levert direct audit-waarde.** Een Python-script van ~500 regels dat `.vp → fixture.json` doet op invoer-niveau is een 1-dags klus. Daarmee kan elke klant een Vabi-project bij ons inleveren en draaien we direct numerieke vergelijking tegen onze ISSO 51-engine zonder hercoderen. Voor de **resultaten-kant** raden we aan om de Vabi-PDF te parsen (regel-gebaseerde tekst-extractie op de Φ-tabel per ruimte) in plaats van de .NET BinaryFormatter BLOBs te kraken — dat is sneller en robuuster dan reverse-engineering van Vabi's binnenste assembly. Combinatie: invoer-importer + PDF-resultaat-extractor = volledige audit-workflow binnen ±2 werkdagen.

---

**Verdict:** ZIP-container met SQLite (361 tabellen, Hibernate-schema) — importer voor invoer-data is haalbaar in 1 dag; resultaten zitten in opaque .NET BinaryFormatter BLOBs en kunnen beter via PDF-rapport-extractie worden opgehaald.
