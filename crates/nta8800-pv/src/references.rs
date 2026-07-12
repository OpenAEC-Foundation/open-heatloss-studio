//! Norm-identifier constanten voor `nta8800-pv`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_FORMULE16_2` vindt alle call-sites voor de PV-formule,
//! ook als de Rust-functienaam later verandert.

// ---------------------------------------------------------------------------
// Paragrafen — hoofdstuk 16 PV-systemen
// ---------------------------------------------------------------------------

/// H.16 Fotovoltaïsche systemen — overkoepelend.
pub const NTA_8800_2025_PARAG16: &str = "nta_8800_2025_parag16";

/// §16.1 Principe — PV-systeem indeling en configuratie.
pub const NTA_8800_2025_PARAG16_1: &str = "nta_8800_2025_parag16_1";

/// §16.2 Maandelijkse PV-opbrengst bepaling.
pub const NTA_8800_2025_PARAG16_2: &str = "nta_8800_2025_parag16_2";

/// §16.2.1 Basis PV-berekening per maand.
pub const NTA_8800_2025_PARAG16_2_1: &str = "nta_8800_2025_parag16_2_1";

/// §16.3 Correctiefactoren voor oriëntatie en hellingshoek.
pub const NTA_8800_2025_PARAG16_3: &str = "nta_8800_2025_parag16_3";

/// §16.4 Systeem-efficiëntie en verliezen.
pub const NTA_8800_2025_PARAG16_4: &str = "nta_8800_2025_parag16_4";

/// §16.4.1 Inverter-efficiëntie bepaling.
pub const NTA_8800_2025_PARAG16_4_1: &str = "nta_8800_2025_parag16_4_1";

/// §16.5 Vervuiling en schaduwfactoren.
pub const NTA_8800_2025_PARAG16_5: &str = "nta_8800_2025_parag16_5";

// ---------------------------------------------------------------------------
// Formules — hoofdstuk 16
// ---------------------------------------------------------------------------

/// Formule (16.2) — maandelijkse PV-opbrengst per systeem (PDF p. 677).
///
/// `E_el;PV;out;i,mi = E_sol;mi · P_pk;i · f_perf;i · c_sh,PV;mi;i · f_prac,PV;i / I_ref`.
/// De opbrengstfactor `f_perf` komt uit Tabel 16.2, de schaduwcorrectie
/// `c_sh` uit Tabel 16.3, `f_prac = 0,95`, `I_ref = 1 kW/m²`.
pub const NTA_8800_2025_FORMULE16_2: &str = "nta_8800_2025_formule16_2";

/// Formule (16.3) — maandelijkse opvallende zonnestraling op het PV-vlak
/// (PDF p. 678): `E_sol;mi = I_sol;mi · t_mi · F_sh;obst;mi / 1000` [kWh/m²].
///
/// **`I_sol;mi` komt uit Tabel 17.2** per hellingshoek β én oriëntatie γ
/// (OPMERKING 2 bij deze formule, PDF p. 678) — hierin, níet in een aparte
/// "correctiefactor", zit de volledige tilt/azimut-afhankelijkheid.
pub const NTA_8800_2025_FORMULE16_3: &str = "nta_8800_2025_formule16_3";

/// Formule (16.103) — systeem-efficiëntie totaal.
///
/// `η_sys;tot = η_module * η_bekabeling * η_vervuiling * η_mismatch`
pub const NTA_8800_2025_FORMULE16_103: &str = "nta_8800_2025_formule16_103";

/// Formule (16.104) — inverter-efficiëntie correctie.
///
/// `η_inv;eff = η_inv;nom * f_load` met f_load afhankelijk van P_load/P_rated.
pub const NTA_8800_2025_FORMULE16_104: &str = "nta_8800_2025_formule16_104";

/// Formule (16.105) — vervuilingsfactor seizoensafhankelijk.
///
/// `f_vervuiling;mi = f_base + Δf_seizoen;mi` per maand variërend.
pub const NTA_8800_2025_FORMULE16_105: &str = "nta_8800_2025_formule16_105";

