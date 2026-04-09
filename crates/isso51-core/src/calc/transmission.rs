//! Transmission heat loss calculations.
//! ISSO 51 §2.5.1 through §2.5.5.
//!
//! Calculates the specific heat loss coefficient H_T for each boundary type:
//! - H_T,ie: to exterior (outside air)
//! - H_T,ia: to adjacent rooms within the dwelling
//! - H_T,io: to unheated spaces
//! - H_T,ib: to neighboring dwellings/buildings
//! - H_T,ig: to the ground

use crate::model::construction::ConstructionElement;
use crate::model::enums::{BoundaryType, VerticalPosition};
use crate::tables::thermal_bridge;

/// Calculate the specific heat loss H_T,ie to exterior for a single element.
/// [`ISSO_51_2023_FORMULE4_3A`](crate::formulas::ISSO_51_2023_FORMULE4_3A):
/// H_T,ie = Σ(A_k × f_k × (U_k + ΔU_TB))
///
/// # Arguments
/// * `element` - The construction element facing the exterior
///
/// # Returns
/// Contribution to H_T,ie in W/K for this element.
pub fn h_t_exterior_element(element: &ConstructionElement) -> f64 {
    let f_k = element.temperature_factor.unwrap_or(1.0);
    let delta_u = thermal_bridge::delta_u_tb(
        element.use_forfaitaire_thermal_bridge,
        element.custom_delta_u_tb,
    );
    element.area * f_k * (element.u_value + delta_u)
}

/// Calculate the specific heat loss H_T,ia to an adjacent room.
/// [`ISSO_51_2023_FORMULE4_6`](crate::formulas::ISSO_51_2023_FORMULE4_6):
/// H_T,ia = Σ(A_k × U_k × f_ia,k)
///
/// The temperature factor f_ia,k depends on element position.
/// For horizontal constructions between heated rooms, temperature stratification
/// applies on BOTH sides of the tussenvloer:
/// - Wall: f_ia = (θ_i - θ_a) / (θ_i - θ_e)
/// - Ceiling: f_ia = ((θ_i + Δθ₁) - (θ_a + Δθ₂)) / (θ_i - θ_e)
/// - Floor: f_ia = ((θ_i + Δθ₂) - (θ_a + Δθ₁)) / (θ_i - θ_e)
///
/// The ceiling surface (this room) is at θ_i + Δθ₁ (warm air rises),
/// while the floor surface (adjacent room above) is at θ_a + Δθ₂ (floor cooler).
/// Vice versa for floor elements.
///
/// Note: assumes the adjacent heated room has the same heating system (same Δθ values).
///
/// # Arguments
/// * `element` - The construction element facing the adjacent room
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_a` - Design temperature of the adjacent room in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `delta_1` - Δθ₁ from Table 2.12 (ceiling correction)
/// * `delta_2` - Δθ₂ from Table 2.12 (floor correction)
///
/// # Returns
/// Contribution to H_T,ia in W/K for this element.
pub fn h_t_adjacent_room_element(
    element: &ConstructionElement,
    theta_i: f64,
    theta_a: f64,
    theta_e: f64,
    delta_1: f64,
    delta_2: f64,
) -> f64 {
    let f_ia = if let Some(f) = element.temperature_factor {
        f
    } else {
        match element.vertical_position {
            VerticalPosition::Wall => (theta_i - theta_a) / (theta_i - theta_e),
            VerticalPosition::Ceiling => {
                ((theta_i + delta_1) - (theta_a + delta_2)) / (theta_i - theta_e)
            }
            VerticalPosition::Floor => {
                ((theta_i + delta_2) - (theta_a + delta_1)) / (theta_i - theta_e)
            }
        }
    };
    element.area * element.u_value * f_ia
}

/// Calculate the specific heat loss H_T,io to unheated spaces.
/// [`ISSO_51_2023_FORMULE4_10`](crate::formulas::ISSO_51_2023_FORMULE4_10):
/// H_T,io = Σ(A_k × U_k × f_k)
///
/// # Arguments
/// * `element` - The construction element facing the unheated space
///
/// # Returns
/// Contribution to H_T,io in W/K for this element.
pub fn h_t_unheated_element(element: &ConstructionElement) -> f64 {
    let f_k = element.temperature_factor.unwrap_or(0.5);
    element.area * element.u_value * f_k
}

/// Calculate the specific heat loss H_T,ib to neighboring buildings.
/// [`ISSO_51_2023_FORMULE4_14`](crate::formulas::ISSO_51_2023_FORMULE4_14):
/// H_T,ib = c_z × Σ(A_k × U_k × f_b)
///
/// The temperature factor f_b depends on the element position:
/// - Wall: [`ISSO_51_2023_FORMULE4_15`](crate::formulas::ISSO_51_2023_FORMULE4_15):
///   f_b = (θ_i - θ_b) / (θ_i - θ_e)
/// - Ceiling: [`ISSO_51_2023_FORMULE4_17`](crate::formulas::ISSO_51_2023_FORMULE4_17):
///   f_b = (θ_i + Δθ_1 - θ_b) / (θ_i - θ_e)
/// - Floor: [`ISSO_51_2023_FORMULE4_16`](crate::formulas::ISSO_51_2023_FORMULE4_16):
///   f_b = (θ_i + Δθ_2 - θ_b) / (θ_i - θ_e)
///
/// Note: c_z is applied at the room level, not per element.
///
/// # Arguments
/// * `element` - The construction element facing the neighboring building
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_b` - Temperature of neighboring building in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `delta_1` - Δθ₁ from Table 2.12 (ceiling correction)
/// * `delta_2` - Δθ₂ from Table 2.12 (floor correction)
///
/// # Returns
/// Contribution to H_T,ib in W/K for this element (before c_z multiplication).
pub fn h_t_adjacent_building_element(
    element: &ConstructionElement,
    theta_i: f64,
    theta_b: f64,
    theta_e: f64,
    delta_1: f64,
    delta_2: f64,
) -> f64 {
    let f_b = if let Some(f) = element.temperature_factor {
        f
    } else {
        match element.vertical_position {
            VerticalPosition::Wall => (theta_i - theta_b) / (theta_i - theta_e),
            VerticalPosition::Ceiling => (theta_i + delta_1 - theta_b) / (theta_i - theta_e),
            VerticalPosition::Floor => (theta_i + delta_2 - theta_b) / (theta_i - theta_e),
        }
    };
    element.area * element.u_value * f_b
}

