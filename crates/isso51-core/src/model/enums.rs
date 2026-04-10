//! Enumerations used throughout the ISSO 51 calculation model.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type of boundary for a construction element.
/// Determines which heat loss formula is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryType {
    /// Direct to outside air (ISSO 51 §2.5.1)
    Exterior,
    /// To an unheated space adjacent to the dwelling (ISSO 51 §2.5.2)
    UnheatedSpace,
    /// To another heated room within the same dwelling (ISSO 51 §2.5.3)
    AdjacentRoom,
    /// To a neighboring dwelling/building (ISSO 51 §2.5.4)
    AdjacentBuilding,
    /// To the ground (ISSO 51 §2.5.5)
    Ground,
    /// To open water (canal/river/lake — woonboot use case).
    ///
    /// Not a norm category — this is an engineering choice for constructions
    /// that sit directly against open water. The design temperature comes
    /// from `DesignConditions.theta_water` (default 5 °C, override per project).
    /// Reports must include a footnote when this variant is used.
    Water,
}

/// Room function determines the design indoor temperature (θ_i).
/// Values from ISSO 51 Table 2.11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoomFunction {
    /// Living room / verblijfsruimte (20°C)
    LivingRoom,
    /// Kitchen / keuken (20°C)
    Kitchen,
    /// Bedroom / slaapkamer (20°C)
    Bedroom,
    /// Bathroom / badkamer (22°C)
    Bathroom,
    /// Toilet / toiletruimte (15°C)
    Toilet,
    /// Hallway / entree/gang (15°C)
    Hallway,
    /// Landing / overloop (15°C)
    Landing,
    /// Storage room / bergruimte (5°C if frost protection needed)
    Storage,
    /// Attic / zolder used as living space (20°C)
    Attic,
    /// Custom temperature
    Custom,
}

impl RoomFunction {
    /// Returns the design indoor temperature θ_i in °C.
    /// ISSO 51 Table 2.11.
    pub fn design_temperature(&self) -> f64 {
        match self {
            RoomFunction::LivingRoom => 20.0,
            RoomFunction::Kitchen => 20.0,
            RoomFunction::Bedroom => 20.0,
            RoomFunction::Bathroom => 22.0,
            RoomFunction::Toilet => 15.0,
            RoomFunction::Hallway => 15.0,
            RoomFunction::Landing => 15.0,
            RoomFunction::Storage => 5.0,
            RoomFunction::Attic => 20.0,
            RoomFunction::Custom => 20.0, // default, should be overridden
        }
    }
}

/// Type of heating system installed.
/// Affects Δθ values (Table 2.12) and system losses (§2.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HeatingSystem {
    /// Gas heater, wall-mounted heater
    LocalGasHeater,
    /// IR panels wall-mounted
    IrPanelWall,
    /// IR panels ceiling-mounted
    IrPanelCeiling,
    /// High-temperature radiators/convectors (medium temp > 50°C)
    RadiatorHt,
    /// Low-temperature radiators/convectors (medium temp ≤ 50°C)
    RadiatorLt,
    /// Ceiling heating
    CeilingHeating,
    /// Wall heating
    WallHeating,
    /// Baseboard/plinth heating
    PlinthHeating,
    /// Floor heating + HT radiators
    FloorHeatingWithRadiatorHt,
    /// Floor heating + LT radiators
    FloorHeatingWithRadiatorLt,
    /// Floor heating as main system (floor temp ≥ 27°C)
    FloorHeatingMainHigh,
    /// Floor heating as main system (floor temp < 27°C)
    FloorHeatingMainLow,
    /// Floor + wall heating combined
    FloorAndWallHeating,
    /// Fan-driven convectors/radiators (NEN-EN 16430)
    FanConvector,
}

