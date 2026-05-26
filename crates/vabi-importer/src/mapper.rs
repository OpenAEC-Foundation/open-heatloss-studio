use rusqlite::Connection;
use openaec_project_shared::{
    BuildingTypeShared, ProjectV2, ResidentialType, SharedGeometry, SharedProject, Space,
    UtilityType,
};
use openaec_project_shared::calcs::{Calcs, Iso53Inputs};
use openaec_project_shared::geometry::{
    BoundaryKind as V2BoundaryKind, Construction as V2Construction, ConstructionKind,
};
use openaec_project_shared::shared::{HeatRecovery, VentilationSystemKind};
use isso51_core::import::vabi::{
    calculate_room_height, map_building, map_climate, map_constructions_per_room,
    map_project_info, map_ventilation,
};
use isso51_core::model::{BoundaryType, ConstructionElement, VerticalPosition};

use crate::error::{Result, VabiImporterError};

/// Import Vabi Elements project to ProjectV2 format
///
/// **Capabilities (Phase 1c):**
/// - ✓ Basic project info mapping
/// - ✓ Building type detection (utiliteit vs woning)
/// - ✓ Floor area calculation and room height
/// - ✓ Room setpoint propagation
/// - ✓ Boundary type expansion (CrawlSpace, OtherBuilding, Ground)
/// - ✓ OrganisationData JOIN for gebruiksFunctie/ruimteType
/// - △ AdjacentRoom cell-coupling (complex, may defer)
pub fn import_vabi_project_v2(
    db_path: &std::path::Path,
) -> Result<ProjectV2> {
    let conn = Connection::open(db_path)?;

    // Map core project data
    let project_info = map_project_info(&conn)?;
    let building = map_building(&conn)?;
    let climate = map_climate(&conn)?;
    let ventilation = map_ventilation(&conn)?;

    // Create shared project section
    let shared = map_shared_project(&conn, &project_info, &building)?;

    // Map rooms with V2-specific enrichment
    let spaces = map_spaces_v2(&conn)?;
    let geometry = SharedGeometry { spaces };

    // Create ISSO 53 calc inputs
    let calcs = create_isso53_calcs(&conn, &building)?;

    Ok(ProjectV2 {
        schema_version: 2,
        shared,
        geometry,
        calcs,
    })
}

/// Map project info to V2 SharedProject format
fn map_shared_project(
    conn: &Connection,
    project_info: &isso51_core::model::ProjectInfo,
    building: &isso51_core::model::Building,
) -> Result<SharedProject> {
    // Detect building type from OrganisationData
    let building_type = detect_building_type_v2(conn)?;

    // Calculate total floor area from rooms
    let gross_floor_area_m2 = calculate_total_floor_area(conn)?;

    // Map ventilation system
    let ventilation_system = map_ventilation_system_v2(&building);

    Ok(SharedProject {
        name: project_info.name.clone(),
        project_number: project_info.project_number.clone(),
        address: project_info.address.clone(),
        postcode: None,
        location: None,
        client: project_info.client.clone(),
        date: project_info.date.clone(),
        engineer: project_info.engineer.clone(),
        notes: project_info.notes.clone(),
        building_type,
        construction_year: None,
        gross_floor_area_m2: Some(gross_floor_area_m2),
        num_storeys: Some(building.num_floors),
        ventilation_system,
        heat_recovery: None, // TODO: map from ventilation config
        infiltration_m3_per_h: None,
        mechanical_supply_m3_per_h: None,
        mechanical_exhaust_m3_per_h: None,
    })
}

/// Detect building type from OrganisationData usage patterns
fn detect_building_type_v2(conn: &Connection) -> Result<BuildingTypeShared> {
    let mut stmt = conn.prepare(
        "SELECT od.UsageFunction, COUNT(*) as room_count
         FROM Room r
         JOIN VarAsp_OrganisationData var ON var.AspectID = r.OrganisationID
         JOIN OrganizationTemplate ot ON ot.ID = var.TemplateID
         JOIN OrganisationData od ON od.ID = ot.DataID
         WHERE r.UseInCalculations = 1
         GROUP BY od.UsageFunction
         ORDER BY room_count DESC"
    )?;

    let mut rows = stmt.query([])?;

    // Get the most common usage function
    if let Some(row) = rows.next()? {
        let usage_function: Option<String> = row.get(0).unwrap_or(None);

        match usage_function.as_deref() {
            Some("OfficeFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Office
            }),
            Some("IndustrialFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Industrial
            }),
            Some("EducationFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Education
            }),
            Some("HealthcareFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Healthcare
            }),
            Some("RetailFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Retail
            }),
            Some("LodgingFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Lodging
            }),
            Some("MeetingFunction") => Ok(BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Assembly
            }),
            _ => {
                // Fallback to residential (V1 behavior)
                Ok(BuildingTypeShared::Woning {
                    subtype: ResidentialType::Detached
                })
            }
        }
    } else {
        // No organization data found, default to residential
        Ok(BuildingTypeShared::Woning {
            subtype: ResidentialType::Detached
        })
    }
}

