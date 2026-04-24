//! Resultaat-struct voor de vereenvoudigde koelbehoefte-bepaling (bijlage AA,
//! pad 2).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Resultaat van [`crate::calc::calculate_simplified_cooling`].
///
/// Alle velden zijn per rekenzone. `peak_cooling_load_w` is de sommatie van
/// de individuele componenten (interne + buitenlucht + transmissie-ntr +
/// zoninstraling + glas) in de piekzomer; `minimum_capacity_w` is de
/// dezelfde piekbelasting uitgedrukt via formule (AA.11) — d.w.z.
/// gecorrigeerd met de vaste aftrek 35 W/m² en geclamped op ≥ 0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SimplifiedCoolingResult {
    /// Minimale koelcapaciteit B_C;req;TO per rekenzone in W (AA.11 × 1000).
    pub minimum_capacity_w: f64,
    /// Interne warmtelast totaal P_int;zi (AA.1), in W.
    pub internal_load_w: f64,
    /// Buitenlucht-bijdrage P_V;zi (AA.4), in W.
    pub outdoor_load_w: f64,
    /// Transmissie door ondoorzichtige delen P_tr;ntr;zi (AA.5), in W.
    pub opaque_transmission_w: f64,
    /// Zoninstraling via transparante delen P_sol;zi (AA.6), in W. Input.
    pub solar_load_w: f64,
    /// Transmissie via transparante delen P_gl;zi (AA.7), in W. Input.
    pub glazing_transmission_w: f64,
    /// Totale koellast in piekzomer P_totaal = Σ componenten, in W.
    pub peak_cooling_load_w: f64,
    /// Maatgevende koelbehoefte q_C;zi (AA.8), in W/m².
    pub maatgevende_koelbehoefte_w_per_m2: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplified_result_serde_round_trip() {
        let r = SimplifiedCoolingResult {
            minimum_capacity_w: 1800.0,
            internal_load_w: 540.0,
            outdoor_load_w: 554.0,
            opaque_transmission_w: 220.0,
            solar_load_w: 4_400.0,
            glazing_transmission_w: 286.0,
            peak_cooling_load_w: 6_000.0,
            maatgevende_koelbehoefte_w_per_m2: 50.0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: SimplifiedCoolingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
