//! Opwekkings-systeem met forfaitair of user-supplied rendement.
//!
//! Vier types in V1:
//!
//! | Variant | Energiedrager | η_gen bron |
//! |---|---|---|
//! | HR-ketel | Gas | NTA 8800 pg 327, tabel individueel cv-toestel, HT-kolom |
//! | Warmtepomp | Electricity | User-supplied SCOP (seizoensgemiddelde COP) |
//! | Elektrische weerstand | Electricity | Vaste 1,0 (100 % conversie) |
//! | Stadsverwarming | DistrictHeat | Forfaitaire factor ≤ 1,0 |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{HeatingCalcResult, HeatingError};

/// Energiedrager-annotatie voor Q_H;use — bepaalt hoe de downstream
/// nEP-berekening de energie moet wegen (primaire energie factor,
/// CO₂-factor, tarief, enzovoort).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCarrier {
    /// Aardgas (onderwaarde). Toegepast bij HR-ketel.
    Gas,
    /// Elektriciteit. Toegepast bij warmtepomp en elektrische weerstand.
    Electricity,
    /// Externe warmtelevering (stadsverwarming / warmtenet).
    DistrictHeat,
}

/// HR-ketel classificatie (NTA 8800 H.9).
///
/// Onderscheid op basis van deellastrendement op onderwaarde:
///
/// | Klasse | Deellast (%) |
/// |---|---|
/// | HR-100 | ≥ 100 |
/// | HR-104 | ≥ 104 |
/// | HR-107 | ≥ 107 |
///
/// Forfaitaire η_gen-waarden in [`HRClass::default_efficiency`] zijn afkomstig
/// uit de tabel op pg 327 van NTA 8800:2025+C1:2026, **HT-kolom** (aanvoer
/// 70-90 °C), "individueel cv-toestel binnen thermische begrenzing,
/// hoofdverwarming".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HRClass {
    /// HR-100 — deellastrendement ≥ 100 % op onderwaarde.
    HR100,
    /// HR-104 — deellastrendement ≥ 104 % op onderwaarde.
    HR104,
    /// HR-107 — deellastrendement ≥ 107 % op onderwaarde.
    HR107,
}

impl HRClass {
    /// Forfaitair opwekkingsrendement η_gen per HR-klasse, HT-kolom.
    ///
    /// Bron: NTA 8800:2025+C1:2026 pg 327, "Individueel cv-toestel (water)
    /// exclusief waakvlam, geplaatst binnen de thermische begrenzing van het
    /// gebouw, hoofdverwarming", kolom HT (aanvoer 70-90 °C).
    ///
    /// - HR-100: 0,90
    /// - HR-104: 0,925
    /// - HR-107: 0,95
    #[must_use]
    pub fn default_efficiency(&self) -> f64 {
        match self {
            HRClass::HR100 => 0.90,
            HRClass::HR104 => 0.925,
            HRClass::HR107 => 0.95,
        }
    }
}

/// Opwekkings-systeem voor verwarming (V1 scope: 4 types).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerationSystem {
    /// HR-ketel met classificatie conform NTA 8800 pg 327.
    HRBoiler {
        /// HR-100/104/107 klasse.
        class: HRClass,
    },
    /// Warmtepomp met seizoensgemiddelde COP (SCOP).
    ///
    /// SCOP > 1 is typisch (warmtepomp levert meer warmte dan elektrische
    /// input). Q_H;use (elektrisch) = Q_H;nd / SCOP.
    HeatPump {
        /// Seizoensgemiddelde COP (warmteafgifte ÷ elektrische input).
        scop: f64,
    },
    /// Elektrische weerstandsverwarming — η_gen = 1,0 (100 % conversie).
    ///
    /// Iedere J elektra = 1 J warmte. De `Q_H;use` is dus gelijk aan de
    /// warmtebehoefte gedeeld door η_em × η_dist × f_reg.
    ElectricResistance,
    /// Stadsverwarming met forfaitair factor (≤ 1,0).
    ///
    /// Factor reflecteert het lokale verlies tussen gebouwgrens en
    /// warmtedistributie. In NTA 8800 wordt de keten-verlies van de
    /// stadswarmte-bron zelf via primaire energie factor (H.13) meegewogen —
    /// de factor hier dekt alleen het verlies tussen grensvlak en afgiftezijde.
    DistrictHeating {
        /// Forfaitaire factor (0 < factor ≤ 1).
        factor: f64,
    },
}