/// Calculate the specific heat loss H_T,ig to the ground.
/// [`ISSO_51_2023_FORMULE4_18`](crate::formulas::ISSO_51_2023_FORMULE4_18):
/// H_T,ig = 1.45 × G_w × Σ(A_k × f_g2 × U_e,k)
///
/// # Arguments
/// * `element` - The construction element in contact with the ground
///
/// # Returns
/// Contribution to H_T,ig in W/K for this element.
pub fn h_t_ground_element(element: &ConstructionElement) -> f64 {
    if let Some(ref gp) = element.ground_params {
        1.45 * gp.ground_water_factor * element.area * gp.fg2 * gp.u_equivalent
    } else {
        0.0
    }
}

/// Calculate all specific heat loss coefficients for a set of construction elements.
///
/// # Arguments
/// * `elements` - All construction elements of a room
/// * `theta_i` - Design indoor temperature of this room in °C
/// * `theta_e` - Design outdoor temperature in °C
/// * `theta_b` - Temperature of neighboring buildings in °C
/// * `c_z` - Security factor for neighbor heat loss
/// * `delta_1` - Δθ₁ from Table 2.12
/// * `delta_2` - Δθ₂ from Table 2.12
///
/// # Returns
/// Tuple of (H_T,ie, H_T,ia, H_T,io, H_T,ib, H_T,ig) in W/K.
pub fn calculate_all_h_t(
    elements: &[ConstructionElement],
    theta_i: f64,
    theta_e: f64,
    theta_b: f64,
    c_z: f64,
    delta_1: f64,
    delta_2: f64,
) -> (f64, f64, f64, f64, f64) {
    let mut h_t_ie = 0.0;
    let mut h_t_ia = 0.0;
    let mut h_t_io = 0.0;
    let mut h_t_ib_sum = 0.0;
    let mut h_t_ig = 0.0;

    for element in elements {
        match element.boundary_type {
            BoundaryType::Exterior => {
                h_t_ie += h_t_exterior_element(element);
            }
            BoundaryType::AdjacentRoom => {
                let theta_a = element.adjacent_temperature.unwrap_or(theta_i);
                h_t_ia += h_t_adjacent_room_element(
                    element, theta_i, theta_a, theta_e, delta_1, delta_2,
                );
            }
            BoundaryType::UnheatedSpace => {
                h_t_io += h_t_unheated_element(element);
            }
            BoundaryType::AdjacentBuilding => {
                h_t_ib_sum += h_t_adjacent_building_element(
                    element, theta_i, theta_b, theta_e, delta_1, delta_2,
                );
            }
            BoundaryType::Ground => {
                h_t_ig += h_t_ground_element(element);
            }
        }
    }

    let h_t_ib = c_z * h_t_ib_sum;

    (h_t_ie, h_t_ia, h_t_io, h_t_ib, h_t_ig)
}

