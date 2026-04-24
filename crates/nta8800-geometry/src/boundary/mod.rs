//! Gebouwbegrenzing en zonering — NTA 8800 H.6.
//!
//! Valideert de hiërarchie `Gebouw → Rekenzone → EnergiefunctieRuimte` tegen
//! de indelings-voorschriften uit §6.5.2 en verwante paragrafen.
//!
//! Zie:
//! - [`rules`] — declaratieve regels per gebruiksfunctie (welke mix van
//!   gebruiksfuncties mag in één rekenzone).
//! - [`validation`] — `validate_building` retourneert `ModelError` bij
//!   schending.

pub mod rules;
pub mod validation;

pub use rules::{rules_for_usage_function, ZoneGroupingRule};
pub use validation::validate_building;
