//! SQLite database mapping from Vabi Elements to ISSO 51 Project structures.
//!
//! Maps Vabi's ERM (Entity-Relationship Model) to our domain model, handling
//! the multi-table joins needed to extract project, building, climate, ventilation,
//! and room data.

use crate::error::{Isso51Error, Result};
use crate::model::*;
use crate::model::enums::*;
use crate::model::construction::*;
use crate::tables::thermal_bridge;
use rusqlite::Connection;
use std::path::Path;

/// Custom error type for Vabi import failures
#[derive(Debug)]
pub struct VabiImportError(pub String);

impl std::fmt::Display for VabiImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vabi import error: {}", self.0)
    }
}

impl std::error::Error for VabiImportError {}

impl From<Isso51Error> for VabiImportError {
    fn from(err: Isso51Error) -> Self {
        VabiImportError(err.to_string())
    }
}

impl From<rusqlite::Error> for VabiImportError {
    fn from(err: rusqlite::Error) -> Self {
        VabiImportError(format!("SQLite error: {}", err))
    }
}

/// Resultaat van een Vabi-import: het gemapte project plus alle niet-fatale
/// import-warnings (afwijkingen, fallbacks, niet-afleidbare velden).
///
/// Volgt het patroon van [`crate::import::thermal::ThermalImportResult`]:
/// warnings horen in het resultaat, niet op stderr — de caller (UI/API)
/// beslist hoe ze getoond worden.
#[derive(Debug, Clone)]
pub struct VabiImportResult {
    /// Het gemapte ISSO 51-project, klaar voor berekening.
    pub project: Project,
    /// Niet-fatale warnings die tijdens het mappen zijn gegenereerd
    /// (gededupliceerd op exact gelijke meldingen).
    pub warnings: Vec<String>,
}

/// Import a complete project from a Vabi `.vp` file, including import warnings.
///
/// This is the preferred public API for Vabi import. Opens the ZIP archive,
/// extracts the SQLite database, and maps all relevant data to our Project model.
///
/// # Arguments
/// * `vp_path` - Path to the Vabi `.vp` project file
///
/// # Returns
/// [`VabiImportResult`] with the complete `Project` structure ready for
/// ISSO 51 calculation plus any non-fatal import warnings.
pub fn import_vabi_project_with_warnings(vp_path: &Path) -> Result<VabiImportResult> {
    let (db_path, _temp_file) = super::unzip::extract_elements_database(vp_path)?;

    let conn = Connection::open(&db_path).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Cannot open Elements.sqlite3: {}", e))
    })?;

    let mut warnings: Vec<String> = Vec::new();

    let project_info = map_project_info(&conn)?;
    let building = map_building_with_warnings(&conn, &mut warnings)?;
    let climate = map_climate(&conn)?;
    let ventilation = map_ventilation(&conn)?;
    let rooms = map_rooms(&conn, climate.theta_e, &mut warnings)?;

    Ok(VabiImportResult {
        project: Project {
            info: project_info,
            building,
            climate,
            ventilation,
            rooms,
        },
        warnings,
    })
}

/// Import a complete project from a Vabi `.vp` file.
///
/// Backward-compatible wrapper around [`import_vabi_project_with_warnings`]
/// for callers without a warning channel (Tauri command, examples). Warnings
/// are echoed to stderr so they are not silently lost (precedent:
/// `calc/transmission.rs::resolve_adjacent_temperature` — stderr is the only
/// logging vehicle the pure-Rust core has). New code should prefer
/// [`import_vabi_project_with_warnings`].
///
/// # Returns
/// Complete `Project` structure ready for ISSO 51 calculation.
pub fn import_vabi_project(vp_path: &Path) -> Result<Project> {
    let result = import_vabi_project_with_warnings(vp_path)?;
    for warning in &result.warnings {
        eprintln!("vabi-import: {warning}");
    }
    Ok(result.project)
}

/// Voeg een import-warning toe met dedupe op exact gelijke meldingen.
/// Voorkomt warning-spam wanneer honderden vlakken dezelfde oorzaak delen.
fn push_warning(warnings: &mut Vec<String>, msg: String) {
    if !warnings.iter().any(|w| w == &msg) {
        warnings.push(msg);
    }
}


/// Map basic project information from Project and ProjectData tables.
pub fn map_project_info(conn: &Connection) -> Result<ProjectInfo> {
    let mut stmt = conn
        .prepare(
            "SELECT
                p.Name as project_name,
                p.Description as description,
                pd.ReferenceNumber as ref_number
             FROM Project p
             JOIN ProjectData pd ON p.ProjectDataID = pd.ID
             LIMIT 1"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Project query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Project query execution failed: {}", e))
    })?;

    if let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Project row fetch failed: {}", e))
    })? {
        let name: String = row.get(0).unwrap_or_else(|_| "Unnamed Project".to_string());
        let description: Option<String> = row.get(1).unwrap_or(None);
        let project_number: Option<String> = row.get(2).unwrap_or(None);

        Ok(ProjectInfo {
            name,
            notes: description,
            project_number,
            address: None,
            client: None,
            date: None,
            engineer: None,
        })
    } else {
        Err(Isso51Error::VabiImport(
            "No project data found in Vabi database".to_string(),
        ))
    }
}

/// Map building design conditions from BuildingDesignConditions table.
///
/// `Building.RequirementsID` is an **AspectID**, not a direct FK to BuildingRequirementsData.
/// Vabi uses an Aspect/Template/Variant pattern: AspectID → VarAsp_BuildingRequirementsData →
/// BuildingRequirementsTemplate.DataID → BuildingRequirementsData.ID → ConditionsID →
/// BuildingDesignConditions.ID. See `docs/vabi-schema-reference.md`.
///
/// Phase 1 uses the Template path (IsOverridden=0 case). Phase 3 should handle
/// `IsOverridden=1` + `CustomID` for project-specific overrides.
///
/// Backward-compatible wrapper around [`map_building_with_warnings`] that
/// discards the warnings (used by the V2 importer in `crates/vabi-importer`).
pub fn map_building(conn: &Connection) -> Result<Building> {
    map_building_with_warnings(conn, &mut Vec::new())
}

/// Map building design conditions from BuildingDesignConditions, with warnings.
///
/// Zie [`map_building`] voor de join-keten. Verschillen met de eerdere
/// Phase 1-versie (audit §2.2-fixes):
/// - `dwelling_class` wordt afgeleid uit de Vabi-data (zie
///   [`derive_dwelling_class`]) zodat de `VabiCompat`-infiltratiemethode
///   (ISSO 51:2023 Tabel 2.8) niet meer faalt met
///   `Isso51Error::InfiltrationConfig`.
/// - `has_night_setback` is **`false`** in plaats van hardcoded `true`. De
///   gereverse-engineerde Vabi-structuren (`docs/vabi-schema-reference.md`,
///   `tools/vabi-validation/extract_vp.py`) bevatten géén expliciet
///   bedrijfsbeperking-veld. `DesignTemperatures.TemperatureNight` is géén
///   betrouwbare indicator: het Vabi worked example heeft dag/nacht-setpoints
///   terwijl het rapport "Continu / Afzien van bedrijfsbeperking" meldt (zie
///   `docs/2026-05-12-vabi-isso51-2023-worked-examples.md`). Norm-veilige
///   default is dus `false` — géén fantoom-opwarmtoeslag `Φ_hu = P × A_g`
///   (ISSO 51:2023 §4.3) — met een import-warning zodat de gebruiker het
///   bewust kan aanzetten.
pub fn map_building_with_warnings(
    conn: &Connection,
    warnings: &mut Vec<String>,
) -> Result<Building> {
    let mut stmt = conn
        .prepare(
            "SELECT
                bdc.SpecificQv10,
                bdc.MeasuredQv10,
                bdc.Qv10Type,
                bdc.BuildingShapeType,
                bdc.BuildingWithHoodType,
                bdc.CertaintyClass,
                b.UsageArea,
                b.NumberOfFloors,
                bdc.BuildingWithoutHoodType,
                bdc.MultiStoreyBuildingType
             FROM Building b
             JOIN Project p ON b.ProjectVersionID = p.CurrentProjectVersionID
             JOIN VarAsp_BuildingRequirementsData var ON var.AspectID = b.RequirementsID
             JOIN BuildingRequirementsTemplate brt ON brt.ID = var.TemplateID
             JOIN BuildingRequirementsData brd ON brd.ID = brt.DataID
             JOIN BuildingDesignConditions bdc ON bdc.ID = brd.ConditionsID
             LIMIT 1"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Building query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Building query execution failed: {}", e))
    })?;

    if let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Building row fetch failed: {}", e))
    })? {
        let specific_qv10: Option<f64> = row.get(0).unwrap_or(None);
        let measured_qv10: Option<f64> = row.get(1).unwrap_or(None);
        let qv10_type: Option<String> = row.get(2).unwrap_or(None);
        let building_shape: Option<String> = row.get(3).unwrap_or(None);
        let with_hood: Option<String> = row.get(4).unwrap_or(None);
        let certainty_class: Option<String> = row.get(5).unwrap_or(None);
        let usage_area: Option<f64> = row.get(6).unwrap_or(None);
        let num_floors: Option<i32> = row.get(7).unwrap_or(None);
        let without_hood: Option<String> = row.get(8).unwrap_or(None);
        let multi_storey: Option<String> = row.get(9).unwrap_or(None);

        // Map qv10 value based on type
        let (qv10, infiltration_method) = match qv10_type.as_deref() {
            Some("Specific") => (
                specific_qv10.unwrap_or(0.5), // fallback to reasonable default
                InfiltrationMethod::VabiCompat,
            ),
            Some("Measured") => (
                measured_qv10.unwrap_or(100.0), // fallback to reasonable default
                InfiltrationMethod::MeasuredQv10,
            ),
            _ => {
                // Unknown type - use VabiCompat as default
                (specific_qv10.or(measured_qv10).unwrap_or(0.5), InfiltrationMethod::VabiCompat)
            }
        };

        // Map building type
        let building_type = map_building_type(building_shape.as_deref(), with_hood.as_deref());

        // Map security class
        let security_class = match certainty_class.as_deref() {
            Some("ClassA") => SecurityClass::A,
            Some("ClassB") => SecurityClass::B,
            Some("ClassC") => SecurityClass::C,
            _ => SecurityClass::B, // default
        };

        // Determine total floor area - use Building.UsageArea or fallback to room sum
        let total_floor_area = usage_area.unwrap_or_else(|| {
            // TODO: sum room areas when room import is implemented
            0.0
        });

        // Determine number of floors - convert i32 to u32
        let num_floors = num_floors.unwrap_or(1) as u32;

        // Woningklasse (ISSO 51:2023 Tabel 2.8) — vereist voor de
        // VabiCompat-infiltratieketen; zonder dit veld faalt elke berekening
        // van het geïmporteerde project met Isso51Error::InfiltrationConfig.
        let dwelling_class = derive_dwelling_class(
            conn,
            building_type,
            with_hood.as_deref(),
            without_hood.as_deref(),
            multi_storey.as_deref(),
            warnings,
        );

        // Bedrijfsbeperking/nachtverlaging is niet uit de geparste
        // Vabi-structuren af te leiden — zie doc-comment van
        // `map_building_with_warnings`. Default `false` = geen opwarmtoeslag
        // Φ_hu (ISSO 51:2023 §4.3), mét warning.
        push_warning(
            warnings,
            "Nachtverlaging/bedrijfsbeperking is niet uit het Vabi-bestand af te leiden — \
             'has_night_setback' staat op uit (geen opwarmtoeslag Φ_hu, ISSO 51:2023 §4.3). \
             Zet dit handmatig aan als het project met bedrijfsbeperking rekent."
                .to_string(),
        );

        Ok(Building {
            building_type,
            qv10,
            infiltration_method,
            total_floor_area,
            num_floors,
            security_class,
            has_night_setback: false, // niet afleidbaar uit Vabi-data → norm-veilig uit
            warmup_time: 2.0,         // ISSO 51 standard
            building_height: None,
            dwelling_class: Some(dwelling_class),
            construction_variant: None,
            construction_year: None,
            aggregation_method: AggregationMethod::default(),
            heating_control_type: Default::default(),
            c_eff: None,
            built_after_2015: true,
            all_floor_heating: false,
        })
    } else {
        Err(Isso51Error::VabiImport(
            "No building design conditions found in Vabi database".to_string(),
        ))
    }
}

