//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 hoofdstuk 7 (maandmethode)
//! en hoofdstuk 8 (transmissie).
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html) voor
//! de naamgevings-conventie. Deze crate levert de sub-verzameling constanten
//! die daadwerkelijk in V1 is geïmplementeerd; bijlagen A, B, D, J en delen van
//! §8.3 (volledige grond-NEN-EN-ISO 13370) volgen in latere releases.
//!
//! # Scope V1
//!
//! | Formule / paragraaf | Implementatie |
//! |---|---|
//! | (7.14)/(7.15) — maand-Q transmissie | `calc::monthly_energy` |
//! | (7.16) — som deelcoëfficiënten | `calc::monthly_energy` |
//! | (8.1) — H_D = ΣAU + ΣψL + Σχ | `calc::h_t_outdoor` |
//! | (8.52)–(8.55) — H_U via onverwarmde ruimte | `calc::h_t_unheated` |
//! | §8.5 / (8.60)–(8.61) — H_A aangrenzend verwarmd | `calc::h_t_adjacent_zone` |
//! | §8.3.1 vereenvoudigd — H_g via bijlage I.2.3 | `calc::h_t_ground` |
//! | §8.2.3 / §8.2.4 — lineaire + punt-bruggen | `calc::thermal_bridges` |

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 7 (maandmethode)
// ---------------------------------------------------------------------------

/// §7.3.2 — Rekenprocedure totale warmteoverdracht door transmissie per maand.
///
/// Definieert formules (7.14) en (7.15) voor Q_H;tr;zi;mi en Q_C;tr;zi;mi.
pub const NTA_8800_2025_PARAG7_3_2: &str = "nta_8800_2025_parag7_3_2";

/// Formule (7.14) — maandelijkse transmissiewarmte voor verwarming in kWh.
///
/// `Q_H;tr;zi;mi = (H_H;tr(excl.gf;m);zi;mi · (θ_int;calc;H;zi;mi − θ_e;avg;mi)
///                   + H_g;an;zi;mi · (θ_int;calc;H;zi;mi − θ_e;avg;an)) · 0.001 · t_mi`
pub const NTA_8800_2025_FORMULE7_14: &str = "nta_8800_2025_formule7_14";

/// Formule (7.15) — maandelijkse transmissiewarmte voor koeling in kWh.
///
/// Identieke structuur als (7.14) maar met C-setpoints. Niet geïmplementeerd in V1
/// (koeling leeft in `nta8800-cooling`).
pub const NTA_8800_2025_FORMULE7_15: &str = "nta_8800_2025_formule7_15";

/// Formule (7.16) — som van deelcoëfficiënten exclusief grondvloer.
///
/// `H_tr(excl.gf) = H_D + H_U + H_A + H_p`
pub const NTA_8800_2025_FORMULE7_16: &str = "nta_8800_2025_formule7_16";

// ---------------------------------------------------------------------------
// Paragrafen hoofdstuk 8 (transmissie)
// ---------------------------------------------------------------------------

/// §8.1 — Modellering van de gebouwomhulling.
pub const NTA_8800_2025_PARAG8_1: &str = "nta_8800_2025_parag8_1";

/// §8.2.1 — Berekening van de directe warmteverliescoëfficiënt H_D.
pub const NTA_8800_2025_PARAG8_2_1: &str = "nta_8800_2025_parag8_2_1";

/// Formule (8.1) — Directe warmteverliescoëfficiënt tussen verwarmde ruimte en
/// buitenlucht.
///
/// `H_D = Σ(A_T;i · U_C;i) + Σ(L_k · ψ_k) + Σχ_j` — in W/K.
pub const NTA_8800_2025_FORMULE8_1: &str = "nta_8800_2025_formule8_1";

/// Formule (8.2) — Forfaitaire alternatief voor H_D met ΔU_for toeslag.
///
/// Niet geïmplementeerd in V1 (forfaitaire thermische-brug-verrekening is een
/// optionele vereenvoudiging die het gebouw als geheel betreft; consumers die
/// dit wensen kunnen het in de UI uitschakelen door ψ/χ op 0 te zetten).
pub const NTA_8800_2025_FORMULE8_2: &str = "nta_8800_2025_formule8_2";

