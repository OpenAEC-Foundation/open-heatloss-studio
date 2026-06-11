//! Result types for the ISSO 74 assessment (verdict + plot-data).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::AtgVariant;

/// One point on the ATG scatter plot (afb. 4.1): x = θ_rm, y = θ_o + bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AtgPlotPoint {
    /// Hour-of-year of this sample (1-based).
    pub hour_of_year: u32,
    /// Running mean outdoor temperature [°C] (x-axis).
    pub theta_rm: f64,
    /// Operative temperature [°C] (y-axis).
    pub theta_o: f64,
    /// ATG lower bound [°C] at this θ_rm.
    pub lower: f64,
    /// ATG upper bound [°C] at this θ_rm.
    pub upper: f64,
    /// True when this hour exceeds the upper bound (te warm).
    pub over_upper: bool,
    /// True when this hour falls below the lower bound (te koud).
    pub under_lower: bool,
}

/// ATG bandwidth result for one room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AtgResult {
    /// ATG variant used for this room.
    pub variant: AtgVariant,
    /// Usage hours that fell inside the θ_rm validity band and were assessed.
    pub assessed_hours: u32,
    /// Usage hours above the upper bound.
    pub hours_over_upper: u32,
    /// Usage hours below the lower bound.
    pub hours_under_lower: u32,
    /// Fraction (0..1) of assessed hours that exceed either bound.
    pub exceedance_fraction: f64,
    /// True when no usage hour exceeds either bound.
    pub passes: bool,
}

/// TO-uren result (ISSO 74 Bijlage A.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToHoursResult {
    /// Usage hours with θ_o > 25 °C.
    pub hours_over_25: u32,
    /// Usage hours with θ_o > 28 °C.
    pub hours_over_28: u32,
    /// Richtwaarde for >25 (default 100 u/jr).
    pub limit_25: f64,
    /// Richtwaarde for >28 (default 20 u/jr).
    pub limit_28: f64,
    /// True when both TO limits are met.
    pub passes: bool,
}

/// GTO result (ISSO 74 Bijlage A.2) — weighted overheating/undercooling hours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GtoResult {
    /// Σ wf over usage hours with PMV > 0,5 (overheating).
    pub weighted_hours_summer: f64,
    /// Σ wf over usage hours with PMV < −0,5 (undercooling).
    pub weighted_hours_winter: f64,
    /// Richtwaarde per grens (default 150 weeguren/jaar).
    pub limit: f64,
    /// True when both summer and winter weighted hours are within the limit.
    pub passes: bool,
}

/// Full per-room assessment result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RoomResult {
    /// Room name (CSV column header).
    pub room: String,
    /// ATG bandwidth verdict.
    pub atg: AtgResult,
    /// TO-uren verdict.
    pub to_hours: ToHoursResult,
    /// GTO weighted-hours verdict.
    pub gto: GtoResult,
    /// Overall room verdict — passes when ATG, TO and GTO all pass.
    pub passes: bool,
    /// ATG scatter-plot data (one point per assessed usage hour).
    pub plot: Vec<AtgPlotPoint>,
}

/// Project-level summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectSummary {
    /// Total rooms assessed.
    pub rooms_total: u32,
    /// Rooms that pass all three toetsen.
    pub rooms_passing: u32,
    /// Rooms that fail at least one toets.
    pub rooms_failing: u32,
}

/// The metadata describing the toets-laag assumptions used (surfaced for
/// reviewers — these are NOT norm values but engineering assumptions, ISSO 74
/// §A).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AssumptionNotes {
    /// Fixed text describing the PMV toets-laag assumption set.
    pub pmv_basis: String,
    /// Relative humidity used [%].
    pub relative_humidity_pct: f64,
    /// Air velocity used [m/s].
    pub air_velocity_m_s: f64,
    /// Summer clo.
    pub clo_summer: f64,
    /// Winter clo.
    pub clo_winter: f64,
    /// Metabolic rate [met].
    pub metabolic_rate_met: f64,
}

/// Top-level assessment result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Isso74Result {
    /// Per-room results.
    pub rooms: Vec<RoomResult>,
    /// Project-level summary.
    pub summary: ProjectSummary,
    /// Toets-laag assumption notes.
    pub assumptions: AssumptionNotes,
}
