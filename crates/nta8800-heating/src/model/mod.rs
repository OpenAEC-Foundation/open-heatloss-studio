//! Invoer-modellen voor de vier keten-componenten.
//!
//! | Module | Component | Norm-referentie |
//! |---|---|---|
//! | [`emission`] | afgifte (η_em) | §9.3 + tabel 9.2 |
//! | [`distribution`] | distributie (η_dist) | §9.4 |
//! | [`generation`] | opwekking (η_gen) | §9.5 + bijlagen M/N/Q/R + pg 327 |
//! | [`control`] | regeling (f_reg) | §9.6 |

pub mod control;
pub mod distribution;
pub mod emission;
pub mod generation;

pub use control::ControlFactor;
pub use distribution::DistributionSystem;
pub use emission::EmissionSystem;
pub use generation::{EnergyCarrier, GenerationSystem, HRClass};
