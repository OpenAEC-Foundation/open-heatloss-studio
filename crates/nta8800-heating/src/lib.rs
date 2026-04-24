//! # nta8800-heating
//!
//! NTA 8800:2025+C1:2026 H.9 — verwarming (afgifte, distributie, opwekking, regeling).
//!
//! Integrator-crate die Q_H;nd (netto warmtebehoefte per maand uit
//! [`nta8800_demand`]) omrekent naar **Q_H;use** — het eindenergiegebruik per
//! energiedrager voor verwarming, gedeeld door de keten-rendementen:
//!
//! ```text
//! Q_H;use;mi = Q_H;nd;mi / (η_em × η_dist × η_gen × f_reg)   [MJ]
//! ```
//!
//! ## V1 scope (strict beperkt)
//!
//! **IN V1:**
//! - §9 afgifte via een eenvoudig afgifterendement η_em per afgifte-type
//!   ([`EmissionSystem`]). NTA 8800 H.9 drukt afgifteverliezen uit via
//!   ΔT-correcties (tabel 9.2) ipv een directe η_em. V1 reduceert dit tot
//!   één forfaitair η_em per type; de volledige ΔT-methodiek is V2.
//! - §9 distributie als lineair rendement η_dist ([`DistributionSystem`]).
//!   Een vaste default 0,95 voor "goed geïsoleerd, standaard leidingwerk".
//! - §9 opwekking — 4 generatortypes ([`GenerationSystem`]):
//!   - [`HRBoiler`](model::generation::GenerationSystem::HRBoiler) met
//!     [`HRClass`](model::generation::HRClass) (HR100/104/107). Waarden uit
//!     NTA 8800 H.9 tabel individueel cv-toestel, **pg 327**. Voor V1
//!     gebruiken we de HT-kolom (70-90 °C aanvoer), de meest voorkomende
//!     situatie in woningbouw.
//!   - `HeatPump { scop }` met user-supplied SCOP (seizoensgemiddelde COP).
//!   - `ElectricResistance` met η_gen = 1,0 (100 % elektrische conversie).
//!   - `DistrictHeating { factor }` met forfaitair factor ≤ 1,0.
//! - §9 regeling via een forfaitaire factor f_reg ([`ControlFactor`]).
//!
//! **NIET in V1 (V2 scope):**
//! - Bijlage M (verbrandingssystemen detail, EN 15316-4-1 mapping)
//! - Bijlage N (warmteopwekkers incl. kachels, pellets)
//! - Bijlage O (elektrisch hulpenergiegebruik CV — hulpenergie-berekening)
//! - Bijlage Q (warmtepompen W/W, B/W, L/W, L/L type-specifieke correcties)
//! - Bijlage R (biomassa-emissie)
//! - Hybride systemen (ketel + WP combinatie)
//! - ΔT-correcties H.9 tabel 9.2 voor afgifte-verliezen
//! - Differentiatie LT/HT opwekkingsrendement op basis van aanvoertemperatuur
//!   (V1 kiest één kolom — HT — en exposeert HR-klasse enum).
//!
//! ## Publieke API
//!
//! - [`calculate_heating`] — entry-point die een [`DemandResult`] omzet naar
//!   [`HeatingResult`] gegeven de vier keten-componenten.
//! - [`EmissionSystem`], [`DistributionSystem`], [`GenerationSystem`],
//!   [`ControlFactor`] — invoer-modellen per keten-component.
//! - [`HeatingResult`] + [`HeatingBreakdown`] — resultaat met traceability.
//! - [`EnergyCarrier`] — energiedrager-annotatie voor downstream nEP-berekening.
//!
//! ## Energiedragers
//!
//! De eenheid van `Q_H;use` blijft altijd **MJ**, maar de betekenis hangt af
//! van de opwekker:
//!
//! | Generator | EnergyCarrier | Betekenis van Q_H;use |
//! |---|---|---|
//! | HR-ketel | Gas | Gas-energie (onderwaarde) |
//! | Warmtepomp | Electricity | Elektrische input-energie |
//! | Elektr. weerstand | Electricity | Elektrische input-energie |
//! | Stadsverwarming | DistrictHeat | Warmte-input bij grensvlak gebouw |
//!
//! [`DemandResult`]: nta8800_demand::DemandResult

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken NTA 8800-symbolen (η_em, Q_H;use, etc.) die de
// backtick-heuristiek als false positive oppikt — consistent met demand-crate.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_heating;
pub use errors::{HeatingCalcResult, HeatingError};
pub use model::{
    ControlFactor, DistributionSystem, EmissionSystem, EnergyCarrier, GenerationSystem, HRClass,
};
pub use result::{HeatingBreakdown, HeatingResult};
