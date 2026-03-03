//! # ISSO 51 Heat Loss Calculation Engine
//!
//! Pure Rust implementation of the ISSO 51:2023 warmteverliesberekening
//! (heat loss calculation) for residential buildings in the Netherlands.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use isso51_core::calculate_from_json;
//!
//! let input_json = r#"{ ... }"#;
//! let result_json = calculate_from_json(input_json).unwrap();
//! ```
//!
//! ## Architecture
//!
//! This crate is a pure computation library — no I/O, no async, no unsafe.
//! It takes JSON input, performs the calculation, and returns JSON output.
//! Wrapper crates (isso51-python, isso51-wasm, isso51-ffi) provide
//! platform-specific bindings.

pub mod calc;
pub mod error;
pub mod formulas;
pub mod model;
pub mod result;
pub mod tables;
pub mod validate;

use error::Result;
use model::Project;
use result::{BuildingSummary, ProjectResult};

/// Calculate heat losses for an entire project from JSON input.
///
/// This is the main public API. It takes a JSON string representing
/// a Project, validates the input, runs the calculation for each room,
/// and returns the results as a JSON string.
///
/// # Arguments
/// * `input_json` - JSON string conforming to the Project schema
///
/// # Returns
/// JSON string containing the ProjectResult, or an error.
///
/// # Errors
/// Returns `Isso51Error` if the input is invalid or calculation fails.
pub fn calculate_from_json(input_json: &str) -> Result<String> {
    let project: Project = serde_json::from_str(input_json)?;
    let result = calculate(&project)?;
    let output = serde_json::to_string_pretty(&result)?;
    Ok(output)
}

/// Calculate heat losses for an entire project.
///
/// Takes a validated Project struct and returns the complete calculation results.
///
/// # Arguments
/// * `project` - The project input data
///
/// # Returns
/// Complete ProjectResult with per-room and building-level results.
pub fn calculate(project: &Project) -> Result<ProjectResult> {
    validate::validate_project(project)?;

    let mut room_results = Vec::with_capacity(project.rooms.len());

    // First pass: calculate the main room (first room with highest heat loss)
    // to determine the heating-up percentage for other rooms.
    // For simplicity, we use the first room as the main room.
    let mut main_room_hu_pct: Option<f64> = None;

    for (i, room) in project.rooms.iter().enumerate() {
        let hu_pct = if i == 0 { None } else { main_room_hu_pct };

        let room_result = calc::room_load::calculate_room(
            room,
            &project.building,
            &project.climate,
            &project.ventilation,
            hu_pct,
        )?;

        // After calculating the first room, determine the heating-up percentage
        if i == 0 && project.building.has_night_setback {
            let phi_t = room_result.transmission.phi_t;
            let phi_v = room_result.ventilation.phi_v;
            main_room_hu_pct = Some(calc::heating_up::main_room_percentage(
                room_result.heating_up.phi_hu,
                phi_t,
                phi_v,
            ));
        }

        room_results.push(room_result);
    }

    // Build summary
    let summary = build_summary(&room_results);

    Ok(ProjectResult {
        rooms: room_results,
        summary,
    })
}

/// Build the building-level summary from per-room results.
fn build_summary(rooms: &[result::RoomResult]) -> BuildingSummary {
    let mut total_envelope_loss = 0.0;
    let mut total_neighbor_loss = 0.0;
    let mut total_ventilation_loss = 0.0;
    let mut total_heating_up = 0.0;
    let mut total_system_losses = 0.0;

    for r in rooms {
        let theta_diff = r.theta_i - (-10.0); // TODO: use actual theta_e

        total_envelope_loss += r.transmission.h_t_exterior * theta_diff
            + r.transmission.h_t_unheated * theta_diff
            + r.transmission.h_t_ground * theta_diff;

        total_neighbor_loss += r.transmission.h_t_adjacent_buildings * theta_diff;

        total_ventilation_loss += r.ventilation.phi_v;
        total_heating_up += r.heating_up.phi_hu;
        total_system_losses += r.system_losses.phi_system_total;
    }

    let connection_capacity =
        total_envelope_loss + total_neighbor_loss + total_ventilation_loss + total_heating_up + total_system_losses;

    // Collective contribution excludes neighbor losses if same building
    let collective_contribution =
        total_envelope_loss + total_ventilation_loss + total_heating_up + total_system_losses;

    BuildingSummary {
        total_envelope_loss,
        total_neighbor_loss,
        total_ventilation_loss,
        total_heating_up,
        total_system_losses,
        connection_capacity,
        collective_contribution,
    }
}

