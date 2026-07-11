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
    element.custom_delta_u_tb.unwrap_or(if element.use_forfaitaire_thermal_bridge {
        DELTA_U_TB_DEFAULT
    } else {
        0.0
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
/// Norm-vorm (ISSO 53 PDF p.44, visueel geverifieerd — PM-verificatie
/// 2026-06-10; de formule staat als gerenderde afbeelding in de PDF):
///
/// ```text
/// U_equiv,k = a / ( b + (c₁ + B')^n₁ + (c₂ + z)^n₂ + (c₃ + U_k + ΔU_TB)^n₃ ) + d
/// ```
///
/// De c-parameters zijn **addenden binnen de machten**, `b` is een somterm in
/// de noemer en `d` staat buiten de breuk. Dit vervangt de eerdere
/// quotiëntvorm `|a·b| / (c₁·B'^n₁ + c₂·U_k^n₂ + c₃·z^n₃ + d)`, die
/// structureel fout was: omgekeerde monotonie in U_k (hogere U_k gaf lágere
/// U_equiv) en een z=0-singulariteit bij wanden. In de norm-vorm bestaan die
/// artefacten niet: U_equiv stijgt met U_k, daalt met z en (bij vloeren) met
/// B', en z = 0 is voor zowel vloer als wand een regulier punt.
///
/// IJkpunten uit de norm-voorbeelden (beide reproduceren, zie tests):
/// - **Schilvoorbeeld PDF p.59/60**: vloer Rc = 3,5 → U_k = 1/3,71 ≈ 0,2695,
///   + ΔU_TB 0,1 (tabel 3.1 "overige situaties", in het voorbeeld overal
///     `(Uk + 0,1)`); B' = 2·(50·20)/140 = 14,29; z = 0 → U_equiv ≈ 0,181
///     (norm rekent met 0,18 op p.59 resp. 0,17 op p.60).
/// - **Detailvoorbeeld PDF p.65**: B' = 12,07, beganegrondvloer U = 0,26
///   (tabel 6.2-lagen) + ΔU_TB 0,05 (tabel 3.1 "nieuw gebouw, goed
///   vakmanschap") = 0,31; z = 0 → U_equiv = 0,1774 (norm: 0,177).
///
/// ⚠️ De eerdere ijking "U_k = 2,43 → 0,177" was een misread: 2,43 op p.65 is
/// de **plafond**-U uit de H_T,ia-tabel ("Vertrek boven"), niet de grondvloer.
///
/// Where:
/// - B' = 2 × A_vl / O (geometric factor), clamped [2, 50] (§4.6, PDF p.43)
/// - z = depth below ground level, clamped [0, 5] m (formule 4.24, PDF p.44)
/// - U_k = construction U-value **inclusief** thermal bridge correction ΔU_TB
///   (de caller telt ΔU_TB al op vóór deze functie — zie A4-fix)
/// - Parameters a, b, c1-c3, n1-n3, d from tabel 4.3 (PDF p.44) for floor/wall
///
/// Result is clamped to minimum U_equiv ≥ 0.1 W/(m²·K) (§4.6).
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

    // Calculate B' geometric factor: B' = 2 × A_vl / O, clamped [2, 50].
    // De clamp-ondergrens 2 borgt meteen de norm-voetnoot dat B' bij wanden
    // niet 0 mag zijn (de wand-term is (0 + B')^0 = 1, gedefinieerd voor B'>0).
    let b_prime = (2.0 * area / perimeter).clamp(B_PRIME_MIN, B_PRIME_MAX);

    // Clamp depth: 0 ≤ z ≤ 5 m; z > 5 → z = 5 (formule 4.24, PDF p.44).
    // z = 0 is in de norm-vorm een regulier punt voor vloer én wand:
    // (c₂ + 0)^n₂ = c₂^n₂ — geen singulariteit. De oude z=0-wand-Err-guard
    // was een artefact van de afgeschreven quotiëntvorm (0^n₃ met n₃<0 → ∞)
    // en is daarom verwijderd.
    let z_clamped = depth.clamp(0.0, Z_DEPTH_MAX);

    // Get parameters for floor or wall
    let kind = if is_wall {
        GroundSurfaceKind::Wall
    } else {
        GroundSurfaceKind::Floor
    };
    let params = ground_params(kind);

    // Formule 4.24 (norm-vorm): noemer b + (c₁+B')^n₁ + (c₂+z)^n₂ + (c₃+U_k)^n₃.
    let term1 = (params.c1 + b_prime).powf(params.n1); // wanden: (0+B')^0 = 1
    let term2 = (params.c2 + z_clamped).powf(params.n2);
    let term3 = (params.c3 + u_construction).powf(params.n3);

    let denom = params.b + term1 + term2 + term3;
    if denom <= 1e-9 {
        // Buiten het geldigheidsdomein van formule 4.24 (komt pas voor bij
        // fysisch onzinnige invoer, bv. wand-U_k ≳ 27 W/(m²·K)). Expliciete
        // fout i.p.v. een stilzwijgend onfysisch (negatief/oneindig) resultaat.
        return Err(crate::error::Isso53Error::InvalidInput(format!(
            "formule 4.24 buiten geldigheidsdomein: noemer = {denom:.4} ≤ 0 \
             (B' = {b_prime:.2}, z = {z_clamped:.2}, U_k = {u_construction:.2})"
        )));
    }

    let u_equiv = params.a / denom + params.d;

    // Norm-ondergrens §4.6: U_equiv,k ≥ 0,1 W/(m²·K).
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

    /// Norm-vorm 4.24: een ondiepe wand (z=0,5) levert een geldige,
    /// positieve U_equiv. (B' = 5, U_k = 0,4 → ≈ 0,471.)
    #[test]
    fn test_ground_shallow_wall_no_longer_errors() {
        let result = calculate_u_equivalent(100.0, 40.0, 0.5, 0.4, true);
        assert!(result.is_ok(), "ondiepe wand mag geen error geven: {result:?}");
        let u = result.unwrap();
        assert!(u >= U_EQUIV_MIN);
        assert!((u - 0.4714).abs() < 0.002, "wand z=0,5 U_equiv ≈ 0,4714, got {u}");
    }

    /// Norm-vorm 4.24: een normale maaiveld-grondvloer (z=0) is een geldige
    /// invoer. De z-term wordt (c₂ + 0)^n₂ = c₂^n₂ — regulier punt.
    /// B' = 5, z = 0, U_k = 0,4 → U_equiv ≈ 0,2727.
    #[test]
    fn test_ground_floor_z_zero_ok() {
        let result = calculate_u_equivalent(100.0, 40.0, 0.0, 0.4, false);
        assert!(result.is_ok(), "z=0 grondvloer moet geldig zijn: {result:?}");
        let u = result.unwrap();
        assert!((u - 0.2727).abs() < 0.002, "z=0 floor U_equiv ≈ 0,2727, got {u}");
    }

    /// Norm-vorm 4.24: z = 0 is óók voor een wand een regulier punt —
    /// (c₂ + 0)^n₂ = 26,586^0,5012, geen singulariteit. De oude Err-guard was
    /// een artefact van de afgeschreven quotiëntvorm (0^n₃ met n₃ < 0 → ∞).
    /// B' = 5 (geen invloed), z = 0, U_k = 0,4 → U_equiv ≈ 0,6316.
    #[test]
    fn test_ground_wall_z_zero_now_valid() {
        let result = calculate_u_equivalent(100.0, 40.0, 0.0, 0.4, true);
        assert!(result.is_ok(), "z=0 wand is in de norm-vorm geldig: {result:?}");
        let u = result.unwrap();
        assert!((u - 0.6316).abs() < 0.002, "z=0 wand U_equiv ≈ 0,6316, got {u}");
    }

    /// Wand met z > 0 geeft een geldige, positieve U_equiv.
    #[test]
    fn test_ground_wall_positive_z_still_ok() {
        let result = calculate_u_equivalent(100.0, 40.0, 1.5, 0.4, true);
        assert!(result.is_ok(), "wand met z>0 moet geldig blijven: {result:?}");
        assert!(result.unwrap() >= U_EQUIV_MIN);
    }

    /// Monotonie (norm-vorm 4.24): U_equiv **stijgt** met U_k — een slechter
    /// geïsoleerde vloer verliest méér naar de grond. (De afgeschreven
    /// quotiëntvorm had hier de omgekeerde, onfysische richting.)
    #[test]
    fn test_u_equivalent_monotonic_increasing_in_u_k() {
        let mut prev = 0.0;
        for u_k in [0.2, 0.3, 0.4, 0.6, 1.0, 2.0] {
            let u_eq = calculate_u_equivalent(100.0, 40.0, 0.0, u_k, false).unwrap();
            assert!(
                u_eq > prev,
                "U_equiv moet stijgen met U_k: U_k={u_k} gaf {u_eq} ≤ {prev}"
            );
            prev = u_eq;
        }
        // Zelfde richting voor wanden.
        let w_low = calculate_u_equivalent(100.0, 40.0, 1.0, 0.3, true).unwrap();
        let w_high = calculate_u_equivalent(100.0, 40.0, 1.0, 0.6, true).unwrap();
        assert!(w_high > w_low, "wand: U_equiv stijgt met U_k ({w_high} !> {w_low})");
    }

    /// Monotonie (norm-vorm 4.24): U_equiv **daalt** met z — dieper in de
    /// grond betekent meer gronddemping.
    #[test]
    fn test_u_equivalent_monotonic_decreasing_in_z() {
        let mut prev = f64::INFINITY;
        for z in [0.0, 0.5, 1.0, 2.5, 5.0] {
            let u_eq = calculate_u_equivalent(100.0, 40.0, z, 0.4, false).unwrap();
            assert!(u_eq < prev, "U_equiv moet dalen met z: z={z} gaf {u_eq} ≥ {prev}");
            prev = u_eq;
        }
        // z > 5 wordt geclampt op 5 (norm: indien z > 5 m dan z = 5 m).
        let at_5 = calculate_u_equivalent(100.0, 40.0, 5.0, 0.4, false).unwrap();
        let above_5 = calculate_u_equivalent(100.0, 40.0, 8.0, 0.4, false).unwrap();
        assert!((at_5 - above_5).abs() < 1e-12, "z>5 moet clampen op z=5");
        // Wanden: zelfde dalende richting.
        let w_shallow = calculate_u_equivalent(100.0, 40.0, 0.5, 0.4, true).unwrap();
        let w_deep = calculate_u_equivalent(100.0, 40.0, 3.0, 0.4, true).unwrap();
        assert!(w_deep < w_shallow, "wand: U_equiv daalt met z ({w_deep} !< {w_shallow})");
    }

    /// Monotonie (norm-vorm 4.24): U_equiv **daalt** met B' bij vloeren
    /// (compacter gebouw → relatief minder randverlies); bij wanden heeft B'
    /// per norm-voetnoot geen invloed ((0+B')^0 = 1).
    #[test]
    fn test_u_equivalent_monotonic_decreasing_in_b_prime() {
        // B' = 2·A/O: variieer de omtrek bij vast oppervlak.
        let b_small = calculate_u_equivalent(100.0, 80.0, 0.0, 0.4, false).unwrap(); // B'=2,5
        let b_mid = calculate_u_equivalent(100.0, 40.0, 0.0, 0.4, false).unwrap(); // B'=5
        let b_large = calculate_u_equivalent(100.0, 10.0, 0.0, 0.4, false).unwrap(); // B'=20
        assert!(
            b_small > b_mid && b_mid > b_large,
            "vloer: U_equiv daalt met B': {b_small}, {b_mid}, {b_large}"
        );

        // Wand: B'-invariant.
        let w_a = calculate_u_equivalent(100.0, 80.0, 1.0, 0.4, true).unwrap();
        let w_b = calculate_u_equivalent(100.0, 10.0, 1.0, 0.4, true).unwrap();
        assert!(
            (w_a - w_b).abs() < 1e-12,
            "wand: B' mag geen invloed hebben ({w_a} vs {w_b})"
        );
    }

    /// D4 (end-to-end): een z=0 maaiveld-grondvloer met auto-U_equiv (perimeter
    /// + depth gezet, u_equivalent = 0,0) levert via `calculate_h_t_ground` een
    /// geldig, positief H_T,ig — geen weigering op de callsite. Verifieert het
    /// volledige pad form. 4.21 → 4.24 met z = 0.
    #[test]
    fn test_d4_ground_floor_z_zero_end_to_end() {
        use crate::model::{ConstructionElement, BoundaryType, MaterialType, VerticalPosition, DesignConditions, enums::HeatingSystem};
        use crate::model::construction::GroundParameters;

        let climate = DesignConditions { theta_e: -10.0, theta_me: 9.0, theta_b_adjacent_building: 15.0 };

        let mk = |z: f64| ConstructionElement {
            id: "vloer".into(),
            description: "Maaiveld-grondvloer".into(),
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
                depth: Some(z),
            }),
            has_embedded_heating: false,
            unheated_space: None,
        };

        // z = 0 (maaiveld) moet gewoon werken — geen Err, positief verlies.
        let floor_z0 = mk(0.0);
        let h_z0 = calculate_h_t_ground(&[&floor_z0], 20.0, &climate, HeatingSystem::default());
        assert!(h_z0.is_ok(), "z=0 grondvloer moet via callsite geldig zijn: {h_z0:?}");
        assert!(h_z0.unwrap() > 0.0, "z=0 grondvloer moet positief H_T,ig geven");

        // Norm-voorbeelden z = 0 / 0,5 / 5 zijn alle geldig en positief.
        // In de norm-vorm 4.24 zit z in de noemer via (c₂+z)^n₂ (n₂ > 0):
        // grotere z → grotere noemer → láger U_equiv → láger H_T,ig (meer
        // gronddemping). We borgen geldigheid + de juiste (dalende) richting,
        // geen weigering op enige z in [0, 5].
        let h0 = calculate_h_t_ground(&[&floor_z0], 20.0, &climate, HeatingSystem::default()).unwrap();
        let h_half = calculate_h_t_ground(&[&mk(0.5)], 20.0, &climate, HeatingSystem::default()).unwrap();
        let h_five = calculate_h_t_ground(&[&mk(5.0)], 20.0, &climate, HeatingSystem::default()).unwrap();
        for (z, h) in [(0.0, h0), (0.5, h_half), (5.0, h_five)] {
            assert!(h > 0.0, "z={z} grondvloer moet positief H_T,ig geven, kreeg {h}");
        }
        assert!(
            h_half <= h0 && h_five <= h_half,
            "H_T,ig daalt licht met z (z in noemer): z0={h0}, z0.5={h_half}, z5={h_five}"
        );
    }

    /// IJkpunt 1 — norm-schilvoorbeeld (ISSO 53 PDF p.59/60): kantoorgebouw
    /// 50×20 m, vloer Rc = 3,5 → U_k = 1/(3,5+0,17+0,04) ≈ 0,2695, + ΔU_TB
    /// 0,1 (het voorbeeld rekent overal met `(Uk + 0,1)`, tabel 3.1 "overige
    /// situaties"); B' = 2·1000/140 = 14,29; z = 0. Norm-uitkomst: 0,18
    /// (p.59) resp. 0,17 (p.60); exacte norm-vorm: 0,18112.
    #[test]
    fn test_u_equivalent_worked_example_p59_schil() {
        let u_k = 1.0 / (3.5 + 0.17 + 0.04) + 0.1; // ≈ 0,36954
        let u_equiv = calculate_u_equivalent(1000.0, 140.0, 0.0, u_k, false).unwrap();
        assert!(
            (u_equiv - 0.181).abs() < 0.005,
            "schilvoorbeeld p.59 verwacht U_equiv ≈ 0,181, kreeg {u_equiv}"
        );
    }

    /// IJkpunt 2 — norm-detailvoorbeeld (ISSO 53 PDF p.65): B' = 12,07
    /// (gebouwniveau), beganegrondvloer U = 0,26 (lagen uit tabel 6.2:
    /// afwerkvloer + EPS 132 mm + beton) + ΔU_TB 0,05 (tabel 3.1 "nieuw
    /// gebouw volgens regels goed vakmanschap") = 0,31; z = 0.
    /// Norm-uitkomst: U_equiv = 0,177 (gebruikt in H_T,ig = 1,45 × 18,7 ×
    /// 0,177 × 1 × 0,351 = 1,69 W/K); exacte norm-vorm: 0,17741.
    ///
    /// NB: de 2,43 die op p.65 vlakbij staat is de **plafond**-U uit de
    /// H_T,ia-tabel ("Vertrek boven"), niet de grondvloer — de oude ijking
    /// "U_k = 2,43 → 0,177" was een misread.
    #[test]
    fn test_u_equivalent_worked_example_p65_detail() {
        // A/O zo gekozen dat B' = 2·A/O = 12,07 exact.
        let u_equiv = calculate_u_equivalent(120.7, 20.0, 0.0, 0.31, false).unwrap();
        assert!(
            (u_equiv - 0.177).abs() < 0.005,
            "detailvoorbeeld p.65 verwacht U_equiv ≈ 0,177, kreeg {u_equiv}"
        );
    }

    /// Smoke — vloer levert een fysisch plausibele U_equiv in [0,1; 1,0).
    #[test]
    fn test_u_equivalent_calculation_smoke() {
        // A=100, O=40 → B'=5; z=1; U=0,4; vloer. Norm-vorm: ≈ 0,2529.
        let u_equiv = calculate_u_equivalent(100.0, 40.0, 1.0, 0.4, false).unwrap();
        assert!(u_equiv >= U_EQUIV_MIN, "≥ ondergrens, kreeg {u_equiv}");
        assert!(u_equiv < 1.0, "fysisch plausibel, kreeg {u_equiv}");
        assert!((u_equiv - 0.2529).abs() < 0.002, "verwacht ≈ 0,2529, kreeg {u_equiv}");
    }

    /// ΔU_TB verhoogt U_k → verhoogt U_equiv (norm-vorm: (c₃+U_k)^n₃ met
    /// n₃ < 0 wordt kleiner → noemer kleiner → U_equiv groter). Borgt dat de
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
        // Hogere U_k → hogere U_equiv → hogere H_T,ig (norm-vorm-monotonie).
        assert!(h_custom > h_none, "ΔU_TB moet U_equiv verhogen: {h_custom} !> {h_none}");
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