//! Norm-identifier constanten voor NTA 8800:2025+C1:2026 H.10 (koeling) en
//! bijlage AA (vereenvoudigde koelbehoefte woningen, TOjuli-opvolger).
//!
//! Zie [`nta8800_model::references`](../../nta8800_model/references/index.html)
//! voor de naamgevings-conventie. Deze crate levert de sub-verzameling
//! constanten voor actieve koeling (H.10) en de vereenvoudigde rekenmethode
//! uit bijlage AA.

// ---------------------------------------------------------------------------
// Hoofdstuk 10 — Koeling
// ---------------------------------------------------------------------------

/// H.10 — Koeling (koudeopwek, distributie, afgifte).
pub const NTA_8800_2025_PARAG10: &str = "nta_8800_2025_parag10";

/// §10.1 — Principe: indien een rekenzone geen koelsysteem heeft, worden alle
/// koel-gerelateerde energiestromen op 0 gesteld.
pub const NTA_8800_2025_PARAG10_1: &str = "nta_8800_2025_parag10_1";

// ---------------------------------------------------------------------------
// Bijlage AA — Vereenvoudigde bepalingsmethode koelbehoefte
// ---------------------------------------------------------------------------

/// Bijlage AA — Vereenvoudigde bepaling van de koelbehoefte en de minimaal
/// benodigde koelcapaciteit in woningen.
pub const NTA_8800_2025_BIJLAGE_AA: &str = "nta_8800_2025_bijlage_aa";

/// AA.1 — Vereenvoudigde bepalingsmethode koelbehoefte (inleiding).
pub const NTA_8800_2025_BIJLAGE_AA_PARAG1: &str = "nta_8800_2025_bijlage_aa_parag1";

/// AA.2 — Stappenplan bepalingsmethode koelbehoefte.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2: &str = "nta_8800_2025_bijlage_aa_parag2";

/// AA.2.2.1 — Koellast door interne warmtelast.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2_2_1: &str = "nta_8800_2025_bijlage_aa_parag2_2_1";

/// AA.2.2.2 — Koellast door buitenluchttoetreding.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2_2_2: &str = "nta_8800_2025_bijlage_aa_parag2_2_2";

/// AA.2.2.3 — Koellast door transmissie door ondoorzichtige delen.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2_2_3: &str = "nta_8800_2025_bijlage_aa_parag2_2_3";

/// AA.2.2.4 — Koellast door zoninstraling via transparante delen.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2_2_4: &str = "nta_8800_2025_bijlage_aa_parag2_2_4";

/// AA.2.2.5 — Koellast door transmissie via transparante delen.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG2_2_5: &str = "nta_8800_2025_bijlage_aa_parag2_2_5";

/// AA.3 — Toetsing koelcapaciteit aan beperken risico op oververhitting.
pub const NTA_8800_2025_BIJLAGE_AA_PARAG3: &str = "nta_8800_2025_bijlage_aa_parag3";

// ---------------------------------------------------------------------------
// Bijlage AA — formules
// ---------------------------------------------------------------------------

/// Formule (AA.1) — basiswaarde interne warmtelast per rekenzone.
///
/// `N_int;zi = 180 · N_woon;zi · P_p;woon;zi`  [W]
///
/// Met `N_woon` het aantal woonfuncties in de rekenzone en `P_p;woon` het
/// gemiddelde aantal bewoners per woonfunctie (conform 7.5.2.1).
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE1: &str = "nta_8800_2025_bijlage_aa_formule1";

/// Formule (AA.2) — rekenwaarde interne warmtelast per m².
///
/// `q_int;calc;zi = N_int;zi / (2 · Σ(A_vr;woon;zi) + Σ(A_vr;overig;zi))`  [W/m²]
///
/// Woonkamer/keuken/eetkamer tellen dubbel mee in de noemer vanwege het
/// grotere aandeel in de interne warmtelast.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE2: &str = "nta_8800_2025_bijlage_aa_formule2";

/// Formule (AA.3a) — interne warmtelast voor woonkamer/keuken/eetkamer.
///
/// `P_int;calc;woon;zi = 2 · q_int;calc;zi · A_vg;woon;zi`  [W]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE3A: &str = "nta_8800_2025_bijlage_aa_formule3a";

/// Formule (AA.3b) — interne warmtelast voor overige verblijfsruimten.
///
/// `P_int;calc;overig;zi = q_int;calc;zi · A_vg;overig;zi`  [W]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE3B: &str = "nta_8800_2025_bijlage_aa_formule3b";

/// Formule (AA.4) — koellast door buitenluchttoetreding.
///
/// `P_V;zi = ((q_v;C;eff;lea;in + q_v;C;eff;vent;in + q_v;C;SUP;eff) / 3600)
///   · ρ_a · c_a · (θ_e;max;zi − 24)`  [W]
///
/// Met ρ_a = 1,205 kg/m³, c_a = 1 005 J/kgK en θ_e uit tabel AA.1 op het
/// tijdstip van de maximale koellast.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE4: &str = "nta_8800_2025_bijlage_aa_formule4";

/// Formule (AA.5) — koellast door transmissie door ondoorzichtige delen.
///
/// `P_tr;ntr;zi = f_iso · A_in;zi`  [W]
///
/// Met `f_iso` uit tabel AA.2 (bouwjaarklasse) en A_in de binnenwerkse
/// oppervlakte van de ondoorzichtige delen van buitenwand + dak.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE5: &str = "nta_8800_2025_bijlage_aa_formule5";

