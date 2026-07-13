//! Typed errors voor de `.uniec3`-import (F8 fase 4e).
//!
//! De importer is **tolerant** waar het kan (onbekende veldcodes/entiteiten →
//! `warnings`-lijst op het resultaat, niet falen) en **hard** waar de invoer
//! buiten de bewezen V1-scope valt (utiliteitsbouw, multi-unit) of het archief
//! corrupt is. Zo blijft een nieuwere app-versie met extra velden importeren,
//! terwijl een appartementen-export een nette, specifieke fout geeft in plaats
//! van een stille eerste-unit-keuze.

use thiserror::Error;

/// Foutcondities bij het importeren van een `.uniec3`-archief.
#[derive(Error, Debug)]
pub enum Uniec3ImportError {
    /// Het ZIP-archief kon niet gelezen worden (corrupt / geen PKZIP).
    #[error("ZIP-fout: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// IO-fout tijdens het uitlezen van een archief-entry.
    #[error("IO-fout: {0}")]
    Io(#[from] std::io::Error),

    /// Een verwacht bestand ontbreekt in het archief.
    #[error("ontbrekend bestand in archief: {0}")]
    MissingFile(String),

    /// Het archief bevat meer entries dan de veiligheidslimiet toestaat
    /// (zip-bomb-bescherming).
    #[error("archief bevat te veel entries ({count} > limiet {limit})")]
    TooManyEntries {
        /// Aantal entries in het archief.
        count: usize,
        /// De toegestane bovengrens.
        limit: usize,
    },

    /// Een archief-entry is (uitgepakt) groter dan de veiligheidslimiet
    /// (zip-bomb-/oversized-entry-bescherming).
    #[error("archief-entry {file} is te groot (uitgepakt {size} bytes > limiet {limit})")]
    EntryTooLarge {
        /// Het archief-pad van de te grote entry.
        file: String,
        /// De (gedeclareerde of gelezen) uitgepakte grootte in bytes.
        size: u64,
        /// De toegestane bovengrens in bytes.
        limit: u64,
    },

    /// Een archief-entry bevatte geen geldige JSON.
    #[error("JSON-fout in {file}: {source}")]
    Json {
        /// Het archief-pad van het bestand dat niet parste.
        file: String,
        /// De onderliggende serde-fout.
        source: serde_json::Error,
    },

    /// `buildings.json` bevatte geen enkel gebouw.
    #[error("geen gebouw in archief (buildings.json is leeg)")]
    NoBuilding,

    /// Meerdere rekenzones/units — buiten de V1-scope (woningbouw, single-unit).
    /// Zie TODO.md F8-V2 (appartementen/multi-zone).
    #[error("meerdere units/rekenzones niet ondersteund in V1 (single-unit woningbouw): {0}")]
    MultiUnitUnsupported(String),

    /// Utiliteitsbouw — buiten de V1-scope (onbeproefd, geen sample-export).
    /// Zie TODO.md F8-V2 (utiliteit).
    #[error("utiliteitsbouw niet ondersteund in V1 (gebouwtype {0})")]
    UtilityUnsupported(String),

    /// Geen bruikbare rekenzone-geometrie gevonden (geen UNIT/UNIT-RZ/BEGR).
    #[error("geen rekenzone-geometrie gevonden: {0}")]
    MissingGeometry(String),

    /// De opgebouwde [`openaec_project_shared::BengGeometry`] faalde de
    /// referentie-/plausibiliteitsvalidatie.
    #[error("geometrie-validatie faalde: {0}")]
    GeometryValidation(String),
}

/// Result-alias voor de import-API.
pub type Result<T> = std::result::Result<T, Uniec3ImportError>;
