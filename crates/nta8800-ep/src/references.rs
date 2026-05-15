//! Norm-identifier constanten voor `nta8800-ep`.
//!
//! Conventie: zie [`nta8800_model::references`]. Alle constanten zijn canonieke
//! strings voor audit-traceability — een `grep` op bv.
//! `NTA_8800_2025_FORMULE5_1` vindt alle call-sites voor de EP-score formule,
//! ook als de Rust-functienaam later verandert.

// ---------------------------------------------------------------------------
// Paragrafen — hoofdstuk 5 EP-score integratie
// ---------------------------------------------------------------------------

/// H.5 EP-score integratie — overkoepelend.
pub const NTA_8800_2025_PARAG5: &str = "nta_8800_2025_parag5";

/// §5.1 Primair energiegebruik per energiedrager.
pub const NTA_8800_2025_PARAG5_1: &str = "nta_8800_2025_parag5_1";

/// §5.2 Totaal primair energiegebruik en EP-score bepaling.
pub const NTA_8800_2025_PARAG5_2: &str = "nta_8800_2025_parag5_2";

/// §5.3 Hernieuwbaar aandeel en PV-saldering.
pub const NTA_8800_2025_PARAG5_3: &str = "nta_8800_2025_parag5_3";

/// §5.4 EP-label classificatie.
pub const NTA_8800_2025_PARAG5_4: &str = "nta_8800_2025_parag5_4";

// ---------------------------------------------------------------------------
// Bijlagen — primaire energiefactoren en CO2-beleidsfactoren
// ---------------------------------------------------------------------------

/// Bijlage Z — Primaire energiefactoren per energiedrager (2023 waarden).
pub const NTA_8800_2025_BIJLAGE_Z: &str = "nta_8800_2025_bijlage_z";

/// Bijlage AB — CO2-beleidsfactoren per energiedrager (2023 waarden).
pub const NTA_8800_2025_BIJLAGE_AB: &str = "nta_8800_2025_bijlage_ab";

// ---------------------------------------------------------------------------
// Formules — hoofdstuk 5
// ---------------------------------------------------------------------------

/// Formule (5.1) — Primair energiegebruik per dienst.
/// `E_P;dienst = Σ(Q_netto;dienst,drager × f_prim;drager)`
pub const NTA_8800_2025_FORMULE5_1: &str = "nta_8800_2025_formule5_1";

/// Formule (5.2) — Totaal primair energiegebruik.
/// `E_P;tot = E_P;heating + E_P;cooling + E_P;dhw + E_P;lighting + E_P;vent + E_P;automation`
pub const NTA_8800_2025_FORMULE5_2: &str = "nta_8800_2025_formule5_2";

/// Formule (5.3) — Specifiek primair energiegebruik (EP-score).
/// `E_P;tot,spec = E_P;tot / A_g [MJ/m²]`
pub const NTA_8800_2025_FORMULE5_3: &str = "nta_8800_2025_formule5_3";

/// Formule (5.4) — Hernieuwbaar aandeel berekening.
/// `f_renewable = min(1.0, E_PV;yield / E_P;tot)`
pub const NTA_8800_2025_FORMULE5_4: &str = "nta_8800_2025_formule5_4";

/// Formule (5.5) — CO2-uitstoot per dienst.
/// `CO2_dienst = Σ(Q_netto;dienst,drager × f_CO2;drager)`
pub const NTA_8800_2025_FORMULE5_5: &str = "nta_8800_2025_formule5_5";

// ---------------------------------------------------------------------------
// Tabellen — hoofdstuk 5 en bijlagen
// ---------------------------------------------------------------------------

/// Tabel 5.1 — EP-label drempels voor woonfuncties.
/// Kolommen: Label (A++++ t/m G), EP-score drempel [MJ/m²].
pub const NTA_8800_2025_TABEL5_1: &str = "nta_8800_2025_tabel5_1";