/// Ventilation system type (A through E).
/// ISSO 51 §2.5.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VentilationSystemType {
    /// System A: Natural supply and natural exhaust
    SystemA,
    /// System B: Mechanical supply and natural exhaust
    SystemB,
    /// System C: Natural supply and mechanical exhaust
    SystemC,
    /// System D: Mechanical supply and mechanical exhaust (balanced)
    SystemD,
    /// System E: Combination of systems within one dwelling
    SystemE,
}

/// Frost protection method for heat recovery units.
/// Determines the supply temperature θ_t (ISSO 51 Table 2.14, erratum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrostProtectionType {
    /// Unknown type of frost protection
    Unknown,
    /// Reduced fan speed and/or temporary imbalance (central)
    CentralReducedSpeed,
    /// Enthalpy exchanger (central, min 70% thermal efficiency)
    CentralEnthalpy,
    /// Pre-heating (central)
    CentralPreheating,
    /// Reduced fan speed and/or temporary imbalance (decentral)
    DecentralReducedSpeed,
    /// Enthalpy exchanger (decentral, min 70% thermal efficiency)
    DecentralEnthalpy,
    /// Pre-heating (decentral)
    DecentralPreheating,
    /// Electric pre-heating without heat recovery
    ElectricPreheating,
}

/// Security class for heat loss to neighbors.
/// ISSO 51 Table 2.16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClass {
    /// Class A: no heat loss to neighbors assumed (c_z = 0)
    A,
    /// Class B: moderate risk (c_z = 0.5)
    B,
    /// Class C: high risk, neighbors may not heat (c_z = 1.0)
    C,
}

impl SecurityClass {
    /// Returns the security factor c_z.
    /// ISSO 51 Table 2.16.
    pub fn factor(&self) -> f64 {
        match self {
            SecurityClass::A => 0.0,
            SecurityClass::B => 0.5,
            SecurityClass::C => 1.0,
        }
    }
}

/// Material type for thermal bridge correction.
/// Determines whether ΔU_TB = 0.1 is applied per the forfaitaire method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    /// Masonry/stone-like materials (steenachtig)
    Masonry,
    /// Non-masonry materials like glass, doors (niet-steenachtig)
    NonMasonry,
}

/// Infiltration calculation method.
/// Determines how the specific infiltration rate is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InfiltrationMethod {
    /// Per m² exterior construction area (ISSO 51:2023 Table 4.3).
    /// q_i = qi_spec_ext × ΣA_exterior
    PerExteriorArea,
    /// Per m² floor area (ISSO 51:2024).
    /// q_i = qi_spec × A_floor (uses same qi_spec table values)
    PerFloorArea,
}

impl Default for InfiltrationMethod {
    fn default() -> Self {
        InfiltrationMethod::PerExteriorArea
    }
}

/// Building type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Detached house
    Detached,
    /// Semi-detached house
    SemiDetached,
    /// Terraced/row house (tussenwoning)
    Terraced,
    /// End-of-terrace (hoekwoning)
    EndOfTerrace,
    /// Apartment in a porch building (portiekwoning)
    Porch,
    /// Gallery apartment (galerijwoning)
    Gallery,
    /// Stacked housing (gestapeld)
    Stacked,
}

/// Position of a construction element relative to adjacent buildings.
/// Used for floor/ceiling fb calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerticalPosition {
    /// Floor (below the room)
    Floor,
    /// Ceiling (above the room)
    Ceiling,
    /// Vertical wall
    Wall,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_function_temperatures() {
        assert_eq!(RoomFunction::LivingRoom.design_temperature(), 20.0);
        assert_eq!(RoomFunction::Bathroom.design_temperature(), 22.0);
        assert_eq!(RoomFunction::Hallway.design_temperature(), 15.0);
        assert_eq!(RoomFunction::Toilet.design_temperature(), 15.0);
        assert_eq!(RoomFunction::Storage.design_temperature(), 5.0);
    }

    #[test]
    fn test_security_class_factors() {
        assert_eq!(SecurityClass::A.factor(), 0.0);
        assert_eq!(SecurityClass::B.factor(), 0.5);
        assert_eq!(SecurityClass::C.factor(), 1.0);
    }
}
