//! Norm-identifier constanten voor `nta8800-geometry`.
//!
//! Conventie: zie [`nta8800_model::references`] module-documentatie. Deze
//! constanten verwijzen naar H.6 Gebouwbegrenzing en bijlage K Oppervlakte-
//! en lengtebepaling.

// ---------------------------------------------------------------------------
// H.6 Gebouwbegrenzing
// ---------------------------------------------------------------------------

/// H.6 Gebouwbegrenzing en schematisering — overkoepelende paragraaf.
pub const NTA_8800_2025_PARAG6: &str = "nta_8800_2025_parag6";

/// §6.1 Principe — stappenplan (4 stappen) voor gebouwbegrenzing.
pub const NTA_8800_2025_PARAG6_1: &str = "nta_8800_2025_parag6_1";

/// §6.2 Benoemen gebruiksfuncties (stap 1) — onderscheid volgens Bouwbesluit
/// 2012 / Bbl.
pub const NTA_8800_2025_PARAG6_2: &str = "nta_8800_2025_parag6_2";

/// §6.3 Bepaling gebouwbegrenzing (stap 2) — thermische zone (verwarmd
/// gedeelte) vs. aangrenzende ruimten.
pub const NTA_8800_2025_PARAG6_3: &str = "nta_8800_2025_parag6_3";

/// §6.4 Indeling in klimatiseringszones (stap 3).
pub const NTA_8800_2025_PARAG6_4: &str = "nta_8800_2025_parag6_4";

/// §6.5 Indeling in rekenzones (stap 4).
pub const NTA_8800_2025_PARAG6_5: &str = "nta_8800_2025_parag6_5";

/// §6.5.2 Indelingsvoorschrift — woonfunctie mag geen andere gebruiksfuncties
/// bevatten; voorwaarden voor het samenvoegen van delen in één rekenzone.
pub const NTA_8800_2025_PARAG6_5_2: &str = "nta_8800_2025_parag6_5_2";

/// §6.8 Verliesoppervlakte — formule (6.3): som van geprojecteerde
/// oppervlakten maal weegfactor per type begrenzing.
pub const NTA_8800_2025_PARAG6_8: &str = "nta_8800_2025_parag6_8";

/// §6.9 Geprojecteerde oppervlakten van scheidingsconstructies — verwijst
/// door naar K.1.3 en K.2.
pub const NTA_8800_2025_PARAG6_9: &str = "nta_8800_2025_parag6_9";

// ---------------------------------------------------------------------------
// Bijlage K — Oppervlakte- en lengtebepaling
// ---------------------------------------------------------------------------

/// Bijlage K — Bepaling oppervlakte van vlakvormige en lengte van lijnvormige
/// elementen (normatief).
pub const NTA_8800_2025_BIJLAGE_K: &str = "nta_8800_2025_bijlage_k";

/// Bijlage K.1 — Schematiseringsregels voor het berekenen van de oppervlakte
/// van vlakvormige elementen en lijnvormige warmteverliezen (ψ).
pub const NTA_8800_2025_BIJLAGE_K_1: &str = "nta_8800_2025_bijlage_k_1";

/// Bijlage K.1.2 — Oppervlakte van een (constructie)onderdeel `A_con` voor
/// de bepaling van de warmtedoorgangscoëfficiënt `U`.
///
/// Conventie: binnenwerks gemeten, op twee decimalen nauwkeurig, begrensd
/// door de als adiabatisch beschouwde afsnijvlakken van het onderdeel.
pub const NTA_8800_2025_BIJLAGE_K_1_2: &str = "nta_8800_2025_bijlage_k_1_2";

/// Bijlage K.1.3 — Geprojecteerde oppervlakte `A_T` voor scheidings-
/// constructies (dichte vlakken, ramen, deuren, kozijnen, vloeren grenzend
/// aan grond- of kruipruimte).
///
/// Conventie: buitenwerks gemeten (behalve grondvloer: binnenwerks), ook op
/// twee decimalen nauwkeurig.
pub const NTA_8800_2025_BIJLAGE_K_1_3: &str = "nta_8800_2025_bijlage_k_1_3";

/// Bijlage K.1.4 — Principes voor afsnijvlakken (als adiabatisch
/// verondersteld) die de begrenzing van een onderdeel bepalen.
pub const NTA_8800_2025_BIJLAGE_K_1_4: &str = "nta_8800_2025_bijlage_k_1_4";

/// Bijlage K.2 — Geometrische karakteristieken van ramen/deuren:
/// raamoppervlakte `A_w = A_fr + A_gl`, zichtbare beglaasde oppervlakte,
/// kozijnoppervlakte (in- en uitwendig).
pub const NTA_8800_2025_BIJLAGE_K_2: &str = "nta_8800_2025_bijlage_k_2";

/// Bijlage K.2 formule voor raamoppervlakte: `A_w = A_fr + A_gl` (of
/// `A_fr + A_p` voor ondoorschijnend paneel).
pub const NTA_8800_2025_BIJLAGE_K_2_RAAMOPPERVLAKTE: &str =
    "nta_8800_2025_bijlage_k_2_raamoppervlakte";

/// Projectie van een schuin vlak op het horizontale grondvlak:
/// `A_horizontaal = A_hellend · cos(tilt)`. Volgt uit de definitie van
/// geprojecteerde oppervlakte in K.1.3 (denkbeeldig plat vlak).
pub const NTA_8800_2025_BIJLAGE_K_PROJECTIE_HORIZONTAAL: &str =
    "nta_8800_2025_bijlage_k_projectie_horizontaal";

/// Projectie van een schuin vlak op het verticale vlak:
/// `A_verticaal = A_hellend · sin(tilt)`. Gebruikt bij gevel-in-dak
/// constructies en onderscheid wand/dak (§6.8 dakoppervlakte = vlakken met
/// helling ≥ 15° t.o.v. de verticaal).
pub const NTA_8800_2025_BIJLAGE_K_PROJECTIE_VERTICAAL: &str =
    "nta_8800_2025_bijlage_k_projectie_verticaal";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Centraal register van alle in deze module gedefinieerde constanten.
    /// Bij toevoegen van een constante: óók hier registreren.
    const ALL: &[&str] = &[
        NTA_8800_2025_PARAG6,
        NTA_8800_2025_PARAG6_1,
        NTA_8800_2025_PARAG6_2,
        NTA_8800_2025_PARAG6_3,
        NTA_8800_2025_PARAG6_4,
        NTA_8800_2025_PARAG6_5,
        NTA_8800_2025_PARAG6_5_2,
        NTA_8800_2025_PARAG6_8,
        NTA_8800_2025_PARAG6_9,
        NTA_8800_2025_BIJLAGE_K,
        NTA_8800_2025_BIJLAGE_K_1,
        NTA_8800_2025_BIJLAGE_K_1_2,
        NTA_8800_2025_BIJLAGE_K_1_3,
        NTA_8800_2025_BIJLAGE_K_1_4,
        NTA_8800_2025_BIJLAGE_K_2,
        NTA_8800_2025_BIJLAGE_K_2_RAAMOPPERVLAKTE,
        NTA_8800_2025_BIJLAGE_K_PROJECTIE_HORIZONTAAL,
        NTA_8800_2025_BIJLAGE_K_PROJECTIE_VERTICAAL,
    ];

    #[test]
    fn canonical_strings_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(
            set.len(),
            ALL.len(),
            "Dubbele canonieke string gevonden in references.rs"
        );
    }

    #[test]
    fn all_constants_have_nta_prefix() {
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
