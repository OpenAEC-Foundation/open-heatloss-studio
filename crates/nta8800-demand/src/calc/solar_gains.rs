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
//! - `F_sh` — globale afschermingsfactor (whole-zone override); default 1,0
//! - `F_F` — kozijnfractie (opaak aandeel) [-]
//! - `I_sol;or;mi` — cumulatieve maand-zoninstraling per oriëntatie [MJ/m²]
//!
//! **Balans-gescheiden zonwinst (H/C).** De zonwinst wordt per balanstak
//! ([`SolarBalance`]) bepaald, omdat de NTA 8800 beide beschaduwingsmechanismen
//! asymmetrisch behandelt:
//!
//! - **Beweegbare zonwering** ([`Window::movable_shading`], §7.6.6.1.4,
//!   formule 7.42/7.43): bij [`SolarBalance::Heating`] geldt voor de
//!   woning-warmtevraag `f_sh;with = 0` (lid 1) → geen g-reductie; bij
//!   [`SolarBalance::Cooling`] het volledige maandprofiel (tabel 7.7/7.9).
//! - **Externe belemmering** ([`Window::obstruction`], §17.3, factor
//!   `F_sh;obst;mi`): tabel 17.4 op de warmtebalans (winterreductie), tabel 17.5
//!   op de koudebalans (bij minimale belemmering uniform 1,00).
//!
//! Beide factoren vermenigvuldigen met de globale `shading_factor` (grove
//! whole-zone override). Zonder `movable_shading` én zonder `obstruction` zijn
//! beide per-raam factoren 1,0 in elke maand → byte-identiek aan het gedrag
//! vóór deze uitbreiding, ongeacht de balanstak.
//!
//! De invoer [`nta8800_model::ClimateData::solar_irradiation`] is reeds in
//! MJ/m² per maand geserialiseerd — conversie is niet nodig.

use nta8800_model::geometry::Window;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::ClimateData;

use crate::calc::shading::{movable_shading_g_factor, obstruction_g_factor, SolarBalance};

