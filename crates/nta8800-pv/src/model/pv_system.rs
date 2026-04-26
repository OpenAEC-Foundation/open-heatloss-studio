//! PV-systeem specificatie en validatie.

use crate::errors::PvError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Specificatie van een fotovoltaïsch systeem.
///
/// Definieert de technische eigenschappen van een PV-installatie die nodig
/// zijn voor de NTA 8800 H.16 berekening van maandelijkse opbrengst.
///
/// Alle hoeken worden opgegeven in graden decimaal. Efficiënties als
/// dimensieloze getallen tussen 0 en 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PvSystem {
    /// Piek-vermogen van het PV-systeem in kWp.
    ///
    /// Het nominale vermogen onder STC-condities (Standard Test Conditions:
    /// 1000 W/m², 25°C, AM 1.5). Moet groter zijn dan 0.
    pub peak_power_kwp: f64,

    /// Hellingshoek β van de PV-modules in graden.
    ///
    /// 0° = horizontaal, 90° = verticaal. Voor dakintegratie typisch
    /// 30-45°. Moet tussen 0° en 90° liggen.
    pub tilt_degrees: f64,

    /// Azimuth-hoek γ van de PV-modules in graden.
    ///
    /// 0° = noord, 90° = oost, 180° = zuid, 270° = west.
    /// Voor optimale opbrengst in Nederland ~180° (zuid-oriëntatie).
    /// Moet tussen -180° en +180° liggen.
    pub azimuth_degrees: f64,

    /// Totale systeem-efficiëntie (dimensieloos, 0-1).
    ///
    /// Combineert alle DC-verliezen: module-mismatch, bekabeling,
    /// vervuiling, temperatuur-effecten. Typisch 0.80-0.90 voor
    /// moderne systemen.
    pub system_efficiency: f64,

    /// Inverter-efficiëntie (dimensieloos, 0-1).
    ///
    /// Omzetting DC → AC. Moderne string-omvormers ~0.96-0.98.
    /// Micro-omvormers iets lager ~0.94-0.97.
    pub inverter_efficiency: f64,

    /// Optionele schaduw-factor (dimensieloos, 0-1).
    ///
    /// Forfaitaire correctie voor partiële beschaduwing door bomen,
    /// gebouwen, etc. 1.0 = geen schaduw, 0.8 = 20% schaduw-verlies.
    /// V1: handmatig opgegeven, V2: automatisch uit geometrie.
    #[serde(default = "default_shadow_factor")]
    pub shadow_factor: f64,
}

/// Default schaduw-factor (geen schaduw).
fn default_shadow_factor() -> f64 {
    1.0
}

impl PvSystem {
    /// Creëert een nieuw PV-systeem met validatie.
    ///
    /// # Argumenten
    ///
    /// * `peak_power_kwp` - Piek-vermogen in kWp (moet > 0)
    /// * `tilt_degrees` - Hellingshoek in graden (0° - 90°)
    /// * `azimuth_degrees` - Azimuth in graden (-180° - +180°)
    /// * `system_efficiency` - Systeem-efficiëntie (0 < η ≤ 1)
    /// * `inverter_efficiency` - Inverter-efficiëntie (0 < η ≤ 1)
    ///
    /// # Errors
    ///
    /// Retourneert [`PvError`] als een van de parameters buiten het
    /// geldige bereik ligt.
    ///
    /// # Example
    ///
    /// ```
    /// use nta8800_pv::PvSystem;
    ///
    /// let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96)?;
    /// assert_eq!(system.peak_power_kwp, 5.5);
    /// assert_eq!(system.tilt_degrees, 35.0);
    /// assert_eq!(system.shadow_factor, 1.0); // default
    /// # Ok::<(), nta8800_pv::PvError>(())
    /// ```
    pub fn new(
        peak_power_kwp: f64,
        tilt_degrees: f64,
        azimuth_degrees: f64,
        system_efficiency: f64,
        inverter_efficiency: f64,
    ) -> Result<Self, PvError> {
        Self::validate_peak_power(peak_power_kwp)?;
        Self::validate_tilt(tilt_degrees)?;
        Self::validate_azimuth(azimuth_degrees)?;
        Self::validate_system_efficiency(system_efficiency)?;
        Self::validate_inverter_efficiency(inverter_efficiency)?;

        Ok(Self {
            peak_power_kwp,
            tilt_degrees,
            azimuth_degrees,
            system_efficiency,
            inverter_efficiency,
            shadow_factor: default_shadow_factor(),
        })
    }

    /// Creëert een PV-systeem met optionele schaduw-factor.
    ///
    /// Zoals [`Self::new`], maar met expliciete schaduw-factor.
    ///
    /// # Example
    ///
    /// ```
    /// use nta8800_pv::PvSystem;
    ///
    /// // Systeem met 15% schaduw-verlies
    /// let system = PvSystem::with_shadow(5.5, 35.0, 180.0, 0.85, 0.96, 0.85)?;
    /// assert_eq!(system.shadow_factor, 0.85);
    /// # Ok::<(), nta8800_pv::PvError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Retourneert [`PvError`] als een van de parameters of de schaduw-factor buiten het geldige bereik ligt.
    pub fn with_shadow(
        peak_power_kwp: f64,
        tilt_degrees: f64,
        azimuth_degrees: f64,
        system_efficiency: f64,
        inverter_efficiency: f64,
        shadow_factor: f64,
    ) -> Result<Self, PvError> {
        let mut system = Self::new(
            peak_power_kwp,
            tilt_degrees,
            azimuth_degrees,
            system_efficiency,
            inverter_efficiency,
        )?;

        if !(0.0..=1.0).contains(&shadow_factor) {
            return Err(PvError::InvalidSystemEfficiency(shadow_factor));
        }

        system.shadow_factor = shadow_factor;
        Ok(system)
    }