/// Map building type from Vabi enum values.
fn map_building_type(building_shape: Option<&str>, with_hood: Option<&str>) -> BuildingType {
    match (building_shape, with_hood) {
        (Some("Detached"), _) => BuildingType::Detached,
        (Some("SemiDetached"), _) => BuildingType::SemiDetached,
        (Some("CornerBuilding"), _) => BuildingType::EndOfTerrace, // TODO Phase 3: verify Corner -> EndOfTerrace mapping
        (Some("Terraced"), _) => BuildingType::Terraced,
        (Some("Gallery"), _) => BuildingType::Gallery,
        (Some("Porch"), _) => BuildingType::Porch,
        (Some("Apartment"), _) => BuildingType::Stacked, // TODO Phase 3: verify Apartment -> Stacked mapping
        (_, Some("WithHood")) => BuildingType::Stacked, // assume stacked for hood buildings
        _ => BuildingType::Detached, // safe default
    }
}

/// Is een Vabi-enumveld daadwerkelijk gezet (niet leeg/None/Unknown)?
fn vabi_enum_set(value: Option<&str>) -> bool {
    matches!(
        value,
        Some(s) if !s.is_empty()
            && !s.eq_ignore_ascii_case("none")
            && !s.eq_ignore_ascii_case("unknown")
    )
}

/// Leid de [`DwellingClass`] (ISSO 51:2023 Tabel 2.8) af uit de Vabi-data.
///
/// Tabel 2.8 keyt `q_i,spec` op drie woningklassen: eengezinswoning met kap
/// (1,0 dm³/(s·m²)), eengezinswoning met plat dak (0,7) en
/// etage/flat/portiek (0,5). Afleidingsvolgorde:
///
/// 1. **Gestapelde bouw**: `BuildingShapeType` → Gallery/Porch/Stacked, of
///    `MultiStoreyBuildingType` gezet → [`DwellingClass::EtageFlatOfPortiek`].
/// 2. **Geometrie-scan** (hardste bewijs): dakvlakken in de Vabi-DB met een
///    hellende `FaceGeometryEngine.Slope` → met kap; alleen vlakke daken →
///    plat dak. (Slope-conventie: 0 = vloer, 90 = wand, 180 = plat dak —
///    zie `docs/vabi-schema-reference.md`; alles daartussen is hellend.)
/// 3. **Kap-velden**: `BuildingWithHoodType` gezet → met kap;
///    `BuildingWithoutHoodType` gezet → plat dak.
/// 4. **Fallback**: eengezinswoning met kap (hoogste `q_i,spec` = 1,0 →
///    conservatief) + import-warning. Géén stille fout: de berekening blijft
///    werken én de gebruiker ziet dat de klasse gecontroleerd moet worden.
fn derive_dwelling_class(
    conn: &Connection,
    building_type: BuildingType,
    with_hood: Option<&str>,
    without_hood: Option<&str>,
    multi_storey: Option<&str>,
    warnings: &mut Vec<String>,
) -> DwellingClass {
    // 1. Gestapelde bouw → etage/flat/portiek.
    if matches!(
        building_type,
        BuildingType::Gallery | BuildingType::Porch | BuildingType::Stacked
    ) || vabi_enum_set(multi_storey)
    {
        return DwellingClass::EtageFlatOfPortiek;
    }

    // 2. Geometrie-scan over de dakvlakken.
    if let Some((pitched, total)) = scan_roof_slopes(conn) {
        if total > 0 {
            return if pitched > 0 {
                DwellingClass::EengezinswoningMetKap
            } else {
                DwellingClass::EengezinswoningPlatdak
            };
        }
    }

    // 3. Kap-indicatie uit de BuildingDesignConditions-velden.
    if vabi_enum_set(with_hood) {
        return DwellingClass::EengezinswoningMetKap;
    }
    if vabi_enum_set(without_hood) {
        return DwellingClass::EengezinswoningPlatdak;
    }

    // 4. Niet afleidbaar → conservatieve fallback mét warning.
    push_warning(
        warnings,
        "Woningklasse (ISSO 51:2023 Tabel 2.8) niet af te leiden uit de Vabi-data \
         (geen dakvlakken of kap-indicatie gevonden) — conservatieve aanname \
         'eengezinswoning met kap' (q_i,spec = 1,0 dm³/(s·m²)); controleer dit \
         in de gebouwinstellingen."
            .to_string(),
    );
    DwellingClass::EengezinswoningMetKap
}

/// Tel hellende en totale dakvlakken in de Vabi-DB.
///
/// Retourneert `Some((hellend, totaal))` of `None` wanneer de query faalt
/// (bv. afwijkend schema in een oudere Vabi-versie) — de caller valt dan
/// terug op de overige afleidingsstappen.
fn scan_roof_slopes(conn: &Connection) -> Option<(i64, i64)> {
    let mut stmt = conn
        .prepare(
            // Hellend = slope tussen de vlakke banden in (0 = horizontaal
            // onder, 180 = horizontaal boven). Robuust voor beide mogelijke
            // slope-conventies (vanaf horizontaal of vanaf nadir).
            "SELECT
                COALESCE(SUM(CASE WHEN fge.Slope > 5.0 AND fge.Slope < 175.0 THEN 1 ELSE 0 END), 0),
                COUNT(*)
             FROM BuildingPart bp
             JOIN Face f ON f.ID = bp.FaceID
             JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
             WHERE bp.BuildingPartType IN ('Roof', 'FlatRoof') AND bp.IsVirtual = 0",
        )
        .ok()?;

    stmt.query_row([], |row| {
        let pitched: i64 = row.get(0)?;
        let total: i64 = row.get(1)?;
        Ok((pitched, total))
    })
    .ok()
}

/// Map climate conditions from ClimateHeatLossCalculation table.
pub fn map_climate(conn: &Connection) -> Result<DesignConditions> {
    let mut stmt = conn
        .prepare(
            "SELECT DesignOutsideTemperatureWinter
             FROM ClimateHeatLossCalculation
             LIMIT 1"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Climate query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Climate query execution failed: {}", e))
    })?;

    let theta_e = if let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Climate row fetch failed: {}", e))
    })? {
        let design_temp: Option<f64> = row.get(0).unwrap_or(None);
        design_temp.unwrap_or(-10.0) // default for Netherlands
    } else {
        -10.0 // default for Netherlands
    };

    Ok(DesignConditions {
        theta_e,
        ..DesignConditions::default()
    })
}

