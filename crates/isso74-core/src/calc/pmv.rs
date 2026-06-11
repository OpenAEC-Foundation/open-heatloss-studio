//! Fanger PMV / PPD model (ISO 7730) and the ISSO 74 GTO weighting factor.
//!
//! # Toets-laag aanname (ISSO 74 §A)
//!
//! The assessment layer only knows the operative temperature θ_o per hour. The
//! full Fanger model needs air temperature, mean radiant temperature, air
//! velocity, humidity, clothing and metabolism. We therefore assume:
//!
//! * `t_air ≈ t_mrt ≈ θ_o` (operative temperature substituted for both).
//! * relative humidity `RH = 50%` (configurable),
//! * relative air velocity `v_ar = 0,1 m/s` (configurable),
//! * clothing `clo = 0,7` (summer, PMV ≥ 0) / `0,9` (winter, PMV < 0) — the
//!   summer/winter split is resolved by sign of the resulting PMV (see
//!   [`pmv_for_operative`]),
//! * metabolism `M = 1,2 met` (= 70 W/m²), external work `W = 0`.
//!
//! These are **toets-laag aannames, geen volledige comfortsimulatie**. They are
//! surfaced explicitly in the result so reviewers see them.
//!
//! # GTO weegfactor (ISSO 74 Bijlage A.2)
//!
//! ```text
//! PPD = 100 − 95·exp(−(0,03353·PMV⁴ + 0,2179·PMV²))
//! wf  = PPD(PMV) / 10
//! ```
//! Oracle (verplichte test): PMV 0,5/0,6/0,7/0,8 → wf ≈ 1,02/1,26/1,53/1,85.

use crate::model::PmvParams;

/// Predicted Percentage of Dissatisfied from PMV (ISO 7730 / Fanger).
pub fn ppd(pmv: f64) -> f64 {
    100.0 - 95.0 * (-(0.03353 * pmv.powi(4) + 0.2179 * pmv.powi(2))).exp()
}

/// GTO weighting factor wf = PPD(PMV) / 10 (ISSO 74 Bijlage A.2).
pub fn weighting_factor(pmv: f64) -> f64 {
    ppd(pmv) / 10.0
}

/// Full Fanger PMV (ISO 7730 §A) with iterative clothing-surface temperature.
///
/// # Arguments
/// * `t_air` — air temperature [°C]
/// * `t_mrt` — mean radiant temperature [°C]
/// * `v_ar`  — relative air velocity [m/s]
/// * `rh`    — relative humidity [%]
/// * `met`   — metabolic rate [met] (1 met = 58,15 W/m²)
/// * `clo`   — clothing insulation [clo] (1 clo = 0,155 m²·K/W)
/// * `wme`   — external work [met]
#[allow(clippy::too_many_arguments)]
pub fn fanger_pmv(t_air: f64, t_mrt: f64, v_ar: f64, rh: f64, met: f64, clo: f64, wme: f64) -> f64 {
    let m = met * 58.15; // W/m²
    let w = wme * 58.15; // W/m²
    let mw = m - w;

    let icl = 0.155 * clo; // m²·K/W
    let fcl = if icl <= 0.078 {
        1.0 + 1.29 * icl
    } else {
        1.05 + 0.645 * icl
    };

    // Water-vapour partial pressure [Pa] from RH and Antoine-ish saturation.
    let pa = rh * 10.0 * (16.6536 - 4030.183 / (t_air + 235.0)).exp();

    let hcf = 12.1 * v_ar.sqrt();
    let taa = t_air + 273.0;
    let tra = t_mrt + 273.0;

    // Iterative solution for clothing surface temperature t_cl.
    let mut tcla = taa + (35.5 - t_air) / (3.96 * fcl);
    let p1 = icl * fcl;
    let p2 = p1 * 3.96;
    let p3 = p1 * 100.0;
    let p4 = p1 * taa;
    let p5 = 308.7 - 0.028 * mw + p2 * (tra / 100.0).powi(4);

    let mut xn = tcla / 100.0;
    let mut xf = xn;
    let mut hc;
    let mut n = 0;
    loop {
        xf = (xf + xn) / 2.0;
        let hcn = 2.38 * (100.0 * xf - taa).abs().powf(0.25);
        hc = if hcf > hcn { hcf } else { hcn };
        xn = (p5 + p4 * hc - p2 * xf.powi(4)) / (100.0 + p3 * hc);
        n += 1;
        if (xn - xf).abs() <= 0.00015 || n > 150 {
            break;
        }
    }
    tcla = 100.0 * xn - 273.0;

    // Heat-loss components.
    let hl1 = 3.05 * 0.001 * (5733.0 - 6.99 * mw - pa);
    let hl2 = if mw > 58.15 { 0.42 * (mw - 58.15) } else { 0.0 };
    let hl3 = 1.7 * 0.00001 * m * (5867.0 - pa);
    let hl4 = 0.0014 * m * (34.0 - t_air);
    let hl5 = 3.96 * fcl * ((xn).powi(4) - (tra / 100.0).powi(4));
    let hl6 = fcl * hc * (tcla - t_air);

    let ts = 0.303 * (-0.036 * m).exp() + 0.028;
    ts * (mw - hl1 - hl2 - hl3 - hl4 - hl5 - hl6)
}

