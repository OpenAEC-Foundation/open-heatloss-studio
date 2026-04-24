//! Resultaat-struct voor één maand-balans berekening.
//!
//! Alle energiewaarden zijn in MJ, conform [`nta8800_model::units::Energy`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

/// Detail-uitsplitsing van de maand-balans per rekenzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DemandBreakdown {
    /// Totale maandelijkse warmte-overdracht door transmissie + ventilatie
    /// `Q_H;ht;zi;mi` in MJ.
    pub monthly_q_ht: MonthlyProfile<Energy>,

    /// Totale maandelijkse warmtewinst `Q_H;gn;zi;mi = Q_int + Q_sol` in MJ.
    pub monthly_q_gn: MonthlyProfile<Energy>,

    /// Maandelijkse zoninstraling door transparante delen `Q_sol;mi` in MJ.
    pub monthly_q_sol: MonthlyProfile<Energy>,

    /// Maandelijkse interne warmtelast `Q_int;mi` in MJ.
    pub monthly_q_int: MonthlyProfile<Energy>,

    /// Benuttingsfactor voor warmtewinst η_H;gn per maand (dimensieloos, 0..=1).
    pub monthly_utilization_heating: MonthlyProfile<f64>,

    /// Benuttingsfactor voor koudeverlies η_C;ls per maand (dimensieloos, 0..=1).
    pub monthly_utilization_cooling: MonthlyProfile<f64>,

    /// Tijdconstante van de rekenzone τ in uren (constant, §7.8).
    pub time_constant_hours: f64,
}

/// Volledig resultaat van de maand-balans voor één rekenzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DemandResult {
    /// Maandelijkse netto warmtebehoefte `Q_H;nd;zi;mi` in MJ (≥ 0).
    pub monthly_heating_demand: MonthlyProfile<Energy>,

    /// Maandelijkse netto koudebehoefte `Q_C;nd;zi;mi` in MJ (≥ 0).
    pub monthly_cooling_demand: MonthlyProfile<Energy>,

    /// Jaarlijkse netto warmtebehoefte `Q_H;nd;an` in MJ.
    pub annual_heating_demand: Energy,

    /// Jaarlijkse netto koudebehoefte `Q_C;nd;an` in MJ.
    pub annual_cooling_demand: Energy,

    /// Detail-uitsplitsing (monthly_q_ht, monthly_q_gn, η-profielen, τ).
    pub breakdown: DemandBreakdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    fn zero_eta() -> MonthlyProfile<f64> {
        MonthlyProfile::from_constant(0.0)
    }

    #[test]
    fn serde_round_trip() {
        let r = DemandResult {
            monthly_heating_demand: zero_profile(),
            monthly_cooling_demand: zero_profile(),
            annual_heating_demand: 0.0,
            annual_cooling_demand: 0.0,
            breakdown: DemandBreakdown {
                monthly_q_ht: zero_profile(),
                monthly_q_gn: zero_profile(),
                monthly_q_sol: zero_profile(),
                monthly_q_int: zero_profile(),
                monthly_utilization_heating: zero_eta(),
                monthly_utilization_cooling: zero_eta(),
                time_constant_hours: 48.5,
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: DemandResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
