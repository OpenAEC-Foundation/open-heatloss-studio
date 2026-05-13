//! # ISSO 51 IFCX Bridge
//!
//! Reads and writes IFCX (IFC5 JSON) documents with `isso51::` namespace
//! extensions for warmteverliesberekening data.
//!
//! ## Usage
//!
//! ```rust,ignore
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

/// Generate the JSON schema for the `IfcxDocument` type.
///
/// Includes all isso51:: namespace types via schemars.
pub fn ifcx_schema() -> String {
    let schema = schemars::schema_for!(IfcxDocument);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}

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
                        catalog_ref: None,
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
                        catalog_ref: None,
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
        // HashMap iteration order is non-deterministic, so check both values exist.
        let mut u_values: Vec<f64> = r.constructions.iter().map(|c| c.u_value).collect();
        u_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(u_values, vec![0.22, 1.1]);
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

    #[test]
    fn test_adjacent_room_roundtrip() {
        let mut project = test_project();

        // Add a second room (Slaapkamer).
        let bedroom = Room {
            id: "r2".to_string(),
            name: "Slaapkamer".to_string(),
            function: RoomFunction::Bedroom,
            custom_temperature: None,
            floor_area: 15.0,
            height: 2.6,
            constructions: vec![
                construction::ConstructionElement {
                    id: "c1".to_string(),
                    description: "Buitenwand".to_string(),
                    area: 8.0,
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
                    catalog_ref: None,
                },
                construction::ConstructionElement {
                    id: "c2".to_string(),
                    description: "Binnenwand naar woonkamer".to_string(),
                    area: 6.0,
                    u_value: 1.5,
                    boundary_type: enums::BoundaryType::AdjacentRoom,
                    material_type: enums::MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: Some("r1".to_string()), // → woonkamer
                    adjacent_temperature: None,
                    vertical_position: enums::VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                },
            ],
            heating_system: HeatingSystem::RadiatorLt,
            ventilation_rate: Some(14.0),
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            internal_air_temperature: None,
            clamp_positive: true,
        };
        project.rooms.push(bedroom);

        // Add reciprocal wall on woonkamer pointing to slaapkamer.
        project.rooms[0]
            .constructions
            .push(construction::ConstructionElement {
                id: "c3".to_string(),
                description: "Binnenwand naar slaapkamer".to_string(),
                area: 6.0,
                u_value: 1.5,
                boundary_type: enums::BoundaryType::AdjacentRoom,
                material_type: enums::MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: Some("r2".to_string()), // → slaapkamer
                adjacent_temperature: None,
                vertical_position: enums::VerticalPosition::Wall,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                catalog_ref: None,
            });

        // Roundtrip
        let doc = project_to_ifcx(&project);
        let restored = project_from_ifcx(&doc).unwrap();

        // Verify adjacent_room_id survived the roundtrip.
        let woonkamer = &restored.rooms[0];
        let slaapkamer = &restored.rooms[1];

        // Find the adjacent room construction on slaapkamer.
        let adj_constr = slaapkamer
            .constructions
            .iter()
            .find(|c| c.boundary_type == enums::BoundaryType::AdjacentRoom)
            .expect("Slaapkamer should have an AdjacentRoom construction");
        assert_eq!(
            adj_constr.adjacent_room_id.as_deref(),
            Some(woonkamer.id.as_str()),
            "Slaapkamer's adjacent wall should point to woonkamer"
        );

        // Reciprocal: woonkamer's adjacent wall should point to slaapkamer.
        let adj_constr_wk = woonkamer
            .constructions
            .iter()
            .find(|c| c.boundary_type == enums::BoundaryType::AdjacentRoom)
            .expect("Woonkamer should have an AdjacentRoom construction");
        assert_eq!(
            adj_constr_wk.adjacent_room_id.as_deref(),
            Some(slaapkamer.id.as_str()),
            "Woonkamer's adjacent wall should point to slaapkamer"
        );
    }

    #[test]
    fn test_ground_params_roundtrip() {
        let mut project = test_project();

        // Add a floor construction with ground params.
        project.rooms[0]
            .constructions
            .push(construction::ConstructionElement {
                id: "c3".to_string(),
                description: "Begane grond vloer".to_string(),
                area: 30.0,
                u_value: 0.3,
                boundary_type: enums::BoundaryType::Ground,
                material_type: enums::MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: None,
                vertical_position: enums::VerticalPosition::Floor,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: Some(construction::GroundParameters {
                    u_equivalent: 0.25,
                    ground_water_factor: 1.15,
                    fg2: 0.45,
                }),
                has_embedded_heating: false,
                catalog_ref: None,
            });

        // Roundtrip
        let doc = project_to_ifcx(&project);
        let restored = project_from_ifcx(&doc).unwrap();

        let floor = restored.rooms[0]
            .constructions
            .iter()
            .find(|c| c.boundary_type == enums::BoundaryType::Ground)
            .expect("Should have a Ground construction");

        let gp = floor.ground_params.as_ref().expect("Should have ground_params");
        assert!((gp.u_equivalent - 0.25).abs() < 1e-10);
        assert!((gp.ground_water_factor - 1.15).abs() < 1e-10);
        assert!((gp.fg2 - 0.45).abs() < 1e-10);
    }

    #[test]
    fn test_project_info_roundtrip() {
        let mut project = test_project();
        project.info.project_number = Some("2025-042".to_string());
        project.info.address = Some("Keizersgracht 123, Amsterdam".to_string());
        project.info.client = Some("Woningcorporatie XYZ".to_string());
        project.info.date = Some("2025-03-15".to_string());
        project.info.engineer = Some("Ir. J. de Vries".to_string());
        project.info.notes = Some("Renovatie bestaand portiekblok".to_string());

        // Roundtrip
        let doc = project_to_ifcx(&project);
        let restored = project_from_ifcx(&doc).unwrap();

        assert_eq!(
            restored.info.project_number.as_deref(),
            Some("2025-042")
        );
        assert_eq!(
            restored.info.address.as_deref(),
            Some("Keizersgracht 123, Amsterdam")
        );
        assert_eq!(
            restored.info.client.as_deref(),
            Some("Woningcorporatie XYZ")
        );
        assert_eq!(restored.info.date.as_deref(), Some("2025-03-15"));
        assert_eq!(
            restored.info.engineer.as_deref(),
            Some("Ir. J. de Vries")
        );
        assert_eq!(
            restored.info.notes.as_deref(),
            Some("Renovatie bestaand portiekblok")
        );
    }

    #[test]
    fn test_ifcx_schema_generation() {
        let schema = ifcx_schema();
        assert!(!schema.is_empty());
        let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();
        // Should reference IfcxDocument as root type
        assert!(parsed.get("title").is_some() || parsed.get("$ref").is_some());
    }
}
