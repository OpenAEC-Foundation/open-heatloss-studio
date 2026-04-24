//! Resultaat-struct voor één warm-tapwater keten-berekening.
//!
//! Alle energiewaarden zijn in **MJ**, conform [`nta8800_model::units::Energy`].
//! De betekenis van `Q_W;use` hangt af van [`DhwResult::energy_carrier`]:
//! gas-energie (HR-combi), elektrische input (warmtepomp/elektrische boiler)
//! of warmte-input aan gebouwgrens (stadsverwarming).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

use crate::model::EnergyCarrier;

/// Detail-uitsplitsing per keten-component voor traceability en rapportage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DhwBreakdown {
    /// Afgifte-rendement η_W;em dat gebruikt is.
    pub emission_efficiency: f64,

    /// Distributie-rendement η_W;dis dat gebruikt is.
    pub distribution_efficiency: f64,

    /// Opwekkings-rendement η_W;gen (of SCOP_W voor warmtepomp).
    pub generation_efficiency: f64,

    /// Product η_W;em × η_W;dis × η_W;gen (keten-rendement).
    ///
    /// Voor tapwater-warmtepompen kan dit > 1 zijn.
    pub total_efficiency: f64,

    /// DWTW netto rendement dat gebruikt is (0,0 als geen DWTW geïnstalleerd).
    pub recovery_efficiency: f64,

    /// C_W;nd;sh (douche-aandeel) dat gebruikt is bij DWTW.
    pub recovery_aandeel: f64,

    /// Maandprofiel van `Q_W;nd` (input, in MJ) — de netto vraag vóór DWTW-aftrek.
    pub monthly_q_w_nd: MonthlyProfile<Energy>,

    /// Maandprofiel van `Q_W;rcd` (teruggewonnen via DWTW, in MJ). Nul als
    /// geen recovery opgegeven.
    pub monthly_q_w_rcd: MonthlyProfile<Energy>,
}

/// Volledig resultaat van de warm-tapwater keten-berekening voor één systeem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DhwResult {
    /// Energiedrager waarin `monthly_q_w_use` en `annual_q_w_use` gedrukt zijn.
    ///
    /// - [`EnergyCarrier::Gas`] voor HR-combi-ketel
    /// - [`EnergyCarrier::Electricity`] voor warmtepomp / elektrische boiler
    /// - [`EnergyCarrier::DistrictHeat`] voor stadsverwarming
    pub energy_carrier: EnergyCarrier,

    /// Maandelijks eindenergiegebruik voor warm tapwater `Q_W;use;mi` in MJ.
    pub monthly_q_w_use: MonthlyProfile<Energy>,

    /// Jaarlijks eindenergiegebruik voor warm tapwater `Q_W;use;an` in MJ.
    pub annual_q_w_use: Energy,

    /// Detail-uitsplitsing (η-waarden + Q_W;nd / Q_W;rcd input).
    pub breakdown: DhwBreakdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    #[test]
    fn serde_round_trip() {
        let r = DhwResult {
            energy_carrier: EnergyCarrier::Gas,
            monthly_q_w_use: zero_profile(),
            annual_q_w_use: 0.0,
            breakdown: DhwBreakdown {
                emission_efficiency: 0.88,
                distribution_efficiency: 1.0,
                generation_efficiency: 0.80,
                total_efficiency: 0.88 * 1.0 * 0.80,
                recovery_efficiency: 0.0,
                recovery_aandeel: 0.0,
                monthly_q_w_nd: zero_profile(),
                monthly_q_w_rcd: zero_profile(),
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: DhwResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
