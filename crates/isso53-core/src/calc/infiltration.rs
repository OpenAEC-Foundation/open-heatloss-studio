//! Infiltration heat loss for ISSO 53 (§4.7.1).
//!
//! Two methods depending on whether `q_v10,kar` is known:
//! - Known: lookup in tabel 4.5
//! - Unknown: formule 4.31 with f_wind, f_type, f_inf, f_jaar

use crate::error::Result;
use crate::formulas::RHO_CP_AIR;
use crate::model::{BoundaryType, Building, DesignConditions, Room};
use crate::tables::infiltration::{q_is_known, BuildingHeightClass, Qv10Class};
use crate::tables::temperature::resolve_theta_i;

/// Infiltration calculation method.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InfiltrationMethod {
    /// q_v10,kar is known - use tabel 4.5 lookup.
    Known { qv10_kar_class: Qv10Class },
    /// q_v10,kar is unknown - use formule 4.31.
    Unknown {
        construction_year: u32,
        building_length: f64,
        building_width: f64,
        building_height: f64,
    },
    /// Vabi-conforme infiltratie via NEN 8088-1 (f_type/f_inf) + NTA 8800 (f_jaar)
    /// met power-law drukconversie (Δp/10)^0.67. Default Δp = 3.14 Pa (Vabi-fit).
    /// Bron: docs/2026-05-12-nta8800-infiltratie-verificatie.md
    #[serde(rename = "unknownVabiCompat")]
    UnknownVabiCompat {
        construction_year: u16,
        building_length: f64,
        building_width: f64,
        building_height: f64,
        #[serde(default)]
        delta_p_pa: Option<f64>, // None = 3.14
    },
}

/// Calculate the specific infiltration heat loss H_i for a room.
/// ISSO 53 formule 4.27, PDF p.44: H_i = z × q_i × 1200 × f_v
/// where f_v = 1.0 for infiltration (no preheating).
pub fn calculate_h_i(
    room: &Room,
    building: &Building,
    method: &InfiltrationMethod
) -> Result<f64> {
    // Calculate q_i based on method (different area calculations)
    let q_is = calculate_q_is(building, method)?;

    let q_i = match method {
        InfiltrationMethod::Known { .. } => {
            // Formule 4.28: q_i = q_is × A_u (uitwendige gevel = verticale buitenwanden)
            let a_u: f64 = room.constructions
                .iter()
                .filter(|element| element.boundary_type == BoundaryType::Exterior)
                // Tabel 4.5: A_u = uitwendige gevel (verticale buitenwanden), excl. plat dak
                // én vloer. Bij een zwevend gebouw zou een exterior vloer anders ten
                // onrechte als gevel meetellen (infiltratie te hoog).
                .filter(|element| element.vertical_position == crate::model::VerticalPosition::Wall)
                .map(|element| element.area)
                .sum();
            q_is * a_u
        }
        InfiltrationMethod::Unknown { .. } | InfiltrationMethod::UnknownVabiCompat { .. } => {
            // Formule 4.29: q_i = q_is × A_g (gebruiksoppervlakte = vloeroppervlak)
            q_is * room.floor_area
        }
    };

    // Apply reduction factor z from room infiltration_reduction_z
    let z = room.infiltration_reduction_z;

    // f_v = 1.0 for infiltration (no WTW or preheating)
    let f_v = 1.0;

    // H_i = z × q_i × ρ × c_p × f_v
    let h_i = z * q_i * RHO_CP_AIR * f_v;

    Ok(h_i)
}

/// Calculate the infiltration heat loss Φ_i for a room.
/// ISSO 53 formule 4.25, PDF p.44: Φ_i = H_i × (θ_i - θ_e)
pub fn calculate_phi_i(
    room: &Room,
    building: &Building,
    climate: &DesignConditions,
    method: &InfiltrationMethod,
) -> Result<f64> {
    let h_i = calculate_h_i(room, building, method)?;

    let theta_i: f64 = resolve_theta_i(room, climate.theta_e);

    let phi_i = h_i * (theta_i - climate.theta_e);

    Ok(phi_i)
}

