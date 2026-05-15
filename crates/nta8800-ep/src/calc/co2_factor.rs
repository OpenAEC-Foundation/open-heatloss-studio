//! CO2-beleidsfactoren volgens NTA 8800:2025 bijlage AB.
//!
//! Implementeert lookup-functie voor f_CO2 per energiedrager conform
//! [`NTA_8800_2025_TABEL_AB1`]. Waarden zijn gebaseerd op 2023 beleidskaders.

use crate::{EnergyCarrier, EpError};

/// Geeft de CO2-beleidsfactor f_CO2 voor een energiedrager in kg CO2/MJ.
///
/// Waarden conform NTA 8800:2025 tabel AB.1 (bijlage AB, 2023 update):
/// - Aardgas: 0.0506 kg CO2/MJ (verbrandingsemissie + upstream)
/// - Elektriciteit (net-mix): 0.0900 kg CO2/MJ (afnemend per jaar)
/// - Stadswarmte: 0.0270 kg CO2/MJ (gemiddeld Nederlandse warmtenetten)
/// - Biomassa: 0.0070 kg CO2/MJ (transport en bewerking, CO2-neutraal verbanding)
/// - Pellets: 0.0070 kg CO2/MJ (gestandaardiseerde biomassa)
/// - Hernieuwbare elektriciteit (PV): 0.0000 kg CO2/MJ (geen operationele emissies)
///
/// **SOURCE:** NTA 8800:2025 bijlage AB
///
/// # Errors
///
/// Retourneert [`EpError::MissingCo2Factor`] als de energiedrager
/// niet in bijlage AB is gedefinieerd (theoretisch onmogelijk met huidige enum).
///
/// # Voorbeeld
///
/// ```
/// # use nta8800_ep::{EnergyCarrier, calc::co2_factor::co2_factor};
/// let f_co2_gas = co2_factor(EnergyCarrier::Aardgas)?;
/// assert!((f_co2_gas - 0.0506).abs() < 1e-9);
///
/// let f_co2_pv = co2_factor(EnergyCarrier::HernieuwbareElektriciteit)?;
/// assert!(f_co2_pv.abs() < 1e-9);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn co2_factor(carrier: EnergyCarrier) -> Result<f64, EpError> {
    let factor = match carrier {
        // Fossiele energiedragers — bijlage AB sectie 1
        EnergyCarrier::Aardgas => 0.0506, // Verbrandingsemissie + upstream
        EnergyCarrier::Elektriciteit => 0.0900, // Nederlandse energiemix 2023, dalend

        // Collectieve systemen — bijlage AB sectie 2
        EnergyCarrier::Stadswarmte => 0.0270, // Gemiddeld Nederlandse warmtenetten

        // Hernieuwbare energiedragers — bijlage AB sectie 3
        EnergyCarrier::Biomassa | EnergyCarrier::Pellets => 0.0070, // Transport/bewerking
        EnergyCarrier::HernieuwbareElektriciteit => 0.0,            // Geen operationele emissies
    };

    Ok(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn co2_factors_match_bijlage_ab_2023() {
        assert_relative_eq!(
            co2_factor(EnergyCarrier::Aardgas).unwrap(),
            0.0506,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            co2_factor(EnergyCarrier::Elektriciteit).unwrap(),
            0.0900,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            co2_factor(EnergyCarrier::Stadswarmte).unwrap(),
            0.0270,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            co2_factor(EnergyCarrier::Biomassa).unwrap(),
            0.0070,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            co2_factor(EnergyCarrier::Pellets).unwrap(),
            0.0070,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            co2_factor(EnergyCarrier::HernieuwbareElektriciteit).unwrap(),
            0.0,
            epsilon = 1e-9
        );
    }

    #[test]
    fn all_energy_carriers_have_co2_factor() {
        for &carrier in EnergyCarrier::all() {
            assert!(
                co2_factor(carrier).is_ok(),
                "Energiedrager {carrier:?} heeft geen CO2-beleidsfactor"
            );
        }
    }

    #[test]
    fn co2_factors_are_non_negative() {
        for &carrier in EnergyCarrier::all() {
            let factor = co2_factor(carrier).unwrap();
            assert!(
                factor >= 0.0,
                "CO2-beleidsfactor voor {carrier:?} is negatief: {factor}"
            );
        }
    }

    #[test]
    fn renewable_carriers_have_low_co2() {
        for &carrier in EnergyCarrier::all() {
            if carrier.is_renewable() {
                let factor = co2_factor(carrier).unwrap();
                assert!(
                    factor <= 0.01, // Max 10 g CO2/MJ voor hernieuwbare bronnen
                    "Hernieuwbare energiedrager {carrier:?} heeft hoge CO2-factor: {factor}"
                );
            }
        }
    }
}
