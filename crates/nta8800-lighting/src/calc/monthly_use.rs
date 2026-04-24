//! Maandelijks eindenergiegebruik `W_L;use;mi` — atomaire formule.
//!
//! NTA 8800 §14.2.2 formule (14.7), V1 lumped decompositie:
//!
//! ```text
//! W_L;use;mi = P_n × F_u × F_d × F_c × A_f × t_mi × 3600 / 10^6   [MJ]
//! ```
//!
//! De factor `3600 / 10^6 = 0,0036` is de conversie van `W·h` naar `MJ`.

use nta8800_model::time::Month;
use nta8800_model::units::Energy;

use crate::model::LightingSystem;

/// Conversiefactor van `P [W] × t [h]` naar `E [MJ]`.
///
/// Afgeleid uit `1 W·h = 3600 J` en `1 MJ = 10^6 J` → factor
/// `3600 / 10^6 = 0,0036 MJ/(W·h)`. Identiek aan
/// [`nta8800_demand::calc::internal_gains::WH_TO_MJ`], hier lokaal
/// geduplicerd om geen dependency op de demand-crate te introduceren
/// (H.14 is onafhankelijk van H.7).
pub const WH_TO_MJ: f64 = 0.0036;

/// Uren per maand volgens NTA 8800 §17.2 (standaard-jaar, som = 8760 h).
///
/// Identiek aan de tabel in [`nta8800_demand`], hier lokaal geduplicerd om
/// cross-crate coupling op monthly-calendar-constants te vermijden. Bij
/// toekomstige consolidatie verhuizen naar `nta8800-model::time`.
pub const MONTH_HOURS: [f64; 12] = [
    744.0, // januari
    672.0, // februari (niet-schrikkeljaar)
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

/// Bereken `W_L;use;mi` in MJ voor één maand.
///
/// Geen validatie — de caller ([`crate::calculate_lighting`]) heeft
/// [`LightingSystem::validate`](crate::model::LightingSystem::validate)
/// al aangeroepen.
#[must_use]
pub fn monthly_w_l_use(system: &LightingSystem, floor_area_m2: f64, month: Month) -> Energy {
    let t_mi = MONTH_HOURS[month.index()];
    system.installed_power_w_per_m2
        * system.utilization_factor
        * system.daylight_factor
        * system.control_factor
        * floor_area_m2
        * t_mi
        * WH_TO_MJ
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn month_hours_sum_to_8760() {
        let total: f64 = MONTH_HOURS.iter().sum();
        assert_relative_eq!(total, 8760.0, epsilon = 1e-9);
    }

    #[test]
    fn januari_handberekening() {
        // P_n = 10, F_u = 0.5, F_d = 1.0, F_c = 1.0, A = 50, t_jan = 744
        // W_L;use = 10 × 0.5 × 1.0 × 1.0 × 50 × 744 × 0.0036 = 669.6 MJ
        let s = LightingSystem::new(10.0, 0.5, 1.0, 1.0).unwrap();
        let w = monthly_w_l_use(&s, 50.0, Month::Januari);
        assert_relative_eq!(w, 669.6, epsilon = 1e-6);
    }
}
