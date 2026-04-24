//! Gestandaardiseerde norm-identifier constanten voor NTA 8800:2025+C1:2026.
//!
//! Elke constante verwijst naar een specifieke formule, tabel, figuur, paragraaf
//! of bijlage in de norm. Gebruik deze constanten:
//!
//! - In de doc-comment van een Rust-functie die een norm-regel implementeert
//!   (bv. `/// Implementeert [NTA_8800_2025_FORMULE8_3]`)
//! - In audit-logs en rapport-traceability (zodat een rekenresultaat terug te
//!   leiden is naar een norm-regel)
//! - In foutmeldingen die verwijzen naar een specifieke norm-eis
//!
//! # Naamgevings-conventie
//!
//! | Soort | Patroon | Voorbeeld |
//! |---|---|---|
//! | Formule in hoofdstuk | `NTA_8800_2025_FORMULE{h_sub}` | `NTA_8800_2025_FORMULE8_3` |
//! | Formule sub-genummerd | letter-suffix | `NTA_8800_2025_FORMULE8_3A` |
//! | Tabel in hoofdstuk | `NTA_8800_2025_TABEL{h_nr}` | `NTA_8800_2025_TABEL17_1` |
//! | Figuur | `NTA_8800_2025_FIGUUR{h_nr}` | `NTA_8800_2025_FIGUUR7_2` |
//! | Paragraaf | `NTA_8800_2025_PARAG{h_sub}` | `NTA_8800_2025_PARAG6_2` |
//! | Bijlage formule | `NTA_8800_2025_BIJLAGE_{L}_FORMULE{nr}` | `NTA_8800_2025_BIJLAGE_AA_FORMULE3` |
//! | Bijlage tabel | `NTA_8800_2025_BIJLAGE_{L}_TABEL{nr}` | `NTA_8800_2025_BIJLAGE_G_TABEL2` |
//! | Bijlage paragraaf | `NTA_8800_2025_BIJLAGE_{L}_PARAG{nr}` | `NTA_8800_2025_BIJLAGE_I_PARAG2` |
//! | C1:2026 correctie | suffix `_C1` | `NTA_8800_2025_PARAG_P_5_3_2_C1` |
//!
//! # Waarom dit patroon
//!
//! Het parallel-patroon (zie `isso51-core::formulas`) heeft zich bewezen voor
//! audit-traceability in de ISSO 51 engine. Norm-nummers blijven stabiel onder
//! code-refactors: de constante fungeert als onafhankelijke identifier. Een
//! `grep` op `NTA_8800_2025_FORMULE8_3` vindt alle call-sites voor die formule,
//! ook als de Rust-functienaam verandert.
//!
//! # Correcties (C1:2026)
//!
//! NTA 8800:2025+C1:2026 bevat twee officiële correcties ten opzichte van
//! NTA 8800:2025:
//!
//! - **P.3.2** (onder "Distributie van warmte of koude via een distributienet"),
//!   punt 6): tekst aangepast.
//! - **P.5.3.2**: vijfde alinea aangepast.
//!
//! Identifiers die naar deze gecorrigeerde passages verwijzen krijgen het
//! suffix `_C1` (bv. `NTA_8800_2025_PARAG_P_5_3_2_C1`).

// ---------------------------------------------------------------------------
// Paragrafen — hoofdtekst
// ---------------------------------------------------------------------------

/// Termen, definities en grootheden — eenheden en notatie-conventies.
///
/// Gebruikt in: [`crate::units`].
pub const NTA_8800_2025_PARAG3: &str = "nta_8800_2025_parag3";

/// Gebouwbegrenzing en schematisering — overkoepelende zonerings-paragraaf.
///
/// Gebruikt in: [`crate::zoning`].
pub const NTA_8800_2025_PARAG6: &str = "nta_8800_2025_parag6";

/// Rekenzone — definitie van een thermisch samenhangend gebied.
///
/// Gebruikt in: [`crate::zoning::rekenzone`], [`crate::zoning`].
pub const NTA_8800_2025_PARAG6_2: &str = "nta_8800_2025_parag6_2";

/// Energiefunctieruimte (EFR) — binnenklimaat-cluster binnen een rekenzone.
///
/// Gebruikt in: [`crate::zoning::energy_function_room`], [`crate::zoning`].
pub const NTA_8800_2025_PARAG6_3: &str = "nta_8800_2025_parag6_3";

/// Transmissie — overkoepelende paragraaf voor warmtetransport door de schil.
///
/// Gebruikt in: [`crate::units`], [`crate::geometry::thermal_bridge`].
pub const NTA_8800_2025_PARAG8: &str = "nta_8800_2025_parag8";

/// Constructie-R-waarde — som van laagweerstanden plus oppervlakteweerstanden.
///
/// Gebruikt in: [`crate::geometry::construction`].
pub const NTA_8800_2025_PARAG8_3: &str = "nta_8800_2025_parag8_3";

