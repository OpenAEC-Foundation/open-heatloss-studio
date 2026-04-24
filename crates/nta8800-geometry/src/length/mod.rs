//! Lengte-bepaling voor lijnvormige elementen — NTA 8800 bijlage K.
//!
//! Bijlage K beschrijft de bepaling van de lengte `ℓ_k` van lineaire
//! warmteverliezen (koudebruggen, aansluitingen). Deze module levert
//! hulpfuncties voor:
//!
//! - Totaalsom van [`ThermalBridgeLinear`]-lengten (voor `Σ ℓ · ψ`
//!   sommatie in H.8 transmissie-berekening).
//! - Perimeter van een rechthoekige plattegrond — nodig voor
//!   vloer-op-grond transmissie-correctie (§8.3.2.2).

use nta8800_model::error::{ModelError, ModelResult};
use nta8800_model::geometry::ThermalBridgeLinear;
use nta8800_model::units::Length;

/// Bereken de totale lengte over een lijst lineaire koudebruggen.
///
/// Σ `ℓ_k` voor alle opgegeven bruggen. Retourneert 0.0 bij een lege lijst.
///
/// # Errors
/// [`ModelError::InvalidInput`] als één van de bruggen een niet-eindige of
/// negatieve `length` heeft.
pub fn thermal_bridge_total_length(bridges: &[&ThermalBridgeLinear]) -> ModelResult<Length> {
    let mut total = 0.0_f64;
    for b in bridges {
        if !b.length.is_finite() || b.length < 0.0 {
            return Err(ModelError::InvalidInput {
                context: format!("ThermalBridgeLinear {}.length", b.id),
                reason: format!("moet eindig en ≥ 0 zijn, gekregen {}", b.length),
            });
        }
        total += b.length;
    }
    Ok(total)
}

/// Bereken de perimeter van een rechthoekige plattegrond.
///
/// Gebruikt bij §8.3.2.2 vloer-op-grond: de perimeter `P` is nodig voor de
/// forfaitaire toeslag op de vloer-U-waarde.
///
/// `P = 2 · (width + depth)`
///
/// # Errors
/// [`ModelError::InvalidInput`] als een van de parameters niet-eindig of
/// ≤ 0 is.
pub fn perimeter_rectangle(width: Length, depth: Length) -> ModelResult<Length> {
    if !width.is_finite() || width <= 0.0 {
        return Err(ModelError::InvalidInput {
            context: "perimeter_rectangle.width".into(),
            reason: format!("moet > 0 en eindig zijn, gekregen {width}"),
        });
    }
    if !depth.is_finite() || depth <= 0.0 {
        return Err(ModelError::InvalidInput {
            context: "perimeter_rectangle.depth".into(),
            reason: format!("moet > 0 en eindig zijn, gekregen {depth}"),
        });
    }
    Ok(2.0 * (width + depth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::geometry::ThermalBridgeCategory;

    fn mk_bridge(id: &str, length: Length) -> ThermalBridgeLinear {
        ThermalBridgeLinear {
            id: id.into(),
            length,
            psi: 0.1,
            category: ThermalBridgeCategory::AansluitingVloerGevel,
        }
    }

    #[test]
    fn total_length_sommeert_alle_bruggen() {
        let b1 = mk_bridge("b1", 10.0);
        let b2 = mk_bridge("b2", 5.5);
        let b3 = mk_bridge("b3", 2.25);
        let total = thermal_bridge_total_length(&[&b1, &b2, &b3]).unwrap();
        assert!((total - 17.75).abs() < 1e-9);
    }

    #[test]
    fn total_length_leeg_is_nul() {
        let total = thermal_bridge_total_length(&[]).unwrap();
        assert!(total.abs() < 1e-12);
    }

    #[test]
    fn total_length_weigert_negatieve_brug() {
        let b = mk_bridge("bad", -1.0);
        let err = thermal_bridge_total_length(&[&b]).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { .. }));
    }

    #[test]
    fn perimeter_3x4_is_14() {
        let p = perimeter_rectangle(3.0, 4.0).unwrap();
        assert!((p - 14.0).abs() < 1e-9);
    }

    #[test]
    fn perimeter_vierkant_10x10_is_40() {
        let p = perimeter_rectangle(10.0, 10.0).unwrap();
        assert!((p - 40.0).abs() < 1e-9);
    }

    #[test]
    fn perimeter_weigert_nul_breedte() {
        assert!(perimeter_rectangle(0.0, 4.0).is_err());
        assert!(perimeter_rectangle(3.0, -1.0).is_err());
        assert!(perimeter_rectangle(f64::INFINITY, 4.0).is_err());
    }
}
