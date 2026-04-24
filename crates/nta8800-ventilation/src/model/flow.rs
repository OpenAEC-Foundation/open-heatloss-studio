//! [`AirFlow`] — rekenkundige luchtstromen per rekenzone.
//!
//! Alle stromen in **m³/h** conform NTA 8800 §11.2 (de norm rekent intern
//! in m³/h, niet m³/s — let op bij het overnemen van oudere NEN 1087 data).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Luchtstromen per rekenzone, in **m³/h**.
///
/// Conform NTA 8800 §11.2 stappen 1–5 (stroomtypen `q_V;ODA;req`,
/// `q_V;SUP;eff`, `q_V;ETA;eff`, `q_V;lea`). Voor V1 modelleren we:
/// - mechanische toevoer (supply fan)
/// - mechanische afvoer (extract fan)
/// - infiltratie (q_V;lea)
///
/// Spui (`q_V;argI`), ventilatieve koeling (`q_V;argII`) en
/// verbrandingslucht (`q_V;comb`) zijn **V2-scope** — expliciet niet
/// gemodelleerd.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AirFlow {
    /// Mechanische toevoerluchtstroom `q_V;SUP;eff` in m³/h.
    ///
    /// 0 bij systemen A en C (geen mechanische toevoer).
    pub mechanical_supply: f64,

    /// Mechanische afvoerluchtstroom `q_V;ETA;eff` in m³/h.
    ///
    /// 0 bij systemen A en B (geen mechanische afvoer).
    pub mechanical_exhaust: f64,

    /// Infiltratie-luchtstroom `q_V;lea` in m³/h.
    ///
    /// Uit metingen (qv;10 luchtdicht-klasse × omhullingsoppervlak ×
    /// stromingsweerstand) of user-supplied forfaitair.
    pub infiltration: f64,
}

impl AirFlow {
    /// Bouw een `AirFlow` zonder validatie (debiet-controles gebeuren in
    /// [`crate::calculate_ventilation`]).
    #[must_use]
    pub const fn new(mechanical_supply: f64, mechanical_exhaust: f64, infiltration: f64) -> Self {
        Self {
            mechanical_supply,
            mechanical_exhaust,
            infiltration,
        }
    }

    /// Nul-stroom — handig als default voor systeem A.
    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Totale mechanische stroom = supply + exhaust.
    ///
    /// Gebruikt als input voor ventilator-energie bij systeem D
    /// (beide ventilatoren actief).
    #[must_use]
    pub fn total_mechanical(&self) -> f64 {
        self.mechanical_supply + self.mechanical_exhaust
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_airflow_is_all_zero() {
        let zero = AirFlow::zero();
        assert!(zero.mechanical_supply.abs() < 1e-9);
        assert!(zero.mechanical_exhaust.abs() < 1e-9);
        assert!(zero.infiltration.abs() < 1e-9);
    }

    #[test]
    fn total_mechanical_sums_supply_and_exhaust() {
        let flow = AirFlow::new(100.0, 90.0, 20.0);
        assert!((flow.total_mechanical() - 190.0).abs() < 1e-9);
    }

    #[test]
    fn serde_round_trip() {
        let flow = AirFlow::new(120.5, 115.0, 25.3);
        let json = serde_json::to_string(&flow).unwrap();
        let back: AirFlow = serde_json::from_str(&json).unwrap();
        assert_eq!(flow, back);
    }
}
