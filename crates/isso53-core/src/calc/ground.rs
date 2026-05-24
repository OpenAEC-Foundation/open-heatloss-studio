//! Ground heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::{ConstructionElement, DesignConditions, enums::HeatingSystem};
use crate::tables::ground_params::{
    ground_params, GroundSurfaceKind, B_PRIME_MIN, B_PRIME_MAX, U_EQUIV_MIN, Z_DEPTH_MAX,
};
use crate::tables::delta_theta_2;

/// Ground water factor correction for U_equiv calculation.
/// ISSO 53 §4.6: 1.0 if groundwater ≥1m below floor, 1.15 otherwise.
pub const GROUND_WATER_FACTOR_NORMAL: f64 = 1.0;
pub const GROUND_WATER_FACTOR_HIGH: f64 = 1.15;

/// Ground correction factor for H_T,ig calculation.
/// ISSO 53 formule 4.21: H_T,ig = 1.45 × Σ(...)
pub const GROUND_CORRECTION_FACTOR: f64 = 1.45;

/// Calculate ground heat loss coefficient H_T,ig.
/// ISSO 53 formule 4.21, PDF p.43: H_T,ig = 1.45 × Σ(A_k × U_equiv,k × f_gw × f_ig,k)
///
/// **§4.6 clause**: voor elementen met `has_embedded_heating = true` (bv.
/// vloerverwarming) wordt `f_ig` overschreven naar 0.0 conform de norm-tekst
/// "f_ig,k = 0 voor het verwarmde deel van wand/vloer/plafond".
///
/// **Auto-f_ig**: als `element.ground_params.f_ig.is_none()`, wordt f_ig berekend
/// via formule 4.22 (Wall) of 4.23 (Floor) op basis van `vertical_position`.
pub fn calculate_h_t_ground(
    elements: &[&ConstructionElement],
    theta_i: f64,
    climate: &DesignConditions,
    heating_system: HeatingSystem,
) -> Result<f64> {
    let mut h_t_ig = 0.0;

    for element in elements {
        if let Some(ref ground_params) = element.ground_params {
            // Calculate U_equivalent if not provided (u_equivalent ≤ 0.0)
            let u_equiv = if ground_params.u_equivalent <= 0.0 {
                if let (Some(perimeter), Some(depth)) = (ground_params.perimeter, ground_params.depth) {
                    // Determine if wall or floor from vertical position
                    let is_wall = matches!(element.vertical_position, crate::model::enums::VerticalPosition::Wall);

                    calculate_u_equivalent(
                        element.area,
                        perimeter,
                        depth,
                        element.u_value,
                        is_wall
                    )?
                } else {
                    // Cannot calculate U_equiv without perimeter and depth
                    return Err(crate::error::Isso53Error::InvalidInput(
                        "Ground element requires perimeter and depth for U_equiv calculation".to_string()
                    ));
                }
            } else {
                ground_params.u_equivalent
            };

            let f_ig = if element.has_embedded_heating {
                0.0  // ISSO 53 §4.6: verwarmd deel van vloer/wand bij vloer-/wand-/CKM-verwarming
            } else if let Some(f_ig_override) = ground_params.f_ig {
                f_ig_override  // User-specified override
            } else {
                // Auto-calculate via formule 4.22 (Wall) or 4.23 (Floor)
                calculate_f_ig_auto(element, theta_i, climate.theta_me, climate.theta_e, heating_system)?
            };

            h_t_ig += GROUND_CORRECTION_FACTOR
                * element.area
                * u_equiv
                * ground_params.ground_water_factor
                * f_ig;
        }
    }

    Ok(h_t_ig)
}

