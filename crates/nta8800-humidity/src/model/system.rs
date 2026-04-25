//! Humidity systemen: bevochtiging en ontvochtiging.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::model::target::HumidityTarget;

/// Bevochtigingssysteem configuratie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum HumidificationSystem {
    /// Stoomgenerator — elektrisch of gas-gevoed.
    Steam {
        /// Rendement η_steam in range [0,1] — typisch 0.95 voor moderne systemen.
        efficiency: f64,
    },

    /// Sproeikoeler met recirculatie — adiabatische koeling.
    SprayCoiler {
        /// Effectiviteit van de sproeikoeler in range [0,1] — typisch 0.80.
        effectiveness: f64,
    },

    /// Ultrasonic humidifier — fijnverstuiving.
    Ultrasonic {
        /// Elektrisch rendement in range [0,1] — typisch 0.85.
        efficiency: f64,
    },
}

/// Ontvochtigingssysteem configuratie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum DehumidificationSystem {
    /// Koeling onder dauwpunt — directe condensatie.
    Cooling {
        /// COP van de koelinstallatie — typisch 2.5-4.0.
        cop: f64,
    },

    /// Adsorptie dehumidifier — zeoliet/silica gel.
    Adsorption {
        /// COP van het adsorptiesysteem — typisch 3.0-4.5.
        cop: f64,
    },

    /// Sproeikoeler in cooling mode — evaporatieve koeling.
    EvaporativeCooling {
        /// Effectiviteit van de sproeikoeler in range [0,1] — typisch 0.75.
        effectiveness: f64,
    },
}

/// Complete humidity systeem configuratie voor een zone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HumiditySystemConfig {
    /// Bevochtigingssysteem — `None` betekent geen actieve bevochtiging.
    pub humidification: Option<HumidificationSystem>,

    /// Ontvochtigingssysteem — `None` betekent geen actieve ontvochtiging.
    pub dehumidification: Option<DehumidificationSystem>,

    /// Vochtigheidssetpoints voor deze zone.
    pub target: HumidityTarget,
}

impl HumidificationSystem {
    /// Geeft het systeem-rendement voor energieberekening.
    ///
    /// Voor Steam en Ultrasonic is dit de directe efficiency.
    /// Voor SprayCoiler wordt de effectiveness gebruikt als benaderend rendement.
    #[must_use]
    pub fn efficiency(&self) -> f64 {
        match self {
            Self::Steam { efficiency } | Self::Ultrasonic { efficiency } => *efficiency,
            Self::SprayCoiler { effectiveness } => *effectiveness,
        }
    }
}

impl DehumidificationSystem {
    /// Geeft de COP of effectiveness voor energieberekening.
    ///
    /// Voor Cooling en Adsorption is dit de COP.
    /// Voor EvaporativeCooling wordt effectiveness omgerekend naar een effectieve COP.
    #[must_use]
    pub fn cop(&self) -> f64 {
        match self {
            Self::Cooling { cop } | Self::Adsorption { cop } => *cop,
            // Voor evaporatieve koeling benaderen we COP via effectiveness:
            // hoe effectiever de koeler, hoe lager het energiegebruik.
            Self::EvaporativeCooling { effectiveness } => effectiveness * 5.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn steam_system_efficiency() {
        let steam = HumidificationSystem::Steam { efficiency: 0.95 };
        assert_abs_diff_eq!(steam.efficiency(), 0.95, epsilon = 1e-9);
    }

    #[test]
    fn spray_coiler_efficiency_via_effectiveness() {
        let spray = HumidificationSystem::SprayCoiler {
            effectiveness: 0.80,
        };
        assert_abs_diff_eq!(spray.efficiency(), 0.80, epsilon = 1e-9);
    }

    #[test]
    fn ultrasonic_efficiency() {
        let ultrasonic = HumidificationSystem::Ultrasonic { efficiency: 0.85 };
        assert_abs_diff_eq!(ultrasonic.efficiency(), 0.85, epsilon = 1e-9);
    }

    #[test]
    fn cooling_dehumidifier_cop() {
        let cooling = DehumidificationSystem::Cooling { cop: 3.5 };
        assert_abs_diff_eq!(cooling.cop(), 3.5, epsilon = 1e-9);
    }

    #[test]
    fn adsorption_dehumidifier_cop() {
        let adsorption = DehumidificationSystem::Adsorption { cop: 4.0 };
        assert_abs_diff_eq!(adsorption.cop(), 4.0, epsilon = 1e-9);
    }

    #[test]
    fn evaporative_cooling_effective_cop() {
        let evap = DehumidificationSystem::EvaporativeCooling {
            effectiveness: 0.75,
        };
        assert_abs_diff_eq!(evap.cop(), 0.75 * 5.0, epsilon = 1e-9);
    }
}