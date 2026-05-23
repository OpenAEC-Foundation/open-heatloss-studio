//! Heating-up supplement for ISSO 53 (§4.8).
//!
//! ISSO 53 uses a specific supplement P [W/m²] based on warm-up time and
//! building thermal mass, differing from the ISSO 51 main-room percentage method.

use crate::error::Result;
use crate::model::{HeatingUpConfig, Room};

/// Calculate the heating-up supplement Φ_hu for a room.
/// ISSO 53 §4.8.
///
/// ## TODO: P-table implementation
/// The specific supplement P [W/m²] should be looked up from the P-table in
/// PDF p.51-53 based on thermal mass and warm-up time. For now, this function
/// uses the configured `p_w_per_m2` value directly.
///
/// # Arguments
/// * `room` - The room to calculate for
/// * `config` - Heating-up configuration with P-value and setback settings
///
/// # Returns
/// Heating-up supplement Φ_hu in W
pub fn calculate_heating_up(
    room: &Room,
    config: &HeatingUpConfig,
) -> Result<f64> {
    if !config.setback_active {
        return Ok(0.0);
    }

    // Formule 4.40: Φ_hu = P × A_floor
    // where P comes from PDF p.51-53 P-table (not yet implemented)
    let phi_hu = config.p_w_per_m2 * room.floor_area;

    Ok(phi_hu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heating_up_inactive() {
        let room = create_test_room(25.0);
        let config = HeatingUpConfig {
            setback_active: false,
            p_w_per_m2: 10.0, // Should be ignored
            warmup_minutes: 120.0,
        };

        let result = calculate_heating_up(&room, &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_heating_up_active() {
        let room = create_test_room(25.0);
        let config = HeatingUpConfig {
            setback_active: true,
            p_w_per_m2: 10.0,
            warmup_minutes: 120.0,
        };

        let result = calculate_heating_up(&room, &config);
        assert!(result.is_ok());
        // Expected: 10.0 W/m² × 25.0 m² = 250.0 W
        assert_eq!(result.unwrap(), 250.0);
    }

    fn create_test_room(floor_area: f64) -> Room {
        use crate::model::{Bezetting, GebruiksFunctie, RuimteType};

        Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area,
            height: 3.0,
            custom_temperature: None,
            constructions: vec![],
            bezetting: Bezetting {
                personen: None,
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
        }
    }
}
