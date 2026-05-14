# Vabi `.vp` SQLite Schema Reference

Geverifieerd schema van `Voorweg 210a - nieuw.vp` (2026-05-14). Gebruik dit document als bron voor importer-queries — niet meer gokken.

## Top-level project chain

```
Project (1 row)
  ID  BIGINT
  Name  TEXT                              -- info.name
  Description  TEXT                       -- info.notes
  CurrentProjectVersionID  BIGINT  -->    ProjectVersion.ID
  ProjectDataID  BIGINT             -->   ProjectData.ID

ProjectData (1 row)
  ID  BIGINT
  ReferenceNumber  TEXT                   -- info.project_number
  Location  TEXT
  PrincipalID  BIGINT
  ConsultantID  BIGINT

ProjectVersion (1 row)
  ID  INTEGER
  ProjectVersionDataID  INTEGER
  -- alle inhoudelijke data hangt aan deze ID via *.ProjectVersionID
```

**Join-pattern Project → ProjectData:** `Project.ProjectDataID = ProjectData.ID`

## Building chain (gebouw-niveau)

```
Building (9 rows in sample)                -- meerdere "buildings" per project mogelijk
  ID  INTEGER
  ProjectVersionID  INTEGER  -->           ProjectVersion.ID
  RequirementsID  INTEGER    -->           BuildingRequirementsData.ID
  NumberOfFloors  INTEGER
  UsageArea  REAL                          -- building.total_floor_area fallback
  BuildingHeight  REAL
  GrossBuildingVolume  REAL
  HasUserDefinedBuildingDimension  INTEGER

BuildingRequirementsData (6 rows)
  ID  BIGINT
  RequirementsID  BIGINT     -->           BuildingDesignRequirements.ID
  ConditionsID  BIGINT       -->           BuildingDesignConditions.ID

BuildingDesignConditions (6 rows)          -- DE belangrijke tabel
  ID  INTEGER
  CertaintyClass  TEXT                     -- building.security_class (ClassA/B/C)
  BuildingShapeType  TEXT                  -- building.building_type input
  BuildingWithHoodType  TEXT               -- building.building_type input (caphH!)
  BuildingWithoutHoodType  TEXT
  MultiStoreyBuildingType  TEXT
  BuildType  TEXT
  ThermalMassType  TEXT
  Qv10Type  TEXT                           -- "Specific" of "Measured" — building.infiltration_method
  MeasuredQv10  REAL                       -- building.qv10 als Qv10Type=Measured
  SpecificQv10  REAL                       -- building.qv10 als Qv10Type=Specific
  HasOpenableWindows  INTEGER
```

**Join Project → Building:** `Building.ProjectVersionID = Project.CurrentProjectVersionID`

**Join Building → BuildingDesignConditions — 5-table chain via Aspect/Template patroon:**

`Building.RequirementsID` is **GEEN directe FK naar BuildingRequirementsData**. Het is een AspectID die via `VarAsp_BuildingRequirementsData` (variant-override resolver) leidt naar ofwel een Template (default) of een CustomID (project-specifieke override). Voor MVP nemen we de Template-path.

```sql
SELECT bdc.*
FROM Building b
JOIN VarAsp_BuildingRequirementsData var ON var.AspectID = b.RequirementsID
JOIN BuildingRequirementsTemplate brt ON brt.ID = var.TemplateID
JOIN BuildingRequirementsData brd ON brd.ID = brt.DataID
JOIN BuildingDesignConditions bdc ON bdc.ID = brd.ConditionsID
WHERE b.ProjectVersionID = ?
LIMIT 1
```

**Detail VarAsp_BuildingRequirementsData:**
- `IsOverridden=0` + `TemplateID` gezet → gebruik template-path (zoals boven)
- `IsOverridden=1` + `CustomID` gezet → `BuildingRequirementsData.ID = var.CustomID` direct
- Voor MVP: gebruik altijd template-path, log warning als IsOverridden=1 (Fase 3 werk)

**Voorweg sample-verificatie:**
- `Building[0].RequirementsID=308419` → `VarAsp.TemplateID=3410` ("Vrijstaande woning met kap") → `BRT.DataID=3441` → `BRD.ConditionsID=3534` → `BDC.ID=3534` ✓

## Climate

```
Climate (1 row)
  ID  BIGINT
  ClimateHeatLossCalculationID  BIGINT  -->  ClimateHeatLossCalculation.ID

ClimateHeatLossCalculation (1 row)
  ID  BIGINT
  HasDefaultDesignOutsideTemperat  BOOL
  DesignOutsideTemperatureWinter  DOUBLE   -- climate.theta_e
```

