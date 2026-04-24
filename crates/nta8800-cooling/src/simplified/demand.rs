//! Formule (AA.5) — transmissie door ondoorzichtige delen; en
//! formule (AA.8) — maatgevende koelbehoefte q_C;zi van de rekenzone.
//!
//! AA.2.2.4 (zoninstraling via transparante delen, formule (AA.6a/b)) en
//! AA.2.2.5 (transmissie via glas, formule (AA.7)) worden in deze V1 als
//! **caller-supplied inputs** behandeld: de raam-lastsommatie vereist
//! ram-level data (g-waarde, U, F_sh, F_C, I_sol uit tabel AA.3) die in het
//! huidige model-laag nog niet met tijdstip-informatie beschikbaar zijn.
//! In V2 volgt de integratie met `nta8800-model::geometry::Window` +
//! tabel AA.3 uit `nta8800-tables`.

use crate::errors::{CoolingCalcResult, CoolingError};

/// Bouwjaarklasse voor tabel AA.2 (factor thermische isolatie ondoorzichtige
/// delen, f_iso in W/m²).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BouwjaarKlasse {
    /// Bouwjaar ≤ 1975 — f_iso = 17 W/m².
    Tot1975,
    /// 1975 ≤ bouwjaar < 1992 — f_iso = 10 W/m².
    Van1975Tot1992,
    /// 1992 ≤ bouwjaar < 2015 — f_iso = 3,2 W/m².
    Van1992Tot2015,
    /// bouwjaar ≥ 2015 — f_iso = 2,2 W/m².
    Van2015,
}

impl BouwjaarKlasse {
    /// Leid de bouwjaarklasse af uit een bouwjaar.
    #[must_use]
    pub const fn from_year(year: u32) -> Self {
        if year < 1975 {
            Self::Tot1975
        } else if year < 1992 {
            Self::Van1975Tot1992
        } else if year < 2015 {
            Self::Van1992Tot2015
        } else {
            Self::Van2015
        }
    }

    /// Factor f_iso uit tabel AA.2 in W/m².
    #[must_use]
    pub const fn f_iso(self) -> f64 {
        match self {
            Self::Tot1975 => 17.0,
            Self::Van1975Tot1992 => 10.0,
            Self::Van1992Tot2015 => 3.2,
            Self::Van2015 => 2.2,
        }
    }
}

/// Formule (AA.5) — koellast door transmissie door ondoorzichtige delen.
///
/// `P_tr;ntr;zi = f_iso · A_in;zi`  [W]
///
/// # Parameters
/// - `klasse` — bouwjaarklasse voor f_iso (tabel AA.2).
/// - `opaque_area_m2` — binnenwerkse oppervlakte ondoorzichtig deel
///   buitenwand + dak van de rekenzone.
#[must_use]
pub fn koellast_transmissie_ondoorzichtig(klasse: BouwjaarKlasse, opaque_area_m2: f64) -> f64 {
    klasse.f_iso() * opaque_area_m2
}

/// Invoer voor de maatgevende koelbehoefte-berekening (AA.8).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KoelbehoefteComponenten {
    /// P_int;calc;zi — interne warmtelast (AA.1/AA.2/AA.3), in W.
    pub p_int_calc_w: f64,
    /// P_V;zi — koellast door buitenluchttoetreding (AA.4), in W.
    pub p_outdoor_w: f64,
    /// P_tr;ntr;zi — transmissie door ondoorzichtige delen (AA.5), in W.
    pub p_tr_ntr_w: f64,
    /// P_sol;zi — zoninstraling via transparante delen (AA.6a/b), in W.
    pub p_sol_w: f64,
    /// P_gl;zi — transmissie via transparante delen (AA.7), in W.
    pub p_gl_w: f64,
}

