//! Primaire energiefactoren volgens NTA 8800:2025 bijlage Z.
//!
//! Implementeert lookup-functie voor f_prim per energiedrager conform
//! [`NTA_8800_2025_TABEL_Z1`]. Waarden zijn gebaseerd op 2023 Nederlandse energiemix.

use crate::{EnergyCarrier, EpError};

/// Geeft de primaire energiefactor f_prim voor een energiedrager.
///
/// Waarden conform NTA 8800:2025 tabel Z.1 (bijlage Z, 2023 update):
/// - Aardgas: 1.000 (minimale omzettingsverliezen)
/// - Elektriciteit (net-mix): 1.450 (gemiddelde Nederlandse energiemix 2023)
/// - Stadswarmte: 0.000 (default tijdens hervorming warmtewet)
/// - Biomassa: 0.500 (transport en bewerking)
/// - Pellets: 0.500 (gestandaardiseerde biomassa)
/// - Hernieuwbare elektriciteit (PV): 0.000 (ter plaatse opwekking)
///
/// **SOURCE:** NTA 8800:2025 bijlage Z
///
/// # Errors
///
/// Retourneert [`EpError::MissingPrimaryEnergyFactor`] als de energiedrager
/// niet in bijlage Z is gedefinieerd (theoretisch onmogelijk met huidige enum).
///
/// # Warnings
///
/// - **Stadswarmte factor 0.000:** Dit is een beleidsmatige waarde tijdens
///   de overgangsperiode warmtewet 2023-2026. In praktijk kan dit per leverancier
///   variëren van 0.3-1.2 afhankelijk van de bron (restwarmte vs biomassa vs gas).
/// - **Elektriciteit factor 1.450:** Deze waarde daalt geleidelijk door
///   toenemend aandeel hernieuwbare opwekking in de Nederlandse energiemix.
///
/// # Voorbeeld
///
/// ```
/// # use nta8800_ep::{EnergyCarrier, calc::primary_energy::primary_factor};
/// let f_prim_gas = primary_factor(EnergyCarrier::Aardgas)?;
/// assert!((f_prim_gas - 1.000).abs() < 1e-9);
///
/// let f_prim_pv = primary_factor(EnergyCarrier::HernieuwbareElektriciteit)?;
/// assert!(f_prim_pv.abs() < 1e-9);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(clippy::match_same_arms)] // Beleidsmatige nul (Stadswarmte) vs. ter-plaatse-nul (PV) zijn semantisch verschillend
pub fn primary_factor(carrier: EnergyCarrier) -> Result<f64, EpError> {
    let factor = match carrier {
        // Fossiele energiedragers — bijlage Z sectie 1
        EnergyCarrier::Aardgas => 1.000,
        EnergyCarrier::Elektriciteit => 1.450, // Nederlandse energiemix 2023

        // Collectieve systemen — bijlage Z sectie 2
        // Collectieve systemen — bijlage Z sectie 2
        EnergyCarrier::Stadswarmte => 0.000, // Beleidsmatige waarde warmtewet 2023-2026
        // TODO Wave 3: per-leverancier variatie (0.3-1.2)

        // Hernieuwbare energiedragers — bijlage Z sectie 3
        EnergyCarrier::Biomassa | EnergyCarrier::Pellets => 0.500, // Transport/bewerking
        EnergyCarrier::HernieuwbareElektriciteit => 0.000,         // Ter plaatse opwekking
    };

    Ok(factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn primary_factors_match_bijlage_z_2023() {
        // Fossiele energiedragers
        assert_relative_eq!(
            primary_factor(EnergyCarrier::Aardgas).unwrap(),
            1.000,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            primary_factor(EnergyCarrier::Elektriciteit).unwrap(),
            1.450,
            epsilon = 1e-9
        );

        // Collectieve systemen
        assert_relative_eq!(
            primary_factor(EnergyCarrier::Stadswarmte).unwrap(),
            0.000,
            epsilon = 1e-9
        );

        // Hernieuwbare energiedragers
        assert_relative_eq!(
            primary_factor(EnergyCarrier::Biomassa).unwrap(),
            0.500,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            primary_factor(EnergyCarrier::Pellets).unwrap(),
            0.500,
            epsilon = 1e-9
        );
        assert_relative_eq!(
            primary_factor(EnergyCarrier::HernieuwbareElektriciteit).unwrap(),
            0.000,
            epsilon = 1e-9
        );
    }

    #[test]
    fn all_energy_carriers_have_primary_factor() {
        for &carrier in EnergyCarrier::all() {
            assert!(
                primary_factor(carrier).is_ok(),
                "Energiedrager {carrier:?} heeft geen primaire energiefactor"
            );
        }
    }

    #[test]
    fn primary_factors_are_non_negative() {
        for &carrier in EnergyCarrier::all() {
            let factor = primary_factor(carrier).unwrap();
            assert!(
                factor >= 0.0,
                "Primaire energiefactor voor {carrier:?} is negatief: {factor}"
            );
        }
    }
}