/// Map ventilation configuration from Ventilation and related tables.
pub fn map_ventilation(conn: &Connection) -> Result<VentilationConfig> {
    // Join Ventilation with its optional LocalHeatRecoverySystemX (WTW).
    // Take the first row — for residential projects there's typically one
    // building-level ventilation config. Per-room overrides are Fase 4 werk.
    let mut stmt = conn
        .prepare(
            "SELECT
                v.SupplySource,
                v.CirculationRateMethod2017,
                v.LocalHeatRecoverySystemXID,
                hr.ValueBasedOnUnit
             FROM Ventilation v
             LEFT JOIN LocalHeatRecoverySystemX hr ON hr.ID = v.LocalHeatRecoverySystemXID
             LIMIT 1"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Ventilation query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Ventilation query execution failed: {}", e))
    })?;

    let (system_type, has_heat_recovery, hr_efficiency) = if let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Ventilation row fetch failed: {}", e))
    })? {
        let supply_source: Option<String> = row.get(0).unwrap_or(None);
        let circulation_method: Option<String> = row.get(1).unwrap_or(None);
        let wtw_id: Option<i64> = row.get(2).unwrap_or(None);
        let wtw_efficiency: Option<f64> = row.get(3).unwrap_or(None);

        let has_hr = wtw_id.is_some();

        // WTW (heat recovery) implies balanced ventilation = SystemD,
        // regardless of SupplySource value. Vabi gebruikt vaak generieke
        // strings als 'UserDefined' / 'AccordingToSystemType' die geen
        // duidelijk supply/exhaust pattern aangeven, maar WTW vereist
        // mechanische in- én uitlucht (anders heeft warmteterugwinning
        // geen zin).
        let system_type = if has_hr {
            VentilationSystemType::SystemD
        } else {
            map_ventilation_system_type(supply_source.as_deref(), circulation_method.as_deref())
        };

        (system_type, has_hr, wtw_efficiency)
    } else {
        // No ventilation data found - use defaults
        (VentilationSystemType::SystemC, false, None)
    };

    Ok(VentilationConfig {
        system_type,
        has_heat_recovery,
        heat_recovery_efficiency: hr_efficiency,
        frost_protection: None,
        supply_temperature: None,
        has_preheating: false,
        preheating_temperature: None,
    })
}

/// Map Vabi ventilation system types to ISSO 51 enum values.
///
/// This is a simplified mapping for Phase 1. Complex scenarios that don't map
/// cleanly will fall back to MechanicalExhaust and log a TODO comment.
fn map_ventilation_system_type(supply_source: Option<&str>, circulation_method: Option<&str>) -> VentilationSystemType {
    match (supply_source, circulation_method) {
        // System C: Natural supply, mechanical exhaust
        (Some("Natural"), Some("Mechanical")) => VentilationSystemType::SystemC,

        // System A: Natural supply, natural exhaust
        (Some("Natural"), _) => VentilationSystemType::SystemA,

        // System B: Mechanical supply, natural exhaust
        (Some("Mechanical"), Some("Natural")) => VentilationSystemType::SystemB,

        // System D: Mechanical supply, mechanical exhaust (balanced)
        (Some("Mechanical"), Some("Mechanical")) => VentilationSystemType::SystemD,

        // TODO for Phase 3: map more complex Vabi configurations
        // For now, fall back to mechanical exhaust (most common in NL residential)
        _ => {
            // TODO: log this mapping for Phase 3 improvement
            VentilationSystemType::SystemC // Mechanical exhaust default
        }
    }
}

/// Map constructions for a specific room.
///
/// Backward-compatible wrapper around [`map_constructions_per_room_with_warnings`]
/// (used by the V2 importer in `crates/vabi-importer`, which drops
/// `ground_params` during its V1→V2 conversion). Uses the Dutch design
/// defaults θ_i = 20 °C / θ_e = −10 °C for the f_g2-derivation and discards
/// warnings. New code should prefer the `_with_warnings` variant with the
/// actual room/climate temperatures.
pub fn map_constructions_per_room(conn: &Connection, room_id: i64, _room_cell_id: i64) -> Result<Vec<ConstructionElement>> {
    map_constructions_per_room_with_warnings(conn, room_id, 20.0, -10.0, &mut Vec::new())
}

/// Map constructions for a specific room, with warnings and temperatures.
///
/// Links Room to BuildingParts via the cell-based geometry system:
/// Room.CellID → MainFace → CellFace → BuildingPart
/// Then extracts area, U-value, boundary type, perimeter and the winter
/// boundary temperature (`BoundaryConditions.BoundaryTemperaturesWinterID →
/// BoundaryTemperatures.TemperatureDay`) for each BuildingPart.
///
/// # Arguments
/// * `theta_i` — design indoor temperature of this room in °C (drives f_g2).
/// * `theta_e` — design outdoor temperature in °C (drives f_g2).
/// * `warnings` — non-fatal import warnings (deduplicated).
pub fn map_constructions_per_room_with_warnings(
    conn: &Connection,
    room_id: i64,
    theta_i: f64,
    theta_e: f64,
    warnings: &mut Vec<String>,
) -> Result<Vec<ConstructionElement>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                bp.ID,
                bp.BuildingPartType,
                bp.HasConstruction,
                bp.IsVirtual,
                bp.ConstructionID,
                bp.BoundaryConditionsID,
                fge.Area,
                fge.Slope,
                bc.Type as BoundaryType,
                COALESCE(cd.Type, 'Unknown') as ConstructionType,
                bp.PsiThermalBridge,
                fge.Perimeter,
                btw.TemperatureDay as BoundaryTempWinter
             FROM Room r
             JOIN MainFace mf ON mf.CellID = r.CellID
             JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
             JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
             JOIN Face f ON f.ID = bp.FaceID
             JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
             LEFT JOIN BoundaryConditions bc ON bc.ID = bp.BoundaryConditionsID
             LEFT JOIN BoundaryTemperatures btw ON btw.ID = bc.BoundaryTemperaturesWinterID
             LEFT JOIN Construction c ON c.ID = bp.ConstructionID
             LEFT JOIN ConstructionData cd ON cd.ID = c.DataID
             WHERE r.ID = ? AND bp.HasConstruction = 1 AND bp.IsVirtual = 0
             ORDER BY bp.BuildingPartType, bp.ID"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Construction query failed: {}", e)))?;

    let mut rows = stmt.query([room_id]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Construction query execution failed: {}", e))
    })?;

    let mut constructions = Vec::new();
    let mut construction_counter = 1;

    while let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Construction row fetch failed: {}", e))
    })? {
        let building_part_id: i64 = row.get(0).unwrap_or(0);
        let part_type: String = row.get(1).unwrap_or_else(|_| "Unknown".to_string());
        let has_construction: bool = row.get(2).unwrap_or(false);
        let _is_virtual: bool = row.get(3).unwrap_or(false);
        let construction_id: Option<i64> = row.get(4).unwrap_or(None);
        let _boundary_conditions_id: Option<i64> = row.get(5).unwrap_or(None);
        let area: f64 = row.get(6).unwrap_or(0.0);
        let slope: f64 = row.get(7).unwrap_or(90.0);
        let boundary_type_str: Option<String> = row.get(8).unwrap_or(None);
        let construction_type: String = row.get(9).unwrap_or_else(|_| "Unknown".to_string());
        let psi_thermal_bridge: Option<f64> = row.get(10).unwrap_or(None);
        let perimeter: Option<f64> = row.get(11).unwrap_or(None);
        let boundary_temp_winter: Option<f64> = row.get(12).unwrap_or(None);

        // Skip if no construction or area is zero
        if !has_construction || construction_id.is_none() || area <= 0.0 {
            continue;
        }

        let construction_id = construction_id.unwrap();

        // Compute U-value based on construction type
        let u_value = if construction_type.contains("Window") || construction_type.contains("Door") {
            compute_u_window(conn, construction_id).unwrap_or(2.5) // fallback for transparent
        } else {
            compute_u_value(conn, construction_id, slope).unwrap_or(0.5) // fallback for opaque
        };

        // Map boundary type
        let boundary_type = map_boundary_type(boundary_type_str.as_deref(), warnings);

        // Generate description
        let description = format!("{} {}", part_type, construction_counter);
        construction_counter += 1;

        // Vabi populates PsiThermalBridge per BuildingPart. If > 0, use it
        // as a per-element ΔU_TB override; if 0/None, fall back to forfaitaire.
        let use_forfaitaire_thermal_bridge =
            psi_thermal_bridge.map(|p| p <= 0.0).unwrap_or(true);
        let custom_delta_u_tb = psi_thermal_bridge.filter(|&p| p > 0.0);

        // Temperatuurcorrectiefactor f_k — alleen voor Exterior is f_k = 1.0
        // een norm-waarde (ISSO 51:2023 formule 4.3a). Voor alle andere
        // grensvlakken laten we `None` staan zodat de rekenkern de factor
        // norm-conform bepaalt:
        // - UnheatedSpace  → f_k-default 0,5 (§2.5.2, `h_t_unheated_element`)
        // - AdjacentRoom   → formule 2.17/4.6 met θ_a (`h_t_adjacent_room_element`)
        // - AdjacentBuilding → formule 4.15–4.17 met θ_b (`h_t_adjacent_building_element`)
        // - Ground         → ground_params-route (formule 4.18), f_k ongebruikt
        // De eerdere onvoorwaardelijke `Some(1.0)` liet binnen-/buurwanden als
        // volle buitenschil meetellen (audit §2.2 bevinding 1).
        let temperature_factor = if boundary_type == BoundaryType::Exterior {
            Some(1.0)
        } else {
            None
        };

        // Aangrenzende-ruimte temperatuur uit de Vabi-DB. De importer kan
        // (nog) geen `adjacent_room_id`-koppeling leggen (cell-coupling is
        // Fase 3); het legacy-veld `adjacent_temperature` is exact voor dit
        // doel als fallback-route in de rekenkern behouden — zie de
        // doc-comment op `ConstructionElement::adjacent_temperature`.
        let adjacent_temperature = if boundary_type == BoundaryType::AdjacentRoom {
            if boundary_temp_winter.is_none() {
                push_warning(
                    warnings,
                    "Binnenwand-vlak(ken) zonder Vabi-grenstemperatuur (BoundaryTemperatures) — \
                     aangrenzende temperatuur onbekend; de rekenkern neemt ΔT = 0 (0 W) aan \
                     voor die vlakken."
                        .to_string(),
                );
            }
            boundary_temp_winter
        } else {
            None
        };

        // Grondparameters voor formule 4.18 — zonder deze parameters levert
        // `h_t_ground_element` stil 0 W op (audit §2.2 bevinding 2).
        let ground_params = if boundary_type == BoundaryType::Ground {
            Some(derive_ground_params(
                area,
                perimeter,
                slope,
                u_value,
                use_forfaitaire_thermal_bridge,
                custom_delta_u_tb,
                boundary_temp_winter,
                theta_i,
                theta_e,
                warnings,
            ))
        } else {
            None
        };

        let construction = ConstructionElement {
            id: format!("bp_{}", building_part_id),
            description,
            area,
            u_value,
            boundary_type,
            material_type: MaterialType::Masonry, // Default - could be improved by analyzing materials
            temperature_factor,
            adjacent_room_id: None,
            adjacent_temperature,
            vertical_position: match part_type.as_str() {
                "Floor" => VerticalPosition::Floor,
                "Roof" | "FlatRoof" => VerticalPosition::Ceiling,
                _ => VerticalPosition::Wall,
            },
            use_forfaitaire_thermal_bridge,
            custom_delta_u_tb,
            ground_params,
            has_embedded_heating: false,
            catalog_ref: None,
            uw_breakdown: None,
        };

        constructions.push(construction);
    }

    Ok(constructions)
}

