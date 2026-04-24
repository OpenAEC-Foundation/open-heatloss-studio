//! Referentieklimaat De Bilt — maandelijkse buitentemperatuur, zoninstraling,
//! ventilatieve-koeltemperatuur, windsnelheid en WTW-preheat-temperatuur.
//!
//! Bron: NTA 8800:2025+C1:2026 hoofdstuk 17 (PDF p. 689-694). Deze waarden zijn
//! overgenomen uit NEN 5060 en vertegenwoordigen een jaar-gemiddeld
//! referentieklimaat voor Nederland.
//!
//! # Structuur
//!
//! - [`DE_BILT_MONTH_LENGTHS_HOURS`] — lengte van elke maand in uren,
//!   tabel 17.1. Samen 8 760 h (1 jaar).
//! - [`DE_BILT_OUTDOOR_TEMPERATURE`] — `ϑ_e;avg;mi` in °C, tabel 17.1.
//! - [`DE_BILT_COOLING_REFERENCE_TEMPERATURE`] — `ϑ_e;argII,mi` in °C,
//!   tabel 17.1. Januari en december zijn `None` (norm rapporteert `-`
//!   omdat het criterium 13 °C < `ϑ_e` < 24 °C in die maanden niet wordt
//!   gehaald).
//! - [`DE_BILT_WIND_SPEED`] — `u_site;mi` in m/s, tabel 17.1.
//! - [`DE_BILT_WTW_PREHEAT_TEMPERATURE`] — `ϑ_ODA;preh;WTWC;zi;mi` in °C,
//!   tabel 17.1. Waarde 0,00 voor maanden zonder koudeterugwinning.
//! - [`DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2`] — `I_sol;mi` in W/m²
//!   voor β=90° (verticale vlakken / gevels) per oriëntatie + maand,
//!   tabel 17.2 kolom β=90°.
//! - [`DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2`] — `I_sol;mi` in W/m²
//!   voor β=0° (horizontaal vlak / dak), tabel 17.2 eerste kolom.
//! - [`DE_BILT_SOLAR_IRRADIATION`] — samengestelde `BTreeMap<Orientation,
//!   MonthlyProfile<SolarIrradiation>>` met **cumulatieve MJ/m² per maand**
//!   (conversie: `W/m² × uren × 3600 / 10⁶`). Dit is het formaat dat
//!   [`ClimateData`] verwacht.
//! - [`de_bilt_climate_data`] — convenience constructor die alles in één
//!   `ClimateData` struct bundelt.
//!
//! # Conventie — eenheden
//!
//! NTA 8800 tabel 17.2 geeft zonnestraling in **W/m² maandgemiddeld over
//! alle uren**. Het `ClimateData` container-type in `nta8800-model` verwacht
//! echter `SolarIrradiation` als **MJ/m² cumulatief per maand** (zie
//! `nta8800_model::units::SolarIrradiation` docstring). De conversie:
//!
//! ```text
//! I_MJ[mi] = I_W[mi] × t_mi [h] × 3600 [s/h] / 10⁶
//! ```
//!
//! Voorbeeld: Zuid verticaal in juli = 109,7 W/m² × 744 h × 3600 / 10⁶
//! ≈ 293,8 MJ/m² cumulatief voor de maand juli.
//!
//! # Waarom alleen β=0° en β=90°
//!
//! Het `ClimateData`-model in `nta8800-model` koppelt aan [`Orientation`]
//! (8 kompasrichtingen + horizontaal). De verticale zoninstraling (β=90°)
//! gebruik je voor gevels en ramen; de horizontale (β=0°) voor platte daken
//! en zonnepanelen zonder helling. De tussenliggende hellingen (30°, 45°,
//! 60°, 135°, 180°) uit tabel 17.2 zijn bedoeld voor expliciet hellende
//! vlakken; die passen niet één-op-één in `Orientation` en worden later
//! afzonderlijk geïmplementeerd als een `TiltedSolarIrradiation`-tabel
//! (buiten D3-scope).

use std::collections::BTreeMap;
use std::sync::LazyLock;

use nta8800_model::climate::ClimateData;
use nta8800_model::location::Orientation;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{SolarIrradiation, Temperature, WindSpeed};

