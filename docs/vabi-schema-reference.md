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

## Fase 2 — BuildingPart, Constructies, U-waardes, Geometrie

Verified op Voorweg sample (2026-05-14).

### BuildingPart (154 rows)

```
BuildingPart
  ID  BIGINT
  HasConstruction  BOOL                    -- 0 voor virtuele/boundary parts
  BuildingPartType  TEXT                   -- 'Wall', 'Floor', 'Roof' (FlatRoof, Door via Construction.Type)
  HasBoundaryConditions  BOOL
  IsOpenable  BOOL
  PsiThermalBridge  DOUBLE                 -- thermal bridge psi op part-niveau
  IsVirtual  BOOL
  AreaInfoID  BIGINT       -->             VariantAreaInfo via AreaInfoID
  PerimeterInfoID  BIGINT  -->             VariantPerimeterInfo
  ConstructionID  BIGINT (nullable)  -->   Construction.ID (only if HasConstruction=1)
  FaceID  BIGINT           -->             Face.ID (geometry)
  BoundaryConditionsID  BIGINT  -->        BoundaryConditions.ID
  ProjectVersionID  BIGINT
  LoadBearing  TEXT
```

### Room → BuildingPart linkage (cell-based geometry)

```
Room.CellID (e.g. 420112)
  → MainFace.CellID = Room.CellID
  → MainFace.CellFaceID (= Face.ID's voor deze cel)
  → CellFace.FaceID = MainFace.CellFaceID, CellFace.BuildingPartID
  → BuildingPart.ID = CellFace.BuildingPartID
```

```sql
SELECT bp.*
FROM Room r
JOIN MainFace mf ON mf.CellID = r.CellID
JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
WHERE r.ID = ?
```

### Geometry (Area, Orientation)

```
BuildingPart.FaceID  → Face.ID
Face.FaceGeometryEngineID  → FaceGeometryEngine
   .Orientation  DOUBLE                    -- azimuth in degrees (0=N, 90=E, ...)
   .Slope  DOUBLE                          -- 0=floor, 90=wall, 180=roof
   .Area  DOUBLE                           -- m²
   .Perimeter  DOUBLE                      -- m
```

Voor multi-dimension support gebruikt Vabi ook `TypedAreaData` (per `Type='CentreToCentreDimensions'` of `'InternalDimensionsIncludingPlenum'`). Voor MVP-import gebruik `FaceGeometryEngine.Area` direct.

### Construction → U-waarde (opaque)

```
BuildingPart.ConstructionID
  → Construction.DataID
  → ConstructionData.OpaqueConstructionDataID
  → OpaqueConstructionData.LayeredConstructionID  (als IsLayered=1)
  → ConstructionLayer.LayeredConstructionID = LayeredConstructionID
```

```sql
SELECT cl.Thickness, cl.SortNumber, md.HeatConductivity, md.HeatResistance
FROM BuildingPart bp
JOIN Construction c ON c.ID = bp.ConstructionID
JOIN ConstructionData cd ON cd.ID = c.DataID
JOIN OpaqueConstructionData ocd ON ocd.ID = cd.OpaqueConstructionDataID
JOIN ConstructionLayer cl ON cl.LayeredConstructionID = ocd.LayeredConstructionID
JOIN Material m ON m.ID = cl.MaterialID
JOIN MaterialData md ON md.ID = m.DataID
WHERE bp.ID = ?
ORDER BY cl.SortNumber
```

**U-waarde berekening (ISO 6946):**
```
R_layer_i = Thickness_mm * 1e-3 / HeatConductivity_W_per_mK       (if λ > 0)
          = HeatResistance                                          (if λ == 0, pre-computed)
R_total = R_si + Σ R_layer_i + R_se
U = 1 / R_total
```

R_si / R_se afhankelijk van slope + boundary type:
| Surface | R_si | R_se (outside) |
|---|---:|---:|
| Vertical wall | 0.13 | 0.04 |
| Floor (heat flow down) | 0.17 | 0.04 |
| Roof/ceiling (heat flow up) | 0.10 | 0.04 |
| Ground boundary | — | 0.0 (via separate model) |

**Alternatief — gebruik StandardConstruction.RcValue als pre-computed:**
```
OpaqueConstructionData.StandardConstructionID → StandardConstruction.RcValue
```
Als RcValue > 0: gebruik direct; anders compute from layers.

### Construction → U-waarde (transparent, kozijnen)

```
ConstructionData.TransparentConstructionDataID
  → TransparentConstructionData
       .FrameID → Frame.U  (W/m²K)
       .StandardWindowID → StandardWindow.GlazingID → Glazing.U
       .Psi  (kozijn-glas verbinding psi)
```

U-window combineren met framepercentage:
```
U_window = FramePercentage * Frame.U + (1 - FramePercentage) * Glazing.U + 2 * Psi * L_glass / A_window
```
(Voor MVP: simpele weighted average, sla psi-correctie eerst over en mark TODO.)

