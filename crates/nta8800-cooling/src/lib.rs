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
//! - Twee parallelle paden:
//!   - [`calculate_simplified_cooling`] — orchestratie via
//!     [`crate::simplified`] building blocks. P_sol en P_gl zijn
//!     caller-supplied, geen geïntegreerde tabel AA.3.
//!   - [`calculate_bijlage_aa`] — **volledige** bijlage AA met
//!     hardcoded tabel AA.3 (β × γ × tijdstip, 7 × 8 × 10 = 560 waarden),
//!     lineaire β-interpolatie, max-over-tijdstip per verblijfsruimte
//!     (AA.6b), AA.7 op maatgevend zone-tijdstip, AA.8-AA.13 in één call.
//!
//! **NIET in V1:**
//! - Koelopwekker-type-specifieke correcties (bijlage Q analoog) — V2
//! - Koude-opslag / buffervaten — V2
//! - Vrije koeling via grondwisselaar met complexe regimes — V2
//! - Automatische F_sh-bepaling uit overstek/zijbelemmering (tab 17.5/17.9/17.11)
//! - Cross-validatie tegen RVO-rekentool xlsm (placeholder #[ignore]-test)
//!
//! ## Publieke API
//!
//! - Pad 1 (actief): [`calculate_cooling`] + [`CoolingResult`]
//! - Pad 2 (vereenvoudigd, low-level): [`calculate_simplified_cooling`] +
//!   [`SimplifiedCoolingResult`]
//! - Pad 3 (Bijlage AA, complete): [`calculate_bijlage_aa`] +
//!   [`BijlageAaResult`] — zie [`bijlage_aa`] module-doc voor details.
//!
//! ## Bronnen Bijlage AA
//!
//! 1. Concept-tekst bijlage AA NTA 8800:2025 (internetconsultatie EPG2026):
//!    `https://www.internetconsultatie.nl/epg2026/document/14174`
//! 2. RVO-rekentool xlsm versie 2025.04: `tests/references/
//!    rekentool-bijlage-aa-nta8800-2025.04.xlsm` (gitignored)
//! 3. NEN 5060:2018+A1:2021 voor klimaat-data tabel AA.1.
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

pub mod bijlage_aa;
pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;
pub mod simplified;

pub use bijlage_aa::{
    calculate_bijlage_aa, BijlageAaInput, BijlageAaResult, BouwjaarKlasseAa, Orientatie, RaamAa,
    RuimteAa, RuimteResultaatAa, ZonweringType,
};
pub use calc::{
    calculate_cooling, calculate_simplified_cooling, SimplifiedAreaInput, SimplifiedLoadInput,
};
pub use errors::{CoolingCalcResult, CoolingError};
pub use model::{
    CoolingDistribution, CoolingEmission, CoolingSystem, EnergyCarrier, FIXED_INDOOR_TEMPERATURE_C,
    FIXED_OUTDOOR_DEDUCTION_W_PER_M2,
};
pub use result::{CoolingBreakdown, CoolingResult, SimplifiedCoolingResult};