/// Formule (AA.6a/b) — koellast door zoninstraling via transparante delen.
///
/// `P_sol;i = Σ P_sol;vr;j`  met per verblijfsruimte het maximum over
/// `t = 9..18 h` van `Σ (0,75 · A_wi · g_gl · F_sh · F_C · I_sol;wi;t)`.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE6: &str = "nta_8800_2025_bijlage_aa_formule6";

/// Formule (AA.7) — koellast door transmissie via transparante delen (glas).
///
/// `P_gl;zi = Σ (A_wi · U_w;wi) · (θ_e − 24)`  [W]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE7: &str = "nta_8800_2025_bijlage_aa_formule7";

/// Formule (AA.8) — maatgevende koelbehoefte van de rekenzone.
///
/// `q_C;zi = (P_int;calc + P_V + P_tr;ntr + P_sol + P_gl) / A_g;vr;zi`  [W/m²]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE8: &str = "nta_8800_2025_bijlage_aa_formule8";

/// Formule (AA.9) — maatgevende koelbehoefte per verblijfsruimte.
///
/// `q_C;vr;zi,j = (P_int;calc;vr + P_V;vr + P_tr;ntr;vr + P_sol;vr + P_gl;vr)
///   / A_g;vr;zi,j`  [W/m²]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE9: &str = "nta_8800_2025_bijlage_aa_formule9";

/// Formule (AA.10) — criterium koelcapaciteit opwekker rekenzone.
///
/// `B_C;inst;zi ≥ B_C;req;TO;zi`  [kW]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE10: &str = "nta_8800_2025_bijlage_aa_formule10";

/// Formule (AA.11) — benodigde koelcapaciteit opwekker rekenzone.
///
/// `B_C;req;TO;zi = (q_C;zi − 35) / 1000 · A_g;vr;zi`  [kW], min. 0.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE11: &str = "nta_8800_2025_bijlage_aa_formule11";

/// Formule (AA.12) — criterium koelcapaciteit afgifte per verblijfsruimte.
///
/// `B_C;inst;zi,j ≥ B_C;req;TO;zi,j`  [kW]
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE12: &str = "nta_8800_2025_bijlage_aa_formule12";

/// Formule (AA.13) — benodigde koelcapaciteit afgifte per verblijfsruimte.
///
/// `B_C;req;TO;zi,j = (q_C;zi,j − 35) / 1000 · A_g;vr;zi,j`  [kW], min. 0.
pub const NTA_8800_2025_BIJLAGE_AA_FORMULE13: &str = "nta_8800_2025_bijlage_aa_formule13";

/// Tabel AA.1 — Aan te houden buitenluchttemperatuur θ_e per tijdstip (9..21h).
pub const NTA_8800_2025_BIJLAGE_AA_TABEL1: &str = "nta_8800_2025_bijlage_aa_tabel1";

/// Tabel AA.2 — factor f_iso voor thermische isolatie ondoorzichtige delen
/// per bouwjaarklasse.
pub const NTA_8800_2025_BIJLAGE_AA_TABEL2: &str = "nta_8800_2025_bijlage_aa_tabel2";

/// Tabel AA.3 — totale opvallende zonnestraling I_sol;wi;t [W/m²] per
/// oriëntatie (γ) en hellingshoek (β), uurwaarde 9..18h,
/// grondreflectiecoëfficiënt ρ = 0,2.
pub const NTA_8800_2025_BIJLAGE_AA_TABEL3: &str = "nta_8800_2025_bijlage_aa_tabel3";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG10,
        NTA_8800_2025_PARAG10_1,
        NTA_8800_2025_BIJLAGE_AA,
        NTA_8800_2025_BIJLAGE_AA_PARAG1,
        NTA_8800_2025_BIJLAGE_AA_PARAG2,
        NTA_8800_2025_BIJLAGE_AA_PARAG2_2_1,
        NTA_8800_2025_BIJLAGE_AA_PARAG2_2_2,
        NTA_8800_2025_BIJLAGE_AA_PARAG2_2_3,
        NTA_8800_2025_BIJLAGE_AA_PARAG2_2_4,
        NTA_8800_2025_BIJLAGE_AA_PARAG2_2_5,
        NTA_8800_2025_BIJLAGE_AA_PARAG3,
        NTA_8800_2025_BIJLAGE_AA_FORMULE1,
        NTA_8800_2025_BIJLAGE_AA_FORMULE2,
        NTA_8800_2025_BIJLAGE_AA_FORMULE3A,
        NTA_8800_2025_BIJLAGE_AA_FORMULE3B,
        NTA_8800_2025_BIJLAGE_AA_FORMULE4,
        NTA_8800_2025_BIJLAGE_AA_FORMULE5,
        NTA_8800_2025_BIJLAGE_AA_FORMULE6,
        NTA_8800_2025_BIJLAGE_AA_FORMULE7,
        NTA_8800_2025_BIJLAGE_AA_FORMULE8,
        NTA_8800_2025_BIJLAGE_AA_FORMULE9,
        NTA_8800_2025_BIJLAGE_AA_FORMULE10,
        NTA_8800_2025_BIJLAGE_AA_FORMULE11,
        NTA_8800_2025_BIJLAGE_AA_FORMULE12,
        NTA_8800_2025_BIJLAGE_AA_FORMULE13,
        NTA_8800_2025_BIJLAGE_AA_TABEL1,
        NTA_8800_2025_BIJLAGE_AA_TABEL2,
        NTA_8800_2025_BIJLAGE_AA_TABEL3,
    ];

    #[test]
    fn canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(
            set.len(),
            ALL.len(),
            "Dubbele canonieke string gevonden in cooling references.rs"
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
