//! Room model for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::construction::ConstructionElement;
use super::enums::{GebruiksFunctie, RuimteType};

/// A single room/space to be calculated.
/// ISSO 53 uses gebruiksfunctie + ruimtetype instead of room function.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    /// Unique identifier for this room.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Gebruiksfunctie volgens Bouwbesluit.
    pub gebruiks_functie: GebruiksFunctie,

    /// Ruimtetype binnen de gebruiksfunctie.
    pub ruimte_type: RuimteType,

    /// Floor area in m².
    pub floor_area: f64,

    /// Ceiling height in m. Must be ≤ 4.0 for ISSO 53.
    pub height: f64,

    /// Custom indoor design temperature in °C.
    /// If None, use lookup from tabel 2.2 based on gebruiksfunctie + ruimtetype.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_temperature: Option<f64>,

    /// Construction elements (walls, windows, doors, etc.) forming the room boundary.
    pub constructions: Vec<ConstructionElement>,

    /// Bezettingsinformatie voor ventilatie-eis berekening.
    pub bezetting: Bezetting,

    /// Reductiefactor z voor infiltratie op vertrekniveau (tabel 4.4).
    /// 1.0 = 1 buitengevel of 2 niet-tegenover, 0.5 = 2 tegenover, 0.7 = overig.
    #[serde(default = "default_infiltration_z")]
    pub infiltration_reduction_z: f64,

    /// Of de ruimte mechanische toevoer van ventilatielucht heeft. In ISSO 53
    /// telt alleen toevoer mee voor het ventilatiewarmteverlies; `Some(false)`
    /// → q_v = 0. `None` (veld afwezig in oudere fixtures) → geen gate.
    #[serde(default)]
    pub has_mechanical_supply: Option<bool>,

    /// Vastgestelde toevoer-luchtvolumestroom q_v in m³/s (fase 3, uitvoering).
    /// Indien `Some` gebruikt de ventilatieberekening deze waarde direct en
    /// negeert de BBL/bezetting-afleiding én de has_mechanical_supply-gate.
    /// `None` (oudere fixtures) → reguliere afleiding.
    #[serde(default)]
    pub ventilation_q_v_established: Option<f64>,
}

/// Bezettingsinformatie voor ventilatie-eisen.
/// ISSO 53 gebruikt dm³/s per persoon × personen/m².
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bezetting {
    /// Override aantal personen in deze ruimte.
    /// None = gebruik floor_area × personen_per_m2 uit tabel 4.11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personen: Option<f64>,

    /// Override bezettingsdichtheid in personen/m².
    /// None = gebruik default uit tabel 4.11 voor de gebruiksfunctie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personen_per_m2_default: Option<f64>,
}

fn default_infiltration_z() -> f64 {
    1.0
}