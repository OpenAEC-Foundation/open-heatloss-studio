//! H_U — warmteverlies via onverwarmde ruimte (§8.4).
//!
//! Implementeert formule (8.52): `H_U = H_D;zi,j;ztu · b_U` — waarbij
//! `H_D;zi,j;ztu = Σ(A · U)` over alle elementen die naar de onverwarmde ruimte
//! grenzen, en `b_U` de dimensieloze reductiefactor is die de consumer per
//! onverwarmde ruimte aanlevert (zie [`crate::references::NTA_8800_2025_FORMULE8_53`]).
//!
//! V1 accepteert b-factors als direct-aangeleverde invoer (conform bijlage I.2.4
//! pad voor basisopname ISSO 82.1/75.1). Het afleiden van `b_U` uit
//! `H_ue / (H_zi,j;ztu + H_ue)` met ventilatie-bijdragen uit §11 volgt later
//! in een dedicated `nta8800-ventilation`-integratie.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::errors::{TransmissionError, TransmissionResult};
use crate::model::{BoundaryType, TransmissionElement};

/// Bereken `H_U = Σ_per_space (Σ_el(A·U) · b_U)` over alle onverwarmde-ruimte
/// elementen.
///
/// Itereert over elementen met [`BoundaryType::UnheatedSpace`], groepeert per
/// `id`, vermenigvuldigt de som-A·U met de bijbehorende b-factor uit de
/// lookup-map.
///
/// # Errors
///
/// - [`TransmissionError::MissingUnheatedBFactor`] als een referentie-id in de
///   elementen niet in de map voorkomt.
/// - [`TransmissionError::BFactorOutOfRange`] als een b-factor buiten
///   `0..=1` valt of niet-eindig is.
pub fn conductance_via_unheated<S: BuildHasher>(
    elements: &[TransmissionElement],
    b_factors: &HashMap<String, f64, S>,
) -> TransmissionResult<f64> {
    let mut h_u = 0.0_f64;
    for el in elements {
        if let BoundaryType::UnheatedSpace { id } = &el.boundary_type {
            let b = b_factors
                .get(id)
                .copied()
                .ok_or_else(|| TransmissionError::MissingUnheatedBFactor { id: id.clone() })?;
            if !b.is_finite() || !(0.0..=1.0).contains(&b) {
                return Err(TransmissionError::BFactorOutOfRange {
                    id: id.clone(),
                    value: b,
                });
            }
            h_u += el.conductance() * b;
        }
    }
    Ok(h_u)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unheated_el(id: &str, area: f64, u: f64, space_id: &str) -> TransmissionElement {
        TransmissionElement {
            id: id.into(),
            area,
            u_value: u,
            boundary_type: BoundaryType::UnheatedSpace {
                id: space_id.into(),
            },
            construction_id: None,
        }
    }

    #[test]
    fn empty_slice_yields_zero() {
        let b = HashMap::new();
        assert!((conductance_via_unheated(&[], &b).unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn half_b_factor_halves_conductance() {
        let els = vec![unheated_el("wall", 10.0, 0.5, "garage")];
        let mut b = HashMap::new();
        b.insert("garage".into(), 0.5);
        // 10 · 0.5 · 0.5 = 2.5 W/K
        assert!((conductance_via_unheated(&els, &b).unwrap() - 2.5).abs() < 1e-12);
    }

    #[test]
    fn multiple_unheated_spaces_aggregate() {
        let els = vec![
            unheated_el("wall-garage", 8.0, 0.5, "garage"),
            unheated_el("wall-attic", 20.0, 0.3, "zolder"),
        ];
        let mut b = HashMap::new();
        b.insert("garage".into(), 0.6);
        b.insert("zolder".into(), 0.8);
        // 8·0.5·0.6 = 2.4  en  20·0.3·0.8 = 4.8  → 7.2 W/K
        assert!((conductance_via_unheated(&els, &b).unwrap() - 7.2).abs() < 1e-12);
    }

    #[test]
    fn missing_b_factor_is_error() {
        let els = vec![unheated_el("wall", 8.0, 0.5, "garage")];
        let b = HashMap::new();
        let err = conductance_via_unheated(&els, &b).unwrap_err();
        assert!(matches!(
            err,
            TransmissionError::MissingUnheatedBFactor { ref id } if id == "garage"
        ));
    }

    #[test]
    fn out_of_range_b_factor_is_error() {
        let els = vec![unheated_el("wall", 8.0, 0.5, "garage")];
        let mut b = HashMap::new();
        b.insert("garage".into(), 1.5);
        let err = conductance_via_unheated(&els, &b).unwrap_err();
        assert!(matches!(err, TransmissionError::BFactorOutOfRange { .. }));
    }

    #[test]
    fn outdoor_elements_are_ignored() {
        let els = vec![TransmissionElement {
            id: "outdoor".into(),
            area: 10.0,
            u_value: 1.0,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        }];
        let b = HashMap::new();
        assert!((conductance_via_unheated(&els, &b).unwrap() - 0.0).abs() < f64::EPSILON);
    }
}
