//! # ISSO 53 Heat Loss Calculation Engine
//!
//! Pure Rust implementation of the ISSO 53:2016 warmteverliesberekening
//! (heat loss calculation) for utility buildings with ceiling heights up to 4 meters
//! in the Netherlands.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use isso53_core::calculate_from_json;
//!
//! let input_json = r#"{ ... }"#;
//! let result_json = calculate_from_json(input_json).unwrap();
//! ```
//!
//! ## Architecture
//!
//! This crate is a pure computation library — no I/O, no async, no unsafe.
//! It takes JSON input, performs the calculation, and returns JSON output.
//! Wrapper crates (isso53-python, isso53-wasm, isso53-ffi) provide
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
/// Returns `Isso53Error` if the input is invalid or calculation fails.
pub fn calculate_from_json(input_json: &str) -> Result<String> {
    let project: Project = serde_json::from_str(input_json)?;
    let result = calculate(&project)?;
    Ok(serde_json::to_string_pretty(&result)?)
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
    for room in &project.rooms {
        room_results.push(calc::room_load::calculate_room(
            room, &project.rooms, &project.building, &project.climate,
            &project.ventilation, &project.infiltration_method, &project.heating_up,
        )?);
    }

    let z = 0.5;  // Default for tabel 5.1 - TODO: lookup from tables::source_fraction
    let phi_source_individual = calc::source_capacity::calculate_individual(&room_results, z, project.climate.theta_e)?;
    let phi_source_collective = calc::source_capacity::calculate_collective(&room_results, z, project.climate.theta_e)?;
    let phi_shell = calc::shell::calculate_shell(project)?;

    // Calculate building-level summaries
    let mut total_transmission_loss = 0.0;
    let mut total_ventilation_loss = 0.0;
    let mut total_infiltration_loss = 0.0;
    let mut total_heating_up = 0.0;
    let mut total_system_losses = 0.0;
    let mut total_internal_gains = 0.0;
    let mut total_building_heat_loss = 0.0;

    for room_result in &room_results {
        total_transmission_loss += room_result.phi_t;
        total_ventilation_loss += room_result.phi_v;
        total_infiltration_loss += room_result.phi_i;
        total_heating_up += room_result.phi_hu;
        total_system_losses += room_result.phi_system;
        total_internal_gains += room_result.phi_gain;
        total_building_heat_loss += room_result.total_heat_loss;
    }

    Ok(ProjectResult {
        rooms: room_results,
        summary: BuildingSummary {
            total_transmission_loss,
            total_ventilation_loss,
            total_infiltration_loss,
            total_heating_up,
            total_system_losses,
            total_internal_gains,
            total_building_heat_loss,
            connection_capacity_individual: phi_source_individual,
            connection_capacity_collective: phi_source_collective,
            shell_heat_loss: phi_shell,
            infiltration_reduction_factor_z: z,
        },
    })
}

/// Base URL for published schemas.
const SCHEMA_BASE_URL: &str = "https://warmteverlies.open-aec.com/schemas/v1";

/// Current schema version.
const SCHEMA_VERSION: &str = "1.0.0";

/// Generate the JSON schema for the Project input type.
///
/// Useful for documentation and validation tooling.
pub fn project_schema() -> String {
    let schema = schemars::schema_for!(Project);
    add_schema_metadata(schema, "project")
}

/// Generate the JSON schema for the ProjectResult output type.
pub fn result_schema() -> String {
    let schema = schemars::schema_for!(ProjectResult);
    add_schema_metadata(schema, "result")
}

/// Add `$id` and `version` to a generated JSON schema.
fn add_schema_metadata(schema: schemars::schema::RootSchema, name: &str) -> String {
    let mut value = serde_json::to_value(&schema).unwrap_or_default();
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            serde_json::Value::String(format!(
                "{SCHEMA_BASE_URL}/{name}.schema.json"
            )),
        );
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(SCHEMA_VERSION.to_string()),
        );
    }
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn test_crate_compiles_and_validate_rejects_tall_room() {
        let project = create_test_project_with_tall_room();
        let result = validate::validate_project(&project);

        assert!(result.is_err(), "Validation should reject room with height > 4.0m");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("ISSO 57"),
            "Error should reference ISSO 57 for tall rooms: {}",
            error
        );
    }

    /// Create a test project with a room that's too tall (>4m) for ISSO 53.
    fn create_test_project_with_tall_room() -> Project {
        Project {
            info: ProjectInfo {
                name: "Test Project - Tall Room".to_string(),
                project_number: None,
                address: None,
                client: None,
                date: None,
                engineer: None,
                notes: None,
            },
            building: Building {
                building_shape: BuildingShape::Meerlaags,
                construction_year: 2020,
                building_position: GebouwTypePositie::MeerlaagsTussen,
                ventilation_system: VentilationSystemType::SystemB,
                thermal_mass: ThermalMass::Gemiddeld,
                wind_pressure_type: crate::model::enums::GebouwTypeWinddruk::MeerlaagsStandaard,
                building_height: None,
            },
            climate: DesignConditions::default(),
            ventilation: VentilationConfig {
                system_type: VentilationSystemType::SystemB,
                has_heat_recovery: false,
                heat_recovery_efficiency: None,
                frost_protection: None,
                supply_temperature: None,
                has_preheating: false,
                preheating_temperature: None,
            },
            heating_up: HeatingUpConfig {
                setback_active: false,
                p_w_per_m2: 0.0,
                warmup_minutes: 60.0,
            },
            infiltration_method: crate::calc::infiltration::InfiltrationMethod::Known {
                qv10_kar_class: crate::tables::infiltration::Qv10Class::From040To060,
            },
            rooms: vec![Room {
                id: "tall_room".to_string(),
                name: "Test Room".to_string(),
                gebruiks_functie: GebruiksFunctie::Kantoor,
                ruimte_type: RuimteType::Verblijfsruimte,
                floor_area: 25.0,
                height: 4.5, // Too tall for ISSO 53
                custom_temperature: None,
                constructions: vec![],
                bezetting: Bezetting {
                    personen: None,
                    personen_per_m2_default: None,
                },
                infiltration_reduction_z: 1.0,
            }],
        }
    }
}