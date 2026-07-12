//! NTA 8800:2025+C1:2026 Tabel 17.2 — maandgemiddelde totale opvallende
//! zonnestraling `I_sol;mi` [W/m²] per hellingshoek β en oriëntatie γ voor
//! referentieklimaat De Bilt (grondreflectie ρ = 0,2).
//!
//! ## Waarom hier en niet als "correctiefactor"
//!
//! De PV-opbrengstformule (16.2 + 16.3, PDF p. 677-678) kent **geen** aparte
//! tilt/azimut-*correctiefactor*. De hellingshoek- en oriëntatie-afhankelijkheid
//! zit volledig in de keuze van `I_sol;mi` uit **Tabel 17.2** (PDF p. 690-693;
//! zie OPMERKING 2 bij formule 16.3, p. 678). Deze module transcribeert die
//! tabel en levert de interpolatie-/selectieregels die de norm op p. 693
//! voorschrijft:
//!
//! - **Tussenliggende oriëntatie** → de waarde bij de *dichtstbijzijnde*
//!   oriëntatie; ligt de oriëntatie exact tussen twee tabelkolommen, dan de
//!   *hoogste* naastliggende waarde.
//! - **Tussenliggende hellingshoek** → *lineair interpoleren* tussen de
//!   tabelwaarden.
//!
//! De oude V1-benadering (`f = cos(β − 35°)·cos((γ − 180°)/2)`) week hier
//! systematisch van af en klapte west/noord door de `.max(0.0)`-clamp op 0 —
//! zie [`crate::calc`].
//!
//! Bron-provenance: NTA 8800:2025+C1:2026, Tabel 17.2, PDF p. 690-693
//! (β = 0°, 30°, 45°, 60°, 90°, 135°, 180°; γ = Z/ZW/W/NW/N/NO/O/ZO). De
//! β = 0°- en β = 180°-rijen zijn oriëntatie-onafhankelijk (horizontaal
//! omhoog resp. omlaag).

use nta8800_model::time::Month;

/// Hellingshoeken β [°] waarvoor Tabel 17.2 waarden geeft, oplopend.
const TILTS_DEG: [f64; 7] = [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0];

/// Oriëntatie-kolommen van Tabel 17.2 in tabelvolgorde, met azimut γ in graden
/// (0/360 = noord, kloksgewijs; 90 = oost, 180 = zuid, 270 = west).
const ORIENTATION_DEG: [f64; 8] = [180.0, 225.0, 270.0, 315.0, 360.0, 45.0, 90.0, 135.0];
//                                 Z      ZW     W      NW     N      NO    O     ZO

/// β = 0° — horizontaal omhoog (oriëntatie-onafhankelijke "–"-kolom).
const HORIZONTAL: [f64; 12] = [
    28.0, 49.3, 96.6, 160.5, 197.0, 209.3, 191.0, 177.2, 123.9, 73.2, 34.3, 21.0,
];

/// β = 180° — horizontaal omlaag (oriëntatie-onafhankelijke "–"-kolom).
const DOWN: [f64; 12] = [
    5.6, 9.8, 19.3, 32.1, 39.3, 41.8, 38.2, 35.3, 24.7, 14.6, 6.9, 4.2,
];

/// β = 30°, `[oriëntatie][maand]` (oriëntatie-index conform [`ORIENTATION_DEG`]).
const ISOL_30: [[f64; 12]; 8] = [
    // Z (180°)
    [50.5, 69.1, 122.5, 189.5, 211.1, 211.2, 196.1, 197.9, 154.0, 102.4, 54.8, 38.3],
    // ZW (225°)
    [44.4, 61.2, 109.3, 174.5, 201.5, 210.7, 193.2, 198.3, 146.2, 91.5, 47.7, 32.6],
    // W (270°)
    [29.0, 46.2, 87.7, 146.5, 179.9, 199.4, 180.2, 178.4, 121.1, 68.8, 32.9, 20.6],
    // NW (315°)
    [16.2, 32.9, 66.7, 115.6, 155.8, 180.6, 162.1, 147.6, 91.6, 47.3, 20.5, 12.5],
    // N (360°)
    [14.9, 27.2, 56.4, 104.6, 148.5, 171.0, 153.0, 125.8, 73.7, 36.3, 18.6, 12.2],
    // NO (45°)
    [15.8, 34.5, 72.8, 125.1, 160.6, 173.0, 156.9, 127.5, 86.5, 48.9, 20.9, 12.5],
    // O (90°)
    [26.9, 49.4, 97.6, 158.9, 186.3, 189.7, 175.0, 152.8, 113.7, 71.6, 33.8, 21.2],
    // ZO (135°)
    [42.2, 63.7, 117.7, 184.1, 206.3, 204.4, 190.0, 179.3, 140.1, 93.6, 48.6, 33.1],
];

