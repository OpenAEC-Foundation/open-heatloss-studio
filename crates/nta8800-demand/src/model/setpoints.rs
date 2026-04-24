//! Setpoint-profielen voor verwarming en koeling.
//!
//! NTA 8800 §7.4 gebruikt `θ_int;calc;H;zi;mi` (verwarmings-setpoint) en
//! §7.5 `θ_int;calc;C;zi;mi` (koel-setpoint). Deze module biedt type-veilige
//! wrappers om verwisseling tussen beide te voorkomen.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Temperature;

/// Verwarmings-setpoint `θ_int;calc;H;zi;mi` in °C, per maand.
///
/// Typisch 20 °C voor Woonfunctie (jaarrond) conform NTA 8800 tabel 7.2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HeatingSetpoint {
    /// Maandprofiel van de verwarmings-setpoint-temperatuur in °C.
    pub temperature: MonthlyProfile<Temperature>,
}

impl HeatingSetpoint {
    /// Construct vanuit maandprofiel.
    #[must_use]
    pub const fn new(profile: MonthlyProfile<Temperature>) -> Self {
        Self {
            temperature: profile,
        }
    }

    /// Constante waarde voor alle 12 maanden (typisch 20 °C).
    #[must_use]
    pub fn constant(value: Temperature) -> Self {
        Self {
            temperature: MonthlyProfile::from_constant(value),
        }
    }
}

/// Koel-setpoint `θ_int;calc;C;zi;mi` in °C, per maand.
///
/// Typisch 24 °C voor Woonfunctie (NTA 8800 tabel 7.2). `None` buiten het
/// koel-seizoen (januari/december) wordt uitgedrukt door een hogere setpoint
/// die niet wordt bereikt — bewust géén `Option` om de maand-balans simpel te
/// houden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingSetpoint {
    /// Maandprofiel van de koel-setpoint-temperatuur in °C.
    pub temperature: MonthlyProfile<Temperature>,
}

impl CoolingSetpoint {
    /// Construct vanuit maandprofiel.
    #[must_use]
    pub const fn new(profile: MonthlyProfile<Temperature>) -> Self {
        Self {
            temperature: profile,
        }
    }

    /// Constante waarde voor alle 12 maanden (typisch 24 °C).
    #[must_use]
    pub fn constant(value: Temperature) -> Self {
        Self {
            temperature: MonthlyProfile::from_constant(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::Month;

    #[test]
    fn heating_constant_20c() {
        let sp = HeatingSetpoint::constant(20.0);
        assert!((sp.temperature[Month::Januari] - 20.0).abs() < 1e-9);
        assert!((sp.temperature[Month::Juli] - 20.0).abs() < 1e-9);
    }

    #[test]
    fn cooling_constant_24c() {
        let sp = CoolingSetpoint::constant(24.0);
        assert!((sp.temperature[Month::Juli] - 24.0).abs() < 1e-9);
    }

    #[test]
    fn serde_round_trip() {
        let sp = HeatingSetpoint::constant(20.5);
        let json = serde_json::to_string(&sp).unwrap();
        let back: HeatingSetpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(sp, back);
    }
}
