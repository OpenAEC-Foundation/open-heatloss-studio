//! # nta8800-automation
//!
//! NTA 8800:2025+C1:2026 H.15 Gebouwautomatisering en regeltechniek (BACS).
//!
//! Berekent correctiefactoren `f_BAC` voor energiegebruik afhankelijk van:
//! - Automatiseringsklasse (A, B, C, D) volgens NEN-EN 15232
//! - Gebruiksfunctie (woon vs. utiliteit)
//! - Type energiedienst (heating, cooling, lighting, dhw, ventilation)
//!
//! De factoren worden toegepast op het netto energiegebruik van heating/cooling/
//! lighting/dhw-modules om de invloed van gebouwautomatisering op efficiency
//! weer te geven.
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | BAC-klassen A/B/C/D | Ja (lookup-tabellen) | — |
//! | Correctiefactoren per dienst | Ja (heating/cooling/lighting/dhw/ventilation) | — |
//! | Woon vs. utiliteit discriminatie | Ja (2 sets tabellen) | — |
//! | Temporele regeling | Nee | Continue regeling, klok-schema's |
//! | Zonale regeling | Nee | Zone-specifieke f_BAC |
//! | Sensorische feedback | Nee | Aanwezigheids-/lichtdetectie |
//!
//! **V1 uitgangspunten:**
//! - Klasse D = niet-energie-efficiënt (f_BAC ≥ 1.0)
//! - Klasse C = standaard regeling
//! - Klasse B = geavanceerde regeling
//! - Klasse A = high performance regeling (f_BAC ≤ 1.0)
//! - Eenvoudige tabel-lookup zonder complexe temporele modellen
//!
//! Conventie voor norm-referentie constanten: zie
//! [`nta8800_model::references`].
//!
//! ## Eenheden
//!
//! Correctiefactoren zijn dimensieloos (ratio's). Alle andere eenheden
//! conform workspace-conventie: energie in **MJ**, temperaturen in **°C**.
//!
//! ## Voorbeeld
//!
//! ```
//! use nta8800_automation::{
//!     calculate_automation_factors, AutomationConfig, BacsClass,
//! };
//! use nta8800_model::zoning::UsageFunction;
//!
//! let config = AutomationConfig {
//!     heating: BacsClass::B,
//!     cooling: BacsClass::C,
//!     lighting: BacsClass::A,
//!     dhw: BacsClass::C,
//!     ventilation: BacsClass::B,
//! };
//!
//! let factors = calculate_automation_factors(&config, UsageFunction::Kantoorfunctie).unwrap();
//! assert!(factors.f_bac_heating <= 1.0); // Klasse B is beter dan D
//! assert!(factors.f_bac_lighting <= 1.0); // Klasse A is best
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments bevatten veel BACS/BAC/NEN-acroniemen en tabel-nummers.
// Deze zijn standaard terminologie — backticks zouden leesbaarheid verminderen.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_automation_factors;
pub use errors::AutomationError;
pub use model::{AutomationConfig, BacsClass};
pub use result::AutomationFactors;
