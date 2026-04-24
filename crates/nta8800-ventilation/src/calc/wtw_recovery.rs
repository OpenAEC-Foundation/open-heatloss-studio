//! WTW-warmteterugwinning — toevoertemperatuur na WTW en teruggewonnen energie.
//!
//! NTA 8800 formule (11.107)–(11.108) in vereenvoudigde V1-vorm (zonder
//! bypass-factor, warmtelekken, onbalans, condensvorming — die factoren
//! zijn V2):
//!
//! ```text
//! ϑ_sup = ϑ_e + η_hr · (ϑ_i − ϑ_e)           (11.108, vereenvoudigd)
//! Q_WTW;mi = q · ρ_a · c_a · (ϑ_sup − ϑ_e) · t_mi / 10⁶
//! ```

use nta8800_model::units::{Energy, Temperature};

use super::AIR_VOLUMETRIC_HEAT_J_PER_M3_K;

/// Bereken de toevoerluchttemperatuur na WTW + de teruggewonnen energie.
///
/// Retourneert `(ϑ_sup, Q_WTW)`:
/// - `ϑ_sup`: temperatuur van toevoerlucht na WTW in °C
/// - `Q_WTW`: teruggewonnen warmte in MJ voor deze maand
///
/// # Parameters
///
/// - `theta_outdoor`: buitentemperatuur ϑ_e in °C
/// - `theta_indoor`: binnentemperatuur ϑ_i in °C
/// - `efficiency`: effectief WTW-rendement η_hr in `[0, 1]` (V1: geen
///   `f_prac;hr` correctie)
/// - `q_m3_per_h`: ventilatiestroom in m³/h
/// - `t_mi_hours`: duur van de maand in h
///
/// # Edge cases
///
/// - `η = 0` → ϑ_sup = ϑ_e (geen WTW-effect), Q_WTW = 0
/// - `η = 1` → ϑ_sup = ϑ_i (perfecte terugwinning), Q_WTW = q·ρc·(ϑ_i−ϑ_e)·t
/// - `ϑ_e ≥ ϑ_i` → `max(0)` clamp, Q_WTW = 0 (geen terugwinning in zomer
///   tijdens warmtebehoefte)
#[must_use]
pub fn apply_wtw(
    theta_outdoor: Temperature,
    theta_indoor: Temperature,
    efficiency: f64,
    q_m3_per_h: f64,
    t_mi_hours: f64,
) -> (Temperature, Energy) {
    let delta_inside_outside = theta_indoor - theta_outdoor;
    if delta_inside_outside <= 0.0 {
        // zomer-situatie: geen warmtebehoefte → geen terugwinning in dit model
        return (theta_outdoor, 0.0);
    }

    let theta_supply = theta_outdoor + efficiency * delta_inside_outside;
    let delta_wtw = theta_supply - theta_outdoor; // = η · (ϑ_i − ϑ_e)
    let q_recovered_j = q_m3_per_h * AIR_VOLUMETRIC_HEAT_J_PER_M3_K * delta_wtw * t_mi_hours;
    (theta_supply, q_recovered_j / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn efficiency_zero_passes_through_outdoor() {
        let (theta_sup, q) = apply_wtw(5.0, 20.0, 0.0, 150.0, 744.0);
        assert_relative_eq!(theta_sup, 5.0, epsilon = 1e-9);
        assert_relative_eq!(q, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn efficiency_one_reaches_indoor() {
        let (theta_sup, q) = apply_wtw(5.0, 20.0, 1.0, 150.0, 744.0);
        assert_relative_eq!(theta_sup, 20.0, epsilon = 1e-9);
        assert!(q > 0.0);
    }

    #[test]
    fn efficiency_one_q_wtw_equals_heat_loss_without_wtw() {
        // Als η = 1, dan wordt ALLE warmteverlies teruggewonnen → Q_WTW
        // moet gelijk zijn aan q·ρc·ΔT·t / 10⁶.
        let q_recovered = apply_wtw(5.0, 20.0, 1.0, 150.0, 744.0).1;
        let expected = 150.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K * 15.0 * 744.0 / 1_000_000.0;
        assert_relative_eq!(q_recovered, expected, epsilon = 1e-6);
    }

    #[test]
    fn efficiency_80_pct_typical() {
        // η = 0,8, ϑ_e = 5, ϑ_i = 20 → ϑ_sup = 5 + 0,8·15 = 17
        let (theta_sup, _) = apply_wtw(5.0, 20.0, 0.80, 100.0, 744.0);
        assert_relative_eq!(theta_sup, 17.0, epsilon = 1e-9);
    }

    #[test]
    fn summer_no_recovery_returned() {
        // ϑ_e > ϑ_i → geen warmtebehoefte → recovery = 0
        let (theta_sup, q) = apply_wtw(25.0, 18.0, 0.80, 150.0, 744.0);
        assert_relative_eq!(theta_sup, 25.0, epsilon = 1e-9);
        assert_relative_eq!(q, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn recovery_scales_with_efficiency() {
        let q_low = apply_wtw(5.0, 20.0, 0.40, 150.0, 744.0).1;
        let q_high = apply_wtw(5.0, 20.0, 0.80, 150.0, 744.0).1;
        assert_relative_eq!(q_high, 2.0 * q_low, epsilon = 1e-6);
    }
}
