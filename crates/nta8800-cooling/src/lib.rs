//! # NTA 8800 Cooling
//!
//! NTA 8800:2025+C1:2026 H.10 — koeling (actieve koelsystemen) en bijlage AA —
//! vereenvoudigde bepaling van de koelbehoefte en de minimaal benodigde
//! koelcapaciteit in woningen (TOjuli-opvolger).
//!
//! ## V1 scope
//!
//! **H.10 — actieve koeling (IN V1):**
//! - [`CoolingSystem`] enum met compressie, absorptie en vrije-koeling
//! - [`CoolingDistribution`] + [`CoolingEmission`] — η_dist / η_em voor koude
//! - [`calculate_cooling`] — Q_C;use = Q_C;nd / (η_em · η_dist · COP · f_reg)
//!
//! **Bijlage AA — vereenvoudigde koelbehoefte (IN V1):**
//! - Formules (AA.1) t/m (AA.13) voor woningen
//! - [`calculate_simplified_cooling`] — maatgevende koelbehoefte q_C (W/m²)
//!   per rekenzone plus minimum benodigde koelcapaciteit B_C;req;TO (kW)
//!
//! **NIET in V1:**
//! - Koelopwekker-type-specifieke correcties (bijlage Q analoog) — V2
//! - Koude-opslag / buffervaten — V2
//! - Vrije koeling via grondwisselaar met complexe regimes — V2
//!
//! ## Publieke API
//!
//! - Pad 1 (actief): [`calculate_cooling`] + [`CoolingResult`]
//! - Pad 2 (vereenvoudigd): [`calculate_simplified_cooling`] +
//!   [`SimplifiedCoolingResult`]
//!
//! Conventie voor norm-referentie constanten: zie [`references`] en
//! [`nta8800_model::references`].

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken veel NTA 8800-symbolen (Q_C;nd, P_int;calc, θ_e, …)
// die de backtick-heuristiek als false positive oppikt — consistent met
// nta8800-demand / nta8800-transmission.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;
pub mod simplified;

pub use calc::{
    calculate_cooling, calculate_simplified_cooling, SimplifiedAreaInput, SimplifiedLoadInput,
};
pub use errors::{CoolingCalcResult, CoolingError};
pub use model::{
    CoolingDistribution, CoolingEmission, CoolingSystem, EnergyCarrier, FIXED_INDOOR_TEMPERATURE_C,
    FIXED_OUTDOOR_DEDUCTION_W_PER_M2,
};
pub use result::{CoolingBreakdown, CoolingResult, SimplifiedCoolingResult};
