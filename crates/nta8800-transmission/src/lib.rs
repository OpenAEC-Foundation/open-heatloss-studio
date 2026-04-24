//! # nta8800-transmission
//!
//! NTA 8800:2025+C1:2026 H.8 Transmissie — maandmethode.
//!
//! Berekent maandelijkse transmissiewarmteverliezen per rekenzone naar:
//! - Buitenlucht (`H_D` — §8.2.1, formule (8.1))
//! - Onverwarmde ruimtes (`H_U` — §8.4, formule (8.52), via user-supplied b-factor)
//! - Grond (`H_g;an` — §8.3 vereenvoudigd via bijlage I.2.3 pad)
//! - Aangrenzende verwarmde zone (`H_A` — §8.5; NTA-default 0, opt-in via
//!   formules (8.60)/(8.61))
//! - Lineaire + puntvormige koudebruggen (§8.2.3, §8.2.4)
//!
//! De maand-formule is §7.3.2 formule (7.14): `Q_H;tr;zi;mi = H_tr · ΔT · 0.001 · t_mi`
//! met `H_tr = H_D + H_U + H_A + H_p` (formule (7.16), `H_p` niet in V1 — die
//! leeft in [`nta8800_model`] via `Rekenzone::thermal_bridges_*` en valt onder
//! §7.3.3 verticale leidingen, uit te werken in een latere iteratie).
//!
//! ## Scope-grens V1
//!
//! **In V1:**
//! - §7.3.2 formule (7.14) maand-Q voor verwarming
//! - §7.3.2 formule (7.16) som deelcoëfficiënten
//! - §8.2.1 formule (8.1) H_D met ondoorschijnende delen + ramen + bruggen
//! - §8.4.1 formule (8.52)/(8.53) H_U met user-supplied b-factor
//! - §8.5 (H_A default 0) en opt-in formules (8.60)/(8.61) via
//!   `adjacent_zone_temperatures` map
//! - §8.3.1 vereenvoudigd via bijlage I.2.3 (user-supplied `H_g;an`)
//! - §8.2.3 + §8.2.4 lineaire en punt-bruggen
//!
//! **Niet in V1:**
//! - Bijlage A — Dynamisch transparante gebouwelementen (zonwering-logica), V2
//! - Bijlage B — Effectieve interne warmtecapaciteit (hoort bij `nta8800-demand`)
//! - Volledige bijlage C R-waarde procedure (correcties voor luchtlagen,
//!   niet-homogene lagen, etc.); `Construction::r_total()` uit model volstaat
//! - Bijlage D — maandelijkse faseverschuiving voor grondtransmissie
//! - Bijlage J — gedeclareerde λ/R waarden (certificeringsmateriaal)
//! - §7.3.3 — warmteoverdracht via verticale leidingen
//! - §8.2.2 forfaitaire ΔU_for toeslag (formule (8.2)/(8.3))
//! - Koeling (formule (7.15)) — leeft in een equivalent cooling-crate
//!
//! ## Eenheden-conventie
//!
//! Deze crate rekent intern conform formule (7.14) in kWh en converteert naar
//! MJ (factor 3.6) in lijn met [`nta8800_model::units::Energy`]. Alle
//! resultaat-velden in [`TransmissionResult`] zijn in MJ.
//!
//! Conventie voor norm-referentie-constanten: zie [`references`].

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Norm-notatie (θ_i, H_D;ue, H_g;an, Q_H;tr;zi;mi, ψ·L, etc.) wordt veelvuldig
// in de module-doc gebruikt en past niet op de `item in documentation is
// missing backticks` lint — consistent met `nta8800-ventilation`.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::{calculate_transmission, GroundParameters, KWH_TO_MJ, MONTH_HOURS};
pub use errors::{TransmissionError, TransmissionResult as CalcResult};
pub use model::{BoundaryType, TransmissionElement};
pub use result::{TransmissionBreakdown, TransmissionResult};
