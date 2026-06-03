//! Ground heat loss calculation for ISSO 53.

use crate::error::Result;
use crate::model::{ConstructionElement, DesignConditions, enums::HeatingSystem};
use crate::tables::ground_params::{
    ground_params, GroundSurfaceKind, B_PRIME_MIN, B_PRIME_MAX, U_EQUIV_MIN, Z_DEPTH_MAX,
};
use crate::tables::delta_theta_2;
use crate::tables::thermal_bridge::DELTA_U_TB_DEFAULT;

/// Resolveer de thermische-brug-toeslag ΔU_TB voor een grondelement.
///
/// Voorkeursvolgorde identiek aan `calc/transmission.rs`/`calc/shell.rs`
/// (A6-fix, commit f815c1f): een expliciete `custom_delta_u_tb` wint altijd
/// over de forfaitaire vlag; de forfaitaire default geldt alleen als de vlag
/// aanstaat én er geen custom-waarde is; anders 0.
fn resolve_delta_u_tb(element: &ConstructionElement) -> f64 {
    element.custom_delta_u_tb.unwrap_or_else(|| {
        if element.use_forfaitaire_thermal_bridge {
            DELTA_U_TB_DEFAULT
        } else {
            0.0
        }
    })
}

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

                    // Form. 4.24 vereist (U_k + ΔU_TB) als construction-U, niet de
                    // rauwe element.u_value. Zelfde forfaitair/custom-prioriteit als
                    // de A6-fix in transmission.rs/shell.rs.
                    let u_k = element.u_value + resolve_delta_u_tb(element);

                    calculate_u_equivalent(
                        element.area,
                        perimeter,
                        depth,
                        u_k,
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
/// Geverifieerde vorm (ref-doc `audit-reports/07-isso53-formules-ref.md` §A4,
/// gerenderd uit ISSO 53 PDF p.44 + worked example p.65):
///
/// ```text
/// U_equiv,k = a · b / ( c₁·(B')^n₁ + c₂·(U_k + ΔU_TB)^n₂ + c₃·z^n₃ + d )
/// ```
///
/// Het is een **quotiëntvorm** (niet de eerdere `a·(…)^b`-machtvorm, die door
/// de negatieve `b = −7,455` altijd ≈0 opleverde en stilzwijgend op de
/// U_EQUIV_MIN-clamp landde — een latente bug, want geen enkele fixture raakt
/// dit pad: alle leveren `uEquivalent` expliciet aan). De tabel-`b` is negatief
/// en `a·b` dus eveneens; de norm levert echter een positieve U_equiv, dus
/// nemen we de absolute waarde van de teller. Worked-example check (Vloer,
/// U_k=2,43, B'≈4,1) → U_equiv ≈ 0,177 W/(m²·K), conform p.65.
///
/// Where:
/// - B' = 2 × A_vl / O (geometric factor), clamped [2, 50]
/// - z = depth below ground level [0, 5] m
/// - U_k = construction U-value **inclusief** thermal bridge correction ΔU_TB
///   (de caller telt ΔU_TB al op vóór deze functie — zie A4-fix)
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

    // Clamp depth (0 ≤ z ≤ 5). z = 0 is een normale maaiveld-grondvloer:
    // 0^n₃ = 0 (n₃ > 0), dus de z-term valt netjes weg, geen guard nodig.
    let z_clamped = depth.clamp(0.0, Z_DEPTH_MAX);

    // Get parameters for floor or wall
    let kind = if is_wall {
        GroundSurfaceKind::Wall
    } else {
        GroundSurfaceKind::Floor
    };
    let params = ground_params(kind);

    // Form. 4.24 (quotiëntvorm): teller a·b, noemer c₁·B'^n₁ + c₂·U_k^n₂ + c₃·z^n₃ + d.
    let term1 = params.c1 * b_prime.powf(params.n1); // wanden: c₁=0 → 0 (B' vervalt)
    let term2 = params.c2 * u_construction.powf(params.n2);
    let term3 = params.c3 * z_clamped.powf(params.n3);

    let denom = term1 + term2 + term3 + params.d;
    if denom.abs() < 1e-9 {
        // Degeneraat: noemer ~0 → val terug op de norm-ondergrens.
        return Ok(U_EQUIV_MIN);
    }

    // a·b is negatief (b < 0); de norm levert een positieve U_equiv → |a·b|.
    let u_equiv = (params.a * params.b).abs() / denom;

    // Apply minimum constraint (en negatieve noemer-uitkomsten defensief).
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

    /// A4: de geverifieerde 4.24 quotiëntvorm heeft GEEN z+d-guard meer.
    /// Een ondiepe wand (z=0,5) levert nu een geldige, positieve U_equiv.
    /// (De oude `(z+d)^n₃`-machtvorm crashte hier op een negatieve basis.)
    #[test]
    fn test_ground_shallow_wall_no_longer_errors() {
        let result = calculate_u_equivalent(100.0, 40.0, 0.5, 0.4, true);
        assert!(result.is_ok(), "ondiepe wand mag geen error meer geven: {result:?}");
        assert!(result.unwrap() >= U_EQUIV_MIN);
    }

    /// A4: een normale maaiveld-grondvloer (z=0) is een geldige invoer.
    /// 0^n₃ = 0 (n₃ > 0) → z-term valt weg, geen guard, geen error.
    /// (D4 = de z=0-weigering in de ground.rs-callsite hierboven blijft Ronde 4.)
    #[test]
    fn test_ground_floor_z_zero_ok() {
        let result = calculate_u_equivalent(100.0, 40.0, 0.0, 0.4, false);
        assert!(result.is_ok(), "z=0 grondvloer moet geldig zijn: {result:?}");
        let u = result.unwrap();
        // z=0 vs z=1 verschilt alleen via de kleine c₃·z^n₃-term (~0,026).
        assert!((u - 0.2266).abs() < 0.002, "z=0 floor U_equiv ≈ 0,2266, got {u}");
    }

    /// A4 worked example (ref-doc §A4 / ISSO 53 p.65): Vloer, U_k = 2,43,
    /// geometrie zo dat B' ≈ 4,08 → U_equiv ≈ 0,177 W/(m²·K).
    /// A=200, O=98 → B' = 2·200/98 = 4,0816.
    #[test]
    fn test_u_equivalent_worked_example_p65() {
        let u_equiv = calculate_u_equivalent(200.0, 98.0, 0.0, 2.43, false).unwrap();
        assert!(
            (u_equiv - 0.177).abs() < 0.005,
            "worked example p.65 verwacht U_equiv ≈ 0,177, kreeg {u_equiv}"
        );
    }

    /// A4: smoke — vloer levert een fysisch plausibele U_equiv in [0,1; 1,0].
    #[test]
    fn test_u_equivalent_calculation_smoke() {
        // A=100, O=40 → B'=5 (clamped); z=1; U=0,4; vloer.
        let u_equiv = calculate_u_equivalent(100.0, 40.0, 1.0, 0.4, false).unwrap();
        assert!(u_equiv >= U_EQUIV_MIN, "≥ ondergrens, kreeg {u_equiv}");
        assert!(u_equiv < 1.0, "fysisch plausibel, kreeg {u_equiv}");
        assert!((u_equiv - 0.2264).abs() < 0.002, "verwacht ≈ 0,2264, kreeg {u_equiv}");
    }

    /// A4: ΔU_TB verhoogt U_k → verlaagt U_equiv (hogere noemer). Borgt dat de
    /// thermische-brug-toeslag daadwerkelijk in de 4.24-keten meegaat en de
    /// forfaitair/custom-prioriteit (A6) hetzelfde is als in transmission.rs.
    #[test]
    fn test_ground_delta_u_tb_priority_in_u_equiv() {
        use crate::model::{ConstructionElement, BoundaryType, MaterialType, VerticalPosition, DesignConditions, enums::HeatingSystem};
        use crate::model::construction::GroundParameters;

        let climate = DesignConditions { theta_e: -10.0, theta_me: 9.0, theta_b_adjacent_building: 15.0 };

        let base = ConstructionElement {
            id: "vloer".into(),
            description: "Grondvloer".into(),
            area: 100.0,
            u_value: 0.4,
            boundary_type: BoundaryType::Ground,
            material_type: MaterialType::NonMasonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Floor,
            use_forfaitaire_thermal_bridge: false,
            custom_delta_u_tb: None,
            ground_params: Some(GroundParameters {
                u_equivalent: 0.0, // forceer auto-berekening via form. 4.24
                ground_water_factor: 1.0,
                f_ig: Some(1.0),
                perimeter: Some(40.0),
                depth: Some(1.0),
            }),
            has_embedded_heating: false,
            unheated_space: None,
        };

        // Geen ΔU_TB (vlag uit, geen custom).
        let h_none = calculate_h_t_ground(&[&base], 20.0, &climate, HeatingSystem::default()).unwrap();

        // Custom ΔU_TB = 0,05 → U_k = 0,45.
        let mut e_custom = base.clone();
        e_custom.custom_delta_u_tb = Some(0.05);
        let h_custom = calculate_h_t_ground(&[&e_custom], 20.0, &climate, HeatingSystem::default()).unwrap();

        // Custom wint over forfaitaire vlag (A6-prioriteit): vlag AAN + custom 0,05
        // moet identiek zijn aan custom 0,05 met vlag UIT.
        let mut e_both = base.clone();
        e_both.use_forfaitaire_thermal_bridge = true;
        e_both.custom_delta_u_tb = Some(0.05);
        let h_both = calculate_h_t_ground(&[&e_both], 20.0, &climate, HeatingSystem::default()).unwrap();

        assert!((h_custom - h_both).abs() < 1e-9, "custom moet forfaitaire vlag overrulen: {h_custom} vs {h_both}");
        // Hogere U_k → lagere U_equiv → lagere H_T,ig.
        assert!(h_custom < h_none, "ΔU_TB moet U_equiv verlagen: {h_custom} !< {h_none}");
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