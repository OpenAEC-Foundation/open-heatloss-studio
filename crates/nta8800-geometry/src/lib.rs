//! # nta8800-geometry
//!
//! NTA 8800:2025+C1:2026 gebouwbegrenzing- en oppervlakte/lengte-bepaling.
//!
//! Implementeert:
//!
//! - **H.6 Gebouwbegrenzing en schematisering** — indelingsregels voor
//!   `Gebouw → Rekenzone → EnergiefunctieRuimte` plus validatie. Zie
//!   [`boundary`].
//! - **Bijlage K Oppervlakte- en lengtebepaling** — bruto/netto vlakken,
//!   aftrek van openingen, projectie van schuine vlakken, lengtebepaling
//!   van lijnvormige elementen. Zie [`area`] en [`length`].
//!
//! Deze crate levert **hulpfuncties**, geen rekenstromen — thema-crates
//! (`nta8800-transmission`, `nta8800-demand`, etc.) gebruiken deze als
//! geometrische basis.
//!
//! Conventie voor norm-referentie-constanten: zie
//! [`nta8800_model::references`] en [`references`] voor deze crate.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod area;
pub mod boundary;
pub mod length;
pub mod references;

// ----- flat re-exports voor ergonomic `use nta8800_geometry::Foo;` -----

pub use area::{
    flat::{gross_wall_area, net_floor_area, net_wall_area},
    inclined::{horizontal_projection, vertical_projection},
    opening_deduction::net_construction_area,
    MeasurementReference,
};
pub use boundary::{rules_for_usage_function, validate_building, ZoneGroupingRule};
pub use length::{perimeter_rectangle, thermal_bridge_total_length};
