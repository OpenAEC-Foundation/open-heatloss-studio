//! Resultaat van humidity berekening per zone.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

/// Resultaat van humidity berekening voor één [`nta8800_model::Rekenzone`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HumidityResult {
    /// Jaarlijkse bevochtigingsbehoefte in MJ thermisch.
    pub annual_q_hum: Energy,

    /// Jaarlijkse ontvochtigingsbehoefte in MJ thermisch.
    pub annual_q_dhum: Energy,

    /// Jaarlijks elektrisch energiegebruik humidification systemen in MJ.
    pub annual_w_hum: Energy,

    /// Maandelijkse bevochtigingsbehoefte in MJ thermisch.
    pub monthly_humidification: MonthlyProfile<Energy>,

    /// Maandelijkse ontvochtigingsbehoefte in MJ thermisch.
    pub monthly_dehumidification: MonthlyProfile<Energy>,

    /// Maandelijks elektrisch energiegebruik humidification systemen in MJ.
    pub monthly_electrical: MonthlyProfile<Energy>,
}

impl HumidityResult {
    /// Creëert een nieuw `HumidityResult` met jaarwaarden berekend uit maandwaarden.
    #[must_use]
    pub fn new(
        monthly_humidification: MonthlyProfile<Energy>,
        monthly_dehumidification: MonthlyProfile<Energy>,
        monthly_electrical: MonthlyProfile<Energy>,
    ) -> Self {
        Self {
            annual_q_hum: monthly_humidification.as_array().iter().sum(),
            annual_q_dhum: monthly_dehumidification.as_array().iter().sum(),
            annual_w_hum: monthly_electrical.as_array().iter().sum(),
            monthly_humidification,
            monthly_dehumidification,
            monthly_electrical,
        }
    }

    /// Totaal jaarlijks energiegebruik (thermisch + elektrisch) in MJ.
    #[must_use]
    pub fn annual_total_energy(&self) -> Energy {
        self.annual_q_hum + self.annual_q_dhum + self.annual_w_hum
    }

    /// Totaal maandelijks energiegebruik (thermisch + elektrisch) in MJ.
    #[must_use]
    pub fn monthly_total_energy(&self) -> MonthlyProfile<Energy> {
        let mut monthly_total = [0.0; 12];
        for (i, monthly_val) in monthly_total.iter_mut().enumerate() {
            *monthly_val = self.monthly_humidification.as_array()[i]
                + self.monthly_dehumidification.as_array()[i]
                + self.monthly_electrical.as_array()[i];
        }
        MonthlyProfile::new(monthly_total)
    }

    /// Controleert of er überhaupt humidity activiteit is (alle waarden > 0.001 MJ).
    #[must_use]
    pub fn has_humidity_activity(&self) -> bool {
        self.annual_total_energy() > 0.001
    }

    /// Geeft de dominante activiteit: "humidification", "dehumidification", of "none".
    #[must_use]
    pub fn dominant_activity(&self) -> &'static str {
        if self.annual_q_hum > self.annual_q_dhum && self.annual_q_hum > 0.001 {
            "humidification"
        } else if self.annual_q_dhum > 0.001 {
            "dehumidification"
        } else {
            "none"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn humidity_result_creation_and_totals() {
        let humidification = MonthlyProfile::new([1.0; 12]); // 12 MJ/jaar
        let dehumidification = MonthlyProfile::new([0.5; 12]); // 6 MJ/jaar
        let electrical = MonthlyProfile::new([0.2; 12]); // 2.4 MJ/jaar

        let result = HumidityResult::new(humidification, dehumidification, electrical);

        assert_abs_diff_eq!(result.annual_q_hum, 12.0, epsilon = 1e-9);
        assert_abs_diff_eq!(result.annual_q_dhum, 6.0, epsilon = 1e-9);
        assert_abs_diff_eq!(result.annual_w_hum, 2.4, epsilon = 1e-9);
        assert_abs_diff_eq!(result.annual_total_energy(), 20.4, epsilon = 1e-9);
    }

    #[test]
    fn monthly_total_energy() {
        let humidification = MonthlyProfile::new([2.0; 12]);
        let dehumidification = MonthlyProfile::new([1.0; 12]);
        let electrical = MonthlyProfile::new([0.5; 12]);

        let result = HumidityResult::new(humidification, dehumidification, electrical);
        let monthly_total = result.monthly_total_energy();

        for month_total in monthly_total.as_array() {
            assert_abs_diff_eq!(*month_total, 3.5, epsilon = 1e-9); // 2.0 + 1.0 + 0.5
        }
    }

    #[test]
    fn has_humidity_activity() {
        let zero_monthly = MonthlyProfile::new([0.0; 12]);
        let small_monthly = MonthlyProfile::new([0.0001; 12]);
        let significant_monthly = MonthlyProfile::new([0.1; 12]);

        let no_activity = HumidityResult::new(zero_monthly.clone(), zero_monthly.clone(), zero_monthly.clone());
        assert!(!no_activity.has_humidity_activity());

        let minimal_activity = HumidityResult::new(small_monthly, zero_monthly.clone(), zero_monthly.clone());
        assert!(minimal_activity.has_humidity_activity()); // 0.0001 * 12 = 0.0012, dat is > 0.001

        let clear_activity = HumidityResult::new(significant_monthly, zero_monthly.clone(), zero_monthly);
        assert!(clear_activity.has_humidity_activity()); // 0.1 * 12 = 1.2 > 0.001
    }

    #[test]
    fn dominant_activity() {
        let zero_monthly = MonthlyProfile::new([0.0; 12]);
        let humidification_monthly = MonthlyProfile::new([1.0; 12]);
        let dehumidification_monthly = MonthlyProfile::new([0.5; 12]);

        // Humidification dominant
        let hum_dominant = HumidityResult::new(humidification_monthly.clone(), dehumidification_monthly.clone(), zero_monthly.clone());
        assert_eq!(hum_dominant.dominant_activity(), "humidification");

        // Dehumidification dominant
        let dehumidification_large = MonthlyProfile::new([2.0; 12]);
        let dhum_dominant = HumidityResult::new(humidification_monthly, dehumidification_large, zero_monthly.clone());
        assert_eq!(dhum_dominant.dominant_activity(), "dehumidification");

        // No activity
        let no_activity = HumidityResult::new(zero_monthly.clone(), zero_monthly.clone(), zero_monthly);
        assert_eq!(no_activity.dominant_activity(), "none");
    }
}