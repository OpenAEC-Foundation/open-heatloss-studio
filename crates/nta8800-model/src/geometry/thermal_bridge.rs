//! Koudebruggen — lineair (ψ) en punt (χ).
//!
//! NTA 8800 §8.6 + bijlage I. Default ψ-waarden per categorie worden later
//! in `nta8800-tables` ontsloten; hier definiëren we alleen de types en
//! categorie-indeling.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::units::{Length, LinearThermalTransmittance};

/// Indeling van koudebruggen conform NTA 8800 bijlage I.
///
/// De enum is `#[non_exhaustive]` zodat extra categorieën (nieuwe
/// detail-typen) geen breaking change vormen.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThermalBridgeCategory {
    /// Fundering / aansluiting grond.
    Fundering,
    /// Hoek in gevel (uitwendig of inwendig).
    HoekGevel,
    /// Aansluiting dak op gevel.
    AansluitingDakGevel,
    /// Aansluiting vloer op gevel (bv. begane grondvloer).
    AansluitingVloerGevel,
    /// Doorgaande constructie (bv. balkon dat door gevel loopt).
    DoorgaandeConstructie,
    /// Raamkader / kozijn-aansluiting.
    RaamKader,
    /// Overige, niet nader gespecificeerde koudebrug.
    Overig,
}

/// Lineaire koudebrug — warmteverlies langs een lijn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThermalBridgeLinear {
    /// Unieke identificatie.
    pub id: String,

    /// Lengte van de koudebrug in m.
    pub length: Length,

    /// Lineaire warmtedoorgangscoëfficiënt ψ in W/(m·K).
    pub psi: LinearThermalTransmittance,

    /// Categorie volgens bijlage I.
    pub category: ThermalBridgeCategory,
}

/// Puntkoudebrug — warmteverlies op één punt (bv. balkonanker, gevelanker).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThermalBridgePoint {
    /// Unieke identificatie.
    pub id: String,

    /// Puntwarmtedoorgangscoëfficiënt χ in W/K.
    pub chi: f64,

    /// Categorie volgens bijlage I.
    pub category: ThermalBridgeCategory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_bridge_serde_round_trip() {
        let b = ThermalBridgeLinear {
            id: "tb-lin-1".into(),
            length: 12.5,
            psi: 0.15,
            category: ThermalBridgeCategory::AansluitingVloerGevel,
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: ThermalBridgeLinear = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn point_bridge_serde_round_trip() {
        let b = ThermalBridgePoint {
            id: "tb-pt-1".into(),
            chi: 0.02,
            category: ThermalBridgeCategory::DoorgaandeConstructie,
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: ThermalBridgePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(b, back);
    }

    #[test]
    fn category_serde_snake_case() {
        let json = serde_json::to_string(&ThermalBridgeCategory::AansluitingDakGevel).unwrap();
        assert_eq!(json, "\"aansluiting_dak_gevel\"");
    }

    #[test]
    fn categories_are_distinct() {
        // sanity: geen twee varianten delen dezelfde serde-naam
        let names = [
            ThermalBridgeCategory::Fundering,
            ThermalBridgeCategory::HoekGevel,
            ThermalBridgeCategory::AansluitingDakGevel,
            ThermalBridgeCategory::AansluitingVloerGevel,
            ThermalBridgeCategory::DoorgaandeConstructie,
            ThermalBridgeCategory::RaamKader,
            ThermalBridgeCategory::Overig,
        ];
        let mut jsons: Vec<String> = names
            .iter()
            .map(|c| serde_json::to_string(c).unwrap())
            .collect();
        jsons.sort();
        jsons.dedup();
        assert_eq!(jsons.len(), 7);
    }
}
