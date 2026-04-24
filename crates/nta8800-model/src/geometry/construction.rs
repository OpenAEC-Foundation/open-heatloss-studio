//! Constructie-opbouw: materiaallagen + oppervlakteweerstanden.
//!
//! NTA 8800 §8.3 — de warmteweerstand van een constructie wordt opgebouwd
//! uit de som van lagen plus de binnen- en buitenoppervlakteweerstanden
//! (`R_si` en `R_se`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::units::{Length, ThermalResistance, ThermalTransmittance};

/// Eén materiaallaag in een gelaagde constructie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConstructionLayer {
    /// Materiaalomschrijving (bv. `"PIR-isolatie"`, `"kalkzandsteen"`).
    pub material_name: String,

    /// Dikte van de laag in m.
    pub thickness: Length,

    /// Warmtegeleidingscoëfficiënt λ in W/(m·K).
    pub lambda: f64,
}

impl ConstructionLayer {
    /// Thermische weerstand R van deze laag in m²·K/W, berekend als `d / λ`.
    #[must_use]
    pub fn thermal_resistance(&self) -> ThermalResistance {
        self.thickness / self.lambda
    }
}

/// Gelaagde constructie met expliciete oppervlakteweerstanden.
///
/// In tegenstelling tot ISSO 51 modelleren we hier géén `Default`-implementatie
/// met forfaitaire `R_si` / `R_se`-waarden — de normconforme waarden (bijv. 0.13
/// voor binnen en 0.04 voor buiten bij verticale vlakken) moeten **expliciet**
/// worden gezet per constructie om te voorkomen dat een vergeten override
/// stilzwijgend de standaardwaarde gebruikt waar een afwijkende situatie
/// geldt (kruipruimte, dakrand, enzovoort).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Construction {
    /// Unieke identificatie binnen het project.
    pub id: String,

    /// Menselijk leesbare naam (bv. `"Spouwmuur Rc=4.5"`).
    pub name: String,

    /// Materiaallagen, van binnen naar buiten.
    pub layers: Vec<ConstructionLayer>,

    /// Binnenoppervlakteweerstand `R_si` in m²·K/W.
    pub r_si: ThermalResistance,

    /// Buitenoppervlakteweerstand `R_se` in m²·K/W.
    pub r_se: ThermalResistance,
}

impl Construction {
    /// Totale warmteweerstand `R_tot` = `R_si` + Σ `R_laag` + `R_se` in m²·K/W.
    #[must_use]
    pub fn r_total(&self) -> ThermalResistance {
        let layer_sum: ThermalResistance = self
            .layers
            .iter()
            .map(ConstructionLayer::thermal_resistance)
            .sum();
        self.r_si + layer_sum + self.r_se
    }

    /// Warmtedoorgangscoëfficiënt U = 1 / `R_tot` in W/(m²·K).
    ///
    /// Geeft `f64::INFINITY` als `R_tot` nul is — de rekencrate moet dat
    /// geval detecteren vóór gebruik.
    #[must_use]
    pub fn u_value(&self) -> ThermalTransmittance {
        1.0 / self.r_total()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_layers() -> Vec<ConstructionLayer> {
        vec![
            ConstructionLayer {
                material_name: "gipskarton".into(),
                thickness: 0.012,
                lambda: 0.25,
            },
            ConstructionLayer {
                material_name: "minerale wol".into(),
                thickness: 0.150,
                lambda: 0.035,
            },
            ConstructionLayer {
                material_name: "kalkzandsteen".into(),
                thickness: 0.100,
                lambda: 1.00,
            },
        ]
    }

    #[test]
    fn layer_thermal_resistance_matches_manual() {
        let layer = ConstructionLayer {
            material_name: "isolatie".into(),
            thickness: 0.100,
            lambda: 0.035,
        };
        let r = layer.thermal_resistance();
        // 0.100 / 0.035 ≈ 2.857
        assert!((r - 2.857_142_857).abs() < 1e-6);
    }

    #[test]
    fn construction_r_total_sums_layers_and_surface_resistances() {
        let construction = Construction {
            id: "c1".into(),
            name: "test-wand".into(),
            layers: sample_layers(),
            r_si: 0.13,
            r_se: 0.04,
        };
        // Σ R_laag = 0.012/0.25 + 0.150/0.035 + 0.100/1.00
        //          = 0.048 + 4.2857143 + 0.100
        //          ≈ 4.4337
        // R_tot = 0.13 + 4.4337 + 0.04 ≈ 4.6037
        let expected = 0.13 + 0.048 + (0.150 / 0.035) + 0.100 + 0.04;
        assert!(
            (construction.r_total() - expected).abs() < 1e-6,
            "R_tot mismatch: got {}, expected {}",
            construction.r_total(),
            expected
        );
    }

    #[test]
    fn construction_u_value_is_reciprocal_of_r_total() {
        let construction = Construction {
            id: "c2".into(),
            name: "simpele wand".into(),
            layers: vec![ConstructionLayer {
                material_name: "baksteen".into(),
                thickness: 0.100,
                lambda: 1.0,
            }],
            r_si: 0.13,
            r_se: 0.04,
        };
        // R_tot = 0.13 + 0.1 + 0.04 = 0.27 ⇒ U ≈ 3.7037
        let expected_u = 1.0 / 0.27;
        assert!((construction.u_value() - expected_u).abs() < 1e-6);
    }

    #[test]
    fn construction_serde_round_trip() {
        let construction = Construction {
            id: "c1".into(),
            name: "test-wand".into(),
            layers: sample_layers(),
            r_si: 0.13,
            r_se: 0.04,
        };
        let json = serde_json::to_string(&construction).unwrap();
        let back: Construction = serde_json::from_str(&json).unwrap();
        assert_eq!(construction, back);
    }
}