/// β = 45°, `[oriëntatie][maand]`.
const ISOL_45: [[f64; 12]; 8] = [
    [57.9, 74.1, 126.6, 189.7, 202.7, 197.3, 185.0, 193.5, 157.6, 109.4, 61.0, 44.1],
    [49.4, 63.2, 109.1, 171.0, 191.1, 199.3, 182.5, 194.9, 147.0, 94.2, 51.1, 36.1],
    [28.7, 44.0, 82.0, 136.7, 164.4, 186.2, 166.8, 169.8, 115.3, 64.8, 31.3, 19.9],
    [14.9, 29.2, 56.6, 96.5, 128.7, 156.3, 139.0, 127.2, 78.0, 40.2, 18.5, 11.7],
    [14.3, 25.9, 44.3, 70.0, 113.6, 139.6, 123.5, 91.5, 52.9, 33.5, 17.8, 11.7],
    [14.5, 30.4, 63.1, 107.1, 134.5, 145.9, 132.7, 102.9, 72.2, 41.4, 18.8, 11.7],
    [26.2, 47.9, 94.2, 152.2, 172.0, 173.3, 160.4, 137.9, 106.2, 68.4, 32.4, 20.5],
    [46.3, 66.5, 120.2, 183.5, 197.3, 190.7, 179.1, 171.0, 139.2, 97.2, 52.2, 36.7],
];

/// β = 60°, `[oriëntatie][maand]`.
const ISOL_60: [[f64; 12]; 8] = [
    [62.2, 75.4, 124.3, 180.2, 184.5, 175.1, 165.9, 179.7, 153.3, 110.7, 63.9, 47.4],
    [51.8, 62.1, 103.9, 160.4, 173.4, 180.9, 165.4, 182.9, 141.5, 92.6, 51.8, 37.6],
    [27.8, 41.1, 74.8, 125.1, 146.3, 169.1, 150.6, 156.9, 107.2, 59.9, 28.9, 19.0],
    [13.8, 26.4, 49.6, 83.1, 107.5, 134.1, 119.2, 110.2, 68.6, 35.9, 17.0, 10.9],
    [13.4, 24.1, 41.5, 57.8, 78.5, 102.9, 90.4, 68.0, 48.6, 31.5, 16.6, 10.9],
    [13.5, 27.3, 56.3, 93.9, 113.2, 123.3, 112.3, 85.8, 62.3, 36.6, 17.3, 10.9],
    [24.7, 45.4, 88.5, 142.0, 154.7, 154.5, 143.2, 122.0, 97.2, 63.5, 30.4, 19.6],
    [48.1, 66.3, 116.9, 174.2, 179.9, 170.7, 161.8, 156.4, 132.6, 96.0, 53.2, 38.4],
];

/// β = 90°, `[oriëntatie][maand]`.
const ISOL_90: [[f64; 12]; 8] = [
    [60.1, 66.7, 101.8, 135.1, 124.9, 112.7, 109.7, 128.5, 122.3, 96.2, 59.5, 46.2],
    [48.1, 52.2, 82.1, 121.9, 122.1, 127.8, 117.1, 137.1, 112.2, 76.3, 45.6, 34.9],
    [23.4, 32.8, 57.3, 96.2, 107.3, 125.7, 112.7, 120.0, 83.9, 46.7, 22.7, 15.2],
    [11.4, 20.9, 38.5, 64.1, 78.9, 97.8, 88.5, 83.1, 53.6, 28.7, 13.8, 8.9],
    [11.1, 19.5, 34.8, 49.4, 61.9, 73.0, 66.7, 55.9, 41.4, 26.4, 13.6, 8.9],
    [11.1, 21.5, 44.2, 72.9, 82.9, 92.0, 81.2, 63.9, 47.9, 29.1, 14.0, 8.9],
    [20.2, 36.5, 70.7, 112.2, 114.6, 114.8, 104.9, 89.0, 73.7, 49.8, 23.9, 15.9],
    [43.9, 56.8, 95.4, 135.8, 128.4, 118.0, 113.2, 112.4, 103.6, 80.3, 47.1, 35.8],
];