/// Bereken de maandelijkse totale zoninstraling door alle ramen in MJ.
///
/// # Parameters
///
/// - `windows` — ramen in de rekenzone. Ontbrekende oriëntaties in
///   `climate.solar_irradiation` geven 0 bijdrage (geen error).
/// - `climate` — klimaatdata met per-oriëntatie `I_sol` profielen.
/// - `shading_factor` — `F_sh` (0..=1), afschermingsfactor. V1-default 1,0
///   wanneer de caller geen schaduwmodel heeft.
/// - `balance` — [`SolarBalance::Heating`] of [`SolarBalance::Cooling`]; bepaalt
///   de balans-asymmetrische beschaduwingsfactoren (zie module-doc).
///
/// # Formule
///
/// Per raam, per maand:
///
/// ```text
/// Q = A_w · g · F_sh · r_sh;mv;mi · F_sh;obst;mi · (1 − F_F) · I_sol(or, mi)
/// ```
///
/// met `r_sh;mv;mi` de beweegbare-zonwering-reductie (1,0 op de warmtebalans) en
/// `F_sh;obst;mi` de §17.3-belemmering (1,0 op de koudebalans bij minimale
/// belemmering). De bijdragen worden per maand over alle ramen gesommeerd.
#[must_use]
pub fn monthly_solar_gains(
    windows: &[&Window],
    climate: &ClimateData,
    shading_factor: f64,
    balance: SolarBalance,
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
        // Beweegbare zonwering (NTA 8800 §7.6.6.1.4). Op de warmtebalans geldt
        // voor woningen f_sh;with = 0 → geen g-reductie (factor 1,0); op de
        // koudebalans het maand-afhankelijke profiel. Zonder zonwering: 1,0.
        let movable = match balance {
            SolarBalance::Heating => None,
            SolarBalance::Cooling => window.movable_shading.map(|s| {
                movable_shading_g_factor(s.f_c, window.orientation, window.tilt, s.control)
            }),
        };
        // Externe belemmering (§17.3) — 1,0 in elke maand bij Obstruction::None
        // of op de koudebalans (tabel 17.5 uniform 1,00).
        let obstruction =
            obstruction_g_factor(window.obstruction, window.orientation, window.tilt, balance);
        for month in Month::all() {
            let i_sol = profile[month];
            let m_factor = movable.as_ref().map_or(1.0, |p| p[month]);
            monthly[month.index()] += effective_area * m_factor * obstruction[month] * i_sol;
        }
    }
    MonthlyProfile::new(monthly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::location::{Orientation, Tilt};
    use nta8800_model::Obstruction;
    use std::collections::BTreeMap;

    // De meeste bestaande tests bekijken de koudebalans (waar beweegbare
    // zonwering werkt); een alias houdt de call-sites leesbaar.
    const C: SolarBalance = SolarBalance::Cooling;

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
        let q_zuid = monthly_solar_gains(&[&zuid], &climate, 1.0, C);
        let q_noord = monthly_solar_gains(&[&noord], &climate, 1.0, C);
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
        let q = monthly_solar_gains(&[&w], &climate, 1.0, C);
        assert!(q[Month::Juni] > q[Month::December]);
    }

    #[test]
    fn shading_halveert_resultaat() {
        let climate = sample_climate();
        let w = zuid_window();
        let full = monthly_solar_gains(&[&w], &climate, 1.0, C);
        let half = monthly_solar_gains(&[&w], &climate, 0.5, C);
        for month in Month::all() {
            assert!((half[month] - 0.5 * full[month]).abs() < 1e-6);
        }
    }

    #[test]
    fn lege_ramen_lijst_geeft_nul_profiel() {
        let climate = sample_climate();
        let q = monthly_solar_gains(&[], &climate, 1.0, C);
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
        let q = monthly_solar_gains(&[&w], &climate, 1.0, C);
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
        let q = monthly_solar_gains(&[&w], &climate, 1.0, C);
        for month in Month::all() {
            assert!(q[month].abs() < 1e-12);
        }
    }

    #[test]
    fn beweegbare_zonwering_reduceert_zomer_niet_winter() {
        use nta8800_model::geometry::{MovableSunShading, ShadingControl};
        let climate = sample_climate();
        let plain = zuid_window();
        let shaded = zuid_window().with_movable_shading(MovableSunShading {
            f_c: 0.2,
            control: ShadingControl::ManualResidential,
        });
        let q_plain = monthly_solar_gains(&[&plain], &climate, 1.0, C);
        let q_shaded = monthly_solar_gains(&[&shaded], &climate, 1.0, C);
        // Juli: handbediend Zuid f_sh;with = 0,59 → reductie tot factor 0,528.
        assert!(q_shaded[Month::Juli] < q_plain[Month::Juli]);
        assert!((q_shaded[Month::Juli] - 0.528 * q_plain[Month::Juli]).abs() < 1e-6);
        // Januari: f_sh;with = 0 → geen reductie.
        assert!((q_shaded[Month::Januari] - q_plain[Month::Januari]).abs() < 1e-9);
    }

    #[test]
    fn geen_zonwering_is_identiek_aan_voorheen() {
        // Regressie-pin: een raam zonder movable_shading én zonder obstruction
        // levert exact de kale A·g·(1−F_F)·I_sol-berekening — voor BEIDE
        // balanstakken identiek (default-gedrag ongewijzigd).
        let climate = sample_climate();
        let w = zuid_window();
        let base = w.area * w.g_value * (1.0 - w.frame_fraction);
        for balance in [SolarBalance::Heating, SolarBalance::Cooling] {
            let q = monthly_solar_gains(&[&w], &climate, 1.0, balance);
            for month in Month::all() {
                let expected = base * climate.solar_irradiation[&Orientation::Zuid][month];
                assert!((q[month] - expected).abs() < 1e-9, "{balance:?} {month:?}");
            }
        }
    }

    #[test]
    fn warmtebalans_negeert_beweegbare_zonwering() {
        // §7.6.6.1.4 lid 1: f_sh;with = 0 voor de woning-warmtevraag → beweegbare
        // zonwering raakt de Q_H-zonwinst NIET (identiek aan het kale raam).
        use nta8800_model::geometry::{MovableSunShading, ShadingControl};
        let climate = sample_climate();
        let plain = zuid_window();
        let shaded = zuid_window().with_movable_shading(MovableSunShading {
            f_c: 0.2,
            control: ShadingControl::ManualResidential,
        });
        let q_plain = monthly_solar_gains(&[&plain], &climate, 1.0, SolarBalance::Heating);
        let q_shaded = monthly_solar_gains(&[&shaded], &climate, 1.0, SolarBalance::Heating);
        for month in Month::all() {
            assert!((q_shaded[month] - q_plain[month]).abs() < 1e-12, "{month:?}");
        }
    }

    #[test]
    fn belemmering_verlaagt_warmtebalans_niet_koudebalans() {
        // §17.3: tabel 17.4 (H) reduceert de winter-zonwinst van een zuidraam;
        // tabel 17.5 (C) is uniform 1,00 → koudebalans onaangetast.
        let climate = sample_climate();
        let plain = zuid_window();
        let obstructed = zuid_window().with_obstruction(Obstruction::Minimal);

        let heat_plain = monthly_solar_gains(&[&plain], &climate, 1.0, SolarBalance::Heating);
        let heat_obstructed =
            monthly_solar_gains(&[&obstructed], &climate, 1.0, SolarBalance::Heating);
        // Januari (Zuid vert.): F_sh;obst = 0,23 → forse reductie van de warmtewinst.
        assert!(heat_obstructed[Month::Januari] < heat_plain[Month::Januari]);
        assert!((heat_obstructed[Month::Januari] - 0.23 * heat_plain[Month::Januari]).abs() < 1e-9);

        let cool_plain = monthly_solar_gains(&[&plain], &climate, 1.0, SolarBalance::Cooling);
        let cool_obstructed =
            monthly_solar_gains(&[&obstructed], &climate, 1.0, SolarBalance::Cooling);
        for month in Month::all() {
            assert!(
                (cool_obstructed[month] - cool_plain[month]).abs() < 1e-12,
                "koeling {month:?}"
            );
        }
    }

    #[test]
    fn numerieke_sanity_check_zuid() {
        // Ruim rekenen: A=5, g=0.6, F_sh=1, (1-F_F)=0.8, I_juni=310 MJ/m²
        // Q_juni = 5 × 0.6 × 1 × 0.8 × 310 = 744 MJ
        let climate = sample_climate();
        let w = zuid_window();
        let q = monthly_solar_gains(&[&w], &climate, 1.0, C);
        assert!((q[Month::Juni] - 744.0).abs() < 1e-6);
    }
}
