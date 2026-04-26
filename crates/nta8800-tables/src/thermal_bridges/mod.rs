//! NTA 8800 bijlage I — forfaitaire ψ-waarden voor lineaire koudebruggen.
//!
//! V1 defaults — representatieve NL-praktijkwaarden, NIET de volledige NTA 8800 tabel.
//! Volledige tabel-overname volgt in V2. Validatie tegen de norm-tekst is open punt.
//!
//! Deze module bevat forfaitaire ψ-waarden voor lineaire koudebruggen volgens
//! NTA 8800:2025+C1:2026 bijlage I. Koudebruggen veroorzaken extra warmteverlies
//! op aansluitingen tussen bouwdelen. De ψ-waarde in W/(m·K) geeft het lineaire
//! extra warmteverlies per meter aansluiting.
//!
//! # Structuur
//!
//! - [`ThermalBridgeKind`] — type koudebrug-aansluiting
//! - [`ThermalBridgeDefault`] — eigenschappen van één koudebrug-type
//! - [`get_thermal_bridge`] — lookup functie op koudebrug-type
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_I`](crate::references::NTA_8800_2025_BIJLAGE_I).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type koudebrug voor ψ-waarde lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ThermalBridgeKind {
    /// Fundering-buitenwand aansluiting
    FoundationExteriorWall,
    /// Hoek tussen twee buitenwanden
    ExteriorWallCorner,
    /// Dak-buitenwand aansluiting
    RoofExteriorWall,
    /// Vloer-tussenwand aansluiting
    FloorInteriorWall,
    /// Kozijn-aansluiting bovenzijde
    WindowTop,
    /// Kozijn-aansluiting zijkant (links/rechts)
    WindowSide,
    /// Kozijn-aansluiting onderzijde
    WindowBottom,
    /// Kozijn-aansluiting drempel
    WindowSill,
}

/// Default eigenschappen van koudebruggen volgens NTA 8800 bijlage I.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct ThermalBridgeDefault {
    /// Type koudebrug
    pub kind: ThermalBridgeKind,
    /// Lineaire warmtedoorgangscoëfficiënt ψ in W/(m·K)
    pub psi: f64,
}

/// Default koudebrug eigenschappen uit NTA 8800 bijlage I.
///
/// V1-scope: 8 representatieve detail-types voor Nederlandse bouwpraktijk.
const THERMAL_BRIDGE_DEFAULTS: &[ThermalBridgeDefault] = &[
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::FoundationExteriorWall,
        psi: 0.15,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::ExteriorWallCorner,
        psi: 0.05,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::RoofExteriorWall,
        psi: 0.10,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::FloorInteriorWall,
        psi: 0.08,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::WindowTop,
        psi: 0.12,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::WindowSide,
        psi: 0.08,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::WindowBottom,
        psi: 0.10,
    },
    ThermalBridgeDefault {
        kind: ThermalBridgeKind::WindowSill,
        psi: 0.15,
    },
];

/// Zoek koudebrug eigenschappen op type.
///
/// # Parameters
/// - `kind`: Type koudebrug
///
/// # Returns
/// Eigenschappen van de koudebrug, of `None` als het type niet gevonden wordt.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::thermal_bridges::{get_thermal_bridge, ThermalBridgeKind};
///
/// let corner = get_thermal_bridge(ThermalBridgeKind::ExteriorWallCorner)
///     .expect("Buitenwandhoek moet bestaan");
/// assert_eq!(corner.psi, 0.05);
/// ```
#[must_use]
pub fn get_thermal_bridge(kind: ThermalBridgeKind) -> Option<&'static ThermalBridgeDefault> {
    THERMAL_BRIDGE_DEFAULTS.iter().find(|t| t.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_thermal_bridge_types_have_data() {
        assert!(get_thermal_bridge(ThermalBridgeKind::FoundationExteriorWall).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::ExteriorWallCorner).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::RoofExteriorWall).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::FloorInteriorWall).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::WindowTop).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::WindowSide).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::WindowBottom).is_some());
        assert!(get_thermal_bridge(ThermalBridgeKind::WindowSill).is_some());
    }

    #[test]
    fn psi_values_are_plausible() {
        for default in THERMAL_BRIDGE_DEFAULTS {
            assert!(
                default.psi >= 0.0 && default.psi < 1.0,
                "ψ-waarde {} voor {:?} valt buiten plausibel bereik 0-1 W/(m·K)",
                default.psi,
                default.kind
            );
        }
    }

    #[test]
    fn foundation_and_sill_have_higher_psi() {
        // Fundering en drempel zijn typisch kritieke koudebruggen
        let foundation = get_thermal_bridge(ThermalBridgeKind::FoundationExteriorWall).unwrap();
        let sill = get_thermal_bridge(ThermalBridgeKind::WindowSill).unwrap();
        let corner = get_thermal_bridge(ThermalBridgeKind::ExteriorWallCorner).unwrap();

        assert!(
            foundation.psi > corner.psi,
            "Fundering-aansluiting moet hogere ψ hebben dan hoek"
        );
        assert!(
            sill.psi > corner.psi,
            "Drempel moet hogere ψ hebben dan hoek"
        );
    }
}