/// Calculate specific infiltration q_is in m³/(s·m² gevel).
/// ISSO 53 §4.2: either tabel 4.5 lookup or formule 4.31.
fn calculate_q_is(building: &Building, method: &InfiltrationMethod) -> Result<f64> {
    match method {
        InfiltrationMethod::Known { qv10_kar_class } => {
            // Tabel 4.5: q_is afhankelijk van q_v10,kar-klasse × gebouwhoogte-klasse.
            let building_height = building.building_height.unwrap_or(3.0);
            let height_class = BuildingHeightClass::from_height(building_height);

            Ok(q_is_known(*qv10_kar_class, height_class))
        }
        InfiltrationMethod::Unknown { construction_year, .. } => {
            // Formule 4.31: q_is = f_wind · f_type · f_inf · (0,23 · q_i,spec)
            // Formule 4.33: q_i,spec = f_typ · f_jaar · q_i,spec,reken (tabel 4.9)
            use crate::tables::{building_type, ventilation_system, infiltration::q_i_spec_reken};

            let l = building.building_length.unwrap_or(0.0);
            let w = building.building_width.unwrap_or(0.0);
            let h = building.building_height.unwrap_or(3.0);

            let f_wind = calculate_f_wind(l, w, h);
            let f_type = building_type::f_type(building.wind_pressure_type);
            let f_inf = ventilation_system::f_inf(building.ventilation_system);
            let f_typ = building_type::f_typ(building.building_position);
            let f_jaar = calculate_f_jaar(*construction_year);
            let q_i_spec_basis = q_i_spec_reken(building.building_shape);
            let q_i_spec = f_typ * f_jaar * q_i_spec_basis;
            Ok(f_wind * f_type * f_inf * 0.23 * q_i_spec)
        }
        InfiltrationMethod::UnknownVabiCompat { construction_year, delta_p_pa, .. } => {
            // Vabi-compatibele infiltratie via NEN 8088-1 + NTA 8800 + power-law drukconversie
            use crate::tables::{building_type, nen8088, infiltration::q_i_spec_reken};

            let l = building.building_length.unwrap_or(0.0);
            let w = building.building_width.unwrap_or(0.0);
            let h = building.building_height.unwrap_or(3.0);

            #[allow(clippy::approx_constant)]
            let dp = delta_p_pa.unwrap_or(3.14); // Specific pressure value, not π
            if dp <= 0.0 {
                return Err(crate::error::Isso53Error::InvalidInput(
                    format!("delta_p_pa must be positive, got {}", dp)
                ));
            }
            let k = (dp / 10.0).powf(0.67); // ≈ 0.461 bij 3.14 Pa

            let f_wind = calculate_f_wind(l, w, h);
            let f_type = nen8088::f_type_nen8088(building.wind_pressure_type);
            let f_inf = nen8088::f_inf_nen8088(building.ventilation_system);
            let f_typ = building_type::f_typ(building.building_position);
            let f_jaar = nen8088::f_jaar_nta8800(*construction_year);
            let q_i_spec_basis = q_i_spec_reken(building.building_shape);
            let q_i_spec = f_typ * f_jaar * q_i_spec_basis;

            Ok(f_wind * f_type * f_inf * k * q_i_spec)
        }
    }
}

/// Correctiefactor f_wind volgens ISSO 53 formule 4.32 (PDF p.46):
/// `f_wind = max[1; (0,01 · (24 + 0,555 · √(L² + B²) + 4,5 · H))^0,65]`.
/// L, B, H in meter.
fn calculate_f_wind(length: f64, width: f64, height: f64) -> f64 {
    if length <= 0.0 || width <= 0.0 || height <= 0.0 {
        // Onvoldoende gebouwdimensies: conservatieve fallback f_wind = 1.
        return 1.0;
    }
    let diagonal = (length * length + width * width).sqrt();
    let inner = 0.01 * (24.0 + 0.555 * diagonal + 4.5 * height);
    inner.powf(0.65).max(1.0)
}

