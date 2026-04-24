//! Afgifte-systeem met forfaitair η_em per type.
//!
//! NTA 8800 §9.3 drukt afgifte-verliezen in werkelijkheid uit via
//! temperatuur-correcties ΔT (tabel 9.2) — deze corrigeren de werkelijke
//! aanvoertemperatuur voor stratificatie, embedded, radiatie en imperfecte
//! regeling. Voor V1 reduceren we dit tot een enkel rendement per type;
//! het volledige ΔT-model is V2.
//!
//! De hier gegeven defaults zijn **V1-engineeringswaarden**, geen directe
//! kopie uit de norm — ze zijn afgeleid uit Tabel 9.2 (ΔT-correcties)
//! door te interpreteren hoeveel van de Q_H;nd effectief in de ruimte
//! terechtkomt bij een 20 °C setpoint. Zie rustdoc per variant.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{HeatingCalcResult, HeatingError};

/// Afgifte-systeem — bepaalt η_em (afgifte-rendement, 0 < η ≤ 1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmissionSystem {
    /// Radiator hoge temperatuur (typisch 70-90 °C aanvoer, HT).
    ///
    /// Relatief hoge stratificatie en stralingscomponent. V1 default η_em = 0,95.
    RadiatorHighTemp,

    /// Radiator lage temperatuur (typisch 55 °C aanvoer).
    ///
    /// Lagere stratificatie dan HT. V1 default η_em = 0,95.
    RadiatorLowTemp,

    /// Vloerverwarming (typisch 35 °C aanvoer, embedded).
    ///
    /// Grote oppervlakte, lage ΔT, gelijkmatige temperatuurverdeling. Tabel 9.2
    /// geeft Δθ = 0,3 K (iets beter dan radiator 0,35 K). V1 default η_em = 0,96.
    FloorHeating,

    /// Luchtverwarming (forced convection via kanaalsysteem).
    ///
    /// Tabel 9.2 geeft Δθ = 0,0 K (referentie), maar praktische verliezen in
    /// kanalen en stratificatie drukken η_em omlaag. V1 default η_em = 0,88.
    AirHeating,

    /// Stralingspanelen (plafond- of wandverwarming).
    ///
    /// Stralingscomponent levert comfort bij iets lagere lucht-temperatuur,
    /// maar opstart-verliezen zijn groter. V1 default η_em = 0,92.
    RadiantPanel,

    /// User-supplied custom η_em (0 < η ≤ 1).
    ///
    /// Gebruik deze variant als er een specifiek afgifte-systeem is waarvoor
    /// een kwaliteitsverklaring of betere engineering-waarde beschikbaar is.
    Custom {
        /// Η_em waarde in (0, 1].
        efficiency: f64,
    },
}

impl EmissionSystem {
    /// Constructor-helper voor de `Custom`-variant.
    #[must_use]
    pub const fn custom(efficiency: f64) -> Self {
        EmissionSystem::Custom { efficiency }
    }

    /// Default forfaitair afgifte-rendement η_em per type.
    ///
    /// Geef een getal in het interval (0, 1]. V1-waarden zijn engineering-
    /// benaderingen en worden in V2 vervangen door het volledige ΔT-model
    /// van §9.3.
    #[must_use]
    pub fn default_efficiency(&self) -> f64 {
        match self {
            EmissionSystem::RadiatorHighTemp | EmissionSystem::RadiatorLowTemp => 0.95,
            EmissionSystem::FloorHeating => 0.96,
            EmissionSystem::AirHeating => 0.88,
            EmissionSystem::RadiantPanel => 0.92,
            EmissionSystem::Custom { efficiency } => *efficiency,
        }
    }

    /// Valideert dat η_em in (0, 1] valt en eindig is.
    ///
    /// # Errors
    ///
    /// [`HeatingError::InvalidEfficiency`] als `default_efficiency()` niet
    /// in (0, 1] valt of niet-eindig is.
    pub fn validated_efficiency(&self) -> HeatingCalcResult<f64> {
        let eta = self.default_efficiency();
        if eta.is_finite() && eta > 0.0 && eta <= 1.0 {
            Ok(eta)
        } else {
            Err(HeatingError::InvalidEfficiency {
                name: "η_em",
                value: eta,
                upper: 1.0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_in_range() {
        for s in [
            EmissionSystem::RadiatorHighTemp,
            EmissionSystem::RadiatorLowTemp,
            EmissionSystem::FloorHeating,
            EmissionSystem::AirHeating,
            EmissionSystem::RadiantPanel,
        ] {
            let eta = s.default_efficiency();
            assert!(
                eta > 0.0 && eta <= 1.0,
                "η_em voor {s:?} buiten (0,1]: {eta}"
            );
        }
    }

    #[test]
    fn floor_heating_default_matches_v1() {
        assert!((EmissionSystem::FloorHeating.default_efficiency() - 0.96).abs() < 1e-12);
    }

    #[test]
    fn floor_heating_beats_radiator() {
        assert!(
            EmissionSystem::FloorHeating.default_efficiency()
                > EmissionSystem::RadiatorHighTemp.default_efficiency()
        );
    }

    #[test]
    fn air_heating_is_lowest() {
        let air = EmissionSystem::AirHeating.default_efficiency();
        let rad = EmissionSystem::RadiatorHighTemp.default_efficiency();
        assert!(air < rad);
    }

    #[test]
    fn custom_passes_through() {
        assert!((EmissionSystem::custom(0.87).default_efficiency() - 0.87).abs() < 1e-12);
    }

    #[test]
    fn validated_rejects_zero() {
        let err = EmissionSystem::custom(0.0)
            .validated_efficiency()
            .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidEfficiency { .. }));
    }

    #[test]
    fn validated_rejects_above_one() {
        let err = EmissionSystem::custom(1.1)
            .validated_efficiency()
            .unwrap_err();
        assert!(matches!(err, HeatingError::InvalidEfficiency { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let s = EmissionSystem::custom(0.93);
        let json = serde_json::to_string(&s).unwrap();
        let back: EmissionSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn serde_round_trip_unit_variant() {
        let s = EmissionSystem::FloorHeating;
        let json = serde_json::to_string(&s).unwrap();
        let back: EmissionSystem = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
