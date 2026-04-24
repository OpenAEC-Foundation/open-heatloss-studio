//! Norm-identifier constanten voor `nta8800-ventilation`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_FORMULE11_107` vindt alle call-sites voor de WTW-formule,
//! ook als de Rust-functienaam later verandert.

// ---------------------------------------------------------------------------
// Paragrafen — hoofdstuk 11 Ventilatie
// ---------------------------------------------------------------------------

/// H.11 Ventilatie — overkoepelend.
pub const NTA_8800_2025_PARAG11: &str = "nta_8800_2025_parag11";

/// §11.1 Principe — ventilatie-indeling en systeem-benamingen.
pub const NTA_8800_2025_PARAG11_1: &str = "nta_8800_2025_parag11_1";

/// §11.2 Bepalen effectieve luchtvolumestromen.
pub const NTA_8800_2025_PARAG11_2: &str = "nta_8800_2025_parag11_2";

/// §11.2.1 Luchtstroommodel — stappenplan + massabalans.
pub const NTA_8800_2025_PARAG11_2_1: &str = "nta_8800_2025_parag11_2_1";

/// §11.3 Temperatuur van de luchtstromen.
pub const NTA_8800_2025_PARAG11_3: &str = "nta_8800_2025_parag11_3";

/// §11.3.2.2 Temperatuursprong warmteterugwinning (WTW).
pub const NTA_8800_2025_PARAG11_3_2_2: &str = "nta_8800_2025_parag11_3_2_2";

/// §11.4.3.3 Effectief ventilatorvermogen forfaitair.
pub const NTA_8800_2025_PARAG11_4_3_3: &str = "nta_8800_2025_parag11_4_3_3";

// ---------------------------------------------------------------------------
// Formules — hoofdstuk 11
// ---------------------------------------------------------------------------

/// Formule (11.106) — verwarmingsvermogen elektrische vorstbeveiliging.
///
/// Leest ook als de canonieke formule voor ventilatie-warmtestroom:
/// `P = q · ρ_a · c_a · ΔT / 3600` (q in m³/h, P in W).
pub const NTA_8800_2025_FORMULE11_106: &str = "nta_8800_2025_formule11_106";

/// Formule (11.106a) — WTW-temperatuursprong bij 100%-bypass met koudeterugwinning.
pub const NTA_8800_2025_FORMULE11_106A: &str = "nta_8800_2025_formule11_106a";

/// Formule (11.107) — WTW-temperatuursprong standaard (warmtebehoefte).
pub const NTA_8800_2025_FORMULE11_107: &str = "nta_8800_2025_formule11_107";

/// Formule (11.108) — `ϑ_ODA;preh = ϑ_ODA + ΔT_preh` (toevoertemperatuur na voorverwarming).
pub const NTA_8800_2025_FORMULE11_108: &str = "nta_8800_2025_formule11_108";

/// Formule (11.140) — `P_eff;for;BAL_DEC` (forfaitair ventilatorvermogen, decentraal).
pub const NTA_8800_2025_FORMULE11_140: &str = "nta_8800_2025_formule11_140";

/// Formule (11.141) — `P_eff;for;overig` (forfaitair ventilatorvermogen, overig).
pub const NTA_8800_2025_FORMULE11_141: &str = "nta_8800_2025_formule11_141";

/// Formule (11.142) — `P_eff;for` (forfaitair ventilatorvermogen, standaard).
pub const NTA_8800_2025_FORMULE11_142: &str = "nta_8800_2025_formule11_142";

// ---------------------------------------------------------------------------
// Tabellen — hoofdstuk 11
// ---------------------------------------------------------------------------

/// Tabel 11.18 — Rendement van WTW-installaties (forfaitair η_hr).
pub const NTA_8800_2025_TABEL11_18: &str = "nta_8800_2025_tabel11_18";

/// Tabel 11.23 — Specifiek ventilatorvermogen f_SFP als functie van fabricagejaar.
pub const NTA_8800_2025_TABEL11_23: &str = "nta_8800_2025_tabel11_23";

// ---------------------------------------------------------------------------
// Bijlage S — systeemvarianten ventilatie
// ---------------------------------------------------------------------------

/// Bijlage S (informatief) — Systeemvarianten ventilatie A/B/C/D/E.
pub const NTA_8800_2025_BIJLAGE_S: &str = "nta_8800_2025_bijlage_s";

/// Bijlage S §2.4 — Systeemvariant D (balansventilatie), subvarianten D.1–D.5.
pub const NTA_8800_2025_BIJLAGE_S_PARAG2_4: &str = "nta_8800_2025_bijlage_s_parag2_4";

// ---------------------------------------------------------------------------
// Tests — sanity checks op de canonieke strings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG11,
        NTA_8800_2025_PARAG11_1,
        NTA_8800_2025_PARAG11_2,
        NTA_8800_2025_PARAG11_2_1,
        NTA_8800_2025_PARAG11_3,
        NTA_8800_2025_PARAG11_3_2_2,
        NTA_8800_2025_PARAG11_4_3_3,
        NTA_8800_2025_FORMULE11_106,
        NTA_8800_2025_FORMULE11_106A,
        NTA_8800_2025_FORMULE11_107,
        NTA_8800_2025_FORMULE11_108,
        NTA_8800_2025_FORMULE11_140,
        NTA_8800_2025_FORMULE11_141,
        NTA_8800_2025_FORMULE11_142,
        NTA_8800_2025_TABEL11_18,
        NTA_8800_2025_TABEL11_23,
        NTA_8800_2025_BIJLAGE_S,
        NTA_8800_2025_BIJLAGE_S_PARAG2_4,
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
