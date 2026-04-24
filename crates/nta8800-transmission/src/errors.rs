//! Error-types voor de transmissie-crate.
//!
//! Bouwt voort op [`nta8800_model::ModelError`] voor invoer-validatie. Deze
//! enum voegt transmissie-specifieke fouten toe (ontbrekende b-factor lookup,
//! ongeldige constructie-referentie, etc.).

use thiserror::Error;

use nta8800_model::ModelError;

/// Error-type voor transmissie-berekeningen.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum TransmissionError {
    /// Fout tijdens validatie van de invoer (model-laag).
    #[error(transparent)]
    Model(#[from] ModelError),

    /// Verplichte b-factor ontbreekt in de lookup-map voor een
    /// [`BoundaryType::UnheatedSpace`](crate::model::BoundaryType::UnheatedSpace).
    #[error("ontbrekende b-factor voor onverwarmde ruimte met id {id:?}")]
    MissingUnheatedBFactor {
        /// Id van de onverwarmde ruimte waarvoor geen b-factor is opgegeven.
        id: String,
    },

    /// Verplichte temperatuur-profile ontbreekt voor een
    /// [`BoundaryType::AdjacentZone`](crate::model::BoundaryType::AdjacentZone).
    #[error("ontbrekend maandprofiel voor aangrenzende zone met id {id:?}")]
    MissingAdjacentZoneTemperature {
        /// Id van de aangrenzende zone waarvoor geen profile is opgegeven.
        id: String,
    },

    /// Constructie heeft een R_tot van 0 of niet-eindig; U-waarde zou
    /// oneindig worden. Detecteer vóór gebruik.
    #[error(
        "constructie {construction_id:?} heeft R_tot ≤ 0 of niet-eindig (R = {r_total}) — U zou oneindig zijn"
    )]
    InvalidRTotal {
        /// Id van de constructie met ongeldig R_tot.
        construction_id: String,
        /// Berekende R_tot-waarde.
        r_total: f64,
    },

    /// Negatieve of niet-eindige oppervlakte op een [`TransmissionElement`](crate::model::TransmissionElement).
    #[error("element {element_id:?} heeft ongeldige oppervlakte {area} (moet > 0 en eindig)")]
    InvalidArea {
        /// Id of label van het element.
        element_id: String,
        /// De opgegeven oppervlakte.
        area: f64,
    },

    /// b-factor uit opgegeven lookup ligt buiten [0, 1].
    #[error("b-factor voor onverwarmde ruimte {id:?} buiten bereik: {value} (moet 0..=1)")]
    BFactorOutOfRange {
        /// Id van de onverwarmde ruimte.
        id: String,
        /// De opgegeven waarde.
        value: f64,
    },
}

/// Result-alias voor transmissie-berekeningen.
pub type TransmissionResult<T> = Result<T, TransmissionError>;
