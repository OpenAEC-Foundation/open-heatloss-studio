//! PV-berekening uitvoer en resultaat-structuren.

use nta8800_model::time::MonthlyProfile;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// References removed from imports - they're used in doc comments only

/// Resultaat van een PV-opbrengst berekening conform NTA 8800 H.16.
///
/// Bevat de maandelijkse en jaarlijkse PV-opbrengst in MJ elektrisch,
/// alsmede een breakdown van de verschillende verlies-componenten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PvResult {
    /// Maandelijkse PV-opbrengst Q_PV;mi in MJ elektrisch.
    ///
    /// Nettolevering van het PV-systeem (na alle verliezen) conform
    /// formule [`NTA_8800_2025_FORMULE16_101`]:
    /// `Q_PV;mi = P_peak * I_sol;mi * η_total * t_maand / 1000`
    pub monthly_yield_mj: MonthlyProfile<f64>,

    /// Jaarlijkse PV-opbrengst Q_PV;jaar in MJ elektrisch.
    ///
    /// Som van alle 12 maandwaarden uit [`Self::monthly_yield_mj`].
    pub annual_yield_mj: f64,

    /// Maandelijkse inverter-verliezen in MJ.
    ///
    /// DC→AC omzetting-verliezen per maand, berekend als:
    /// `inverter_loss = dc_yield * (1 - η_inv)` waarbij `dc_yield` de
    /// opbrengst vóór de inverter is conform [`NTA_8800_2025_FORMULE16_103`].
    pub inverter_losses_mj: MonthlyProfile<f64>,

    /// Maandelijkse systeem-verliezen in MJ.
    ///
    /// DC-verliezen per maand (bekabeling, vervuiling, mismatch, etc.),
    /// berekend als: `system_loss = peak_yield * (1 - η_sys * shadow)`
    /// waarbij `peak_yield` de theoretische piek-opbrengst is conform
    /// [`NTA_8800_2025_FORMULE16_102`].
    pub system_losses_mj: MonthlyProfile<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pv_result() -> PvResult {
        PvResult {
            monthly_yield_mj: MonthlyProfile::new([
                1000.0, 1200.0, 1800.0, 2200.0, 2400.0, 2500.0, 2400.0, 2200.0, 1800.0, 1400.0,
                1000.0, 800.0,
            ]),
            annual_yield_mj: 20700.0,
            inverter_losses_mj: MonthlyProfile::new([
                50.0, 60.0, 90.0, 110.0, 120.0, 125.0, 120.0, 110.0, 90.0, 70.0, 50.0, 40.0,
            ]),
            system_losses_mj: MonthlyProfile::new([
                100.0, 120.0, 180.0, 220.0, 240.0, 250.0, 240.0, 220.0, 180.0, 140.0, 100.0, 80.0,
            ]),
        }
    }

    #[test]
    fn pv_result_constructor() {
        let result = sample_pv_result();
        assert_eq!(result.annual_yield_mj, 20700.0);
        assert_eq!(result.monthly_yield_mj.as_array().len(), 12); // MonthlyProfile is always 12 elements by design
    }

    #[test]
    fn monthly_profile_has_twelve_entries() {
        let result = sample_pv_result();
        assert_eq!(result.monthly_yield_mj.as_array().len(), 12); // MonthlyProfile is always 12 elements by design
        assert_eq!(result.inverter_losses_mj.as_array().len(), 12); // MonthlyProfile is always 12 elements by design
        assert_eq!(result.system_losses_mj.as_array().len(), 12); // MonthlyProfile is always 12 elements by design
    }

    #[test]
    fn annual_equals_sum_of_monthly() {
        let result = sample_pv_result();
        let calculated_annual: f64 = result.monthly_yield_mj.as_array().iter().sum();
        assert!((result.annual_yield_mj - calculated_annual).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_round_trip_json() {
        let original = sample_pv_result();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PvResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
