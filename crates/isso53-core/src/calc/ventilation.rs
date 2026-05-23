//! Ventilation heat loss for ISSO 53 (§4.7.2).

use crate::error::Result;
use crate::formulas::RHO_CP_AIR;
use crate::model::{Room, VentilationConfig};
use crate::tables::ventilation_requirements::{requirement, ventilation_rate_per_person};
use crate::model::enums::VentilatieBouwfase;
use crate::tables::temperature::design_indoor_temperature;

/// Results from ventilation calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct VentilationResult {
    /// Total ventilation heat loss Φ_vent in W.
    pub phi_vent: f64,
    /// Ventilation heat loss coefficient H_v in W/K.
    pub h_v: f64,
    /// Ventilation flow rate q_v in m³/s.
    pub q_v: f64,
    /// Temperature reduction factor f_v (dimensionless).
    pub f_v: f64,
}

/// Calculate ventilation heat loss for a room.
/// ISSO 53 formules 4.35-4.39, PDF p.47-50.
pub fn calculate_ventilation(
    room: &Room,
    config: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
) -> Result<VentilationResult> {
    // Calculate ventilation flow rate q_v in m³/s
    let q_v = calculate_ventilation_flow_rate(room)?;

    // Calculate temperature reduction factor f_v
    let f_v = calculate_f_v(config, theta_i, theta_e)?;

    // Calculate H_v: formule 4.37 - H_v = q_v × 1200 × f_v
    let h_v = q_v * RHO_CP_AIR * f_v;

    // Calculate Φ_vent: formule 4.35 - Φ_vent = H_v × (θ_i - θ_e)
    let phi_vent = h_v * (theta_i - theta_e);

    Ok(VentilationResult {
        phi_vent,
        h_v,
        q_v,
        f_v,
    })
}

/// Calculate the specific ventilation heat loss H_v for a room.
/// ISSO 53 formule 4.37, PDF p.47: H_v = q_v × 1200 × f_v
pub fn calculate_h_v(
    room: &Room,
    ventilation: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
) -> Result<f64> {
    let result = calculate_ventilation(room, ventilation, theta_i, theta_e)?;
    Ok(result.h_v)
}

/// Calculate the ventilation heat loss Φ_vent for a room.
/// ISSO 53 formule 4.35, PDF p.47: Φ_vent = H_v × (θ_i - θ_e)
pub fn calculate_phi_vent(
    room: &Room,
    ventilation: &VentilationConfig,
    theta_e: f64,
) -> Result<f64> {
    let theta_i = room.custom_temperature
        .unwrap_or_else(|| design_indoor_temperature(room.gebruiks_functie, room.ruimte_type));

    let result = calculate_ventilation(room, ventilation, theta_i, theta_e)?;
    Ok(result.phi_vent)
}

/// Calculate ventilation flow rate q_v in m³/s based on room occupancy and requirements.
fn calculate_ventilation_flow_rate(room: &Room) -> Result<f64> {
    // Get ventilation requirement for this room type
    let req = requirement(room.gebruiks_functie, room.ruimte_type)
        .ok_or_else(|| crate::error::Isso53Error::NotSupported(
            format!("No ventilation requirement found for {:?} {:?}", room.gebruiks_functie, room.ruimte_type)
        ))?;

    // Calculate number of people
    let people = if let Some(explicit_people) = room.bezetting.personen {
        explicit_people
    } else {
        let density = room.bezetting.personen_per_m2_default
            .or(req.personen_per_m2)
            .unwrap_or(0.05); // Default density
        room.floor_area * density
    };

    // Get ventilation rate per person in dm³/s
    let dm3_s_per_person = ventilation_rate_per_person(req, VentilatieBouwfase::Nieuwbouw)
        .unwrap_or(6.5); // Default rate

    // Convert to m³/s: q_v = (people × dm³/s per person) / 1000
    let q_v = people * dm3_s_per_person / 1000.0;

    Ok(q_v)
}

