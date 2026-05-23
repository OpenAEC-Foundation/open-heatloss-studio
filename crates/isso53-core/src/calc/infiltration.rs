//! Infiltration heat loss for ISSO 53 (§4.7.1).
//!
//! Two methods depending on whether `q_v10,kar` is known:
//! - Known: lookup in tabel 4.5
//! - Unknown: formule 4.31 with f_wind, f_type, f_inf, f_jaar

use crate::error::Result;
use crate::formulas::RHO_CP_AIR;
use crate::model::{BoundaryType, Building, DesignConditions, Room};
use crate::tables::infiltration::{q_is_known, BuildingHeightClass, Qv10Class};
use crate::tables::temperature::design_indoor_temperature;

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
            // Formule 4.28: q_i = q_is × A_u (uitwendige scheidingsconstructie = gevel excl. plat dak)
            let a_u: f64 = room.constructions
                .iter()
                .filter(|element| element.boundary_type == BoundaryType::Exterior)
                // Tabel 4.5 voetnoot/§2.2: A_u is gevel excl. plat dak.
                .filter(|element| element.vertical_position != crate::model::VerticalPosition::Ceiling)
                .map(|element| element.area)
                .sum();
            q_is * a_u
        }
        InfiltrationMethod::Unknown { .. } => {
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

    let theta_i: f64 = room.custom_temperature
        .unwrap_or_else(|| design_indoor_temperature(room.gebruiks_functie, room.ruimte_type));

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
        }
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
        }
    }
}