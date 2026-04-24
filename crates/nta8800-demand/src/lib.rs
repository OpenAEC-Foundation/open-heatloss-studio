//! # nta8800-demand
//!
//! NTA 8800:2025+C1:2026 H.7 — maandelijkse warmte- en koudebehoefte.
//!
//! Integrator-crate die transmissie + ventilatie + interne warmtelast +
//! zoninstraling combineert tot Q_H;nd en Q_C;nd per maand per rekenzone via
//! de maand-balans met benuttingsfactor.
//!
//! ## V1 scope
//!
//! **IN V1:**
//! - §7.4 formule (7.4) — maand-warmtebehoefte Q_H;nd
//! - §7.5 formule (7.10) — maand-koudebehoefte Q_C;nd
//! - §7.6 formules (7.6)–(7.7) — η_H,gn benuttingsfactor warmtewinst
//! - §7.6 formules (7.12)–(7.13) — η_C,ls benuttingsfactor koudeverlies
//! - §7.8 formule (7.17) — tijdconstante τ uit C_m / (H_tr + H_ve)
//! - §7.9 formule (7.33) — zoninstraling door ramen (A · g · F_sh · (1−F_F) · I_sol)
//! - §7.10 formule (7.35) + tabel 7.6 — forfaitaire interne warmtelast Φ_int
//!
//! **NIET in V1 (V2 of andere crates):**
//! - Uur-balans / hysteresis (V2)
//! - Gebouwautomatisering-correctie (`nta8800-automation`, H.15)
//! - Interzone warmte-uitwisseling (V2)
//! - Overhang/obstructie-schaduwmodel (V2; V1 gebruikt `F_sh = 1,0`)
//! - Ventilatieve koeling in koel-modus (V2)
//!
//! ## Publieke API
//!
//! - [`calculate_demand`] — entry-point voor één rekenzone
//! - [`DemandResult`] + [`DemandBreakdown`] — resultaat met traceability
//! - [`InternalGains::forfaitair`](model::InternalGains::forfaitair) — defaults
//!   per [`nta8800_model::zoning::UsageFunction`]
//! - [`ThermalMassInput`] — wrapper voor floor/wall/ceiling-classificatie
//!
//! Conventie voor norm-referentie constanten: zie [`nta8800_model::references`].

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken veel NTA 8800-symbolen (Q_H;nd, θ_i, η_H,gn, etc.)
// die de backtick-heuristiek als false positive oppikt — consistent met
// nta8800-transmission / nta8800-ventilation.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::{calculate_demand, DEFAULT_SHADING_FACTOR};
pub use errors::{DemandCalcResult, DemandError};
pub use model::{CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
pub use result::{DemandBreakdown, DemandResult};