// ---------------------------------------------------------------------------
// Grondparameters (ISSO 51:2023 §2.5.5, formule 4.18)
// ---------------------------------------------------------------------------

/// Ondergrens voor de equivalente warmtedoorgangscoëfficiënt U_e,k.
/// ISSO 53 §4.6 — `U_equiv,k ≥ 0,1 W/(m²·K)` (zelfde grondmodel als
/// ISSO 51:2023 Figuur 4.2).
const U_EQUIV_MIN: f64 = 0.1;

/// Clamp-grenzen voor de geometrische hulpwaarde B' = 2·A/O.
/// ISSO 53 §4.6: `2 ≤ B' ≤ 50`.
const B_PRIME_MIN: f64 = 2.0;
const B_PRIME_MAX: f64 = 50.0;

/// Equivalente U-waarde U_e voor een grondvloer op maaiveld (z = 0).
///
/// Curve-fit van [`crate::formulas::ISSO_51_2023_FIGUUR4_2`] (U_e als functie
/// van B' en de vloerweerstand). De gebruikte kwotiëntvorm en parameters zijn
/// de in deze repo geverifieerde ISSO 53 formule 4.24 / tabel 4.3
/// (vloer-parameterset) — zie `crates/isso53-core/src/calc/ground.rs`
/// (worked example p.65: U_k = 2,43, B' ≈ 4,1 → U_e ≈ 0,177 ✓) en de
/// Vabi-cross-validatie in `tests/verification/isso53_*`. ISSO 51 en ISSO 53
/// delen hetzelfde grondmodel (`H_T,ig = 1,45 × …`, formule 4.18 resp. 4.21).
///
/// `z = 0` omdat de geparste Vabi-structuren geen vloerdiepte onder maaiveld
/// bevatten; voor een reguliere grondvloer is dat de norm-aanname. De
/// `c₃·z^n₃`-term valt daarmee weg (n₃ > 0 voor vloeren).
///
/// # Arguments
/// * `area` — vloeroppervlak A in m²
/// * `perimeter` — omtrek O in m
/// * `u_construction` — U_k inclusief ΔU_TB in W/(m²·K)
///
/// # Returns
/// `Some(U_e)` (geclampt op minimaal 0,1), of `None` bij ongeldige invoer.
fn u_equivalent_ground_floor(area: f64, perimeter: f64, u_construction: f64) -> Option<f64> {
    if area <= 0.0 || perimeter <= 0.0 || u_construction <= 0.0 {
        return None;
    }

    let b_prime = (2.0 * area / perimeter).clamp(B_PRIME_MIN, B_PRIME_MAX);

    // Vloer-parameterset (ISSO 53 tabel 4.3, PDF p.44).
    const A: f64 = 0.9671;
    const B: f64 = -7.455;
    const C1: f64 = 10.76;
    const N1: f64 = 0.5532;
    const C2: f64 = 9.773;
    const N2: f64 = 0.6027;
    const D: f64 = -0.0203;

    // Kwotiëntvorm: U_e = |a·b| / (c₁·B'^n₁ + c₂·U_k^n₂ + c₃·0^n₃ + d).
    let denom = C1 * b_prime.powf(N1) + C2 * u_construction.powf(N2) + D;
    if denom.abs() < 1e-9 {
        return Some(U_EQUIV_MIN);
    }

    Some(((A * B).abs() / denom).max(U_EQUIV_MIN))
}

/// Leid [`GroundParameters`] af voor een grondvlak uit de beschikbare
/// Vabi-data (vloerafmetingen, U_k, winter-grondtemperatuur).
///
/// ISSO 51:2023 §2.5.5, formule (4.18):
/// `H_T,ig = 1,45 × G_w × Σ(A_k × f_g2 × U_e,k)`.
///
/// - **U_e,k**: voor vloeren via de Figuur 4.2-curve ([`u_equivalent_ground_floor`],
///   B' = 2·A/O uit `FaceGeometryEngine.Area/Perimeter`, z = 0). Waar dat
///   niet kan (grondwand zonder z-diepte, ontbrekende omtrek) → conservatief
///   `U_e = U_k` (géén gronddemping) mét import-warning — nooit stil 0 W.
/// - **f_g2**: uit de Vabi-grondtemperatuur (`BoundaryTemperatures
///   .TemperatureDay`, winter): `f_g2 = (θ_i − θ_grond) / (θ_i − θ_e)`,
///   geclampt op [0, 1]. Zonder grondtemperatuur → conservatief 1,0 + warning.
/// - **G_w**: 1,0 (grondwater ≥ 1 m onder vloer; niet in de Vabi-data).
#[allow(clippy::too_many_arguments)]
fn derive_ground_params(
    area: f64,
    perimeter: Option<f64>,
    slope: f64,
    u_value: f64,
    use_forfaitaire_thermal_bridge: bool,
    custom_delta_u_tb: Option<f64>,
    boundary_temp_winter: Option<f64>,
    theta_i: f64,
    theta_e: f64,
    warnings: &mut Vec<String>,
) -> GroundParameters {
    // U_k inclusief ΔU_TB — formule 4.24-conventie (zelfde forfaitair/custom-
    // prioriteit als de rekenkern, zie `tables::thermal_bridge::delta_u_tb`).
    let u_k = u_value
        + thermal_bridge::delta_u_tb(use_forfaitaire_thermal_bridge, custom_delta_u_tb);

    // Vloer vs. wand: zelfde slope-conventie als `get_surface_resistances`
    // (0 = vloer, 90 = wand).
    let is_floor = slope < 30.0;

    let u_equivalent = if is_floor {
        match perimeter.and_then(|p| u_equivalent_ground_floor(area, p, u_k)) {
            Some(u_e) => u_e,
            None => {
                push_warning(
                    warnings,
                    "Grondvloer zonder bruikbare omtrek-/U-gegevens — U_e niet bepaalbaar \
                     via ISSO 51:2023 Figuur 4.2; conservatief U_e = U_k (geen gronddemping) \
                     gebruikt."
                        .to_string(),
                );
                u_k
            }
        }
    } else {
        // Grondwand/kelderwand: de U_e-curve vereist een z-diepte onder
        // maaiveld die niet in de geparste Vabi-structuren zit.
        push_warning(
            warnings,
            "Grondwand: z-diepte onder maaiveld niet beschikbaar in de Vabi-data — \
             U_e niet bepaalbaar via ISSO 51:2023 Figuur 4.2; conservatief U_e = U_k \
             (geen gronddemping) gebruikt."
                .to_string(),
        );
        u_k
    };

    let denom = theta_i - theta_e;
    let fg2 = match boundary_temp_winter {
        Some(theta_g) if denom.abs() > 1e-9 => ((theta_i - theta_g) / denom).clamp(0.0, 1.0),
        _ => {
            push_warning(
                warnings,
                "Grondvlak zonder Vabi-grondtemperatuur (BoundaryTemperatures) — \
                 f_g2 = 1,0 (conservatief, volle ΔT) gebruikt."
                    .to_string(),
            );
            1.0
        }
    };

    GroundParameters {
        u_equivalent,
        ground_water_factor: 1.0,
        fg2,
    }
}

