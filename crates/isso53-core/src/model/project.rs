//! Project-level model for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Building, DesignConditions, Room, VentilationConfig};

/// Complete project data for ISSO 53 heat loss calculation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Project metadata.
    pub info: ProjectInfo,

    /// Building-level configuration.
    pub building: Building,

    /// Climate conditions.
    pub climate: DesignConditions,

    /// Ventilation system configuration.
    pub ventilation: VentilationConfig,

    /// List of rooms to calculate.
    pub rooms: Vec<Room>,
}

/// Project metadata and documentation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Project name.
    pub name: String,

    /// Optional project number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_number: Option<String>,

    /// Building address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Client name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,

    /// Calculation date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Engineer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engineer: Option<String>,

    /// Additional notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}