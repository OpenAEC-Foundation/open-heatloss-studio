//! Invoer-modellen voor de vier keten-componenten + DWTW.
//!
//! | Module | Component | Norm-referentie |
//! |---|---|---|
//! | [`demand`] | Q_W;nd (nettowarmtebehoefte) | §13.2 + tabel 13.1 |
//! | [`emission`] | η_W;em (afgifte) | §13.3 + tabellen 13.2/13.3 |
//! | [`distribution`] | η_W;dis (distributie) | §13.4 |
//! | [`generation`] | η_W;gen (opwekking) | §13.8 + bijlagen T, W |
//! | [`recovery`] | DWTW (douche-warmteterugwinning) | §13.5 + bijlage U |

pub mod demand;
pub mod distribution;
pub mod emission;
pub mod generation;
pub mod recovery;

pub use demand::DhwDemand;
pub use distribution::DhwDistribution;
pub use emission::DhwEmission;
pub use generation::{DhwGenerationSystem, EnergyCarrier};
pub use recovery::DouchewtwRecovery;
