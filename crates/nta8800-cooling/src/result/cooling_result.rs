//! Resultaat-struct voor actieve koel-berekening (H.10, pad 1).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

use crate::model::EnergyCarrier;

/// Detail-uitsplitsing van de koel-berekening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingBreakdown {
    /// Maandelijkse netto koudebehoefte Q_C;nd;zi;mi in MJ (afkomstig uit
    /// demand-crate, opgenomen voor traceability).
    pub monthly_q_c_nd: MonthlyProfile<Energy>,

    /// η_em — afgifte-rendement gebruikt in berekening.
    pub emission_efficiency: f64,

    /// η_dist — distributie-rendement gebruikt in berekening.
    pub distribution_efficiency: f64,

    /// f_reg — regelfactor gebruikt in berekening.
    pub regulation_factor: f64,

    /// Effectieve COP / SCOP gebruikt voor de opwek-efficiëntie; voor vrije
    /// koeling geldt `1,0` (zie [`crate::model::CoolingSystem::nominal_cop`]).
    pub effective_cop: f64,

    /// Vrije-koeling factor (alleen relevant voor `FreeCooling`; anders 0).
    pub free_cooling_factor: f64,
}

/// Volledig resultaat van de actieve koel-berekening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingResult {
    /// Energiedrager van het koelsysteem (elektriciteit/gas/stadskoude).
    pub energy_carrier: EnergyCarrier,

    /// Maandelijks eindgebruik voor koeling Q_C;use;zi;mi in MJ (≥ 0).
    pub monthly_q_c_use: MonthlyProfile<Energy>,

    /// Jaarlijks eindgebruik voor koeling Q_C;use;an in MJ.
    pub annual_q_c_use: Energy,

    /// Detail-uitsplitsing (rendementen, COP, vrije-koeling factor).
    pub breakdown: CoolingBreakdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    #[test]
    fn cooling_result_serde_round_trip() {
        let r = CoolingResult {
            energy_carrier: EnergyCarrier::Electricity,
            monthly_q_c_use: zero_profile(),
            annual_q_c_use: 0.0,
            breakdown: CoolingBreakdown {
                monthly_q_c_nd: zero_profile(),
                emission_efficiency: 0.92,
                distribution_efficiency: 0.95,
                regulation_factor: 0.97,
                effective_cop: 4.0,
                free_cooling_factor: 0.0,
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: CoolingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
