//! Resultaat-struct voor één rekenzone-transmissieberekening.
//!
//! Alle energiewaarden zijn uitgedrukt in MJ, in overeenstemming met
//! [`nta8800_model::units::Energy`]. De norm zelf rekent in kWh (factor 0.001
//! in formule (7.14)); de conversie kWh → MJ gebeurt met factor 3.6 binnen
//! [`crate::calc::monthly_energy`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_model::time::MonthlyProfile;
use nta8800_model::units::Energy;

/// Opsplitsing van maandelijkse transmissiewarmte per boundary-type.
///
/// Elk profiel geeft 12 maandelijkse energiewaarden in MJ. De som van de vijf
/// profielen is gelijk aan [`TransmissionResult::monthly_q_t`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransmissionBreakdown {
    /// Transmissie direct naar buitenlucht (`H_D` — formule (8.1)).
    pub outdoor: MonthlyProfile<Energy>,

    /// Transmissie via onverwarmde ruimten (`H_U` — formule (8.52)).
    pub unheated_space: MonthlyProfile<Energy>,

    /// Transmissie via grond (`H_g;an` — §8.3, jaargemiddelde temperatuur).
    pub ground: MonthlyProfile<Energy>,

    /// Transmissie naar aangrenzende verwarmde zone (`H_A` — §8.5, standaard 0).
    pub adjacent_zone: MonthlyProfile<Energy>,

    /// Transmissie via lineaire + puntvormige thermische bruggen.
    pub thermal_bridges: MonthlyProfile<Energy>,
}

/// Volledig resultaat van de transmissie-berekening voor één rekenzone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransmissionResult {
    /// Totale maandelijkse transmissiewarmte Q_H;tr;zi;mi in MJ.
    pub monthly_q_t: MonthlyProfile<Energy>,

    /// Som over 12 maanden in MJ.
    pub annual_q_t: Energy,

    /// Uitsplitsing per boundary-type.
    pub breakdown: TransmissionBreakdown,

    /// Totale directe warmteverliescoëfficiënt `H_D` in W/K
    /// (formule (8.1) — enkel vlakken met [`crate::model::BoundaryType::Outdoor`]
    /// plus lineaire/punt-bruggen).
    pub h_d: f64,

    /// Totale warmteverliescoëfficiënt via onverwarmde ruimten `H_U` in W/K
    /// (formule (8.52)).
    pub h_u: f64,

    /// Jaarlijkse warmteoverdrachtcoëfficiënt `H_g;an` via grond in W/K
    /// (§8.3; in maandmethode gecombineerd met `θ_e;avg;an`).
    pub h_g_an: f64,

    /// Warmteoverdrachtcoëfficiënt `H_A` naar aangrenzende verwarmde zones in W/K.
    ///
    /// NTA 8800 verwaarloost `H_A` standaard; deze waarde is non-zero alleen
    /// wanneer de consumer maandprofielen voor aangrenzende zones heeft
    /// meegegeven (opt-in volgens formule (8.60)/(8.61)).
    pub h_a: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::Month;

    fn zero_profile() -> MonthlyProfile<Energy> {
        MonthlyProfile::from_constant(0.0)
    }

    #[test]
    fn result_serde_round_trip() {
        let r = TransmissionResult {
            monthly_q_t: zero_profile(),
            annual_q_t: 0.0,
            breakdown: TransmissionBreakdown {
                outdoor: zero_profile(),
                unheated_space: zero_profile(),
                ground: zero_profile(),
                adjacent_zone: zero_profile(),
                thermal_bridges: zero_profile(),
            },
            h_d: 0.0,
            h_u: 0.0,
            h_g_an: 0.0,
            h_a: 0.0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: TransmissionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn breakdown_all_twelve_months_addressable() {
        let bd = TransmissionBreakdown {
            outdoor: MonthlyProfile::new([
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ]),
            unheated_space: zero_profile(),
            ground: zero_profile(),
            adjacent_zone: zero_profile(),
            thermal_bridges: zero_profile(),
        };
        assert!((bd.outdoor[Month::Januari] - 1.0).abs() < 1e-12);
        assert!((bd.outdoor[Month::December] - 12.0).abs() < 1e-12);
    }
}
