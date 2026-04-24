//! Opwekkingsrendement warm tapwater `η_W;gen`.
//!
//! NTA 8800:2025+C1:2026 §13.8 definieert het opwekkingsrendement onder
//! praktijkomstandigheden `η_W;gen;prac` — de verhouding tussen de aan het
//! tapwater nuttig afgegeven hoeveelheid warmte onder gebruiksomstandigheden
//! en de hoeveelheid energie die de opwekker afneemt van de energiedrager.
//!
//! Koppeling met Gaskeur (bijlage T) en boosterwarmtepomp-methodiek
//! (bijlage W) zijn V2 scope. V1 gebruikt forfaitaire waarden per type.
//!
//! ## V1 defaults
//!
//! | Generator | Energiedrager | η_W;gen default | Ratio |
//! |---|---|---|---|
//! | HR-combi-ketel | Gas | 0,80 | Lager dan CV: meer deellast, kleinere tappingen |
//! | Elektrische boiler | Electricity | 0,90 | Opslagverlies voorraadvat |
//! | Tapwater-warmtepomp | Electricity | SCOP_W (user) | Typisch 2,0-3,5 |
//! | Stadsverwarming | DistrictHeat | user factor ≤ 1,0 | Grensvlak-verlies |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{DhwCalcResult, DhwError};

/// Energiedrager-annotatie voor `Q_W;use`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCarrier {
    /// Aardgas (onderwaarde). HR-combi-ketel.
    Gas,
    /// Elektriciteit. Warmtepomp of elektrische boiler.
    Electricity,
    /// Externe warmtelevering (stadsverwarming / warmtenet).
    DistrictHeat,
}

/// Opwekkings-systeem voor warm tapwater (V1 scope: 4 types).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DhwGenerationSystem {
    /// HR-combi-ketel (gas, gemeenschappelijk met verwarming).
    ///
    /// V1 default η_W;gen = 0,80. Lager dan CV-rendement (HR-107 ≈ 0,95) omdat
    /// tapwaterbereiding per natura een hoger deellast-aandeel heeft en
    /// kleinere tappingen niet efficiënt modulerend uitgevoerd worden.
    HRCombiBoiler,

    /// Elektrische boiler (voorraadvat of doorstromer).
    ///
    /// V1 default η_W;gen = 0,90. Waakvlam-loze elektrische weerstand is 100%
    /// efficiënt, maar opslagverlies + stand-by corrigeert dit naar ~0,90
    /// voor een typisch 80-150 l voorraadvat.
    ElectricBoiler {
        /// Opslagverlies-correctie-factor (0 < f ≤ 1). V1 default 0,90.
        storage_loss_factor: f64,
    },

    /// Tapwater-warmtepomp met seizoensgemiddelde COP voor tapwater (SCOP_W).
    ///
    /// SCOP_W voor tapwater is typisch 2,0-3,5 — lager dan CV-warmtepomp
    /// omdat tapwater op 60-65 °C moet (legionella) terwijl CV op 35-55 °C
    /// kan. Bij bron-temperatuur 7 °C → levertemperatuur 60 °C: COP ≈ 2,5.
    HeatPumpDhw {
        /// Seizoensgemiddelde COP voor tapwater (> 0). Bijlage Q/W-detail
        /// is V2 scope; V1 vertrouwt op user-supplied meet- of KV-waarde.
        scop_dhw: f64,
    },

    /// Stadsverwarming / warmtenet voor tapwater.
    ///
    /// Factor modelleert het verlies tussen gebouwgrens en tapwater-afgifte.
    /// De primaire-energie-factor van het warmtenet zelf wordt in de nEP-
    /// berekening (H.5) meegewogen.
    DistrictHeating {
        /// Forfaitaire factor (0 < factor ≤ 1).
        factor: f64,
    },
}

impl DhwGenerationSystem {
    /// Opwekkings-rendement η_W;gen (dimensieloos).
    ///
    /// Voor [`DhwGenerationSystem::HeatPumpDhw`] is dit > 1 mogelijk (SCOP).
    /// Voor alle andere varianten in (0, 1].
    ///
    /// # Errors
    ///
    /// - [`DhwError::InvalidScop`] als SCOP ≤ 0 of niet-eindig
    /// - [`DhwError::InvalidDistrictHeatingFactor`] als factor buiten (0, 1]
    /// - [`DhwError::InvalidEfficiency`] als storage_loss_factor ongeldig
    pub fn efficiency(&self) -> DhwCalcResult<f64> {
        match self {
            DhwGenerationSystem::HRCombiBoiler => Ok(0.80),
            DhwGenerationSystem::ElectricBoiler {
                storage_loss_factor,
            } => {
                if storage_loss_factor.is_finite()
                    && *storage_loss_factor > 0.0
                    && *storage_loss_factor <= 1.0
                {
                    Ok(*storage_loss_factor)
                } else {
                    Err(DhwError::InvalidEfficiency {
                        name: "η_W;gen (electric boiler)",
                        value: *storage_loss_factor,
                        upper: 1.0,
                    })
                }
            }
            DhwGenerationSystem::HeatPumpDhw { scop_dhw } => {
                if scop_dhw.is_finite() && *scop_dhw > 0.0 {
                    Ok(*scop_dhw)
                } else {
                    Err(DhwError::InvalidScop { scop: *scop_dhw })
                }
            }
            DhwGenerationSystem::DistrictHeating { factor } => {
                if factor.is_finite() && *factor > 0.0 && *factor <= 1.0 {
                    Ok(*factor)
                } else {
                    Err(DhwError::InvalidDistrictHeatingFactor { factor: *factor })
                }
            }
        }
    }

