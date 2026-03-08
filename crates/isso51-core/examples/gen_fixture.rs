//! Generate the ISSO 51 portiekwoning test fixture as JSON.
use isso51_core::model::construction::ConstructionElement;
use isso51_core::model::enums::*;
use isso51_core::model::*;

fn main() {
    let project = create_portiekwoning();
    let json = serde_json::to_string_pretty(&project).unwrap();
    std::fs::write("tests/fixtures/portiekwoning.json", &json).unwrap();
    println!("Written portiekwoning.json ({} bytes)", json.len());

    // Also run the calculation and save the result
    let result = isso51_core::calculate(&project).unwrap();
    let result_json = serde_json::to_string_pretty(&result).unwrap();
    std::fs::write("tests/fixtures/portiekwoning_result.json", &result_json).unwrap();
    println!(
        "Written portiekwoning_result.json ({} bytes)",
        result_json.len()
    );
}

fn create_portiekwoning() -> Project {
    Project {
        info: ProjectInfo {
            name: "ISSO 51 Voorbeeld 1 - Portiekwoning".to_string(),
            project_number: Some("ISSO-51-V1".to_string()),
            address: None,
            client: None,
            date: Some("2024-01-01".to_string()),
            engineer: Some("3BM Bouwkunde".to_string()),
            notes: Some("Tussenliggende portiekwoning eerste etage".to_string()),
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
            infiltration_method: InfiltrationMethod::PerExteriorArea,
        },
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
        rooms: vec![
            create_room(
                "r1",
                "Woonkamer",
                RoomFunction::LivingRoom,
                28.2,
                25.38,
                1.0,
                None,
                woonkamer_constructions(),
            ),
            create_room(
                "r2",
                "Keuken",
                RoomFunction::Kitchen,
                14.94,
                0.0,
                0.0,
                None,
                keuken_constructions(),
            ),
            create_room(
                "r3",
                "Badkamer",
                RoomFunction::Bathroom,
                4.77,
                14.0,
                0.0,
                Some(15.0),
                badkamer_constructions(),
            ),
            create_room(
                "r4",
                "Slaapkamer 1",
                RoomFunction::Bedroom,
                8.76,
                7.88,
                1.0,
                None,
                slaapkamer1_constructions(),
            ),
            create_room(
                "r5",
                "Slaapkamer 2",
                RoomFunction::Bedroom,
                12.9,
                11.6,
                1.0,
                None,
                slaapkamer2_constructions(),
            ),
            create_room(
                "r6",
                "Slaapkamer 3",
                RoomFunction::Bedroom,
                13.67,
                12.3,
                1.0,
                None,
                slaapkamer3_constructions(),
            ),
            create_room(
                "r7",
                "Entree",
                RoomFunction::Hallway,
                6.24,
                4.4,
                1.0,
                None,
                entree_constructions(),
            ),
            create_room(
                "r8",
                "Toilet",
                RoomFunction::Toilet,
                1.32,
                0.0,
                0.0,
                Some(15.0),
                toilet_constructions(),
            ),
        ],
    }
}

fn create_room(
    id: &str,
    name: &str,
    func: RoomFunction,
    area: f64,
    vent_rate: f64,
    frac_outside: f64,
    internal_temp: Option<f64>,
    constructions: Vec<ConstructionElement>,
) -> Room {
    Room {
        id: id.to_string(),
        name: name.to_string(),
        function: func,
        custom_temperature: None,
        floor_area: area,
        height: 2.6,
        constructions,
        heating_system: HeatingSystem::RadiatorLt,
        ventilation_rate: Some(vent_rate),
        has_mechanical_exhaust: false,
        has_mechanical_supply: false,
        fraction_outside_air: frac_outside,
        supply_air_temperature: None,
        internal_air_temperature: internal_temp,
        clamp_positive: true,
    }
}

fn ext(id: &str, desc: &str, area: f64, u: f64, mat: MaterialType) -> ConstructionElement {
    ConstructionElement {
        id: id.to_string(),
        description: desc.to_string(),
        area,
        u_value: u,
        boundary_type: BoundaryType::Exterior,
        material_type: mat,
        temperature_factor: None,
        adjacent_room_id: None,
        adjacent_temperature: None,
        vertical_position: VerticalPosition::Wall,
        use_forfaitaire_thermal_bridge: true,
        custom_delta_u_tb: None,
        ground_params: None,
        has_embedded_heating: false,
    }
}

