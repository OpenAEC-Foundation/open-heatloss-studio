//! Norm-identifier constanten voor `nta8800-automation`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_TABEL15_1` vindt alle call-sites voor de BAC-factorentabel,
//! ook als de Rust-functienaam later verandert.

// ---------------------------------------------------------------------------
// Paragrafen — hoofdstuk 15 Gebouwautomatisering
// ---------------------------------------------------------------------------

/// H.15 Gebouwautomatisering en regeltechniek — overkoepelend.
pub const NTA_8800_2025_PARAG15: &str = "nta_8800_2025_parag15";

/// §15.1 Algemeen — BAC-klassen volgens NEN-EN 15232.
pub const NTA_8800_2025_PARAG15_1: &str = "nta_8800_2025_parag15_1";

/// §15.2 Bepaling correctiefactoren per energiedienst.
pub const NTA_8800_2025_PARAG15_2: &str = "nta_8800_2025_parag15_2";

/// §15.3 Woonfuncties — residentiële correctiefactoren.
pub const NTA_8800_2025_PARAG15_3: &str = "nta_8800_2025_parag15_3";

/// §15.4 Utiliteitsgebouwen — niet-residentiële correctiefactoren.
pub const NTA_8800_2025_PARAG15_4: &str = "nta_8800_2025_parag15_4";

// ---------------------------------------------------------------------------
// Formules — hoofdstuk 15
// ---------------------------------------------------------------------------

/// Formule (15.1) — Toepassing correctiefactor op netto energiegebruik.
/// `E_netto;gecorrigeerd = f_BAC × E_netto;basis`
pub const NTA_8800_2025_FORMULE15_1: &str = "nta_8800_2025_formule15_1";

/// Formule (15.2) — Samengestelde correctiefactor voor gemengde systemen.
pub const NTA_8800_2025_FORMULE15_2: &str = "nta_8800_2025_formule15_2";

// ---------------------------------------------------------------------------
// Tabellen — hoofdstuk 15
// ---------------------------------------------------------------------------

/// Tabel 15.1 — Correctiefactoren f_BAC voor woonfuncties.
/// Kolommen: heating, cooling, lighting, dhw, ventilation per BAC-klasse A/B/C/D.
pub const NTA_8800_2025_TABEL15_1: &str = "nta_8800_2025_tabel15_1";

/// Tabel 15.2 — Correctiefactoren f_BAC voor utiliteitsgebouwen.
/// Kolommen: heating, cooling, lighting, dhw, ventilation per BAC-klasse A/B/C/D.
pub const NTA_8800_2025_TABEL15_2: &str = "nta_8800_2025_tabel15_2";

/// Tabel 15.3 — BAC-klasse-definities volgens NEN-EN 15232.
/// Omschrijving van automatiseringsniveaus A (high perf) t/m D (non-efficient).
pub const NTA_8800_2025_TABEL15_3: &str = "nta_8800_2025_tabel15_3";

// ---------------------------------------------------------------------------
// Normverwijzingen
// ---------------------------------------------------------------------------

/// NEN-EN 15232 — Energy performance of buildings - Impact of building automation.
pub const NEN_EN_15232: &str = "nen_en_15232";

/// NTA 8800 C1:2026 — Correctieblad met updates op oorspronkelijke tabellen.
pub const NTA_8800_2025_C1_2026: &str = "nta_8800_2025_c1_2026";

// ---------------------------------------------------------------------------
// Tests — sanity checks op de canonieke strings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG15,
        NTA_8800_2025_PARAG15_1,
        NTA_8800_2025_PARAG15_2,
        NTA_8800_2025_PARAG15_3,
        NTA_8800_2025_PARAG15_4,
        NTA_8800_2025_FORMULE15_1,
        NTA_8800_2025_FORMULE15_2,
        NTA_8800_2025_TABEL15_1,
        NTA_8800_2025_TABEL15_2,
        NTA_8800_2025_TABEL15_3,
        NEN_EN_15232,
        NTA_8800_2025_C1_2026,
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
            // Norm-verwijzingen hebben eigen prefix
            let valid_prefixes = ["nta_8800_2025_", "nen_en_", "nen_iso_"];
            assert!(
                valid_prefixes.iter().any(|prefix| id.starts_with(prefix)),
                "Constante {id:?} heeft geen geldige prefix"
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