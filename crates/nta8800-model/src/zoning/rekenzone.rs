//! `Rekenzone` — thermisch homogeen deel van het gebouw.
//!
//! NTA 8800 §6.2 — een rekenzone groepeert één of meerdere
//! energiefunctieruimten met dezelfde thermische karakteristieken. Alle
//! schil-elementen (constructies, ramen, openingen, koudebruggen) worden
//! op rekenzone-niveau toegewezen.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::units::Area;

/// Rekenzone volgens NTA 8800 §6.2.
///
/// De verwijzingen naar constructies, ramen, openingen en koudebruggen zijn
/// id-lijsten; de daadwerkelijke objecten leven op project-manifest niveau.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Rekenzone {
    /// Unieke identificatie.
    pub id: String,

    /// Menselijk leesbare naam.
    pub name: String,

    /// Id van het [`super::Gebouw`] waar deze zone onder valt.
    pub gebouw_id: String,

    /// Gebruiksoppervlak `A_g` van de zone in m².
    pub floor_area: Area,

    /// Bruto volume van de zone in m³ (buitenwerks gemeten).
    pub volume: f64,

    /// Id's van de [`super::EnergiefunctieRuimte`]-objecten in deze zone.
    pub efr_ids: Vec<String>,

    /// Id's van [`crate::geometry::Construction`]-objecten die aan deze zone
    /// zijn toegewezen.
    pub constructions: Vec<String>,

    /// Id's van [`crate::geometry::Window`]-objecten in deze zone.
    pub windows: Vec<String>,

    /// Id's van [`crate::geometry::Opening`]-objecten in deze zone.
    pub openings: Vec<String>,

    /// Id's van [`crate::geometry::ThermalBridgeLinear`]-objecten.
    pub thermal_bridges_linear: Vec<String>,

    /// Id's van [`crate::geometry::ThermalBridgePoint`]-objecten.
    pub thermal_bridges_point: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Rekenzone {
        Rekenzone {
            id: "rz1".into(),
            name: "Hoofdzone".into(),
            gebouw_id: "g1".into(),
            floor_area: 125.0,
            volume: 312.5,
            efr_ids: vec!["efr1".into()],
            constructions: vec!["c1".into(), "c2".into()],
            windows: vec!["w1".into()],
            openings: vec![],
            thermal_bridges_linear: vec!["tb1".into()],
            thermal_bridges_point: vec![],
        }
    }

    #[test]
    fn rekenzone_serde_round_trip() {
        let rz = sample();
        let json = serde_json::to_string(&rz).unwrap();
        let back: Rekenzone = serde_json::from_str(&json).unwrap();
        assert_eq!(rz, back);
    }

    #[test]
    fn rekenzone_id_references_present() {
        let rz = sample();
        assert_eq!(rz.constructions.len(), 2);
        assert_eq!(rz.efr_ids[0], "efr1");
    }
}