/// Calculate total floor area by summing room floor areas
fn calculate_total_floor_area(conn: &Connection) -> Result<f64> {
    let mut stmt = conn.prepare(
        "SELECT SUM(
            COALESCE(
                (SELECT SUM(fge.Area)
                 FROM MainFace mf
                 JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
                 JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
                 JOIN Face f ON f.ID = bp.FaceID
                 JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
                 WHERE mf.CellID = r.CellID AND bp.BuildingPartType = 'Floor'
                ), 0.0
            )
         ) as total_floor_area
         FROM Room r
         WHERE r.UseInCalculations = 1"
    )?;

    match stmt.query_row([], |row| {
        let total: f64 = row.get(0)?;
        Ok(total)
    }) {
        Ok(area) => Ok(area.max(0.0)),
        Err(_) => Ok(0.0), // Fallback
    }
}

/// Map V1 ventilation system to V2 enum
fn map_ventilation_system_v2(building: &isso51_core::model::Building) -> Option<VentilationSystemKind> {
    // This is a placeholder - we don't have access to ventilation config here
    // TODO: pass ventilation config to this function or refactor
    None
}

/// Map rooms to V2 Space format with enriched data
fn map_spaces_v2(conn: &Connection) -> Result<Vec<Space>> {
    let mut stmt = conn.prepare(
        "SELECT
            r.ID,
            r.CellID,
            r.RoomNumber,
            r.Name,
            r.RoomRequirementsID,
            r.OrganisationID,
            dt.TemperatureDay,
            od.UsageFunction,
            od.RoomType
         FROM Room r
         JOIN Project p ON r.ProjectVersionID = p.CurrentProjectVersionID
         LEFT JOIN VarAsp_RoomRequirementsData var ON var.AspectID = r.RoomRequirementsID
         LEFT JOIN RoomRequirementsTemplate rrt ON rrt.ID = var.TemplateID
         LEFT JOIN RoomRequirementsData rrd ON rrd.ID = rrt.DataID
         LEFT JOIN RoomDesignConditions rdc ON rdc.ID = rrd.ConditionsID
         LEFT JOIN DesignTemperatures dt ON dt.ID = rdc.DesignTemperaturesWinterID
         LEFT JOIN VarAsp_OrganisationData voa ON voa.AspectID = r.OrganisationID
         LEFT JOIN OrganizationTemplate ot ON ot.ID = voa.TemplateID
         LEFT JOIN OrganisationData od ON od.ID = ot.DataID
         WHERE r.UseInCalculations = 1
         ORDER BY r.RoomNumber"
    )?;

    let mut rows = stmt.query([])?;
    let mut spaces = Vec::new();

    while let Some(row) = rows.next()? {
        let room_id: i64 = row.get(0)?;
        let room_cell_id: i64 = row.get(1)?;
        let room_number: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());
        let room_name: String = row.get(3).unwrap_or_else(|_| "Unnamed Room".to_string());
        let theta_i: Option<f64> = row.get(6).unwrap_or(None);
        let theta_i = theta_i.unwrap_or(20.0);

        // Extract organization data for function mapping
        let usage_function: Option<String> = row.get(7).unwrap_or(None);
        let room_type: Option<String> = row.get(8).unwrap_or(None);

        // Map function from OrganisationData
        let function = map_room_function(usage_function.as_deref(), room_type.as_deref());

        // Calculate floor area from Floor BuildingParts
        let floor_area_m2 = calculate_room_floor_area(conn, room_cell_id)?;

        // Calculate room height
        let height_m = calculate_room_height(conn, room_id, floor_area_m2)?;

        // Map constructions for this room - reuse existing V1 logic
        let constructions_v1 = map_constructions_per_room(conn, room_id, room_cell_id)?;

        // Convert V1 constructions to V2 format
        let constructions = constructions_v1.iter()
            .map(v1_to_v2_construction)
            .collect();

        let space = Space {
            id: room_number.clone(),
            name: room_name,
            function,
            floor_area_m2,
            height_m,
            constructions,
            theta_i_winter_c: Some(theta_i),
            theta_i_summer_c: None,
        };

        spaces.push(space);
    }

    Ok(spaces)
}