    /// Berekent de totale DC→AC efficiëntie.
    ///
    /// Combineert systeem-efficiëntie × inverter-efficiëntie × schaduw-factor
    /// voor gebruik in de PV-opbrengst formule.
    #[must_use]
    pub fn total_efficiency(&self) -> f64 {
        self.system_efficiency * self.inverter_efficiency * self.shadow_factor
    }

    fn validate_peak_power(peak_power_kwp: f64) -> Result<(), PvError> {
        if peak_power_kwp <= 0.0 {
            Err(PvError::InvalidPeakPower(peak_power_kwp))
        } else {
            Ok(())
        }
    }

    fn validate_tilt(tilt_degrees: f64) -> Result<(), PvError> {
        if (0.0..=90.0).contains(&tilt_degrees) {
            Ok(())
        } else {
            Err(PvError::InvalidTilt(tilt_degrees))
        }
    }

    fn validate_azimuth(azimuth_degrees: f64) -> Result<(), PvError> {
        if (-180.0..=180.0).contains(&azimuth_degrees) {
            Ok(())
        } else {
            Err(PvError::InvalidAzimuth(azimuth_degrees))
        }
    }

    fn validate_system_efficiency(system_efficiency: f64) -> Result<(), PvError> {
        if 0.0 < system_efficiency && system_efficiency <= 1.0 {
            Ok(())
        } else {
            Err(PvError::InvalidSystemEfficiency(system_efficiency))
        }
    }

    fn validate_inverter_efficiency(inverter_efficiency: f64) -> Result<(), PvError> {
        if 0.0 < inverter_efficiency && inverter_efficiency <= 1.0 {
            Ok(())
        } else {
            Err(PvError::InvalidInverterEfficiency(inverter_efficiency))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_valid_parameters_succeeds() {
        let system = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).unwrap();
        assert_eq!(system.peak_power_kwp, 5.5);
        assert_eq!(system.tilt_degrees, 35.0);
        assert_eq!(system.azimuth_degrees, 180.0);
        assert_eq!(system.system_efficiency, 0.85);
        assert_eq!(system.inverter_efficiency, 0.96);
        assert_eq!(system.shadow_factor, 1.0);
    }

    #[test]
    fn with_shadow_sets_shadow_factor() {
        let system = PvSystem::with_shadow(5.5, 35.0, 180.0, 0.85, 0.96, 0.85).unwrap();
        assert_eq!(system.shadow_factor, 0.85);
    }

    #[test]
    fn total_efficiency_calculation() {
        let system = PvSystem::with_shadow(5.5, 35.0, 180.0, 0.85, 0.96, 0.9).unwrap();
        let expected = 0.85 * 0.96 * 0.9;
        assert!((system.total_efficiency() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_peak_power_returns_error() {
        let result = PvSystem::new(0.0, 35.0, 180.0, 0.85, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidPeakPower(0.0));
    }

    #[test]
    fn invalid_tilt_returns_error() {
        let result = PvSystem::new(5.5, -1.0, 180.0, 0.85, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidTilt(-1.0));

        let result = PvSystem::new(5.5, 91.0, 180.0, 0.85, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidTilt(91.0));
    }

    #[test]
    fn invalid_azimuth_returns_error() {
        let result = PvSystem::new(5.5, 35.0, -181.0, 0.85, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidAzimuth(-181.0));

        let result = PvSystem::new(5.5, 35.0, 181.0, 0.85, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidAzimuth(181.0));
    }

    #[test]
    fn invalid_system_efficiency_returns_error() {
        let result = PvSystem::new(5.5, 35.0, 180.0, 0.0, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidSystemEfficiency(0.0));

        let result = PvSystem::new(5.5, 35.0, 180.0, 1.1, 0.96);
        assert_eq!(result.unwrap_err(), PvError::InvalidSystemEfficiency(1.1));
    }

    #[test]
    fn invalid_inverter_efficiency_returns_error() {
        let result = PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.0);
        assert_eq!(result.unwrap_err(), PvError::InvalidInverterEfficiency(0.0));

        let result = PvSystem::new(5.5, 35.0, 180.0, 0.85, 1.1);
        assert_eq!(result.unwrap_err(), PvError::InvalidInverterEfficiency(1.1));
    }

    #[test]
    fn edge_case_tilt_angles() {
        // 0° (horizontaal) en 90° (verticaal) moeten geldig zijn
        assert!(PvSystem::new(5.5, 0.0, 180.0, 0.85, 0.96).is_ok());
        assert!(PvSystem::new(5.5, 90.0, 180.0, 0.85, 0.96).is_ok());
    }

    #[test]
    fn edge_case_azimuth_angles() {
        // -180° en +180° moeten geldig zijn
        assert!(PvSystem::new(5.5, 35.0, -180.0, 0.85, 0.96).is_ok());
        assert!(PvSystem::new(5.5, 35.0, 180.0, 0.85, 0.96).is_ok());
    }

    #[test]
    fn edge_case_efficiencies() {
        // 100% efficiëntie moet geldig zijn, 0% niet
        assert!(PvSystem::new(5.5, 35.0, 180.0, 1.0, 1.0).is_ok());
        assert!(PvSystem::new(5.5, 35.0, 180.0, 0.01, 0.01).is_ok());
    }
}
