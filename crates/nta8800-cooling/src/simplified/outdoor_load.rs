//! Formule (AA.4) — Koellast door buitenluchttoetreding.
//!
//! `P_V;zi = ((q_v;C;eff;lea;in + q_v;C;eff;vent;in + q_v;C;SUP;eff) / 3600)
//!   · ρ_a · c_a · (θ_e − 24)`  [W]
//!
//! Met vaste fysische constanten ρ_a = 1,205 kg/m³ en c_a = 1 005 J/kgK.

/// Dichtheid van lucht ρ_a in kg/m³ (AA.4, §17 klimaatconvenant).
pub const RHO_AIR_KG_PER_M3: f64 = 1.205;

/// Specifieke warmtecapaciteit van lucht c_a in J/(kg·K).
pub const C_AIR_J_PER_KGK: f64 = 1005.0;

/// Tabel AA.1 — aan te houden buitenluchttemperatuur θ_e [°C] per tijdstip
/// (9..21 h). Index 0 = 9 h, index 12 = 21 h. Bron: NEN 5060:2018+A1:2021.
///
/// De buitentemperatuur is altijd hoger dan 24 °C (de binnentemperatuur bij
/// koel-piek), waardoor de koellast door buitenluchttoetreding nooit
/// negatief is.
pub const TABEL_AA_1_BUITENLUCHTTEMPERATUUR: [(u8, f64); 13] = [
    (9, 24.7),
    (10, 26.9),
    (11, 28.2),
    (12, 28.9),
    (13, 29.7),
    (14, 29.9),
    (15, 29.8),
    (16, 30.4),
    (17, 30.6),
    (18, 30.1),
    (19, 29.5),
    (20, 25.9),
    (21, 23.4),
];

/// Zoek de θ_e-waarde uit tabel AA.1 voor een tijdstip.
///
/// Tijdstippen buiten bereik (9..21h) geven `None`.
#[must_use]
pub fn tabel_aa1_buitentemperatuur(uur: u8) -> Option<f64> {
    TABEL_AA_1_BUITENLUCHTTEMPERATUUR
        .iter()
        .find(|(h, _)| *h == uur)
        .map(|(_, t)| *t)
}

/// Formule (AA.4) — Koellast door buitenluchttoetreding.
///
/// # Parameters
/// - `q_v_lea_m3_per_h` — infiltratie-luchtvolumestroom in juli (§11.2.1.7).
/// - `q_v_vent_m3_per_h` — natuurlijke toevoer-luchtvolumestroom (tabel 11.4).
/// - `q_v_mech_m3_per_h` — mechanische toevoer-luchtvolumestroom (tabel 11.4).
/// - `outdoor_temperature_c` — θ_e op tijdstip t_max;zi uit tabel AA.1.
///
/// Returns de koellast in W. Kan niet negatief zijn (de norm stelt dat de
/// buitentemperatuur altijd > 24 °C is op de maatgevende uren), maar als een
/// caller toch een te lage temperatuur meegeeft wordt de uitkomst geclamped
/// op 0.
#[must_use]
pub fn koellast_buitenlucht(
    q_v_lea_m3_per_h: f64,
    q_v_vent_m3_per_h: f64,
    q_v_mech_m3_per_h: f64,
    outdoor_temperature_c: f64,
) -> f64 {
    let totaal_m3_per_s = (q_v_lea_m3_per_h + q_v_vent_m3_per_h + q_v_mech_m3_per_h) / 3600.0;
    let delta_t = outdoor_temperature_c - 24.0;
    if delta_t <= 0.0 {
        return 0.0;
    }
    totaal_m3_per_s * RHO_AIR_KG_PER_M3 * C_AIR_J_PER_KGK * delta_t
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn aa4_constants_match_norm() {
        assert!((RHO_AIR_KG_PER_M3 - 1.205).abs() < 1e-9);
        assert!((C_AIR_J_PER_KGK - 1005.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_aa1_piek_17h() {
        // Hoogste waarde 30,6 °C om 17:00
        assert_eq!(tabel_aa1_buitentemperatuur(17), Some(30.6));
    }

    #[test]
    fn tabel_aa1_ontbreekt_voor_8h() {
        assert_eq!(tabel_aa1_buitentemperatuur(8), None);
    }

    #[test]
    fn tabel_aa1_heeft_13_entries() {
        assert_eq!(TABEL_AA_1_BUITENLUCHTTEMPERATUUR.len(), 13);
    }

    #[test]
    fn aa4_zero_at_24_degrees() {
        let p = koellast_buitenlucht(100.0, 50.0, 30.0, 24.0);
        assert_abs_diff_eq!(p, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn aa4_numeriek_voorbeeld_woning_juli_17h() {
        // 100 m³/h infiltratie + 150 m³/h mechanische ventilatie, θ_e = 30,6°C
        // → 250/3600 × 1,205 × 1005 × 6,6 ≈ 554,4 W
        let p = koellast_buitenlucht(100.0, 0.0, 150.0, 30.6);
        let expected = (250.0 / 3600.0) * RHO_AIR_KG_PER_M3 * C_AIR_J_PER_KGK * (30.6 - 24.0);
        assert_abs_diff_eq!(p, expected, epsilon = 1e-9);
        assert!(p > 500.0 && p < 600.0);
    }

    #[test]
    fn aa4_clamp_bij_te_lage_outdoor() {
        // Edge-case: caller geeft 20°C mee → fysisch onmogelijk in julipiek,
        // maar clamped op 0 ipv negatief.
        let p = koellast_buitenlucht(100.0, 50.0, 30.0, 20.0);
        assert_abs_diff_eq!(p, 0.0, epsilon = 1e-9);
    }
}