// ---------------------------------------------------------------------------
// Tabel 17.1 — Maandlengtes & buitentemperatuur
// ---------------------------------------------------------------------------

/// Lengte van elke maand in uren volgens NTA 8800 tabel 17.1.
///
/// Som = 8 760 h = 1 jaar (rekenperiode). Januari zit op index 0.
///
/// Referentie: [`NTA_8800_2025_TABEL17_1`](crate::references::NTA_8800_2025_TABEL17_1),
/// PDF p. 690.
pub const DE_BILT_MONTH_LENGTHS_HOURS: MonthlyProfile<f64> = MonthlyProfile::new([
    744.0, // Januari
    672.0, // Februari
    744.0, // Maart
    720.0, // April
    744.0, // Mei
    720.0, // Juni
    744.0, // Juli
    744.0, // Augustus
    720.0, // September
    744.0, // Oktober
    720.0, // November
    744.0, // December
]);

/// Maandgemiddelde buitenluchttemperatuur `ϑ_e;avg;mi` in °C voor
/// referentieklimaat De Bilt.
///
/// Referentie: [`NTA_8800_2025_TABEL17_1`](crate::references::NTA_8800_2025_TABEL17_1),
/// PDF p. 690. Jaargemiddelde ≈ 10,7 °C.
pub const DE_BILT_OUTDOOR_TEMPERATURE: MonthlyProfile<Temperature> = MonthlyProfile::new([
    2.61,  // Januari
    4.82,  // Februari
    5.91,  // Maart
    9.32,  // April
    14.73, // Mei
    16.12, // Juni
    18.05, // Juli
    18.48, // Augustus
    15.63, // September
    10.40, // Oktober
    7.99,  // November
    4.00,  // December
]);

/// Maandgemiddelde buitenluchttemperatuur voor ventilatieve koeling
/// `ϑ_e;argII,mi` in °C voor referentieklimaat De Bilt.
///
/// Gemiddelde over uren waarbij 13 °C < `ϑ_e` < 24 °C (oktober–april alleen
/// 22:00–06:00). `None` voor januari en december — de norm rapporteert in
/// tabel 17.1 `-` voor die maanden omdat het criterium niet wordt gehaald.
///
/// Referentie: [`NTA_8800_2025_TABEL17_1`](crate::references::NTA_8800_2025_TABEL17_1),
/// PDF p. 690 kolom `ϑ_e;argII,mi`.
pub const DE_BILT_COOLING_REFERENCE_TEMPERATURE: MonthlyProfile<Option<Temperature>> =
    MonthlyProfile::new([
        None,        // Januari (norm: "-")
        Some(13.97), // Februari
        Some(13.00), // Maart
        Some(13.70), // April
        Some(16.42), // Mei
        Some(16.76), // Juni
        Some(17.51), // Juli
        Some(18.24), // Augustus
        Some(16.74), // September
        Some(15.04), // Oktober
        Some(13.43), // November
        None,        // December (norm: "-")
    ]);

/// Maandgemiddelde windsnelheid op locatie `u_site;mi` in m/s voor
/// referentieklimaat De Bilt.
///
/// Wijkt licht af van de rekenkundige windsnelheid uit NEN 5060 —
/// uitschieters buiten één standaarddeviatie zijn weggefilterd (§17.2
/// opmerking 2). Gebruikt in het luchtstroommodel (§11.2) voor
/// winddruk-afhankelijke infiltratie.
///
/// Referentie: [`NTA_8800_2025_TABEL17_1`](crate::references::NTA_8800_2025_TABEL17_1),
/// PDF p. 690 kolom `u_site;mi`.
pub const DE_BILT_WIND_SPEED: MonthlyProfile<WindSpeed> = MonthlyProfile::new([
    3.04, // Januari
    4.15, // Februari
    2.99, // Maart
    3.06, // April
    2.97, // Mei
    2.78, // Juni
    2.63, // Juli
    2.51, // Augustus
    2.71, // September
    2.78, // Oktober
    2.83, // November
    2.83, // December
]);