### BoundaryConditions → boundary type mapping

```
BoundaryConditions.Type  TEXT  -- 'OutsideAir', 'Ground', 'AdjacentRoom', 'AdjacentBuilding', etc.
BoundaryConditions.BoundaryTemperaturesWinterID → BoundaryTemperatures.TemperatureDay (theta voor adjacent)
```

Mapping naar ons model (zie `crates/isso51-core/src/model/`):
- `'OutsideAir'` → `BoundaryType::Exterior`
- `'Ground'` → `BoundaryType::Ground`
- `'AdjacentRoom'` → `BoundaryType::AdjacentRoom` met theta uit BoundaryTemperatures
- `'AdjacentBuilding'` → `BoundaryType::AdjacentBuilding` met theta uit BoundaryTemperatures
- onbekend → log warning, default Exterior

### Room.height bron — TypedVolumeData via VolumeInfoID

```
Room.VolumeInfoID  → VariantVolumeInfo.VolumeInfoID = Room.VolumeInfoID
VariantVolumeInfo.ID  → TypedVolumeData.VariantVolumeInfoID
TypedVolumeData.Type = 'InternalDimensionsIncludingPlenum'  → Volume (m³)
```

Room.height kan berekend worden als: `Volume / floor_area` waar floor_area = som van alle Floor BuildingPart areas voor die Room.

```sql
SELECT tvd.Volume / NULLIF(SUM(fge_floor.Area), 0) as room_height
FROM Room r
JOIN VariantVolumeInfo vvi ON vvi.VolumeInfoID = r.VolumeInfoID
JOIN TypedVolumeData tvd ON tvd.VariantVolumeInfoID = vvi.ID AND tvd.Type = 'InternalDimensionsIncludingPlenum'
LEFT JOIN (
    SELECT r2.ID as room_id, SUM(fge.Area) as floor_area
    FROM Room r2
    JOIN MainFace mf ON mf.CellID = r2.CellID
    JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
    JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
    JOIN Face f ON f.ID = bp.FaceID
    JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
    WHERE bp.BuildingPartType = 'Floor' AND bp.HasConstruction = 1
    GROUP BY r2.ID
) floor_data ON floor_data.room_id = r.ID
WHERE r.ID = ?
```

**Verified op Voorweg sample:** Room 210A.-1 heeft Volume=26.93m³, floor_area=12.10m², height=2.23m.

### Fase 2 U-waarde berekening — geverifieerd

**Opaque constructions (lagen-gebaseerd):**
```sql
SELECT cl.Thickness, cl.SortNumber, md.HeatConductivity, md.HeatResistance
FROM BuildingPart bp
JOIN Construction c ON c.ID = bp.ConstructionID
JOIN ConstructionData cd ON cd.ID = c.DataID
JOIN OpaqueConstructionData ocd ON ocd.ID = cd.OpaqueConstructionDataID
JOIN ConstructionLayer cl ON cl.LayeredConstructionID = ocd.LayeredConstructionID
JOIN Material m ON m.ID = cl.MaterialID
JOIN MaterialData md ON md.ID = m.DataID
WHERE bp.ID = ? AND ocd.IsLayered = 1
ORDER BY cl.SortNumber
```

R_layer berekening per laag:
- Indien `HeatConductivity > 0`: `R = (Thickness_mm * 1e-3) / HeatConductivity`
- Indien `HeatConductivity == 0`: `R = HeatResistance` (direct, voor spouwen/lucht)

**StandardConstruction fallback:**
```sql
SELECT sc.RcValue
FROM BuildingPart bp → ... → OpaqueConstructionData ocd
JOIN StandardConstruction sc ON sc.ID = ocd.StandardConstructionID
WHERE bp.ID = ? AND sc.RcValue > 0
```

Indien `RcValue > 0`: `U = 1 / (R_si + RcValue + R_se)`, sla layered berekening over.

**Geverifieerd voorbeeld** (Wand - Buiten spouw + VZW):
- Layer 1: Baksteen 100mm, λ=0.8 → R=0.125
- Layer 2: Spouw 60mm → R=0.17 (direct)
- Layer 3: Baksteen 100mm, λ=0.8 → R=0.125
- Layer 4: PIR 93mm, λ=0.022 → R=4.227
- Layer 5: Gipsplaat 12mm, λ=0.23 → R=0.052
- **Totaal:** R_layers = 4.699, U = 1/(0.13 + 4.699 + 0.04) = 0.205 W/(m²·K)

**Transparent constructions:** via `TransparentConstructionData → Frame.U + Glazing.U` (layered method niet van toepassing).

### Aspect/Variant ook hier?

`BuildingPartAspect` (328 rows) bestaat. Voor MVP: negeren — gebruik direct de BuildingPart data zonder aspect-resolving. Documenteer als TODO voor Fase 3 als project-varianten ondersteund worden.
