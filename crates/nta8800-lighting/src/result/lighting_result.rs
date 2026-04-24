//! Resultaat-struct voor één verlichtings-berekening.
//!
//! Energie altijd in **MJ**. De energiedrager is een constante
//! [`EnergyCarrier::Electricity`] — zie [`crate`] module-doc.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

use crate::model::EnergyCarrier;

/// Volledig resultaat van de verlichting-berekening voor één rekenzone.
///
/// Bevat het maandelijkse verbruik `W_L;use;mi` (MJ), het jaartotaal
/// `W_L;use;an` (MJ), de energiedrager (altijd elektrisch) en de
/// vloeroppervlakte die gebruikt is (voor traceability).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LightingResult {
    /// Energiedrager waarin `monthly_w_l_use` en `annual_w_l_use` gedrukt zijn.
    ///
    /// Altijd [`EnergyCarrier::Electricity`] voor verlichting (H.14).
    pub energy_carrier: EnergyCarrier,

    /// Maandelijks eindenergiegebruik voor verlichting `W_L;use;mi` in MJ.
    pub monthly_w_l_use: MonthlyProfile<Energy>,

    /// Jaarlijks eindenergiegebruik voor verlichting `W_L;use;an` in MJ.
    pub annual_w_l_use: Energy,

    /// Gebruiksoppervlakte `A_f` van de rekenzone in m² — input voor de
    /// berekening, bewaard voor audit-trace.
    pub floor_area_m2: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nta8800_model::time::Month;

    #[test]
    fn serde_round_trip() {
        let r = LightingResult {
            energy_carrier: EnergyCarrier::Electricity,
            monthly_w_l_use: MonthlyProfile::from_constant(100.0),
            annual_w_l_use: 1200.0,
            floor_area_m2: 100.0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: LightingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
        assert_relative_eq!(back.monthly_w_l_use[Month::Januari], 100.0, epsilon = 1e-12);
    }
}