fn adj(id: &str, desc: &str, area: f64, u: f64, adj_temp: f64) -> ConstructionElement {
    ConstructionElement {
        id: id.to_string(),
        description: desc.to_string(),
        area,
        u_value: u,
        boundary_type: BoundaryType::AdjacentRoom,
        material_type: MaterialType::Masonry,
        temperature_factor: None,
        adjacent_room_id: None,
        adjacent_temperature: Some(adj_temp),
        vertical_position: VerticalPosition::Wall,
        use_forfaitaire_thermal_bridge: false,
        custom_delta_u_tb: None,
        ground_params: None,
        has_embedded_heating: false,
    }
}

fn nb(id: &str, desc: &str, area: f64, u: f64, pos: VerticalPosition) -> ConstructionElement {
    ConstructionElement {
        id: id.to_string(),
        description: desc.to_string(),
        area,
        u_value: u,
        boundary_type: BoundaryType::AdjacentBuilding,
        material_type: MaterialType::Masonry,
        temperature_factor: None,
        adjacent_room_id: None,
        adjacent_temperature: None,
        vertical_position: pos,
        use_forfaitaire_thermal_bridge: false,
        custom_delta_u_tb: None,
        ground_params: None,
        has_embedded_heating: false,
    }
}

fn unheated(id: &str, desc: &str, area: f64, u: f64, f_k: f64) -> ConstructionElement {
    ConstructionElement {
        id: id.to_string(),
        description: desc.to_string(),
        area,
        u_value: u,
        boundary_type: BoundaryType::UnheatedSpace,
        material_type: MaterialType::Masonry,
        temperature_factor: Some(f_k),
        adjacent_room_id: None,
        adjacent_temperature: None,
        vertical_position: VerticalPosition::Wall,
        use_forfaitaire_thermal_bridge: false,
        custom_delta_u_tb: None,
        ground_params: None,
        has_embedded_heating: false,
    }
}

fn woonkamer_constructions() -> Vec<ConstructionElement> {
    vec![
        ext("w1", "Buitenwand", 7.29, 0.36, MaterialType::Masonry),
        ext("w2", "Raam", 4.32, 3.2, MaterialType::NonMasonry),
        ext(
            "w3",
            "Buitenwand bij deur",
            0.36,
            0.36,
            MaterialType::Masonry,
        ),
        ext(
            "w4",
            "Deur naar balkon",
            2.16,
            2.78,
            MaterialType::NonMasonry,
        ),
        adj("w5", "Naar keuken", 7.36, 2.17, 20.0),
        adj("w6", "Naar slaapkamer 1", 11.20, 2.17, 20.0),
        adj("w7", "Naar entree", 2.51, 2.17, 15.0),
        adj("w8", "Naar toilet", 3.12, 2.17, 15.0),
        adj("w9", "Naar badkamer", 3.64, 2.17, 22.0),
        nb(
            "w10",
            "Woningscheidende wand",
            18.09,
            2.08,
            VerticalPosition::Wall,
        ),
        nb("w11", "Plafond", 28.20, 2.5, VerticalPosition::Ceiling),
        nb("w12", "Vloer", 28.20, 2.5, VerticalPosition::Floor),
    ]
}

fn keuken_constructions() -> Vec<ConstructionElement> {
    vec![
        ext("k1", "Buitenwand", 3.98, 0.36, MaterialType::Masonry),
        ext("k2", "Raam", 1.20, 3.20, MaterialType::NonMasonry),
        ext("k3", "Deur", 2.18, 2.78, MaterialType::NonMasonry),
        adj("k4", "Naar badkamer", 4.86, 2.17, 22.0),
        adj("k5", "Naar slaapkamer 2", 9.05, 2.17, 20.0),
        adj("k6", "Naar woonkamer", 7.36, 2.17, 20.0),
        nb(
            "k7",
            "Woningscheidende wand",
            13.91,
            2.08,
            VerticalPosition::Wall,
        ),
        nb("k8", "Plafond", 14.94, 2.5, VerticalPosition::Ceiling),
        nb("k9", "Vloer", 14.94, 2.5, VerticalPosition::Floor),
    ]
}