/// Maandgemiddelde temperatuur van de toevoerlucht vóór de WTW tijdens
/// koudeterugwinning `ϑ_ODA;preh;WTWC;zi;mi` in °C voor referentieklimaat
/// De Bilt.
///
/// Waarde 0,00 voor januari–april en oktober–december — in die maanden vindt
/// geen koudeterugwinning plaats (`ϑ_e` is laag genoeg dat de WTW in
/// standaard-warmteterugwin-modus blijft). Werkelijke preheat-waarden voor
/// mei–september.
///
/// Referentie: [`NTA_8800_2025_TABEL17_1`](crate::references::NTA_8800_2025_TABEL17_1),
/// PDF p. 690 kolom `ϑ_ODA;preh;WTWC;zi;mi`.
pub const DE_BILT_WTW_PREHEAT_TEMPERATURE: MonthlyProfile<Temperature> = MonthlyProfile::new([
    0.00,  // Januari
    0.00,  // Februari
    0.00,  // Maart
    0.00,  // April
    25.63, // Mei
    27.49, // Juni
    26.34, // Juli
    27.29, // Augustus
    25.30, // September
    0.00,  // Oktober
    0.00,  // November
    0.00,  // December
]);

// ---------------------------------------------------------------------------
// Tabel 17.2 — Zoninstraling (raw W/m² data)
// ---------------------------------------------------------------------------

/// Maandgemiddelde opvallende zonnestraling `I_sol;mi` in **W/m²** op een
/// **horizontaal** vlak (β=0°, plat dak) voor referentieklimaat De Bilt.
///
/// Referentie: [`NTA_8800_2025_TABEL17_2`](crate::references::NTA_8800_2025_TABEL17_2),
/// PDF p. 691 eerste kolom. Grondreflectiecoëfficiënt ρ = 0,2.
pub const DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2: MonthlyProfile<f64> =
    MonthlyProfile::new([
        28.0,  // Januari
        49.3,  // Februari
        96.6,  // Maart
        160.5, // April
        197.0, // Mei
        209.3, // Juni
        191.0, // Juli
        177.2, // Augustus
        123.9, // September
        73.2,  // Oktober
        34.3,  // November
        21.0,  // December
    ]);

/// Maandgemiddelde opvallende zonnestraling `I_sol;mi` in **W/m²** op een
/// **verticaal** vlak (β=90°, gevel/raam) per oriëntatie voor
/// referentieklimaat De Bilt.
///
/// Kolom-volgorde per maand: `Noord`, `NoordOost`, `Oost`, `ZuidOost`, `Zuid`,
/// `ZuidWest`, `West`, `NoordWest`.
///
/// Referentie: [`NTA_8800_2025_TABEL17_2`](crate::references::NTA_8800_2025_TABEL17_2),
/// PDF p. 693 blok β=90°. Grondreflectiecoëfficiënt ρ = 0,2.
pub const DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2: [(Orientation, MonthlyProfile<f64>); 8] = [
    (
        Orientation::Noord,
        MonthlyProfile::new([
            11.1, 19.5, 34.8, 49.4, 61.9, 73.0, 66.7, 55.9, 41.4, 26.4, 13.6, 8.9,
        ]),
    ),
    (
        Orientation::NoordOost,
        MonthlyProfile::new([
            11.1, 21.5, 44.2, 72.9, 82.9, 92.0, 81.2, 63.9, 47.9, 29.1, 14.0, 8.9,
        ]),
    ),
    (
        Orientation::Oost,
        MonthlyProfile::new([
            20.2, 36.5, 70.7, 112.2, 114.6, 114.8, 104.9, 89.0, 73.7, 49.8, 23.9, 15.9,
        ]),
    ),
    (
        Orientation::ZuidOost,
        MonthlyProfile::new([
            43.9, 56.8, 95.4, 135.8, 128.4, 118.0, 113.2, 112.4, 103.6, 80.3, 47.1, 35.8,
        ]),
    ),
    (
        Orientation::Zuid,
        MonthlyProfile::new([
            60.1, 66.7, 101.8, 135.1, 124.9, 112.7, 109.7, 128.5, 122.3, 96.2, 59.5, 46.2,
        ]),
    ),
    (
        Orientation::ZuidWest,
        MonthlyProfile::new([
            48.1, 52.2, 82.1, 121.9, 122.1, 127.8, 117.1, 137.1, 112.2, 76.3, 45.6, 34.9,
        ]),
    ),
    (
        Orientation::West,
        MonthlyProfile::new([
            23.4, 32.8, 57.3, 96.2, 107.3, 125.7, 112.7, 120.0, 83.9, 46.7, 22.7, 15.2,
        ]),
    ),
    (
        Orientation::NoordWest,
        MonthlyProfile::new([
            11.4, 20.9, 38.5, 64.1, 78.9, 97.8, 88.5, 83.1, 53.6, 28.7, 13.8, 8.9,
        ]),
    ),
];

