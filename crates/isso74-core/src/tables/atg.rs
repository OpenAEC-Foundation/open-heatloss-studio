//! ATG (Adaptieve Temperatuur Grenswaarde) bounds — ISSO 74 Tabel 3.3 (PDF p.58).
//!
//! # Norm-interpretatie (gedocumenteerd — review-punt)
//!
//! Tabel 3.3 geeft per klasse drie kolommen (Winter / Tussenseizoen / Zomer)
//! voor zowel boven- als ondergrens, als functie van de running mean outdoor
//! temperature θ_rm. De afbeeldingen 3.1–3.4 tonen deze als **doorlopende
//! lijnen** waarbij het winter-"plateau" overgaat in de meeglijdende lijn.
//! Onze interpretatie van de knikpunten:
//!
//! ## Ondergrens
//! * Winter-plateau (klasse B/C/D = 20/19/18 °C) bij lage θ_rm.
//! * Meeglijdende lijn `X + 0,2·(θ_rm − 10)` (X = 20/19/18). Voetnoot 12/17/22:
//!   deze lijn is een extrapolatie met startpunt (θ_rm=10, winter-ondergrens)
//!   en eindpunt (θ_rm=25, zomer-ondergrens). Bij θ_rm = 10 geeft de lijn exact
//!   de winter-ondergrens, dus voor θ_rm < 10 ligt de lijn **onder** het
//!   plateau. → effectieve ondergrens = `max(winter_plateau, meeglijdende_lijn)`.
//!   Praktisch valt dit samen met: plateau voor θ_rm ≤ 10, lijn voor θ_rm > 10.
//!
//! ## Bovengrens
//! * Winter-plateau (klasse B/C/D = 24/25/26 °C) bij lage θ_rm.
//! * Bij `Alpha`: meeglijdende lijn `18,8 + 0,33·θ_rm + offset`
//!   (offset = 2/3/4 voor klasse B/C/D). Deze lijn snijdt het winter-plateau:
//!   onder het snijpunt geldt het plateau, erboven de lijn.
//!   → bovengrens(α) = `max(winter_plateau, meeglijdende_lijn)`.
//!   (Snijpunt klasse B: 18,8+0,33·θ_rm+2 = 24 → θ_rm ≈ 9,7 °C.)
//! * Bij `Beta`: vaste zomer-bovengrens (26/27/28). Onder het snijpunt met het
//!   winter-plateau geldt nog steeds het plateau; daarboven het vaste plafond.
//!   Omdat het zomer-plafond ≥ winter-plateau is voor alle klassen, geldt:
//!   bovengrens(β) = `max(winter_plateau, vast_plafond)` = `vast_plafond`
//!   zodra θ_rm hoog genoeg is. Praktisch: `max(winter_plateau, beta_cap)`.
//!
//! Deze `max`-constructie reproduceert de doorlopende V-/Λ-vorm van de
//! grafieken zonder harde seizoensgrenzen — de overgang volgt uitsluitend uit
//! θ_rm, conform de norm-tekst ("geen harde seizoensgrenzen").
//!
//! # Geldigheidsband (PDF p.57)
//! "Wanneer de running mean outdoor temperature onder de −5 of boven de +25 °C
//! komt gelden geen eisen." → uren met θ_rm buiten [−5, +25] tellen niet mee.

use crate::model::{AtgVariant, ComfortClass};

/// Onderste geldigheidsgrens van θ_rm [°C] (p.57).
pub const RMOT_VALID_MIN: f64 = -5.0;
/// Bovenste geldigheidsgrens van θ_rm [°C] (p.57).
pub const RMOT_VALID_MAX: f64 = 25.0;

/// Evaluated ATG bounds for one hour (operative temperature θ_o, [°C]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtgBounds {
    /// Lower bound [°C] — below this is an "te koud" overschrijding.
    pub lower: f64,
    /// Upper bound [°C] — above this is an "te warm" overschrijding.
    pub upper: f64,
}

/// Per-class constant offsets from Tabel 3.3.
struct ClassParams {
    /// Winter upper-bound plateau [°C].
    winter_upper: f64,
    /// Winter lower-bound plateau [°C].
    winter_lower: f64,
    /// α summer line offset (added to `18,8 + 0,33·θ_rm`).
    alpha_offset: f64,
    /// β fixed summer upper cap [°C].
    beta_cap: f64,
    /// Lower-bound sloped-line intercept at θ_rm = 10 (= winter_lower).
    lower_intercept: f64,
}