/// Compute U-value for opaque constructions.
///
/// First tries StandardConstruction.RcValue, then falls back to layered calculation.
/// Uses R_si/R_se values based on slope per ISO 6946.
fn compute_u_value(conn: &Connection, construction_id: i64, slope: f64) -> Result<f64> {
    // First try StandardConstruction.RcValue
    let rc_value = get_standard_rc_value(conn, construction_id)?;

    if let Some(rc) = rc_value {
        if rc > 0.0 {
            let (r_si, r_se) = get_surface_resistances(slope);
            return Ok(1.0 / (r_si + rc + r_se));
        }
    }

    // Fall back to layered calculation
    let mut stmt = conn
        .prepare(
            "SELECT cl.Thickness, md.HeatConductivity, md.HeatResistance
             FROM Construction c
             JOIN ConstructionData cd ON cd.ID = c.DataID
             JOIN OpaqueConstructionData ocd ON ocd.ID = cd.OpaqueConstructionDataID
             JOIN ConstructionLayer cl ON cl.LayeredConstructionID = ocd.LayeredConstructionID
             JOIN Material m ON m.ID = cl.MaterialID
             JOIN MaterialData md ON md.ID = m.DataID
             WHERE c.ID = ? AND ocd.IsLayered = 1
             ORDER BY cl.SortNumber"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Layer query failed: {}", e)))?;

    let mut rows = stmt.query([construction_id]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Layer query execution failed: {}", e))
    })?;

    let mut r_total = 0.0;
    let mut layer_count = 0;

    while let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Layer row fetch failed: {}", e))
    })? {
        let thickness_mm: f64 = row.get(0).unwrap_or(0.0);
        let heat_conductivity: f64 = row.get(1).unwrap_or(0.0);
        let heat_resistance: f64 = row.get(2).unwrap_or(0.0);

        let r_layer = if heat_conductivity > 0.0 {
            (thickness_mm * 1e-3) / heat_conductivity
        } else {
            heat_resistance // Direct resistance for air gaps
        };

        r_total += r_layer;
        layer_count += 1;
    }

    if layer_count == 0 {
        return Err(Isso51Error::VabiImport(
            "No construction layers found".to_string(),
        ));
    }

    let (r_si, r_se) = get_surface_resistances(slope);
    let u_value = 1.0 / (r_si + r_total + r_se);

    Ok(u_value)
}

/// Get StandardConstruction.RcValue if available.
fn get_standard_rc_value(conn: &Connection, construction_id: i64) -> Result<Option<f64>> {
    let mut stmt = conn
        .prepare(
            "SELECT sc.RcValue
             FROM Construction c
             JOIN ConstructionData cd ON cd.ID = c.DataID
             JOIN OpaqueConstructionData ocd ON ocd.ID = cd.OpaqueConstructionDataID
             JOIN StandardConstruction sc ON sc.ID = ocd.StandardConstructionID
             WHERE c.ID = ? AND sc.RcValue > 0"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("StandardConstruction query failed: {}", e)))?;

    match stmt.query_row([construction_id], |row| {
        let rc_value: f64 = row.get(0)?;
        Ok(rc_value)
    }) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Isso51Error::VabiSqliteError(format!("StandardConstruction query failed: {}", e))),
    }
}

/// Compute U-value for transparent constructions (windows).
///
/// Uses weighted average of frame and glazing U-values.
/// TODO: Add psi-term correction for glass-frame interface.
fn compute_u_window(conn: &Connection, construction_id: i64) -> Result<f64> {
    let mut stmt = conn
        .prepare(
            "SELECT f.U as FrameU, g.U as GlazingU, tcd.FramePercentage
             FROM Construction c
             JOIN ConstructionData cd ON cd.ID = c.DataID
             JOIN TransparentConstructionData tcd ON tcd.ID = cd.TransparentConstructionDataID
             LEFT JOIN Frame f ON f.ID = tcd.FrameID
             LEFT JOIN StandardWindow sw ON sw.ID = tcd.StandardWindowID
             LEFT JOIN Glazing g ON g.ID = sw.GlazingID
             WHERE c.ID = ?"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Window query failed: {}", e)))?;

    match stmt.query_row([construction_id], |row| {
        let frame_u: Option<f64> = row.get(0).unwrap_or(None);
        let glazing_u: Option<f64> = row.get(1).unwrap_or(None);
        let frame_percentage: Option<f64> = row.get(2).unwrap_or(None);
        Ok((frame_u, glazing_u, frame_percentage))
    }) {
        Ok((Some(frame_u), Some(glazing_u), Some(frame_pct))) => {
            // Weighted average: U = frame_pct * U_frame + (1 - frame_pct) * U_glazing
            let u_window = (frame_pct / 100.0) * frame_u + (1.0 - frame_pct / 100.0) * glazing_u;
            Ok(u_window)
        }
        Ok((Some(frame_u), Some(glazing_u), None)) => {
            // Assume 30% frame percentage if not specified
            let u_window = 0.3 * frame_u + 0.7 * glazing_u;
            Ok(u_window)
        }
        Ok((Some(frame_u), None, _)) => Ok(frame_u), // Frame only
        Ok((None, Some(glazing_u), _)) => Ok(glazing_u), // Glazing only
        Ok((None, None, _)) => Ok(2.5), // Fallback for missing data
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(2.5), // Fallback for no data
        Err(e) => Err(Isso51Error::VabiSqliteError(format!("Window query failed: {}", e))),
    }
}

/// Get surface resistances R_si and R_se based on slope.
///
/// Values from ISO 6946 based on heat flow direction and boundary conditions.
fn get_surface_resistances(slope: f64) -> (f64, f64) {
    if slope < 30.0 {
        // Floor (heat flow down)
        (0.17, 0.04)
    } else if slope > 150.0 {
        // Roof/ceiling (heat flow up)
        (0.10, 0.04)
    } else {
        // Vertical wall
        (0.13, 0.04)
    }
}

/// Map Vabi boundary type to our BoundaryType enum.
///
/// Onbekende of ontbrekende types vallen terug op `Exterior` (volle ΔT,
/// conservatief) — maar nooit meer **stil**: elk onbekend type levert een
/// import-warning op (audit §2.2 bevinding 5).
fn map_boundary_type(vabi_type: Option<&str>, warnings: &mut Vec<String>) -> BoundaryType {
    match vabi_type {
        Some("OutsideAir") => BoundaryType::Exterior,
        Some("Ground") => BoundaryType::Ground,
        Some("AdjacentRoom") => BoundaryType::AdjacentRoom,
        Some("AdjacentBuilding") => BoundaryType::AdjacentBuilding,
        // V2 Batch 1 extensions:
        Some("CrawlSpace") => BoundaryType::Ground,
        Some("OtherBuilding") => BoundaryType::AdjacentBuilding,
        Some("InternalSpace") => BoundaryType::AdjacentRoom,
        // Onverwarmde aangrenzende ruimte → ISSO 51:2023 §2.5.2 (f_k-route).
        // De eerdere `_ => Exterior`-fallback liet deze vlakken met volle ΔT
        // als buitenschil meetellen.
        Some("UnconditionedSpace") | Some("AdjacentUnheatedSpace") => BoundaryType::UnheatedSpace,
        Some(other) => {
            push_warning(
                warnings,
                format!(
                    "Onbekend Vabi-grensvlaktype '{other}' — als buitenlucht (Exterior, \
                     volle ΔT) gemapt; controleer deze vlakken handmatig."
                ),
            );
            BoundaryType::Exterior
        }
        None => {
            push_warning(
                warnings,
                "Grensvlak zonder BoundaryConditions-type in de Vabi-data — als \
                 buitenlucht (Exterior, volle ΔT) gemapt; controleer deze vlakken handmatig."
                    .to_string(),
            );
            BoundaryType::Exterior
        }
    }
}

/// Calculate room height from volume and floor area.
///
/// Room.VolumeInfoID → VariantVolumeInfo → TypedVolumeData with Type='InternalDimensionsIncludingPlenum'
/// height = volume / floor_area
pub fn calculate_room_height(conn: &Connection, room_id: i64, floor_area: f64) -> Result<f64> {
    if floor_area <= 0.0 {
        return Ok(2.7); // Fallback height
    }

    let mut stmt = conn
        .prepare(
            "SELECT tvd.Volume
             FROM Room r
             JOIN VariantVolumeInfo vvi ON vvi.VolumeInfoID = r.VolumeInfoID
             JOIN TypedVolumeData tvd ON tvd.VariantVolumeInfoID = vvi.ID
             WHERE r.ID = ? AND tvd.Type = 'InternalDimensionsIncludingPlenum'"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Room volume query failed: {}", e)))?;

    match stmt.query_row([room_id], |row| {
        let volume: f64 = row.get(0)?;
        Ok(volume)
    }) {
        Ok(volume) if volume > 0.0 => Ok(volume / floor_area),
        _ => Ok(2.7), // Fallback height
    }
}

