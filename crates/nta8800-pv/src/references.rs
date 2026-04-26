//! Norm-identifier constanten voor `nta8800-pv`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_FORMULE16_101` vindt alle call-sites voor de PV-formule,
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

/// Formule (16.101) — maandelijkse PV-opbrengst basis.
///
/// `Q_PV;mi = P_PV;peak * I_sol;mi * η_sys * η_inv * t_maand / 1000`
/// waarbij P_PV;peak in kWp, I_sol;mi in W/m² gemiddeld, η factoren dimensieloos.
pub const NTA_8800_2025_FORMULE16_101: &str = "nta_8800_2025_formule16_101";

/// Formule (16.102) — correctiefactor voor tilt en azimuth.
///
/// `f_tilt_az = f_tilt(β) * f_az(γ)` met β = hellingshoek, γ = azimuth.
pub const NTA_8800_2025_FORMULE16_102: &str = "nta_8800_2025_formule16_102";

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

/// Tabel 16.1 — Correctiefactoren hellingshoek β (0° tot 90°).
pub const NTA_8800_2025_TABEL16_1: &str = "nta_8800_2025_tabel16_1";

/// Tabel 16.2 — Correctiefactoren azimuth γ (-180° tot +180°).
pub const NTA_8800_2025_TABEL16_2: &str = "nta_8800_2025_tabel16_2";

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
        NTA_8800_2025_FORMULE16_101,
        NTA_8800_2025_FORMULE16_102,
        NTA_8800_2025_FORMULE16_103,
        NTA_8800_2025_FORMULE16_104,
        NTA_8800_2025_FORMULE16_105,
        NTA_8800_2025_TABEL16_1,
        NTA_8800_2025_TABEL16_2,
        NTA_8800_2025_TABEL16_3,
        NTA_8800_2025_TABEL16_4,
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
