//! Building-level configuration for ISSO 53 calculations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::enums::{BuildingShape, GebouwTypePositie, GebouwTypeWinddruk, ThermalMass, VentilationSystemType};

/// Building-level configuration for heat loss calculation.
/// ISSO 53 requires building-level properties for infiltration and thermal mass.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    /// Gebouwvorm voor infiltratie-berekening bij onbekende q_v10,kar (tabel 4.9).
    pub building_shape: BuildingShape,

    /// Bouwjaar voor infiltratie-berekening (formule 4.34).
    pub construction_year: u32,

    /// Positie binnen gebouw (enkellaags/meerlaags, tussen/kop/vrijstaand).
    /// Voor f_typ in tabel 4.8.
    pub building_position: GebouwTypePositie,

    /// Ventilatiesysteemtype A/B/C/D/E (tabel 4.7).
    pub ventilation_system: VentilationSystemType,

    /// Thermische massa van het gebouw (tabel 2.4).
    pub thermal_mass: ThermalMass,

    /// Gebouwtype voor winddrukberekening (tabel 4.6).
    /// Voor f_type factor bij onbekende q_v10,kar.
    #[serde(default = "default_wind_pressure_type")]
    pub wind_pressure_type: GebouwTypeWinddruk,

    /// Hoogte van het gebouwcomplex in meter, gemeten vanaf maaiveld
    /// tot de bovenste verdiepingsvloer. Gebruikt voor q_is-lookup uit
    /// tabel 4.5 (Known-pad infiltratie). Default 3,0 m bij None.
    #[serde(default)]
    pub building_height: Option<f64>,
}

fn default_wind_pressure_type() -> GebouwTypeWinddruk {
    GebouwTypeWinddruk::MeerlaagsStandaard
}