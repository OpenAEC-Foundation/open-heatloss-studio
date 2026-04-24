//! Norm-identifier constanten voor `nta8800-tables`.
//!
//! Conventie: zie de module-documentatie van [`nta8800_model::references`] voor
//! het naamgevings-patroon. Elke constante in deze module verwijst naar een
//! specifieke tabel of bijlage in NTA 8800:2025+C1:2026 die in deze crate als
//! data is geïmplementeerd.

// ---------------------------------------------------------------------------
// Hoofdstuk 7 — Effectieve interne warmtecapaciteit
// ---------------------------------------------------------------------------

/// §7.7 — Effectieve interne warmtecapaciteit (`C_m;int;eff;zi`).
///
/// Overkoepelende paragraaf die de forfaitaire methode (tabellen 7.10/7.11/
/// 7.12 + formule 7.45) en de detailmethode (bijlage B) introduceert.
///
/// Bron: PDF p. 204.
///
/// Gebruikt in: [`crate::thermal_capacity`].
pub const NTA_8800_2025_PARAG7_7: &str = "nta_8800_2025_parag7_7";

/// Tabel 7.10 — Forfaitaire waarden voor de specifieke interne
/// warmtecapaciteit `D_m;int;eff;zi` in kJ/(m²·K) per combinatie van
/// vloer-massaklasse, wand-massaklasse en plafondtype.
///
/// Bron: PDF p. 205.
///
/// Gebruikt in: [`crate::thermal_capacity`].
pub const NTA_8800_2025_TABEL7_10: &str = "nta_8800_2025_tabel7_10";

/// Tabel 7.11 — Specificatie van het type bouwwijze voor **vloeren** ten
/// behoeve van de bepaling van de specifieke interne warmtecapaciteit
/// (Licht / Zwaar / Zeer zwaar).
///
/// Bron: PDF p. 206.
///
/// Gebruikt in: [`crate::thermal_capacity`].
pub const NTA_8800_2025_TABEL7_11: &str = "nta_8800_2025_tabel7_11";

/// Tabel 7.12 — Specificatie van het type bouwwijze voor **wanden** ten
/// behoeve van de bepaling van de specifieke interne warmtecapaciteit
/// (Licht / Zwaar / Zeer zwaar).
///
/// Bron: PDF p. 206.
///
/// Gebruikt in: [`crate::thermal_capacity`].
pub const NTA_8800_2025_TABEL7_12: &str = "nta_8800_2025_tabel7_12";

/// Formule 7.45 — `C_m;int;eff;zi = D_m;int;eff;zi × 1000 × A_g;zi`.
///
/// Berekent de effectieve interne warmtecapaciteit van een rekenzone in
/// J/K op basis van de specifieke interne warmtecapaciteit (tabel 7.10) en
/// de gebruiksoppervlakte van de rekenzone.
///
/// Bron: PDF p. 204.
///
/// Gebruikt in: [`crate::thermal_capacity`].
pub const NTA_8800_2025_FORMULE7_45: &str = "nta_8800_2025_formule7_45";

// ---------------------------------------------------------------------------
// Hoofdstuk 17 — Klimaatgegevens
// ---------------------------------------------------------------------------

/// H.17 — Klimaatgegevens, overkoepelende paragraaf.
///
/// Gebruikt in: [`crate::climate`].
pub const NTA_8800_2025_PARAG17: &str = "nta_8800_2025_parag17";

/// H.17 §17.2 — Getalswaarden (tabellen 17.1 en 17.2).
///
/// Gebruikt in: [`crate::climate::de_bilt`].
pub const NTA_8800_2025_PARAG17_2: &str = "nta_8800_2025_parag17_2";

/// Tabel 17.1 — Lengte van de maand `t_mi`, maandgemiddelde
/// buitenluchttemperatuur `ϑ_e;avg;mi`, ventilatieve-koeling-temperatuur
/// `ϑ_e;argII,mi`, windsnelheid `u_site;mi` en preheat WTW-temperatuur
/// `ϑ_ODA;preh;WTWC;zi;mi` voor referentieklimaat De Bilt.
///
/// Bron: PDF p. 690. De gegevens zijn overgenomen uit NEN 5060.
///
/// Gebruikt in: [`crate::climate::de_bilt`].
pub const NTA_8800_2025_TABEL17_1: &str = "nta_8800_2025_tabel17_1";

/// Tabel 17.2 — Maandgemiddelde totale opvallende zonnestraling `I_sol;mi`
/// in W/m² per combinatie van hellingshoek β en oriëntatie γ voor
/// referentieklimaat De Bilt; grondreflectiecoëfficiënt ρ = 0,2.
///
/// Bron: PDF p. 691-694. Overgenomen uit NEN 5060.
///
/// Gebruikt in: [`crate::climate::de_bilt`].
pub const NTA_8800_2025_TABEL17_2: &str = "nta_8800_2025_tabel17_2";

// ---------------------------------------------------------------------------
// Bijlagen
// ---------------------------------------------------------------------------

/// Bijlage X — Significante cijfers (klassenindeling van vermogens en andere
/// componenteigenschappen via afronding op twee significante cijfers, tabel X.1).
///
/// Bron: PDF p. 1129-1130.
///
/// Gebruikt in: [`crate::rounding`].
pub const NTA_8800_2025_BIJLAGE_X: &str = "nta_8800_2025_bijlage_x";

/// Bijlage X tabel X.1 — Significante cijfers voor klassenindeling.
///
/// 33 toegestane twee-cijferige basiswaarden: 10, 11, 12, ..., 20, 22, 24, ...
/// 40, 44, 48, 52, 56, 60, 65, 70, 75, 80, 85, 90, 95.
///
/// Gebruikt in: [`crate::rounding`].
pub const NTA_8800_2025_BIJLAGE_X_TABEL1: &str = "nta_8800_2025_bijlage_x_tabel1";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG7_7,
        NTA_8800_2025_TABEL7_10,
        NTA_8800_2025_TABEL7_11,
        NTA_8800_2025_TABEL7_12,
        NTA_8800_2025_FORMULE7_45,
        NTA_8800_2025_PARAG17,
        NTA_8800_2025_PARAG17_2,
        NTA_8800_2025_TABEL17_1,
        NTA_8800_2025_TABEL17_2,
        NTA_8800_2025_BIJLAGE_X,
        NTA_8800_2025_BIJLAGE_X_TABEL1,
    ];

    #[test]
    fn test_canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(
            set.len(),
            ALL.len(),
            "Dubbele canonieke string gevonden in references.rs"
        );
    }

    #[test]
    fn test_all_constants_have_prefix() {
        for id in ALL {
            assert!(
                id.starts_with("nta_8800_2025_"),
                "Constante {id:?} mist prefix \"nta_8800_2025_\""
            );
        }
    }

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
