//! Domeintypen voor ventilatiesystemen, luchtstromen en WTW.
//!
//! De types in deze module beschrijven de **input** van de ventilatieberekening
//! (systeemkeuze, gemeten luchtstromen, WTW-specificatie). Rekenresultaten
//! leven in [`crate::result`].

pub mod flow;
pub mod pressure_context;
pub mod system;
pub mod wtw;

pub use flow::AirFlow;
pub use pressure_context::{BuildingLeakageType, BuildingPressureContext, C2_MAX_BUILDING_HEIGHT_M};
pub use system::VentilationSystem;
pub use wtw::WtwSpecification;
