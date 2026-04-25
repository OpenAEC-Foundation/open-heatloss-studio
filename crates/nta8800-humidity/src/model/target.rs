//! Vochtigheidssetpoints en targets.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Vochtigheidssetpoints voor een zone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HumidityTarget {
    /// Minimum absolute vochtigheid in g/kg droge lucht.
    ///
    /// Typische waarden:
    /// - Kantoor: 4-6 g/kg
    /// - Woning: 6-8 g/kg
    /// - Ziekenhuis: 6-10 g/kg
    /// - Laboratorium: 4-8 g/kg (afhankelijk van proces)
    pub min_g_per_kg: f64,

    /// Maximum absolute vochtigheid in g/kg droge lucht.
    ///
    /// Typische waarden:
    /// - Kantoor: 10-12 g/kg
    /// - Woning: 10-14 g/kg
    /// - Ziekenhuis: 10-14 g/kg
    /// - Laboratorium: 8-12 g/kg (afhankelijk van proces)
    pub max_g_per_kg: f64,
}

impl HumidityTarget {
    /// Creëer een nieuwe `HumidityTarget` met validatie.
    ///
    /// # Errors
    ///
    /// Returnt een error als `min_g_per_kg >= max_g_per_kg` of als een van de
    /// waarden negatief is.
    pub fn new(min_g_per_kg: f64, max_g_per_kg: f64) -> Result<Self, &'static str> {
        if min_g_per_kg < 0.0 || max_g_per_kg < 0.0 {
            return Err("Vochtigheidswaarden kunnen niet negatief zijn");
        }
        if min_g_per_kg >= max_g_per_kg {
            return Err("Minimum vochtigheid moet lager zijn dan maximum");
        }
        Ok(Self {
            min_g_per_kg,
            max_g_per_kg,
        })
    }

    /// Standaard kantoor-setpoints: 6-12 g/kg.
    #[must_use]
    pub fn office() -> Self {
        Self {
            min_g_per_kg: 6.0,
            max_g_per_kg: 12.0,
        }
    }

    /// Standaard woning-setpoints: 6-12 g/kg.
    #[must_use]
    pub fn residential() -> Self {
        Self {
            min_g_per_kg: 6.0,
            max_g_per_kg: 12.0,
        }
    }

    /// Standaard ziekenhuis-setpoints: 8-12 g/kg (stricter voor comfort).
    #[must_use]
    pub fn hospital() -> Self {
        Self {
            min_g_per_kg: 8.0,
            max_g_per_kg: 12.0,
        }
    }

    /// Standaard laboratorium-setpoints: 4-8 g/kg (droog voor apparatuur).
    #[must_use]
    pub fn laboratory() -> Self {
        Self {
            min_g_per_kg: 4.0,
            max_g_per_kg: 8.0,
        }
    }

    /// Controleert of de gegeven vochtigheid binnen het target bereik ligt.
    #[must_use]
    pub fn is_within_range(&self, humidity_g_per_kg: f64) -> bool {
        humidity_g_per_kg >= self.min_g_per_kg && humidity_g_per_kg <= self.max_g_per_kg
    }

    /// Berekent de benodigde bevochtiging (positief) of ontvochtiging (negatief).
    ///
    /// Als de huidige vochtigheid binnen de range ligt, wordt 0.0 geretourneerd.
    #[must_use]
    pub fn required_adjustment(&self, current_humidity_g_per_kg: f64) -> f64 {
        if current_humidity_g_per_kg < self.min_g_per_kg {
            self.min_g_per_kg - current_humidity_g_per_kg
        } else if current_humidity_g_per_kg > self.max_g_per_kg {
            self.max_g_per_kg - current_humidity_g_per_kg // negatief = ontvochtiging
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn valid_humidity_target_creation() {
        let target = HumidityTarget::new(6.0, 12.0).unwrap();
        assert_abs_diff_eq!(target.min_g_per_kg, 6.0, epsilon = 1e-9);
        assert_abs_diff_eq!(target.max_g_per_kg, 12.0, epsilon = 1e-9);
    }

    #[test]
    fn invalid_humidity_target_negative() {
        assert!(HumidityTarget::new(-1.0, 12.0).is_err());
        assert!(HumidityTarget::new(6.0, -1.0).is_err());
    }

    #[test]
    fn invalid_humidity_target_min_equals_max() {
        assert!(HumidityTarget::new(10.0, 10.0).is_err());
    }

    #[test]
    fn invalid_humidity_target_min_greater_than_max() {
        assert!(HumidityTarget::new(12.0, 6.0).is_err());
    }

    #[test]
    fn preset_targets() {
        let office = HumidityTarget::office();
        assert_abs_diff_eq!(office.min_g_per_kg, 6.0, epsilon = 1e-9);
        assert_abs_diff_eq!(office.max_g_per_kg, 12.0, epsilon = 1e-9);

        let hospital = HumidityTarget::hospital();
        assert_abs_diff_eq!(hospital.min_g_per_kg, 8.0, epsilon = 1e-9);
        assert_abs_diff_eq!(hospital.max_g_per_kg, 12.0, epsilon = 1e-9);

        let lab = HumidityTarget::laboratory();
        assert_abs_diff_eq!(lab.min_g_per_kg, 4.0, epsilon = 1e-9);
        assert_abs_diff_eq!(lab.max_g_per_kg, 8.0, epsilon = 1e-9);
    }

    #[test]
    fn is_within_range_tests() {
        let target = HumidityTarget::office(); // 6-12 g/kg

        assert!(!target.is_within_range(5.9)); // below min
        assert!(target.is_within_range(6.0));  // at min
        assert!(target.is_within_range(9.0));  // within range
        assert!(target.is_within_range(12.0)); // at max
        assert!(!target.is_within_range(12.1)); // above max
    }

    #[test]
    fn required_adjustment_tests() {
        let target = HumidityTarget::office(); // 6-12 g/kg

        // Bevochtiging nodig
        assert_abs_diff_eq!(target.required_adjustment(4.0), 2.0, epsilon = 1e-9); // 6.0 - 4.0
        assert_abs_diff_eq!(target.required_adjustment(5.5), 0.5, epsilon = 1e-9); // 6.0 - 5.5

        // Binnen bereik
        assert_abs_diff_eq!(target.required_adjustment(6.0), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(target.required_adjustment(9.0), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(target.required_adjustment(12.0), 0.0, epsilon = 1e-9);

        // Ontvochtiging nodig (negatieve waarde)
        assert_abs_diff_eq!(target.required_adjustment(13.0), -1.0, epsilon = 1e-9); // 12.0 - 13.0
        assert_abs_diff_eq!(target.required_adjustment(14.5), -2.5, epsilon = 1e-9); // 12.0 - 14.5
    }
}