/// β = 135°, `[oriëntatie][maand]`.
const ISOL_135: [[f64; 12]; 8] = [
    [33.4, 31.5, 37.3, 39.0, 45.5, 48.3, 44.9, 41.6, 40.2, 41.6, 30.9, 26.3],
    [25.1, 24.2, 35.1, 50.7, 50.4, 52.3, 49.7, 54.3, 47.5, 33.2, 21.7, 18.3],
    [12.7, 17.3, 29.9, 49.9, 55.2, 62.4, 57.7, 59.6, 43.2, 24.7, 12.0, 8.1],
    [7.6, 13.2, 25.2, 41.8, 50.7, 57.8, 53.5, 50.2, 33.8, 19.3, 9.2, 5.8],
    [7.5, 12.9, 24.5, 38.3, 46.7, 50.6, 46.5, 42.1, 30.4, 18.6, 9.1, 5.8],
    [7.5, 13.5, 27.6, 45.5, 51.9, 55.4, 48.9, 42.9, 31.4, 19.4, 9.3, 5.8],
    [10.6, 18.6, 36.7, 57.1, 57.8, 59.9, 53.0, 47.7, 37.9, 25.4, 12.7, 8.4],
    [22.2, 26.7, 42.0, 56.3, 51.9, 51.7, 48.0, 47.5, 43.2, 35.2, 22.7, 19.0],
];

