//! # NTA 8800 Model
//!
//! Gedeelde foundation-types voor de Nederlandse energieprestatienorm
//! **NTA 8800:2025+C1:2026**. Deze crate bevat geen rekenlogica — alle 13
//! andere `nta8800-*` crates (transmission, ventilation, heating, dhw,
//! cooling, humidity, lighting, pv, demand, ep, automation, geometry,
//! tables) bouwen op deze types voort.
//!
//! ## Opzet
//!
//! - [`time`] — maanden en maandelijkse data-profielen
//! - [`location`] — postcode, coördinaten, klimaatzone, oriëntatie, helling
//! - [`climate`] — maandelijkse buitentemperatuur + zoninstraling per oriëntatie
//! - [`geometry`] — constructie-opbouw, ramen, openingen, koudebruggen
//! - [`zoning`] — Gebouw → Rekenzone → Energiefunctieruimte hiërarchie
//! - [`units`] — type-aliases voor fysische grootheden (documentair)
//! - [`error`] — gedeelde `ModelError` enum voor validatie-fouten
//!
//! ## Scope-grens
//!
//! Installatie-types (verwarming, ventilatie, tapwater, PV, verlichting,
//! bevochtiging, automation) én rekenresultaat-types (`EnergyFlow`, `EPResult`,
//! `Demand`) leven expliciet in afzonderlijke thema-crates. Dit model houdt
//! alleen het thermische schil- en zoneringsmodel plus cross-cutting
//! utility-types.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod climate;
pub mod error;
pub mod geometry;
pub mod location;
pub mod references;
pub mod time;
pub mod units;
pub mod zoning;

// ----- flat re-exports for ergonomic `use nta8800_model::Foo;` -----

pub use climate::ClimateData;
pub use error::{ModelError, ModelResult};
pub use geometry::{
    Construction, ConstructionLayer, FrameType, GlassType, Opening, ThermalBridgeCategory,
    ThermalBridgeLinear, ThermalBridgePoint, Window,
};
pub use location::{ClimateZone, LatLon, Location, Orientation, Tilt};
pub use time::{Month, MonthlyProfile};
pub use units::{
    Area, Energy, Length, LinearThermalTransmittance, Power, SolarIrradiation, Temperature,
    ThermalConductance, ThermalResistance, ThermalTransmittance, WindSpeed,
};
pub use zoning::{EnergiefunctieRuimte, Gebouw, Rekenzone, UsageFunction};