    /// Energiedrager voor `Q_W;use`.
    #[must_use]
    pub const fn energy_carrier(&self) -> EnergyCarrier {
        match self {
            DhwGenerationSystem::HRCombiBoiler => EnergyCarrier::Gas,
            DhwGenerationSystem::ElectricBoiler { .. }
            | DhwGenerationSystem::HeatPumpDhw { .. } => EnergyCarrier::Electricity,
            DhwGenerationSystem::DistrictHeating { .. } => EnergyCarrier::DistrictHeat,
        }
    }

    /// Constructor-helper: elektrische boiler met default opslagverlies-factor 0,90.
    #[must_use]
    pub const fn electric_boiler_default() -> Self {
        DhwGenerationSystem::ElectricBoiler {
            storage_loss_factor: 0.90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn hr_combi_is_80_percent() {
        let eta = DhwGenerationSystem::HRCombiBoiler.efficiency().unwrap();
        assert_relative_eq!(eta, 0.80, max_relative = 1e-12);
    }

    #[test]
    fn electric_boiler_default_is_90_percent() {
        let eta = DhwGenerationSystem::electric_boiler_default()
            .efficiency()
            .unwrap();
        assert_relative_eq!(eta, 0.90, max_relative = 1e-12);
    }

    #[test]
    fn heat_pump_scop_passes_through() {
        let eta = DhwGenerationSystem::HeatPumpDhw { scop_dhw: 2.8 }
            .efficiency()
            .unwrap();
        assert_relative_eq!(eta, 2.8, max_relative = 1e-12);
    }

    #[test]
    fn heat_pump_rejects_zero() {
        let err = DhwGenerationSystem::HeatPumpDhw { scop_dhw: 0.0 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(err, DhwError::InvalidScop { .. }));
    }

    #[test]
    fn heat_pump_rejects_negative() {
        let err = DhwGenerationSystem::HeatPumpDhw { scop_dhw: -1.5 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(err, DhwError::InvalidScop { .. }));
    }

    #[test]
    fn district_heating_valid() {
        let eta = DhwGenerationSystem::DistrictHeating { factor: 0.92 }
            .efficiency()
            .unwrap();
        assert_relative_eq!(eta, 0.92, max_relative = 1e-12);
    }

    #[test]
    fn district_heating_rejects_above_one() {
        let err = DhwGenerationSystem::DistrictHeating { factor: 1.2 }
            .efficiency()
            .unwrap_err();
        assert!(matches!(err, DhwError::InvalidDistrictHeatingFactor { .. }));
    }

    #[test]
    fn electric_boiler_rejects_invalid_loss_factor() {
        let err = DhwGenerationSystem::ElectricBoiler {
            storage_loss_factor: 1.1,
        }
        .efficiency()
        .unwrap_err();
        assert!(matches!(err, DhwError::InvalidEfficiency { .. }));
    }

    #[test]
    fn energy_carrier_mapping() {
        assert_eq!(
            DhwGenerationSystem::HRCombiBoiler.energy_carrier(),
            EnergyCarrier::Gas
        );
        assert_eq!(
            DhwGenerationSystem::electric_boiler_default().energy_carrier(),
            EnergyCarrier::Electricity
        );
        assert_eq!(
            DhwGenerationSystem::HeatPumpDhw { scop_dhw: 3.0 }.energy_carrier(),
            EnergyCarrier::Electricity
        );
        assert_eq!(
            DhwGenerationSystem::DistrictHeating { factor: 0.9 }.energy_carrier(),
            EnergyCarrier::DistrictHeat
        );
    }

    #[test]
    fn serde_round_trip_heat_pump() {
        let s = DhwGenerationSystem::HeatPumpDhw { scop_dhw: 2.5 };
        let json = serde_json::to_string(&s).unwrap();
        let back: DhwGenerationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_round_trip_unit_like_hr_combi() {
        let s = DhwGenerationSystem::HRCombiBoiler;
        let json = serde_json::to_string(&s).unwrap();
        let back: DhwGenerationSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
