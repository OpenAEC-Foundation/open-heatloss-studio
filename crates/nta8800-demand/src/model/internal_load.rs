//! Interne warmtelast Φ_int — forfaitaire waarden uit NTA 8800 tabel 7.6.
//!
//! Deze module levert:
//! - [`InternalGains`] — container met maandprofiel `Φ_int` in W/m².
//! - [`InternalGains::forfaitair`] — default per [`UsageFunction`] uit
//!   NTA 8800:2025+C1:2026 tabel 7.6 (§7.10).
//!
//! ## Forfaitaire waarden per gebruiksfunctie
//!
//! | Gebruiksfunctie | Φ_int (W/m², jaarrond) |
//! |---|---:|
//! | Woonfunctie | 3,0 |
//! | Kantoorfunctie | 4,0 |
//! | Onderwijsfunctie | 4,0 |
//! | Bijeenkomstfunctie | 5,0 |
//! | Gezondheidszorgfunctie | 5,0 |
//! | Logiesfunctie | 3,0 |
//! | Winkelfunctie | 4,0 |
//! | Celfunctie | 3,5 |
//! | Sportfunctie | 3,5 |
//! | Industriefunctie | 3,5 |
//! | Overige gebruiksfunctie | 3,5 |
//!
//! Deze waarden zijn representatieve defaults voor V1; NTA 8800 tabel 7.6
//! specificeert per gebruiksfunctie gedetailleerde uur- en dag-profielen. De
//! maand-balans uit deze crate gebruikt alleen het jaarrond-gemiddelde — de
//! gedifferentieerde uurprofielen horen bij de uur-balans (V2).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::UsageFunction;

use crate::errors::{DemandCalcResult, DemandError};

/// Interne warmtelast-flux per m² vloeroppervlak, per maand.
///
/// Eenheid: W/m² (= J/(s·m²)). De conversie naar MJ/maand gebeurt in
/// [`crate::calc::internal_gains::monthly_internal_gains`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct InternalGains {
    /// Maandelijks profiel van de interne warmtelast `Φ_int` in W/m².
    pub heat_flux_per_m2: MonthlyProfile<f64>,
}

impl InternalGains {
    /// Construct met expliciet maandprofiel.
    ///
    /// # Errors
    /// - [`DemandError::InvalidInternalHeatFlux`] als een maand-waarde negatief
    ///   of niet-eindig is.
    pub fn new(profile: MonthlyProfile<f64>) -> DemandCalcResult<Self> {
        for (_, &v) in profile.iter() {
            if !v.is_finite() || v < 0.0 {
                return Err(DemandError::InvalidInternalHeatFlux { flux_w_per_m2: v });
            }
        }
        Ok(Self {
            heat_flux_per_m2: profile,
        })
    }

    /// Forfaitaire default-flux voor een gegeven gebruiksfunctie.
    ///
    /// Zie de module-doc voor de volledige tabel. Gebruikt NTA 8800 tabel 7.6
    /// jaarrond-gemiddelden (V1; gedifferentieerde uurprofielen in V2).
    #[must_use]
    pub fn forfaitair(usage: UsageFunction) -> Self {
        let flux = Self::default_flux_w_per_m2(usage);
        Self {
            heat_flux_per_m2: MonthlyProfile::from_constant(flux),
        }
    }

    /// Forfaitaire jaarrond-gemiddelde flux in W/m² per gebruiksfunctie.
    ///
    /// Bron: NTA 8800:2025+C1:2026 tabel 7.6 (zie module-doc).
    #[must_use]
    pub const fn default_flux_w_per_m2(usage: UsageFunction) -> f64 {
        match usage {
            UsageFunction::Woonfunctie | UsageFunction::Logiesfunctie => 3.0,
            UsageFunction::Kantoorfunctie
            | UsageFunction::Onderwijsfunctie
            | UsageFunction::Winkelfunctie => 4.0,
            UsageFunction::Bijeenkomstfunctie | UsageFunction::Gezondheidszorgfunctie => 5.0,
            UsageFunction::Celfunctie
            | UsageFunction::Sportfunctie
            | UsageFunction::Industriefunctie
            | UsageFunction::OverigeGebruiksfunctie => 3.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::Month;

    #[test]
    fn forfaitair_woonfunctie_is_3_w_per_m2() {
        let g = InternalGains::forfaitair(UsageFunction::Woonfunctie);
        assert!((g.heat_flux_per_m2[Month::Januari] - 3.0).abs() < 1e-9);
        assert!((g.heat_flux_per_m2[Month::Juli] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn forfaitair_alle_functies_positief() {
        for usage in [
            UsageFunction::Woonfunctie,
            UsageFunction::Bijeenkomstfunctie,
            UsageFunction::Celfunctie,
            UsageFunction::Gezondheidszorgfunctie,
            UsageFunction::Industriefunctie,
            UsageFunction::Kantoorfunctie,
            UsageFunction::Logiesfunctie,
            UsageFunction::Onderwijsfunctie,
            UsageFunction::Sportfunctie,
            UsageFunction::Winkelfunctie,
            UsageFunction::OverigeGebruiksfunctie,
        ] {
            let g = InternalGains::forfaitair(usage);
            for (_m, &v) in g.heat_flux_per_m2.iter() {
                assert!(v > 0.0, "{usage:?}: Φ_int ≤ 0");
                assert!(v < 10.0, "{usage:?}: Φ_int > 10 W/m² is onrealistisch");
            }
        }
    }

    #[test]
    fn gezondheidszorg_hoger_dan_woonfunctie() {
        let zorg = InternalGains::default_flux_w_per_m2(UsageFunction::Gezondheidszorgfunctie);
        let woon = InternalGains::default_flux_w_per_m2(UsageFunction::Woonfunctie);
        assert!(zorg > woon);
    }

    #[test]
    fn new_rejects_negative_flux() {
        let profile =
            MonthlyProfile::new([3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, -1.0]);
        let err = InternalGains::new(profile).unwrap_err();
        assert!(matches!(err, DemandError::InvalidInternalHeatFlux { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let g = InternalGains::forfaitair(UsageFunction::Kantoorfunctie);
        let json = serde_json::to_string(&g).unwrap();
        let back: InternalGains = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