fn class_params(class: ComfortClass) -> ClassParams {
    match class {
        // Klasse A = numeriek identiek aan klasse B (Tabel 3.3 "Zie bij klasse B").
        ComfortClass::A | ComfortClass::B => ClassParams {
            winter_upper: 24.0,
            winter_lower: 20.0,
            alpha_offset: 2.0,
            beta_cap: 26.0,
            lower_intercept: 20.0,
        },
        ComfortClass::C => ClassParams {
            winter_upper: 25.0,
            winter_lower: 19.0,
            alpha_offset: 3.0,
            beta_cap: 27.0,
            lower_intercept: 19.0,
        },
        ComfortClass::D => ClassParams {
            winter_upper: 26.0,
            winter_lower: 18.0,
            alpha_offset: 4.0,
            beta_cap: 28.0,
            lower_intercept: 18.0,
        },
    }
}

/// Compute the ATG lower/upper operative-temperature bounds for a given θ_rm,
/// comfort class, and ATG variant.
///
/// Returns `None` when θ_rm is outside the validity band [−5, +25] °C — the
/// hour must then be excluded from the assessment (p.57).
pub fn atg_bounds(theta_rm: f64, class: ComfortClass, variant: AtgVariant) -> Option<AtgBounds> {
    if theta_rm < RMOT_VALID_MIN || theta_rm > RMOT_VALID_MAX {
        return None;
    }
    let p = class_params(class);

    // Ondergrens: meeglijdende lijn X + 0,2·(θ_rm − 10), begrensd op winter-plateau.
    let lower_line = p.lower_intercept + 0.2 * (theta_rm - 10.0);
    let lower = lower_line.max(p.winter_lower);

    // Bovengrens.
    let summer_upper = match variant {
        AtgVariant::Alpha => 18.8 + 0.33 * theta_rm + p.alpha_offset,
        AtgVariant::Beta => p.beta_cap,
    };
    let upper = summer_upper.max(p.winter_upper);

    Some(AtgBounds { lower, upper })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn outside_validity_band_returns_none() {
        assert!(atg_bounds(-6.0, ComfortClass::B, AtgVariant::Alpha).is_none());
        assert!(atg_bounds(26.0, ComfortClass::B, AtgVariant::Alpha).is_none());
    }

    #[test]
    fn class_b_alpha_summer_line() {
        // θ_rm = 20 → 18,8 + 0,33·20 + 2 = 27,4; plateau 24 → max = 27,4.
        let b = atg_bounds(20.0, ComfortClass::B, AtgVariant::Alpha).unwrap();
        assert_relative_eq!(b.upper, 27.4, epsilon = 1e-9);
        // Ondergrens: 20 + 0,2·(20−10) = 22.
        assert_relative_eq!(b.lower, 22.0, epsilon = 1e-9);
    }

    #[test]
    fn class_b_beta_fixed_cap() {
        // β: vast plafond 26 zodra > winter-plateau 24.
        let b = atg_bounds(20.0, ComfortClass::B, AtgVariant::Beta).unwrap();
        assert_relative_eq!(b.upper, 26.0, epsilon = 1e-9);
    }

    #[test]
    fn winter_plateau_dominates_at_low_rmot() {
        // θ_rm = 0: α-lijn = 18,8 + 0 + 2 = 20,8 < plateau 24 → upper = 24.
        let b = atg_bounds(0.0, ComfortClass::B, AtgVariant::Alpha).unwrap();
        assert_relative_eq!(b.upper, 24.0, epsilon = 1e-9);
        // ondergrens-lijn = 20 + 0,2·(0−10) = 18 < plateau 20 → lower = 20.
        assert_relative_eq!(b.lower, 20.0, epsilon = 1e-9);
    }

    #[test]
    fn class_a_equals_class_b() {
        let a = atg_bounds(18.0, ComfortClass::A, AtgVariant::Alpha).unwrap();
        let b = atg_bounds(18.0, ComfortClass::B, AtgVariant::Alpha).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn lower_bound_at_rmot_10_equals_winter() {
        // Voetnoot 12: bij θ_rm = 10 geeft de lijn exact de winter-ondergrens.
        let c = atg_bounds(10.0, ComfortClass::C, AtgVariant::Alpha).unwrap();
        assert_relative_eq!(c.lower, 19.0, epsilon = 1e-9);
    }
}