/// Calculate equivalent U-value for ground element using ISSO 53 formule 4.24.
///
/// Based on parameters from tabel 4.3 (PDF p.44) and the formula structure
/// reconstructed from the norm. The exact formula 4.24 from PDF p.44 is:
///
/// U_equiv,k = a × (c1 × (B')^n1 + c2 × (U_k + ΔU_TB)^n2 + c3 × (z + d)^n3)^b
///
/// Where:
/// - B' = 2 × A_vl / O (geometric factor), clamped [2, 50]
/// - z = depth below ground level [0, 5] m
/// - U_k = construction U-value + thermal bridge correction ΔU_TB
/// - Parameters a, b, c1-c3, n1-n3, d from tabel 4.3 for floor/wall
///
/// Result is clamped to minimum U_equiv ≥ 0.1 W/(m²·K).
///
/// # Arguments
/// * `area` - Floor area A_vl in m²
/// * `perimeter` - Perimeter O in m
/// * `depth` - Depth below ground level z in m [0, 5]
/// * `u_construction` - Base U-value including thermal bridges in W/(m²·K)
/// * `is_wall` - true for wall, false for floor (determines parameter set)
///
/// # Returns
/// U_equivalent value in W/(m²·K), minimum 0.1
pub fn calculate_u_equivalent(
    area: f64,
    perimeter: f64,
    depth: f64,
    u_construction: f64,
    is_wall: bool,
) -> Result<f64> {
    // Input validation
    if area <= 0.0 {
        return Err(crate::error::Isso53Error::InvalidInput(
            "Area must be positive".to_string(),
        ));
    }
    if perimeter <= 0.0 {
        return Err(crate::error::Isso53Error::InvalidInput(
            "Perimeter must be positive".to_string(),
        ));
    }
    if u_construction < 0.0 {
        return Err(crate::error::Isso53Error::InvalidInput(
            "U-value cannot be negative".to_string(),
        ));
    }

    // Calculate B' geometric factor: B' = 2 × A_vl / O
    let b_prime = (2.0 * area / perimeter).clamp(B_PRIME_MIN, B_PRIME_MAX);

    // Clamp depth
    let z_clamped = depth.clamp(0.0, Z_DEPTH_MAX);

    // Get parameters for floor or wall
    let kind = if is_wall {
        GroundSurfaceKind::Wall
    } else {
        GroundSurfaceKind::Floor
    };
    let params = ground_params(kind);

    // Guard against negative base for power function (z + d must be > 0)
    let depth_sum = z_clamped + params.d;
    if depth_sum <= 0.0 {
        return Err(crate::error::Isso53Error::InvalidInput(
            format!("invalid depth z={} for {:?}: z + d = {} must be > 0",
                depth, kind, depth_sum)
        ));
    }

    // Apply formule 4.24: U_equiv,k = a × (c1 × (B')^n1 + c2 × (U_k)^n2 + c3 × (z + d)^n3)^b
    let term1 = params.c1 * b_prime.powf(params.n1);
    let term2 = params.c2 * u_construction.powf(params.n2);
    let term3 = params.c3 * depth_sum.powf(params.n3);

    let inner_sum = term1 + term2 + term3;
    let u_equiv = params.a * inner_sum.powf(params.b);

    // Apply minimum constraint
    Ok(u_equiv.max(U_EQUIV_MIN))
}

