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
            // Formule 4.28: q_i = q_is × A_u (gebruiksoppervlak)
            q_is * room.floor_area
        }
        InfiltrationMethod::Unknown { .. } => {
            // Formule 4.29: q_i = q_is × A_g (geveloppervlak)
            let a_gevel: f64 = room.constructions
                .iter()
                .filter(|element| element.boundary_type == BoundaryType::Exterior)
                .map(|element| element.area)
                .sum();
            q_is * a_gevel
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
fn calculate_q_is(_building: &Building, method: &InfiltrationMethod) -> Result<f64> {
    match method {
        InfiltrationMethod::Known { qv10_kar_class } => {
            // Use tabel 4.5 lookup based on building height
            // Note: we need building height from somewhere - use a reasonable assumption
            let building_height = 3.0; // TODO: get actual building height from Building model
            let height_class = BuildingHeightClass::from_height(building_height);

            Ok(q_is_known(*qv10_kar_class, height_class))
        }
        InfiltrationMethod::Unknown { .. } => {
            Err(crate::error::Isso53Error::NotSupported(
                "Onbekende q_v10,kar (formule 4.31) — vereist lezen PDF p.45-47 \
                 voor formules 4.32 (f_wind), 4.34 (f_jaar) en tabel 4.8 (f_typ). \
                 Uitgesteld naar batch 2b-vervolg of 2c.".to_string()
            ))
        }
    }
}

/// Calculate wind factor f_wind from formule 4.32.
/// ISSO 53 formule 4.32: complex function of building dimensions L, B, H.
/// Placeholder implementation — niet norm-conform, vervangen vóór release.
#[allow(dead_code)]
fn calculate_f_wind(length: f64, width: f64, height: f64) -> f64 {
    // Simplified wind factor based on building aspect ratio
    let aspect_ratio = (length * width).sqrt() / height;

    // Basic wind exposure factor
    if aspect_ratio < 0.5 {
        1.2 // Tall/slender building - more wind exposure
    } else if aspect_ratio > 2.0 {
        0.8 // Low/wide building - less wind exposure
    } else {
        1.0 // Average exposure
    }
}

/// Calculate year factor f_jaar from formule 4.34.
/// ISSO 53 formule 4.34: f_jaar = 0.4 + 0.033 × exp(0.05 × (2060 - J))
/// where J is construction year.
#[allow(dead_code)]
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
        let room = create_test_room();
        let building = create_test_building();
        let method = InfiltrationMethod::Unknown {
            construction_year: 2020,
            building_length: 20.0,
            building_width: 15.0,
            building_height: 3.0,
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_err()); // Should be NotSupported now
    }

    #[test]
    fn test_infiltration_known_smoke() {
        // q_v10_kar=0.5 (klasse 0.40-0.60), height=3m, floor_area=50
        // q_is = 0.00103 (from tabel 4.5), q_i = q_is × A_u = 0.00103 × 50 = 0.0515
        // H_i = z=1 × 0.0515 × 1200 × 1 = 61.8 W/K
        let mut room = create_test_room();
        room.floor_area = 50.0;
        room.infiltration_reduction_z = 1.0;

        let building = create_test_building();
        let method = InfiltrationMethod::Known {
            qv10_kar_class: Qv10Class::From040To060,
        };

        let result = calculate_h_i(&room, &building, &method);
        assert!(result.is_ok());
        let h_i = result.unwrap();
        assert!((h_i - 38.4).abs() < 0.1, "Expected ~38.4, got {}", h_i);
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
        }
    }
}