// ---------------------------------------------------------------------------
// Afgeleide: cumulatieve MJ/m² per maand (voor ClimateData)
// ---------------------------------------------------------------------------

/// Converteer een maandprofiel van W/m² (gemiddeld over alle uren) naar
/// MJ/m² cumulatief per maand.
///
/// Rekenregel: `I_MJ = I_W × t_h × 3600 / 10⁶`.
fn convert_w_per_m2_to_mj_per_m2(
    average_w_per_m2: &MonthlyProfile<f64>,
    month_hours: &MonthlyProfile<f64>,
) -> MonthlyProfile<SolarIrradiation> {
    let mut values = [0.0_f64; 12];
    for month in Month::all() {
        let idx = month.index();
        let w = average_w_per_m2[month];
        let h = month_hours[month];
        // W/m² × h × 3600 s/h / 1e6 J/MJ = MJ/m²
        values[idx] = w * h * 3600.0 / 1.0e6;
    }
    MonthlyProfile::new(values)
}

/// Maandelijkse cumulatieve zoninstraling in **MJ/m²** per oriëntatie voor
/// referentieklimaat De Bilt.
///
/// Bevat 9 oriëntaties: 8 kompasrichtingen (verticaal, β=90°, gevel/raam) +
/// [`Orientation::Horizontaal`] (β=0°, plat dak). De waarden zijn afgeleid
/// van [`DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2`] +
/// [`DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2`] via
/// [`DE_BILT_MONTH_LENGTHS_HOURS`].
///
/// `LazyLock` zorgt dat de conversie exact één keer wordt uitgevoerd bij
/// eerste gebruik.
///
/// Referentie: [`NTA_8800_2025_TABEL17_2`](crate::references::NTA_8800_2025_TABEL17_2).
pub static DE_BILT_SOLAR_IRRADIATION: LazyLock<
    BTreeMap<Orientation, MonthlyProfile<SolarIrradiation>>,
> = LazyLock::new(|| {
    let mut map: BTreeMap<Orientation, MonthlyProfile<SolarIrradiation>> = BTreeMap::new();

    for (orientation, profile_w) in &DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2 {
        let profile_mj = convert_w_per_m2_to_mj_per_m2(profile_w, &DE_BILT_MONTH_LENGTHS_HOURS);
        map.insert(*orientation, profile_mj);
    }

    let horizontal_mj = convert_w_per_m2_to_mj_per_m2(
        &DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2,
        &DE_BILT_MONTH_LENGTHS_HOURS,
    );
    map.insert(Orientation::Horizontaal, horizontal_mj);

    map
});

