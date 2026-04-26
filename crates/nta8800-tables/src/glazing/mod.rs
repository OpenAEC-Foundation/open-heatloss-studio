//! NTA 8800 bijlage G — default U-waarde en g-waarde voor beglazing.
//!
//! V1 defaults — representatieve NL-praktijkwaarden, NIET de volledige NTA 8800 tabel.
//! Volledige tabel-overname volgt in V2. Validatie tegen de norm-tekst is open punt.
//!
//! Deze module bevat default U-waarden en g-waarden voor verschillende beglazingstypen
//! volgens NTA 8800:2025+C1:2026 bijlage G. De waarden zijn relevant voor:
//! - Warmteverliesberekeningen (U-waarde in W/(m²·K))
//! - Zonnewinst-berekeningen (g-waarde, dimensieloos 0-1)
//!
//! # Structuur
//!
//! - [`GlazingKind`] — type beglazing (enkel, dubbel, triple met coatings)
//! - [`GlazingDefault`] — eigenschappen van één beglazingstype
//! - [`get_glazing`] — lookup functie op beglazingstype
//!
//! Referentie: [`NTA_8800_2025_BIJLAGE_G`](crate::references::NTA_8800_2025_BIJLAGE_G).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type beglazing voor U-waarde en g-waarde lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum GlazingKind {
    /// Enkel glas (legacy)
    Single,
    /// Dubbel glas standaard (zonder coating)
    DoubleStandard,
    /// Dubbel HR-glas (low-E coating)
    DoubleHR,
    /// Dubbel HR+ glas (verbeterde low-E coating)
    DoubleHRPlus,
    /// Triple HR+++ glas (twee low-E coatings)
    TripleHRPlusPlus,
    /// Triple isolerend glas (maximale isolatie)
    TripleInsulating,
}

/// Default eigenschappen van beglazing volgens NTA 8800 bijlage G.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct GlazingDefault {
    /// Type beglazing
    pub kind: GlazingKind,
    /// U-waarde beglazing in W/(m²·K)
    pub u_glazing: f64,
    /// g-waarde (zonnetoetreding), dimensieloos 0-1
    pub g_value: f64,
}

/// Default beglazing eigenschappen uit NTA 8800 bijlage G.
///
/// V1-scope: 6 representatieve types voor Nederlandse bouwpraktijk.
const GLAZING_DEFAULTS: &[GlazingDefault] = &[
    GlazingDefault {
        kind: GlazingKind::Single,
        u_glazing: 5.8,
        g_value: 0.85,
    },
    GlazingDefault {
        kind: GlazingKind::DoubleStandard,
        u_glazing: 2.8,
        g_value: 0.75,
    },
    GlazingDefault {
        kind: GlazingKind::DoubleHR,
        u_glazing: 1.6,
        g_value: 0.65,
    },
    GlazingDefault {
        kind: GlazingKind::DoubleHRPlus,
        u_glazing: 1.2,
        g_value: 0.60,
    },
    GlazingDefault {
        kind: GlazingKind::TripleHRPlusPlus,
        u_glazing: 0.8,
        g_value: 0.55,
    },
    GlazingDefault {
        kind: GlazingKind::TripleInsulating,
        u_glazing: 0.5,
        g_value: 0.50,
    },
];

/// Zoek beglazing eigenschappen op type.
///
/// # Parameters
/// - `kind`: Type beglazing
///
/// # Returns
/// Eigenschappen van de beglazing, of `None` als het type niet gevonden wordt.
///
/// # Voorbeeld
/// ```
/// use nta8800_tables::glazing::{get_glazing, GlazingKind};
///
/// let glazing = get_glazing(GlazingKind::DoubleHR)
///     .expect("HR-glas moet bestaan");
/// assert_eq!(glazing.u_glazing, 1.6);
/// assert_eq!(glazing.g_value, 0.65);
/// ```
#[must_use]
pub fn get_glazing(kind: GlazingKind) -> Option<&'static GlazingDefault> {
    GLAZING_DEFAULTS.iter().find(|g| g.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_glazing_types_have_data() {
        assert!(get_glazing(GlazingKind::Single).is_some());
        assert!(get_glazing(GlazingKind::DoubleStandard).is_some());
        assert!(get_glazing(GlazingKind::DoubleHR).is_some());
        assert!(get_glazing(GlazingKind::DoubleHRPlus).is_some());
        assert!(get_glazing(GlazingKind::TripleHRPlusPlus).is_some());
        assert!(get_glazing(GlazingKind::TripleInsulating).is_some());
    }

    #[test]
    fn u_values_are_plausible() {
        for default in GLAZING_DEFAULTS {
            assert!(
                default.u_glazing > 0.0 && default.u_glazing < 10.0,
                "U-waarde {} voor {:?} valt buiten plausibel bereik 0-10 W/(m²·K)",
                default.u_glazing,
                default.kind
            );
        }
    }

    #[test]
    fn g_values_are_valid_fractions() {
        for default in GLAZING_DEFAULTS {
            assert!(
                default.g_value >= 0.0 && default.g_value <= 1.0,
                "g-waarde {} voor {:?} moet tussen 0-1 zijn",
                default.g_value,
                default.kind
            );
        }
    }

    #[test]
    fn better_glazing_has_lower_u_value() {
        let single = get_glazing(GlazingKind::Single).unwrap();
        let double_std = get_glazing(GlazingKind::DoubleStandard).unwrap();
        let triple = get_glazing(GlazingKind::TripleInsulating).unwrap();

        assert!(
            single.u_glazing > double_std.u_glazing,
            "Dubbel glas moet beter isoleren dan enkel"
        );
        assert!(
            double_std.u_glazing > triple.u_glazing,
            "Triple glas moet beter isoleren dan dubbel"
        );
    }
}
