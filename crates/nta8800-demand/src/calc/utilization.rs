//! Benuttingsfactor η_H,gn (verwarming) en η_C,ls (koeling).
//!
//! NTA 8800 §7.6, formules (7.6)/(7.7) en (7.12)/(7.13). Gebruikt de
//! tijdconstante τ [h] van de rekenzone (zie [`super::time_constant`]) om
//! de parameter a te bepalen:
//!
//! ```text
//! a_H = a_H;0 + τ / τ_H;0   (a_H;0 = 1.0, τ_H;0 = 15 h)
//! ```
//!
//! Daarna de Siegenthaler-vorm van η:
//!
//! ```text
//!                ┌  (1 − γ^a) / (1 − γ^(a+1))   voor γ > 0, γ ≠ 1
//!  η_H;gn(γ) = ┤  a / (a + 1)                    voor γ = 1
//!                └  1                             voor γ ≤ 0  (géén warmtewinst)
//! ```
//!
//! Voor koeling spiegelen γ en a: `γ_C = Q_gn / Q_ht` (zelfde definitie),
//! maar η_C,ls gebruikt de inverse γ^(-a).

/// Norm-constante a_0 in formule (7.7) / (7.13) voor maandmethode.
///
/// Tabel 7.4 van NTA 8800 (maandmethode): a_0 = 1,0.
pub const A_0_MONTHLY: f64 = 1.0;

/// Norm-constante τ_0 (referentie-tijdconstante) in formule (7.7) / (7.13).
///
/// Tabel 7.4 van NTA 8800 (maandmethode): τ_0 = 15 h.
pub const TAU_0_MONTHLY_HOURS: f64 = 15.0;

/// Bereken dimensieloze parameter `a` uit tijdconstante τ.
///
/// Formule (7.7) / (7.13): `a = a_0 + τ / τ_0`.
#[must_use]
pub fn a_parameter(tau_hours: f64) -> f64 {
    A_0_MONTHLY + tau_hours / TAU_0_MONTHLY_HOURS
}

/// Benuttingsfactor voor warmtewinst η_H,gn — formule (7.6).
///
/// # Parameters
/// - `gamma`: `γ_H = Q_H;gn / Q_H;ht`
/// - `a`: parameter uit [`a_parameter`]
///
/// # Speciale gevallen
/// - `gamma ≤ 0` → geen (positieve) winst te benutten → retour 1,0 stub:
///   per §7.6 wordt η_H,gn niet gebruikt als `Q_gn = 0`; we retourneren 1
///   wat Q_nd = Q_ht geeft zonder correctie.
/// - `gamma → 1` → limietvorm `a / (a+1)` (L'Hôpital op de basisformule).
/// - `gamma → +∞` → `η → 1/γ` (nauwelijks winst benutbaar).
#[must_use]
pub fn utilization_heating(gamma: f64, a: f64) -> f64 {
    if !gamma.is_finite() || gamma <= 0.0 {
        // Geen warmtewinst relevant → benuttingsfactor heeft geen effect.
        return 1.0;
    }
    // γ = 1 exact (of bijna) → limietvorm
    if (gamma - 1.0).abs() < 1e-9 {
        return a / (a + 1.0);
    }
    let num = 1.0 - gamma.powf(a);
    let den = 1.0 - gamma.powf(a + 1.0);
    if den.abs() < f64::EPSILON {
        return a / (a + 1.0);
    }
    num / den
}

