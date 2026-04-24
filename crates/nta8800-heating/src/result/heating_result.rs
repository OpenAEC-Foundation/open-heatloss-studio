//! Resultaat-struct voor één verwarming-keten berekening.
//!
//! Alle energiewaarden zijn in **MJ**, conform [`nta8800_model::units::Energy`].
//! De betekenis van Q_H;use hangt af van [`HeatingResult::energy_carrier`]:
//! gas-energie (HR-ketel), elektrische input (warmtepomp / weerstand) of
//! warmte-input aan gebouwgrens (stadsverwarming).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

use crate::model::EnergyCarrier;

/// Detail-uitsplitsing per keten-component voor traceability en rapportage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HeatingBreakdown {
    /// Afgifte-rendement η_em dat gebruikt is.
    pub emission_efficiency: f64,

    /// Distributie-rendement η_dist dat gebruikt is.
    pub distribution_efficiency: f64,

    /// Opwekkings-rendement η_gen dat gebruikt is (of SCOP voor warmtepomp).
    pub generation_efficiency: f64,

    /// Regel-factor f_reg die gebruikt is.
    pub control_factor: f64,

    /// Product η_em × η_dist × η_gen × f_reg (keten-rendement).
    ///
    /// Voor warmtepompen kan dit > 1 zijn. Q_H;use = Q_H;nd / total_efficiency.
    pub total_efficiency: f64,

    /// Maandprofiel van Q_H;nd (input uit demand-crate, in MJ).
    ///
    /// Meegenomen voor audit-trace — laat zien welke maandwaarden het keten-
    /// rendement gedeeld hebben.
    pub monthly_q_h_nd: MonthlyProfile<Energy>,
}

/// Volledig resultaat van de verwarming-keten berekening voor één rekenzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HeatingResult {
    /// Energiedrager waarin `monthly_q_h_use` en `annual_q_h_use` gedrukt zijn.
    ///
    /// - [`EnergyCarrier::Gas`] voor HR-ketel
    /// - [`EnergyCarrier::Electricity`] voor warmtepomp of elektr. weerstand
    /// - [`EnergyCarrier::DistrictHeat`] voor stadsverwarming
    pub energy_carrier: EnergyCarrier,

    /// Maandelijks eindenergiegebruik voor verwarming Q_H;use;mi in MJ.
    pub monthly_q_h_use: MonthlyProfile<Energy>,

    /// Jaarlijks eindenergiegebruik voor verwarming Q_H;use;an in MJ.
    pub annual_q_h_use: Energy,

    /// Detail-uitsplitsing (η-waarden + Q_H;nd input).
    pub breakdown: HeatingBreakdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::Month;

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    #[test]
    fn serde_round_trip() {
        let r = HeatingResult {
            energy_carrier: EnergyCarrier::Gas,
            monthly_q_h_use: zero_profile(),
            annual_q_h_use: 0.0,
            breakdown: HeatingBreakdown {
                emission_efficiency: 0.95,
                distribution_efficiency: 0.95,
                generation_efficiency: 0.95,
                control_factor: 1.0,
                total_efficiency: 0.95 * 0.95 * 0.95,
                monthly_q_h_nd: zero_profile(),
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: HeatingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn breakdown_field_count_is_stable() {
        // Sanity: de JSON round-trip test bewijst dat alle velden serialiseerbaar zijn.
        // Dit test voegt een compile-time check toe dat het publieke oppervlak
        // constant blijft door alle velden expliciet te noemen.
        let b = HeatingBreakdown {
            emission_efficiency: 1.0,
            distribution_efficiency: 1.0,
            generation_efficiency: 1.0,
            control_factor: 1.0,
            total_efficiency: 1.0,
            monthly_q_h_nd: MonthlyProfile::new([1.0; 12]),
        };
        assert!((b.monthly_q_h_nd[Month::Juli] - 1.0).abs() < 1e-12);
    }
}
