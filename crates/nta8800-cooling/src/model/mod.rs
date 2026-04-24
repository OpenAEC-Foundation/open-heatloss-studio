//! Domein-types voor de cooling-crate.
//!
//! Bevat de installatie-types (koelsysteem, distributie, afgifte) en vaste
//! constanten voor bijlage AA (binnentemperatuur 24 °C, vaste aftrek 35 W/m²).

pub mod cooling_system;
pub mod distribution;
pub mod emission;

pub use cooling_system::{CoolingSystem, EnergyCarrier};
pub use distribution::CoolingDistribution;
pub use emission::CoolingEmission;

/// Vaste binnentemperatuur 24 °C voor bijlage AA.
///
/// Gebruikt in formule (AA.4) buitenlucht-bijdrage en (AA.7) transmissie via
/// transparante delen — de norm gaat uit van een vaste binnentemperatuur van
/// 24 °C tijdens de koel-piek in juli.
pub const FIXED_INDOOR_TEMPERATURE_C: f64 = 24.0;

/// Vaste aftrek 35 W/m² op de maatgevende koelbehoefte voor het bepalen van
/// de minimale benodigde koelcapaciteit — formules (AA.11) en (AA.13).
///
/// Komt overeen met de situatie waarin net voldaan wordt aan het criterium
/// TO_juli < 1,2 of GTO < 450 h (zie opmerking 1 bij formule (AA.11)).
pub const FIXED_OUTDOOR_DEDUCTION_W_PER_M2: f64 = 35.0;
