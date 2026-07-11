//! # nta8800-ep
//!
//! NTA 8800:2025+C1:2026 H.5 EP-score integratie + primair energiegebruik + beleidsfactoren.
//!
//! Berekent de totale EP-score van een gebouw door integratie van netto energiegebruik
//! van alle diensten (heating, cooling, dhw, lighting, ventilation, automation)
//! met primaire energiefactoren en CO2-beleidsfactoren.
//!
//! Dekt:
//! - **§5.5**: Karakteristiek primair-fossiel energiegebruik + PV-saldering
//! - **Tabel 5.2** (§5.5.5): primaire energiefactoren f_prim per energiedrager
//! - **Tabel 5.3** (§5.5.6): CO2-emissiecoëfficiënten per energiedrager
//! - **§5.6**: hernieuwbaar aandeel (RERPrenTot)
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | EP-score berekening (A++++ t/m G) | Ja (tabel-lookup) | — |
//! | Primaire energiefactoren tabel 5.2 | Ja (2023 waarden) | Jaarlijkse updates |
//! | CO2-emissiecoëfficiënten tabel 5.3 | Ja (2023 waarden) | Jaarlijkse updates |
//! | PV-saldering (BENG 2) | Ja (§5.5, PV × 1,45) | Batterij-correctie §5.5.14a |
//! | Hernieuwbaar aandeel RER (BENG 3) | Ja (§5.6 formule 5.3) | rencold via koel-keten (F3b) |
//! | Energiedragers | Basis set (5 typen) | Uitgebreide set warmtepompen |
//! | Gebouwfuncties | Woon/utiliteit | Subsector-specifieke drempels |
//!
//! **V1 uitgangspunten:**
//! - EP-score volgens §5.5: E_PTot = Σ(afgenomen × fP;del) − PV × 1,45 [MJ], /A_g voor MJ/m²
//! - Hernieuwbaar aandeel: `RERPrenTot = EPrenTot / (EPTot + EPrenTot)` (formule 5.3),
//!   met omgevingswarmte van warmtepompen + PV in de teller (§5.6)
//! - Label-drempels conform 2023 tabellen (woon/utiliteit verschillende criteria)
//! - Vaste factoren (geen temporele/regionale variatie)
//!
//! Conventie voor norm-referentie constanten: zie [`references`].
//!
//! ## Eenheden
//!
//! - Energie: **MJ** (netto energiegebruik per dienst)
//! - Oppervlakte: **m²** (gebruiksoppervlakte A_g)
//! - EP-score: **MJ/m²** (specifiek primair energiegebruik)
//! - CO2: **kg/m²** (specifieke CO2-uitstoot)
//! - Factoren: dimensieloos
//!
//! ## Voorbeeld
//!
//! ```ignore
//! use nta8800_ep::{calculate_ep_score, EpInputs, BuildingArea, EnergyCarrier};
//! use nta8800_model::zoning::UsageFunction;
//! use std::collections::HashMap;
//!
//! let inputs = EpInputs {
//!     heating: [(EnergyCarrier::Aardgas, 15000.0)].into_iter().collect(),
//!     cooling: [(EnergyCarrier::Elektriciteit, 3000.0)].into_iter().collect(),
//!     dhw: HashMap::new(),
//!     lighting: HashMap::new(),
//!     ventilation_aux: HashMap::new(),
//!     automation: HashMap::new(),
//!     pv_yield: 0.0,
//!     renewable_ambient_heat_mj: 0.0,
//!     renewable_ambient_cold_mj: 0.0,
//!     building_area: BuildingArea { a_g: 150.0 },
//! };
//!
//! let result = calculate_ep_score(&inputs, UsageFunction::Woonfunctie).unwrap();
//! assert!(matches!(result.ep_label, nta8800_ep::EpLabel::B | nta8800_ep::EpLabel::C));
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments bevatten veel EP/NTA-acroniemen en formule-nummers.
// Deze zijn standaard terminologie — backticks zouden leesbaarheid verminderen.
#![allow(clippy::doc_markdown)]

pub mod beng;
pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;
pub mod tojuli;

pub use beng::{
    beng1_limit_woonfunctie_grondgebonden, BengAssessment, BengIndicators, BengLimits,
    IndicatorAssessment,
};
pub use tojuli::{
    tojuli_orientation, tojuli_zone, TojuliOrientationInput, TojuliOrientationResult,
    TojuliZoneResult,
};
pub use calc::calculate_ep_score;
pub use errors::EpError;
pub use model::{BuildingArea, EnergyCarrier, EpInputs};
pub use result::{EpBreakdown, EpLabel, EpResult, ServiceBreakdown};