/// Raam — samengestelde U-waarde (glas + kozijn) en g-waarde.
///
/// Gebruikt in: [`crate::geometry::window`].
pub const NTA_8800_2025_PARAG8_5: &str = "nta_8800_2025_parag8_5";

/// Koudebruggen — lineaire en puntkoudebruggen met ψ/χ-waarden.
///
/// Gebruikt in: [`crate::geometry::thermal_bridge`].
pub const NTA_8800_2025_PARAG8_6: &str = "nta_8800_2025_parag8_6";

/// Energiestromen — rekenkundige boekhouding van energievraag en -levering.
///
/// Gebruikt in: [`crate::units`].
pub const NTA_8800_2025_PARAG9: &str = "nta_8800_2025_parag9";

// ---------------------------------------------------------------------------
// Paragrafen — correcties C1:2026
// ---------------------------------------------------------------------------

/// Bijlage P §3.2 punt 6) — distributie van warmte of koude via een
/// distributienet, tekst-correctie in C1:2026.
pub const NTA_8800_2025_PARAG_P_3_2_C1: &str = "nta_8800_2025_parag_p_3_2_c1";

/// Bijlage P §5.3.2 vijfde alinea — tekst-correctie in C1:2026.
pub const NTA_8800_2025_PARAG_P_5_3_2_C1: &str = "nta_8800_2025_parag_p_5_3_2_c1";

// ---------------------------------------------------------------------------
// Bijlagen
// ---------------------------------------------------------------------------

/// Bijlage D — gebruiksprofielen per gebruiksfunctie.
///
/// Gebruikt in: [`crate::zoning::usage_function`].
pub const NTA_8800_2025_BIJLAGE_D: &str = "nta_8800_2025_bijlage_d";

/// Bijlage E — klimaatdata (KNMI-referentie De Bilt): maandelijkse
/// buitentemperatuur en zoninstraling per oriëntatie.
///
/// Gebruikt in: [`crate::climate`], [`crate::location`], [`crate::units`].
pub const NTA_8800_2025_BIJLAGE_E: &str = "nta_8800_2025_bijlage_e";

/// Bijlage G — default U- en g-waarden voor beglazing en kozijnen.
///
/// Gebruikt in: [`crate::geometry::window`].
pub const NTA_8800_2025_BIJLAGE_G: &str = "nta_8800_2025_bijlage_g";

/// Bijlage I — default ψ-waarden voor lineaire koudebruggen per categorie.
///
/// Gebruikt in: [`crate::geometry::thermal_bridge`], [`crate::units`].
pub const NTA_8800_2025_BIJLAGE_I: &str = "nta_8800_2025_bijlage_i";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Alle gedefinieerde constanten, centraal beheerd voor de drie sanity
    /// tests hieronder. Bij toevoegen van een nieuwe constante: ook hier
    /// registreren.
    const ALL: &[&str] = &[
        // Paragrafen — hoofdtekst
        NTA_8800_2025_PARAG3,
        NTA_8800_2025_PARAG6,
        NTA_8800_2025_PARAG6_2,
        NTA_8800_2025_PARAG6_3,
        NTA_8800_2025_PARAG8,
        NTA_8800_2025_PARAG8_3,
        NTA_8800_2025_PARAG8_5,
        NTA_8800_2025_PARAG8_6,
        NTA_8800_2025_PARAG9,
        // Paragrafen — C1:2026 correcties
        NTA_8800_2025_PARAG_P_3_2_C1,
        NTA_8800_2025_PARAG_P_5_3_2_C1,
        // Bijlagen
        NTA_8800_2025_BIJLAGE_D,
        NTA_8800_2025_BIJLAGE_E,
        NTA_8800_2025_BIJLAGE_G,
        NTA_8800_2025_BIJLAGE_I,
    ];

    /// Canonieke strings moeten uniek zijn — duplicaten verraden copy-paste
    /// fouten die de audit-trail ondermijnen.
    #[test]
    fn test_canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(
            set.len(),
            ALL.len(),
            "Dubbele canonieke string gevonden in references.rs"
        );
    }

    /// Alle constanten volgen het prefix-patroon `nta_8800_2025_`.
    #[test]
    fn test_all_constants_have_prefix() {
        for id in ALL {
            assert!(
                id.starts_with("nta_8800_2025_"),
                "Constante {id:?} mist prefix \"nta_8800_2025_\""
            );
        }
    }

    /// Canonieke strings mogen geen whitespace bevatten — dat breekt
    /// log-parsing en URL-encoding van de identifier.
    #[test]
    fn test_no_whitespace_in_canonical_strings() {
        for id in ALL {
            assert!(
                !id.chars().any(char::is_whitespace),
                "Constante {id:?} bevat whitespace"
            );
        }
    }
}