/// Map room data from Room table and related temperature settings.
///
/// `Room.RoomRequirementsID` is an **AspectID**, mirroring the Building pattern.
/// Chain: AspectID → VarAsp_RoomRequirementsData → RoomRequirementsTemplate.DataID →
/// RoomRequirementsData.ID → ConditionsID → RoomDesignConditions →
/// DesignTemperaturesWinterID → DesignTemperatures.TemperatureDay.
/// See `docs/vabi-schema-reference.md` for the full join.
///
/// **PHASE 2**: Now populates constructions and calculates floor_area and height.
///
/// `theta_e` (ontwerp-buitentemperatuur uit `map_climate`) wordt samen met de
/// per-kamer θ_i doorgegeven aan de constructie-mapping voor de
/// f_g2-afleiding van grondvlakken (ISSO 51:2023 §2.5.5).
fn map_rooms(conn: &Connection, theta_e: f64, warnings: &mut Vec<String>) -> Result<Vec<Room>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                r.ID,
                r.CellID,
                r.RoomNumber,
                r.Name,
                r.RoomRequirementsID,
                dt.TemperatureDay
             FROM Room r
             JOIN Project p ON r.ProjectVersionID = p.CurrentProjectVersionID
             LEFT JOIN VarAsp_RoomRequirementsData var ON var.AspectID = r.RoomRequirementsID
             LEFT JOIN RoomRequirementsTemplate rrt ON rrt.ID = var.TemplateID
             LEFT JOIN RoomRequirementsData rrd ON rrd.ID = rrt.DataID
             LEFT JOIN RoomDesignConditions rdc ON rdc.ID = rrd.ConditionsID
             LEFT JOIN DesignTemperatures dt ON dt.ID = rdc.DesignTemperaturesWinterID
             WHERE r.UseInCalculations = 1
             ORDER BY r.RoomNumber"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Room query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Room query execution failed: {}", e))
    })?;

    let mut rooms = Vec::new();

    while let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Room row fetch failed: {}", e))
    })? {
        let room_id: i64 = row.get(0).unwrap_or(0);
        let _room_cell_id: i64 = row.get(1).unwrap_or(0);
        let room_number: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());
        let room_name: String = row.get(3).unwrap_or_else(|_| "Unnamed Room".to_string());
        let _requirements_id: Option<i32> = row.get(4).unwrap_or(None);
        let theta_i: Option<f64> = row.get(5).unwrap_or(None);

        // Use temperature from joined table or default
        let theta_i = theta_i.unwrap_or(20.0);

        // Map constructions for this room. Propagate errors instead of silently
        // swallowing — query bugs were masked here in early Phase 2.
        let constructions =
            map_constructions_per_room_with_warnings(conn, room_id, theta_i, theta_e, warnings)?;

        // Calculate floor area from Floor BuildingParts
        let floor_area: f64 = constructions
            .iter()
            .filter(|c| c.vertical_position == VerticalPosition::Floor)
            .map(|c| c.area)
            .sum();

        // Calculate room height
        let height = calculate_room_height(conn, room_id, floor_area).unwrap_or(2.7);

        let room = Room {
            id: room_number.clone(),
            name: room_name,
            function: RoomFunction::LivingRoom, // TODO: map from Vabi room types in Phase 3
            custom_temperature: None,
            floor_area,
            height,
            constructions,
            heating_system: HeatingSystem::RadiatorLt, // TODO: map from Vabi data in Phase 3
            ventilation_rate: None, // TODO: calculate from Vabi data
            has_mechanical_exhaust: false, // TODO: map from system configuration
            has_mechanical_supply: false,  // TODO: map from system configuration
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            air_source_room_id: None,
            internal_air_temperature: Some(theta_i),
            clamp_positive: true,
        };

        rooms.push(room);
    }

    if rooms.is_empty() {
        return Err(Isso51Error::VabiImport(
            "No rooms found in Vabi database".to_string(),
        ));
    }

    Ok(rooms)
}

#[cfg(test)]
mod tests {
    //! Unit-tests voor de audit §2.2-fixes op de Vabi-mapper, tegen een
    //! in-memory SQLite met het (minimale) Vabi-schema uit
    //! `docs/vabi-schema-reference.md`. De integratietests tegen echte
    //! `.vp`-bestanden staan in `tests/vabi_import_test.rs` (skippen als de
    //! gitignored referentiebestanden ontbreken).

    use super::*;
    use crate::calc::transmission::{
        h_t_adjacent_room_element, h_t_ground_element, h_t_unheated_element,
    };
    use rusqlite::params;

    // ------------------------------------------------------------------
    // In-memory Vabi-schema fixture
    // ------------------------------------------------------------------

