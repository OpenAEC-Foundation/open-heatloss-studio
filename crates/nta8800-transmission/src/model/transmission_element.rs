//! Transmissie-element: binding tussen een constructie en een gevelvlak.
//!
//! Elk TransmissionElement representeert één vlak in de gebouwomhulling uit
//! NTA 8800 §8.1 figuur 8.1 — hetzij een ondoorschijnende constructie, een raam,
//! of een opake deur/opening. Het element koppelt een U-waarde (of R-waarde via
//! Construction) aan een oppervlakte en een boundary-type.
//!
//! De U-waarde is expliciet in het element opgeslagen om ramen en deuren (die
//! een samengestelde U-waarde hebben — glas + kozijn — uit bijlage G of een
//! fabrikant-declaratie) direct te kunnen meenemen zonder omweg via een
//! [`nta8800_model::Construction`] met fictieve lagen. Voor ondoorschijnende
//! constructies rekent de consumer de U-waarde zelf uit via
//! [`Construction::u_value`](nta8800_model::Construction::u_value) en levert
//! die hier aan.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::units::{Area, ThermalTransmittance};

use super::boundary::BoundaryType;

/// Eén vlak in de gebouwomhulling met zijn thermische eigenschappen en grens.
///
/// # Velden
///
/// - `id` — projectbreed unieke identifier (voor audit-logs en
///   resultaat-breakdown).
/// - `area` — geprojecteerde oppervlakte in m² volgens bijlage K.1.2.
/// - `u_value` — warmtedoorgangscoëfficiënt U in W/(m²·K). Voor ramen en deuren
///   is dit de samengestelde waarde; voor ondoorschijnende delen wordt deze
///   berekend als `1 / Construction::r_total()`.
/// - `boundary_type` — classificatie van de grens (zie [`BoundaryType`]).
/// - `construction_id` — optionele verwijzing naar de
///   [`Construction`](nta8800_model::Construction) waaruit de U-waarde is
///   afgeleid. Puur documentair; de berekening leunt op `u_value` direct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransmissionElement {
    /// Unieke id binnen het project.
    pub id: String,
    /// Geprojecteerde oppervlakte in m².
    pub area: Area,
    /// Warmtedoorgangscoëfficiënt U in W/(m²·K).
    pub u_value: ThermalTransmittance,
    /// Type grens aan de buitenzijde.
    pub boundary_type: BoundaryType,
    /// Optionele referentie naar een `Construction` (alleen documentair).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub construction_id: Option<String>,
}

impl TransmissionElement {
    /// Warmteoverdrachtcoëfficiënt `A · U` in W/K.
    ///
    /// Dit is de bijdrage van dit enkele vlak aan de sommatie in formule (8.1):
    /// `H_D = Σ(A_T;i · U_C;i) + …`
    #[must_use]
    pub fn conductance(&self) -> f64 {
        self.area * self.u_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conductance_multiplies_area_and_u() {
        let el = TransmissionElement {
            id: "wall-1".into(),
            area: 10.0,
            u_value: 0.2,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        };
        assert!((el.conductance() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn serde_round_trip_outdoor() {
        let el = TransmissionElement {
            id: "wall-1".into(),
            area: 12.5,
            u_value: 0.24,
            boundary_type: BoundaryType::Outdoor,
            construction_id: Some("c-wall-rc4".into()),
        };
        let json = serde_json::to_string(&el).unwrap();
        let back: TransmissionElement = serde_json::from_str(&json).unwrap();
        assert_eq!(el, back);
    }

    #[test]
    fn serde_round_trip_unheated_with_id() {
        let el = TransmissionElement {
            id: "wall-to-garage".into(),
            area: 8.0,
            u_value: 0.5,
            boundary_type: BoundaryType::UnheatedSpace {
                id: "garage".into(),
            },
            construction_id: None,
        };
        let json = serde_json::to_string(&el).unwrap();
        let back: TransmissionElement = serde_json::from_str(&json).unwrap();
        assert_eq!(el, back);
    }

    #[test]
    fn construction_id_is_skipped_when_none() {
        let el = TransmissionElement {
            id: "w".into(),
            area: 1.0,
            u_value: 1.0,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        };
        let json = serde_json::to_string(&el).unwrap();
        assert!(
            !json.contains("construction_id"),
            "construction_id moet ontbreken in JSON wanneer None: {json}"
        );
    }
}
