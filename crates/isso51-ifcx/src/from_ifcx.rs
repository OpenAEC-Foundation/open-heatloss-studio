//! Convert an IFCX document to an isso51-core `Project`.
//!
//! Reads the `isso51::` namespace attributes from IfcSpace, IfcBuilding,
//! and IfcProject entries and maps them to the isso51-core domain model.

use std::collections::HashMap;

use isso51_core::model::{
    Building, BuildingType, ConstructionElement, DesignConditions, InfiltrationMethod, Project,
    ProjectInfo, Room, SecurityClass, VentilationConfig, VentilationSystemType,
};

use crate::document::{ifc_class, IfcxDataEntry, IfcxDocument};
use crate::error::{IfcxError, Result};
use crate::namespace::{
    ns, Isso51Building, Isso51Conditions, Isso51Construction, Isso51Room, Isso51Ventilation,
};

/// Extract an isso51-core `Project` from an IFCX document.
///
/// The document must contain:
/// - One IfcProject entry with `isso51::conditions` (design climate)
/// - One IfcBuilding entry with `isso51::building` (building properties)
/// - One or more IfcSpace entries with `isso51::room` (room properties)
/// - Construction elements as children of IfcSpace with `isso51::construction`
pub fn project_from_ifcx(doc: &IfcxDocument) -> Result<Project> {
    // Find the project entry
    let project_entries = doc.find_by_class(ifc_class::PROJECT);
    let project_entry = project_entries
        .first()
        .ok_or(IfcxError::MissingEntry("IfcProject"))?;

    // Find the building entry
    let building_entries = doc.find_by_class(ifc_class::BUILDING);
    let building_entry = building_entries
        .first()
        .ok_or(IfcxError::MissingEntry("IfcBuilding"))?;

    // Extract conditions from project
    let conditions: Isso51Conditions = project_entry
        .get_attr(ns::CONDITIONS)
        .ok_or(IfcxError::MissingAttribute(
            "IfcProject",
            ns::CONDITIONS,
        ))?;

    // Extract building data
    let building_data: Isso51Building = building_entry
        .get_attr(ns::BUILDING)
        .ok_or(IfcxError::MissingAttribute(
            "IfcBuilding",
            ns::BUILDING,
        ))?;

    // Extract ventilation config from project or building
    let ventilation_data: Isso51Ventilation = project_entry
        .get_attr(ns::VENTILATION)
        .or_else(|| building_entry.get_attr(ns::VENTILATION))
        .ok_or(IfcxError::MissingAttribute(
            "IfcProject|IfcBuilding",
            ns::VENTILATION,
        ))?;

    // Build path→entry index for resolving children
    let entry_map: HashMap<&str, &IfcxDataEntry> =
        doc.data.iter().map(|e| (e.path.as_str(), e)).collect();

    // Find all IfcSpace entries
    let space_entries = doc.find_by_class(ifc_class::SPACE);
    if space_entries.is_empty() {
        return Err(IfcxError::MissingEntry("IfcSpace"));
    }

    // Convert spaces to rooms
    let mut rooms = Vec::with_capacity(space_entries.len());
    for (i, space) in space_entries.iter().enumerate() {
        let room = space_to_room(space, &entry_map, i)?;
        rooms.push(room);
    }

    // Map to isso51-core types
    let project = Project {
        info: ProjectInfo {
            name: project_entry
                .ifc_prop("Name")
                .and_then(|v| v.as_str())
                .unwrap_or("IFCX Import")
                .to_string(),
            project_number: None,
            address: None,
            client: None,
            date: None,
            engineer: None,
            notes: None,
        },
        building: map_building(&building_data),
        climate: map_conditions(&conditions),
        ventilation: map_ventilation(&ventilation_data),
        rooms,
    };

    Ok(project)
}

fn space_to_room(
    space: &IfcxDataEntry,
    entry_map: &HashMap<&str, &IfcxDataEntry>,
    index: usize,
) -> Result<Room> {
    let room_data: Isso51Room = space.get_attr(ns::ROOM).ok_or_else(|| {
        IfcxError::MissingAttribute("IfcSpace", ns::ROOM)
    })?;

    let name = space
        .ifc_prop("Name")
        .and_then(|v| v.as_str())
        .unwrap_or("Ruimte")
        .to_string();

    let id = format!("r{}", index + 1);

    // Collect construction elements from children
    let mut constructions = Vec::new();
    for (child_name, child_path) in &space.children {
        if let Some(child_entry) = entry_map.get(child_path.as_str()) {
            if let Some(constr) = child_entry.get_attr::<Isso51Construction>(ns::CONSTRUCTION) {
                constructions.push(map_construction(
                    &constr,
                    child_name,
                    constructions.len(),
                ));
            }
        }
    }

    Ok(Room {
        id,
        name,
        function: parse_room_function(&room_data.function),
        custom_temperature: room_data.custom_temperature,
        floor_area: room_data.floor_area,
        height: room_data.height,
        constructions,
        heating_system: room_data
            .heating_system
            .as_deref()
            .map(parse_heating_system)
            .unwrap_or(isso51_core::model::HeatingSystem::RadiatorLt),
        ventilation_rate: room_data.ventilation_rate,
        has_mechanical_exhaust: room_data.has_mechanical_exhaust,
        has_mechanical_supply: room_data.has_mechanical_supply,
        fraction_outside_air: room_data.fraction_outside_air,
        supply_air_temperature: None,
        internal_air_temperature: None,
        clamp_positive: true,
    })
}