/// Calculate temperature reduction factor f_v.
/// ISSO 53 formules 4.38-4.39, PDF p.47-48.
fn calculate_f_v(config: &VentilationConfig, theta_i: f64, theta_e: f64) -> Result<f64> {
    if (theta_i - theta_e).abs() < 0.001 {
        return Ok(0.0); // Avoid division by zero
    }

    if config.has_heat_recovery {
        // WTW system - formule 4.38: f_v = (θ_t - θ_e - Δθ_v) / (θ_i - θ_e)
        let theta_t = if let Some(supply_temp) = config.supply_temperature {
            supply_temp
        } else {
            // Calculate based on heat recovery efficiency
            let efficiency = config.heat_recovery_efficiency.unwrap_or(0.75);
            theta_e + efficiency * (theta_i - theta_e)
        };

        // Simplified: Δθ_v = 0 (no frost protection considered)
        let delta_theta_v = 0.0;
        let f_v = (theta_t - theta_e - delta_theta_v) / (theta_i - theta_e);

        // Clamp to [0, 1]
        Ok(f_v.clamp(0.0, 1.0))
    } else if config.has_preheating {
        // Preheating system - check if luchtverwarming (θ_t > θ_i)
        let theta_t = config.preheating_temperature.unwrap_or(theta_i);

        if theta_t > theta_i {
            // Luchtverwarming: f_v = 0
            Ok(0.0)
        } else {
            // Formule 4.38: f_v = (θ_t - θ_e - Δθ_v) / (θ_i - θ_e)
            // Δθ_v = 0 (vereenvoudigd, geen kanaalverliezen)
            let f_v = (theta_t - theta_e) / (theta_i - theta_e);
            Ok(f_v.clamp(0.0, 1.0))
        }
    } else {
        // Natural ventilation - f_v = 1.0
        Ok(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        GebruiksFunctie, RuimteType, VentilationSystemType, ConstructionElement,
        BoundaryType, MaterialType, VerticalPosition, Bezetting
    };

    #[test]
    fn test_ventilation_calculation_basic() {
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
        assert!(result.is_ok());

        let ventilation = result.unwrap();
        assert!(ventilation.phi_vent > 0.0);
        assert!(ventilation.h_v > 0.0);
        assert!(ventilation.q_v > 0.0);
        assert!((ventilation.f_v - 1.0).abs() < 0.001); // Natural ventilation f_v = 1.0
    }

    #[test]
    fn test_ventilation_with_heat_recovery() {
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.8),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
        assert!(result.is_ok());

        let ventilation = result.unwrap();
        assert!(ventilation.phi_vent >= 0.0);
        assert!(ventilation.h_v >= 0.0);
        assert!(ventilation.f_v < 1.0); // Heat recovery reduces f_v
        assert!(ventilation.f_v >= 0.0);
    }

    #[test]
    fn test_ventilation_smoke() {
        // Kantoor, 1 persoon, geen WTW
        // q_v = 6.5/1000 = 0.0065 m³/s; H_v = 0.0065 × 1200 × 1 = 7.8 W/K
        let mut room = create_test_room();
        room.bezetting.personen = Some(1.0);

        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
        assert!(result.is_ok());
        let ventilation = result.unwrap();
        assert!((ventilation.h_v - 7.8).abs() < 0.1, "Expected ~7.8, got {}", ventilation.h_v);
        assert!((ventilation.f_v - 1.0).abs() < 0.001, "f_v should be 1.0 for natural ventilation");
    }

    fn create_test_room() -> Room {
        Room {
            id: "test_room".to_string(),
            name: "Test Office".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Kantoorruimte,
            floor_area: 25.0,
            height: 3.0,
            custom_temperature: Some(20.0),
            constructions: vec![
                ConstructionElement {
                    id: "wall".to_string(),
                    description: "Wall".to_string(),
                    area: 20.0,
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
                personen: Some(2.0),
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
        }
    }
}