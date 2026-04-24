//! Error type voor NTA 8800 model-constructie en -validatie.
//!
//! Alle fouten die ontstaan bij het opbouwen van de model-objecten (via
//! constructors met validatie of bij cross-referenties tussen id's) worden
//! als [`ModelError`] uitgedrukt. Rekenfouten (numerieke convergentie,
//! ontbrekende tabelwaarden, etc.) horen in de rekencrates thuis — deze
//! crate heeft géén formule-logica.

use thiserror::Error;

/// Error type voor model-validatie en constructie van NTA 8800 data.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ModelError {
    /// Ongeldige invoerwaarde — context + reden.
    #[error("ongeldige invoer: {context} — {reason}")]
    InvalidInput {
        /// Veld- of contextbeschrijving waar de fout zich voordeed.
        context: String,
        /// Reden waarom de invoer ongeldig is.
        reason: String,
    },

    /// Verplichte referentie niet gevonden (bv. Rekenzone verwijst naar onbekende EFR).
    #[error("referentie niet gevonden: {kind} met id {id}")]
    ReferenceNotFound {
        /// Type van het gezochte object (bv. `"EnergiefunctieRuimte"`).
        kind: String,
        /// Id waarvan geen overeenkomstig object gevonden kon worden.
        id: String,
    },

    /// Waarde buiten toegestane bereik (bv. `tilt` buiten `0..=180`).
    #[error("waardebereik: {field} moet in {range}, gekregen {value}")]
    OutOfRange {
        /// Naam van het veld dat buiten bereik ligt.
        field: String,
        /// Beschrijving van het toegestane bereik (bv. `"0..=180"`).
        range: String,
        /// String-representatie van de gegeven waarde.
        value: String,
    },
}

/// Result-alias voor functies die een [`ModelError`] kunnen retourneren.
pub type ModelResult<T> = Result<T, ModelError>;