fn map_construction(constr: &Isso51Construction, name: &str, index: usize) -> ConstructionElement {
    use isso51_core::model::enums::*;

    ConstructionElement {
        id: format!("c{}", index + 1),
        description: if constr.description.is_empty() {
            name.to_string()
        } else {
            constr.description.clone()
        },
        area: constr.area,
        u_value: constr.u_value,
        boundary_type: parse_boundary_type(&constr.boundary_type),
        material_type: parse_material_type(&constr.material_type),
        temperature_factor: constr.temperature_factor,
        adjacent_room_id: None, // resolved by path in a second pass if needed
        adjacent_temperature: constr.adjacent_temperature,
        vertical_position: constr
            .vertical_position
            .as_deref()
            .map(parse_vertical_position)
            .unwrap_or(VerticalPosition::Wall),
        use_forfaitaire_thermal_bridge: constr.use_forfaitaire_thermal_bridge,
        custom_delta_u_tb: constr.custom_delta_u_tb,
        ground_params: None,
        has_embedded_heating: constr.has_embedded_heating,
    }
}

fn map_building(data: &Isso51Building) -> Building {
    Building {
        building_type: parse_building_type(&data.building_type),
        qv10: data.qv10,
        total_floor_area: data.total_floor_area,
        security_class: parse_security_class(&data.security_class),
        has_night_setback: data.has_night_setback,
        warmup_time: data.warmup_time,
        building_height: None,
        num_floors: data.num_floors.unwrap_or(1),
        infiltration_method: data
            .infiltration_method
            .as_deref()
            .map(parse_infiltration_method)
            .unwrap_or_default(),
    }
}

fn map_conditions(data: &Isso51Conditions) -> DesignConditions {
    let mut dc = DesignConditions::default();
    dc.theta_e = data.theta_e;
    if let Some(tb) = data.theta_b_residential {
        dc.theta_b_residential = tb;
    }
    dc
}

fn map_ventilation(data: &Isso51Ventilation) -> VentilationConfig {
    VentilationConfig {
        system_type: parse_ventilation_system(&data.system_type),
        has_heat_recovery: data.has_heat_recovery,
        heat_recovery_efficiency: data.heat_recovery_efficiency,
        frost_protection: None,
        supply_temperature: None,
        has_preheating: false,
        preheating_temperature: None,
    }
}

// ---------------------------------------------------------------------------
// String → enum parsers (snake_case from IFCX JSON)
// ---------------------------------------------------------------------------

fn parse_room_function(s: &str) -> isso51_core::model::RoomFunction {
    use isso51_core::model::RoomFunction::*;
    match s {
        "living_room" => LivingRoom,
        "kitchen" => Kitchen,
        "bedroom" => Bedroom,
        "bathroom" => Bathroom,
        "toilet" => Toilet,
        "hallway" => Hallway,
        "landing" => Landing,
        "storage" => Storage,
        "attic" => Attic,
        _ => Custom,
    }
}

fn parse_heating_system(s: &str) -> isso51_core::model::HeatingSystem {
    use isso51_core::model::HeatingSystem::*;
    match s {
        "local_gas_heater" => LocalGasHeater,
        "radiator_ht" => RadiatorHt,
        "radiator_lt" => RadiatorLt,
        "floor_heating_main_low" => FloorHeatingMainLow,
        "floor_heating_main_high" => FloorHeatingMainHigh,
        "ceiling_heating" => CeilingHeating,
        "wall_heating" => WallHeating,
        "plinth_heating" => PlinthHeating,
        "fan_convector" => FanConvector,
        _ => RadiatorLt,
    }
}

fn parse_boundary_type(s: &str) -> isso51_core::model::enums::BoundaryType {
    use isso51_core::model::enums::BoundaryType::*;
    match s {
        "exterior" => Exterior,
        "unheated_space" => UnheatedSpace,
        "adjacent_room" => AdjacentRoom,
        "adjacent_building" => AdjacentBuilding,
        "ground" => Ground,
        _ => Exterior,
    }
}

fn parse_material_type(s: &str) -> isso51_core::model::enums::MaterialType {
    use isso51_core::model::enums::MaterialType::*;
    match s {
        "masonry" => Masonry,
        "non_masonry" => NonMasonry,
        _ => Masonry,
    }
}

fn parse_vertical_position(s: &str) -> isso51_core::model::enums::VerticalPosition {
    use isso51_core::model::enums::VerticalPosition::*;
    match s {
        "floor" => Floor,
        "ceiling" => Ceiling,
        "wall" => Wall,
        _ => Wall,
    }
}

fn parse_building_type(s: &str) -> BuildingType {
    match s {
        "detached" => BuildingType::Detached,
        "semi_detached" => BuildingType::SemiDetached,
        "terraced" => BuildingType::Terraced,
        "end_of_terrace" => BuildingType::EndOfTerrace,
        "porch" => BuildingType::Porch,
        "gallery" => BuildingType::Gallery,
        "stacked" => BuildingType::Stacked,
        _ => BuildingType::Detached,
    }
}

fn parse_security_class(s: &str) -> SecurityClass {
    match s {
        "a" => SecurityClass::A,
        "b" => SecurityClass::B,
        "c" => SecurityClass::C,
        _ => SecurityClass::B,
    }
}

fn parse_ventilation_system(s: &str) -> VentilationSystemType {
    match s {
        "system_a" => VentilationSystemType::SystemA,
        "system_b" => VentilationSystemType::SystemB,
        "system_c" => VentilationSystemType::SystemC,
        "system_d" => VentilationSystemType::SystemD,
        "system_e" => VentilationSystemType::SystemE,
        _ => VentilationSystemType::SystemC,
    }
}

fn parse_infiltration_method(s: &str) -> InfiltrationMethod {
    match s {
        "per_floor_area" => InfiltrationMethod::PerFloorArea,
        _ => InfiltrationMethod::PerExteriorArea,
    }
}
