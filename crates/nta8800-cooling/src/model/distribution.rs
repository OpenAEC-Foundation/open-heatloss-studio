//! Koudedistributiesysteem — rendement η_dist voor koude.
//!
//! NTA 8800 H.10 onderscheidt analoog aan de warmte-kant een distributie-
//! component. V1 beperkt zich tot één rendement-factor; detail-modellering
//! (pompen, pijpverliezen, preheat-correcties) volgt in V2.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Koudedistributie-rendement η_dist voor koude.
///
/// Typische waarden:
/// - CV-leidingen binnen de gekoelde zone: 0,95–1,00
/// - Leidingen door ongekoelde zone met goede isolatie: 0,85–0,95
/// - Slecht geïsoleerde lange leidingen: 0,70–0,85
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoolingDistribution {
    /// η_dist;C — distributie-rendement voor koude, dimensieloos in (0, 1].
    pub efficiency: f64,
}

impl CoolingDistribution {
    /// Forfaitair default η_dist = 0,95 (geïsoleerde leidingen).
    #[must_use]
    pub const fn default_insulated() -> Self {
        Self { efficiency: 0.95 }
    }
}

impl Default for CoolingDistribution {
    fn default() -> Self {
        Self::default_insulated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_insulated() {
        let d = CoolingDistribution::default();
        assert!((d.efficiency - 0.95).abs() < 1e-9);
    }

    #[test]
    fn distribution_serde_round_trip() {
        let d = CoolingDistribution { efficiency: 0.88 };
        let json = serde_json::to_string(&d).unwrap();
        let back: CoolingDistribution = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
