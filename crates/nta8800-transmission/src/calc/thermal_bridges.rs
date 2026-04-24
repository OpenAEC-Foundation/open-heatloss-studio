//! Koudebruggen — `Σ(L_k · ψ_k) + Σχ_j` uit formule (8.1).
//!
//! §8.2.3 (lineair, ψ) en §8.2.4 (punt, χ) zijn in [`nta8800_model::geometry`]
//! al gedefinieerd als [`ThermalBridgeLinear`] en [`ThermalBridgePoint`]. Deze
//! module implementeert alleen de sommatie die nodig is in formule (8.1).
//!
//! NTA 8800 §8.2.1 OPMERKING 4: incidentele puntvormige bruggen (spouwankers)
//! worden expliciet **niet** meegerekend — consumers filteren die zelf uit de
//! input. Deze module vertrouwt op de meegeleverde lijst.

use nta8800_model::{ThermalBridgeLinear, ThermalBridgePoint};

/// Som van lineaire koudebrug-bijdragen `Σ(ψ · L)` in W/K.
#[must_use]
pub fn linear_bridge_conductance(bridges: &[ThermalBridgeLinear]) -> f64 {
    bridges.iter().map(|tb| tb.psi * tb.length).sum()
}

/// Som van puntvormige koudebrug-bijdragen `Σχ` in W/K.
#[must_use]
pub fn point_bridge_conductance(bridges: &[ThermalBridgePoint]) -> f64 {
    bridges.iter().map(|tb| tb.chi).sum()
}

/// Bereken beide sommaties in één call; retourneert `(Σ(ψ·L), Σχ)` in W/K.
#[must_use]
pub fn bridge_conductances(
    linear: &[ThermalBridgeLinear],
    point: &[ThermalBridgePoint],
) -> (f64, f64) {
    (
        linear_bridge_conductance(linear),
        point_bridge_conductance(point),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::geometry::ThermalBridgeCategory;

    fn linear(id: &str, length: f64, psi: f64) -> ThermalBridgeLinear {
        ThermalBridgeLinear {
            id: id.into(),
            length,
            psi,
            category: ThermalBridgeCategory::Overig,
        }
    }

    fn point(id: &str, chi: f64) -> ThermalBridgePoint {
        ThermalBridgePoint {
            id: id.into(),
            chi,
            category: ThermalBridgeCategory::Overig,
        }
    }

    #[test]
    fn empty_linear_is_zero() {
        assert!((linear_bridge_conductance(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_point_is_zero() {
        assert!((point_bridge_conductance(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn linear_sum_of_psi_times_length() {
        let bridges = vec![linear("b1", 10.0, 0.1), linear("b2", 5.0, 0.2)];
        // 10·0.1 + 5·0.2 = 2.0 W/K
        assert!((linear_bridge_conductance(&bridges) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn point_sum_of_chi() {
        let bridges = vec![point("p1", 0.02), point("p2", 0.05), point("p3", 0.01)];
        assert!((point_bridge_conductance(&bridges) - 0.08).abs() < 1e-12);
    }

    #[test]
    fn combined_returns_tuple() {
        let lin = vec![linear("l", 10.0, 0.15)];
        let pts = vec![point("p", 0.02)];
        let (l, p) = bridge_conductances(&lin, &pts);
        assert!((l - 1.5).abs() < 1e-12);
        assert!((p - 0.02).abs() < 1e-12);
    }
}
