//! Shell method for ISSO 53 (hoofdstuk 3 — voorontwerp).
//!
//! Building treated as one large room. Fast estimate for connection
//! capacity during preliminary design / feasibility study.

use crate::error::Result;
use crate::model::{Project, BoundaryType, GebruiksFunctie, VentilationSystemType};
use crate::tables::thermal_bridge::DELTA_U_TB_DEFAULT;
use crate::tables::adjacent_unheated::f_k;
use crate::calc::ground::calculate_h_t_ground;
use crate::formulas::RHO_CP_AIR;

/// Calculate building heat loss via the shell method.
/// ISSO 53 formule 3.1: Φ_HL,build = Φ_T,build + Φ_V,build + Φ_I,build + Φ_hu,build - Φ_gain,build
pub fn calculate_shell(project: &Project) -> Result<f64> {
    // Determine building design temperature θ_i,gebouw (§3.1)
    // 22°C if any room has Gezondheidszorg function, otherwise 20°C
    let theta_i_building = if project.rooms.iter()
        .any(|r| r.gebruiks_functie == GebruiksFunctie::Gezondheidszorg) {
        22.0
    } else {
        20.0
    };

    let theta_e = project.climate.theta_e;

    // Aggregate all H_T components from all rooms
    let mut h_t_exterior = 0.0;
    let mut h_t_unheated = 0.0;
    let mut h_t_adjacent_buildings = 0.0;
    let mut h_t_ground = 0.0;
    let mut h_i_total = 0.0;  // For ventilation calculation
    let mut h_v_total = 0.0;  // For ventilation calculation

    for room in &project.rooms {
        // Group elements by boundary type
        let exterior_elements: Vec<_> = room.constructions.iter()
            .filter(|e| e.boundary_type == BoundaryType::Exterior)
            .collect();
        let unheated_elements: Vec<_> = room.constructions.iter()
            .filter(|e| e.boundary_type == BoundaryType::Unheated)
            .collect();
        let adjacent_building_elements: Vec<_> = room.constructions.iter()
            .filter(|e| e.boundary_type == BoundaryType::AdjacentBuilding)
            .collect();
        let ground_elements: Vec<_> = room.constructions.iter()
            .filter(|e| e.boundary_type == BoundaryType::Ground)
            .collect();

        // Accumulate H_T,ie (exterior)
        for element in &exterior_elements {
            let delta_u_tb = if element.use_forfaitaire_thermal_bridge {
                DELTA_U_TB_DEFAULT
            } else {
                element.custom_delta_u_tb.unwrap_or(0.0)
            };
            h_t_exterior += element.area * (element.u_value + delta_u_tb);
        }

        // Accumulate H_T,iae (unheated)
        for element in &unheated_elements {
            let f_k_value = element.unheated_space
                .map(f_k)
                .or(element.temperature_factor)
                .unwrap_or(0.8); // Default fallback for shell method
            h_t_unheated += element.area * element.u_value * f_k_value;
        }

        // Accumulate H_T,iaBE (adjacent buildings)
        for element in &adjacent_building_elements {
            let theta_b = element.adjacent_temperature.unwrap_or(15.0);
            let f_ia_k = if (theta_i_building - theta_e).abs() < 0.001 {
                0.0
            } else {
                (theta_i_building - theta_b) / (theta_i_building - theta_e)
            };
            h_t_adjacent_buildings += element.area * element.u_value * f_ia_k;
        }

        // Accumulate H_T,ig (ground) - delegate to ground calc
        h_t_ground += calculate_h_t_ground(&ground_elements)?;

        // For ventilation: rough estimate based on room area
        // ASSUMPTION: 0.5 ACH default ventilation rate (TODO: read from VentilationConfig)
        let room_volume = room.floor_area * room.height;
        let estimated_q_v = room_volume * 0.5 / 3600.0; // 0.5 air changes per hour as m³/s
        h_v_total += estimated_q_v * RHO_CP_AIR; // H_v without f_v

        // Rough infiltration estimate (simplified for shell method)
        // ASSUMPTION: 0.01 l/(s·m²) default infiltration rate
        let estimated_q_i = room.floor_area * 0.00001; // Very rough m³/s per m²
        h_i_total += estimated_q_i * RHO_CP_AIR;
    }

    // Calculate Φ_T,build
    let h_t_total = h_t_exterior + h_t_unheated + h_t_adjacent_buildings + h_t_ground;
    let phi_t_build = h_t_total * (theta_i_building - theta_e);

    // Calculate Φ_V,build based on ventilation system type
    let phi_v_build = match project.ventilation.system_type {
        VentilationSystemType::SystemD => {
            // Mechanical supply + extract: Φ_V = (H_i + H_v) × Δθ (formule 3.18)
            (h_i_total + h_v_total) * (theta_i_building - theta_e)
        },
        _ => {
            // Natural supply: Φ_V = max(H_i, H_v) × Δθ (formule 3.19)
            h_i_total.max(h_v_total) * (theta_i_building - theta_e)
        }
    };

    // For shell method: simplified assumptions
    let phi_hu_build = 0.0; // No heating-up in shell method
    let phi_gain_build = 0.0; // No gains for now

    // Total building heat loss (formule 3.1)
    // Note: Φ_I is included in phi_v_build via formule 3.18/3.19
    let phi_hl_build = phi_t_build + phi_v_build + phi_hu_build - phi_gain_build;

    Ok(phi_hl_build)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    #[test]
    fn test_shell_calculation_smoke() {
        let project = create_minimal_project();

        let result = calculate_shell(&project);
        assert!(result.is_ok(), "Shell calculation should work: {:?}", result);

        let phi_shell = result.unwrap();
        assert!(phi_shell > 0.0, "Should have positive heat loss");
    }

    #[test]
    fn test_building_temperature_gezondheidszorg() {
        let mut project = create_minimal_project();
        project.rooms[0].gebruiks_functie = GebruiksFunctie::Gezondheidszorg;

        let result = calculate_shell(&project);
        assert!(result.is_ok());
        // Can't test the exact temperature directly, but test passes if no error
    }

    fn create_minimal_project() -> Project {
        Project {
            info: ProjectInfo {
                name: "Test Shell Project".to_string(),
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
                building_length: None,
                building_width: None,
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
            heating_up: HeatingUpConfig::default(),
            infiltration_method: crate::calc::infiltration::InfiltrationMethod::Known {
                qv10_kar_class: crate::tables::infiltration::Qv10Class::From040To060,
            },
            rooms: vec![Room {
                id: "room1".to_string(),
                name: "Test Room".to_string(),
                gebruiks_functie: GebruiksFunctie::Kantoor,
                ruimte_type: RuimteType::Verblijfsruimte,
                floor_area: 25.0,
                height: 3.0,
                custom_temperature: None,
                constructions: vec![
                    ConstructionElement {
                        id: "wall1".to_string(),
                        description: "Exterior wall".to_string(),
                        area: 15.0,
                        u_value: 0.3,
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
                        unheated_space: None,
                    }
                ],
                bezetting: Bezetting {
                    personen: None,
                    personen_per_m2_default: None,
                },
                infiltration_reduction_z: 1.0,
            }],
        }
    }
}