/// Calculate total transmission heat loss Φ_T for a room.
/// [`ISSO_51_2023_FORMULE4_2`](crate::formulas::ISSO_51_2023_FORMULE4_2):
/// Φ_T = (H_T,ie + H_T,ia + H_T,io + H_T,ib + H_T,ig) × (θ_i - θ_e)
pub fn phi_transmission(h_t_total: f64, theta_i: f64, theta_e: f64) -> f64 {
    h_t_total * (theta_i - theta_e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::construction::ConstructionElement;
    use crate::model::enums::{BoundaryType, MaterialType, VerticalPosition};

    fn make_exterior_element(area: f64, u_value: f64, material: MaterialType) -> ConstructionElement {
        ConstructionElement {
            id: "test".to_string(),
            description: "test".to_string(),
            area,
            u_value,
            boundary_type: BoundaryType::Exterior,
            material_type: material,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            catalog_ref: None,
        }
    }

    #[test]
    fn test_isso51_example_room1_h_t_ie() {
        // ISSO 51 Example 1, Room 1 (woonkamer):
        // buitenwand: A=7.29, U=0.36, f=1 → 7.29 × 1 × (0.36+0.1) = 3.35
        // raam: A=4.32, U=3.2, f=1 → 4.32 × 1 × (3.2+0.1) = 14.26
        // buitenwand bij deur: A=0.36, U=0.36, f=1 → 0.36 × 1 × (0.36+0.1) = 0.17
        // deur naar balkon: A=2.16, U=2.78, f=1 → 2.16 × 1 × (2.78+0.1) = 6.22
        // Total H_T,ie = 24.00

        let elements = vec![
            make_exterior_element(7.29, 0.36, MaterialType::Masonry),
            make_exterior_element(4.32, 3.2, MaterialType::NonMasonry),
            make_exterior_element(0.36, 0.36, MaterialType::Masonry),
            make_exterior_element(2.16, 2.78, MaterialType::NonMasonry),
        ];

        let h_t_ie: f64 = elements.iter().map(|e| h_t_exterior_element(e)).sum();

        // The example gives 24.00 (rounded)
        assert!((h_t_ie - 24.00).abs() < 0.1, "H_T,ie = {h_t_ie}, expected ~24.00");
    }

    #[test]
    fn test_isso51_example_room1_h_t_ia() {
        // ISSO 51 Example 1, Room 1 (woonkamer, θ_i=20, θ_e=-10):
        // naar keuken (θ_a=20): A=7.36, U=2.17, f=0 → 0
        // naar slaapkamer1 (θ_a=20): A=11.20, U=2.17, f=0 → 0
        // naar entree (θ_a=15): A=2.51, U=2.17, f=(20-15)/(20--10)=0.1667 → 0.91
        // naar toilet (θ_a=15): A=3.12, U=2.17, f=0.1667 → 1.13
        // naar badkamer (θ_a=22): A=3.64, U=2.17, f=(20-22)/(20--10)=-0.0667 → -0.53
        // Total H_T,ia = 1.51

        let theta_i = 20.0;
        let theta_e = -10.0;

        let tests = vec![
            (7.36, 2.17, 20.0, 0.0),     // keuken
            (11.20, 2.17, 20.0, 0.0),     // slaapkamer1
            (2.51, 2.17, 15.0, 0.91),     // entree
            (3.12, 2.17, 15.0, 1.13),     // toilet
            (3.64, 2.17, 22.0, -0.53),    // badkamer
        ];

        let mut total = 0.0;
        for (area, u, theta_a, expected) in &tests {
            let element = ConstructionElement {
                id: "test".to_string(),
                description: "test".to_string(),
                area: *area,
                u_value: *u,
                boundary_type: BoundaryType::AdjacentRoom,
                material_type: MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: Some(*theta_a),
                vertical_position: VerticalPosition::Wall,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                catalog_ref: None,
            };
            let h = h_t_adjacent_room_element(&element, theta_i, *theta_a, theta_e, 2.0, -1.0);
            assert!(
                (h - expected).abs() < 0.02,
                "Element with A={area}, θ_a={theta_a}: got {h}, expected {expected}"
            );
            total += h;
        }

        assert!((total - 1.51).abs() < 0.1, "H_T,ia = {total}, expected ~1.51");
    }

    #[test]
    fn test_isso51_example_room1_h_t_ib() {
        // ISSO 51 Example 1, Room 1 (woonkamer):
        // θ_i=20, θ_b=15 (assumed from f_b values), θ_e=-10
        // Δθ_1=2.0 (LT-radiator), Δθ_2=-1.0
        //
        // woningscheidende wand: A=18.09, U=2.08, f_b=(20-15)/30=0.1667 → 6.27
        // plafond: A=28.20, U=2.5, f_b=(20+2-15)/30=0.2333 → 16.45
        // vloer: A=28.20, U=2.5, f_b=(20-1-15)/30=0.1333 → 9.40
        // Sum = 32.12, c_z=0.5 → H_T,ib = 16.06

        let theta_i = 20.0;
        let theta_b = 15.0;
        let theta_e = -10.0;
        let delta_1 = 2.0; // LT radiator
        let delta_2 = -1.0;

        let elements = vec![
            (18.09, 2.08, VerticalPosition::Wall),
            (28.20, 2.5, VerticalPosition::Ceiling),
            (28.20, 2.5, VerticalPosition::Floor),
        ];

        let mut sum = 0.0;
        for (area, u, pos) in &elements {
            let element = ConstructionElement {
                id: "test".to_string(),
                description: "test".to_string(),
                area: *area,
                u_value: *u,
                boundary_type: BoundaryType::AdjacentBuilding,
                material_type: MaterialType::Masonry,
                temperature_factor: None,
                adjacent_room_id: None,
                adjacent_temperature: None,
                vertical_position: *pos,
                use_forfaitaire_thermal_bridge: false,
                custom_delta_u_tb: None,
                ground_params: None,
                has_embedded_heating: false,
                catalog_ref: None,
            };
            sum += h_t_adjacent_building_element(&element, theta_i, theta_b, theta_e, delta_1, delta_2);
        }

        let h_t_ib = 0.5 * sum;
        assert!(
            (h_t_ib - 16.06).abs() < 0.1,
            "H_T,ib = {h_t_ib}, expected ~16.06"
        );
    }

    #[test]
    fn test_isso51_example_room1_total_transmission() {
        // Room 1 total: Φ_T = (24.00 + 1.51 + 0 + 16.06 + 0) × (20 - -10) = 1247 W
        let h_t_total = 24.00 + 1.51 + 0.0 + 16.06 + 0.0;
        let phi_t = phi_transmission(h_t_total, 20.0, -10.0);
        assert!(
            (phi_t - 1247.0).abs() < 5.0,
            "Φ_T = {phi_t}, expected ~1247"
        );
    }
}
