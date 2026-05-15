//! Error types voor EP-score berekeningen.

use thiserror::Error;

use crate::EnergyCarrier;

/// Fouten die kunnen optreden tijdens EP-score berekeningen.
#[derive(Debug, Error)]
pub enum EpError {
    /// Primaire energiefactor voor energiedrager niet beschikbaar in bijlage Z.
    #[error("Primaire energiefactor voor energiedrager {carrier:?} niet gevonden in bijlage Z")]
    MissingPrimaryEnergyFactor {
        /// De energiedrager waarvoor geen factor beschikbaar is.
        carrier: EnergyCarrier,
    },

    /// CO2-beleidsfactor voor energiedrager niet beschikbaar in bijlage AB.
    #[error("CO2-beleidsfactor voor energiedrager {carrier:?} niet gevonden in bijlage AB")]
    MissingCo2Factor {
        /// De energiedrager waarvoor geen CO2-factor beschikbaar is.
        carrier: EnergyCarrier,
    },

    /// Gebruiksoppervlakte A_g is nul of negatief.
    #[error("Gebruiksoppervlakte A_g moet positief zijn, gevonden: {a_g} m²")]
    InvalidBuildingArea {
        /// De ongeldige oppervlakte waarde.
        a_g: f64,
    },

    /// Energiegebruik bevat negatieve waarden (fysiek onmogelijk).
    #[error("Negatief energiegebruik voor {service}: {energy_mj} MJ")]
    NegativeEnergyUse {
        /// De dienst met negatief energiegebruik.
        service: String,
        /// De negatieve energie waarde.
        energy_mj: f64,
    },

    /// Energiebalans validatie gefaald (bv. PV-yield > totaal gebruik).
    #[error("Energiebalans validatiefout: {message}")]
    EnergyBalanceError {
        /// Beschrijving van de balansfout.
        message: String,
    },

    /// EP-label kan niet bepaald worden (buiten bekende drempels).
    #[error("EP-score {ep_score_mj_per_m2} MJ/m² valt buiten bekende label-drempels")]
    UnknownEpLabel {
        /// De EP-score die niet geclassificeerd kan worden.
        ep_score_mj_per_m2: f64,
    },
}
