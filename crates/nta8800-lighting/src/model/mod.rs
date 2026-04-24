//! Invoer-model voor een [`LightingSystem`].
//!
//! NTA 8800 H.14 onderscheidt vier dimensieloze factoren die tezamen het
//! verlichtings-energiegebruik bepalen:
//!
//! | Factor | Symbool (norm) | Bron |
//! |---|---|---|
//! | Geïnstalleerd vermogen | `P_n` in W/m² | §14.3 + tabel 14.3 |
//! | Bezettingsfactor | `F_o;D / F_o;N → F_u` | §14.5 (V1 lumped) |
//! | Daglichtcorrectie | `F_D` | §14.6 + bijlage Y |
//! | Nieuwwaarde-compensatie | `F_C` | §14.4 + tabel 14.4 |
//!
//! V1 abstraheert deze tot één [`LightingSystem`]-struct met vier scalars.

pub mod lighting_system;

pub use lighting_system::{EnergyCarrier, LightingSystem};
