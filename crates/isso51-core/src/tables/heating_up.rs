//! Heating-up allowance (opwarmtoeslag) lookup table.
//! ISSO 51:2023 §2.5.8 Tabel 2.10 + Afb. 2.6 beslis-schema.
//!
//! De 2023-norm vervangt het oude `f_RH × ΣA_metselwerk`-model (NEN-EN 12831 /
//! ISSO 51:2017) door `Φ_hu = P × A_g`, met `P` [W/m² vloeroppervlak] uit
//! Tabel 2.10. `P` wordt geïndexeerd op drie ingangen:
//! 1. **afkoeling** [K] na 8 uur nachtverlaging (nieuwbouw: 2 K, resp. 1 K bij Ū≤0,5);
//! 2. **gebouwzwaarte** (`c_eff ≤ 70 Wh/K` → ZL+L+M, anders Z);
//! 3. **opwarmtijd** [h] (richtwaarde 2 h, instelbaar).
//!
//! Bron: ISSO 51:2023 p.45 Tabel 2.10 (visueel geverifieerd, 50 cellen) —
//! zie `audit-reports/08-isso51-opwarmtoeslag-ref.md`.

use crate::model::enums::ThermalMass;

/// Afkoeling-kolommen van Tabel 2.10 (graden verlaging na 8 uur nachtverlaging).
/// Volgorde komt exact overeen met de index-2 van [`P_TABLE`].
const COOLING_LEVELS_K: [f64; 5] = [1.0, 1.5, 2.0, 2.5, 3.0];

/// Opwarmtijd-rijen van Tabel 2.10 [h]. Volgorde komt exact overeen met de
/// index-0 van [`P_TABLE`].
const WARMUP_TIMES_H: [f64; 5] = [0.5, 1.0, 2.0, 3.0, 4.0];

/// Tabel 2.10 — specifieke toeslag P [W/m² vloeroppervlak], 8 uur nachtverlaging.
///
/// Indexering: `P_TABLE[opwarmtijd][afkoeling][zwaarte]` waarbij
/// - opwarmtijd-index ↔ [`WARMUP_TIMES_H`] (0,5 / 1 / 2 / 3 / 4 h),
/// - afkoeling-index ↔ [`COOLING_LEVELS_K`] (1 / 1,5 / 2 / 2,5 / 3 K),
/// - zwaarte-index 0 = ZL+L+M (lichter), 1 = Z (zwaar).
///
/// Waarden 1:1 overgenomen uit de visueel geverifieerde tabel (p.45).
/// Monotonie (sanity): P ↑ met afkoeling, Z > ZL+L+M, P ↓ met langere opwarmtijd.
const P_TABLE: [[[f64; 2]; 5]; 5] = [
    // opwarmtijd 0,5 h
    [[14.0, 18.0], [22.0, 27.0], [29.0, 35.0], [37.0, 44.0], [44.0, 53.0]],
    // opwarmtijd 1 h
    [[10.0, 14.0], [16.0, 21.0], [21.0, 28.0], [27.0, 36.0], [32.0, 43.0]],
    // opwarmtijd 2 h
    [[7.0, 11.0], [10.0, 17.0], [13.0, 22.0], [17.0, 28.0], [21.0, 33.0]],
    // opwarmtijd 3 h
    [[5.0, 10.0], [8.0, 15.0], [10.0, 19.0], [13.0, 23.0], [15.0, 27.0]],
    // opwarmtijd 4 h
    [[4.0, 9.0], [6.0, 13.0], [8.0, 17.0], [11.0, 21.0], [13.0, 25.0]],
];

/// Zoek de dichtstbijzijnde tabel-index op in een oplopende as.
///
/// Tabel 2.10 is een discreet schema; de norm interpoleert niet. Voor een
/// invoerwaarde tussen twee kolommen kiezen we de dichtstbijzijnde
/// gerasterde waarde (ronden naar het naburige rooster), met een lichte
/// voorkeur voor de hogere index bij exact midden (conservatief: hogere P).
fn nearest_index(axis: &[f64], value: f64) -> usize {
    let mut best = 0usize;
    let mut best_dist = f64::MAX;
    for (i, &a) in axis.iter().enumerate() {
        let dist = (a - value).abs();
        // `<=` zodat bij gelijke afstand de hogere index (conservatiever) wint.
        if dist <= best_dist {
            best_dist = dist;
            best = i;
        }
    }
    best
}

