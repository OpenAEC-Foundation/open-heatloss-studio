//! Domain model for ISSO 74 thermal comfort assessments.
//!
//! This module contains the input types for an ISSO 74 toets (assessment).
//! The assessment is a **toets-laag**: the engineer supplies hourly operative
//! temperatures per room (from an external dynamic simulation, via CSV), and
//! this crate computes RMOT, the ATG bandwidth test, TO-uren, and GTO weighted
//! hours. We do NOT run a dynamic simulation here.

pub mod config;
pub mod request;

pub use config::{AtgVariant, ComfortClass, Isso74Config, PmvParams, UsageHours};
pub use request::Isso74Request;
