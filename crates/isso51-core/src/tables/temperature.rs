//! Temperature correction values Δθ for different heating systems.
//! ISSO 51 Table 2.12 (erratum 2023).

use crate::model::enums::HeatingSystem;

/// Temperature corrections for a heating system.
/// ISSO 51 Table 2.12 (erratum 2023).
pub struct DeltaTheta {
    /// Δθ₁ or Δθ_a1: correction for rooms above/below (vertical temperature
    /// gradient).
    ///
    /// ISSO 51 Table 2.12, voetnoot 2:
    /// *"Bij toepassing van vides etc. waardoor een grotere hoogte ontstaat
    /// moet de waarde van Δθ₁ worden vermenigvuldigd met h/4 waarbij h de
    /// totale hoogte [m] is."*
    ///
    /// The height correction is applied at the consumer side, not in this
    /// table — see [`crate::calc::room_load::calculate_room`] where the
    /// `height_factor = h / 4` multiplier is applied to `delta_1` for rooms
    /// with `room.height > 4.0 m`. Tabulated values below assume a standard
    /// room height (≈ 2.6–3.0 m, factor 1.0).
    pub delta_1: f64,
    /// Δθ₂ or Δθ_a2: correction for adjacent rooms (floor level).
    pub delta_2: f64,
    /// Δθ_v for Ū > 0.5: correction for ventilation air temperature.
    pub delta_v_high: f64,
    /// Δθ_v for Ū ≤ 0.5: correction for ventilation air temperature.
    pub delta_v_low: f64,
}

/// Returns the Δθ correction values for a given heating system.
/// ISSO 51 Table 2.12 (erratum 2023).
pub fn delta_theta(system: HeatingSystem) -> DeltaTheta {
    match system {
        HeatingSystem::LocalGasHeater => DeltaTheta {
            delta_1: 4.0,
            delta_2: -1.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::IrPanelWall => DeltaTheta {
            delta_1: 1.0,
            delta_2: -0.5,
            delta_v_high: -1.5,
            delta_v_low: -1.0,
        },
        HeatingSystem::IrPanelCeiling => DeltaTheta {
            delta_1: 0.0,
            delta_2: 0.0,
            delta_v_high: -1.5,
            delta_v_low: -1.0,
        },
        HeatingSystem::RadiatorHt => DeltaTheta {
            delta_1: 3.0,
            delta_2: -1.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::RadiatorLt => DeltaTheta {
            delta_1: 2.0,
            delta_2: -1.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::CeilingHeating => DeltaTheta {
            delta_1: 3.0,
            delta_2: 0.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::WallHeating => DeltaTheta {
            delta_1: 2.0,
            delta_2: -1.0,
            delta_v_high: -1.0,
            delta_v_low: -0.5,
        },
        HeatingSystem::PlinthHeating => DeltaTheta {
            delta_1: 1.0,
            delta_2: -1.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::FloorHeatingWithRadiatorHt => DeltaTheta {
            delta_1: 3.0,
            delta_2: 0.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
        HeatingSystem::FloorHeatingWithRadiatorLt => DeltaTheta {
            delta_1: 2.0,
            delta_2: 0.0,
            delta_v_high: -1.0,
            delta_v_low: -0.5,
        },
        HeatingSystem::FloorHeatingMainHigh => DeltaTheta {
            delta_1: 0.0,
            delta_2: 0.0,
            delta_v_high: -1.0,
            delta_v_low: -0.5,
        },
        HeatingSystem::FloorHeatingMainLow => DeltaTheta {
            delta_1: 0.0,
            delta_2: 0.0,
            delta_v_high: -0.5,
            delta_v_low: 0.0,
        },
        HeatingSystem::FloorAndWallHeating => DeltaTheta {
            delta_1: 1.0,
            delta_2: 0.0,
            delta_v_high: -1.0,
            delta_v_low: -0.5,
        },
        HeatingSystem::FanConvector => DeltaTheta {
            delta_1: 0.5,
            delta_2: 0.0,
            delta_v_high: 0.0,
            delta_v_low: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radiator_lt_values() {
        let dt = delta_theta(HeatingSystem::RadiatorLt);
        assert_eq!(dt.delta_1, 2.0);
        assert_eq!(dt.delta_2, -1.0);
        assert_eq!(dt.delta_v_high, 0.0);
        assert_eq!(dt.delta_v_low, 0.0);
    }

    #[test]
    fn test_floor_heating_main_high() {
        let dt = delta_theta(HeatingSystem::FloorHeatingMainHigh);
        assert_eq!(dt.delta_1, 0.0);
        assert_eq!(dt.delta_v_high, -1.0);
        assert_eq!(dt.delta_v_low, -0.5);
    }
}
