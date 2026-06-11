//! Lookup tables and constants for ISSO 74 assessments.
//!
//! Pure data + thin band-evaluation helpers. The literal values come from
//! ISSO-publicatie 74 (2e druk) Tabel 3.3 (PDF p.58) and the validity remarks
//! on p.57.

pub mod atg;

pub use atg::{atg_bounds, AtgBounds, RMOT_VALID_MAX, RMOT_VALID_MIN};
