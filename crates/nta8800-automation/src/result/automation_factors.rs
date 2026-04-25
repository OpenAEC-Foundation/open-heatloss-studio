//! Correctiefactoren voor gebouwautomatisering per energiedienst.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Resultaat van gebouwautomatisering-berekening: correctiefactoren per energiedienst.
///
/// Elke factor wordt toegepast op het netto energiegebruik van de corresponderende
/// dienst. Factoren < 1.0 betekenen energiebesparing door automatisering,
/// factoren > 1.0 betekenen energieverlies door slechte regeling.
///
/// Verwijzing: [`crate::references::NTA_8800_2025_FORMULE15_1`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AutomationFactors {
    /// Correctiefactor voor verwarmingssysteem (dimensieloos).
    pub f_bac_heating: f64,

    /// Correctiefactor voor koelingsysteem (dimensieloos).
    pub f_bac_cooling: f64,

    /// Correctiefactor voor verlichtingssysteem (dimensieloos).
    pub f_bac_lighting: f64,

    /// Correctiefactor voor warm tapwater systeem (dimensieloos).
    pub f_bac_dhw: f64,

    /// Correctiefactor voor ventilatiesysteem (dimensieloos).
    pub f_bac_ventilation: f64,
}

impl AutomationFactors {
    /// Maakt nieuwe factoren met opgegeven waarden per dienst.
    #[must_use]
    pub const fn new(
        f_bac_heating: f64,
        f_bac_cooling: f64,
        f_bac_lighting: f64,
        f_bac_dhw: f64,
        f_bac_ventilation: f64,
    ) -> Self {
        Self {
            f_bac_heating,
            f_bac_cooling,
            f_bac_lighting,
            f_bac_dhw,
            f_bac_ventilation,
        }
    }

    /// Factoren voor standaard regeling (alles 1.0 = geen correctie).
    #[must_use]
    pub const fn unity() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0, 1.0)
    }

    /// Geeft de factor voor een specifieke energiedienst.
    #[must_use]
    pub fn get_factor_for_service(&self, service: &str) -> Option<f64> {
        match service {
            "heating" => Some(self.f_bac_heating),
            "cooling" => Some(self.f_bac_cooling),
            "lighting" => Some(self.f_bac_lighting),
            "dhw" => Some(self.f_bac_dhw),
            "ventilation" => Some(self.f_bac_ventilation),
            _ => None,
        }
    }

    /// Controleert of alle factoren binnen een realistische range liggen [0.5, 2.0].
    #[must_use]
    pub fn is_physically_realistic(&self) -> bool {
        let factors = [
            self.f_bac_heating,
            self.f_bac_cooling,
            self.f_bac_lighting,
            self.f_bac_dhw,
            self.f_bac_ventilation,
        ];
        factors.iter().all(|&f| (0.5..=2.0).contains(&f))
    }

    /// Geeft het gemiddelde van alle factoren (voor rapportage-doeleinden).
    #[must_use]
    pub fn average_factor(&self) -> f64 {
        (self.f_bac_heating
            + self.f_bac_cooling
            + self.f_bac_lighting
            + self.f_bac_dhw
            + self.f_bac_ventilation)
            / 5.0
    }

    /// Telt het aantal diensten dat energiebesparing oplevert (factor < 1.0).
    #[must_use]
    pub fn count_energy_saving_services(&self) -> usize {
        [
            self.f_bac_heating,
            self.f_bac_cooling,
            self.f_bac_lighting,
            self.f_bac_dhw,
            self.f_bac_ventilation,
        ]
        .iter()
        .filter(|&&f| f < 1.0)
        .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn unity_factors() {
        let unity = AutomationFactors::unity();
        assert_relative_eq!(unity.f_bac_heating, 1.0);
        assert_relative_eq!(unity.f_bac_cooling, 1.0);
        assert_relative_eq!(unity.f_bac_lighting, 1.0);
        assert_relative_eq!(unity.f_bac_dhw, 1.0);
        assert_relative_eq!(unity.f_bac_ventilation, 1.0);
    }

    #[test]
    fn service_lookup() {
        let factors = AutomationFactors::new(0.9, 1.1, 0.8, 1.0, 0.95);
        assert_relative_eq!(factors.get_factor_for_service("heating").unwrap(), 0.9);
        assert_relative_eq!(factors.get_factor_for_service("lighting").unwrap(), 0.8);
        assert!(factors.get_factor_for_service("unknown").is_none());
    }

    #[test]
    fn physical_realism_check() {
        let realistic = AutomationFactors::new(0.9, 1.1, 0.8, 1.0, 1.5);
        assert!(realistic.is_physically_realistic());

        let unrealistic_low = AutomationFactors::new(0.3, 1.0, 1.0, 1.0, 1.0);
        assert!(!unrealistic_low.is_physically_realistic());

        let unrealistic_high = AutomationFactors::new(1.0, 1.0, 1.0, 1.0, 3.0);
        assert!(!unrealistic_high.is_physically_realistic());
    }

    #[test]
    fn average_calculation() {
        let factors = AutomationFactors::new(1.0, 0.8, 1.2, 0.9, 1.1);
        let expected = (1.0 + 0.8 + 1.2 + 0.9 + 1.1) / 5.0;
        assert_relative_eq!(factors.average_factor(), expected, epsilon = 1e-10);
    }

    #[test]
    fn energy_saving_count() {
        let factors = AutomationFactors::new(0.9, 1.1, 0.8, 1.0, 0.95);
        assert_eq!(factors.count_energy_saving_services(), 3); // heating, lighting, ventilation < 1.0
    }

    #[test]
    fn serde_round_trip() {
        let original = AutomationFactors::new(0.85, 1.15, 0.75, 1.05, 0.92);
        let json = serde_json::to_string(&original).unwrap();
        let back: AutomationFactors = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back);
    }
}