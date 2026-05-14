//! SQLite database mapping from Vabi Elements to ISSO 51 Project structures.
//!
//! Maps Vabi's ERM (Entity-Relationship Model) to our domain model, handling
//! the multi-table joins needed to extract project, building, climate, ventilation,
//! and room data.

use crate::error::{Isso51Error, Result};
use crate::model::*;
use crate::model::enums::*;
use rusqlite::Connection;
use std::path::Path;

/// Import a complete project from a Vabi `.vp` file.
///
/// This is the main public API for Vabi import. Opens the ZIP archive,
/// extracts the SQLite database, and maps all relevant data to our Project model.
///
/// # Arguments
/// * `vp_path` - Path to the Vabi `.vp` project file
///
/// # Returns
/// Complete `Project` structure ready for ISSO 51 calculation.
///
/// # Limitations
/// Phase 1 implementation - constructions (BuildingPart, materials) are not imported.
/// Rooms will have empty `constructions` vectors. Use this for project metadata
/// and validation only.
pub fn import_vabi_project(vp_path: &Path) -> Result<Project> {
    let (db_path, _temp_file) = super::unzip::extract_elements_database(vp_path)?;

    let conn = Connection::open(&db_path).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Cannot open Elements.sqlite3: {}", e))
    })?;

    let project_info = map_project_info(&conn)?;
    let building = map_building(&conn)?;
    let climate = map_climate(&conn)?;
    let ventilation = map_ventilation(&conn)?;
    let rooms = map_rooms(&conn)?;

    Ok(Project {
        info: project_info,
        building,
        climate,
        ventilation,
        rooms,
    })
}

/// Map basic project information from Project and ProjectData tables.
fn map_project_info(conn: &Connection) -> Result<ProjectInfo> {
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
fn map_building(conn: &Connection) -> Result<Building> {
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
                b.NumberOfFloors
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

        Ok(Building {
            building_type,
            qv10,
            infiltration_method,
            total_floor_area,
            num_floors,
            security_class,
            has_night_setback: true, // Vabi default assumption
            warmup_time: 2.0,        // ISSO 51 standard
            building_height: None,
            dwelling_class: None,
            construction_variant: None,
            construction_year: None,
            aggregation_method: AggregationMethod::default(),
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

/// Map climate conditions from ClimateHeatLossCalculation table.
fn map_climate(conn: &Connection) -> Result<DesignConditions> {
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
fn map_ventilation(conn: &Connection) -> Result<VentilationConfig> {
    let mut stmt = conn
        .prepare(
            "SELECT
                v.SupplySource,
                v.CirculationRateMethod2017
             FROM Ventilation v
             LIMIT 1"
        )
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Ventilation query failed: {}", e)))?;

    let mut rows = stmt.query([]).map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Ventilation query execution failed: {}", e))
    })?;

    let (system_type, has_heat_recovery) = if let Some(row) = rows.next().map_err(|e| {
        Isso51Error::VabiSqliteError(format!("Ventilation row fetch failed: {}", e))
    })? {
        let supply_source: Option<String> = row.get(0).unwrap_or(None);
        let circulation_method: Option<String> = row.get(1).unwrap_or(None);

        // Check for heat recovery systems
        let has_hr = check_heat_recovery(conn).unwrap_or(false);

        // Map system type - this is complex, use simplified mapping for Phase 1
        let system_type = map_ventilation_system_type(supply_source.as_deref(), circulation_method.as_deref());

        (system_type, has_hr)
    } else {
        // No ventilation data found - use defaults
        (VentilationSystemType::SystemC, false)
    };

    Ok(VentilationConfig {
        system_type,
        has_heat_recovery,
        heat_recovery_efficiency: None,
        frost_protection: None,
        supply_temperature: None,
        has_preheating: false,
        preheating_temperature: None,
    })
}

/// Check if heat recovery systems are present.
fn check_heat_recovery(conn: &Connection) -> Result<bool> {
    // Look for LocalHeatRecoverySystemX entries
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM LocalHeatRecoverySystemX WHERE ID IS NOT NULL")
        .map_err(|e| Isso51Error::VabiSqliteError(format!("Heat recovery query failed: {}", e)))?;

    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap_or(0);
    Ok(count > 0)
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

/// Map room data from Room table and related temperature settings.
///
/// `Room.RoomRequirementsID` is an **AspectID**, mirroring the Building pattern.
/// Chain: AspectID → VarAsp_RoomRequirementsData → RoomRequirementsTemplate.DataID →
/// RoomRequirementsData.ID → ConditionsID → RoomDesignConditions →
/// DesignTemperaturesWinterID → DesignTemperatures.TemperatureDay.
/// See `docs/vabi-schema-reference.md` for the full join.
fn map_rooms(conn: &Connection) -> Result<Vec<Room>> {
    let mut stmt = conn
        .prepare(
            "SELECT
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
        let room_number: String = row.get(0).unwrap_or_else(|_| "Unknown".to_string());
        let room_name: String = row.get(1).unwrap_or_else(|_| "Unnamed Room".to_string());
        let _requirements_id: Option<i32> = row.get(2).unwrap_or(None);
        let theta_i: Option<f64> = row.get(3).unwrap_or(None);

        // Use temperature from joined table or default
        let theta_i = theta_i.unwrap_or(20.0);

        let room = Room {
            id: room_number.clone(),
            name: room_name,
            function: RoomFunction::LivingRoom, // TODO: map from Vabi room types in Phase 2
            custom_temperature: None,
            floor_area: 0.0, // TODO: get from Room.Area in Phase 2 - not in simplified query
            height: 2.7, // TODO: get from Room.Height in Phase 2 - not in simplified query
            constructions: vec![], // Phase 1: empty constructions
            heating_system: HeatingSystem::RadiatorLt, // TODO: map from Vabi data in Phase 2
            ventilation_rate: None, // TODO: calculate from Vabi data
            has_mechanical_exhaust: false, // TODO: map from system configuration
            has_mechanical_supply: false,  // TODO: map from system configuration
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
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