/// §8.2.3 — Lineaire thermische bruggen (ψ-waarden).
pub const NTA_8800_2025_PARAG8_2_3: &str = "nta_8800_2025_parag8_2_3";

/// §8.2.4 — Puntvormige thermische bruggen (χ-waarden).
pub const NTA_8800_2025_PARAG8_2_4: &str = "nta_8800_2025_parag8_2_4";

/// §8.3.1 — Inleiding grondtransmissie. V1 gebruikt vereenvoudigd pad via
/// bijlage I.2.3 (user-supplied H_g;an), de volledige NEN-EN-ISO 13370 bepaling
/// volgt in V2.
pub const NTA_8800_2025_PARAG8_3_1: &str = "nta_8800_2025_parag8_3_1";

/// §8.4.1 — Warmteverlies via onverwarmde ruimte H_U.
pub const NTA_8800_2025_PARAG8_4_1: &str = "nta_8800_2025_parag8_4_1";

/// Formule (8.52) — H_U = H_D;zi,j;ztu · b_U.
pub const NTA_8800_2025_FORMULE8_52: &str = "nta_8800_2025_formule8_52";

/// Formule (8.53) — dimensieloze reductiefactor b_U = H_ue / (H_zi,j;ztu + H_ue).
pub const NTA_8800_2025_FORMULE8_53: &str = "nta_8800_2025_formule8_53";

/// §8.5 — Warmteverliescoëfficiënt via aangrenzende verwarmde ruimten H_A.
///
/// NTA 8800 verwaarloost H_A (H_A;mi = 0); de norm-afwijkende variant met
/// formules (8.60)/(8.61) uit NEN-EN-ISO 13789:2017 is beschikbaar als opt-in.
pub const NTA_8800_2025_PARAG8_5: &str = "nta_8800_2025_parag8_5";

/// Formule (8.60) — optionele H_A;mi = H_D;ia · b_A;mi.
pub const NTA_8800_2025_FORMULE8_60: &str = "nta_8800_2025_formule8_60";

/// Formule (8.61) — reductiefactor b_A;mi = (θ_i − θ_a) / (θ_i − θ_e;mi).
pub const NTA_8800_2025_FORMULE8_61: &str = "nta_8800_2025_formule8_61";

/// Bijlage I.2.3 — gebouwelementen in contact met grond bij onbekende
/// vloerconstructie-opbouw (fallback voor H_g;an).
pub const NTA_8800_2025_BIJLAGE_I_PARAG2_3: &str = "nta_8800_2025_bijlage_i_parag2_3";

/// Bijlage I.2.4 — warmteoverdrachtcoëfficiënt van onverwarmde ruimte naar
/// buitenomgeving bij basisopname ISSO 82.1/75.1 (H_U;adj pad).
pub const NTA_8800_2025_BIJLAGE_I_PARAG2_4: &str = "nta_8800_2025_bijlage_i_parag2_4";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG7_3_2,
        NTA_8800_2025_FORMULE7_14,
        NTA_8800_2025_FORMULE7_15,
        NTA_8800_2025_FORMULE7_16,
        NTA_8800_2025_PARAG8_1,
        NTA_8800_2025_PARAG8_2_1,
        NTA_8800_2025_FORMULE8_1,
        NTA_8800_2025_FORMULE8_2,
        NTA_8800_2025_PARAG8_2_3,
        NTA_8800_2025_PARAG8_2_4,
        NTA_8800_2025_PARAG8_3_1,
        NTA_8800_2025_PARAG8_4_1,
        NTA_8800_2025_FORMULE8_52,
        NTA_8800_2025_FORMULE8_53,
        NTA_8800_2025_PARAG8_5,
        NTA_8800_2025_FORMULE8_60,
        NTA_8800_2025_FORMULE8_61,
        NTA_8800_2025_BIJLAGE_I_PARAG2_3,
        NTA_8800_2025_BIJLAGE_I_PARAG2_4,
    ];

    #[test]
    fn canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(set.len(), ALL.len());
    }

    #[test]
    fn all_constants_have_prefix() {
        for id in ALL {
            assert!(id.starts_with("nta_8800_2025_"), "missing prefix: {id:?}");
        }
    }

    #[test]
    fn no_whitespace_in_canonical_strings() {
        for id in ALL {
            assert!(!id.chars().any(char::is_whitespace), "ws in {id:?}");
        }
    }
}
