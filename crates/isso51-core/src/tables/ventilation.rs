//! BBL (Besluit bouwwerken leefomgeving) minimum ventilation requirements.
//!
//! Afdeling 3.6: Luchtverversing van verblijfsgebied, verblijfsruimte,
//! toiletruimte en badruimte.
//!
//! These minimum rates are used as defaults when no explicit ventilation
//! rate is specified per room.

use crate::model::enums::RoomFunction;

/// Minimum ventilation rate per room function according to BBL Afdeling 3.6.
///
/// # Arguments
/// * `function` - The room function
/// * `floor_area` - Floor area in m² (used for verblijfsruimten)
///
/// # Returns
/// Minimum ventilation rate in dm³/s.
///
/// # BBL Requirements
/// - Verblijfsruimte (living, bedroom, attic): 0.9 dm³/s per m², min 7 dm³/s
/// - Keuken: 21 dm³/s (afvoer kookdampen)
/// - Badkamer: 14 dm³/s
/// - Toilet: 7 dm³/s
/// - Gang/overloop/berging: 0 dm³/s (doorstroomruimte)
pub fn bbl_minimum_ventilation_rate(function: RoomFunction, floor_area: f64) -> f64 {
    match function {
        // Verblijfsruimten: 0.9 dm³/s per m², minimum 7 dm³/s
        RoomFunction::LivingRoom | RoomFunction::Bedroom | RoomFunction::Attic => {
            (0.9 * floor_area).max(7.0)
        }
        // Keuken: 21 dm³/s afvoer (kookdampen)
        RoomFunction::Kitchen => 21.0,
        // Badkamer: 14 dm³/s afvoer
        RoomFunction::Bathroom => 14.0,
        // Toilet: 7 dm³/s afvoer
        RoomFunction::Toilet => 7.0,
        // Doorstroomruimten: geen eigen ventilatie-eis
        RoomFunction::Hallway | RoomFunction::Landing | RoomFunction::Storage => 0.0,
        // Custom: geen automatische eis, gebruiker moet zelf specificeren
        RoomFunction::Custom => 0.0,
    }
}

/// Specific ventilation rate for verblijfsruimten in dm³/s per m².
/// BBL Afdeling 3.6.
pub const BBL_QV_SPEC_LIVING: f64 = 0.9;

/// Minimum ventilation rate per verblijfsruimte in dm³/s.
/// BBL Afdeling 3.6.
pub const BBL_QV_MIN_LIVING: f64 = 7.0;

/// Kitchen exhaust rate in dm³/s.
/// BBL Afdeling 3.6.
pub const BBL_QV_KITCHEN: f64 = 21.0;

/// Bathroom exhaust rate in dm³/s.
/// BBL Afdeling 3.6.
pub const BBL_QV_BATHROOM: f64 = 14.0;

/// Toilet exhaust rate in dm³/s.
/// BBL Afdeling 3.6.
pub const BBL_QV_TOILET: f64 = 7.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_living_room_large() {
        // 28 m² woonkamer: 0.9 × 28 = 25.2 dm³/s
        let rate = bbl_minimum_ventilation_rate(RoomFunction::LivingRoom, 28.0);
        assert!((rate - 25.2).abs() < 0.01);
    }

    #[test]
    fn test_living_room_minimum_floor() {
        // 5 m² ruimte: 0.9 × 5 = 4.5, maar minimum is 7 dm³/s
        let rate = bbl_minimum_ventilation_rate(RoomFunction::LivingRoom, 5.0);
        assert!((rate - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_bedroom() {
        // 12 m² slaapkamer: 0.9 × 12 = 10.8 dm³/s
        let rate = bbl_minimum_ventilation_rate(RoomFunction::Bedroom, 12.0);
        assert!((rate - 10.8).abs() < 0.01);
    }

    #[test]
    fn test_small_bedroom_minimum() {
        // 6 m² slaapkamer: 0.9 × 6 = 5.4, maar minimum is 7 dm³/s
        let rate = bbl_minimum_ventilation_rate(RoomFunction::Bedroom, 6.0);
        assert!((rate - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_kitchen() {
        // Keuken altijd 21 dm³/s, ongeacht oppervlak
        assert!((bbl_minimum_ventilation_rate(RoomFunction::Kitchen, 8.0) - 21.0).abs() < 0.01);
        assert!((bbl_minimum_ventilation_rate(RoomFunction::Kitchen, 20.0) - 21.0).abs() < 0.01);
    }

    #[test]
    fn test_bathroom() {
        assert!((bbl_minimum_ventilation_rate(RoomFunction::Bathroom, 6.0) - 14.0).abs() < 0.01);
    }

    #[test]
    fn test_toilet() {
        assert!((bbl_minimum_ventilation_rate(RoomFunction::Toilet, 2.0) - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_hallway_no_requirement() {
        assert_eq!(bbl_minimum_ventilation_rate(RoomFunction::Hallway, 10.0), 0.0);
    }

    #[test]
    fn test_landing_no_requirement() {
        assert_eq!(bbl_minimum_ventilation_rate(RoomFunction::Landing, 5.0), 0.0);
    }

    #[test]
    fn test_storage_no_requirement() {
        assert_eq!(bbl_minimum_ventilation_rate(RoomFunction::Storage, 8.0), 0.0);
    }

    #[test]
    fn test_custom_no_requirement() {
        assert_eq!(bbl_minimum_ventilation_rate(RoomFunction::Custom, 20.0), 0.0);
    }

    #[test]
    fn test_attic_as_living_space() {
        // 15 m² zolder: 0.9 × 15 = 13.5 dm³/s
        let rate = bbl_minimum_ventilation_rate(RoomFunction::Attic, 15.0);
        assert!((rate - 13.5).abs() < 0.01);
    }
}
