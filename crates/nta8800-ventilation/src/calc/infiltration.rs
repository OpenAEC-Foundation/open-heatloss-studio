//! Infiltratie-luchtstroom `q_V;lea` — forfaitaire berekening + NTA 8800
//! §11.2.5 referentiedebiet.
//!
//! Deze module levert twee aanpakken:
//!
//! 1. **V1-heuristiek** ([`infiltration_from_qv10`],
//!    [`infiltration_forfait_per_envelope`]) — een directe envelope-schatting
//!    zonder de norm-keten. Blijft bestaan voor backward-compat met de
//!    bestaande [`crate::AirFlow`]-input-API.
//! 2. **NTA 8800 §11.2.5 referentiedebiet** ([`q_v1_lea_ref`],
//!    [`q_v10_lea_ref_forfait`]) — het norm-exacte infiltratie-referentiedebiet
//!    `q_v1;lea;ref` (m³/h bij 1 Pa), volgens formules (11.84)-(11.86). Dit is
//!    de invoer voor de C2 massabalans-/drukoplosroutine (§11.2.1).

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

/// Forfaitaire specifieke luchtdoorlatendheid bij 10 Pa, `q_v10;lea;ref`
/// (dm³/(s·m²)), volgens NTA 8800 formule (11.86).
///
/// Te gebruiken wanneer **geen meetwaarde** uit een luchtdoorlatendheids-
/// meting (NEN 2686:1988) beschikbaar is.
///
/// # Norm-afleiding — formule (11.86)
///
/// ```text
/// q_v10;lea;ref = f_type · f_y · q_v10;spec;reken
/// ```
///
/// waarin:
/// - `q_v10;spec;reken` — de rekenwaarde voor de specifieke luchtdoorlatendheid
///   per gebouwcategorie (NTA 8800 tabel 11.14,
///   [`crate::tables::specific_air_permeability_calc`]);
/// - `f_type` — de van de uitvoeringsvariant afhankelijke correctiefactor
///   (NTA 8800 tabel 11.14, [`crate::tables::building_type_correction_factor`]);
/// - `f_y` — de bouwjaarcorrectiefactor (NTA 8800 tabel 11.13,
///   [`crate::tables::build_year_correction_factor`]).
///
/// # Parameters
///
/// - `leakage_type`: gebouwtype-classificatie (categorie + uitvoeringsvariant).
/// - `build_year`: bouw- of (volledige) renovatiejaar `j`.
///
/// # Resultaat
///
/// `q_v10;lea;ref` in dm³/(s·m²) bij Δp = 10 Pa.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.86), §11.2.5.2, PDF p. 485-486.
#[must_use]
pub fn q_v10_lea_ref_forfait(
    leakage_type: crate::model::BuildingLeakageType,
    build_year: u32,
) -> f64 {
    // Formule (11.86): q_v10;lea;ref = f_type · f_y · q_v10;spec;reken.
    let q_v10_spec_reken = crate::tables::specific_air_permeability_calc(leakage_type);
    let f_type = crate::tables::building_type_correction_factor(leakage_type);
    let f_y = crate::tables::build_year_correction_factor(build_year);
    f_type * f_y * q_v10_spec_reken
}

