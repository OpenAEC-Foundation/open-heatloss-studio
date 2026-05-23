//! Ground heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::ConstructionElement;
use crate::tables::ground_params::{
    ground_params, GroundSurfaceKind, B_PRIME_MIN, B_PRIME_MAX, U_EQUIV_MIN, Z_DEPTH_MAX,
};

/// Ground water factor correction for U_equiv calculation.
/// ISSO 53 §4.6: 1.0 if groundwater ≥1m below floor, 1.15 otherwise.
pub const GROUND_WATER_FACTOR_NORMAL: f64 = 1.0;
pub const GROUND_WATER_FACTOR_HIGH: f64 = 1.15;

/// Ground correction factor for H_T,ig calculation.
/// ISSO 53 formule 4.21: H_T,ig = 1.45 × Σ(...)
pub const GROUND_CORRECTION_FACTOR: f64 = 1.45;

/// Calculate ground heat loss coefficient H_T,ig.
/// ISSO 53 formule 4.21, PDF p.43: H_T,ig = 1.45 × Σ(A_k × U_equiv,k × f_gw × f_ig,k)
pub fn calculate_h_t_ground(elements: &[&ConstructionElement]) -> Result<f64> {
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

            h_t_ig += GROUND_CORRECTION_FACTOR
                * element.area
                * u_equiv
                * ground_params.ground_water_factor
                * ground_params.f_ig;
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
}