/// Invloedfactor f_jaar volgens ISSO 53 formule 4.34 (PDF p.47):
/// `f_jaar = 0,4 + 0,033 · exp(0,05 · (2060 − J))`. J = bouwjaar.
fn calculate_f_jaar(construction_year: u32) -> f64 {
    let j = construction_year as f64;
    0.4 + 0.033 * (0.05 * (2060.0 - j)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{GebruiksFunctie, RuimteType, VentilationSystemType, BuildingShape, ThermalMass, GebouwTypePositie, GebouwTypeWinddruk};

    #[test]
    fn test_f_jaar_calculation() {
        // Test formule 4.34: f_jaar = 0.4 + 0.033 × exp(0.05 × (2060 - J))
        let f_2020 = calculate_f_jaar(2020);
        let f_1990 = calculate_f_jaar(1990);

        assert!(f_1990 > f_2020, "Older buildings should have higher f_jaar");
        assert!(f_2020 > 0.4, "f_jaar should be > 0.4");
        assert!(f_2020 < 2.0, "f_jaar should be reasonable");
    }

    #[test]
    fn test_f_wind_basic() {
        let f_wind = calculate_f_wind(20.0, 10.0, 3.0);
        assert!(f_wind > 0.0);
        assert!(f_wind < 2.0);
    }

    #[test]
    fn test_infiltration_method_known() {
        let room = create_test_room();
        let building = create_test_building();
        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_ok());
        assert!(result.unwrap() >= 0.0);
    }

    #[test]
    fn test_systemd_infiltration_norm_compliant() {
        // Verify SystemD infiltration follows ISSO 53 tabel 4.7 correctly
        // ISSO 53 (2016) tabel 4.7 specifies f_inf = 1.15 for SystemD vs 0.80 for SystemA
        // This results in HIGHER infiltration for balanced ventilation, which is norm-compliant

        let room = create_test_room();
        let mut building = create_test_building();
        building.ventilation_system = VentilationSystemType::SystemD;
        building.building_length = Some(20.0);
        building.building_width = Some(15.0);
        building.building_height = Some(3.0);

        let method = InfiltrationMethod::Unknown {
            construction_year: 2020,
            building_length: 20.0,
            building_width: 15.0,
            building_height: 3.0,
        };

        let h_i_systemd = calculate_h_i(&room, &building, &method).unwrap();

        // Compare with SystemA
        let mut building_natural = building.clone();
        building_natural.ventilation_system = VentilationSystemType::SystemA;
        let h_i_systema = calculate_h_i(&room, &building_natural, &method).unwrap();

        // ISSO 53 norm verification: f_inf(SystemD) = 1.15, f_inf(SystemA) = 0.80
        // Expected ratio: 1.15 / 0.80 = 1.4375
        let expected_ratio = 1.15 / 0.80; // = 1.4375
        let actual_ratio = h_i_systemd / h_i_systema;

        assert!(
            (actual_ratio - expected_ratio).abs() < 0.01,
            "SystemD/SystemA infiltration ratio {} should match f_inf ratio {}",
            actual_ratio, expected_ratio
        );

        // SystemD should have HIGHER infiltration according to ISSO 53 tabel 4.7
        assert!(
            h_i_systemd > h_i_systema,
            "ISSO 53 tabel 4.7: SystemD f_inf=1.15 > SystemA f_inf=0.80, so SystemD infiltration should be higher"
        );

        // All assertions passed - SystemD infiltration is norm-compliant
    }

    #[test]
    fn test_infiltration_method_unknown() {
        // Unknown-pad (formule 4.31) levert nu een geldige berekening op.
        let room = create_test_room();
        let mut building = create_test_building();
        building.building_length = Some(20.0);
        building.building_width = Some(15.0);
        building.building_height = Some(3.0);

        let method = InfiltrationMethod::Unknown {
            construction_year: 2020,
            building_length: 20.0,
            building_width: 15.0,
            building_height: 3.0,
        };

        let h_i = calculate_h_i(&room, &building, &method).unwrap();
        assert!(h_i > 0.0, "Unknown-pad moet positieve H_i geven, kreeg {}", h_i);
    }

    #[test]
    fn test_f_wind_formule_4_32() {
        // DR Engineering voorbeeld: L=30, B=20, H=13 → Vabi rapporteert f_wind=1,0
        // √(900+400)=36,06; 24+0,555·36,06+4,5·13=102,5; 0,01·102,5=1,025
        // 1,025^0,65 ≈ 1,016 → max(1; 1,016) = 1,016
        let f = calculate_f_wind(30.0, 20.0, 13.0);
        assert!((f - 1.016).abs() < 0.01, "Expected ~1.016, got {}", f);

        // Klein gebouw (L=10, B=10, H=3) → inner < 1, dus max clamp → 1.0
        let f_small = calculate_f_wind(10.0, 10.0, 3.0);
        assert!((f_small - 1.0).abs() < 0.001, "Klein gebouw moet 1.0 geven, kreeg {}", f_small);

        // Groot gebouw (L=100, B=100, H=50) → f_wind > 1
        let f_large = calculate_f_wind(100.0, 100.0, 50.0);
        assert!(f_large > 1.5, "Groot gebouw moet f_wind > 1.5 geven, kreeg {}", f_large);

        // Geen dimensies → fallback 1.0
        assert_eq!(calculate_f_wind(0.0, 0.0, 0.0), 1.0);
    }

    #[test]
    fn test_f_jaar_extreme_years() {
        let f_2024 = calculate_f_jaar(2024);  // jong → laag
        let f_1960 = calculate_f_jaar(1960);  // oud → hoog
        assert!(f_2024 < 1.0, "Modern gebouw f_jaar < 1, kreeg {}", f_2024);
        assert!(f_1960 > 1.0, "Oud gebouw f_jaar > 1, kreeg {}", f_1960);
    }

    #[test]
    fn test_infiltration_known_smoke() {
        // q_v10_kar klasse 0.40-0.60, building_height=3m (default) → q_is=0.00064
        // test_room heeft 1 exterior wall area=30, dus A_u=30
        // H_i = z=1 × q_is × A_u × 1200 × f_v = 1 × 0.00064 × 30 × 1200 × 1 = 23.04 W/K
        let mut room = create_test_room();
        room.infiltration_reduction_z = 1.0;

        let building = create_test_building();
        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_ok());
        let h_i = result.unwrap();
        assert!((h_i - 23.04).abs() < 0.1, "Expected ~23.04, got {}", h_i);
    }

    #[test]
    fn test_infiltration_known_height_class() {
        // building_height=10m → klasse 6<h≤20, q_is=0.00103 (i.p.v. 0.00064 bij ≤3m)
        let mut room = create_test_room();
        room.infiltration_reduction_z = 1.0;

        let mut building = create_test_building();
        building.building_height = Some(10.0);

        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };

        let h_i = calculate_h_i(&room, &building, &method).unwrap();
        // 1 × 0.00103 × 30 × 1200 = 37.08 W/K
        assert!((h_i - 37.08).abs() < 0.1, "Expected ~37.08, got {}", h_i);
    }

    #[test]
    fn test_infiltration_known_exterior_floor_excluded_from_a_u() {
        // Regressie: ISSO 53 tabel 4.5 — A_u is de uitwendige GEVEL (verticale
        // buitenwanden). Bij een zwevend gebouw bestaat er een exterior vloer; die
        // mag NIET als gevel meetellen in de infiltratieberekening (Known-methode).
        use crate::model::{ConstructionElement, MaterialType, VerticalPosition};

        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };
        let building = create_test_building();

        // Basis-room: 1 exterior wand (area 30), vloer als Ground (geen gevel).
        let mut room_grounded = create_test_room();
        room_grounded.infiltration_reduction_z = 1.0;
        room_grounded.constructions.push(ConstructionElement {
            id: "ground_floor".to_string(),
            description: "Vloer op grond".to_string(),
            area: 240.0,
            u_value: 0.20,
            boundary_type: BoundaryType::Ground,
            material_type: MaterialType::NonMasonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Floor,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        });

        // Zwevend gebouw: identiek, maar de vloer is een EXTERIOR vloer.
        let mut room_floating = create_test_room();
        room_floating.infiltration_reduction_z = 1.0;
        room_floating.constructions.push(ConstructionElement {
            id: "exterior_floor".to_string(),
            description: "Blootgestelde vloer (zwevend)".to_string(),
            area: 240.0,
            u_value: 0.20,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::NonMasonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Floor,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        });

        let h_i_grounded = calculate_h_i(&room_grounded, &building, &method).unwrap();
        let h_i_floating = calculate_h_i(&room_floating, &building, &method).unwrap();

        assert!(
            (h_i_grounded - h_i_floating).abs() < 1e-9,
            "Exterior vloer mag A_u niet veranderen: grounded={} floating={}",
            h_i_grounded, h_i_floating
        );

        // Sanity: A_u = alléén de gevel (30 m²), niet de vloer (240 m²).
        // H_i = z(1) × q_is(0.00064) × 30 × 1200 = 23.04 W/K.
        assert!(
            (h_i_floating - 23.04).abs() < 0.1,
            "A_u mag alleen de 30 m² gevel tellen, kreeg H_i={}",
            h_i_floating
        );
    }

    #[test]
    fn test_infiltration_known_exterior_ceiling_excluded_from_a_u() {
        // Borg: een exterior plat dak (Ceiling) telt evenmin mee in A_u.
        use crate::model::{ConstructionElement, MaterialType, VerticalPosition};

        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };
        let building = create_test_building();

        let mut room = create_test_room();
        room.infiltration_reduction_z = 1.0;
        room.constructions.push(ConstructionElement {
            id: "flat_roof".to_string(),
            description: "Plat dak".to_string(),
            area: 240.0,
            u_value: 0.18,
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::NonMasonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Ceiling,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: false,
            unheated_space: None,
        });

        let h_i = calculate_h_i(&room, &building, &method).unwrap();
        // A_u blijft 30 m² gevel → 23.04 W/K, dak telt niet mee.
        assert!(
            (h_i - 23.04).abs() < 0.1,
            "Plat dak mag A_u niet vergroten, kreeg H_i={}",
            h_i
        );
    }

    fn create_test_room() -> Room {
        use crate::model::{Room, ConstructionElement, Bezetting, MaterialType, VerticalPosition};

        Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area: 25.0,
            height: 3.0,
            custom_temperature: None,
            constructions: vec![
                ConstructionElement {
                    id: "ext_wall".to_string(),
                    description: "Exterior wall".to_string(),
                    area: 30.0,
                    u_value: 0.28,
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
            has_mechanical_supply: None,
            ventilation_q_v_established: None,
        }
    }

    #[test]
    fn test_unknown_vabi_compat_negative_delta_p() {
        let room = create_test_room();
        let building = create_test_building();
        let method = InfiltrationMethod::UnknownVabiCompat {
            construction_year: 2020,
            building_length: 20.0,
            building_width: 15.0,
            building_height: 3.0,
            delta_p_pa: Some(-5.0), // Negative pressure should error
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_err(), "Negative delta_p should return error");

        let error = result.unwrap_err();
        assert!(format!("{}", error).contains("delta_p_pa must be positive"));
    }

    #[test]
    fn test_unknown_vabi_compat_positive_delta_p() {
        let room = create_test_room();
        let building = create_test_building();
        let method = InfiltrationMethod::UnknownVabiCompat {
            construction_year: 2020,
            building_length: 20.0,
            building_width: 15.0,
            building_height: 3.0,
            delta_p_pa: Some(5.0), // Positive pressure should work
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_ok(), "Positive delta_p should work");
        let h_i = result.unwrap();
        assert!(h_i > 0.0, "H_i should be positive");
    }

    fn create_test_building() -> Building {
        Building {
            building_shape: BuildingShape::EenLaagMetPlatDak,
            construction_year: 2020,
            building_position: GebouwTypePositie::MeerlaagsTussen,
            ventilation_system: VentilationSystemType::SystemB,
            thermal_mass: ThermalMass::Gemiddeld,
            wind_pressure_type: GebouwTypeWinddruk::EenlaagsMetPlatDak,
            building_height: None,
            building_length: None,
            building_width: None,
            heating_system: Default::default(),
            source_zone_config: Default::default(),
        }
    }
}