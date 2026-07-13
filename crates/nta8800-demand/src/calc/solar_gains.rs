//! Zoninstraling door transparante gebouwelementen (ramen).
//!
//! NTA 8800 §7.6.3, formule (7.32):
//!
//! ```text
//! Q_sol;wi;mi = g_gl · A_w · F_sh · (1 − F_F) · F_sh;obst · I_sol;or;mi − Q_sky;wi;mi
//! ```
//!
//! met:
//! - `A_w` — bruto vensteroppervlak [m²]
//! - `g_gl = F_w · g_gl;n` — effectieve zontoetredingsfactor (formule 7.40, met
//!   `F_w = 0,90` invalshoek-correctie op de loodrechte `g_gl;n`) [-]
//! - `F_sh` — globale afschermingsfactor (whole-zone override); default 1,0
//! - `F_F` — kozijnfractie (opaak aandeel) [-]
//! - `I_sol;or;mi` — cumulatieve maand-zoninstraling per oriëntatie [MJ/m²]
//! - `Q_sky;wi;mi` — langgolvige hemelstraling (formule 7.39, §7.6.5): een
//!   warmteverlies naar de koude hemel dat van de bruto-zonwinst wordt
//!   afgetrokken. `Q_sky = F_sky · R_se · U · A · h_lr;e · Δθ_sky · t_mi` met
//!   `h_lr;e = 4,14 W/(m²K)`, `Δθ_sky = 11 K`, `F_sky` uit §7.6.6.4.
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
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;
use nta8800_model::ClimateData;

use crate::calc::internal_gains::{MONTH_HOURS, WH_TO_MJ};
use crate::calc::shading::{movable_shading_g_factor, obstruction_g_factor, SolarBalance};

/// Correctiefactor voor niet-verstrooiende beglazing `F_w` (NTA 8800 §7.6.6.1.2,
/// formule 7.40): de tijdgewogen effectieve zontoetredingsfactor is lager dan de
/// loodrechte `g_gl;n` omdat de meeste straling schuin invalt. Getalswaarde 0,90.
/// Werkt op de warmte- én koudebalans (invalshoek-correctie, balans-onafhankelijk).
pub const F_W_GLAZING: f64 = 0.90;

/// Warmteovergangsweerstand buitenzijde `R_se` (NTA 8800 bijlage C.2) voor de
/// hemelstralingsterm (formule 7.39), in m²·K/W.
pub const R_SE: f64 = 0.04;

/// Warmteoverdrachtcoëfficiënt langgolvige straling buitenzijde `h_lr;e`
/// (NTA 8800 §7.6.5, formule 7.39), in W/(m²·K).
pub const H_LR_E: f64 = 4.14;

/// Gemiddeld verschil schijnbare hemeltemperatuur − buitentemperatuur
/// `Δθ_sky` (NTA 8800 §7.6.5, formule 7.39), in K.
pub const DELTA_THETA_SKY_K: f64 = 11.0;

/// Absorptiecoëfficiënt voor zonnestraling `α_sol` (NTA 8800 §7.6.6.3): het
/// buitenoppervlak van elke niet-transparante constructie krijgt de forfaitaire
/// waarde 0,60. Gebruikt in de opake-zonwinst (formule 7.33). Balans-onafhankelijk.
pub const ALPHA_SOL: f64 = 0.6;

