//! # ISSO 51 IFCX Bridge
//!
//! Reads and writes IFCX (IFC5 JSON) documents with `isso51::` namespace
//! extensions for warmteverliesberekening data.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use isso51_ifcx::{project_from_ifcx, result_to_ifcx, calculate_ifcx};
//!
//! // Parse an IFCX document and extract a Project
//! let doc: isso51_ifcx::IfcxDocument = serde_json::from_str(input_json).unwrap();
//! let project = project_from_ifcx(&doc).unwrap();
//!
//! // Or do it all in one step: IFCX in → IFCX out (with results overlay)
//! let result_doc = calculate_ifcx(&doc).unwrap();
//! ```

pub mod document;
pub mod error;
pub mod from_ifcx;
pub mod namespace;
pub mod to_ifcx;

// Re-export key types
pub use document::{compose, IfcxDataEntry, IfcxDocument};
pub use error::{IfcxError, Result};
pub use from_ifcx::project_from_ifcx;
pub use to_ifcx::{project_to_ifcx, result_to_ifcx};

/// Full pipeline: parse IFCX → calculate → return result overlay IFCX.
///
/// Takes an IFCX document containing isso51:: input data,
/// extracts the Project, runs the calculation, and returns
/// an IFCX overlay document with isso51::calc:: result attributes.
pub fn calculate_ifcx(doc: &IfcxDocument) -> Result<IfcxDocument> {
    let project = project_from_ifcx(doc)?;
    let result = isso51_core::calculate(&project)?;
    Ok(result_to_ifcx(doc, &project, &result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use isso51_core::model::*;
    use isso51_core::result::ProjectResult;

    /// Create a minimal Project for testing roundtrips.
    fn test_project() -> Project {
        Project {
            info: ProjectInfo {
                name: "IFCX Roundtrip Test".to_string(),
                project_number: None,
                address: None,
                client: None,
                date: None,
                engineer: None,
                notes: None,
            },
            building: Building {
                building_type: BuildingType::Detached,
                qv10: 100.0,
                total_floor_area: 120.0,
                security_class: SecurityClass::B,
                has_night_setback: false,
                warmup_time: 2.0,
                building_height: None,
                num_floors: 2,
                infiltration_method: InfiltrationMethod::PerExteriorArea,
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
            rooms: vec![Room {
                id: "r1".to_string(),
                name: "Woonkamer".to_string(),
                function: RoomFunction::LivingRoom,
                custom_temperature: None,
                floor_area: 30.0,
                height: 2.6,
                constructions: vec![
                    construction::ConstructionElement {
                        id: "c1".to_string(),
                        description: "Buitenwand".to_string(),
                        area: 10.0,
                        u_value: 0.22,
                        boundary_type: enums::BoundaryType::Exterior,
                        material_type: enums::MaterialType::Masonry,
                        temperature_factor: None,
                        adjacent_room_id: None,
                        adjacent_temperature: None,
                        vertical_position: enums::VerticalPosition::Wall,
                        use_forfaitaire_thermal_bridge: true,
                        custom_delta_u_tb: None,
                        ground_params: None,
                        has_embedded_heating: false,
                    },
                    construction::ConstructionElement {
                        id: "c2".to_string(),
                        description: "Raam".to_string(),
                        area: 4.0,
                        u_value: 1.1,
                        boundary_type: enums::BoundaryType::Exterior,
                        material_type: enums::MaterialType::NonMasonry,
                        temperature_factor: None,
                        adjacent_room_id: None,
                        adjacent_temperature: None,
                        vertical_position: enums::VerticalPosition::Wall,
                        use_forfaitaire_thermal_bridge: true,
                        custom_delta_u_tb: None,
                        ground_params: None,
                        has_embedded_heating: false,
                    },
                ],
                heating_system: HeatingSystem::RadiatorLt,
                ventilation_rate: Some(21.0),
                has_mechanical_exhaust: false,
                has_mechanical_supply: false,
                fraction_outside_air: 1.0,
                supply_air_temperature: None,
                internal_air_temperature: None,
                clamp_positive: true,
            }],
        }
    }

    #[test]
    fn test_project_to_ifcx_roundtrip() {
        let original = test_project();

        // Project → IFCX
        let doc = project_to_ifcx(&original);

        // Verify IFCX structure
        assert!(!doc.data.is_empty(), "IFCX should have data entries");
        assert_eq!(doc.find_by_class("IfcProject").len(), 1);
        assert_eq!(doc.find_by_class("IfcBuilding").len(), 1);
        assert_eq!(doc.find_by_class("IfcSpace").len(), 1);

        // IFCX → Project
        let restored = project_from_ifcx(&doc).unwrap();

        // Verify key fields survived the roundtrip
        assert_eq!(restored.info.name, original.info.name);
        assert_eq!(restored.climate.theta_e, original.climate.theta_e);
        assert_eq!(restored.building.qv10, original.building.qv10);
        assert_eq!(
            restored.building.total_floor_area,
            original.building.total_floor_area
        );
        assert_eq!(restored.rooms.len(), original.rooms.len());

        let r = &restored.rooms[0];
        assert_eq!(r.floor_area, 30.0);
        assert_eq!(r.height, 2.6);
        assert_eq!(r.constructions.len(), 2);
        assert_eq!(r.constructions[0].u_value, 0.22);
        assert_eq!(r.constructions[1].u_value, 1.1);
    }

    #[test]
    fn test_ifcx_serialization_roundtrip() {
        let project = test_project();
        let doc = project_to_ifcx(&project);

        // Serialize to JSON and back
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let doc2: IfcxDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(doc.data.len(), doc2.data.len());
        assert_eq!(doc.header.ifcx_version, doc2.header.ifcx_version);
    }

    #[test]
    fn test_calculate_ifcx_pipeline() {
        let project = test_project();
        let input_doc = project_to_ifcx(&project);

        // Full pipeline: IFCX → calculate → result IFCX
        let result_doc = calculate_ifcx(&input_doc).unwrap();

        // Result overlay should have entries for spaces and building
        assert!(!result_doc.data.is_empty());

        // Check that space result attributes exist
        let space_entries = input_doc.find_by_class("IfcSpace");
        let space_path = &space_entries[0].path;
        let result_entry = result_doc.find(space_path).unwrap();

        let calc_result: namespace::Isso51CalcResult = result_entry
            .get_attr(namespace::ns::CALC_RESULT)
            .expect("Space should have isso51::calc::result");

        assert!(calc_result.phi_hl > 0.0, "Total heat loss should be > 0");
        assert!(calc_result.phi_t > 0.0, "Transmission loss should be > 0");
        assert!(calc_result.theta_int > 0.0, "θ_int should be > 0");
    }

    #[test]
    fn test_compose_overlays() {
        let project = test_project();
        let input_doc = project_to_ifcx(&project);
        let result_doc = calculate_ifcx(&input_doc).unwrap();

        // Compose input + result overlays
        let composed = compose(&[&input_doc, &result_doc]);

        // The composed set should have entries from both documents
        assert!(!composed.is_empty());

        // Space entries should have both input (isso51::room) and output (isso51::calc::result)
        let space_entries = input_doc.find_by_class("IfcSpace");
        let space_path = &space_entries[0].path;

        let merged = composed.iter().find(|e| e.path == *space_path).unwrap();
        assert!(
            merged.attributes.contains_key(namespace::ns::ROOM),
            "Composed entry should have isso51::room"
        );
        assert!(
            merged.attributes.contains_key(namespace::ns::CALC_RESULT),
            "Composed entry should have isso51::calc::result"
        );
    }
}
