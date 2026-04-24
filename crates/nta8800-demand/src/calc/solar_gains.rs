//! Zoninstraling door transparante gebouwelementen (ramen).
//!
//! NTA 8800 §7.9, formule (7.33):
//!
//! ```text
//! Q_sol;wi;mi = A_w · g · F_sh · (1 − F_F) · I_sol;or;mi
//! ```
//!
//! met:
//! - `A_w` — bruto vensteroppervlak [m²]
//! - `g` — zonnewarmtedoorlatingsfactor [-]
//! - `F_sh` — afschermingsfactor (extern/intern samengevat); V1 default 1,0
//! - `F_F` — kozijnfractie (opaak aandeel) [-]
//! - `I_sol;or;mi` — cumulatieve maand-zoninstraling per oriëntatie [MJ/m²]
//!
//! De invoer [`nta8800_model::ClimateData::solar_irradiation`] is reeds in
//! MJ/m² per maand geserialiseerd — conversie is niet nodig.

use nta8800_model::geometry::Window;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::ClimateData;

/// Bereken de maandelijkse totale zoninstraling door alle ramen in MJ.
///
/// # Parameters
///
/// - `windows` — ramen in de rekenzone. Ontbrekende oriëntaties in
///   `climate.solar_irradiation` geven 0 bijdrage (geen error).
/// - `climate` — klimaatdata met per-oriëntatie `I_sol` profielen.
/// - `shading_factor` — `F_sh` (0..=1), afschermingsfactor. V1-default 1,0
///   wanneer de caller geen schaduwmodel heeft.
///
/// # Formule
///
/// Per raam, per maand:
///
/// ```text
/// Q = A_w · g · F_sh · (1 − F_F) · I_sol(or, mi)
/// ```
///
/// De bijdragen worden per maand over alle ramen gesommeerd.
#[must_use]
pub fn monthly_solar_gains(
    windows: &[&Window],
    climate: &ClimateData,
    shading_factor: f64,
) -> MonthlyProfile<Energy> {
    let mut monthly = [0.0_f64; 12];
    for window in windows {
        // Opake kozijnfractie uitsluiten van g-werkend oppervlak.
        let frame_transmitting = (1.0 - window.frame_fraction).max(0.0);
        let effective_area = window.area * window.g_value * shading_factor * frame_transmitting;
        if effective_area <= 0.0 {
            continue;
        }
        let Some(profile) = climate.solar_irradiation.get(&window.orientation) else {
            continue;
        };
        for month in Month::all() {
            let i_sol = profile[month];
            monthly[month.index()] += effective_area * i_sol;
        }
    }
    MonthlyProfile::new(monthly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::location::{Orientation, Tilt};
    use std::collections::BTreeMap;

    fn zuid_window() -> Window {
        Window::new(
            "w-zuid",
            "c1",
            5.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.1,
            0.6,
            0.2,
        )
        .unwrap()
    }

    fn noord_window() -> Window {
        Window::new(
            "w-noord",
            "c1",
            5.0,
            Orientation::Noord,
            Tilt::VERTICAL,
            1.1,
            0.6,
            0.2,
        )
        .unwrap()
    }

    fn sample_climate() -> ClimateData {
        let mut solar = BTreeMap::new();
        // Zuid-zon: juni piek, december minimum
        solar.insert(
            Orientation::Zuid,
            MonthlyProfile::new([
                60.0_f64, 100.0, 180.0, 240.0, 290.0, 310.0, 300.0, 270.0, 210.0, 140.0, 80.0, 50.0,
            ]),
        );
        solar.insert(
            Orientation::Noord,
            MonthlyProfile::new([
                20.0_f64, 35.0, 55.0, 75.0, 95.0, 105.0, 100.0, 90.0, 70.0, 45.0, 25.0, 15.0,
            ]),
        );
        ClimateData {
            outdoor_temperature: MonthlyProfile::from_constant(10.0),
            solar_irradiation: solar,
            cooling_reference_temperature: MonthlyProfile::from_constant(Some(16.0)),
            wind_speed: MonthlyProfile::from_constant(3.0),
            wtw_preheat_temperature: MonthlyProfile::from_constant(0.0),
        }
    }

    #[test]
    fn zuid_juni_meer_dan_noord_juni() {
        let climate = sample_climate();
        let zuid = zuid_window();
        let noord = noord_window();
        let q_zuid = monthly_solar_gains(&[&zuid], &climate, 1.0);
        let q_noord = monthly_solar_gains(&[&noord], &climate, 1.0);
        assert!(
            q_zuid[Month::Juni] > 2.0 * q_noord[Month::Juni],
            "zuid={}, noord={}",
            q_zuid[Month::Juni],
            q_noord[Month::Juni]
        );
    }

    #[test]
    fn zuid_juni_hoger_dan_zuid_december() {
        let climate = sample_climate();
        let w = zuid_window();
        let q = monthly_solar_gains(&[&w], &climate, 1.0);
        assert!(q[Month::Juni] > q[Month::December]);
    }

    #[test]
    fn shading_halveert_resultaat() {
        let climate = sample_climate();
        let w = zuid_window();
        let full = monthly_solar_gains(&[&w], &climate, 1.0);
        let half = monthly_solar_gains(&[&w], &climate, 0.5);
        for month in Month::all() {
            assert!((half[month] - 0.5 * full[month]).abs() < 1e-6);
        }
    }

    #[test]
    fn lege_ramen_lijst_geeft_nul_profiel() {
        let climate = sample_climate();
        let q = monthly_solar_gains(&[], &climate, 1.0);
        for month in Month::all() {
            assert!(q[month].abs() < 1e-12);
        }
    }

    #[test]
    fn ontbrekende_orientatie_in_climate_geeft_nul() {
        let w = Window::new(
            "w",
            "c",
            5.0,
            Orientation::Oost,
            Tilt::VERTICAL,
            1.1,
            0.6,
            0.2,
        )
        .unwrap();
        let climate = sample_climate(); // alleen Zuid + Noord
        let q = monthly_solar_gains(&[&w], &climate, 1.0);
        for month in Month::all() {
            assert!(q[month].abs() < 1e-12);
        }
    }

    #[test]
    fn frame_fraction_1_geeft_nul() {
        let w = Window::new(
            "w",
            "c",
            5.0,
            Orientation::Zuid,
            Tilt::VERTICAL,
            1.1,
            0.6,
            1.0, // 100% kozijn → 0 glas
        )
        .unwrap();
        let climate = sample_climate();
        let q = monthly_solar_gains(&[&w], &climate, 1.0);
        for month in Month::all() {
            assert!(q[month].abs() < 1e-12);
        }
    }

    #[test]
    fn numerieke_sanity_check_zuid() {
        // Ruim rekenen: A=5, g=0.6, F_sh=1, (1-F_F)=0.8, I_juni=310 MJ/m²
        // Q_juni = 5 × 0.6 × 1 × 0.8 × 310 = 744 MJ
        let climate = sample_climate();
        let w = zuid_window();
        let q = monthly_solar_gains(&[&w], &climate, 1.0);
        assert!((q[Month::Juni] - 744.0).abs() < 1e-6);
    }
}
