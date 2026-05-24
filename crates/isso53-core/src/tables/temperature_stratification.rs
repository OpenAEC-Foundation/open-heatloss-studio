//! Temperatuur-gelaagdheid Δθ_2 per verwarmingssysteem voor ISSO 53 formule 4.23.
//!
//! Bron: ISSO 53 (2016) tabel 2.3, PDF p.21-22.

use crate::model::enums::HeatingSystem;

/// Temperatuur-correctie voor temperatuurgelaagdheid volgens ISSO 53 tabel 2.3.
/// Gebruikt in formule 4.23 voor vloer-f_ig berekening.
///
/// Formule 4.23: f_ig,k = ((θ_i + Δθ_2) − θ_me) / (θ_i − θ_e)
pub fn delta_theta_2(system: HeatingSystem) -> f64 {
    match system {
        HeatingSystem::LokaleVerwarming => -1.0,
        HeatingSystem::RadiatorenConvHtEnLuchtverwarming => -1.0,
        HeatingSystem::RadiatorenConvLt => -1.0,
        HeatingSystem::Plafondverwarming => 0.0,
        HeatingSystem::Wandverwarming => -1.0,
        HeatingSystem::Plintverwarming => -1.0,
        HeatingSystem::VloerverwarmingPlusHtRadi => 0.0,
        HeatingSystem::VloerverwarmingPlusLtRadi => 0.0,
        HeatingSystem::Vloerverwarming => 0.0,
        HeatingSystem::VloerverwarmingPlusWandverwarming => 0.0,
        HeatingSystem::Betonkernactivering => 0.0,
        HeatingSystem::VentilatorgedrevenConvRadi => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_heating_systems() {
        // Systemen met Δθ_2 = -1 K
        assert_eq!(delta_theta_2(HeatingSystem::LokaleVerwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::RadiatorenConvHtEnLuchtverwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::RadiatorenConvLt), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::Wandverwarming), -1.0);
        assert_eq!(delta_theta_2(HeatingSystem::Plintverwarming), -1.0);

        // Systemen met Δθ_2 = 0 K
        assert_eq!(delta_theta_2(HeatingSystem::Plafondverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusHtRadi), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusLtRadi), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::Vloerverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VloerverwarmingPlusWandverwarming), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::Betonkernactivering), 0.0);
        assert_eq!(delta_theta_2(HeatingSystem::VentilatorgedrevenConvRadi), 0.0);

        // Default systeem
        assert_eq!(delta_theta_2(HeatingSystem::default()), -1.0);
    }
}