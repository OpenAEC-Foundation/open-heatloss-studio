//! H_A — warmteverlies naar aangrenzende verwarmde rekenzone (§8.5).
//!
//! NTA 8800 stelt `H_A;mi = 0` (de norm verwaarloost warmtetransport tussen
//! aangrenzende verwarmde zones). Deze module biedt een **opt-in** pad via
//! NEN-EN-ISO 13789:2017 7.6, ontsloten als formules (8.60)/(8.61) in de
//! opmerking onder §8.5:
//!
//! ```text
//! H_A;mi   = H_D;ia · b_A;mi                       (8.60)
//! b_A;mi  = (θ_i − θ_a) / (θ_i − θ_e;mi)           (8.61)
//! ```
//!
//! De orchestrator in [`super`] activeert deze berekening per aangrenzende
//! zone alleen wanneer een maandprofiel in de map `adjacent_zone_temperatures`
//! staat. Zonder profile blijft de NTA-default `H_A = 0` van kracht.

use std::collections::HashMap;

use crate::model::{BoundaryType, TransmissionElement};

/// Groepeer `A · U` over alle elementen met [`BoundaryType::AdjacentZone`]
/// per zone-id.
///
/// Retourneert `HashMap<zone_id, Σ(A·U)>`, oftewel `H_D;ia` per aangrenzende
/// zone. De temperatuur-correctie (`b_A;mi`) wordt maandelijks toegepast door
/// de orchestrator in [`crate::calculate_transmission`].
#[must_use]
pub fn conductance_per_adjacent_zone(elements: &[TransmissionElement]) -> HashMap<String, f64> {
    let mut per_zone: HashMap<String, f64> = HashMap::new();
    for el in elements {
        if let BoundaryType::AdjacentZone { id } = &el.boundary_type {
            *per_zone.entry(id.clone()).or_insert(0.0) += el.conductance();
        }
    }
    per_zone
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adj_el(id: &str, area: f64, u: f64, zone_id: &str) -> TransmissionElement {
        TransmissionElement {
            id: id.into(),
            area,
            u_value: u,
            boundary_type: BoundaryType::AdjacentZone { id: zone_id.into() },
            construction_id: None,
        }
    }

    #[test]
    fn empty_slice_returns_empty_map() {
        let res = conductance_per_adjacent_zone(&[]);
        assert!(res.is_empty());
    }

    #[test]
    fn single_element_single_zone() {
        let els = vec![adj_el("wall", 10.0, 0.5, "buur")];
        let res = conductance_per_adjacent_zone(&els);
        assert_eq!(res.len(), 1);
        assert!((res["buur"] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn multiple_elements_same_zone_sum() {
        let els = vec![
            adj_el("wall-1", 10.0, 0.5, "buur"),
            adj_el("wall-2", 4.0, 0.25, "buur"),
        ];
        let res = conductance_per_adjacent_zone(&els);
        // 5.0 + 1.0 = 6.0 W/K
        assert!((res["buur"] - 6.0).abs() < 1e-12);
    }

    #[test]
    fn multiple_zones_kept_separate() {
        let els = vec![
            adj_el("wall-n", 10.0, 0.5, "noord"),
            adj_el("wall-s", 8.0, 0.25, "zuid"),
        ];
        let res = conductance_per_adjacent_zone(&els);
        assert_eq!(res.len(), 2);
        assert!((res["noord"] - 5.0).abs() < 1e-12);
        assert!((res["zuid"] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn non_adjacent_elements_are_ignored() {
        let els = vec![
            adj_el("adj", 10.0, 0.5, "buur"),
            TransmissionElement {
                id: "outdoor".into(),
                area: 100.0,
                u_value: 1.0,
                boundary_type: BoundaryType::Outdoor,
                construction_id: None,
            },
        ];
        let res = conductance_per_adjacent_zone(&els);
        assert_eq!(res.len(), 1);
        assert!((res["buur"] - 5.0).abs() < 1e-12);
    }
}
