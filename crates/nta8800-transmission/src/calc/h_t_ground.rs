//! H_g;an — warmteverlies via de grond (§8.3).
//!
//! V1 gebruikt de vereenvoudigde bijlage I.2.3 fallback: de consumer levert de
//! jaargemiddelde warmteoverdrachtcoëfficiënt naar de grond `h_g_an` (in W/K)
//! als één getal voor de gehele zone. De volledige NEN-EN-ISO 13370 bepaling
//! — gebaseerd op karakteristieke vloerbreedte `B'`, `U_bf`, `ψ_gr`, en
//! maandelijkse faseverschuivingen uit bijlage D — volgt in V2.
//!
//! Let op: formule (7.14) vermenigvuldigt `H_g;an` met **`θ_e;avg;an`**
//! (jaargemiddelde), niet met `θ_e;avg;mi` (maandgemiddelde). De module
//! [`super`] doet deze uitsplitsing aan de caller-zijde.

use crate::model::{BoundaryType, TransmissionElement};

/// Totale jaargemiddelde warmteoverdrachtcoëfficiënt via grond in W/K.
///
/// Deze helper bestaat om het contract tussen de zone-samenstelling (welke
/// elementen zijn `Ground`?) en de door de consumer aangeleverde `h_g_an`
/// expliciet te maken. De huidige V1-implementatie retourneert gewoon de
/// meegegeven `h_g_an` als er ten minste één [`BoundaryType::Ground`] element
/// is, anders 0.
///
/// Dit voorkomt dat consumers per ongeluk een restwaarde voor `h_g_an`
/// meegeven op zones zonder grondcontact (bovengelegen appartement, etc.).
#[must_use]
pub fn conductance_via_ground(elements: &[TransmissionElement], h_g_an: f64) -> f64 {
    let has_ground = elements
        .iter()
        .any(|el| matches!(el.boundary_type, BoundaryType::Ground));
    if has_ground {
        h_g_an
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground_el(id: &str, area: f64, u: f64) -> TransmissionElement {
        TransmissionElement {
            id: id.into(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Ground,
            construction_id: None,
        }
    }

    #[test]
    fn zero_if_no_ground_elements() {
        let els = vec![TransmissionElement {
            id: "outdoor".into(),
            area: 10.0,
            u_value: 1.0,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        }];
        assert!((conductance_via_ground(&els, 25.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn returns_supplied_value_when_ground_present() {
        let els = vec![ground_el("floor", 60.0, 0.3)];
        assert!((conductance_via_ground(&els, 18.5) - 18.5).abs() < 1e-12);
    }

    #[test]
    fn multiple_ground_elements_still_return_single_hg_an() {
        // V1: h_g_an is reeds geaggregeerd voor de zone. Consumers die dit
        // splitsen per vloer, sommeren dat zelf op vóór aanroep.
        let els = vec![
            ground_el("floor-1", 60.0, 0.3),
            ground_el("floor-2", 20.0, 0.4),
        ];
        assert!((conductance_via_ground(&els, 22.0) - 22.0).abs() < 1e-12);
    }
}
