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

    /// Maandelijkse hernieuwbare omgevingskoude Q_C;gen;out;rencold in MJ —
    /// de door de opwekker geleverde koude die als hernieuwbaar telt (BENG 3).
    ///
    /// NTA 8800:2025+C1:2026 §5.6.2.2 (formule 5.34, p. 105-106): alleen (vrije)
    /// koeling met `EER ≥ 8` levert rencold; compressie-/absorptiekoeling
    /// (forfait EER 3,0 / ζ 0,80 < 8) → 0. De EP-crate rekent dit om met
    /// `fPren;rencold = 1,0` (tabel 5.4).
    pub monthly_rencold_mj: MonthlyProfile<Energy>,

    /// Jaarsom van [`Self::monthly_rencold_mj`] in MJ.
    pub annual_rencold_mj: Energy,

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
            monthly_rencold_mj: zero_profile(),
            annual_rencold_mj: 0.0,
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