/// Benuttingsfactor voor koudeverlies η_C,ls — formule (7.12).
///
/// Spiegel van η_H,gn: gebruikt `γ_C = Q_C;gn / Q_C;ht` en `γ^(-a)`.
///
/// # Speciale gevallen
/// - `gamma ≤ 0` → retour 0,0 (geen warmte-verliesbenutting mogelijk).
/// - `gamma → 1` → limietvorm `a / (a+1)`.
/// - `gamma → +∞` → η → 1 (alle verlies is benutbaar, typisch koel-overschot).
#[must_use]
pub fn utilization_cooling(gamma: f64, a: f64) -> f64 {
    if !gamma.is_finite() || gamma <= 0.0 {
        return 0.0;
    }
    if (gamma - 1.0).abs() < 1e-9 {
        return a / (a + 1.0);
    }
    // Gebruik γ^(-a) = 1 / γ^a
    let g_pow_a = gamma.powf(a);
    let g_pow_a1 = gamma.powf(a + 1.0);
    if g_pow_a < f64::EPSILON || g_pow_a1 < f64::EPSILON {
        return 1.0;
    }
    let num = 1.0 - 1.0 / g_pow_a;
    let den = 1.0 - 1.0 / g_pow_a1;
    if den.abs() < f64::EPSILON {
        return a / (a + 1.0);
    }
    num / den
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parameter_uit_tau_standard() {
        // τ = 15 h → a = 1 + 15/15 = 2
        assert!((a_parameter(15.0) - 2.0).abs() < 1e-9);
        // τ = 0 → a = 1 (massa-loze zone)
        assert!((a_parameter(0.0) - 1.0).abs() < 1e-9);
        // τ = 150 h → a = 11 (zware zone)
        assert!((a_parameter(150.0) - 11.0).abs() < 1e-9);
    }

    #[test]
    fn heating_gamma_1_limietvorm() {
        // γ = 1 exact → a/(a+1)
        let a = 2.0;
        assert!((utilization_heating(1.0, a) - (a / (a + 1.0))).abs() < 1e-9);
    }

    #[test]
    fn heating_gamma_near_1() {
        // γ = 1 + ε → ≈ a/(a+1) (limiet continu)
        let a = 3.0;
        let eta_limit = a / (a + 1.0);
        let eta_near = utilization_heating(1.000_000_001, a);
        assert!(
            (eta_near - eta_limit).abs() < 1e-4,
            "eta_near={eta_near}, limit={eta_limit}"
        );
    }

    #[test]
    fn heating_gamma_zero_geeft_1() {
        // γ = 0 → geen warmtewinst → factor heeft geen effect → retour 1
        assert!((utilization_heating(0.0, 2.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn heating_gamma_klein_geeft_ongeveer_1() {
        // γ << 1 → bijna alle warmtewinst benutbaar → η → 1
        let eta = utilization_heating(0.1, 2.0);
        assert!(eta > 0.95, "η bij γ=0.1 moet ≈ 1: {eta}");
    }

    #[test]
    fn heating_gamma_groot_gaat_naar_nul() {
        // γ >> 1 → winst overstijgt verlies → η → 1/γ → klein
        let eta = utilization_heating(10.0, 2.0);
        assert!(eta < 0.15, "η bij γ=10 moet klein zijn: {eta}");
    }

    #[test]
    fn heating_monotoon_dalend_in_gamma() {
        // η_H is monotoon dalend in γ (intuïtief: meer winst vs. verlies →
        // relatief minder winst effectief benutbaar).
        let a = 2.0;
        let gammas = [0.1, 0.3, 0.5, 0.8, 1.2, 2.0, 5.0];
        let mut prev = utilization_heating(gammas[0], a);
        for &g in &gammas[1..] {
            let cur = utilization_heating(g, a);
            assert!(
                cur < prev,
                "η moet dalen in γ: γ={g}, prev={prev}, cur={cur}"
            );
            prev = cur;
        }
    }

    #[test]
    fn cooling_gamma_groot_geeft_ongeveer_1() {
        // γ_C >> 1 (veel winst tov verlies) → alle verlies benutbaar
        let eta = utilization_cooling(10.0, 2.0);
        assert!(eta > 0.9, "η_C bij γ=10 moet ≈ 1: {eta}");
    }

    #[test]
    fn cooling_gamma_klein_gaat_naar_nul() {
        // γ_C << 1 (weinig winst) → weinig verlies benutbaar
        let eta = utilization_cooling(0.1, 2.0);
        assert!(eta < 0.15, "η_C bij γ=0.1 moet klein zijn: {eta}");
    }

    #[test]
    fn cooling_gamma_1_limietvorm() {
        let a = 2.5;
        assert!((utilization_cooling(1.0, a) - (a / (a + 1.0))).abs() < 1e-9);
    }

    #[test]
    fn cooling_gamma_negatief_geeft_nul() {
        assert!((utilization_cooling(-1.0, 2.0)).abs() < 1e-9);
        assert!((utilization_cooling(0.0, 2.0)).abs() < 1e-9);
    }

    #[test]
    fn heating_returns_finite_for_realistic_range() {
        let a = a_parameter(48.0); // medium-massa woning
        for gamma in [0.01_f64, 0.1, 0.5, 0.9, 1.0, 1.1, 2.0, 5.0, 20.0] {
            let eta = utilization_heating(gamma, a);
            assert!(eta.is_finite(), "η niet-eindig bij γ={gamma}");
            assert!(
                (0.0..=1.0).contains(&eta),
                "η={eta} buiten [0,1] bij γ={gamma}"
            );
        }
    }
}
