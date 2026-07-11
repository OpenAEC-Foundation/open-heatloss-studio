//! Primaire energiefactoren volgens NTA 8800:2025+C1:2026 tabel 5.2 (§5.5.5).
//!
//! Implementeert lookup-functie voor f_prim per energiedrager conform
//! [`NTA_8800_2025_TABEL_Z1`] (historische anchor-naam; de getalswaarden staan in
//! 2025+C1 in tabel 5.2, p. 93 — niet in bijlage Z, dat nu de beleidsfactoren-index
//! bevat). Waarden gebaseerd op de Nederlandse energiemix.

use crate::{EnergyCarrier, EpError};

/// Geeft de primaire energiefactor f_prim voor een energiedrager.
///
/// Waarden conform NTA 8800:2025+C1:2026 tabel 5.2 (§5.5.5, p. 93-94):
/// - Aardgas: 1.000 (minimale omzettingsverliezen)
/// - Elektriciteit (net-mix): 1.450 (gemiddelde Nederlandse energiemix)
/// - Stadswarmte: 0.000 (default tijdens hervorming warmtewet)
/// - Biomassa: 0.500 (transport en bewerking)
/// - Pellets: 0.500 (gestandaardiseerde biomassa)
/// - Hernieuwbare elektriciteit (PV): **1.450** — de op eigen perceel opgewekte
///   PV-elektriciteit vermijdt primair-fossiele energie tegen `fP;pr;us;el` (zelf-
///   gebruik) respectievelijk `fP;exp;el` (export), beide 1,45 volgens tabel 5.2.
///   Omdat afname, zelfgebruik én export dezelfde factor 1,45 hebben, valt de
///   zelfconsumptie/export-splitsing (§5.5.4, formules 5.22-5.26) weg voor het
///   totaal: `EPTot = Σ(afgenomen × fP;del) − PV × 1,45`. Zie
///   `docs/2026-07-11-f3a-norm-analyse-ep.md` §2.
///
/// **SOURCE:** NTA 8800:2025+C1:2026 tabel 5.2 (p. 93-94)
///
/// Elektriciteit (afname) en `HernieuwbareElektriciteit` (PV, vermeden) delen de
/// waarde 1,45 maar zijn semantisch verschillend (afgenomen net-energie vs.
/// vermeden eigen productie) — vandaar de gescheiden match-armen.
///
/// # Errors
///
/// Retourneert [`EpError::MissingPrimaryEnergyFactor`] als de energiedrager
/// niet in tabel 5.2 is gedefinieerd (theoretisch onmogelijk met huidige enum).
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
/// assert!((f_prim_pv - 1.450).abs() < 1e-9);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(clippy::match_same_arms)] // Elektriciteit (afname) vs. PV (vermeden) — beide 1,45, semantisch verschillend
pub fn primary_factor(carrier: EnergyCarrier) -> Result<f64, EpError> {
    let factor = match carrier {
        // Fossiele energiedragers — tabel 5.2 rij 1-2
        EnergyCarrier::Aardgas => 1.000,
        EnergyCarrier::Elektriciteit => 1.450, // Nederlandse energiemix

        // Collectieve systemen — tabel 5.2
        EnergyCarrier::Stadswarmte => 0.000, // Beleidsmatige waarde warmtewet
        // TODO Wave 3: per-leverancier variatie (0.3-1.2)

        // Hernieuwbare energiedragers — tabel 5.2 / tabel 5.4
        EnergyCarrier::Biomassa | EnergyCarrier::Pellets => 0.500, // Transport/bewerking
        // PV: vermeden primair-fossiel bij zelfgebruik/export = fP;pr;us;el = fP;exp;el = 1,45.
        EnergyCarrier::HernieuwbareElektriciteit => 1.450,
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
        // PV: fP;pr;us;el = fP;exp;el = 1,45 (tabel 5.2, p. 93-94).
        assert_relative_eq!(
            primary_factor(EnergyCarrier::HernieuwbareElektriciteit).unwrap(),
            1.450,
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
