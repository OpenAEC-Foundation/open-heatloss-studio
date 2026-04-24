//! # nta8800-tables
//!
//! NTA 8800:2025+C1:2026 normatieve default-tabellen en klimaatdata.
//!
//! Bevat data uit H.17 Klimaatgegevens, bijlage E/F/G/H/I/L/X van de norm.
//! Deze crate bevat **alleen data en eenvoudige lookup-functies**, geen
//! rekenlogica — die hoort in thema-crates (`nta8800-transmission`,
//! `nta8800-heating`, etc.).
//!
//! # Module-indeling
//!
//! | Module | Norm-bron | Inhoud |
//! |---|---|---|
//! | [`climate`] | H.17 | Referentieklimaat De Bilt: temperatuur + zoninstraling per maand |
//! | [`rounding`] | Bijlage X | Significante-cijfers-afronding (tabel X.1) |
//! | [`materials`] | Bijlage E | λ bouwmaterialen *(stub — volgt in D3e)* |
//! | [`air_cavities`] | Bijlage F | λ equivalent luchtruimten *(stub — D3f)* |
//! | [`glazing`] | Bijlage G | U-waarde + g-waarde beglazing *(stub — D3g)* |
//! | [`frame_materials`] | Bijlage H | λ kozijnmaterialen *(stub — D3h)* |
//! | [`thermal_bridges`] | Bijlage I | ψ forfaitair koudebruggen *(stub — D3i)* |
//! | [`glazing_edge`] | Bijlage L | ψ beglazingsrand *(stub — D3l)* |
//!
//! # Conventies
//!
//! Norm-identifier constanten volgen het patroon uit
//! [`nta8800_model::references`]. Zie de module-doc van [`references`] voor
//! de lijst van tabel- en paragraaf-IDs die in deze crate worden gebruikt.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod air_cavities;
pub mod climate;
pub mod frame_materials;
pub mod glazing;
pub mod glazing_edge;
pub mod materials;
pub mod references;
pub mod rounding;
pub mod thermal_bridges;

// ----- Flat re-exports for ergonomic downstream use -----

pub use climate::{
    de_bilt_climate_data, DE_BILT_COOLING_REFERENCE_TEMPERATURE, DE_BILT_OUTDOOR_TEMPERATURE,
    DE_BILT_SOLAR_IRRADIATION, DE_BILT_WIND_SPEED, DE_BILT_WTW_PREHEAT_TEMPERATURE,
};
pub use rounding::{round_to_significant_figures, RoundingDirection, RoundingRule};