/// Vormfactor `F_sky` tussen constructie en hemel (NTA 8800 §7.6.6.4) o.b.v. de
/// hellingshoek met de horizontaal:
/// - ≤ 5° (horizontaal): 1,0
/// - > 5° en ≤ 75° (hellend): 0,75
/// - > 75° (verticaal): 0,5
///
/// De norm-categorie "overhellend (naar de grond gericht) → 0" wordt niet
/// geëvalueerd: ramen in dit model hebben een helling in `[0°, 90°]`.
#[must_use]
fn sky_view_factor(tilt_deg: f64) -> f64 {
    if tilt_deg <= 5.0 {
        1.0
    } else if tilt_deg <= 75.0 {
        0.75
    } else {
        0.5
    }
}

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
/// Per raam, per maand (formule 7.32):
///
/// ```text
/// Q = A_w · g_gl;n · F_w · F_sh · r_sh;mv;mi · F_sh;obst;mi · (1 − F_F) · I_sol(or, mi)
///     − Q_sky;mi
/// ```
///
/// met `F_w = 0,90` (formule 7.40, invalshoek-correctie), `r_sh;mv;mi` de
/// beweegbare-zonwering-reductie (1,0 op de warmtebalans), `F_sh;obst;mi` de
/// §17.3-belemmering (1,0 op de koudebalans bij minimale belemmering) en
/// `Q_sky;mi` de hemelstraling (formule 7.39). De bijdragen worden per maand over
/// alle ramen gesommeerd; `Q_sky` wordt ook afgetrokken bij ontbrekende
/// oriëntatie in het klimaatprofiel (het hangt niet van `I_sol` af).
#[must_use]
pub fn monthly_solar_gains(
    windows: &[&Window],
    climate: &ClimateData,
    shading_factor: f64,
    balance: SolarBalance,
) -> MonthlyProfile<Energy> {
    let mut monthly = [0.0_f64; 12];
    for window in windows {
        // Hemelstralingsterm Q_sky (formule 7.39): een langgolvig warmteverlies
        // naar de koude hemel dat óók bij nul-g (geen zoninstraling-oriëntatie)
        // van de zonwinst wordt afgetrokken. Onafhankelijk van g en balanstak;
        // gebruikt de volle (kozijn+glas) oppervlakte en samengestelde U.
        let f_sky = sky_view_factor(window.tilt.degrees);
        let q_sky_power_w =
            f_sky * R_SE * window.u_value * window.area * H_LR_E * DELTA_THETA_SKY_K;

        // Opake kozijnfractie uitsluiten van g-werkend oppervlak. `F_w` (formule
        // 7.40) corrigeert de loodrechte g_gl;n naar de tijdgewogen effectieve
        // zontoetredingsfactor (schuine inval); werkt op beide balanstakken.
        let frame_transmitting = (1.0 - window.frame_fraction).max(0.0);
        let effective_area =
            window.area * window.g_value * F_W_GLAZING * shading_factor * frame_transmitting;
        let profile = climate.solar_irradiation.get(&window.orientation);
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
            // Netto zonwinst (7.32): bruto zontoetreding − hemelstraling. De
            // bruto-term is 0 als de oriëntatie geen klimaatprofiel heeft; Q_sky
            // wordt hoe dan ook afgetrokken (formule 7.39 hangt niet van I_sol af).
            let gross = match (effective_area > 0.0, profile) {
                (true, Some(p)) => {
                    let m_factor = movable.as_ref().map_or(1.0, |q| q[month]);
                    effective_area * m_factor * obstruction[month] * p[month]
                }
                _ => 0.0,
            };
            let q_sky_mj = q_sky_power_w * MONTH_HOURS[month.index()] * WH_TO_MJ;
            monthly[month.index()] += gross - q_sky_mj;
        }
    }
    MonthlyProfile::new(monthly)
}

/// Een niet-transparant (opaak) bouwschilelement voor de zonwinst-berekening
/// (NTA 8800 §7.6.3, formule 7.33).
///
/// Alleen aan buitenlucht grenzende opake vlakken dragen bij; de caller levert de
/// **geprojecteerde** opake oppervlakte aan (bruto vlak minus de raam-/deur-
/// openingen — die lopen via [`Window`], zodat er geen dubbeltelling ontstaat).
#[derive(Debug, Clone, Copy)]
pub struct OpaqueElement {
    /// Geprojecteerde opake oppervlakte `A_c;op,k` [m²] (§K.1.2).
    pub area: f64,
    /// Warmtedoorgangscoëfficiënt `U_c;op,k` [W/(m²·K)] (§8.2.2).
    pub u_value: f64,
    /// Oriëntatie voor de maand-zoninstraling `I_sol`.
    pub orientation: Orientation,
    /// Helling voor de vormfactor `F_sky` (§7.6.6.4) in de hemelstralingsterm.
    pub tilt: Tilt,
}

