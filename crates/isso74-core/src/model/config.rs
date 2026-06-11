//! Assessment configuration: comfort class, ATG variant, usage hours, PMV params.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ISSO 74 comfort class (afb. 3.1–3.4 + Tabel 3.3).
///
/// The class determines the ATG upper/lower bound offsets. Class A uses the
/// **same numeric bounds as class B** (Tabel 3.3: "Zie bij klasse B"); the
/// difference is the extra requirement on personal influence, which is not a
/// temperature value and is therefore out of scope for this numeric toets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum ComfortClass {
    /// Klasse A — PPD max. ca. 5%. Numeriek identiek aan klasse B.
    A,
    /// Klasse B — PPD max. ca. 10%.
    B,
    /// Klasse C — PPD max. ca. 15%.
    C,
    /// Klasse D — PPD max. ca. 25%.
    D,
}

impl Default for ComfortClass {
    fn default() -> Self {
        ComfortClass::C
    }
}

/// ATG summer upper-bound variant (ISSO 74 §3, afb. 3.5 stroomschema).
///
/// * `Alpha` — ruimte met effectief bruikbare te openen ramen *zónder*
///   waarneembare actieve koeling → meeglijdende (adaptieve) bovengrens
///   `18,8 + 0,33·θ_rm + offset`.
/// * `Beta` — vaste horizontale bovengrens (26/27/28 °C voor klasse B/C/D).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AtgVariant {
    /// Meeglijdende (adaptieve) bovengrens.
    Alpha,
    /// Vaste horizontale bovengrens.
    Beta,
}

impl Default for AtgVariant {
    fn default() -> Self {
        AtgVariant::Alpha
    }
}

/// Usage hours definition — which hours count towards the assessment.
///
/// Default: office hours ma–vr 08:00–18:00. `weekdays` uses ISO weekday
/// numbering (1 = Monday … 7 = Sunday). An hour at index `h` counts as
/// "in use" when its weekday ∈ `weekdays` AND `start_hour <= hour_of_day <
/// end_hour`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UsageHours {
    /// First hour-of-day (inclusive), 0..=23. Default 8.
    pub start_hour: u8,
    /// Last hour-of-day (exclusive), 1..=24. Default 18.
    pub end_hour: u8,
    /// ISO weekdays that count (1 = Mon … 7 = Sun). Default [1,2,3,4,5].
    pub weekdays: Vec<u8>,
}

impl Default for UsageHours {
    fn default() -> Self {
        UsageHours {
            start_hour: 8,
            end_hour: 18,
            weekdays: vec![1, 2, 3, 4, 5],
        }
    }
}

impl UsageHours {
    /// True when the given ISO weekday + hour-of-day fall inside usage time.
    pub fn is_in_use(&self, iso_weekday: u8, hour_of_day: u8) -> bool {
        self.weekdays.contains(&iso_weekday)
            && hour_of_day >= self.start_hour
            && hour_of_day < self.end_hour
    }
}

/// PMV (Fanger / ISO 7730) model parameters used by the GTO weighting.
///
/// **Toets-laag aanname (ISSO 74 §A):** the assessment only has θ_o (operative
/// temperature) per hour, so we assume `t_air ≈ t_mrt ≈ θ_o`. The remaining
/// Fanger inputs use the defaults below and are configurable. This is NOT a
/// full comfort simulation — see [`crate::calc::pmv`] for the documented
/// assumption set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PmvParams {
    /// Relative humidity [%]. Default 50.
    pub relative_humidity_pct: f64,
    /// Relative air velocity [m/s]. Default 0.1.
    pub air_velocity_m_s: f64,
    /// Clothing insulation in summer [clo]. Default 0.7.
    pub clo_summer: f64,
    /// Clothing insulation in winter [clo]. Default 0.9.
    pub clo_winter: f64,
    /// Metabolic rate [met]. Default 1.2 (= 70 W/m²).
    pub metabolic_rate_met: f64,
    /// External work [met]. Default 0.0.
    pub external_work_met: f64,
}

impl Default for PmvParams {
    fn default() -> Self {
        PmvParams {
            relative_humidity_pct: 50.0,
            air_velocity_m_s: 0.1,
            clo_summer: 0.7,
            clo_winter: 0.9,
            metabolic_rate_met: 1.2,
            external_work_met: 0.0,
        }
    }
}

/// Per-room ATG variant override.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RoomAtgOverride {
    /// Room name — must match a θ_o column header in the CSV.
    pub room: String,
    /// The ATG variant to use for this room.
    pub variant: AtgVariant,
}

/// Full assessment configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Isso74Config {
    /// Comfort class (A/B/C/D). Default C.
    #[serde(default)]
    pub comfort_class: ComfortClass,

    /// Global ATG variant; applies to every room unless overridden. Default α.
    #[serde(default)]
    pub atg_variant: AtgVariant,

    /// Per-room ATG variant overrides (header name → variant).
    #[serde(default)]
    pub room_atg_overrides: Vec<RoomAtgOverride>,

    /// Usage-hours window. Default office ma–vr 08:00–18:00.
    #[serde(default)]
    pub usage_hours: UsageHours,

    /// PMV / Fanger model parameters and clo/RH/v overrides.
    #[serde(default)]
    pub pmv: PmvParams,

    /// GTO richtwaarde (max weighted hours per grens). Default 150.
    #[serde(default = "default_gto_limit")]
    pub gto_limit_hours: f64,

    /// TO-uren richtwaarde >25 °C (max hours/jr). Default 100.
    #[serde(default = "default_to25_limit")]
    pub to25_limit_hours: f64,

    /// TO-uren richtwaarde >28 °C (max hours/jr). Default 20.
    #[serde(default = "default_to28_limit")]
    pub to28_limit_hours: f64,
}

fn default_gto_limit() -> f64 {
    150.0
}
fn default_to25_limit() -> f64 {
    100.0
}
fn default_to28_limit() -> f64 {
    20.0
}

impl Default for Isso74Config {
    fn default() -> Self {
        Isso74Config {
            comfort_class: ComfortClass::default(),
            atg_variant: AtgVariant::default(),
            room_atg_overrides: Vec::new(),
            usage_hours: UsageHours::default(),
            pmv: PmvParams::default(),
            gto_limit_hours: default_gto_limit(),
            to25_limit_hours: default_to25_limit(),
            to28_limit_hours: default_to28_limit(),
        }
    }
}

impl Isso74Config {
    /// Resolve the ATG variant for a specific room (override beats global).
    pub fn variant_for(&self, room: &str) -> AtgVariant {
        self.room_atg_overrides
            .iter()
            .find(|o| o.room == room)
            .map(|o| o.variant)
            .unwrap_or(self.atg_variant)
    }
}