/// Auto-calculate f_ig using ISSO 53 formule 4.22 (Wall) or 4.23 (Floor).
///
/// Formule 4.22 (wanden): f_ig,k = (θ_i − θ_me) / (θ_i − θ_e)
/// Formule 4.23 (vloeren): f_ig,k = ((θ_i + Δθ_2) − θ_me) / (θ_i − θ_e)
///
/// waarbij Δθ_2 uit tabel 2.3 per verwarmingssysteem.
fn calculate_f_ig_auto(
    element: &ConstructionElement,
    theta_i: f64,
    theta_me: f64,
    theta_e: f64,
    heating_system: HeatingSystem,
) -> Result<f64> {

    if (theta_i - theta_e).abs() < 0.001 {
        return Ok(0.0);  // Avoid division by zero
    }

    let f_ig = match element.vertical_position {
        crate::model::enums::VerticalPosition::Wall => {
            // Formule 4.22: f_ig,k = (θ_i − θ_me) / (θ_i − θ_e)
            (theta_i - theta_me) / (theta_i - theta_e)
        }
        crate::model::enums::VerticalPosition::Floor => {
            // Formule 4.23: f_ig,k = ((θ_i + Δθ_2) − θ_me) / (θ_i − θ_e)
            let delta_theta_2 = delta_theta_2(heating_system);
            let theta_i_effective = theta_i + delta_theta_2;
            (theta_i_effective - theta_me) / (theta_i - theta_e)
        }
        crate::model::enums::VerticalPosition::Ceiling => {
            // Behandel ceiling als wall (conservatief)
            (theta_i - theta_me) / (theta_i - theta_e)
        }
    };

    Ok(f_ig)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_depth_guard_wall() {
        // Test wall with z=0.5 — should fail because z + d_wall = 0.5 + (-1.074) = -0.574 < 0
        let result = calculate_u_equivalent(100.0, 40.0, 0.5, 0.4, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_ground_depth_guard_floor() {
        // Test floor with z=0 — should fail because z + d_floor = 0 + (-0.0203) = -0.0203 < 0
        let result = calculate_u_equivalent(100.0, 40.0, 0.0, 0.4, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_u_equivalent_calculation_smoke() {
        // Hand-calculated: A=100, perimeter=40, B'=5 (clamped), z=1, U=0.4, floor
        // Floor params: a=0.9671, b=-7.455, c1=10.76, c2=9.773, c3=0.0265, n1=0.5532, n2=0.6027, n3=-0.9296, d=-0.0203
        // term1 = 10.76 * 5^0.5532 ≈ 10.76 * 2.43 ≈ 26.1
        // term2 = 9.773 * 0.4^0.6027 ≈ 9.773 * 0.53 ≈ 5.2
        // term3 = 0.0265 * (1-0.0203)^(-0.9296) ≈ 0.0265 * 0.98^(-0.9296) ≈ 0.027
        // inner = 26.1 + 5.2 + 0.027 ≈ 31.3
        // u_equiv = 0.9671 * 31.3^(-7.455) ≈ very small, but clamped to 0.1
        let result = calculate_u_equivalent(100.0, 40.0, 1.0, 0.4, false);
        assert!(result.is_ok());
        let u_equiv = result.unwrap();
        assert!(u_equiv >= 0.1, "Should be at least minimum value");
        assert!(u_equiv < 1.0, "Should be reasonable");
    }

    #[test]
    fn embedded_heating_zeroes_f_ig() {
        use crate::model::{ConstructionElement, BoundaryType, MaterialType, VerticalPosition, DesignConditions, enums::HeatingSystem};
        use crate::model::construction::GroundParameters;

        let climate = DesignConditions {
            theta_e: -10.0,
            theta_me: 9.0,
            theta_b_adjacent_building: 15.0,
        };

        let element = ConstructionElement {
            id: "vloer-vv".into(),
            description: "Vloer met vloerverwarming".into(),
            area: 100.0,
            u_value: 0.16,
            boundary_type: BoundaryType::Ground,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Floor,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: Some(GroundParameters {
                u_equivalent: 0.16,
                ground_water_factor: 1.0,
                f_ig: Some(1.0),  // Explicit override
                perimeter: None,
                depth: None,
            }),
            has_embedded_heating: true,  // ← key: vloerverwarming
            unheated_space: None,
        };

        let result = calculate_h_t_ground(&[&element], 20.0, &climate, HeatingSystem::Vloerverwarming).expect("calc");
        assert_eq!(result, 0.0, "Embedded heating moet H_T,ig naar 0 brengen (§4.6)");

        // Sanity: zonder embedded heating wel verlies
        let mut e2 = element.clone();
        e2.has_embedded_heating = false;
        let result2 = calculate_h_t_ground(&[&e2], 20.0, &climate, HeatingSystem::Vloerverwarming).expect("calc");
        assert!(result2 > 20.0, "Zonder embedded heating wél H_T,ig > 0, got {result2}");
    }

    #[test]
    fn test_auto_f_ig_formules() {
        use crate::model::{ConstructionElement, BoundaryType, MaterialType, VerticalPosition, DesignConditions, enums::HeatingSystem};
        use crate::model::construction::GroundParameters;

        let climate = DesignConditions {
            theta_e: -10.0,
            theta_me: 9.0,
            theta_b_adjacent_building: 15.0,
        };

        // Test formule 4.22 (Wall): f_ig = (20 - 9) / (20 - (-10)) = 11 / 30 = 0.367
        let wall_element = ConstructionElement {
            id: "wall-ground".into(),
            description: "Kelderband".into(),
            area: 10.0,
            u_value: 0.5,
            boundary_type: BoundaryType::Ground,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Wall,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: Some(GroundParameters {
                u_equivalent: 0.5,
                ground_water_factor: 1.0,
                f_ig: None,  // Auto-calculate
                perimeter: None,
                depth: None,
            }),
            has_embedded_heating: false,
            unheated_space: None,
        };

        let f_ig_wall = calculate_f_ig_auto(&wall_element, 20.0, 9.0, -10.0, HeatingSystem::default()).unwrap();
        assert!((f_ig_wall - 0.367).abs() < 0.01, "Wall f_ig expected ~0.367, got {}", f_ig_wall);

        // Test formule 4.23 (Floor) with Radiator: f_ig = ((20 + (-1)) - 9) / (20 - (-10)) = 10 / 30 = 0.333
        let mut floor_element = wall_element.clone();
        floor_element.vertical_position = VerticalPosition::Floor;

        let f_ig_floor = calculate_f_ig_auto(&floor_element, 20.0, 9.0, -10.0, HeatingSystem::RadiatorenConvHtEnLuchtverwarming).unwrap();
        assert!((f_ig_floor - 0.333).abs() < 0.01, "Floor+Radiator f_ig expected ~0.333, got {}", f_ig_floor);

        // Test formule 4.23 (Floor) with Vloerverwarming: f_ig = ((20 + 0) - 9) / (20 - (-10)) = 11 / 30 = 0.367
        let f_ig_vv = calculate_f_ig_auto(&floor_element, 20.0, 9.0, -10.0, HeatingSystem::Vloerverwarming).unwrap();
        assert!((f_ig_vv - 0.367).abs() < 0.01, "Floor+VV f_ig expected ~0.367, got {}", f_ig_vv);
    }
}