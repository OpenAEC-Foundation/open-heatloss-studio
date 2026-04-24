//! Koelsysteem-types: compressie, absorptie en vrije koeling.
//!
//! Volgt NTA 8800 H.10 — een koelsysteem bestaat uit de combinatie
//! koude-opwekker + koudedistributiesysteem + koudeafgiftesysteem. Dit type
//! modelleert alleen de **opwek-kant** (koudeopwekker); distributie en
//! afgifte worden apart gemodelleerd via [`super::CoolingDistribution`] en
//! [`super::CoolingEmission`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Type koudeopwekker.
///
/// V1 dekt de drie meest-voorkomende types in woning- en utiliteitsbouw.
/// Opwekker-type-specifieke bijlage-correcties (analoog aan bijlage Q voor
/// verwarming) volgen in V2.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoolingSystem {
    /// Compressiekoeling met seizoensgemiddelde SCOP_cooling.
    ///
    /// Dominant in de Nederlandse markt (split-units, VRV, waterbron
    /// warmtepompen in koel-bedrijf). Typische waarde: 3,0–5,0.
    CompressionCooling {
        /// Seasonal Coefficient Of Performance voor koelbedrijf — verhouding
        /// tussen geleverde koude en benodigde elektriciteit op jaarbasis.
        scop_cooling: f64,
    },
    /// Absorptiekoeling — warmte-gedreven proces met COP_cooling.
    ///
    /// Toegepast in gebouwen met restwarmte (WKK, industriële processen).
    /// Typische waarde COP: 0,6–1,3. Energiedrager: gas of warmtenet.
    AbsorptionCooling {
        /// COP (niet seizoens-, want bij absorptie is het proces doorgaans
        /// vrij constant over het seizoen).
        cop: f64,
    },
    /// Vrije koeling — passief via ventilatie of bodem-warmtewisselaar.
    ///
    /// Benuttingsfractie 0..=1 geeft aan welk deel van de koudebehoefte zonder
    /// compressor kan worden gedekt. Typisch: 0,1–0,4 voor ventilatieve
    /// koeling in Nederlandse woningen, hoger bij actieve bodembron.
    FreeCooling {
        /// Benuttingsfractie 0..=1 (dimensieloos).
        factor: f64,
    },
}

impl CoolingSystem {
    /// Typische energiedrager voor dit koelsysteem.
    #[must_use]
    pub const fn energy_carrier(&self) -> EnergyCarrier {
        match self {
            CoolingSystem::CompressionCooling { .. } | CoolingSystem::FreeCooling { .. } => {
                EnergyCarrier::Electricity
            }
            CoolingSystem::AbsorptionCooling { .. } => EnergyCarrier::Gas,
        }
    }

    /// Numerieke efficiëntie die in de teller staat bij Q_C;use = Q_C;nd / eff.
    ///
    /// Voor compressie- en absorptiekoeling is dit de SCOP/COP zelf; voor
    /// vrije koeling geldt dat alleen `(1 − factor)` elektriciteit nodig is
    /// — dit is een benuttingsfractie en wordt in [`super::super::calc`]
    /// apart toegepast, niet als een klassieke COP. Voor algemene
    /// rapportage-doeleinden levert deze helper echter wél een `f64` op.
    #[must_use]
    pub const fn nominal_cop(&self) -> f64 {
        match self {
            CoolingSystem::CompressionCooling { scop_cooling } => *scop_cooling,
            CoolingSystem::AbsorptionCooling { cop } => *cop,
            // FreeCooling levert geen mechanische koude — "COP oneindig"
            // geeft numerieke problemen, we rapporteren 1.0 als "neutraal"
            // voor post-processing die deze waarde wil tonen.
            CoolingSystem::FreeCooling { .. } => 1.0,
        }
    }
}

/// Energiedrager voor een koelsysteem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCarrier {
    /// Elektriciteit — default voor compressiekoeling.
    Electricity,
    /// Gas — bij gas-gedreven absorptiekoeling.
    Gas,
    /// Stadskoude (district cold) — uit extern koudenet.
    DistrictCold,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_has_electricity_as_carrier() {
        let sys = CoolingSystem::CompressionCooling { scop_cooling: 4.0 };
        assert_eq!(sys.energy_carrier(), EnergyCarrier::Electricity);
        assert!((sys.nominal_cop() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn absorption_has_gas_as_carrier() {
        let sys = CoolingSystem::AbsorptionCooling { cop: 0.8 };
        assert_eq!(sys.energy_carrier(), EnergyCarrier::Gas);
    }

    #[test]
    fn free_cooling_has_electricity_as_carrier() {
        let sys = CoolingSystem::FreeCooling { factor: 0.3 };
        assert_eq!(sys.energy_carrier(), EnergyCarrier::Electricity);
    }

    #[test]
    fn cooling_system_serde_round_trip_compression() {
        let sys = CoolingSystem::CompressionCooling { scop_cooling: 4.2 };
        let json = serde_json::to_string(&sys).unwrap();
        let back: CoolingSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }

    #[test]
    fn cooling_system_serde_round_trip_absorption() {
        let sys = CoolingSystem::AbsorptionCooling { cop: 0.85 };
        let json = serde_json::to_string(&sys).unwrap();
        let back: CoolingSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }

    #[test]
    fn cooling_system_serde_round_trip_free() {
        let sys = CoolingSystem::FreeCooling { factor: 0.25 };
        let json = serde_json::to_string(&sys).unwrap();
        let back: CoolingSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }

    #[test]
    fn energy_carrier_serde_snake_case() {
        let json = serde_json::to_string(&EnergyCarrier::DistrictCold).unwrap();
        assert_eq!(json, "\"district_cold\"");
    }
}
