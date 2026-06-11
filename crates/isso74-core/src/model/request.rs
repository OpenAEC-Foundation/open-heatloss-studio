//! The top-level assessment request: CSV content + config.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::config::Isso74Config;

/// An ISSO 74 assessment request.
///
/// `csv` holds the raw simulation export (see [`crate::calc::csv`] for the
/// column contract). `config` selects the comfort class, ATG variant(s),
/// usage hours, and PMV parameters.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Isso74Request {
    /// Raw CSV content (hourly): `hour`/datetime, `T_buiten`, then one column
    /// of operative temperature θ_o per room (header = room name).
    pub csv: String,

    /// Assessment configuration.
    #[serde(default)]
    pub config: Isso74Config,
}
