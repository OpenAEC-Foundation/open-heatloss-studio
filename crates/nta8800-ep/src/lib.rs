//! # nta8800-ep
//!
//! NTA 8800:2025+C1:2026 H.5 EP-score integratie + primair energiegebruik + beleidsfactoren.
//!
//! Berekent de totale EP-score van een gebouw door integratie van netto energiegebruik
//! van alle diensten (heating, cooling, dhw, lighting, ventilation, automation)
//! met primaire energiefactoren en CO2-beleidsfactoren.
//!
//! Dekt:
//! - **H.5**: Primair energiegebruik integratie
//! - **Bijlage Z**: Primaire energiefactoren per energiedrager
//! - **Bijlage AB**: CO2-beleidsfactoren
//!
//! ## V1 scope & bewuste vereenvoudigingen
//!
//! | Element | V1 | V2 |
//! |---|---|---|
//! | EP-score berekening (A++++ t/m G) | Ja (tabel-lookup) | — |
//! | Primaire energiefactoren bijlage Z | Ja (2023 waarden) | Jaarlijkse updates |
//! | CO2-beleidsfactoren bijlage AB | Ja (2023 waarden) | Jaarlijkse updates |
//! | PV hernieuwbaar aandeel | Ja (simpel saldo) | Net-meterings complexiteit |
//! | Energiedragers | Basis set (5 typen) | Uitgebreide set warmtepompen |
//! | Gebouwfuncties | Woon/utiliteit | Subsector-specifieke drempels |
//!
//! **V1 uitgangspunten:**
//! - EP-score volgens H.5: E_P;tot = Σ(Q_dienst × f_prim) / A_g [MJ/m²]
//! - PV-saldering: hernieuwbaar aandeel = min(1.0, PV_yield / Q_totaal)
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

pub mod calc;
pub mod errors;
pub mod model;
pub mod references;
pub mod result;

pub use calc::calculate_ep_score;
pub use errors::EpError;
pub use model::{BuildingArea, EnergyCarrier, EpInputs};
pub use result::{EpBreakdown, EpLabel, EpResult, ServiceBreakdown};
