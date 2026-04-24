//! Klimaatdata — container voor maandelijkse buitentemperatuur, zoninstraling,
//! ventilatieve-koeltemperatuur, windsnelheid en WTW-preheat-temperatuur.
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
use crate::units::{SolarIrradiation, Temperature, WindSpeed};

/// Maandelijks klimaatprofiel voor één klimaatzone.
///
/// Alle velden zijn per maand gedefinieerd (12 waarden). De zoninstraling is
/// gegeven per oriëntatie in **MJ/m² cumulatief per maand** (NTA 8800
/// tabel 17.2). De temperatuur-, wind- en preheat-velden komen uit NTA 8800
/// tabel 17.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClimateData {
    /// Maandgemiddelde buitenluchttemperatuur `ϑ_e;avg;mi` in °C.
    ///
    /// NTA 8800 tabel 17.1 kolom 2. Overgenomen uit NEN 5060 referentieklimaat.
    pub outdoor_temperature: MonthlyProfile<Temperature>,

    /// Maandelijkse cumulatieve zoninstraling per oriëntatie in MJ/m².
    ///
    /// Sleutel is de [`Orientation`] variant; waarde is het maandprofiel.
    /// `BTreeMap` geeft een stabiele JSON-volgorde in geserialiseerde output.
    /// Afgeleid van NTA 8800 tabel 17.2 (`I_sol;mi` in W/m², geconverteerd
    /// naar MJ/m² per maand).
    pub solar_irradiation: BTreeMap<Orientation, MonthlyProfile<SolarIrradiation>>,

    /// Maandgemiddelde buitenluchttemperatuur voor ventilatieve koeling
    /// `ϑ_e;argII,mi` in °C, bepaald volgens §17.2.
    ///
    /// NTA 8800 tabel 17.1 kolom 3. Gemiddelde over uren waarbij
    /// 13 °C < `ϑ_e` < 24 °C (oktober–april alleen 22:00–06:00).
    /// `None` voor januari en december, omdat in die maanden het
    /// temperatuur-criterium niet wordt gehaald en de norm expliciet `-`
    /// rapporteert.
    pub cooling_reference_temperature: MonthlyProfile<Option<Temperature>>,

    /// Maandgemiddelde windsnelheid op locatie `u_site;mi` in m/s.
    ///
    /// NTA 8800 tabel 17.1 kolom 4. Wijkt licht af van `u_10` volgens
    /// NEN 5060 — uitschieters buiten één standaarddeviatie zijn uit het
    /// gemiddelde gefilterd (§17.2 opmerking 2). Gebruikt in het
    /// luchtstroommodel (§11.2).
    pub wind_speed: MonthlyProfile<WindSpeed>,

    /// Maandgemiddelde temperatuur van de toevoerlucht vóór de WTW tijdens
    /// koudeterugwinning `ϑ_ODA;preh;WTWC;zi;mi` in °C.
    ///
    /// NTA 8800 tabel 17.1 kolom 5. Waarde `0,00` in winter- en
    /// vroege-lente-maanden (jan–apr, okt–dec) omdat dan geen koudeterugwinning
    /// plaatsvindt; mei–sep bevatten de werkelijke preheat-waarden.
    pub wtw_preheat_temperature: MonthlyProfile<Temperature>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Month;

    fn sample_climate_data() -> ClimateData {
        let mut solar = BTreeMap::new();
        solar.insert(
            Orientation::Zuid,
            MonthlyProfile::new([
                50.0_f64, 80.0, 140.0, 200.0, 250.0, 270.0, 260.0, 230.0, 170.0, 110.0, 60.0, 40.0,
            ]),
        );
        ClimateData {
            outdoor_temperature: MonthlyProfile::new([
                3.0_f64, 3.5, 6.0, 9.0, 13.0, 16.0, 18.0, 17.5, 15.0, 11.0, 7.0, 4.0,
            ]),
            solar_irradiation: solar,
            cooling_reference_temperature: MonthlyProfile::new([
                None,
                Some(13.97),
                Some(13.00),
                Some(13.70),
                Some(16.42),
                Some(16.76),
                Some(17.51),
                Some(18.24),
                Some(16.74),
                Some(15.04),
                Some(13.43),
                None,
            ]),
            wind_speed: MonthlyProfile::new([
                3.04_f64, 4.15, 2.99, 3.06, 2.97, 2.78, 2.63, 2.51, 2.71, 2.78, 2.83, 2.83,
            ]),
            wtw_preheat_temperature: MonthlyProfile::new([
                0.00_f64, 0.00, 0.00, 0.00, 25.63, 27.49, 26.34, 27.29, 25.30, 0.00, 0.00, 0.00,
            ]),
        }
    }

    #[test]
    fn climate_data_serde_round_trip() {
        let data = sample_climate_data();

        let json = serde_json::to_string(&data).unwrap();
        let back: ClimateData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, back);
        assert!((back.outdoor_temperature[Month::Juli] - 18.0).abs() < 1e-9);
        assert!(
            (back.solar_irradiation.get(&Orientation::Zuid).unwrap()[Month::Juli] - 260.0).abs()
                < 1e-9
        );
        assert_eq!(back.cooling_reference_temperature[Month::Januari], None);
        assert!(
            (back.cooling_reference_temperature[Month::Juli].unwrap() - 17.51).abs() < 1e-9,
            "Juli ϑ_e;argII moet 17,51 °C zijn na round-trip"
        );
        assert!((back.wind_speed[Month::Februari] - 4.15).abs() < 1e-9);
        assert!((back.wtw_preheat_temperature[Month::Juni] - 27.49).abs() < 1e-9);
        assert!(
            back.wtw_preheat_temperature[Month::Januari].abs() < 1e-9,
            "WTW-preheat is 0 °C in januari (geen koudeterugwinning)"
        );
    }

    #[test]
    fn climate_data_empty_solar_is_valid() {
        let data = ClimateData {
            outdoor_temperature: MonthlyProfile::from_constant(10.0_f64),
            solar_irradiation: BTreeMap::new(),
            cooling_reference_temperature: MonthlyProfile::from_constant(Some(15.0_f64)),
            wind_speed: MonthlyProfile::from_constant(3.0_f64),
            wtw_preheat_temperature: MonthlyProfile::from_constant(0.0_f64),
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: ClimateData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn cooling_reference_temperature_heeft_twaalf_entries() {
        let data = sample_climate_data();
        let count = Month::all()
            .into_iter()
            .filter(|m| {
                // Gewoon alle 12 maanden indexeren; elke maand geeft een
                // geldige (None of Some) waarde terug.
                let _ = data.cooling_reference_temperature[*m];
                true
            })
            .count();
        assert_eq!(count, 12);
    }

    #[test]
    fn wind_speed_heeft_twaalf_entries() {
        let data = sample_climate_data();
        let mut count = 0;
        for month in Month::all() {
            assert!(data.wind_speed[month].is_finite());
            count += 1;
        }
        assert_eq!(count, 12);
    }

    #[test]
    fn wtw_preheat_temperature_heeft_twaalf_entries() {
        let data = sample_climate_data();
        let mut count = 0;
        for month in Month::all() {
            assert!(data.wtw_preheat_temperature[month].is_finite());
            count += 1;
        }
        assert_eq!(count, 12);
    }
}