// ---------------------------------------------------------------------------
// Tabellen — hoofdstuk 16
// ---------------------------------------------------------------------------

/// Tabel 16.1 — Piekvermogen `Kpk` [W/m²] per zonnestroompaneeltype (PDF
/// p. 680). **Géén hellingshoekcorrectie** (die zit in Tabel 17.2).
pub const NTA_8800_2025_TABEL16_1: &str = "nta_8800_2025_tabel16_1";

/// Tabel 16.2 — Opbrengstfactor `f_perf` van het zonnestroomsysteem naar
/// bouwintegratie/ventilatie: 0,76 / 0,80 / 0,82 (PDF p. 681). **Géén
/// azimutcorrectie** (die zit in Tabel 17.2).
pub const NTA_8800_2025_TABEL16_2: &str = "nta_8800_2025_tabel16_2";

/// Tabel 17.2 — maandgemiddelde opvallende zonnestraling `I_sol;mi` [W/m²]
/// per hellingshoek β en oriëntatie γ, De Bilt (PDF p. 690-693). Draagt de
/// volledige tilt/azimut-afhankelijkheid van de PV-opbrengst; getranscribeerd
/// in [`crate::tables::irradiation`].
pub const NTA_8800_2025_TABEL17_2: &str = "nta_8800_2025_tabel17_2";

/// Tabel 16.3 — Forfaitaire systeem-efficiënties per PV-type.
pub const NTA_8800_2025_TABEL16_3: &str = "nta_8800_2025_tabel16_3";

/// Tabel 16.4 — Inverter-efficiënties als functie van belastingsfactor.
pub const NTA_8800_2025_TABEL16_4: &str = "nta_8800_2025_tabel16_4";

// ---------------------------------------------------------------------------
// Bijlage V — Bronregeneratie warmtepomp-bronnen
// ---------------------------------------------------------------------------

/// Bijlage V — Regeneratie van warmtepomp-bronnen door PV/zonnethermisch.
pub const NTA_8800_2025_BIJLAGE_V: &str = "nta_8800_2025_bijlage_v";

/// Bijlage V §1 — Principe bronregeneratie.
pub const NTA_8800_2025_BIJLAGE_V_PARAG1: &str = "nta_8800_2025_bijlage_v_parag1";

/// Bijlage V §2 — Energiebalans bodem/aquifer regeneratie.
pub const NTA_8800_2025_BIJLAGE_V_PARAG2: &str = "nta_8800_2025_bijlage_v_parag2";

/// Bijlage V §3 — Zonnethermisch-assisted bronregeneratie.
pub const NTA_8800_2025_BIJLAGE_V_PARAG3: &str = "nta_8800_2025_bijlage_v_parag3";

// ---------------------------------------------------------------------------
// Tests — sanity checks op de canonieke strings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG16,
        NTA_8800_2025_PARAG16_1,
        NTA_8800_2025_PARAG16_2,
        NTA_8800_2025_PARAG16_2_1,
        NTA_8800_2025_PARAG16_3,
        NTA_8800_2025_PARAG16_4,
        NTA_8800_2025_PARAG16_4_1,
        NTA_8800_2025_PARAG16_5,
        NTA_8800_2025_FORMULE16_2,
        NTA_8800_2025_FORMULE16_3,
        NTA_8800_2025_FORMULE16_103,
        NTA_8800_2025_FORMULE16_104,
        NTA_8800_2025_FORMULE16_105,
        NTA_8800_2025_TABEL16_1,
        NTA_8800_2025_TABEL16_2,
        NTA_8800_2025_TABEL16_3,
        NTA_8800_2025_TABEL16_4,
        NTA_8800_2025_TABEL17_2,
        NTA_8800_2025_BIJLAGE_V,
        NTA_8800_2025_BIJLAGE_V_PARAG1,
        NTA_8800_2025_BIJLAGE_V_PARAG2,
        NTA_8800_2025_BIJLAGE_V_PARAG3,
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

    #[test]
    fn references_count_meets_requirement() {
        assert!(
            ALL.len() >= 10,
            "Need ≥10 norm-identifier constants, got {}",
            ALL.len()
        );
    }
}
