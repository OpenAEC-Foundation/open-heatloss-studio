//! H_D,element — directe warmteverliescoëfficiënt element-deel naar buitenlucht.
//!
//! Implementeert het `Σ(A_T;i · U_C;i)` deel van formule (8.1) in
//! [`crate::references::NTA_8800_2025_FORMULE8_1`]. De lineaire en puntvormige
//! bruggen (`Σ(L_k · ψ_k) + Σχ_j`) leven in [`super::thermal_bridges`].
//!
//! Enkel elementen met [`BoundaryType::Outdoor`] tellen mee; andere boundaries
//! gaan via `H_U`, `H_g` of `H_A`.

use crate::model::{BoundaryType, TransmissionElement};

/// Som van `A · U` over alle [`BoundaryType::Outdoor`]-elementen (W/K).
///
/// Vormt samen met [`super::thermal_bridges::bridge_conductances`] de totale
/// `H_D` volgens formule (8.1).
#[must_use]
pub fn conductance_outdoor_elements(elements: &[TransmissionElement]) -> f64 {
    elements
        .iter()
        .filter(|el| matches!(el.boundary_type, BoundaryType::Outdoor))
        .map(TransmissionElement::conductance)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outdoor_el(id: &str, area: f64, u: f64) -> TransmissionElement {
        TransmissionElement {
            id: id.into(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        }
    }

    #[test]
    fn empty_slice_yields_zero() {
        assert!((conductance_outdoor_elements(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_outdoor_element_sums_a_times_u() {
        let els = vec![outdoor_el("w", 10.0, 0.2)];
        assert!((conductance_outdoor_elements(&els) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn multiple_outdoor_elements_sum() {
        let els = vec![
            outdoor_el("w1", 10.0, 0.2),
            outdoor_el("w2", 5.0, 1.0),
            outdoor_el("w3", 2.0, 1.1),
        ];
        // 2.0 + 5.0 + 2.2 = 9.2
        assert!((conductance_outdoor_elements(&els) - 9.2).abs() < 1e-12);
    }

    #[test]
    fn non_outdoor_elements_are_ignored() {
        let els = vec![
            outdoor_el("w", 10.0, 0.2),
            TransmissionElement {
                id: "floor".into(),
                area: 60.0,
                u_value: 0.3,
                boundary_type: BoundaryType::Ground,
                construction_id: None,
            },
            TransmissionElement {
                id: "to-garage".into(),
                area: 12.0,
                u_value: 0.5,
                boundary_type: BoundaryType::UnheatedSpace {
                    id: "garage".into(),
                },
                construction_id: None,
            },
        ];
        // alleen 'w' (10·0.2 = 2.0) telt mee
        assert!((conductance_outdoor_elements(&els) - 2.0).abs() < 1e-12);
    }
}