/// Cirkelvormige hoekafstand [°] tussen twee azimuts (0..=180).
fn angular_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// `I_sol;mi` [W/m²] uit Tabel 17.2 voor een *exacte* tabel-hellingshoek
/// (index in [`TILTS_DEG`]) op de gegeven oriëntatie + maand.
///
/// Voor β = 0° (horizontaal omhoog) en β = 180° (omlaag) is de waarde
/// oriëntatie-onafhankelijk. Voor de tussenliggende hellingshoeken selecteert
/// de functie de dichtstbijzijnde oriëntatiekolom; bij een exacte tie tussen
/// twee kolommen wordt de hoogste waarde genomen (norm-regel, PDF p. 693).
fn isol_at_tilt_level(tilt_idx: usize, azimuth_0_360: f64, month_idx: usize) -> f64 {
    let matrix: &[[f64; 12]; 8] = match tilt_idx {
        0 => return HORIZONTAL[month_idx],
        6 => return DOWN[month_idx],
        1 => &ISOL_30,
        2 => &ISOL_45,
        3 => &ISOL_60,
        4 => &ISOL_90,
        5 => &ISOL_135,
        _ => unreachable!("tilt_idx buiten [0, 6]"),
    };

    let min_dist = ORIENTATION_DEG
        .iter()
        .map(|&o| angular_distance(azimuth_0_360, o))
        .fold(f64::INFINITY, f64::min);

    // Alle kolommen op (numeriek) minimale afstand — bij een tie de hoogste
    // I_sol nemen (norm: "de hoogste, naastliggende waarde").
    ORIENTATION_DEG
        .iter()
        .enumerate()
        .filter(|(_, &o)| (angular_distance(azimuth_0_360, o) - min_dist).abs() < 1e-9)
        .map(|(i, _)| matrix[i][month_idx])
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Opvallende zonnestraling `I_sol;mi` [W/m²] uit Tabel 17.2 voor een
/// willekeurige hellingshoek β + azimut γ (0/360 = noord) in maand `month`.
///
/// Lineaire interpolatie tussen de tabel-hellingshoeken (PDF p. 693);
/// hellingshoeken buiten `[0°, 180°]` worden geklemd op de tabelranden.
#[must_use]
pub fn isol_w_per_m2(tilt_deg: f64, azimuth_0_360: f64, month: Month) -> f64 {
    let m = month.index();
    let t = tilt_deg.clamp(TILTS_DEG[0], TILTS_DEG[TILTS_DEG.len() - 1]);

    // Bracket-hellingshoeken vinden (TILTS_DEG is oplopend).
    let hi = TILTS_DEG.iter().position(|&b| b >= t).unwrap_or(TILTS_DEG.len() - 1);
    if hi == 0 {
        return isol_at_tilt_level(0, azimuth_0_360, m);
    }
    let lo = hi - 1;
    let (beta_lo, beta_hi) = (TILTS_DEG[lo], TILTS_DEG[hi]);
    let isol_lo = isol_at_tilt_level(lo, azimuth_0_360, m);
    let isol_hi = isol_at_tilt_level(hi, azimuth_0_360, m);
    let frac = if (beta_hi - beta_lo).abs() < f64::EPSILON {
        0.0
    } else {
        (t - beta_lo) / (beta_hi - beta_lo)
    };
    isol_lo + frac * (isol_hi - isol_lo)
}

/// Oriëntatie-/hellingsfactor t.o.v. de horizontale instraling:
/// `I_sol(β, γ, mi) / I_sol(0°, mi)` uit Tabel 17.2.
///
/// Vermenigvuldig deze factor met de horizontale maand-instraling om de
/// vlak-instraling voor een PV-systeem te krijgen. `azimuth_deg` mag in elke
/// conventie 0/360 = noord staan (ook negatief, bv. de `nta8800-pv`-conventie
/// −180..180); de functie normaliseert intern.
#[must_use]
pub fn tilt_azimuth_factor(tilt_deg: f64, azimuth_deg: f64, month: Month) -> f64 {
    let az = azimuth_deg.rem_euclid(360.0);
    let horizontal = HORIZONTAL[month.index()];
    if horizontal <= 0.0 {
        return 0.0;
    }
    isol_w_per_m2(tilt_deg, az, month) / horizontal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_factor_is_one() {
        for m in Month::all() {
            assert!((tilt_azimuth_factor(0.0, 0.0, m) - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn south_30_matches_table() {
        // Tabel 17.2, β=30°, Z, juni = 211,2 W/m²; horizontaal juni = 209,3.
        let f = tilt_azimuth_factor(30.0, 180.0, Month::Juni);
        assert!((f - 211.2 / 209.3).abs() < 1e-9, "factor {f}");
    }

    #[test]
    fn west_is_nonzero_and_reasonable() {
        // De kern-bug: west mag niet op 0 klappen. β=30°, W (270°).
        let f = tilt_azimuth_factor(30.0, 270.0, Month::Juli);
        // Tabel 17.2 β=30° W juli = 180,2 ; horizontaal juli = 191,0.
        assert!((f - 180.2 / 191.0).abs() < 1e-9, "factor {f}");
        assert!(f > 0.5);
    }

    #[test]
    fn north_is_positive() {
        // Noord conform tabelwaarde: strikt > 0 (oude cos-benadering gaf 0).
        for m in Month::all() {
            assert!(tilt_azimuth_factor(30.0, 0.0, m) > 0.0);
            assert!(tilt_azimuth_factor(15.0, 360.0, m) > 0.0);
        }
    }

    #[test]
    fn pv_crate_azimuth_convention_west_maps_correctly() {
        // nta8800-pv-conventie: west = −90°. Moet identiek zijn aan 270°.
        for m in Month::all() {
            let a = tilt_azimuth_factor(30.0, -90.0, m);
            let b = tilt_azimuth_factor(30.0, 270.0, m);
            assert!((a - b).abs() < 1e-12);
        }
    }

    #[test]
    fn tilt_interpolation_is_linear_between_levels() {
        // β=15° = midden tussen 0° (horizontaal) en 30°. Noord, januari:
        // horizontaal = 28,0 ; β=30° N = 14,9 → interp = 21,45 W/m².
        let isol = isol_w_per_m2(15.0, 360.0, Month::Januari);
        assert!((isol - (28.0 + 0.5 * (14.9 - 28.0))).abs() < 1e-9, "isol {isol}");
    }

    #[test]
    fn orientation_tie_takes_highest() {
        // Azimut 22,5° ligt exact tussen N (0/360) en NO (45). Norm: hoogste.
        // β=30° januari: N = 14,9 ; NO = 15,8 → verwacht 15,8.
        let isol = isol_w_per_m2(30.0, 22.5, Month::Januari);
        assert!((isol - 15.8).abs() < 1e-9, "isol {isol}");
    }

    #[test]
    fn south_optimum_beats_north() {
        let south = tilt_azimuth_factor(35.0, 180.0, Month::Juni);
        let north = tilt_azimuth_factor(35.0, 0.0, Month::Juni);
        assert!(south > north);
    }
}