/// Bepaal de specifieke toeslag P [W/m² vloeroppervlak] uit Tabel 2.10.
///
/// ISSO 51:2023 §2.5.8, Tabel 2.10 (p.45).
///
/// # Arguments
/// * `cooling_k` - Afkoeling [K] na nachtverlaging (1 / 1,5 / 2 / 2,5 / 3).
///   Buiten bereik wordt geklemd op de dichtstbijzijnde tabelkolom.
/// * `mass` - Gebouwzwaarte (ZL+L+M of Z), bepaald uit `c_eff`.
/// * `warmup_time_h` - Toegestane opwarmtijd [h] (0,5 / 1 / 2 / 3 / 4).
///   Bij `< 0,01` (geen nachtverlaging) wordt 0 teruggegeven.
///
/// # Returns
/// P [W/m² vloeroppervlak].
pub fn specific_heating_up_allowance(cooling_k: f64, mass: ThermalMass, warmup_time_h: f64) -> f64 {
    // Geen nachtverlaging / opwarmtijd → geen opwarmtoeslag.
    if warmup_time_h < 0.01 {
        return 0.0;
    }

    let warmup_idx = nearest_index(&WARMUP_TIMES_H, warmup_time_h);
    let cooling_idx = nearest_index(&COOLING_LEVELS_K, cooling_k);
    let mass_idx = match mass {
        ThermalMass::Light => 0,
        ThermalMass::Heavy => 1,
    };

    P_TABLE[warmup_idx][cooling_idx][mass_idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_2_10_newbuild_2k_examples() {
        // Nieuwbouw, afkoeling 2 K, opwarmtijd 2 h (default richtwaarde).
        // Tabel 2.10: 2K/ZL+L+M/2h = 13, 2K/Z/2h = 22.
        assert_eq!(
            specific_heating_up_allowance(2.0, ThermalMass::Light, 2.0),
            13.0
        );
        assert_eq!(
            specific_heating_up_allowance(2.0, ThermalMass::Heavy, 2.0),
            22.0
        );
    }

    #[test]
    fn test_table_2_10_newbuild_1k_examples() {
        // Goed geïsoleerde nieuwbouw (Ū≤0,5), afkoeling 1 K, opwarmtijd 2 h.
        // Tabel 2.10: 1K/ZL+L+M/2h = 7, 1K/Z/2h = 11.
        assert_eq!(
            specific_heating_up_allowance(1.0, ThermalMass::Light, 2.0),
            7.0
        );
        assert_eq!(
            specific_heating_up_allowance(1.0, ThermalMass::Heavy, 2.0),
            11.0
        );
    }

    #[test]
    fn test_table_2_10_corners() {
        // Hoekwaarden: kortste opwarmtijd + grootste afkoeling = grootste P.
        assert_eq!(
            specific_heating_up_allowance(3.0, ThermalMass::Heavy, 0.5),
            53.0
        );
        // Langste opwarmtijd + kleinste afkoeling + licht = kleinste P.
        assert_eq!(
            specific_heating_up_allowance(1.0, ThermalMass::Light, 4.0),
            4.0
        );
    }

    #[test]
    fn test_no_warmup_returns_zero() {
        assert_eq!(
            specific_heating_up_allowance(2.0, ThermalMass::Heavy, 0.0),
            0.0
        );
    }

    #[test]
    fn test_table_2_10_monotonicity() {
        // P stijgt met afkoeling (bij vaste opwarmtijd + zwaarte).
        for &m in &[ThermalMass::Light, ThermalMass::Heavy] {
            let mut prev = 0.0;
            for &c in &COOLING_LEVELS_K {
                let p = specific_heating_up_allowance(c, m, 2.0);
                assert!(p >= prev, "P moet niet dalen met afkoeling: {p} < {prev}");
                prev = p;
            }
        }
        // Z ≥ ZL+L+M bij gelijke afkoeling + opwarmtijd.
        for &c in &COOLING_LEVELS_K {
            let light = specific_heating_up_allowance(c, ThermalMass::Light, 2.0);
            let heavy = specific_heating_up_allowance(c, ThermalMass::Heavy, 2.0);
            assert!(heavy >= light, "Z ({heavy}) moet ≥ ZL+L+M ({light}) zijn");
        }
        // P daalt met langere opwarmtijd.
        let mut prev = f64::MAX;
        for &w in &WARMUP_TIMES_H {
            let p = specific_heating_up_allowance(2.0, ThermalMass::Heavy, w);
            assert!(p <= prev, "P moet niet stijgen met opwarmtijd: {p} > {prev}");
            prev = p;
        }
    }
}