**Joinpath:** geen directe FK van Project naar Climate gevonden — er is maar 1 row, dus `SELECT DesignOutsideTemperatureWinter FROM ClimateHeatLossCalculation LIMIT 1` is OK voor MVP.

## Ventilation chain

```
Ventilation (6 rows)
  ID  BIGINT
  SupplySource  TEXT                       -- "Natural" of "Mechanical"
  CirculationRateMethod2017  TEXT
  AirCirculationReductionFactor  DOUBLE
  PurgeVentilationProvision  TEXT
  FlowRatesHeatLossID  BIGINT  -->         VentilationFlowRatesHeatLoss.ID
  LocalHeatRecoverySystemXID  BIGINT -->   LocalHeatRecoverySystemX.ID (nullable)

LocalHeatRecoverySystemX (6 rows)          -- aanwezig = WTW; nullable FK
  ID  BIGINT
  ValueBasedOnUnit  DOUBLE
  Unit  TEXT
```

**ventilation.has_heat_recovery:** `Ventilation.LocalHeatRecoverySystemXID IS NOT NULL`

**Note:** Voor gebouw-niveau ventilatie: er zijn 6 Ventilation rows (5 rooms + 1 gebouw?). Onbekend in Fase 1 — neem voor MVP de eerste of join via een nog-te-vinden FK. Documenteer keuze in code.

## Rooms

```
Room (21 rows in Voorweg)                 -- exact 21, klopt met sessie-doc
  ID  BIGINT
  ProjectVersionID  BIGINT
  RoomNumber  TEXT                         -- rooms[].id, bv. "210A.02"
  Name  TEXT                               -- rooms[].name
  UseInCalculations  INTEGER
  RoomRequirementsID  BIGINT  -->          RoomRequirementsData.ID
  VentilationID  BIGINT       -->          Ventilation.ID (per-room override?)

RoomRequirementsData (9 rows)
  ID  BIGINT
  ConditionsID  BIGINT        -->          RoomDesignConditions.ID
  RequirementsID  BIGINT

RoomDesignConditions (9 rows)
  ID  BIGINT
  DesignTemperaturesWinterID  BIGINT  -->  DesignTemperatures.ID
  DesignTemperaturesSummerID  BIGINT

DesignTemperatures (18 rows)
  ID  BIGINT
  TemperatureDay  DOUBLE                   -- rooms[].theta_i
  TemperatureNight  DOUBLE
  ActivityType  TEXT
```

**Full join Room → theta_i — 7-table chain via Aspect/Template:**

Net als Building gaat `Room.RoomRequirementsID` via `VarAsp_RoomRequirementsData.AspectID` → Template → Data → Conditions → DesignTemperatures. **Geen directe FK.**

```sql
SELECT r.RoomNumber, r.Name, dt.TemperatureDay
FROM Room r
JOIN VarAsp_RoomRequirementsData var ON var.AspectID = r.RoomRequirementsID
JOIN RoomRequirementsTemplate rrt ON rrt.ID = var.TemplateID
JOIN RoomRequirementsData rrd ON rrd.ID = rrt.DataID
JOIN RoomDesignConditions rdc ON rdc.ID = rrd.ConditionsID
JOIN DesignTemperatures dt ON dt.ID = rdc.DesignTemperaturesWinterID
WHERE r.ProjectVersionID = ? AND r.UseInCalculations = 1
```

**Ventilation per Room (analoge chain):**
```sql
Room.VentilationID → VarAsp_VentilationData.AspectID → TemplateID
                  → VentilationTemplate.DataID → VentilationData.ID → VentilationID → Ventilation.ID
```

**Generieke regel:** Alle `Room.XxxID` / `Building.XxxID` velden zijn AspectIDs, geen directe data-FKs. Doorloop altijd via `VarAsp_XxxData` of `TmAsp_XxxData` om de werkelijke `XxxData` row te vinden.

## Aspect/Variant patroon (negeren in Fase 1)

`*Aspect`, `*Template`, `*Data`, `TmAsp_*`, `VarAsp_*` zijn variant-overrides. Voor MVP:
- Neem `*Data`-row (effective values)
- Negeer `VarAsp_*` overrides (gebruik default variant)
- Documenteer in code dat dit Fase 3 is

## Voor Fase 2 (BuildingPart, Construction)

Nog niet uitgewerkt — placeholder. Schema-info te halen uit:
- `BuildingPart` (154 rows in Voorweg)
- `BuildingPartType` TEXT (Wall/Floor/Roof)
- `ConstructionID`, `FaceID`, `BoundaryConditionsID`, `AreaInfoID`
- `BuildingPartAspect`, `BuildingPartResult`

Dump full schema voor deze tabellen via `examples/vabi_inspect.rs` als die er komt.