/// Stel de complete [`ClimateData`] samen voor referentieklimaat De Bilt.
///
/// Klonen is goedkoop (de onderliggende arrays zijn `[f64; 12]`).
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::climate::de_bilt_climate_data;
/// use nta8800_model::time::Month;
///
/// let climate = de_bilt_climate_data();
/// // Januari is kouder dan juli:
/// assert!(climate.outdoor_temperature[Month::Januari] < climate.outdoor_temperature[Month::Juli]);
/// ```
#[must_use]
pub fn de_bilt_climate_data() -> ClimateData {
    ClimateData {
        outdoor_temperature: DE_BILT_OUTDOOR_TEMPERATURE.clone(),
        solar_irradiation: DE_BILT_SOLAR_IRRADIATION.clone(),
        cooling_reference_temperature: DE_BILT_COOLING_REFERENCE_TEMPERATURE.clone(),
        wind_speed: DE_BILT_WIND_SPEED.clone(),
        wtw_preheat_temperature: DE_BILT_WTW_PREHEAT_TEMPERATURE.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn month_lengths_sum_to_8760_hours() {
        let sum: f64 = DE_BILT_MONTH_LENGTHS_HOURS.as_array().iter().sum();
        assert!(
            (sum - 8760.0).abs() < 1e-9,
            "De 12 maandlengtes moeten samen 8 760 h = 1 jaar zijn, maar zijn {sum}"
        );
    }

    #[test]
    fn outdoor_temperature_twaalf_maanden_aanwezig() {
        // Alle 12 waarden moeten eindig en binnen realistisch NL-bereik liggen.
        for month in Month::all() {
            let t = DE_BILT_OUTDOOR_TEMPERATURE[month];
            assert!(
                t.is_finite() && (-10.0..=30.0).contains(&t),
                "Temperatuur voor {month:?} = {t} valt buiten plausibel bereik"
            );
        }
    }

    #[test]
    fn januari_kouder_dan_juli() {
        // Sanity check: winter koud, zomer warm.
        let jan = DE_BILT_OUTDOOR_TEMPERATURE[Month::Januari];
        let jul = DE_BILT_OUTDOOR_TEMPERATURE[Month::Juli];
        assert!(
            jan < jul,
            "Januari ({jan} °C) moet kouder zijn dan juli ({jul} °C)"
        );
    }

    #[test]
    fn solar_irradiation_bevat_negen_orientaties() {
        let map = &*DE_BILT_SOLAR_IRRADIATION;
        assert_eq!(map.len(), 9, "Verwacht 8 kompasrichtingen + horizontaal");
        assert!(map.contains_key(&Orientation::Noord));
        assert!(map.contains_key(&Orientation::NoordOost));
        assert!(map.contains_key(&Orientation::Oost));
        assert!(map.contains_key(&Orientation::ZuidOost));
        assert!(map.contains_key(&Orientation::Zuid));
        assert!(map.contains_key(&Orientation::ZuidWest));
        assert!(map.contains_key(&Orientation::West));
        assert!(map.contains_key(&Orientation::NoordWest));
        assert!(map.contains_key(&Orientation::Horizontaal));
    }

    #[test]
    fn solar_irradiation_alle_maanden_aanwezig_per_orientatie() {
        let map = &*DE_BILT_SOLAR_IRRADIATION;
        for (orientation, profile) in map {
            for month in Month::all() {
                let v = profile[month];
                assert!(
                    v.is_finite() && v >= 0.0,
                    "{orientation:?} / {month:?} = {v}: moet eindig en niet-negatief zijn"
                );
            }
        }
    }

    #[test]
    fn zuidzon_juni_groter_dan_noordzon_juni() {
        // Sanity check: zuid verticaal in juni > noord verticaal in juni.
        let map = &*DE_BILT_SOLAR_IRRADIATION;
        let zuid_juni = map[&Orientation::Zuid][Month::Juni];
        let noord_juni = map[&Orientation::Noord][Month::Juni];
        assert!(
            zuid_juni > noord_juni,
            "Zuid juni ({zuid_juni} MJ/m²) moet > Noord juni ({noord_juni} MJ/m²)"
        );
    }

    #[test]
    fn horizontale_zon_juni_piek() {
        // Horizontaal vlak in juni piekt rond 209 W/m² → ~542 MJ/m² cumulatief.
        let map = &*DE_BILT_SOLAR_IRRADIATION;
        let horiz_juni = map[&Orientation::Horizontaal][Month::Juni];
        // 209.3 × 720 × 3600 / 1e6 = 542.5056
        assert_relative_eq!(horiz_juni, 542.5056, max_relative = 1e-6);
    }

    #[test]
    fn conversie_zuid_juli_klopt_met_formule() {
        // Handmatige check: Zuid verticaal juli = 109,7 W/m², juli = 744 h.
        // 109.7 × 744 × 3600 / 1e6 = 293.821 MJ/m²
        let map = &*DE_BILT_SOLAR_IRRADIATION;
        let zuid_juli = map[&Orientation::Zuid][Month::Juli];
        assert_relative_eq!(zuid_juli, 293.82048, max_relative = 1e-5);
    }

    #[test]
    fn climate_data_serde_round_trip() {
        // JSON round-trip met f64 kan subtiel laatste-bit verschil geven na
        // decimaal-encoding + parsing. Dat maakt strict PartialEq onbetrouwbaar;
        // we controleren daarom element-voor-element met float-tolerantie.
        let data = de_bilt_climate_data();
        let json = serde_json::to_string(&data).expect("serialize");
        let back: ClimateData = serde_json::from_str(&json).expect("deserialize");

        for month in Month::all() {
            assert_relative_eq!(
                data.outdoor_temperature[month],
                back.outdoor_temperature[month],
                max_relative = 1e-12
            );
        }
        assert_eq!(
            data.solar_irradiation.len(),
            back.solar_irradiation.len(),
            "Aantal oriëntaties moet gelijk blijven na round-trip"
        );
        for (orientation, profile) in &data.solar_irradiation {
            let back_profile = back
                .solar_irradiation
                .get(orientation)
                .expect("Oriëntatie moet aanwezig zijn na round-trip");
            for month in Month::all() {
                assert_relative_eq!(profile[month], back_profile[month], max_relative = 1e-12);
            }
        }
        // Nieuwe velden round-trip: cooling-ref, wind, WTW-preheat.
        for month in Month::all() {
            assert_eq!(
                data.cooling_reference_temperature[month],
                back.cooling_reference_temperature[month],
                "cooling_reference_temperature {month:?} moet round-trippen"
            );
            assert_relative_eq!(
                data.wind_speed[month],
                back.wind_speed[month],
                max_relative = 1e-12
            );
            assert_relative_eq!(
                data.wtw_preheat_temperature[month],
                back.wtw_preheat_temperature[month],
                max_relative = 1e-12
            );
        }
    }

    #[test]
    fn climate_data_bevat_alle_verwachte_inhoud() {
        let data = de_bilt_climate_data();
        // 12 temperatuur-waarden
        for month in Month::all() {
            assert!(data.outdoor_temperature[month].is_finite());
        }
        // 9 oriëntaties
        assert_eq!(data.solar_irradiation.len(), 9);
        // 9 × 12 = 108 solar waarden
        let total: usize = data
            .solar_irradiation
            .values()
            .map(|p| p.as_array().len())
            .sum();
        assert_eq!(total, 108, "Verwacht 9 oriëntaties × 12 maanden");

        // Nieuwe tabel-17.1-velden: elk 12 entries, finite waar ingevuld.
        for month in Month::all() {
            // Wind + WTW zijn altijd f64
            assert!(data.wind_speed[month].is_finite());
            assert!(data.wtw_preheat_temperature[month].is_finite());
            // Cooling ref: Option<T>; als Some dan finite
            if let Some(t) = data.cooling_reference_temperature[month] {
                assert!(t.is_finite());
            }
        }
    }

    #[test]
    fn cooling_reference_temperature_januari_december_zijn_none() {
        // NTA 8800 tabel 17.1 rapporteert expliciet "-" voor januari en
        // december (criterium 13 °C < ϑ_e < 24 °C wordt niet gehaald).
        assert_eq!(DE_BILT_COOLING_REFERENCE_TEMPERATURE[Month::Januari], None);
        assert_eq!(DE_BILT_COOLING_REFERENCE_TEMPERATURE[Month::December], None);
    }

    #[test]
    fn cooling_reference_temperature_februari_tot_november_zijn_gevuld() {
        // De norm heeft voor 10 maanden expliciete waarden.
        for month in [
            Month::Februari,
            Month::Maart,
            Month::April,
            Month::Mei,
            Month::Juni,
            Month::Juli,
            Month::Augustus,
            Month::September,
            Month::Oktober,
            Month::November,
        ] {
            assert!(
                DE_BILT_COOLING_REFERENCE_TEMPERATURE[month].is_some(),
                "Verwacht norm-waarde voor {month:?}"
            );
        }
    }

    #[test]
    fn cooling_reference_juli_hoger_dan_outdoor_juli() {
        // Sanity: ϑ_e;argII,mi is geselecteerd boven 13 °C-drempel en is in
        // juli dus consistent, maar ligt lager dan de volledige dag-
        // gemiddelde buitentemperatuur in juli (18,05 °C) omdat de selectie
        // 13–24 °C afkapt. Norm-waarde: 17,51 °C.
        let ref_jul = DE_BILT_COOLING_REFERENCE_TEMPERATURE[Month::Juli]
            .expect("Juli moet een cooling-ref-temperatuur hebben");
        let outdoor_jul = DE_BILT_OUTDOOR_TEMPERATURE[Month::Juli];
        // Bij dit specifieke klimaat is cooling-ref lager dan outdoor-ref
        // (ligt boven de 13 °C drempel maar afgekapt op 24 °C).
        assert!(
            ref_jul < outdoor_jul,
            "ϑ_e;argII,mi juli ({ref_jul}) moet < ϑ_e;avg,mi juli ({outdoor_jul})"
        );
        // Specifieke norm-waarde controle
        assert!((ref_jul - 17.51).abs() < 1e-9);
    }

    #[test]
    fn wind_speed_alle_maanden_plausibel() {
        // Nederlandse windsnelheden liggen tussen ongeveer 2 en 5 m/s
        // maandgemiddeld.
        for month in Month::all() {
            let u = DE_BILT_WIND_SPEED[month];
            assert!(
                u.is_finite() && (2.0..=5.0).contains(&u),
                "Wind {month:?} = {u} m/s valt buiten plausibel bereik 2–5"
            );
        }
    }

    #[test]
    fn wind_speed_februari_piek() {
        // NTA 8800 tabel 17.1: februari is de windigste maand in De Bilt
        // met 4,15 m/s. Alle andere maanden liggen onder 3,1 m/s.
        let feb = DE_BILT_WIND_SPEED[Month::Februari];
        assert!((feb - 4.15).abs() < 1e-9);
        for month in Month::all() {
            if month != Month::Februari {
                assert!(
                    DE_BILT_WIND_SPEED[month] < feb,
                    "Februari ({feb}) moet windigste maand zijn, maar {month:?} = {}",
                    DE_BILT_WIND_SPEED[month]
                );
            }
        }
    }

    #[test]
    fn wtw_preheat_nul_in_wintermaanden() {
        // Januari-april + oktober-december: geen koudeterugwinning, WTW-
        // preheat is 0,00 °C volgens norm.
        for month in [
            Month::Januari,
            Month::Februari,
            Month::Maart,
            Month::April,
            Month::Oktober,
            Month::November,
            Month::December,
        ] {
            let v = DE_BILT_WTW_PREHEAT_TEMPERATURE[month];
            assert!(
                v.abs() < 1e-9,
                "WTW-preheat {month:?} moet 0,00 °C zijn, maar is {v}"
            );
        }
    }

    #[test]
    fn wtw_preheat_zomermaanden_positief() {
        // Mei-september: koudeterugwinning actief, preheat-waarden tussen
        // 25 en 28 °C volgens norm.
        for month in [
            Month::Mei,
            Month::Juni,
            Month::Juli,
            Month::Augustus,
            Month::September,
        ] {
            let v = DE_BILT_WTW_PREHEAT_TEMPERATURE[month];
            assert!(
                (25.0..=28.0).contains(&v),
                "WTW-preheat {month:?} = {v} valt buiten verwacht bereik 25–28 °C"
            );
        }
    }

    #[test]
    fn raw_w_per_m2_exports_zijn_beschikbaar() {
        // Deze constantes zijn publiek zodat downstream crates ook de ongeconverteerde
        // waarden kunnen gebruiken voor bijv. piekvermogen-berekeningen.
        assert!((DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2[Month::Juni] - 209.3).abs() < 1e-9);
        let zuid = DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2
            .iter()
            .find(|(o, _)| *o == Orientation::Zuid)
            .map(|(_, p)| p)
            .expect("Zuid moet in verticale dataset zitten");
        assert!((zuid[Month::December] - 46.2).abs() < 1e-9);
    }
}
