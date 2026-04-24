//! Bijlage AA — Vereenvoudigde bepaling van de koelbehoefte en de minimaal
//! benodigde koelcapaciteit in woningen (TOjuli-opvolger).
//!
//! Implementeert formules (AA.1) t/m (AA.13) uit NTA 8800:2025 bijlage AA
//! voor woningen. De bepalingsmethode is bedoeld om te toetsen of een woning
//! of bouwplan volgens de eisen van de bouwregelgeving het risico op
//! oververhitting voldoende beperkt — niet als vervanging voor een
//! dimensioneringsmethode.

pub mod capacity;
pub mod demand;
pub mod internal_load;
pub mod outdoor_load;

pub use capacity::required_cooling_capacity_kw;
pub use demand::maatgevende_koelbehoefte;
pub use internal_load::{
    interne_warmtelast_basis, interne_warmtelast_overig, interne_warmtelast_rekenwaarde,
    interne_warmtelast_woon,
};
pub use outdoor_load::koellast_buitenlucht;
