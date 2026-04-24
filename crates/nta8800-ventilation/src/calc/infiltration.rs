//! Infiltratie-luchtstroom `q_V;lea` — forfaitaire berekening.
//!
//! V1 levert alleen de forfaitaire aanpak: `q_V;lea = q_v10 × A_envelope`
//! waarbij `q_v10` de specifieke luchtlekkage bij 10 Pa is (dm³/s/m²) uit
//! een luchtdichtheidsmeting of forfaitair per bouwjaar-klasse. De volledige
//! stromingsweerstand-aanpak uit §11.2.1 (met drukverschil p_z;ref, tabel
//! 11.1) is V2-scope.

/// Bereken infiltratie-debiet in m³/h uit een qv10-meting.
///
/// # Parameters
///
/// - `qv10_dm3_per_s_per_m2`: specifieke luchtlekkage bij Δp = 10 Pa, in
///   dm³/(s·m²). Typische waarden:
///   - Nieuwbouw 2026: `< 0,4`
///   - Goed gerenoveerd: `0,4 – 0,8`
///   - Matig geïsoleerd pre-2000: `0,8 – 1,5`
///   - Slechte luchtdichtheid: `> 1,5`
/// - `envelope_area_m2`: totaal omhullingsoppervlak (gevel + dak + vloer) in m².
///
/// # Conversie
///
/// NTA 8800 rekent in m³/h bij 1 Pa drukverschil, niet bij 10 Pa.
/// Stromingsweerstand exponent n ≈ 0,67 → `q_1 ≈ q_10 / 10^0,67 ≈ q_10 / 4,642`.
/// Eenheidconversie dm³/s → m³/h = ×3,6.
///
/// Effectief: `q_V;lea [m³/h] = qv10 × A × 3,6 / 4,642`.
#[must_use]
pub fn infiltration_from_qv10(qv10_dm3_per_s_per_m2: f64, envelope_area_m2: f64) -> f64 {
    // 10 Pa → 1 Pa conversion factor (flow exponent n ≈ 0,67 per NEN 2686)
    const PRESSURE_CORRECTION_10PA_TO_1PA: f64 = 4.642;
    const DM3_PER_S_TO_M3_PER_H: f64 = 3.6;

    qv10_dm3_per_s_per_m2 * envelope_area_m2 * DM3_PER_S_TO_M3_PER_H
        / PRESSURE_CORRECTION_10PA_TO_1PA
}

/// User-supplied forfaitaire infiltratie: `q [m³/h] = specific [dm³/s/m²] × A [m²] × 3,6`.
///
/// Handig als de user al een 1-Pa-referentie-debiet heeft (bv. als default
/// `0,25 dm³/s/m²` van een woningbouwnorm). Geen drukconversie.
#[must_use]
pub fn infiltration_forfait_per_envelope(
    specific_flow_dm3_per_s_per_m2: f64,
    envelope_area_m2: f64,
) -> f64 {
    specific_flow_dm3_per_s_per_m2 * envelope_area_m2 * 3.6
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn infiltration_from_qv10_typical_new_build() {
        // Nieuwbouw: qv10 = 0,3 dm³/s/m², omhulling 300 m²
        // q_1 = 0,3 × 300 × 3,6 / 4,642 ≈ 69,8 m³/h
        let q = infiltration_from_qv10(0.3, 300.0);
        assert_relative_eq!(q, 69.796, epsilon = 0.1);
    }

    #[test]
    fn infiltration_from_qv10_scales_linearly() {
        let q1 = infiltration_from_qv10(0.5, 200.0);
        let q2 = infiltration_from_qv10(1.0, 200.0);
        assert_relative_eq!(q2, 2.0 * q1, epsilon = 1e-6);
    }

    #[test]
    fn infiltration_forfait_conversion() {
        // 0,25 dm³/s/m² over 100 m² = 25 dm³/s = 90 m³/h
        let q = infiltration_forfait_per_envelope(0.25, 100.0);
        assert_relative_eq!(q, 90.0, epsilon = 1e-9);
    }

    #[test]
    fn infiltration_zero_flow_gives_zero() {
        assert_relative_eq!(infiltration_from_qv10(0.0, 500.0), 0.0, epsilon = 1e-9);
        assert_relative_eq!(
            infiltration_forfait_per_envelope(0.0, 500.0),
            0.0,
            epsilon = 1e-9
        );
    }
}