impl GenerationSystem {
    /// Opwekkings-rendement η_gen (dimensieloos).
    ///
    /// Voor [`GenerationSystem::HeatPump`] is dit > 1 mogelijk (SCOP).
    /// Voor alle andere varianten in (0, 1].
    ///
    /// # Errors
    ///
    /// - [`HeatingError::InvalidScop`] als SCOP ≤ 0 of niet-eindig
    /// - [`HeatingError::InvalidDistrictHeatingFactor`] als factor ≤ 0 of niet-eindig of > 1
    /// - [`HeatingError::InvalidEfficiency`] bij corrupte HR-ketel waarden
    ///   (onmogelijk via publieke API, maar volledigheidshalve gecheckt)
    pub fn efficiency(&self) -> HeatingCalcResult<f64> {
        match self {
            GenerationSystem::HRBoiler { class } => {
                let eta = class.default_efficiency();
                if eta.is_finite() && eta > 0.0 && eta <= 1.0 {
                    Ok(eta)
                } else {
                    Err(HeatingError::InvalidEfficiency {
                        name: "η_gen (HR)",
                        value: eta,
                        upper: 1.0,
                    })
                }
            }
            GenerationSystem::HeatPump { scop } => {
                if scop.is_finite() && *scop > 0.0 {
                    Ok(*scop)
                } else {
                    Err(HeatingError::InvalidScop { scop: *scop })
                }
            }
            GenerationSystem::ElectricResistance => Ok(1.0),
            GenerationSystem::DistrictHeating { factor } => {
                if factor.is_finite() && *factor > 0.0 && *factor <= 1.0 {
                    Ok(*factor)
                } else {
                    Err(HeatingError::InvalidDistrictHeatingFactor { factor: *factor })
                }
            }
        }
    }

    /// Energiedrager voor de `Q_H;use` output.
    #[must_use]
    pub const fn energy_carrier(&self) -> EnergyCarrier {
        match self {
            GenerationSystem::HRBoiler { .. } => EnergyCarrier::Gas,
            GenerationSystem::HeatPump { .. } | GenerationSystem::ElectricResistance => {
                EnergyCarrier::Electricity
            }
            GenerationSystem::DistrictHeating { .. } => EnergyCarrier::DistrictHeat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hr_class_defaults_exact() {
        assert!((HRClass::HR100.default_efficiency() - 0.90).abs() < 1e-12);
        assert!((HRClass::HR104.default_efficiency() - 0.925).abs() < 1e-12);
        assert!((HRClass::HR107.default_efficiency() - 0.95).abs() < 1e-12);
    }

    #[test]
    fn hr_class_monotone() {
        assert!(HRClass::HR100.default_efficiency() < HRClass::HR104.default_efficiency());
        assert!(HRClass::HR104.default_efficiency() < HRClass::HR107.default_efficiency());
    }

    #[test]
    fn electric_resistance_is_one() {
        let eta = GenerationSystem::ElectricResistance.efficiency().unwrap();
        assert!((eta - 1.0).abs() < 1e-12);
    }

    #[test]
    fn heat_pump_scop_passes_through() {
        let eta = GenerationSystem::HeatPump { scop: 4.2 }
            .efficiency()
            .unwrap();
        assert!((eta - 4.2).abs() < 1e-12);
    }

    #[test]
    fn heat_pump_rejects_zero_scop() {
        let err = GenerationSystem::HeatPump { scop: 0.0 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidScop { .. }));
    }

    #[test]
    fn heat_pump_rejects_negative_scop() {
        let err = GenerationSystem::HeatPump { scop: -1.0 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidScop { .. }));
    }

    #[test]
    fn district_heating_factor_valid() {
        let eta = GenerationSystem::DistrictHeating { factor: 0.90 }
            .efficiency()
            .unwrap();
        assert!((eta - 0.90).abs() < 1e-12);
    }

    #[test]
    fn district_heating_rejects_above_one() {
        let err = GenerationSystem::DistrictHeating { factor: 1.1 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(
            err,
            HeatingError::InvalidDistrictHeatingFactor { .. }
        ));
    }

    #[test]
    fn energy_carrier_mapping() {
        assert_eq!(
            GenerationSystem::HRBoiler {
                class: HRClass::HR107
            }
            .energy_carrier(),
            EnergyCarrier::Gas
        );
        assert_eq!(
            GenerationSystem::HeatPump { scop: 4.0 }.energy_carrier(),
            EnergyCarrier::Electricity
        );
        assert_eq!(
            GenerationSystem::ElectricResistance.energy_carrier(),
            EnergyCarrier::Electricity
        );
        assert_eq!(
            GenerationSystem::DistrictHeating { factor: 0.9 }.energy_carrier(),
            EnergyCarrier::DistrictHeat
        );
    }

    #[test]
    fn serde_round_trip_hr_boiler() {
        let s = GenerationSystem::HRBoiler {
            class: HRClass::HR107,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: GenerationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_round_trip_heat_pump() {
        let s = GenerationSystem::HeatPump { scop: 3.8 };
        let json = serde_json::to_string(&s).unwrap();
        let back: GenerationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
