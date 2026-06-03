//! Ventilation configuration for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{VentilationSystemType, VentilatieBouwfase};

/// Backward-compatibele serde-default voor [`VentilationConfig::bouwfase`].
///
/// Kiest **Nieuwbouw** zodat bestaande opgeslagen projecten en third-party
/// JSON zonder `bouwfase`-veld exact het oude gedrag behouden (de ventilatie-
/// debieten waren vóór D2 hardcoded op `VentilatieBouwfase::Nieuwbouw`). Een
/// stille `Default::default()` zou hier niet bestaan — de enum heeft bewust
/// géén `Default`-impl, omdat de norm-correcte keuze projectafhankelijk is
/// (bestaande bouw mag de soepelere tabel 4.10-eisen gebruiken). Die keuze
/// hoort via de UI gemaakt te worden; deze default is puur compat, geen
/// norm-aanbeveling.
fn default_ventilatie_bouwfase() -> VentilatieBouwfase {
    VentilatieBouwfase::Nieuwbouw
}

/// Ventilation system configuration.
/// ISSO 53 supports systems A/B/C/D/E (E is new for ISSO 53).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VentilationConfig {
    /// Ventilatiesysteemtype (A/B/C/D/E).
    pub system_type: VentilationSystemType,

    /// Bouwfase die de minimale ventilatie-eisen bepaalt (ISSO 53 tabel 4.10):
    /// `Nieuwbouw` (strenger) vs `Bestaand` (soepeler dm³/s·pp). Voorheen
    /// hardcoded op `Nieuwbouw` (D2-bevinding: ~+89% Φ_V voor bestaande bouw).
    /// Serde-default = `Nieuwbouw` voor backward-compat (zie
    /// [`default_ventilatie_bouwfase`]).
    #[serde(default = "default_ventilatie_bouwfase")]
    pub bouwfase: VentilatieBouwfase,

    /// Whether the system has heat recovery (WTW).
    #[serde(default)]
    pub has_heat_recovery: bool,

    /// Heat recovery efficiency (0.0-1.0).
    /// Only relevant if `has_heat_recovery` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_recovery_efficiency: Option<f64>,

    /// Frost protection setting.
    /// Reduces WTW efficiency when outdoor temperature is very low.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frost_protection: Option<f64>,

    /// Supply air temperature θ_t in °C.
    /// For heated supply air systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_temperature: Option<f64>,

    /// Whether the system has preheating.
    #[serde(default)]
    pub has_preheating: bool,

    /// Preheating temperature in °C.
    /// Only relevant if `has_preheating` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheating_temperature: Option<f64>,
}