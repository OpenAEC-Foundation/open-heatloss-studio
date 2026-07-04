//! Fouttypes voor de nta8800-core façade.
//!
//! Volgt het isso51-core/isso53-core patroon: één `thiserror`-enum die de
//! domein-fouten van alle onderliggende reken-crates wrapt via `#[from]`,
//! plus façade-eigen validatie-fouten.

use thiserror::Error;

/// Alias voor `Result<T, CoreError>`.
pub type CoreResult<T> = Result<T, CoreError>;

/// Fouten uit de nta8800-core keten.
#[derive(Debug, Error)]
pub enum CoreError {
    /// JSON-parse of -serialisatie mislukt.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Ongeldige invoer op façade-niveau (pre-orchestratie validatie).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// NTA 8800 model-validatie faalde (oriëntatie/tilt buiten bereik etc.).
    #[error("nta8800-model error: {0}")]
    Model(#[from] nta8800_model::ModelError),

    /// Transmissie-keten (H.8) faalde.
    #[error("nta8800-transmission error: {0}")]
    Transmission(#[from] nta8800_transmission::errors::TransmissionError),

    /// Ventilatie-keten (H.11) faalde.
    #[error("nta8800-ventilation error: {0}")]
    Ventilation(#[from] nta8800_ventilation::VentilationError),

    /// Warmte-/koudebehoefte-keten (H.7) faalde.
    #[error("nta8800-demand error: {0}")]
    Demand(#[from] nta8800_demand::errors::DemandError),

    /// Verwarmings-keten (H.9) faalde.
    #[error("nta8800-heating error: {0}")]
    Heating(#[from] nta8800_heating::errors::HeatingError),

    /// Koelings-keten (H.10) faalde.
    #[error("nta8800-cooling error: {0}")]
    Cooling(#[from] nta8800_cooling::CoolingError),

    /// Warm-tapwater-keten (H.13) faalde.
    #[error("nta8800-dhw error: {0}")]
    Dhw(#[from] nta8800_dhw::errors::DhwError),

    /// Verlichtings-keten (H.14) faalde.
    #[error("nta8800-lighting error: {0}")]
    Lighting(#[from] nta8800_lighting::errors::LightingError),

    /// PV-keten (H.16) faalde.
    #[error("nta8800-pv error: {0}")]
    Pv(#[from] nta8800_pv::PvError),

    /// EP-score-keten (H.5) faalde.
    #[error("nta8800-ep error: {0}")]
    Ep(#[from] nta8800_ep::EpError),
}
