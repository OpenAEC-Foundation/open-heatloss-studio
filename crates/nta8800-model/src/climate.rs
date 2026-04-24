//! Klimaatdata — container voor maandelijkse buitentemperatuur en zoninstraling.
//!
//! Dit module definieert enkel de **container-types** voor klimaatdata. De
//! feitelijke getallen (KNMI-referentie De Bilt, bijlage E van NTA 8800)
//! komen straks uit de `nta8800-tables` crate. Model bevat geen hardcoded
//! klimaat-constanten.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::location::Orientation;
use crate::time::MonthlyProfile;
use crate::units::{SolarIrradiation, Temperature};

/// Maandelijks klimaatprofiel voor één klimaatzone.
///
/// `outdoor_temperature` en `solar_irradiation` zijn beide per maand
/// gedefinieerd (12 waarden). De zoninstraling is gegeven per oriëntatie
/// in **MJ/m² cumulatief per maand** (NTA 8800 bijlage E).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClimateData {
    /// Maandelijkse gemiddelde buitentemperatuur `θ_e` in °C.
    pub outdoor_temperature: MonthlyProfile<Temperature>,

    /// Maandelijkse cumulatieve zoninstraling per oriëntatie in MJ/m².
    ///
    /// Sleutel is de [`Orientation`] variant; waarde is het maandprofiel.
    /// `BTreeMap` geeft een stabiele JSON-volgorde in geserialiseerde output.
    pub solar_irradiation: BTreeMap<Orientation, MonthlyProfile<SolarIrradiation>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Month;

    #[test]
    fn climate_data_serde_round_trip() {
        let mut solar = BTreeMap::new();
        solar.insert(
            Orientation::Zuid,
            MonthlyProfile::new([
                50.0_f64, 80.0, 140.0, 200.0, 250.0, 270.0, 260.0, 230.0, 170.0, 110.0, 60.0, 40.0,
            ]),
        );
        let data = ClimateData {
            outdoor_temperature: MonthlyProfile::new([
                3.0_f64, 3.5, 6.0, 9.0, 13.0, 16.0, 18.0, 17.5, 15.0, 11.0, 7.0, 4.0,
            ]),
            solar_irradiation: solar,
        };

        let json = serde_json::to_string(&data).unwrap();
        let back: ClimateData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, back);
        assert!((back.outdoor_temperature[Month::Juli] - 18.0).abs() < 1e-9);
        assert!(
            (back.solar_irradiation.get(&Orientation::Zuid).unwrap()[Month::Juli] - 260.0).abs()
                < 1e-9
        );
    }

    #[test]
    fn climate_data_empty_solar_is_valid() {
        let data = ClimateData {
            outdoor_temperature: MonthlyProfile::from_constant(10.0_f64),
            solar_irradiation: BTreeMap::new(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: ClimateData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, back);
    }
}
