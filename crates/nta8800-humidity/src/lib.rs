//! # nta8800-humidity
//!
//! NTA 8800:2025+C1:2026 H.12 Bevochtiging en Ontvochtiging — maandmethode.
//!
//! Berekent per [`nta8800_model::Rekenzone`]:
//! - Maandelijkse bevochtigings-energie `Q_hum;mi` in MJ
//! - Maandelijkse ontvochtigings-energie `Q_dhum;mi` in MJ
//! - Elektrisch energiegebruik humidification `W_hum;mi` in MJ elektrisch
//!
//! Ondersteunt verschillende humiditeitssystemen: stoomgeneratoren, sproeikoelers,
//! en gedeeltelijke recirculatie met voorverwarming.
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | Absolute vochtigheid berekening | Ja (x_ODA, x_IDA) | — |
//! | Bevochtigingsbehoefte Q_hum | Ja (formule 12.1) | — |
//! | Ontvochtigingsbehoefte Q_dhum | Ja (formule 12.2) | — |
//! | Stoom-humidifier energie | Ja (forfaitair η = 0.95) | Variabel via opstelling-type |
//! | Sproeikoeler energie | Ja (forfaitair η = 0.80) | Variabel via systeem-complexiteit |
//! | Adsorptie dehumidifier | Forfaitair (COP = 3.5) | Dynamisch via regeneratie-temp |
//! | Seizoensdiscriminatie | Winter/zomer threshold 15°C | Slim op basis van vochtbalans |
//! | Recirculatie-koeling | Nee | Ja (§12.4) |
//! | Gebouwautomatisering | Nee (andere crate) | `nta8800-automation` |
//!
//! Conventie voor norm-referentie constanten: zie
//! [`nta8800_model::references`].
//!
//! ## Eenheden
//!
//! Alle vochtigheid in **g/kg** (gram water per kg droge lucht).
//! Alle energiewaarden in **MJ** conform workspace-conventie.
//! Temperaturen in **°C**.
//!
//! ## Voorbeeld
//!
//! ```
//! use nta8800_model::time::MonthlyProfile;
//! use nta8800_model::zoning::Rekenzone;
//! use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
//! use nta8800_humidity::{
//!     calculate_humidity, HumiditySystemConfig, HumidificationSystem,
//!     DehumidificationSystem, HumidityTarget,
//! };
//!
//! let zone = Rekenzone {
//!     id: "rz1".into(),
//!     name: "Kantoorruimte".into(),
//!     gebouw_id: "g1".into(),
//!     floor_area: 100.0,
//!     volume: 275.0,
//!     efr_ids: vec![],
//!     constructions: vec![],
//!     windows: vec![],
//!     openings: vec![],
//!     thermal_bridges_linear: vec![],
//!     thermal_bridges_point: vec![],
//! };
//! let system_config = HumiditySystemConfig {
//!     humidification: Some(HumidificationSystem::Steam { efficiency: 0.95 }),
//!     dehumidification: Some(DehumidificationSystem::Adsorption { cop: 3.5 }),
//!     target: HumidityTarget { min_g_per_kg: 6.0, max_g_per_kg: 12.0 },
//! };
//! let indoor_temp = MonthlyProfile::from_constant(21.0);
//! let climate = de_bilt_climate_data();
//!
//! let result = calculate_humidity(&zone, &system_config, &indoor_temp, &climate).unwrap();
//! assert!(result.annual_q_hum >= 0.0);
//! assert!(result.annual_q_dhum >= 0.0);
//! assert!(result.annual_w_hum >= 0.0);
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments bevatten veel NTA 8800-symbolen (HVAC, COP, x_ODA, x_IDA,
// formule-nummers met punten/komma's). Backticks om elke enumeratie halen
// heen maakt de docs onleesbaar — clippy's heuristiek matcht ze als
// "missing backticks" false-positief.
#![allow(clippy::doc_markdown)]

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_humidity;
pub use errors::HumidityError;
pub use model::{
    DehumidificationSystem, HumidificationSystem, HumiditySystemConfig, HumidityTarget,
};
pub use result::HumidityResult;