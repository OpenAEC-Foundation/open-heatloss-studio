//! NTA 8800 bijlage H — warmtegeleidingscoëfficiënt van kozijnmaterialen.
//!
//! V1 defaults — representatieve NL-praktijkwaarden, NIET de volledige NTA 8800 tabel.
//! Volledige tabel-overname volgt in V2. Validatie tegen de norm-tekst is open punt.
//!
//! Deze module bevat default λ-waarden en U-frame-waarden voor verschillende
//! kozijnmaterialen volgens NTA 8800:2025+C1:2026 bijlage H. De waarden zijn
//! relevant voor:
//! - Warmtegeleidingsberekeningen binnen kozijnprofielen (λ)
//! - Overall kozijn-U-waarde defaults (`U_frame`)
//!
//! # Structuur
//!
//! - [`FrameMaterialKind`] — type kozijnmateriaal
//! - [`FrameMaterialDefault`] — eigenschappen van één kozijnmateriaal
//! - [`get_frame_material`] — lookup functie op materiaaltype
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_H`](crate::references::NTA_8800_2025_BIJLAGE_H).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type kozijnmateriaal voor λ en U-frame lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum FrameMaterialKind {
    /// Hout (massief of gelamineerd)
    Wood,
    /// Hout met thermische onderbreking
    WoodThermalBreak,
    /// Kunststof 3-kamer profiel
    PlasticThreeChamber,
    /// Kunststof 5-kamer profiel (beter isolerend)
    PlasticFiveChamber,
    /// Aluminium met thermische onderbreking
    AluminiumThermalBreak,
}

/// Default eigenschappen van kozijnmaterialen volgens NTA 8800 bijlage H.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FrameMaterialDefault {
    /// Type kozijnmateriaal
    pub kind: FrameMaterialKind,
    /// Warmtegeleidingscoëfficiënt λ in W/(m·K)
    pub lambda: f64,
    /// Default U-waarde van het kozijn in W/(m²·K)
    pub u_frame_default: f64,
}

/// Default kozijnmateriaal eigenschappen uit NTA 8800 bijlage H.
///
/// V1-scope: 5 representatieve types voor Nederlandse bouwpraktijk.
const FRAME_MATERIAL_DEFAULTS: &[FrameMaterialDefault] = &[
    FrameMaterialDefault {
        kind: FrameMaterialKind::Wood,
        lambda: 0.13,
        u_frame_default: 2.0,
    },
    FrameMaterialDefault {
        kind: FrameMaterialKind::WoodThermalBreak,
        lambda: 0.13,
        u_frame_default: 1.4,
    },
    FrameMaterialDefault {
        kind: FrameMaterialKind::PlasticThreeChamber,
        lambda: 0.17,
        u_frame_default: 2.2,
    },
    FrameMaterialDefault {
        kind: FrameMaterialKind::PlasticFiveChamber,
        lambda: 0.17,
        u_frame_default: 1.6,
    },
    FrameMaterialDefault {
        kind: FrameMaterialKind::AluminiumThermalBreak,
        lambda: 2.0,
        u_frame_default: 3.2,
    },
];

/// Zoek kozijnmateriaal eigenschappen op type.
///
/// # Parameters
/// - `kind`: Type kozijnmateriaal
///
/// # Returns
/// Eigenschappen van het kozijnmateriaal, of `None` als het type niet gevonden wordt.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::frame_materials::{get_frame_material, FrameMaterialKind};
///
/// let wood = get_frame_material(FrameMaterialKind::Wood)
///     .expect("Hout kozijn moet bestaan");
/// assert_eq!(wood.lambda, 0.13);
/// assert_eq!(wood.u_frame_default, 2.0);
/// ```
#[must_use]
pub fn get_frame_material(kind: FrameMaterialKind) -> Option<&'static FrameMaterialDefault> {
    FRAME_MATERIAL_DEFAULTS.iter().find(|f| f.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_frame_material_types_have_data() {
        assert!(get_frame_material(FrameMaterialKind::Wood).is_some());
        assert!(get_frame_material(FrameMaterialKind::WoodThermalBreak).is_some());
        assert!(get_frame_material(FrameMaterialKind::PlasticThreeChamber).is_some());
        assert!(get_frame_material(FrameMaterialKind::PlasticFiveChamber).is_some());
        assert!(get_frame_material(FrameMaterialKind::AluminiumThermalBreak).is_some());
    }

    #[test]
    fn lambda_values_are_plausible() {
        for default in FRAME_MATERIAL_DEFAULTS {
            assert!(
                default.lambda > 0.0 && default.lambda < 10.0,
                "λ {} voor {:?} valt buiten plausibel bereik 0-10 W/(m·K)",
                default.lambda,
                default.kind
            );
        }
    }

    #[test]
    fn u_frame_values_are_plausible() {
        for default in FRAME_MATERIAL_DEFAULTS {
            assert!(
                default.u_frame_default > 0.0 && default.u_frame_default < 10.0,
                "U-frame {} voor {:?} valt buiten plausibel bereik 0-10 W/(m²·K)",
                default.u_frame_default,
                default.kind
            );
        }
    }

    #[test]
    fn thermal_break_improves_performance() {
        let wood = get_frame_material(FrameMaterialKind::Wood).unwrap();
        let wood_tb = get_frame_material(FrameMaterialKind::WoodThermalBreak).unwrap();
        let plastic3 = get_frame_material(FrameMaterialKind::PlasticThreeChamber).unwrap();
        let plastic5 = get_frame_material(FrameMaterialKind::PlasticFiveChamber).unwrap();

        assert!(
            wood.u_frame_default > wood_tb.u_frame_default,
            "Thermische onderbreking moet hout verbeteren"
        );
        assert!(
            plastic3.u_frame_default > plastic5.u_frame_default,
            "5-kamer profiel moet beter zijn dan 3-kamer"
        );
    }
}
