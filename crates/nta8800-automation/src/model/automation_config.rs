//! Automatiseringsconfiguratie per energiedienst.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::bacs_class::BacsClass;

/// Gebouwautomatisering-configuratie per energiedienst.
///
/// Definieert de BAC-klasse voor elke energiedienst afzonderlijk, omdat
/// verschillende systemen verschillende automatiseringsniveaus kunnen
/// hebben. Een gebouw kan bijvoorbeeld geavanceerde verlichting (klasse A)
/// hebben met basis-klimaatregeling (klasse C).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AutomationConfig {
    /// BAC-klasse voor verwarmingssysteem.
    pub heating: BacsClass,

    /// BAC-klasse voor koelingsysteem.
    pub cooling: BacsClass,

    /// BAC-klasse voor verlichtingssysteem.
    pub lighting: BacsClass,

    /// BAC-klasse voor warm tapwater (DHW) systeem.
    pub dhw: BacsClass,

    /// BAC-klasse voor ventilatiesysteem.
    pub ventilation: BacsClass,
}

impl AutomationConfig {
    /// Maakt een nieuwe configuratie met uniforme BAC-klasse voor alle diensten.
    #[must_use]
    pub const fn uniform(class: BacsClass) -> Self {
        Self {
            heating: class,
            cooling: class,
            lighting: class,
            dhw: class,
            ventilation: class,
        }
    }

    /// Standaardconfiguratie voor nieuwe gebouwen (klasse C = standard BACS).
    #[must_use]
    pub const fn standard() -> Self {
        Self::uniform(BacsClass::C)
    }

    /// Configuratie voor verouderde gebouwen zonder automatisering.
    #[must_use]
    pub const fn non_efficient() -> Self {
        Self::uniform(BacsClass::D)
    }

    /// High performance configuratie met geavanceerde automatisering.
    #[must_use]
    pub const fn high_performance() -> Self {
        Self::uniform(BacsClass::A)
    }

    /// Geeft de BAC-klasse voor een specifieke energiedienst.
    #[must_use]
    pub fn get_class_for_service(&self, service: &str) -> Option<BacsClass> {
        match service {
            "heating" => Some(self.heating),
            "cooling" => Some(self.cooling),
            "lighting" => Some(self.lighting),
            "dhw" => Some(self.dhw),
            "ventilation" => Some(self.ventilation),
            _ => None,
        }
    }

    /// Controleert of alle diensten ten minste de opgegeven minimale klasse hebben.
    #[must_use]
    pub fn all_services_at_least(&self, min_class: BacsClass) -> bool {
        use BacsClass::{A, B, C, D};
        let class_rank = |c: BacsClass| match c {
            A => 4,
            B => 3,
            C => 2,
            D => 1,
        };

        let min_rank = class_rank(min_class);
        [self.heating, self.cooling, self.lighting, self.dhw, self.ventilation]
            .iter()
            .all(|&c| class_rank(c) >= min_rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_config() {
        let config = AutomationConfig::uniform(BacsClass::B);
        assert_eq!(config.heating, BacsClass::B);
        assert_eq!(config.cooling, BacsClass::B);
        assert_eq!(config.lighting, BacsClass::B);
        assert_eq!(config.dhw, BacsClass::B);
        assert_eq!(config.ventilation, BacsClass::B);
    }

    #[test]
    fn presets() {
        let standard = AutomationConfig::standard();
        assert!(standard.all_services_at_least(BacsClass::C));

        let non_eff = AutomationConfig::non_efficient();
        assert!(!non_eff.all_services_at_least(BacsClass::C));

        let high_perf = AutomationConfig::high_performance();
        assert!(high_perf.all_services_at_least(BacsClass::A));
    }

    #[test]
    fn service_lookup() {
        let config = AutomationConfig {
            heating: BacsClass::A,
            cooling: BacsClass::B,
            lighting: BacsClass::C,
            dhw: BacsClass::D,
            ventilation: BacsClass::A,
        };

        assert_eq!(config.get_class_for_service("heating"), Some(BacsClass::A));
        assert_eq!(config.get_class_for_service("lighting"), Some(BacsClass::C));
        assert_eq!(config.get_class_for_service("unknown"), None);
    }

    #[test]
    fn at_least_check() {
        let mixed_config = AutomationConfig {
            heating: BacsClass::A,
            cooling: BacsClass::C,
            lighting: BacsClass::B,
            dhw: BacsClass::C,
            ventilation: BacsClass::D, // Dit voldoet niet aan klasse C minimum
        };

        assert!(!mixed_config.all_services_at_least(BacsClass::C));
        assert!(mixed_config.all_services_at_least(BacsClass::D));
    }

    #[test]
    fn serde_round_trip() {
        let original = AutomationConfig {
            heating: BacsClass::A,
            cooling: BacsClass::B,
            lighting: BacsClass::C,
            dhw: BacsClass::D,
            ventilation: BacsClass::A,
        };

        let json = serde_json::to_string(&original).unwrap();
        let back: AutomationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back);
    }
}