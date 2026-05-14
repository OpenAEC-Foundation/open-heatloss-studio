//! # OpenAEC Project Shared (V2 schema)
//!
//! Drielagig project-model voor multi-calc projecten. Zie
//! [`docs/ADR-002-multi-calc-project.md`](../../docs/ADR-002-multi-calc-project.md).
//!
//! ```text
//! ProjectV2
//! ├── shared        Eénmalige cross-calc invoer (info + locatie + gebouwtype)
//! ├── geometry      Gedeelde geometrie (spaces / constructions / openings)
//! └── calcs         Per-norm specifieke inputs (isso51 / tojuli / future)
//! ```
//!
//! ## Backward-compatibility
//!
//! Bestaande ISSO 51-only projecten (`schema_version=1`) worden via
//! [`migration::from_legacy_v1`] geconverteerd. Een [`ProjectV2`] kan via
//! [`view::to_isso51_project`] de huidige [`isso51_core::model::Project`]
//! produceren zodat de bestaande calc-call werkt zonder rewrite.

#![deny(missing_docs)]

pub mod calcs;
pub mod geometry;
pub mod migration;
pub mod project;
pub mod shared;
pub mod view;

pub use calcs::{Calcs, Iso51Inputs, TojuliInputs};
pub use geometry::{Construction, ConstructionLayer, Opening, OpeningKind, SharedGeometry, Space};
pub use project::{ProjectV2, SCHEMA_VERSION};
pub use shared::{BuildingTypeShared, ResidentialType, SharedProject, UtilityType};
