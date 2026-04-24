//! Transmissie-specifieke invoer-types.
//!
//! Deze sub-module voegt types toe die niet in [`nta8800_model`] thuishoren
//! omdat ze puur rekenkundig zijn (boundary-classificatie, transmissie-element
//! binding tussen constructie en oppervlak). De foundation-crate
//! `nta8800-model` blijft vrij van transmissie-specifieke semantiek.

pub mod boundary;
pub mod transmission_element;

pub use boundary::BoundaryType;
pub use transmission_element::TransmissionElement;