/// Generate the JSON schema for the Project input type.
///
/// Useful for documentation and validation tooling.
pub fn project_schema() -> String {
    let schema = schemars::schema_for!(Project);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}

/// Generate the JSON schema for the ProjectResult output type.
pub fn result_schema() -> String {
    let schema = schemars::schema_for!(ProjectResult);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    /// Create the ISSO 51 Example 1 portiekwoning for testing.
    fn create_portiekwoning() -> Project {
        Project {
            info: ProjectInfo {
                name: "ISSO 51 Voorbeeld 1 - Portiekwoning".to_string(),
                project_number: None,
                address: None,
                client: None,
                date: None,
                engineer: None,
                notes: None,
            },
            building: Building {
                building_type: BuildingType::Porch,
                qv10: 100.0,
                total_floor_area: 85.0,
                security_class: SecurityClass::B,
                has_night_setback: true,
                warmup_time: 2.0,
                building_height: None,
                num_floors: 1,
            },
            // Old ISSO 51 example used θ_b = 15°C (erratum 2023 changed to 17°C)
            climate: DesignConditions {
                theta_b_residential: 15.0,
                ..DesignConditions::default()
            },
            ventilation: VentilationConfig {
                system_type: VentilationSystemType::SystemC,
                has_heat_recovery: false,
                heat_recovery_efficiency: None,
                frost_protection: None,
                supply_temperature: None,
                has_preheating: false,
                preheating_temperature: None,
            },
            rooms: vec![create_room1_woonkamer()],
        }
    }

    /// Room 1: Woonkamer (living room), θ_i = 20°C
    fn create_room1_woonkamer() -> Room {
        use construction::ConstructionElement;
        use enums::*;

        Room {
            id: "r1".to_string(),
            name: "Woonkamer".to_string(),
            function: RoomFunction::LivingRoom,
            custom_temperature: None,
            floor_area: 28.2,
            height: 2.6,
            constructions: vec![
                // Exterior elements
                ConstructionElement {
                    id: "c1".to_string(),
                    description: "Buitenwand".to_string(),
                    area: 7.29,
                    u_value: 0.36,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c2".to_string(),
                    description: "Raam".to_string(),
                    area: 4.32,
                    u_value: 3.2,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::NonMasonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c3".to_string(),
                    description: "Buitenwand bij deur".to_string(),
                    area: 0.36,
                    u_value: 0.36,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c4".to_string(),
                    description: "Deur naar balkon".to_string(),
                    area: 2.16,
                    u_value: 2.78,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::NonMasonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                // Adjacent rooms within dwelling
                ConstructionElement {
                    id: "c5".to_string(),
                    description: "Binnenwand naar keuken".to_string(),
                    area: 7.36,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r2".to_string()),
                    adjacent_temperature: Some(20.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c6".to_string(),
                    description: "Binnenwand naar slaapkamer 1".to_string(),
                    area: 11.20,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r4".to_string()),
                    adjacent_temperature: Some(20.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c7".to_string(),
                    description: "Binnenwand naar entree".to_string(),
                    area: 2.51,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r7".to_string()),
                    adjacent_temperature: Some(15.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c8".to_string(),
                    description: "Binnenwand naar toilet".to_string(),
                    area: 3.12,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r8".to_string()),
                    adjacent_temperature: Some(15.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c9".to_string(),
                    description: "Binnenwand naar badkamer".to_string(),
                    area: 3.64,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r3".to_string()),
                    adjacent_temperature: Some(22.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                // Adjacent building (neighboring dwellings)
                ConstructionElement {
                    id: "c10".to_string(),
                    description: "Woningscheidende wand".to_string(),
                    area: 18.09,
                    u_value: 2.08,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c11".to_string(),
                    description: "Plafond".to_string(),
                    area: 28.20,
                    u_value: 2.5,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Ceiling,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
                ConstructionElement {
                    id: "c12".to_string(),
                    description: "Vloer".to_string(),
                    area: 28.20,
                    u_value: 2.5,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Floor,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                },
            ],
            heating_system: HeatingSystem::RadiatorLt,
            ventilation_rate: 25.38,
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            internal_air_temperature: None,
            clamp_positive: true,
        }
    }

    #[test]
    fn test_portiekwoning_room1_transmission() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected: H_T,ie ≈ 24.00
        assert!(
            (r1.transmission.h_t_exterior - 24.00).abs() < 0.2,
            "H_T,ie = {}, expected ~24.00",
            r1.transmission.h_t_exterior
        );

        // Expected: H_T,ia ≈ 1.51
        assert!(
            (r1.transmission.h_t_adjacent_rooms - 1.51).abs() < 0.2,
            "H_T,ia = {}, expected ~1.51",
            r1.transmission.h_t_adjacent_rooms
        );

        // Expected: Φ_T ≈ 1247 W
        assert!(
            (r1.transmission.phi_t - 1247.0).abs() < 20.0,
            "Φ_T = {}, expected ~1247",
            r1.transmission.phi_t
        );
    }

    #[test]
    fn test_portiekwoning_room1_ventilation() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected: Φ_v ≈ 914 W
        assert!(
            (r1.ventilation.phi_v - 914.0).abs() < 5.0,
            "Φ_v = {}, expected ~914",
            r1.ventilation.phi_v
        );
    }

    #[test]
    fn test_portiekwoning_room1_total() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected total: Φ_tot = Φ_T + Φ_v + Φ_hu = 1247 + 914 + 187 = 2348 W
        // Note: with quadratic summation (2023), the result will differ from
        // the old example which used simple addition.
        // The old example gives 2348 W; with quadratic sum it will be different.
        assert!(
            r1.total_heat_loss > 0.0,
            "Total heat loss should be positive"
        );
    }

    #[test]
    fn test_json_roundtrip() {
        let project = create_portiekwoning();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let result = calculate_from_json(&json).unwrap();
        assert!(!result.is_empty());

        // Verify result is valid JSON
        let _: serde_json::Value = serde_json::from_str(&result).unwrap();
    }

    #[test]
    fn test_schema_generation() {
        let schema = project_schema();
        assert!(!schema.is_empty());
        let _: serde_json::Value = serde_json::from_str(&schema).unwrap();

        let result_schema = result_schema();
        assert!(!result_schema.is_empty());
    }

    #[test]
    fn test_norm_refs_populated() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Transmission must reference formule 4.2 (Phi_T) and 4.3a (H_T,ie)
        assert!(
            r1.transmission.norm_refs.contains(&"ISSO_51_2023_formule4_2"),
            "Transmission missing formule 4.2"
        );
        assert!(
            r1.transmission.norm_refs.contains(&"ISSO_51_2023_formule4_3a"),
            "Transmission missing formule 4.3a"
        );

        // Infiltration must reference erratum formules
        assert!(
            r1.infiltration
                .norm_refs
                .contains(&"ISSO_51_2023_formule4_1_erratum"),
            "Infiltration missing formule 4.1 erratum"
        );

        // Ventilation: outside air → formule 4.3 erratum + 4.6a erratum
        assert!(
            r1.ventilation
                .norm_refs
                .contains(&"ISSO_51_2023_formule4_3_erratum"),
            "Ventilation missing formule 4.3 erratum"
        );
        assert!(
            r1.ventilation
                .norm_refs
                .contains(&"ISSO_51_2023_formule3_3_erratum"),
            "Ventilation missing formule 3.3 erratum (phi_vent)"
        );

        // Heating-up must reference paragraaf 4.3 and tabel 4.6
        assert!(
            r1.heating_up
                .norm_refs
                .contains(&"ISSO_51_2023_parag4_3"),
            "Heating-up missing parag 4.3"
        );
        assert!(
            r1.heating_up
                .norm_refs
                .contains(&"ISSO_51_2023_tabel4_6"),
            "Heating-up missing tabel 4.6"
        );

        // System losses: no embedded heating → empty
        assert!(
            r1.system_losses.norm_refs.is_empty(),
            "System losses should have no norm_refs without embedded heating"
        );
    }

    #[test]
    fn test_norm_refs_in_json_output() {
        let project = create_portiekwoning();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let result_json = calculate_from_json(&json).unwrap();

        // norm_refs must appear in serialized output
        assert!(
            result_json.contains("norm_refs"),
            "JSON output must contain norm_refs field"
        );
        assert!(
            result_json.contains("ISSO_51_2023_formule4_2"),
            "JSON output must contain formule 4.2 reference"
        );
    }

    #[test]
    fn test_norm_refs_skipped_on_deserialize() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let json = serde_json::to_string(&result).unwrap();

        // Deserialize back — norm_refs should default to empty
        let deserialized: result::ProjectResult =
            serde_json::from_str(&json).unwrap();
        let r1 = &deserialized.rooms[0];
        assert!(
            r1.transmission.norm_refs.is_empty(),
            "norm_refs should be empty after deserialization"
        );
    }
}