fn badkamer_constructions() -> Vec<ConstructionElement> {
    vec![
        adj("b1", "Naar keuken", 4.86, 2.17, 20.0),
        adj("b2", "Naar woonkamer", 3.64, 2.17, 20.0),
        adj("b3", "Naar toilet", 3.04, 2.17, 15.0),
        adj("b4", "Naar entree", 4.86, 2.17, 15.0),
        adj("b5", "Naar slaapkamer 2", 6.78, 2.17, 20.0),
        nb("b6", "Plafond", 4.77, 2.5, VerticalPosition::Ceiling),
        nb("b7", "Vloer", 4.77, 2.5, VerticalPosition::Floor),
    ]
}

fn slaapkamer1_constructions() -> Vec<ConstructionElement> {
    vec![
        ext("s1_1", "Buitenwand", 2.85, 0.36, MaterialType::Masonry),
        ext("s1_2", "Raam", 1.60, 3.20, MaterialType::NonMasonry),
        ext("s1_3", "Deur", 2.78, 2.78, MaterialType::NonMasonry),
        adj("s1_4", "Naar woonkamer", 11.20, 2.17, 20.0),
        adj("s1_5", "Naar entree", 8.37, 2.17, 15.0),
        unheated("s1_6", "Naar trappenhuis", 8.24, 0.36, 0.5),
        nb("s1_7", "Plafond", 8.76, 2.5, VerticalPosition::Ceiling),
        nb("s1_8", "Vloer", 8.76, 2.5, VerticalPosition::Floor),
    ]
}

fn slaapkamer2_constructions() -> Vec<ConstructionElement> {
    vec![
        ext("s2_1", "Buitenwand", 7.34, 0.36, MaterialType::Masonry),
        ext("s2_2", "Raam", 2.56, 3.20, MaterialType::NonMasonry),
        adj("s2_3", "Naar entree", 3.12, 2.17, 15.0),
        adj("s2_4", "Naar badkamer", 6.78, 2.17, 22.0),
        adj("s2_5", "Naar slaapkamer 3", 9.08, 2.17, 20.0),
        nb("s2_6", "Plafond", 12.9, 2.5, VerticalPosition::Ceiling),
        nb("s2_7", "Vloer", 12.9, 2.5, VerticalPosition::Floor),
    ]
}

fn slaapkamer3_constructions() -> Vec<ConstructionElement> {
    vec![
        ext("s3_1", "Buitenwand", 4.40, 0.36, MaterialType::Masonry),
        ext("s3_2", "Raam", 2.89, 3.20, MaterialType::NonMasonry),
        adj("s3_3", "Naar slaapkamer 2", 9.08, 2.17, 20.0),
        adj("s3_4", "Naar entree", 4.59, 2.17, 15.0),
        unheated("s3_5", "Naar trappenhuis", 7.29, 0.36, 0.5),
        nb(
            "s3_6",
            "Woningscheidende wand",
            13.64,
            2.08,
            VerticalPosition::Wall,
        ),
        nb("s3_7", "Plafond", 13.67, 2.5, VerticalPosition::Ceiling),
        nb("s3_8", "Vloer", 13.67, 2.5, VerticalPosition::Floor),
    ]
}

fn entree_constructions() -> Vec<ConstructionElement> {
    vec![
        adj("e1", "Naar badkamer", 4.86, 2.17, 22.0),
        adj("e2", "Naar toilet", 6.38, 2.17, 15.0),
        adj("e3", "Naar woonkamer", 2.51, 2.17, 20.0),
        adj("e4", "Naar slaapkamer 1", 8.37, 2.17, 20.0),
        adj("e5", "Naar slaapkamer 3", 4.59, 2.17, 20.0),
        adj("e6", "Naar slaapkamer 2", 3.12, 2.17, 20.0),
        unheated("e7", "Naar trappenhuis", 6.19, 0.36, 0.5),
        nb("e8", "Plafond", 6.24, 2.5, VerticalPosition::Ceiling),
        nb("e9", "Vloer", 6.24, 2.5, VerticalPosition::Floor),
    ]
}

fn toilet_constructions() -> Vec<ConstructionElement> {
    vec![
        adj("t1", "Naar woonkamer", 3.12, 2.17, 20.0),
        adj("t2", "Naar entree", 6.38, 2.17, 15.0),
        adj("t3", "Naar badkamer", 3.04, 2.17, 22.0),
        nb("t4", "Plafond", 1.32, 2.5, VerticalPosition::Ceiling),
        nb("t5", "Vloer", 1.32, 2.5, VerticalPosition::Floor),
    ]
}
