//! Het `Gebouw` type — top-level container zonder installatie-types.
//!
//! Installaties (verwarming, ventilatie, tapwater, PV, verlichting,
//! bevochtiging, automation) worden **niet** in dit type opgenomen; die
//! horen in de thema-crates (`nta8800-heating`, `nta8800-ventilation`, etc.)
//! en worden in het samengestelde project-manifest via id-referenties
//! gekoppeld.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::location::Location;
use crate::zoning::usage_function::UsageFunction;

/// Top-level gebouw — metadata + locatie + verwijzingen naar rekenzones.
///
/// Dit type bevat **alleen** het thermische schil- en zoneringsmodel.
/// Installatie-types (`HeatingSystem`, `VentilationSystem`, DHW, PV, Lighting,
/// Humidity, Automation) leven in afzonderlijke thema-crates en worden
/// gekoppeld via id-strings in het project-manifest dat de rekencrates
/// verwerken.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Gebouw {
    /// Unieke identificatie van het gebouw binnen het project.
    pub id: String,

    /// Menselijk leesbare naam (bv. `"Appartementencomplex De Hoven"`).
    pub name: String,

    /// Primaire gebruiksfunctie volgens Bbl.
    pub usage_function: UsageFunction,

    /// Bouwjaar (indien bekend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub construction_year: Option<u32>,

    /// Locatie (postcode, optionele coördinaten, klimaatzone).
    pub location: Location,

    /// Id's van de [`super::Rekenzone`]-objecten die onder dit gebouw vallen.
    pub rekenzone_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::ClimateZone;

    fn sample() -> Gebouw {
        Gebouw {
            id: "g1".into(),
            name: "Testgebouw".into(),
            usage_function: UsageFunction::Woonfunctie,
            construction_year: Some(2020),
            location: Location {
                postcode: "3511AB".into(),
                coordinates: None,
                climate_zone: ClimateZone::DeBilt,
            },
            rekenzone_ids: vec!["rz1".into(), "rz2".into()],
        }
    }

    #[test]
    fn gebouw_serde_round_trip() {
        let g = sample();
        let json = serde_json::to_string(&g).unwrap();
        let back: Gebouw = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }

    #[test]
    fn construction_year_optional() {
        let mut g = sample();
        g.construction_year = None;
        let json = serde_json::to_string(&g).unwrap();
        // field skipped als None
        assert!(!json.contains("construction_year"));
    }

    #[test]
    fn gebouw_references_are_plain_ids() {
        let g = sample();
        assert_eq!(g.rekenzone_ids.len(), 2);
        assert_eq!(g.rekenzone_ids[0], "rz1");
    }
}
