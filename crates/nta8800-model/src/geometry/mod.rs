//! Geometrie-gerelateerde types: constructies, ramen, openingen, koudebruggen.
//!
//! Dit sub-module groepeert de thermische schil-elementen. Installatietypen
//! (verwarmingsopwekkers, ventilatie-units) horen hier **niet** thuis — die
//! worden door de thema-crates (`nta8800-heating`, `nta8800-ventilation`,
//! enzovoort) gemodelleerd.

pub mod construction;
pub mod opening;
pub mod thermal_bridge;
pub mod window;

pub use construction::{Construction, ConstructionLayer};
pub use opening::Opening;
pub use thermal_bridge::{ThermalBridgeCategory, ThermalBridgeLinear, ThermalBridgePoint};
pub use window::{FrameType, GlassType, Window};
