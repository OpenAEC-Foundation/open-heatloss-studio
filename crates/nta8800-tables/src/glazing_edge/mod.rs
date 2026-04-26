//! NTA 8800 bijlage L — ψ-waarden voor beglazingsrand (glazing-to-frame edge).
//!
//! V1 defaults — representatieve NL-praktijkwaarden, NIET de volledige NTA 8800 tabel.
//! Volledige tabel-overname volgt in V2. Validatie tegen de norm-tekst is open punt.
//!
//! Deze module bevat ψ-waarden voor de beglazingsrand volgens NTA 8800:2025+C1:2026
//! bijlage L. De beglazingsrand is de aansluiting tussen het glas en het kozijn,
//! waar extra warmteverlies optreedt door de randafstandhouder (spacer). De
//! ψ-waarde in W/(m·K) geeft het lineaire extra warmteverlies per meter glasrand.
//!
//! # Structuur
//!
//! - [`SpacerKind`] — type randafstandhouder
//! - [`GlazingEdgeDefault`] — eigenschappen van één spacer-type
//! - [`get_glazing_edge`] — lookup functie op spacer-type
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_L`](crate::references::NTA_8800_2025_BIJLAGE_L).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type randafstandhouder voor ψ-waarde lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum SpacerKind {
    /// Aluminium spacer (standaard, koudebruggend)
    Aluminium,
    /// RVS spacer (verbeterde prestatie)
    Stainless,
    /// Warm-edge polymeer spacer
    WarmEdgePolymer,
    /// Warm-edge schuim spacer (beste prestatie)
    WarmEdgeFoam,
}

/// Default eigenschappen van beglazingsrand volgens NTA 8800 bijlage L.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct GlazingEdgeDefault {
    /// Type randafstandhouder
    pub kind: SpacerKind,
    /// Lineaire warmtedoorgangscoëfficiënt ψ in W/(m·K)
    pub psi_edge: f64,
}

/// Default beglazingsrand eigenschappen uit NTA 8800 bijlage L.
///
/// V1-scope: 4 representatieve spacer-types voor Nederlandse bouwpraktijk.
const GLAZING_EDGE_DEFAULTS: &[GlazingEdgeDefault] = &[
    GlazingEdgeDefault {
        kind: SpacerKind::Aluminium,
        psi_edge: 0.08,
    },
    GlazingEdgeDefault {
        kind: SpacerKind::Stainless,
        psi_edge: 0.06,
    },
    GlazingEdgeDefault {
        kind: SpacerKind::WarmEdgePolymer,
        psi_edge: 0.04,
    },
    GlazingEdgeDefault {
        kind: SpacerKind::WarmEdgeFoam,
        psi_edge: 0.02,
    },
];

/// Zoek beglazingsrand eigenschappen op spacer-type.
///
/// # Parameters
/// - `kind`: Type randafstandhouder
///
/// # Returns
/// Eigenschappen van de beglazingsrand, of `None` als het type niet gevonden wordt.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::glazing_edge::{get_glazing_edge, SpacerKind};
///
/// let warm_edge = get_glazing_edge(SpacerKind::WarmEdgePolymer)
///     .expect("Warm-edge spacer moet bestaan");
/// assert_eq!(warm_edge.psi_edge, 0.04);
/// ```
#[must_use]
pub fn get_glazing_edge(kind: SpacerKind) -> Option<&'static GlazingEdgeDefault> {
    GLAZING_EDGE_DEFAULTS.iter().find(|g| g.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_spacer_types_have_data() {
        assert!(get_glazing_edge(SpacerKind::Aluminium).is_some());
        assert!(get_glazing_edge(SpacerKind::Stainless).is_some());
        assert!(get_glazing_edge(SpacerKind::WarmEdgePolymer).is_some());
        assert!(get_glazing_edge(SpacerKind::WarmEdgeFoam).is_some());
    }

    #[test]
    fn psi_edge_values_are_plausible() {
        for default in GLAZING_EDGE_DEFAULTS {
            assert!(
                default.psi_edge >= 0.0 && default.psi_edge < 0.2,
                "ψ-edge {} voor {:?} valt buiten plausibel bereik 0-0.2 W/(m·K)",
                default.psi_edge,
                default.kind
            );
        }
    }

    #[test]
    fn warm_edge_performs_better_than_metal() {
        let aluminium = get_glazing_edge(SpacerKind::Aluminium).unwrap();
        let stainless = get_glazing_edge(SpacerKind::Stainless).unwrap();
        let warm_polymer = get_glazing_edge(SpacerKind::WarmEdgePolymer).unwrap();
        let warm_foam = get_glazing_edge(SpacerKind::WarmEdgeFoam).unwrap();

        assert!(
            aluminium.psi_edge > stainless.psi_edge,
            "RVS moet beter zijn dan aluminium"
        );
        assert!(
            stainless.psi_edge > warm_polymer.psi_edge,
            "Warm-edge polymeer moet beter zijn dan RVS"
        );
        assert!(
            warm_polymer.psi_edge > warm_foam.psi_edge,
            "Warm-edge schuim moet beste zijn"
        );
    }
}
