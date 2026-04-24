//! Classificatie van de grens aan de buitenzijde van een transmissie-element.
//!
//! NTA 8800 §7.3.2 onderscheidt vier typen warmteoverdrachtcoëfficiënten:
//! - `H_D` direct naar buitenlucht (§8.2)
//! - `H_U` via onverwarmde ruimte (§8.4)
//! - `H_g` via grond (§8.3)
//! - `H_A` naar aangrenzende verwarmde zone (§8.5, standaard 0)
//!
//! Deze enum maakt het mogelijk om per gevel-, dak- of vloervlak expliciet te
//! bepalen welk regime van toepassing is zonder de volledige geometrische
//! opzet uit [`nta8800_model::zoning`] te dupliceren.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type grens waar een transmissie-element op uitkomt.
///
/// Varianten met een `id`-veld verwijzen naar een lookup-map die door de
/// consumer wordt aangeleverd (b-factor voor onverwarmde ruimte,
/// maandtemperatuur-profile voor aangrenzende zone). De id's zijn vrij
/// te kiezen door het project-manifest — er is geen hardcoded vocabulaire.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BoundaryType {
    /// Grenst direct aan buitenlucht. Draagt bij aan `H_D` (§8.2).
    Outdoor,

    /// Grenst aan een onverwarmde ruimte (garage, onverwarmde zolder,
    /// bergruimte). Draagt bij aan `H_U` (§8.4) via een dimensieloze
    /// reductiefactor `b_U`.
    UnheatedSpace {
        /// Referentie-id naar de onverwarmde ruimte; consumer levert de
        /// bijbehorende `b_U`-waarde in de `unheated_space_b_factors` map
        /// van [`crate::calculate_transmission`].
        id: String,
    },

    /// Vloer op/onder maaiveld of kelderwand — warmtetransport via de grond.
    /// Draagt bij aan `H_g;an` (§8.3), die in formule (7.14) apart wordt
    /// meegenomen met de **jaargemiddelde** buitentemperatuur.
    Ground,

    /// Grenst aan een aangrenzende verwarmde rekenzone (bv. naburige woning,
    /// andere gebruiksfunctie in hetzelfde gebouw). NTA 8800 stelt `H_A = 0`;
    /// consumers die een opt-in `H_A;mi`-berekening volgens NEN-EN-ISO 13789
    /// willen, geven via de `adjacent_zone_temperatures` map een maandprofiel
    /// mee.
    AdjacentZone {
        /// Referentie-id naar de aangrenzende zone.
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outdoor_serde_round_trip() {
        let b = BoundaryType::Outdoor;
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(json, r#"{"kind":"outdoor"}"#);
        let back: BoundaryType = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn unheated_space_serde_round_trip() {
        let b = BoundaryType::UnheatedSpace {
            id: "garage".into(),
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: BoundaryType = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn ground_serde_round_trip() {
        let b = BoundaryType::Ground;
        let json = serde_json::to_string(&b).unwrap();
        let back: BoundaryType = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn adjacent_zone_serde_round_trip() {
        let b = BoundaryType::AdjacentZone {
            id: "buurwoning".into(),
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: BoundaryType = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn outdoor_ne_ground() {
        assert_ne!(BoundaryType::Outdoor, BoundaryType::Ground);
    }
}
