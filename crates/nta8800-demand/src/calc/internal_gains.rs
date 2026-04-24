//! Interne warmtewinst per maand.
//!
//! NTA 8800 §7.10, formule (7.35):
//!
//! ```text
//! Q_int;mi = Φ_int [W/m²] · A_g [m²] · t_mi [h] · 0,0036   [MJ]
//! ```
//!
//! De factor 0,0036 komt uit `3600 s/h / 10^6 J/MJ = 0,0036 MJ/(W·h)`.

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Energy;

use crate::model::InternalGains;

/// Conversiefactor van `Φ [W] × t [h]` naar `Q [MJ]`.
///
/// Afgeleid uit `1 W·h = 3600 J` en `1 MJ = 10^6 J` → factor `3600 / 10^6 =
/// 0,0036`. Hardcoded constant om unit-conversie expliciet te documenteren.
pub const WH_TO_MJ: f64 = 0.0036;

/// Uren per maand volgens NTA 8800 §17.2 (standaard-jaar, 8760 h totaal).
pub const MONTH_HOURS: [f64; 12] = [
    744.0, // januari
    672.0, // februari
    744.0, // maart
    720.0, // april
    744.0, // mei
    720.0, // juni
    744.0, // juli
    744.0, // augustus
    720.0, // september
    744.0, // oktober
    720.0, // november
    744.0, // december
];

/// Bereken de maandelijkse interne warmtewinst `Q_int;mi` in MJ.
///
/// # Parameters
///
/// - `gains` — [`InternalGains`] met `Φ_int` per maand in W/m²
/// - `floor_area_m2` — vloeroppervlakte `A_g` in m²
#[must_use]
pub fn monthly_internal_gains(gains: &InternalGains, floor_area_m2: f64) -> MonthlyProfile<Energy> {
    let mut values = [0.0_f64; 12];
    for month in Month::all() {
        let phi = gains.heat_flux_per_m2[month];
        let hours = MONTH_HOURS[month.index()];
        values[month.index()] = phi * floor_area_m2 * hours * WH_TO_MJ;
    }
    MonthlyProfile::new(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nta8800_model::zoning::UsageFunction;

    #[test]
    fn month_hours_sum_to_8760() {
        let total: f64 = MONTH_HOURS.iter().sum();
        assert!((total - 8760.0).abs() < 1e-9);
    }

    #[test]
    fn woonfunctie_100m2_januari() {
        // Φ = 3 W/m², A = 100 m², t = 744 h
        // Q = 3 × 100 × 744 × 0.0036 = 803.52 MJ
        let g = InternalGains::forfaitair(UsageFunction::Woonfunctie);
        let q = monthly_internal_gains(&g, 100.0);
        assert_relative_eq!(q[Month::Januari], 803.52, epsilon = 1e-6);
    }

    #[test]
    fn kantoor_200m2_juli() {
        // Φ = 4 W/m², A = 200 m², t_juli = 744 h
        // Q = 4 × 200 × 744 × 0.0036 = 2142.72 MJ
        let g = InternalGains::forfaitair(UsageFunction::Kantoorfunctie);
        let q = monthly_internal_gains(&g, 200.0);
        assert_relative_eq!(q[Month::Juli], 2142.72, epsilon = 1e-6);
    }

    #[test]
    fn jaarsom_schaalt_lineair_in_floor_area() {
        let g = InternalGains::forfaitair(UsageFunction::Woonfunctie);
        let q1: f64 = Month::all()
            .iter()
            .map(|&m| monthly_internal_gains(&g, 100.0)[m])
            .sum();
        let q2: f64 = Month::all()
            .iter()
            .map(|&m| monthly_internal_gains(&g, 200.0)[m])
            .sum();
        assert_relative_eq!(q2, 2.0 * q1, epsilon = 1e-6);
    }

    #[test]
    fn woonfunctie_100m2_jaartotaal() {
        // Φ = 3 W/m², A = 100 m², Σt = 8760 h
        // Q = 3 × 100 × 8760 × 0.0036 = 9460.8 MJ ≈ 9.5 GJ
        let g = InternalGains::forfaitair(UsageFunction::Woonfunctie);
        let jaar: f64 = Month::all()
            .iter()
            .map(|&m| monthly_internal_gains(&g, 100.0)[m])
            .sum();
        assert_relative_eq!(jaar, 9460.8, epsilon = 1e-3);
    }
}