    /// Bouw een in-memory SQLite met alle tabellen/kolommen die de
    /// mapper-queries raken (kolomnamen conform `docs/vabi-schema-reference.md`).
    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite");
        conn.execute_batch(
            "CREATE TABLE Room (ID INTEGER, CellID INTEGER);
             CREATE TABLE MainFace (CellID INTEGER, CellFaceID INTEGER);
             CREATE TABLE CellFace (FaceID INTEGER, BuildingPartID INTEGER);
             CREATE TABLE BuildingPart (
                 ID INTEGER, BuildingPartType TEXT, HasConstruction INTEGER,
                 IsVirtual INTEGER, ConstructionID INTEGER,
                 BoundaryConditionsID INTEGER, FaceID INTEGER,
                 PsiThermalBridge REAL);
             CREATE TABLE Face (ID INTEGER, FaceGeometryEngineID INTEGER);
             CREATE TABLE FaceGeometryEngine (
                 ID INTEGER, Area REAL, Slope REAL, Perimeter REAL);
             CREATE TABLE BoundaryConditions (
                 ID INTEGER, Type TEXT, BoundaryTemperaturesWinterID INTEGER);
             CREATE TABLE BoundaryTemperatures (ID INTEGER, TemperatureDay REAL);
             CREATE TABLE Construction (ID INTEGER, DataID INTEGER);
             CREATE TABLE ConstructionData (
                 ID INTEGER, Type TEXT, OpaqueConstructionDataID INTEGER,
                 TransparentConstructionDataID INTEGER);
             CREATE TABLE OpaqueConstructionData (
                 ID INTEGER, IsLayered INTEGER, StandardConstructionID INTEGER,
                 LayeredConstructionID INTEGER);
             CREATE TABLE StandardConstruction (ID INTEGER, RcValue REAL);
             CREATE TABLE ConstructionLayer (
                 LayeredConstructionID INTEGER, Thickness REAL,
                 MaterialID INTEGER, SortNumber INTEGER);
             CREATE TABLE Material (ID INTEGER, DataID INTEGER);
             CREATE TABLE MaterialData (
                 ID INTEGER, HeatConductivity REAL, HeatResistance REAL);
             CREATE TABLE TransparentConstructionData (
                 ID INTEGER, FrameID INTEGER, StandardWindowID INTEGER,
                 FramePercentage REAL);
             CREATE TABLE Frame (ID INTEGER, U REAL);
             CREATE TABLE StandardWindow (ID INTEGER, GlazingID INTEGER);
             CREATE TABLE Glazing (ID INTEGER, U REAL);
             CREATE TABLE Project (
                 CurrentProjectVersionID INTEGER, ProjectDataID INTEGER,
                 Name TEXT, Description TEXT);
             CREATE TABLE Building (
                 ProjectVersionID INTEGER, RequirementsID INTEGER,
                 UsageArea REAL, NumberOfFloors INTEGER);
             CREATE TABLE VarAsp_BuildingRequirementsData (
                 AspectID INTEGER, TemplateID INTEGER);
             CREATE TABLE BuildingRequirementsTemplate (ID INTEGER, DataID INTEGER);
             CREATE TABLE BuildingRequirementsData (ID INTEGER, ConditionsID INTEGER);
             CREATE TABLE BuildingDesignConditions (
                 ID INTEGER, SpecificQv10 REAL, MeasuredQv10 REAL, Qv10Type TEXT,
                 BuildingShapeType TEXT, BuildingWithHoodType TEXT,
                 BuildingWithoutHoodType TEXT, MultiStoreyBuildingType TEXT,
                 CertaintyClass TEXT);",
        )
        .expect("schema aanmaken");
        // Eén testkamer met CellID 100 — alle parts hangen hieraan.
        conn.execute("INSERT INTO Room (ID, CellID) VALUES (1, 100)", [])
            .expect("room");
        conn
    }

    /// Voeg een opaque constructie met StandardConstruction-Rc toe.
    /// `compute_u_value` levert dan U = 1 / (R_si + Rc + R_se).
    fn insert_opaque_construction(conn: &Connection, id: i64, rc: f64) {
        conn.execute(
            "INSERT INTO StandardConstruction (ID, RcValue) VALUES (?1, ?2)",
            params![id, rc],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO OpaqueConstructionData
                (ID, IsLayered, StandardConstructionID, LayeredConstructionID)
             VALUES (?1, 0, ?1, NULL)",
            params![id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ConstructionData
                (ID, Type, OpaqueConstructionDataID, TransparentConstructionDataID)
             VALUES (?1, 'Wall', ?1, NULL)",
            params![id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Construction (ID, DataID) VALUES (?1, ?1)",
            params![id],
        )
        .unwrap();
    }

    /// Voeg een BuildingPart inclusief geometrie en (optionele) boundary toe,
    /// gekoppeld aan de testkamer (CellID 100).
    #[allow(clippy::too_many_arguments)]
    fn insert_part(
        conn: &Connection,
        part_id: i64,
        part_type: &str,
        area: f64,
        slope: f64,
        perimeter: f64,
        bc_type: Option<&str>,
        boundary_temp: Option<f64>,
        construction_id: i64,
    ) {
        conn.execute(
            "INSERT INTO FaceGeometryEngine (ID, Area, Slope, Perimeter)
             VALUES (?1, ?2, ?3, ?4)",
            params![part_id, area, slope, perimeter],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Face (ID, FaceGeometryEngineID) VALUES (?1, ?1)",
            params![part_id],
        )
        .unwrap();

        let bc_id: Option<i64> = bc_type.map(|t| {
            let bt_id: Option<i64> = boundary_temp.map(|temp| {
                conn.execute(
                    "INSERT INTO BoundaryTemperatures (ID, TemperatureDay) VALUES (?1, ?2)",
                    params![part_id, temp],
                )
                .unwrap();
                part_id
            });
            conn.execute(
                "INSERT INTO BoundaryConditions (ID, Type, BoundaryTemperaturesWinterID)
                 VALUES (?1, ?2, ?3)",
                params![part_id, t, bt_id],
            )
            .unwrap();
            part_id
        });

        conn.execute(
            "INSERT INTO BuildingPart
                (ID, BuildingPartType, HasConstruction, IsVirtual, ConstructionID,
                 BoundaryConditionsID, FaceID, PsiThermalBridge)
             VALUES (?1, ?2, 1, 0, ?3, ?4, ?1, NULL)",
            params![part_id, part_type, construction_id, bc_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO MainFace (CellID, CellFaceID) VALUES (100, ?1)",
            params![part_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO CellFace (FaceID, BuildingPartID) VALUES (?1, ?1)",
            params![part_id],
        )
        .unwrap();
    }

    /// Voeg de Building → BuildingDesignConditions Aspect/Template-keten toe.
    fn insert_building_chain(
        conn: &Connection,
        shape: &str,
        with_hood: Option<&str>,
        without_hood: Option<&str>,
        multi_storey: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO Project (CurrentProjectVersionID, ProjectDataID, Name, Description)
             VALUES (1, 1, 'Test', NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO Building (ProjectVersionID, RequirementsID, UsageArea, NumberOfFloors)
             VALUES (1, 500, 120.0, 2)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO VarAsp_BuildingRequirementsData (AspectID, TemplateID) VALUES (500, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO BuildingRequirementsTemplate (ID, DataID) VALUES (1, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO BuildingRequirementsData (ID, ConditionsID) VALUES (1, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO BuildingDesignConditions
                (ID, SpecificQv10, MeasuredQv10, Qv10Type, BuildingShapeType,
                 BuildingWithHoodType, BuildingWithoutHoodType,
                 MultiStoreyBuildingType, CertaintyClass)
             VALUES (1, 0.4, NULL, 'Specific', ?1, ?2, ?3, ?4, 'ClassB')",
            params![shape, with_hood, without_hood, multi_storey],
        )
        .unwrap();
    }

    fn find<'a>(elems: &'a [ConstructionElement], id: &str) -> &'a ConstructionElement {
        elems
            .iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("element {id} ontbreekt in {elems:?}"))
    }

    // ------------------------------------------------------------------
    // Fix 1 — temperature_factor per grensvlaktype
    // ------------------------------------------------------------------

    /// Audit §2.2 bevinding 1: `Some(1.0)` mag alleen op Exterior staan;
    /// alle andere grensvlakken moeten `None` krijgen zodat de rekenkern
    /// f_k norm-conform bepaalt (formule 2.17 / 4.6 / 4.10 / 4.14-4.17 / 4.18).
    #[test]
    fn temperature_factor_follows_boundary_type() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 12.0, 90.0, 14.0, Some("OutsideAir"), None, 10);
        insert_part(&conn, 2, "Wall", 10.0, 90.0, 13.0, Some("AdjacentRoom"), Some(15.0), 10);
        insert_part(&conn, 3, "Wall", 8.0, 90.0, 12.0, Some("UnconditionedSpace"), None, 10);
        insert_part(&conn, 4, "Wall", 18.0, 90.0, 17.0, Some("AdjacentBuilding"), Some(17.0), 10);
        insert_part(&conn, 5, "Floor", 50.0, 0.0, 30.0, Some("Ground"), Some(4.0), 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        assert_eq!(elems.len(), 5);

        let exterior = find(&elems, "bp_1");
        assert_eq!(exterior.boundary_type, BoundaryType::Exterior);
        assert_eq!(
            exterior.temperature_factor,
            Some(1.0),
            "Exterior: f_k = 1.0 (formule 4.3a)"
        );

        let adjacent = find(&elems, "bp_2");
        assert_eq!(adjacent.boundary_type, BoundaryType::AdjacentRoom);
        assert_eq!(
            adjacent.temperature_factor, None,
            "AdjacentRoom: f_ia via formule 2.17, niet hardcoded 1.0"
        );

        let unheated = find(&elems, "bp_3");
        assert_eq!(unheated.boundary_type, BoundaryType::UnheatedSpace);
        assert_eq!(
            unheated.temperature_factor, None,
            "UnheatedSpace: f_k-default 0.5 via rekenkern (§2.5.2)"
        );

        let neighbor = find(&elems, "bp_4");
        assert_eq!(neighbor.boundary_type, BoundaryType::AdjacentBuilding);
        assert_eq!(
            neighbor.temperature_factor, None,
            "AdjacentBuilding: f_b via formule 4.15-4.17"
        );

        let ground = find(&elems, "bp_5");
        assert_eq!(ground.boundary_type, BoundaryType::Ground);
        assert_eq!(
            ground.temperature_factor, None,
            "Ground: formule 4.18-route, f_k ongebruikt"
        );
    }

    /// Regressie op de audit-rekensom: een binnenwand naar een 15 °C-ruimte
    /// mag niet meer als volle buitenschil meetellen. Met f_k = None en de
    /// Vabi-grenstemperatuur rekent de kern (θ_i−θ_a)/(θ_i−θ_e) = 5/30 in
    /// plaats van 1.0 — een factor 6 lager.
    #[test]
    fn adjacent_room_uses_vabi_boundary_temperature() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 10.0, 90.0, 13.0, Some("AdjacentRoom"), Some(15.0), 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        let wall = find(&elems, "bp_1");

        assert_eq!(wall.adjacent_temperature, Some(15.0));
        // U = 1/(0.13 + 4.0 + 0.04) = 0.2398
        let h = h_t_adjacent_room_element(wall, 20.0, 15.0, -10.0, 2.0, -1.0);
        let expected = 10.0 * (1.0 / 4.17) * (5.0 / 30.0);
        assert!(
            (h - expected).abs() < 1e-6,
            "H_T,ia = {h}, verwacht {expected}"
        );
        // En géén volle-buitenschil-gedrag meer:
        assert!(h < 0.5, "binnenwand telt niet meer als volle buitenschil");
    }

    // ------------------------------------------------------------------
    // Fix 2 — grondvloeren: geen stil 0 W meer
    // ------------------------------------------------------------------

    /// Audit §2.2 bevinding 2: Ground-elementen kregen `ground_params: None`
    /// waardoor `h_t_ground_element` stil 0 W opleverde. Nu worden U_e
    /// (Figuur 4.2-curve, B' = 2A/O, z = 0) en f_g2 (Vabi-grondtemperatuur)
    /// afgeleid en is het grondverlies > 0.
    #[test]
    fn ground_floor_no_longer_silent_zero_watt() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        // Grondvloer 50 m², omtrek 30 m, θ_grond,winter = 4 °C.
        insert_part(&conn, 1, "Floor", 50.0, 0.0, 30.0, Some("Ground"), Some(4.0), 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        let floor = find(&elems, "bp_1");

        let gp = floor
            .ground_params
            .as_ref()
            .expect("grondvloer moet ground_params krijgen");

        // U_vloer = 1/(0.17+4.0+0.04) = 0.23753; U_k = U + ΔU_TB = 0.33753.
        // B' = 2·50/30 = 3.333 → U_e = |a·b| / (c1·B'^n1 + c2·U_k^n2 + d) ≈ 0.277.
        assert!(
            (gp.u_equivalent - 0.277).abs() < 0.005,
            "U_e = {}, verwacht ≈ 0.277 (Figuur 4.2-curve)",
            gp.u_equivalent
        );
        // f_g2 = (20 − 4) / (20 − (−10)) = 0.5333.
        assert!(
            (gp.fg2 - 16.0 / 30.0).abs() < 1e-9,
            "f_g2 = {}, verwacht 0.5333",
            gp.fg2
        );
        assert_eq!(gp.ground_water_factor, 1.0);

        // De kern-formule 4.18 levert nu daadwerkelijk warmteverlies:
        // 1.45 × 1.0 × 50 × 0.5333 × 0.277 ≈ 10.7 W/K.
        let h = h_t_ground_element(floor);
        assert!(h > 0.0, "grondvloer mag geen stil 0 W meer zijn");
        assert!(
            (h - 10.72).abs() < 0.3,
            "H_T,ig = {h}, verwacht ≈ 10.72 W/K"
        );
    }

    /// Grondwand: z-diepte zit niet in de Vabi-data → conservatieve
    /// U_e = U_k-fallback mét expliciete warning (geen stil 0 W).
    #[test]
    fn ground_wall_falls_back_to_u_k_with_warning() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 12.0, 90.0, 14.0, Some("Ground"), Some(4.0), 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        let wall = find(&elems, "bp_1");

        let gp = wall.ground_params.as_ref().expect("ground_params");
        // U_wand = 1/(0.13+4.0+0.04) = 0.23981; U_k = 0.33981.
        let u_k = 1.0 / 4.17 + 0.1;
        assert!(
            (gp.u_equivalent - u_k).abs() < 1e-9,
            "U_e = {}, verwacht U_k-fallback {u_k}",
            gp.u_equivalent
        );
        assert!(h_t_ground_element(wall) > 0.0);
        assert!(
            warnings.iter().any(|w| w.contains("Grondwand")),
            "fallback moet een warning geven: {warnings:?}"
        );
    }

    /// CrawlSpace mapt (ongewijzigd) naar Ground en krijgt nu óók
    /// ground_params in plaats van stil 0 W.
    #[test]
    fn crawlspace_floor_gets_ground_params() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Floor", 40.0, 0.0, 26.0, Some("CrawlSpace"), Some(8.0), 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        let floor = find(&elems, "bp_1");

        assert_eq!(floor.boundary_type, BoundaryType::Ground);
        assert!(floor.ground_params.is_some());
        assert!(h_t_ground_element(floor) > 0.0);
    }

    // ------------------------------------------------------------------
    // Fix 5 — UnconditionedSpace + onbekende grensvlaktypes
    // ------------------------------------------------------------------

    /// Audit §2.2 bevinding 5: `UnconditionedSpace` viel in de
    /// `_ => Exterior`-arm en telde met volle ΔT mee. Nu → UnheatedSpace
    /// (§2.5.2) met f_k-default 0.5 in de rekenkern.
    #[test]
    fn unconditioned_space_maps_to_unheated_space() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 8.0, 90.0, 12.0, Some("UnconditionedSpace"), None, 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");
        let wall = find(&elems, "bp_1");

        assert_eq!(wall.boundary_type, BoundaryType::UnheatedSpace);
        // Geen warning voor dit (bekende) type.
        assert!(
            !warnings.iter().any(|w| w.contains("UnconditionedSpace")),
            "bekend type mag geen onbekend-type-warning geven: {warnings:?}"
        );
        // Rekenkern: H_T,io = A × U × 0.5 (formule 4.10, f_k-default).
        let h = h_t_unheated_element(wall);
        let expected = 8.0 * (1.0 / 4.17) * 0.5;
        assert!((h - expected).abs() < 1e-6, "H_T,io = {h}, verwacht {expected}");
    }

    /// Onbekende types blijven op Exterior vallen maar nooit meer stil:
    /// er komt een warning, gededupliceerd over meerdere vlakken.
    #[test]
    fn unknown_boundary_type_warns_not_silent() {
        let conn = test_conn();
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 8.0, 90.0, 12.0, Some("FancyNewBoundary"), None, 10);
        insert_part(&conn, 2, "Wall", 6.0, 90.0, 10.0, Some("FancyNewBoundary"), None, 10);

        let mut warnings = Vec::new();
        let elems =
            map_constructions_per_room_with_warnings(&conn, 1, 20.0, -10.0, &mut warnings)
                .expect("mapping");

        assert_eq!(find(&elems, "bp_1").boundary_type, BoundaryType::Exterior);
        let hits: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("FancyNewBoundary"))
            .collect();
        assert_eq!(
            hits.len(),
            1,
            "precies één (gededupliceerde) warning verwacht: {warnings:?}"
        );
    }

    // ------------------------------------------------------------------
    // Fix 3 — dwelling_class afleiding (VabiCompat rekent weer)
    // ------------------------------------------------------------------

    /// Gestapelde bouw (Apartment) → etage/flat/portiek (Tabel 2.8, rij 3).
    /// Hiermee heeft het geïmporteerde project een dwelling_class en faalt
    /// de VabiCompat-keten niet meer met Isso51Error::InfiltrationConfig.
    #[test]
    fn dwelling_class_apartment_maps_to_etage() {
        let conn = test_conn();
        insert_building_chain(&conn, "Apartment", None, None, None);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        assert_eq!(building.infiltration_method, InfiltrationMethod::VabiCompat);
        assert_eq!(
            building.dwelling_class,
            Some(DwellingClass::EtageFlatOfPortiek),
            "VabiCompat-import zonder dwelling_class faalt bij elke berekening"
        );
    }

    /// Eengezinswoning met hellend dakvlak (slope 135°) → met kap.
    #[test]
    fn dwelling_class_pitched_roof_maps_to_met_kap() {
        let conn = test_conn();
        insert_building_chain(&conn, "Detached", None, None, None);
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Roof", 30.0, 135.0, 24.0, Some("OutsideAir"), None, 10);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        assert_eq!(
            building.dwelling_class,
            Some(DwellingClass::EengezinswoningMetKap)
        );
    }

    /// Eengezinswoning met uitsluitend vlakke daken (slope 180°) → plat dak.
    #[test]
    fn dwelling_class_flat_roof_maps_to_platdak() {
        let conn = test_conn();
        insert_building_chain(&conn, "Detached", None, None, None);
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Roof", 30.0, 180.0, 24.0, Some("OutsideAir"), None, 10);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        assert_eq!(
            building.dwelling_class,
            Some(DwellingClass::EengezinswoningPlatdak)
        );
    }

    /// Geen daken en geen kap-indicatie → gedocumenteerde conservatieve
    /// fallback (met kap, q_i,spec = 1.0) mét warning — de berekening blijft
    /// werken (geen Isso51Error meer), maar de gebruiker wordt gewaarschuwd.
    #[test]
    fn dwelling_class_fallback_warns_but_still_calculates() {
        let conn = test_conn();
        insert_building_chain(&conn, "Detached", None, None, None);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        assert_eq!(
            building.dwelling_class,
            Some(DwellingClass::EengezinswoningMetKap)
        );
        assert!(
            warnings.iter().any(|w| w.contains("Woningklasse")),
            "fallback moet een warning geven: {warnings:?}"
        );
    }

    /// Kap-indicatie via BuildingWithHoodType (zonder dakvlakken in de DB).
    #[test]
    fn dwelling_class_hood_field_maps_to_met_kap_without_warning() {
        let conn = test_conn();
        insert_building_chain(&conn, "Detached", Some("WithHood"), None, None);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        // Let op: BuildingShapeType='Detached' wint van de WithHood→Stacked
        // arm in map_building_type, dus dit blijft een eengezinswoning.
        assert_eq!(
            building.dwelling_class,
            Some(DwellingClass::EengezinswoningMetKap)
        );
        assert!(
            !warnings.iter().any(|w| w.contains("Woningklasse")),
            "kap-indicatie aanwezig → geen fallback-warning: {warnings:?}"
        );
    }

    // ------------------------------------------------------------------
    // Fix 4 — nachtverlaging niet meer hardcoded true
    // ------------------------------------------------------------------

    /// Audit §2.2 bevinding 4: `has_night_setback: true` hardcoded gaf een
    /// fictieve opwarmtoeslag Φ_hu = P × A_g (§4.3) ongeacht de
    /// Vabi-instelling. De Vabi-DB heeft geen afleidbaar
    /// bedrijfsbeperking-veld → norm-veilige default `false` + warning.
    #[test]
    fn night_setback_defaults_to_false_with_warning() {
        let conn = test_conn();
        insert_building_chain(&conn, "Apartment", None, None, None);

        let mut warnings = Vec::new();
        let building = map_building_with_warnings(&conn, &mut warnings).expect("building");

        assert!(
            !building.has_night_setback,
            "geen fantoom-opwarmtoeslag: has_night_setback moet false zijn"
        );
        assert!(
            warnings.iter().any(|w| w.contains("achtverlaging")),
            "default-false moet een warning geven: {warnings:?}"
        );
    }

    // ------------------------------------------------------------------
    // Backward-compat wrappers
    // ------------------------------------------------------------------

    /// De oude publieke signatures (gebruikt door `crates/vabi-importer`)
    /// blijven werken en leveren dezelfde elementen als de
    /// `_with_warnings`-varianten.
    #[test]
    fn legacy_wrappers_keep_working() {
        let conn = test_conn();
        insert_building_chain(&conn, "Apartment", None, None, None);
        insert_opaque_construction(&conn, 10, 4.0);
        insert_part(&conn, 1, "Wall", 12.0, 90.0, 14.0, Some("OutsideAir"), None, 10);

        let building = map_building(&conn).expect("legacy map_building");
        assert!(!building.has_night_setback);
        assert!(building.dwelling_class.is_some());

        let elems = map_constructions_per_room(&conn, 1, 100).expect("legacy constructions");
        assert_eq!(elems.len(), 1);
        assert_eq!(elems[0].temperature_factor, Some(1.0));
    }

    // ------------------------------------------------------------------
    // Pure helpers
    // ------------------------------------------------------------------

    /// Worked-example check van de Figuur 4.2-curve (ISSO 53 p.65):
    /// U_k = 2.43, A = 200, O = 98 (B' ≈ 4.1), z = 0 → U_e ≈ 0.177.
    #[test]
    fn u_equivalent_curve_matches_worked_example() {
        let u_e = u_equivalent_ground_floor(200.0, 98.0, 2.43).expect("curve");
        assert!(
            (u_e - 0.177).abs() < 0.01,
            "U_e = {u_e}, verwacht ≈ 0.177 (ISSO 53 worked example p.65)"
        );
    }

    /// Ongeldige invoer → None (caller valt terug op U_k + warning).
    #[test]
    fn u_equivalent_curve_rejects_invalid_input() {
        assert!(u_equivalent_ground_floor(0.0, 30.0, 0.3).is_none());
        assert!(u_equivalent_ground_floor(50.0, 0.0, 0.3).is_none());
        assert!(u_equivalent_ground_floor(50.0, 30.0, 0.0).is_none());
    }
}

