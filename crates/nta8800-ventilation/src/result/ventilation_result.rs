//! [`VentilationResult`] — complete maandelijkse én jaarlijkse uitkomst.

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Rekenresultaat voor één rekenzone — één jaar, één run van
/// [`crate::calculate_ventilation`].
///
/// Alle energie-velden in **MJ** conform de workspace-conventie (zie
/// [`nta8800_model::units`] — conversies naar kWh pas op eindresultaat-niveau
/// met factor 3,6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VentilationResult {
    /// Maandelijkse ventilatie-warmteverlies Q_V;mi in MJ.
    ///
    /// Positief = warmteverlies door ventilatie. Nul in zomer-maanden waar
    /// ϑ_toevoer ≥ ϑ_binnen.
    pub monthly_q_v: MonthlyProfile<Energy>,

    /// Jaarlijkse totaal ventilatie-warmteverlies Q_V;an in MJ.
    pub annual_q_v: Energy,

    /// Maandelijkse ventilator-energie W_fan;mi in MJ elektrisch.
    ///
    /// Primair energiegebruik, niet gecorrigeerd voor primaire-energiefactor
    /// (die wordt in `nta8800-ep` toegepast).
    pub monthly_w_fan: MonthlyProfile<Energy>,

    /// Jaarlijkse totaal ventilator-energie W_fan;an in MJ elektrisch.
    pub annual_w_fan: Energy,

    /// Maandelijkse WTW-warmteterugwinning Q_WTW;mi in MJ.
    ///
    /// 0 voor systemen zonder WTW. Geeft de "gratis" warmte die niet hoeft
    /// te worden opgewekt door de verwarmingsinstallatie.
    pub monthly_wtw_recovery: MonthlyProfile<Energy>,

    /// Jaarlijkse WTW-warmteterugwinning Q_WTW;an in MJ.
    pub annual_wtw_recovery: Energy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::{Month, MonthlyProfile};

    fn sample() -> VentilationResult {
        VentilationResult {
            monthly_q_v: MonthlyProfile::new([
                2300.0, 1900.0, 1700.0, 1200.0, 700.0, 300.0, 100.0, 150.0, 500.0, 1100.0, 1700.0,
                2100.0,
            ]),
            annual_q_v: 13_750.0,
            monthly_w_fan: MonthlyProfile::from_constant(100.0),
            annual_w_fan: 1200.0,
            monthly_wtw_recovery: MonthlyProfile::from_constant(0.0),
            annual_wtw_recovery: 0.0,
        }
    }

    #[test]
    fn serde_round_trip() {
        let r = sample();
        let json = serde_json::to_string(&r).unwrap();
        let back: VentilationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn january_is_largest_q_v() {
        let r = sample();
        let jan = r.monthly_q_v[Month::Januari];
        let jul = r.monthly_q_v[Month::Juli];
        assert!(jan > jul);
    }
}
