//! SQLite database mapping from Vabi Elements to ISSO 51 Project structures.
//!
//! Maps Vabi's ERM (Entity-Relationship Model) to our domain model, handling
//! the multi-table joins needed to extract project, building, climate, ventilation,
//! and room data.

use crate::error::{Isso51Error, Result};
use crate::model::*;
use crate::model::enums::*;
use crate::model::construction::*;
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
/// Links Room to BuildingParts via the cell-based geometry system:
/// Room.CellID → MainFace → CellFace → BuildingPart
/// Then extracts area, U-value, and boundary type for each BuildingPart.
fn map_constructions_per_room(conn: &Connection, room_id: i64, _room_cell_id: i64) -> Result<Vec<ConstructionElement>> {
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
                bp.PsiThermalBridge
             FROM Room r
             JOIN MainFace mf ON mf.CellID = r.CellID
             JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
             JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
             JOIN Face f ON f.ID = bp.FaceID
             JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
             LEFT JOIN BoundaryConditions bc ON bc.ID = bp.BoundaryConditionsID
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
        let boundary_type = map_boundary_type(boundary_type_str.as_deref());

        // Generate description
        let description = format!("{} {}", part_type, construction_counter);
        construction_counter += 1;

        let construction = ConstructionElement {
            id: format!("bp_{}", building_part_id),
            description,
            area,
            u_value,
            boundary_type,
            material_type: MaterialType::Masonry, // Default - could be improved by analyzing materials
            temperature_factor: Some(1.0), // Default f_k
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: match part_type.as_str() {
                "Floor" => VerticalPosition::Floor,
                "Roof" | "FlatRoof" => VerticalPosition::Ceiling,
                _ => VerticalPosition::Wall,
            },
            // Vabi populates PsiThermalBridge per BuildingPart. If > 0, use it
            // as a per-element ΔU_TB override; if 0/None, fall back to forfaitaire.
            use_forfaitaire_thermal_bridge: psi_thermal_bridge.map(|p| p <= 0.0).unwrap_or(true),
            custom_delta_u_tb: psi_thermal_bridge.filter(|&p| p > 0.0),
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        };

        constructions.push(construction);
    }

    Ok(constructions)
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
fn map_boundary_type(vabi_type: Option<&str>) -> BoundaryType {
    match vabi_type {
        Some("OutsideAir") => BoundaryType::Exterior,
        Some("Ground") => BoundaryType::Ground,
        Some("AdjacentRoom") => BoundaryType::AdjacentRoom,
        Some("AdjacentBuilding") => BoundaryType::AdjacentBuilding,
        _ => {
            // TODO: Log warning for unknown boundary type
            BoundaryType::Exterior // Safe fallback
        }
    }
}

/// Calculate room height from volume and floor area.
///
/// Room.VolumeInfoID → VariantVolumeInfo → TypedVolumeData with Type='InternalDimensionsIncludingPlenum'
/// height = volume / floor_area
fn calculate_room_height(conn: &Connection, room_id: i64, floor_area: f64) -> Result<f64> {
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
fn map_rooms(conn: &Connection) -> Result<Vec<Room>> {
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
        let room_cell_id: i64 = row.get(1).unwrap_or(0);
        let room_number: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());
        let room_name: String = row.get(3).unwrap_or_else(|_| "Unnamed Room".to_string());
        let _requirements_id: Option<i32> = row.get(4).unwrap_or(None);
        let theta_i: Option<f64> = row.get(5).unwrap_or(None);

        // Use temperature from joined table or default
        let theta_i = theta_i.unwrap_or(20.0);

        // Map constructions for this room. Propagate errors instead of silently
        // swallowing — query bugs were masked here in early Phase 2.
        let constructions = map_constructions_per_room(conn, room_id, room_cell_id)?;

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