/// Infiltratie-referentiedebiet `q_v1;lea;ref` (m³/h bij een uniform
/// drukverschil van 1 Pa), volgens NTA 8800 formule (11.85).
///
/// Dit is het referentiedebiet dat de C2 massabalans-/drukoplosroutine
/// (§11.2.1) als infiltratie-invoer gebruikt. Het drukt de luchtlekkage van de
/// gebouwschil uit bij het uniforme referentiedrukverschil van 1 Pa; de
/// werkelijke infiltratie volgt later uit `q_V;lea = C_lea · Δp^n_lea` met het
/// per-maand opgeloste drukverschil `p_z;ref` (formule (11.84) + §11.2.1.6).
///
/// # Norm-afleiding — formule (11.85)
///
/// ```text
/// q_v1;lea;ref = q_v10;lea;ref · (1/10)^n_lea · A_g · 3,6
/// ```
///
/// waarin:
/// - `q_v10;lea;ref` — de specifieke luchtdoorlatendheid bij 10 Pa, in
///   dm³/(s·m²) — uit een meting (NEN 2686:1988) óf forfaitair via
///   [`q_v10_lea_ref_forfait`] (formule (11.86));
/// - `n_lea = 0,67` — de stromingsexponent voor lekverliezen (NTA 8800
///   tabel 11.2, [`crate::tables::FLOW_EXPONENT_LEAKAGE`]). De factor
///   `(1/10)^n_lea` rekent het 10 Pa-debiet om naar het 1 Pa-referentiedebiet;
/// - `A_g` — de gebruiksoppervlakte van de rekenzone, in m²;
/// - `3,6` — eenheidsconversie dm³/s → m³/h.
///
/// # Parameters
///
/// - `q_v10_lea_ref_dm3_per_s_per_m2`: `q_v10;lea;ref` in dm³/(s·m²).
/// - `gross_floor_area_m2`: gebruiksoppervlakte `A_g` in m².
///
/// # Resultaat
///
/// `q_v1;lea;ref` in m³/h bij Δp = 1 Pa.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.85), §11.2.5, PDF p. 485.
#[must_use]
pub fn q_v1_lea_ref(q_v10_lea_ref_dm3_per_s_per_m2: f64, gross_floor_area_m2: f64) -> f64 {
    /// Eenheidsconversie dm³/s → m³/h — NTA 8800 formule (11.85) factor 3,6.
    const DM3_PER_S_TO_M3_PER_H: f64 = 3.6;
    /// Referentiedrukverhouding 1 Pa / 10 Pa uit formule (11.85): `(1/10)`.
    const PRESSURE_RATIO_1PA_TO_10PA: f64 = 0.1;

    // Formule (11.85): q_v1;lea;ref = q_v10;lea;ref · (1/10)^n_lea · A_g · 3,6.
    q_v10_lea_ref_dm3_per_s_per_m2
        * PRESSURE_RATIO_1PA_TO_10PA.powf(crate::tables::FLOW_EXPONENT_LEAKAGE)
        * gross_floor_area_m2
        * DM3_PER_S_TO_M3_PER_H
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BuildingLeakageType;
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

    // --- NTA 8800 §11.2.5 q_v1;lea;ref + q_v10;lea;ref forfait ---------------

    #[test]
    fn q_v10_lea_ref_forfait_matches_hand_calc() {
        // Formule (11.86): q_v10;lea;ref = f_type · f_y · q_v10;spec;reken.
        // Grondgebonden tussenwoning met kap, bouwjaar 2015:
        //   q_v10;spec;reken = 1,0 (tabel 11.14 grondgebonden categorie)
        //   f_type           = 1,0 (tabel 11.14 tussenligging met kap)
        //   f_y              = 0,7 (tabel 11.13, j ≥ 2010)
        // → q_v10;lea;ref = 1,0 · 0,7 · 1,0 = 0,7 dm³/(s·m²)
        let q = q_v10_lea_ref_forfait(BuildingLeakageType::GroundBoundTerracedPitchedRoof, 2015);
        assert_relative_eq!(q, 0.7, epsilon = 1e-12);
    }

    #[test]
    fn q_v10_lea_ref_forfait_old_detached_house() {
        // Vrijstaande woning met hellend dak, bouwjaar 1965 (pre-1970):
        //   q_v10;spec;reken = 1,0 (grondgebonden categorie)
        //   f_type           = 1,4 (vrijstaand gebouw, hellend dak)
        //   f_y              = 3,0 (tabel 11.13, j < 1970)
        // → q_v10;lea;ref = 1,4 · 3,0 · 1,0 = 4,2 dm³/(s·m²)
        let q = q_v10_lea_ref_forfait(BuildingLeakageType::GroundBoundDetachedPitchedRoof, 1965);
        assert_relative_eq!(q, 4.2, epsilon = 1e-12);
    }

    #[test]
    fn q_v10_lea_ref_forfait_multistorey_flat() {
        // Flatwoning tussenligging onderste verdieping, bouwjaar 1985:
        //   q_v10;spec;reken = 0,5 (meerlaagse categorie)
        //   f_type           = 1,0 (tussenligging onderste/tussenverdieping)
        //   f_y              = 2,0 (tabel 11.13, 1980 ≤ j < 1990)
        // → q_v10;lea;ref = 1,0 · 2,0 · 0,5 = 1,0 dm³/(s·m²)
        let q = q_v10_lea_ref_forfait(BuildingLeakageType::MultiStoreyLowerTerraced, 1985);
        assert_relative_eq!(q, 1.0, epsilon = 1e-12);
    }

    #[test]
    fn q_v1_lea_ref_matches_norm_formula_11_85() {
        // Formule (11.85): q_v1;lea;ref = q_v10;lea;ref · (1/10)^0,67 · A_g · 3,6.
        // q_v10;lea;ref = 0,7 dm³/(s·m²), A_g = 120 m².
        //   (1/10)^0,67 = 0,1^0,67 ≈ 0,2137962...
        //   q_v1 = 0,7 · 0,2137962 · 120 · 3,6 ≈ 64,6520 m³/h
        let q = q_v1_lea_ref(0.7, 120.0);
        let expected = 0.7 * 0.1_f64.powf(0.67) * 120.0 * 3.6;
        assert_relative_eq!(q, expected, epsilon = 1e-9);
        // Numerieke verankering: ≈ 64,652 m³/h.
        assert_relative_eq!(q, 64.652_032, epsilon = 1e-3);
    }

    #[test]
    fn q_v1_lea_ref_scales_linearly_with_area_and_permeability() {
        // q_v1;lea;ref is lineair in zowel A_g als q_v10;lea;ref.
        let base = q_v1_lea_ref(0.7, 100.0);
        assert_relative_eq!(q_v1_lea_ref(0.7, 200.0), 2.0 * base, epsilon = 1e-9);
        assert_relative_eq!(q_v1_lea_ref(1.4, 100.0), 2.0 * base, epsilon = 1e-9);
    }

    #[test]
    fn q_v1_lea_ref_full_forfait_chain() {
        // Volledige norm-keten (11.86) → (11.85) voor een nieuwbouw-
        // tussenwoning met kap (2015), A_g = 150 m²:
        //   q_v10;lea;ref = 1,0 · 0,7 · 1,0 = 0,7 dm³/(s·m²)   [11.86]
        //   q_v1;lea;ref  = 0,7 · 0,1^0,67 · 150 · 3,6          [11.85]
        let q_v10 = q_v10_lea_ref_forfait(BuildingLeakageType::GroundBoundTerracedPitchedRoof, 2015);
        let q_v1 = q_v1_lea_ref(q_v10, 150.0);
        let expected = 0.7 * 0.1_f64.powf(0.67) * 150.0 * 3.6;
        assert_relative_eq!(q_v1, expected, epsilon = 1e-9);
    }

    #[test]
    fn q_v1_lea_ref_zero_inputs_give_zero() {
        assert_relative_eq!(q_v1_lea_ref(0.0, 120.0), 0.0, epsilon = 1e-12);
        assert_relative_eq!(q_v1_lea_ref(0.7, 0.0), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn q_v1_lea_ref_below_q_v10_per_m2_basis() {
        // Sanity: het 1 Pa-debiet moet kleiner zijn dan het naïeve 10 Pa-
        // debiet zonder drukcorrectie (q_v10 · A_g · 3,6), want (1/10)^0,67 < 1.
        let q_v10 = 0.7;
        let area = 120.0;
        let naive_10pa = q_v10 * area * 3.6;
        assert!(q_v1_lea_ref(q_v10, area) < naive_10pa);
    }
}