/// Bereken de maandelijkse netto zonwinst door opake (niet-transparante)
/// bouwschilelementen in MJ (NTA 8800 §7.6.3, formule 7.33).
///
/// Per element, per maand:
///
/// ```text
/// Q_op = α_sol · R_se · U_c · A_c · F_sh;obst · I_sol(or, mi)  −  Q_sky(tilt, mi)
/// ```
///
/// met `α_sol = 0,60` (§7.6.6.3), `R_se = 0,04 m²K/W` (C.2) en — norm-expliciet
/// voor opake vlakken — `F_sh;obst;op = 1` (§7.6.3: "Voor de dimensieloze
/// beschaduwingsreductiefactor voor externe belemmeringen van niet-transparant
/// element op,k, geldt: F_sh;obst = 1"). Er is voor opake vlakken géén g-waarde en
/// géén beweegbare zonwering.
///
/// De term is **balans-onafhankelijk** (identiek op de warmte- en koudebalans):
/// α, R_se, U en F_sky hangen niet van de balanstak af, en er is geen zonwering
/// die H/C-asymmetrisch werkt. Daarom is er, anders dan bij de ramen, geen
/// `SolarBalance`-parameter.
///
/// **Eenheden.** De invoer `climate.solar_irradiation` is per maand in MJ/m²
/// (reeds geïntegreerd, zie [`monthly_solar_gains`]); `α·R_se·U·A` is dimensieloos
/// × m², dus `effective_area · I_sol` levert direct MJ — dezelfde MJ-conventie als
/// de raam-zonwinst. `Q_sky` (formule 7.39) is identiek aan de raam-term, met de
/// opake `U`/`A`, en wordt óók afgetrokken bij een oriëntatie zonder
/// klimaatprofiel (hij hangt niet van `I_sol` af).
///
/// **Fysische richting.** Voor een goed-geïsoleerd, naar de koude hemel gericht
/// vlak (plat dak, `F_sky = 1,0`) overheerst doorgaans `Q_sky` de zon-absorptie →
/// netto een **verlies** (verwarming omhoog, koeling omlaag); voor een
/// zonbeschenen verticale gevel (`F_sky = 0,5`) kan de absorptie overheersen.
#[must_use]
pub fn monthly_opaque_solar_gains(
    elements: &[OpaqueElement],
    climate: &ClimateData,
) -> MonthlyProfile<Energy> {
    let mut monthly = [0.0_f64; 12];
    for el in elements {
        // Effectieve "zon-absorberende oppervlakte" α·R_se·U·A [m²]; × I_sol
        // [MJ/m²] → bruto-absorptie in MJ (formule 7.33, F_sh;obst = 1).
        let effective_area = ALPHA_SOL * R_SE * el.u_value * el.area;
        let profile = climate.solar_irradiation.get(&el.orientation);
        // Hemelstraling Q_sky (formule 7.39): langgolvig verlies naar de hemel met
        // de opake U + oppervlakte en de vormfactor uit de helling. Onafhankelijk
        // van I_sol en de balanstak.
        let f_sky = sky_view_factor(el.tilt.degrees);
        let q_sky_power_w = f_sky * R_SE * el.u_value * el.area * H_LR_E * DELTA_THETA_SKY_K;
        for month in Month::all() {
            let gross = match (effective_area > 0.0, profile) {
                (true, Some(p)) => effective_area * p[month],
                _ => 0.0,
            };
            let q_sky_mj = q_sky_power_w * MONTH_HOURS[month.index()] * WH_TO_MJ;
            monthly[month.index()] += gross - q_sky_mj;
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

    /// Hemelstralingsverlies Q_sky (MJ) voor een raam in maand `m` — spiegelt de
    /// productie-formule zodat tests de netto-uitkomst (bruto − Q_sky) kunnen
    /// verifiëren zonder de constanten te dupliceren.
    fn q_sky_mj(w: &Window, m: Month) -> f64 {
        sky_view_factor(w.tilt.degrees)
            * R_SE
            * w.u_value
            * w.area
            * H_LR_E
            * DELTA_THETA_SKY_K
            * MONTH_HOURS[m.index()]
            * WH_TO_MJ
    }

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
    fn shading_halveert_de_brutowinst() {
        // `shading_factor` schaalt alleen de bruto-zontoetreding; de
        // hemelstraling Q_sky (formule 7.39) hangt niet van g/F_sh af en blijft
        // gelijk. Dus: (half + Q_sky) = 0,5·(full + Q_sky).
        let climate = sample_climate();
        let w = zuid_window();
        let full = monthly_solar_gains(&[&w], &climate, 1.0, C);
        let half = monthly_solar_gains(&[&w], &climate, 0.5, C);
        for month in Month::all() {
            let qs = q_sky_mj(&w, month);
            assert!(((half[month] + qs) - 0.5 * (full[month] + qs)).abs() < 1e-6);
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
    fn ontbrekende_orientatie_geeft_alleen_negatieve_hemelstraling() {
        // Zonder klimaatprofiel voor de oriëntatie is de bruto-zontoetreding 0,
        // maar Q_sky (formule 7.39) wordt hoe dan ook afgetrokken → netto < 0.
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
            assert!((q[month] + q_sky_mj(&w, month)).abs() < 1e-9, "{month:?}");
        }
    }

    #[test]
    fn frame_fraction_1_geeft_alleen_negatieve_hemelstraling() {
        // 100% kozijn → geen glas → geen bruto-zontoetreding, maar Q_sky blijft
        // (hangt aan de volle raamoppervlakte en U, niet aan de kozijnfractie).
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
            assert!((q[month] + q_sky_mj(&w, month)).abs() < 1e-9, "{month:?}");
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
        // Q_sky is gelijk voor beide ramen en valt weg in het verschil; de
        // beweegbare zonwering werkt alleen op de bruto-zontoetreding. Juli:
        // handbediend Zuid f_sh;with = 0,59 → bruto-reductie tot factor 0,528.
        assert!(q_shaded[Month::Juli] < q_plain[Month::Juli]);
        let gross_plain_juli = q_plain[Month::Juli] + q_sky_mj(&plain, Month::Juli);
        let gross_shaded_juli = q_shaded[Month::Juli] + q_sky_mj(&shaded, Month::Juli);
        assert!((gross_shaded_juli - 0.528 * gross_plain_juli).abs() < 1e-6);
        // Januari: f_sh;with = 0 → geen reductie (netto identiek, incl. Q_sky).
        assert!((q_shaded[Month::Januari] - q_plain[Month::Januari]).abs() < 1e-9);
    }

    #[test]
    fn zonder_zonwering_is_bruto_min_hemelstraling() {
        // Regressie-pin: een raam zonder movable_shading én zonder obstruction
        // levert de norm-formule A·(F_w·g_gl;n)·(1−F_F)·I_sol − Q_sky — voor BEIDE
        // balanstakken identiek (F_w en Q_sky zijn balans-onafhankelijk).
        let climate = sample_climate();
        let w = zuid_window();
        let base = w.area * w.g_value * F_W_GLAZING * (1.0 - w.frame_fraction);
        for balance in [SolarBalance::Heating, SolarBalance::Cooling] {
            let q = monthly_solar_gains(&[&w], &climate, 1.0, balance);
            for month in Month::all() {
                let expected = base * climate.solar_irradiation[&Orientation::Zuid][month]
                    - q_sky_mj(&w, month);
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
        // Januari (Zuid vert.): F_sh;obst = 0,23 op de bruto-zontoetreding. Q_sky
        // is identiek voor beide ramen en valt weg in het verschil.
        assert!(heat_obstructed[Month::Januari] < heat_plain[Month::Januari]);
        let qs = q_sky_mj(&plain, Month::Januari);
        let gross_plain = heat_plain[Month::Januari] + qs;
        let gross_obstructed = heat_obstructed[Month::Januari] + qs;
        assert!((gross_obstructed - 0.23 * gross_plain).abs() < 1e-9);

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
        // A=5, g_gl;n=0.6, F_w=0.9, F_sh=1, (1-F_F)=0.8, I_juni=310 MJ/m²
        // bruto = 5 × 0.6 × 0.9 × 0.8 × 310 = 669,6 MJ
        // Q_sky (verticaal, F_sky=0.5, U=1.1, A=5, t_juni=720):
        //   0.5 × 0.04 × 1.1 × 5 × 4.14 × 11 × 720 × 0.0036 = 12,986 MJ
        // netto = 669,6 − 12,986 = 656,614 MJ
        let climate = sample_climate();
        let w = zuid_window();
        let q = monthly_solar_gains(&[&w], &climate, 1.0, C);
        let bruto = 5.0 * 0.6 * F_W_GLAZING * 0.8 * 310.0;
        let expected = bruto - q_sky_mj(&w, Month::Juni);
        assert!((q[Month::Juni] - expected).abs() < 1e-6, "kreeg {}", q[Month::Juni]);
    }

    // --- Opake zonwinst (formule 7.33) -------------------------------------

    #[test]
    fn opaak_lege_lijst_geeft_nul() {
        let climate = sample_climate();
        let q = monthly_opaque_solar_gains(&[], &climate);
        for month in Month::all() {
            assert!(q[month].abs() < 1e-12);
        }
    }

    #[test]
    fn opaak_zuidgevel_juni_handberekening() {
        // ONAFHANKELIJKE handberekening (formule 7.33), niet via een helper.
        // Zuidgevel: A=10 m², U=0,2 W/(m²K), verticaal (F_sky=0,5), I_zuid;juni=310 MJ/m².
        //   bruto = α·R_se·U·A·I = 0,6·0,04·0,2·10·310                       = 14,88 MJ
        //   Q_sky = F_sky·R_se·U·A·h_lr·Δθ·t·0,0036
        //         = 0,5·0,04·0,2·10·4,14·11·720·0,0036                       = 4,7215872 MJ
        //   netto = 14,88 − 4,7215872                                        = 10,1584128 MJ
        let climate = sample_climate();
        let el = OpaqueElement {
            area: 10.0,
            u_value: 0.2,
            orientation: Orientation::Zuid,
            tilt: Tilt::VERTICAL,
        };
        let q = monthly_opaque_solar_gains(&[el], &climate);
        assert!(
            (q[Month::Juni] - 10.158_412_8).abs() < 1e-6,
            "kreeg {}",
            q[Month::Juni]
        );
    }

    #[test]
    fn opaak_zuidelijk_dakvlak_45gr_juni_handberekening() {
        // Hellend dakvlak 45° (F_sky=0,75), A=40 m², U=0,15, zuid, I=310 MJ/m².
        //   bruto = 0,6·0,04·0,15·40·310                                     = 44,64 MJ
        //   Q_sky = 0,75·0,04·0,15·40·4,14·11·720·0,0036                     = 21,2471424 MJ
        //   netto = 44,64 − 21,2471424                                       = 23,3928576 MJ
        let climate = sample_climate();
        let el = OpaqueElement {
            area: 40.0,
            u_value: 0.15,
            orientation: Orientation::Zuid,
            tilt: Tilt::new(45.0).unwrap(),
        };
        let q = monthly_opaque_solar_gains(&[el], &climate);
        assert!(
            (q[Month::Juni] - 23.392_857_6).abs() < 1e-6,
            "kreeg {}",
            q[Month::Juni]
        );
    }

    #[test]
    fn opaak_ontbrekende_orientatie_geeft_puur_hemelstralingsverlies() {
        // Oost-gevel: sample_climate heeft geen Oost-profiel → bruto = 0, alleen
        // Q_sky-verlies (formule 7.39 hangt niet van I_sol af). Verticaal, A=10,
        // U=0,2, januari (t=744 h):
        //   Q_sky = 0,5·0,04·0,2·10·4,14·11·744·0,0036                       = 4,87897344 MJ
        //   netto = −4,87897344 MJ
        let climate = sample_climate();
        let el = OpaqueElement {
            area: 10.0,
            u_value: 0.2,
            orientation: Orientation::Oost,
            tilt: Tilt::VERTICAL,
        };
        let q = monthly_opaque_solar_gains(&[el], &climate);
        assert!(
            (q[Month::Januari] + 4.878_973_44).abs() < 1e-6,
            "kreeg {}",
            q[Month::Januari]
        );
    }
}