/// Tabel 5.2 — EP-label drempels voor utiliteitsgebouwen.
/// Kolommen: Label (A++++ t/m G), EP-score drempel [MJ/m²].
pub const NTA_8800_2025_TABEL5_2: &str = "nta_8800_2025_tabel5_2";

/// Tabel Z.1 — Primaire energiefactoren f_prim per energiedrager.
/// Kolommen: Energiedrager, f_prim [-].
pub const NTA_8800_2025_TABEL_Z1: &str = "nta_8800_2025_tabel_z1";

/// Tabel AB.1 — CO2-beleidsfactoren f_CO2 per energiedrager.
/// Kolommen: Energiedrager, f_CO2 [kg CO2/MJ].
pub const NTA_8800_2025_TABEL_AB1: &str = "nta_8800_2025_tabel_ab1";

// ---------------------------------------------------------------------------
// EP-score constanten
// ---------------------------------------------------------------------------

/// Primair energiegebruik totaal — symbool E_P;tot.
pub const EP_SCORE_SYMBOL_TOTAL: &str = "E_P;tot";

/// Primair energiegebruik niet-hernieuwbaar — symbool E_P;nb.
pub const EP_SCORE_SYMBOL_NONRENEWABLE: &str = "E_P;nb";

/// Specifiek primair energiegebruik — symbool E_P;tot,spec.
pub const EP_SCORE_SYMBOL_SPECIFIC: &str = "E_P;tot,spec";

/// Gebruiksoppervlakte — symbool A_g.
pub const BUILDING_AREA_SYMBOL: &str = "A_g";

/// Hernieuwbaar aandeel — symbool f_renewable.
pub const RENEWABLE_FRACTION_SYMBOL: &str = "f_renewable";

// ---------------------------------------------------------------------------
// Normverwijzingen
// ---------------------------------------------------------------------------

/// NTA 8800:2025 — Energieprestatie van gebouwen, hoofdnorm.
pub const NTA_8800_2025: &str = "nta_8800_2025";

/// NTA 8800 C1:2026 — Correctieblad met updates op oorspronkelijke tabellen.
pub const NTA_8800_2025_C1_2026: &str = "nta_8800_2025_c1_2026";

/// NEN 7120:2022 — Energieprestatie van gebouwen, bepalingsmethode.
pub const NEN_7120_2022: &str = "nen_7120_2022";

// ---------------------------------------------------------------------------
// Tests — sanity checks op de canonieke strings
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG5,
        NTA_8800_2025_PARAG5_1,
        NTA_8800_2025_PARAG5_2,
        NTA_8800_2025_PARAG5_3,
        NTA_8800_2025_PARAG5_4,
        NTA_8800_2025_BIJLAGE_Z,
        NTA_8800_2025_BIJLAGE_AB,
        NTA_8800_2025_FORMULE5_1,
        NTA_8800_2025_FORMULE5_2,
        NTA_8800_2025_FORMULE5_3,
        NTA_8800_2025_FORMULE5_4,
        NTA_8800_2025_FORMULE5_5,
        NTA_8800_2025_TABEL5_1,
        NTA_8800_2025_TABEL5_2,
        NTA_8800_2025_TABEL_Z1,
        NTA_8800_2025_TABEL_AB1,
        EP_SCORE_SYMBOL_TOTAL,
        EP_SCORE_SYMBOL_NONRENEWABLE,
        EP_SCORE_SYMBOL_SPECIFIC,
        BUILDING_AREA_SYMBOL,
        RENEWABLE_FRACTION_SYMBOL,
        NTA_8800_2025,
        NTA_8800_2025_C1_2026,
        NEN_7120_2022,
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
    fn all_constants_have_valid_prefix() {
        for id in ALL {
            // Norm-verwijzingen en symbolen hebben eigen prefix
            let valid_prefixes = ["nta_8800_2025", "nen_7120_", "E_P;", "A_g", "f_renewable"];
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
