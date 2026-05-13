//! Input validation for ISSO 51 calculations.

use crate::error::{Isso51Error, Result};
use crate::model::building::Project;
use crate::model::enums::BoundaryType;

/// Validate a complete project input.
/// Returns Ok(()) if valid, or an error describing what's wrong.
pub fn validate_project(project: &Project) -> Result<()> {
    if project.rooms.is_empty() {
        return Err(Isso51Error::InvalidInput(
            "project must have at least one room".to_string(),
        ));
    }

    let theta_e = project.climate.theta_e;

    // --- Building-level ventilation checks -----------------------------------
    if let Some(eta) = project.ventilation.heat_recovery_efficiency {
        if !(0.0..=1.0).contains(&eta) {
            return Err(Isso51Error::InvalidInput(format!(
                "heat_recovery_efficiency out of range [0,1]: {eta}"
            )));
        }
    }

    // Build the set of valid room ids once so adjacent_room_id lookups stay
    // O(n·m_avg) instead of O(n·m·r). Using a Vec+contains is fine for
    // typical 10–40 rooms; no need to pull in HashSet.
    let room_ids: Vec<&str> = project.rooms.iter().map(|r| r.id.as_str()).collect();

    for room in &project.rooms {
        if room.floor_area <= 0.0 {
            return Err(Isso51Error::InvalidInput(format!(
                "room '{}' has invalid floor area: {}",
                room.name, room.floor_area
            )));
        }

        if room.height <= 0.0 {
            return Err(Isso51Error::InvalidInput(format!(
                "room '{}' has invalid height: {}",
                room.name, room.height
            )));
        }

        // theta_i != theta_e — a zero gradient means no transmission loss at
        // all, which is almost certainly a data-entry error (custom_temperature
        // accidentally set to θ_e, or RoomFunction::Custom without override).
        let theta_i = room.design_temperature();
        if (theta_i - theta_e).abs() < f64::EPSILON {
            return Err(Isso51Error::InvalidInput(format!(
                "room '{}' ({}): theta_i equals theta_e ({} °C), no temperature gradient",
                room.id, room.name, theta_i
            )));
        }

        // fraction_outside_air is used in ISSO 51 formule 4.7 as a linear
        // blend coefficient; outside [0,1] produces nonsensical H_v values.
        if !(0.0..=1.0).contains(&room.fraction_outside_air) {
            return Err(Isso51Error::InvalidInput(format!(
                "room '{}' ({}): fraction_outside_air out of range [0,1]: {}",
                room.id, room.name, room.fraction_outside_air
            )));
        }

        for element in &room.constructions {
            if element.area <= 0.0 {
                return Err(Isso51Error::InvalidInput(format!(
                    "room '{}', element '{}' has invalid area: {}",
                    room.name, element.description, element.area
                )));
            }

            if element.u_value < 0.0 {
                return Err(Isso51Error::InvalidInput(format!(
                    "room '{}', element '{}' has negative U-value: {}",
                    room.name, element.description, element.u_value
                )));
            }

            // If the element references another room, that room must exist.
            // Only enforced for AdjacentRoom boundary; other boundary types
            // may legitimately carry a stale id (e.g. after deleting a
            // neighbouring room) but don't consume it in the calculation.
            if element.boundary_type == BoundaryType::AdjacentRoom {
                if let Some(ref aid) = element.adjacent_room_id {
                    if !room_ids.contains(&aid.as_str()) {
                        return Err(Isso51Error::InvalidInput(format!(
                            "room '{}', element '{}': adjacent_room_id '{}' does not exist in project",
                            room.name, element.description, aid
                        )));
                    }
                }
            }
        }
    }

    if project.building.qv10 < 0.0 {
        return Err(Isso51Error::InvalidInput(format!(
            "qv10 must be non-negative, got {}",
            project.building.qv10
        )));
    }

    if project.building.total_floor_area <= 0.0 {
        return Err(Isso51Error::InvalidInput(format!(
            "total floor area must be positive, got {}",
            project.building.total_floor_area
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn base_project() -> Project {
        Project {
            info: ProjectInfo {
                name: "test".to_string(),
                project_number: None,
                address: None,
                client: None,
                date: None,
                engineer: None,
                notes: None,
            },
            building: Building {
                building_type: BuildingType::Detached,
                qv10: 50.0,
                total_floor_area: 100.0,
                security_class: SecurityClass::A,
                has_night_setback: false,
                warmup_time: 2.0,
                building_height: None,
                num_floors: 1,
                infiltration_method: InfiltrationMethod::PerExteriorArea,
                dwelling_class: None,
                construction_variant: None,
                construction_year: None,
                aggregation_method: Default::default(),
            },
            climate: DesignConditions::default(),
            ventilation: VentilationConfig {
                system_type: VentilationSystemType::SystemC,
                has_heat_recovery: false,
                heat_recovery_efficiency: None,
                frost_protection: None,
                supply_temperature: None,
                has_preheating: false,
                preheating_temperature: None,
            },
            rooms: vec![base_room("r1", RoomFunction::LivingRoom, None)],
        }
    }

    fn base_room(id: &str, function: RoomFunction, custom_temp: Option<f64>) -> Room {
        Room {
            id: id.to_string(),
            name: format!("Room {id}"),
            function,
            custom_temperature: custom_temp,
            floor_area: 20.0,
            height: 2.6,
            constructions: vec![],
            heating_system: HeatingSystem::RadiatorHt,
            ventilation_rate: Some(18.0),
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            internal_air_temperature: None,
            clamp_positive: true,
        }
    }

    #[test]
    fn accepts_valid_project() {
        assert!(validate_project(&base_project()).is_ok());
    }

    #[test]
    fn rejects_theta_i_equals_theta_e() {
        let mut project = base_project();
        // Force custom temperature to match θ_e (-10°C default).
        project.rooms[0].function = RoomFunction::Custom;
        project.rooms[0].custom_temperature = Some(-10.0);

        let err = validate_project(&project).expect_err("should reject");
        let msg = err.to_string();
        assert!(
            msg.contains("theta_i equals theta_e"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn rejects_fraction_outside_air_below_zero() {
        let mut project = base_project();
        project.rooms[0].fraction_outside_air = -0.1;

        let err = validate_project(&project).expect_err("should reject");
        assert!(err.to_string().contains("fraction_outside_air"));
    }

    #[test]
    fn rejects_fraction_outside_air_above_one() {
        let mut project = base_project();
        project.rooms[0].fraction_outside_air = 1.5;

        let err = validate_project(&project).expect_err("should reject");
        assert!(err.to_string().contains("fraction_outside_air"));
    }

    #[test]
    fn accepts_fraction_outside_air_boundaries() {
        let mut project = base_project();
        project.rooms[0].fraction_outside_air = 0.0;
        assert!(validate_project(&project).is_ok());
        project.rooms[0].fraction_outside_air = 1.0;
        assert!(validate_project(&project).is_ok());
    }

    #[test]
    fn rejects_heat_recovery_efficiency_out_of_range() {
        let mut project = base_project();
        project.ventilation.heat_recovery_efficiency = Some(1.2);

        let err = validate_project(&project).expect_err("should reject");
        assert!(err.to_string().contains("heat_recovery_efficiency"));

        project.ventilation.heat_recovery_efficiency = Some(-0.01);
        let err = validate_project(&project).expect_err("should reject");
        assert!(err.to_string().contains("heat_recovery_efficiency"));
    }

    #[test]
    fn accepts_heat_recovery_efficiency_boundaries() {
        let mut project = base_project();
        project.ventilation.heat_recovery_efficiency = Some(0.0);
        assert!(validate_project(&project).is_ok());
        project.ventilation.heat_recovery_efficiency = Some(1.0);
        assert!(validate_project(&project).is_ok());
    }

    #[test]
    fn rejects_unknown_adjacent_room_id() {
        let mut project = base_project();
        project.rooms.push(base_room("r2", RoomFunction::Bedroom, None));
        project.rooms[0].constructions.push(ConstructionElement {
            id: "c1".to_string(),
            description: "Binnenwand".to_string(),
            area: 5.0,
            u_value: 2.0,
            boundary_type: BoundaryType::AdjacentRoom,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: Some("does-not-exist".to_string()),
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        });

        let err = validate_project(&project).expect_err("should reject");
        let msg = err.to_string();
        assert!(
            msg.contains("adjacent_room_id") && msg.contains("does-not-exist"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn accepts_known_adjacent_room_id() {
        let mut project = base_project();
        project.rooms.push(base_room("r2", RoomFunction::Bedroom, None));
        project.rooms[0].constructions.push(ConstructionElement {
            id: "c1".to_string(),
            description: "Binnenwand naar r2".to_string(),
            area: 5.0,
            u_value: 2.0,
            boundary_type: BoundaryType::AdjacentRoom,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: Some("r2".to_string()),
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        });

        assert!(validate_project(&project).is_ok());
    }

    #[test]
    fn ignores_adjacent_room_id_on_non_adjacent_boundary() {
        // A stale id on a non-AdjacentRoom boundary should not fail
        // validation — that id is not consumed by the calculation.
        let mut project = base_project();
        project.rooms[0].constructions.push(ConstructionElement {
            id: "c1".to_string(),
            description: "Buitenwand".to_string(),
            area: 5.0,
            u_value: 0.3,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: Some("orphaned".to_string()),
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        });

        assert!(validate_project(&project).is_ok());
    }
}
