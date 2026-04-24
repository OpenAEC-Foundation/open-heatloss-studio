//! Oppervlakte-bepaling volgens NTA 8800:2025+C1:2026 bijlage K.
//!
//! Bijlage K onderscheidt twee soorten geprojecteerde oppervlakten:
//!
//! - **`A_T` (geprojecteerd, buitenwerks)** — voor de bepaling van de
//!   energieprestatie-indicator: oppervlakte van het denkbeeldige platte
//!   vlak begrensd door de adiabatisch veronderstelde afsnijvlakken, op
//!   twee decimalen nauwkeurig. Zie [`NTA_8800_2025_BIJLAGE_K_1_3`].
//!
//! - **`A_con` (voor U-bepaling, binnenwerks)** — oppervlakte waarvoor de
//!   warmtedoorgangscoëfficiënt `U` wordt opgegeven. Voor ramen/deuren
//!   geldt `A_con = A_gl + A_fr`. Zie [`NTA_8800_2025_BIJLAGE_K_1_2`].
//!
//! Voor wanden en vloeren kan `A_T` ≠ `A_con` (bijlage K detail 101.0.1.01
//! e.a.). Het verschil wordt toegewezen aan de lineaire koudebrug (ψ).
//!
//! [`NTA_8800_2025_BIJLAGE_K_1_2`]: crate::references::NTA_8800_2025_BIJLAGE_K_1_2
//! [`NTA_8800_2025_BIJLAGE_K_1_3`]: crate::references::NTA_8800_2025_BIJLAGE_K_1_3

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod flat;
pub mod inclined;
pub mod opening_deduction;

/// Conventie voor het meten van een oppervlakte.
///
/// Bijlage K beschrijft per constructie-type of de oppervlakte **buitenwerks**
/// (als denkbeeldig plat vlak tussen adiabatisch veronderstelde afsnijvlakken,
/// zie K.1.3) of **binnenwerks** (begrensd door de binnenwerkse randen van
/// het onderdeel, zie K.1.2) bepaald wordt.
///
/// | Constructie | Referentievlak | Paragraaf |
/// |---|---|---|
/// | Geprojecteerde oppervlakte `A_T` voor EP-indicator | Buitenwerks | K.1.3 |
/// | Oppervlakte `A_con` voor U-bepaling | Binnenwerks | K.1.2 |
/// | Vloer grenzend aan grond-/kruipruimte | Binnenwerks (binnenzijde buitenwand) | §6.9.3 / K.1.3 |
/// | Raam / deur (inclusief kozijn) | Binnenwerkse kozijnranden | K.1.3 / K.2 |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementReference {
    /// Buitenwerks — begrensd door adiabatisch veronderstelde afsnijvlakken
    /// aan de buitenzijde (K.1.3).
    #[default]
    Buitenwerks,
    /// Binnenwerks — begrensd door de binnenwerkse randen van het onderdeel
    /// (K.1.2, en voor ramen/deuren: binnenwerkse kozijnranden).
    Binnenwerks,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_buitenwerks() {
        assert_eq!(
            MeasurementReference::default(),
            MeasurementReference::Buitenwerks
        );
    }

    #[test]
    fn serde_snake_case() {
        let json = serde_json::to_string(&MeasurementReference::Binnenwerks).unwrap();
        assert_eq!(json, "\"binnenwerks\"");
        let json2 = serde_json::to_string(&MeasurementReference::Buitenwerks).unwrap();
        assert_eq!(json2, "\"buitenwerks\"");
    }

    #[test]
    fn serde_round_trip() {
        for mr in [
            MeasurementReference::Buitenwerks,
            MeasurementReference::Binnenwerks,
        ] {
            let json = serde_json::to_string(&mr).unwrap();
            let back: MeasurementReference = serde_json::from_str(&json).unwrap();
            assert_eq!(mr, back);
        }
    }
}
