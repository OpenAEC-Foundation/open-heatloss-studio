//! Thermische-massa classificatie voor een rekenzone.
//!
//! Combineert vloer-, wand- en plafondclassificatie uit NTA 8800 tabel 7.10/
//! 7.11/7.12 (zie [`nta8800_tables::thermal_capacity`]) tot één invoer-struct
//! voor de demand-crate. De crate leest hieruit `D_m;int;eff;zi` en
//! `C_m;int;eff;zi` zonder dat de consumer de drie enums apart hoeft door te
//! geven.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_tables::thermal_capacity::{CeilingType, FloorMassClass, WallMassClass};

/// Classificatie van de thermische massa van een rekenzone.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ThermalMassInput {
    /// Vloer-massaklasse (tabel 7.11).
    pub floor: FloorMassClass,
    /// Wand-massaklasse (tabel 7.12).
    pub wall: WallMassClass,
    /// Plafondtype voor kolomkeuze in tabel 7.10.
    pub ceiling: CeilingType,
}

impl ThermalMassInput {
    /// Construct met de drie classificaties.
    #[must_use]
    pub const fn new(floor: FloorMassClass, wall: WallMassClass, ceiling: CeilingType) -> Self {
        Self {
            floor,
            wall,
            ceiling,
        }
    }

    /// Default voor "lichte woning" (HSB/SFB met gesloten plafond) → `D_m = 55`.
    #[must_use]
    pub const fn light_woning() -> Self {
        Self {
            floor: FloorMassClass::Light,
            wall: WallMassClass::Light,
            ceiling: CeilingType::ClosedOrSuspended,
        }
    }

    /// Default voor "zware woning" (massief beton, open plafond) → `D_m = 450`.
    #[must_use]
    pub const fn zwaar_massief() -> Self {
        Self {
            floor: FloorMassClass::VeryHeavy,
            wall: WallMassClass::VeryHeavy,
            ceiling: CeilingType::OpenOrNone,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_tables::thermal_capacity::specific_heat_capacity;

    #[test]
    fn light_woning_geeft_55() {
        let m = ThermalMassInput::light_woning();
        let d = specific_heat_capacity(m.floor, m.wall, m.ceiling);
        assert!((d - 55.0).abs() < 1e-9);
    }

    #[test]
    fn zwaar_massief_geeft_450() {
        let m = ThermalMassInput::zwaar_massief();
        let d = specific_heat_capacity(m.floor, m.wall, m.ceiling);
        assert!((d - 450.0).abs() < 1e-9);
    }

    #[test]
    fn serde_round_trip() {
        let m = ThermalMassInput::light_woning();
        let json = serde_json::to_string(&m).unwrap();
        let back: ThermalMassInput = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
