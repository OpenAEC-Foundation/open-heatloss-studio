//! # nta8800-ventilation
//!
//! NTA 8800:2025+C1:2026 H.11 Ventilatie — maandmethode.
//!
//! Berekent per [`nta8800_model::Rekenzone`]:
//! - Maandelijkse ventilatie-warmteverliezen `Q_V;mi` in MJ
//! - Ventilator-energiegebruik `W_fan;mi` in MJ elektrisch
//! - WTW-warmteterugwinning `Q_WTW;mi` in MJ
//!
//! Ondersteunt de 5 systeemvarianten uit NTA 8800 bijlage S (A, B, C, D, E).
//! Systeem D kent een `with_wtw` knop; E is decentrale balansventilatie
//! (impliciet met WTW).
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | Luchtstromen per systeemtype | Ja (`q_V;tot` heuristiek) | Volledige massabalans §11.2.1.5 |
//! | Q_V per maand | Ja (formule 11.106-template) | — |
//! | WTW-temperatuursprong | Ja (vereenvoudigd 11.108) | Volledige 11.107 met `f_prac;hr` |
//! | Ventilator-energie | Ja (forfaitair 11.142) | Met `f_regfan` uit 11.137 |
//! | Bypass-logica bij hoge T_buiten | Nee | Ja |
//! | Zomerspui `q_V;argI` | Nee | Ja |
//! | Ventilatieve koeling `q_V;argII` | Nee | Ja |
//! | Gebouwautomatisering (§15) | Nee (andere crate) | `nta8800-automation` |
//!
//! Conventie voor norm-referentie constanten: zie
//! [`nta8800_model::references`].
//!
//! ## Eenheden
//!
//! Alle luchtstromen in **m³/h** (NTA 8800 eenheid, niet m³/s). Alle
//! energiewaarden in **MJ** conform workspace-conventie. Temperaturen in
//! **°C** (verschillen uiteraard in K).
//!
//! ## Voorbeeld
//!
//! ```
//! use nta8800_model::time::MonthlyProfile;
//! use nta8800_model::zoning::Rekenzone;
//! use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
//! use nta8800_ventilation::{
//!     calculate_ventilation, AirFlow, VentilationSystem, WtwSpecification,
//! };
//!
//! let zone = Rekenzone {
//!     id: "rz1".into(),
//!     name: "Woonkamer".into(),
//!     gebouw_id: "g1".into(),
//!     floor_area: 120.0,
//!     volume: 300.0,
//!     efr_ids: vec![],
//!     constructions: vec![],
//!     windows: vec![],
//!     openings: vec![],
//!     thermal_bridges_linear: vec![],
//!     thermal_bridges_point: vec![],
//! };
//! let sys = VentilationSystem::D { with_wtw: true };
//! let flow = AirFlow::new(150.0, 150.0, 30.0);
//! let wtw = WtwSpecification::new(0.80, 0.45 / 3.6, true);
//! let indoor = MonthlyProfile::from_constant(20.0);
//! let climate = de_bilt_climate_data();
//!
//! let result = calculate_ventilation(&zone, &sys, &flow, Some(&wtw), &indoor, &climate).unwrap();
//! assert!(result.annual_q_v > 0.0);
//! assert!(result.annual_w_fan > 0.0);
//! assert!(result.annual_wtw_recovery > 0.0);
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments bevatten veel NTA 8800-symbolen (NEN, AHU, WTW, SFP, DHW,
// formule-nummers met punten/komma's). Backticks om elke enumeratie halen
// heen maakt de docs onleesbaar — clippy's heuristiek matcht ze als
// "missing backticks" false-positief.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_ventilation;
pub use errors::VentilationError;
pub use model::{AirFlow, VentilationSystem, WtwSpecification};
pub use result::VentilationResult;
