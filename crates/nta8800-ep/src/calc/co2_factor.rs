//! CO2-emissiecoëfficiënten volgens NTA 8800:2025+C1:2026 tabel 5.3 (§5.5.6.1).
//!
//! Implementeert lookup-functie voor `KCO2` per energiedrager conform
//! [`NTA_8800_2025_TABEL_AB1`] (historische anchor-naam; de getalswaarden staan in
//! 2025+C1 in tabel 5.3, p. 96 — niet in bijlage AB, dat nu de informatieve
//! ZEB-indicator bevat). De hier gehanteerde absolute waarden zijn 2023-kaders in
//! kg CO2/MJ; tabel 5.3 geeft ze in kg CO2eq/kWh (bv. elektriciteit 0,268/kWh ≈
//! 0,0744/MJ). De absolute actualisatie is out-of-scope; de *structuur* — met name
//! de PV-verrekening — volgt tabel 5.3.

use crate::{EnergyCarrier, EpError};

/// Geeft de CO2-emissiecoëfficiënt `KCO2` voor een energiedrager in kg CO2/MJ.
///
/// Waarden conform NTA 8800:2025+C1:2026 tabel 5.3 (§5.5.6.1, 2023-kaders):
/// - Aardgas: 0.0506 kg CO2/MJ (verbrandingsemissie + upstream)
/// - Elektriciteit (net-mix): 0.0900 kg CO2/MJ (afnemend per jaar)
/// - Stadswarmte: 0.0270 kg CO2/MJ (gemiddeld Nederlandse warmtenetten)
/// - Biomassa: 0.0070 kg CO2/MJ (transport en bewerking, CO2-neutraal verbanding)
/// - Pellets: 0.0070 kg CO2/MJ (gestandaardiseerde biomassa)
/// - Hernieuwbare elektriciteit (PV): **0.0900** kg CO2/MJ — op eigen perceel
///   opgewekte PV die zelf-gebruikt of geëxporteerd wordt, vermijdt net-emissies
///   tegen `KCO2;pr;us;el = KCO2;exp;el = KCO2;del;el` (tabel 5.3 zet alle drie de
///   elektriciteitskolommen gelijk, 0,268/kWh). PV verlaagt dus de CO2-indicator,
///   analoog aan de primaire-energie-saldering.
///
/// **SOURCE:** NTA 8800:2025+C1:2026 tabel 5.3 (§5.5.6.1, p. 96)
///
/// # Errors
///
/// Retourneert [`EpError::MissingCo2Factor`] als de energiedrager niet in
/// tabel 5.3 is gedefinieerd (theoretisch onmogelijk met huidige enum).
///
/// # Voorbeeld
///
/// ```
/// # use nta8800_ep::{EnergyCarrier, calc::co2_factor::co2_factor};
/// let f_co2_gas = co2_factor(EnergyCarrier::Aardgas)?;
/// assert!((f_co2_gas - 0.0506).abs() < 1e-9);
///
/// // PV vermijdt net-CO2 tegen de elektriciteitsfactor (tabel 5.3).
/// let f_co2_pv = co2_factor(EnergyCarrier::HernieuwbareElektriciteit)?;
/// assert!((f_co2_pv - 0.0900).abs() < 1e-9);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(clippy::match_same_arms)] // Elektriciteit (afname) vs. PV (vermeden) — beide 0,0900, semantisch verschillend
pub fn co2_factor(carrier: EnergyCarrier) -> Result<f64, EpError> {
    let factor = match carrier {
        // Fossiele energiedragers — tabel 5.3 rij 1-2
        EnergyCarrier::Aardgas => 0.0506, // Verbrandingsemissie + upstream
        EnergyCarrier::Elektriciteit => 0.0900, // Nederlandse energiemix 2023, dalend

        // Collectieve systemen — tabel 5.3
        EnergyCarrier::Stadswarmte => 0.0270, // Gemiddeld Nederlandse warmtenetten

        // Hernieuwbare energiedragers — tabel 5.3
        EnergyCarrier::Biomassa | EnergyCarrier::Pellets => 0.0070, // Transport/bewerking
        // PV vermeden net-emissie: KCO2;exp;el = KCO2;del;el (tabel 5.3, alle
        // elektriciteitskolommen gelijk).
        EnergyCarrier::HernieuwbareElektriciteit => 0.0900,
    };

    Ok(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn co2_factors_match_tabel_5_3() {
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
        // PV vermijdt net-CO2 tegen KCO2;exp;el = KCO2;del;el (tabel 5.3, p. 96).
        assert_relative_eq!(
            co2_factor(EnergyCarrier::HernieuwbareElektriciteit).unwrap(),
            0.0900,
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
    fn renewable_fuels_have_low_co2() {
        // Hernieuwbare *brandstoffen* (biomassa/pellets) hebben een lage
        // verbrandings-CO2. PV (HernieuwbareElektriciteit) valt hier bewust
        // buiten: zijn "factor" is de vermeden NET-emissie (= elektriciteits-
        // factor, tabel 5.3), geen brandstof-emissie.
        for &carrier in EnergyCarrier::all() {
            if carrier.is_renewable() && carrier != EnergyCarrier::HernieuwbareElektriciteit {
                let factor = co2_factor(carrier).unwrap();
                assert!(
                    factor <= 0.01, // Max 10 g CO2/MJ voor hernieuwbare brandstoffen
                    "Hernieuwbare brandstof {carrier:?} heeft hoge CO2-factor: {factor}"
                );
            }
        }
    }
}
