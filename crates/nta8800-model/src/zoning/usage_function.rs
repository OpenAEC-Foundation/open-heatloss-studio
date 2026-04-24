//! Gebruiksfunctie volgens het Besluit bouwwerken leefomgeving (Bbl).
//!
//! Alle gebruiksfuncties die NTA 8800:2025+C1:2026 onderscheidt, één-op-één
//! mapping naar de Bbl-indeling. Dient als input voor temperatuurprofielen,
//! gebruiksprofielen (bijlage D) en warmtebehoefte-benaderingen.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Gebruiksfunctie — Bbl-categorie.
///
/// `snake_case` voor JSON-compat met bestaande frontend/tools die al deze
/// namen gebruiken in zelf-generated schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageFunction {
    /// Woonfunctie (appartement, eengezinswoning).
    Woonfunctie,
    /// Bijeenkomstfunctie (vergaderruimte, horeca, religieuze bouw).
    Bijeenkomstfunctie,
    /// Celfunctie (penitentiaire inrichting).
    Celfunctie,
    /// Gezondheidszorgfunctie (ziekenhuis, verpleeghuis).
    Gezondheidszorgfunctie,
    /// Industriefunctie.
    Industriefunctie,
    /// Kantoorfunctie.
    Kantoorfunctie,
    /// Logiesfunctie (hotel).
    Logiesfunctie,
    /// Onderwijsfunctie.
    Onderwijsfunctie,
    /// Sportfunctie.
    Sportfunctie,
    /// Winkelfunctie.
    Winkelfunctie,
    /// Overige gebruiksfunctie die niet onder voorgaande categorieën valt.
    OverigeGebruiksfunctie,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_snake_case() {
        let json = serde_json::to_string(&UsageFunction::OverigeGebruiksfunctie).unwrap();
        assert_eq!(json, "\"overige_gebruiksfunctie\"");
    }

    #[test]
    fn serde_round_trip_all_variants() {
        for func in [
            UsageFunction::Woonfunctie,
            UsageFunction::Bijeenkomstfunctie,
            UsageFunction::Celfunctie,
            UsageFunction::Gezondheidszorgfunctie,
            UsageFunction::Industriefunctie,
            UsageFunction::Kantoorfunctie,
            UsageFunction::Logiesfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Winkelfunctie,
            UsageFunction::OverigeGebruiksfunctie,
        ] {
            let json = serde_json::to_string(&func).unwrap();
            let back: UsageFunction = serde_json::from_str(&json).unwrap();
            assert_eq!(func, back);
        }
    }
}
