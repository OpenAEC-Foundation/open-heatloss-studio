//! Domeintypen voor ventilatiesystemen, luchtstromen en WTW.
//!
//! De types in deze module beschrijven de **input** van de ventilatieberekening
//! (systeemkeuze, gemeten luchtstromen, WTW-specificatie). Rekenresultaten
//! leven in [`crate::result`].

pub mod flow;
pub mod system;
pub mod wtw;

pub use flow::AirFlow;
pub use system::VentilationSystem;
pub use wtw::WtwSpecification;