/// PMV from an operative temperature only, using the toets-laag assumptions.
///
/// The clothing value is selected by season: a first pass with `clo_summer`
/// gives a provisional PMV; if that PMV is negative (cooler/winter conditions)
/// we recompute with `clo_winter`. This keeps the clo/season mapping internally
/// consistent without a separate season flag.
pub fn pmv_for_operative(theta_o: f64, params: &PmvParams) -> f64 {
    let provisional = fanger_pmv(
        theta_o,
        theta_o,
        params.air_velocity_m_s,
        params.relative_humidity_pct,
        params.metabolic_rate_met,
        params.clo_summer,
        params.external_work_met,
    );
    if provisional >= 0.0 {
        provisional
    } else {
        fanger_pmv(
            theta_o,
            theta_o,
            params.air_velocity_m_s,
            params.relative_humidity_pct,
            params.metabolic_rate_met,
            params.clo_winter,
            params.external_work_met,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// VERPLICHTE oracle (ISSO 74 Bijlage A.2): wf-ankerpunten.
    #[test]
    fn weighting_factor_anchor_points() {
        assert_relative_eq!(weighting_factor(0.5), 1.02, epsilon = 0.03);
        assert_relative_eq!(weighting_factor(0.6), 1.26, epsilon = 0.03);
        assert_relative_eq!(weighting_factor(0.7), 1.53, epsilon = 0.03);
        assert_relative_eq!(weighting_factor(0.8), 1.85, epsilon = 0.03);
    }

    #[test]
    fn ppd_is_5_pct_at_neutral() {
        // PMV = 0 → PPD = 5% (Fanger minimum).
        assert_relative_eq!(ppd(0.0), 5.0, epsilon = 1e-9);
    }

    #[test]
    fn ppd_symmetric() {
        assert_relative_eq!(ppd(0.7), ppd(-0.7), epsilon = 1e-9);
    }

    /// Cross-check Fanger PMV against an ISO 7730 Annex D reference case.
    /// Annex D row: t_a = t_r = 22 °C, v_ar = 0,1, RH = 60%, M = 1,2, clo = 0,5
    /// → PMV ≈ −0,75 (ISO 7730:2005 Table D.1, ±0,05 tolerance for rounding).
    #[test]
    fn fanger_matches_iso7730_annex_d() {
        let pmv = fanger_pmv(22.0, 22.0, 0.1, 60.0, 1.2, 0.5, 0.0);
        assert_relative_eq!(pmv, -0.75, epsilon = 0.06);
    }

    /// Warm cross-check: t_a = t_r = 27 °C, v = 0,1, RH = 60%, M = 1,2,
    /// clo = 0,5. The canonical ISO 7730 Annex D reference algorithm yields
    /// PMV ≈ +0,77 for this case (slightly warm). This pins the sign and
    /// magnitude of the warm branch against the same Annex D code path that
    /// reproduces the −0,75 cold case above.
    #[test]
    fn fanger_matches_iso7730_annex_d_warm() {
        let pmv = fanger_pmv(27.0, 27.0, 0.1, 60.0, 1.2, 0.5, 0.0);
        assert_relative_eq!(pmv, 0.77, epsilon = 0.05);
    }

    #[test]
    fn operative_clo_switch_by_season() {
        let p = PmvParams::default();
        // Warm room → summer clo, positive PMV.
        let warm = pmv_for_operative(28.0, &p);
        assert!(warm > 0.0);
        // Cool room → winter clo, negative PMV.
        let cool = pmv_for_operative(18.0, &p);
        assert!(cool < 0.0);
    }
}
