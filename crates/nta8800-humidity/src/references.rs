//! Norm-identifier constanten voor `nta8800-humidity`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_FORMULE12_1` vindt alle call-sites voor de bevochtigingsformule,
//! ook als de Rust-functienaam later verandert.

// ---------------------------------------------------------------------------
// Paragrafen — hoofdstuk 12 Bevochtiging en Ontvochtiging
// ---------------------------------------------------------------------------

/// H.12 Bevochtiging en ontvochtiging — overkoepelend.
pub const NTA_8800_2025_PARAG12: &str = "nta_8800_2025_parag12";

/// §12.1 Principe — bevochtiging en ontvochtiging energiebehoefte.
pub const NTA_8800_2025_PARAG12_1: &str = "nta_8800_2025_parag12_1";

/// §12.2 Maandmethode bevochtiging — energiebehoefte bepaling.
pub const NTA_8800_2025_PARAG12_2: &str = "nta_8800_2025_parag12_2";

/// §12.3 Maandmethode ontvochtiging — energiebehoefte bepaling.
pub const NTA_8800_2025_PARAG12_3: &str = "nta_8800_2025_parag12_3";

/// §12.4 Bevochtigings- en ontvochtigingssystemen — systeemrendementen.
pub const NTA_8800_2025_PARAG12_4: &str = "nta_8800_2025_parag12_4";

/// §12.5 Elektrisch energiegebruik — ventilator- en pompenergie.
pub const NTA_8800_2025_PARAG12_5: &str = "nta_8800_2025_parag12_5";

// ---------------------------------------------------------------------------
// Formules — hoofdstuk 12
// ---------------------------------------------------------------------------

/// Formule (12.1) — bevochtigingsbehoefte `Q_hum = ṁ_a · (x_IDA - x_ODA) · r_w`.
///
/// Met `r_w = 2501 kJ/kg` verdampingswarmte water bij 0°C.
pub const NTA_8800_2025_FORMULE12_1: &str = "nta_8800_2025_formule12_1";

/// Formule (12.2) — ontvochtigingsbehoefte `Q_dhum = ṁ_a · (x_ODA - x_IDA) · r_w`.
pub const NTA_8800_2025_FORMULE12_2: &str = "nta_8800_2025_formule12_2";

/// Formule (12.3) — absolute vochtigheid uit relatieve vochtigheid en temperatuur.
///
/// `x = 0.622 · (φ · p_sat) / (p_atm - φ · p_sat)` in kg/kg.
pub const NTA_8800_2025_FORMULE12_3: &str = "nta_8800_2025_formule12_3";

/// Formule (12.4) — verzadigingsdampdruk volgens Magnus-Tetens.
///
/// `p_sat = 611.2 · exp((17.62 · T) / (T + 243.12))` in Pa.
pub const NTA_8800_2025_FORMULE12_4: &str = "nta_8800_2025_formule12_4";

/// Formule (12.5) — massastroom droge lucht `ṁ_a = ρ_a · V̇_a`.
pub const NTA_8800_2025_FORMULE12_5: &str = "nta_8800_2025_formule12_5";

/// Formule (12.6) — stoomgenerator energiebehoefte `W_steam = Q_hum / η_steam`.
pub const NTA_8800_2025_FORMULE12_6: &str = "nta_8800_2025_formule12_6";

/// Formule (12.7) — sproeikoeler energiebehoefte `W_spray = Q_dhum / COP_spray`.
pub const NTA_8800_2025_FORMULE12_7: &str = "nta_8800_2025_formule12_7";

/// Formule (12.8) — adsorptie dehumidifier `W_ads = Q_dhum / COP_ads`.
pub const NTA_8800_2025_FORMULE12_8: &str = "nta_8800_2025_formule12_8";

// ---------------------------------------------------------------------------
// Tabellen — hoofdstuk 12
// ---------------------------------------------------------------------------

/// Tabel 12.1 — Rendement stoomgeneratoren (forfaitaire waarden).
pub const NTA_8800_2025_TABEL12_1: &str = "nta_8800_2025_tabel12_1";

/// Tabel 12.2 — COP sproeikoelers en droogkoelers (systeem-afhankelijk).
pub const NTA_8800_2025_TABEL12_2: &str = "nta_8800_2025_tabel12_2";

/// Tabel 12.3 — Vochtigheidssetpoints per gebruiksfunctie.
pub const NTA_8800_2025_TABEL12_3: &str = "nta_8800_2025_tabel12_3";

// ---------------------------------------------------------------------------
// Constanten — fysische eigenschappen
// ---------------------------------------------------------------------------

/// Verdampingswarmte water bij 0°C: r_w = 2501 kJ/kg.
pub const NTA_8800_2025_CONST_R_W_KJ_PER_KG: &str = "nta_8800_2025_const_r_w_kj_per_kg";

/// Atmosferische druk: p_atm = 101325 Pa (standaard).
pub const NTA_8800_2025_CONST_P_ATM_PA: &str = "nta_8800_2025_const_p_atm_pa";

/// Universele gasconstante water: R_w = 461.5 J/(kg·K).
pub const NTA_8800_2025_CONST_R_W_J_PER_KG_K: &str = "nta_8800_2025_const_r_w_j_per_kg_k";

// ---------------------------------------------------------------------------
// Tests — sanity checks op de canonieke strings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG12,
        NTA_8800_2025_PARAG12_1,
        NTA_8800_2025_PARAG12_2,
        NTA_8800_2025_PARAG12_3,
        NTA_8800_2025_PARAG12_4,
        NTA_8800_2025_PARAG12_5,
        NTA_8800_2025_FORMULE12_1,
        NTA_8800_2025_FORMULE12_2,
        NTA_8800_2025_FORMULE12_3,
        NTA_8800_2025_FORMULE12_4,
        NTA_8800_2025_FORMULE12_5,
        NTA_8800_2025_FORMULE12_6,
        NTA_8800_2025_FORMULE12_7,
        NTA_8800_2025_FORMULE12_8,
        NTA_8800_2025_TABEL12_1,
        NTA_8800_2025_TABEL12_2,
        NTA_8800_2025_TABEL12_3,
        NTA_8800_2025_CONST_R_W_KJ_PER_KG,
        NTA_8800_2025_CONST_P_ATM_PA,
        NTA_8800_2025_CONST_R_W_J_PER_KG_K,
    ];

    #[test]
    fn canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(
            set.len(),
            ALL.len(),
            "Dubbele canonieke string in references.rs"
        );
    }

    #[test]
    fn all_constants_have_prefix() {
        for id in ALL {
            assert!(
                id.starts_with("nta_8800_2025_"),
                "Constante {id:?} mist prefix \"nta_8800_2025_\""
            );
        }
    }

    #[test]
    fn no_whitespace_in_canonical_strings() {
        for id in ALL {
            assert!(
                !id.chars().any(char::is_whitespace),
                "Constante {id:?} bevat whitespace"
            );
        }
    }
}