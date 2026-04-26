//! # NTA 8800 PV
//!
//! NTA 8800:2025+C1:2026 H.16 Fotovoltaïsche systemen en bijlage V Bronregeneratie.
//!
//! Berekent per [`nta8800_model::Gebouw`]:
//! - Maandelijkse PV-opbrengst `Q_PV;mi` in MJ elektrisch
//! - Jaarlijkse PV-opbrengst `Q_PV;jaar` in MJ elektrisch
//! - Inverter-verliezen en systeem-verliezen
//! - Tilt- en azimuth-correctie op zoninstraling
//!
//! Ondersteunt standaard PV-installaties met piek-vermogen, oriëntatie,
//! hellingshoek en systeem-efficiëntie. Bronregeneratie voor warmtepomp-
//! bronnen (bijlage V) is voorbereid als V2-uitbreiding.
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | Maandelijkse PV-opbrengst per systeem | Ja (H.16 formules) | — |
//! | Tilt/azimuth correctiefactoren | Ja (forfaitair) | Volledige interpolatie |
//! | Inverter-efficiëntie | Ja (constant η_inv) | Dynamisch η(P_load/P_rated) |
//! | Systeem-verliezen (bekabeling, vervuiling) | Ja (forfaitair %) | Per component |
//! | Temperatuur-correctie modules | Nee | Ja (T_cel vs T_amb) |
//! | Schaduw-modellering | Nee (handmatig via factor) | Ja (geometrisch) |
//! | Bronregeneratie WP (bijlage V) | Stub-types | Volledige implementatie |
//! | Multi-tracking/omvormers | Nee | Ja |
//!
//! Conventie voor norm-referentie constanten: zie [`references`].
//!
//! ## Eenheden
//!
//! Alle energie-waarden in **MJ** conform workspace-conventie. Vermogen in
//! **kWp** voor PV-piek-vermogen, **W** voor momentaan vermogen. Oriëntaties
//! in **graden** (0° = noord, 90° = oost, 180° = zuid, 270° = west).
//! Hellingshoeken in **graden** (0° = horizontaal, 90° = verticaal).
//!
//! ## Voorbeeld
//!
//! ```
//! use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
//! use nta8800_pv::{calculate_pv_yield, PvSystem, PvLocation};
//!
//! let location = PvLocation::new(52.1, 5.2)?; // Utrecht
//! let system = PvSystem::new(
//!     5.5,    // 5.5 kWp
//!     35.0,   // 35° hellingshoek
//!     180.0,  // zuid-oriëntatie
//!     0.85,   // 85% systeem-efficiëntie
//!     0.96,   // 96% inverter-efficiëntie
//! )?;
//!
//! let climate = de_bilt_climate_data();
//! let result = calculate_pv_yield(&[system], &location, &climate)?;
//!
//! assert!(result.annual_yield_mj > 0.0);
//! assert_eq!(result.monthly_yield_mj.as_array().len(), 12);
//! # Ok::<(), nta8800_pv::PvError>(())
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_pv_yield;
pub use errors::PvError;
pub use model::{BronregeneratieConfig, PvLocation, PvSystem};
pub use result::PvResult;
