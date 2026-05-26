//! Ventilation system configuration for ISSO 51.
//!
//! Defines the building-level ventilation system parameters
//! used in the ventilation heat loss calculation (§2.5.7).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{FrostProtectionType, VentilationSystemType};

/// Building-level ventilation system configuration.
/// ISSO 51 §2.5.7.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VentilationConfig {
    /// Type of ventilation system (A through E).
    pub system_type: VentilationSystemType,

    /// Whether heat recovery (WTW) is installed.
    #[serde(default)]
    pub has_heat_recovery: bool,

    /// Heat recovery efficiency (0.0 to 1.0).
    /// E.g., 0.85 for 85% efficiency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_recovery_efficiency: Option<f64>,

    /// Frost protection type for the heat recovery unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_protection: Option<FrostProtectionType>,

    /// Supply air temperature θ_t in °C after heat recovery.
    /// If not set, will be calculated from efficiency and frost protection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_temperature: Option<f64>,

    /// Whether there is pre-heating of supply air (without WTW).
    #[serde(default)]
    pub has_preheating: bool,

    /// Pre-heating supply temperature in °C.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheating_temperature: Option<f64>,
}

impl VentilationConfig {
    /// Returns the supply air temperature θ_t in °C.
    ///
    /// Prioriteit:
    /// 1. Expliciet ingestelde `supply_temperature` — manual override
    /// 2. Voorverwarming zonder WTW — `preheating_temperature`
    /// 3. WTW met bekend rendement η — fysische formule
    ///    `θ_t = θ_e + η × (θ_i − θ_e)`
    /// 4. WTW zonder rendement — ISSO 51 Tabel 2.14 (erratum 2023) via `frost_protection`
    /// 5. Natuurlijke toevoer — θ_t = θ_e
    pub fn effective_supply_temperature(&self, theta_e: f64, theta_i: f64) -> f64 {
        // If explicitly set, use that
        if let Some(t) = self.supply_temperature {
            return t;
        }

        // If pre-heating without WTW
        if self.has_preheating {
            return self.preheating_temperature.unwrap_or(5.0);
        }

        // If heat recovery is installed
        if self.has_heat_recovery {
            // Fysische formule wanneer η bekend is: θ_t = θ_e + η × (θ_i − θ_e)
            if let Some(eta) = self.heat_recovery_efficiency {
                return theta_e + eta * (theta_i - theta_e);
            }
            // Fallback: ISSO 51 Tabel 2.14 (forfaitaire θ_t per frost-protection type)
            if let Some(fp) = &self.frost_protection {
                return fp.supply_temperature();
            }
            // Default for unknown frost protection
            return 10.0;
        }

        // Natural supply: air comes in at outdoor temperature
        theta_e
    }
}

impl FrostProtectionType {
    /// Returns the supply temperature θ_t in °C.
    /// ISSO 51 Table 2.14 (erratum 2023).
    pub fn supply_temperature(&self) -> f64 {
        match self {
            FrostProtectionType::Unknown => 10.0,
            FrostProtectionType::CentralReducedSpeed => 10.0,
            FrostProtectionType::CentralEnthalpy => 12.0,
            FrostProtectionType::CentralPreheating => 16.0,
            FrostProtectionType::DecentralReducedSpeed => 10.0,
            FrostProtectionType::DecentralEnthalpy => 12.0,
            FrostProtectionType::DecentralPreheating => 14.0,
            FrostProtectionType::ElectricPreheating => 5.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_supply_temperature() {
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemC,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };
        assert_eq!(config.effective_supply_temperature(-10.0, 20.0), -10.0);
    }

    #[test]
    fn test_wtw_supply_temperature_with_efficiency() {
        // WTW met bekend rendement → fysische formule θ_t = θ_e + η × (θ_i − θ_e)
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: Some(FrostProtectionType::CentralReducedSpeed),
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };
        // -10 + 0.85 × (20 − (−10)) = -10 + 25.5 = 15.5
        let theta_t = config.effective_supply_temperature(-10.0, 20.0);
        assert!(
            (theta_t - 15.5).abs() < 1e-6,
            "θ_t bij η=0.85 moet 15.5 °C zijn, kreeg {theta_t}"
        );
    }

    #[test]
    fn test_wtw_supply_temperature_efficiency_varies() {
        // Verschillende rendementen moeten verschillende θ_t geven (bug-regressie)
        let mk = |eta: f64| VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(eta),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };
        let t_50 = mk(0.5).effective_supply_temperature(-10.0, 20.0);
        let t_85 = mk(0.85).effective_supply_temperature(-10.0, 20.0);
        let t_95 = mk(0.95).effective_supply_temperature(-10.0, 20.0);
        assert!((t_50 - 5.0).abs() < 1e-6, "η=0.50 → 5°C, kreeg {t_50}");
        assert!((t_85 - 15.5).abs() < 1e-6, "η=0.85 → 15.5°C, kreeg {t_85}");
        assert!((t_95 - 18.5).abs() < 1e-6, "η=0.95 → 18.5°C, kreeg {t_95}");
    }

    #[test]
    fn test_wtw_fallback_to_frost_protection_table() {
        // WTW zonder rendement → ISSO 51 Tabel 2.14
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: None,
            frost_protection: Some(FrostProtectionType::CentralReducedSpeed),
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };
        assert_eq!(config.effective_supply_temperature(-10.0, 20.0), 10.0);
    }

    #[test]
    fn test_explicit_supply_temperature() {
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: None,
            supply_temperature: Some(15.0),
            has_preheating: false,
            preheating_temperature: None,
        };
        assert_eq!(config.effective_supply_temperature(-10.0, 20.0), 15.0);
    }
}
