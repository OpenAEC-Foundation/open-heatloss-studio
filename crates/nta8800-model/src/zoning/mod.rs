//! Zonering: Gebouw → Rekenzones → Energiefunctieruimten.
//!
//! NTA 8800 §6 — de zonering bepaalt hoe het gebouw wordt opgedeeld voor de
//! energieprestatie­berekening. Een **Rekenzone** is een thermisch homogeen
//! gebied (NTA 8800 §6.2); een **Energiefunctieruimte (EFR)** beschrijft
//! het binnenklimaat-eisen­cluster binnen een rekenzone (§6.3).

pub mod building;
pub mod energy_function_room;
pub mod rekenzone;
pub mod usage_function;

pub use building::Gebouw;
pub use energy_function_room::EnergiefunctieRuimte;
pub use rekenzone::Rekenzone;
pub use usage_function::UsageFunction;