/// Map OrganisationData UsageFunction and RoomType to V2 function string
fn map_room_function(usage_function: Option<&str>, room_type: Option<&str>) -> Option<String> {
    // Priority: RoomType first (more specific), then UsageFunction
    match room_type {
        Some("HabitableSpace") => Some("living_room".to_string()),
        Some("CirculationRoom") | Some("CirculationRoomInApartmentBuilding") => Some("hallway".to_string()),
        Some("ToiletCompartment") => Some("toilet".to_string()),
        Some("BathRoom") => Some("bathroom".to_string()),
        Some("KitchenRoom") => Some("kitchen".to_string()),
        Some("MeterRoom") | Some("BuildingServicesRoom") => Some("storage".to_string()),
        Some("Elevator") => Some("circulation".to_string()),
        Some("Other") => match usage_function {
            Some("OfficeFunction") => Some("office".to_string()),
            Some("IndustrialFunction") => Some("workspace".to_string()),
            Some("EducationFunction") => Some("classroom".to_string()),
            _ => Some("other".to_string()),
        },
        _ => match usage_function {
            Some("OfficeFunction") => Some("office".to_string()),
            Some("IndustrialFunction") => Some("workspace".to_string()),
            Some("EducationFunction") => Some("classroom".to_string()),
            Some("HealthcareFunction") => Some("healthcare".to_string()),
            Some("RetailFunction") => Some("retail".to_string()),
            Some("LodgingFunction") => Some("lodging".to_string()),
            Some("MeetingFunction") => Some("meeting".to_string()),
            _ => None,
        },
    }
}

/// Convert V1 ConstructionElement to V2 Construction format
fn v1_to_v2_construction(c: &ConstructionElement) -> V2Construction {
    // Map boundary type
    let boundary = match c.boundary_type {
        BoundaryType::Exterior => V2BoundaryKind::Exterior,
        BoundaryType::Ground => V2BoundaryKind::Ground,
        BoundaryType::AdjacentRoom => V2BoundaryKind::AdjacentRoom,
        BoundaryType::AdjacentBuilding => V2BoundaryKind::AdjacentBuilding,
        BoundaryType::UnheatedSpace => V2BoundaryKind::AdjacentRoom,
        BoundaryType::Water => V2BoundaryKind::Exterior,
    };

    // Map construction kind from vertical position
    let kind = match c.vertical_position {
        VerticalPosition::Floor => ConstructionKind::Floor,
        VerticalPosition::Ceiling => ConstructionKind::Ceiling,
        VerticalPosition::Wall => ConstructionKind::Wall,
    };

    V2Construction {
        id: c.id.clone(),
        description: c.description.clone(),
        kind,
        boundary,
        area_m2: c.area,
        u_value: c.u_value,
        orientation_deg: None, // TODO: Extract from Vabi if available
        slope_deg: None, // TODO: Extract from Vabi if available
        openings: Vec::new(), // TODO: Map openings in future batch
        layers: Vec::new(), // TODO: Map material layers in future batch
        adjacent_space_id: c.adjacent_room_id.clone(),
        psi_thermal_bridge: c.custom_delta_u_tb,
    }
}

/// Calculate floor area for a specific room cell
fn calculate_room_floor_area(conn: &Connection, room_cell_id: i64) -> Result<f64> {
    let mut stmt = conn.prepare(
        "SELECT SUM(fge.Area)
         FROM MainFace mf
         JOIN CellFace cf ON cf.FaceID = mf.CellFaceID
         JOIN BuildingPart bp ON bp.ID = cf.BuildingPartID
         JOIN Face f ON f.ID = bp.FaceID
         JOIN FaceGeometryEngine fge ON fge.ID = f.FaceGeometryEngineID
         WHERE mf.CellID = ? AND bp.BuildingPartType = 'Floor'"
    )?;

    match stmt.query_row([room_cell_id], |row| {
        let area: Option<f64> = row.get(0)?;
        Ok(area.unwrap_or(0.0))
    }) {
        Ok(area) => Ok(area.max(0.0)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0.0),
        Err(e) => Err(VabiImporterError::from(e)),
    }
}

/// Create ISSO 53 calculation inputs
fn create_isso53_calcs(
    conn: &Connection,
    building: &isso51_core::model::Building,
) -> Result<openaec_project_shared::Calcs> {
    use serde_json::json;

    // Create a legacy-compatible ISSO 53 input JSON
    let legacy_json = json!({
        "infiltrationMethod": {"known": {"qv10_kar_class": "From040To060"}},
        "heatingUp": {
            "setbackActive": true,
            "pWPerM2": 10.0,
            "warmupMinutes": 120.0
        }
    });

    let isso53 = Iso53Inputs {
        legacy: legacy_json,
    };

    Ok(Calcs {
        isso51: None,
        tojuli: None,
        isso53: Some(isso53),
    })
}