/// Formule (AA.8) — maatgevende koelbehoefte rekenzone.
///
/// `q_C;zi = (P_int;calc + P_V + P_tr;ntr + P_sol + P_gl) / A_g;vr;zi`  [W/m²]
///
/// # Errors
/// Geeft [`CoolingError::InvalidFloorArea`] als de totale
/// verblijfsruimte-oppervlakte ≤ 0 is.
pub fn maatgevende_koelbehoefte(
    componenten: &KoelbehoefteComponenten,
    total_verblijfsruimte_m2: f64,
) -> CoolingCalcResult<f64> {
    if !total_verblijfsruimte_m2.is_finite() || total_verblijfsruimte_m2 <= 0.0 {
        return Err(CoolingError::InvalidFloorArea {
            area_m2: total_verblijfsruimte_m2,
        });
    }
    let totaal = componenten.p_int_calc_w
        + componenten.p_outdoor_w
        + componenten.p_tr_ntr_w
        + componenten.p_sol_w
        + componenten.p_gl_w;
    Ok(totaal / total_verblijfsruimte_m2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn bouwjaar_klasse_grenzen() {
        assert_eq!(BouwjaarKlasse::from_year(1974), BouwjaarKlasse::Tot1975);
        assert_eq!(
            BouwjaarKlasse::from_year(1975),
            BouwjaarKlasse::Van1975Tot1992
        );
        assert_eq!(
            BouwjaarKlasse::from_year(1991),
            BouwjaarKlasse::Van1975Tot1992
        );
        assert_eq!(
            BouwjaarKlasse::from_year(1992),
            BouwjaarKlasse::Van1992Tot2015
        );
        assert_eq!(
            BouwjaarKlasse::from_year(2014),
            BouwjaarKlasse::Van1992Tot2015
        );
        assert_eq!(BouwjaarKlasse::from_year(2015), BouwjaarKlasse::Van2015);
        assert_eq!(BouwjaarKlasse::from_year(2025), BouwjaarKlasse::Van2015);
    }

    #[test]
    fn tabel_aa2_waarden() {
        assert_abs_diff_eq!(BouwjaarKlasse::Tot1975.f_iso(), 17.0, epsilon = 1e-9);
        assert_abs_diff_eq!(BouwjaarKlasse::Van1975Tot1992.f_iso(), 10.0, epsilon = 1e-9);
        assert_abs_diff_eq!(BouwjaarKlasse::Van1992Tot2015.f_iso(), 3.2, epsilon = 1e-9);
        assert_abs_diff_eq!(BouwjaarKlasse::Van2015.f_iso(), 2.2, epsilon = 1e-9);
    }

    #[test]
    fn aa5_transmissie_nieuwbouw() {
        // 100 m² ondoorzichtig, nieuwbouw → 2,2 × 100 = 220 W
        let p = koellast_transmissie_ondoorzichtig(BouwjaarKlasse::Van2015, 100.0);
        assert_abs_diff_eq!(p, 220.0, epsilon = 1e-9);
    }

    #[test]
    fn aa5_transmissie_oudbouw() {
        // 100 m² ondoorzichtig, ≤1975 → 17 × 100 = 1 700 W (7,7× hoger)
        let p = koellast_transmissie_ondoorzichtig(BouwjaarKlasse::Tot1975, 100.0);
        assert_abs_diff_eq!(p, 1_700.0, epsilon = 1e-9);
    }

    #[test]
    fn aa8_maatgevende_koelbehoefte_totaal() {
        // Voorbeeld woning 120 m² totaal, componenten sommeren tot 6000 W
        // → q_C = 6000/120 = 50 W/m²
        let componenten = KoelbehoefteComponenten {
            p_int_calc_w: 540.0,
            p_outdoor_w: 554.0,
            p_tr_ntr_w: 220.0,
            p_sol_w: 4_400.0,
            p_gl_w: 286.0,
        };
        let q = maatgevende_koelbehoefte(&componenten, 120.0).unwrap();
        let expected = (540.0 + 554.0 + 220.0 + 4_400.0 + 286.0) / 120.0;
        assert_abs_diff_eq!(q, expected, epsilon = 1e-9);
    }

    #[test]
    fn aa8_rejects_zero_area() {
        let componenten = KoelbehoefteComponenten {
            p_int_calc_w: 540.0,
            p_outdoor_w: 0.0,
            p_tr_ntr_w: 0.0,
            p_sol_w: 0.0,
            p_gl_w: 0.0,
        };
        let err = maatgevende_koelbehoefte(&componenten, 0.0).unwrap_err();
        assert!(matches!(err, CoolingError::InvalidFloorArea { .. }));
    }
}
