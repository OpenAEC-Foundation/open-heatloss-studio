//! NTA 8800 hoofdstuk 17 — Klimaatgegevens.
//!
//! De klimaatdata van NTA 8800 zijn afkomstig uit NEN 5060 (referentie
//! De Bilt) en worden gegeven als:
//!
//! - Tabel 17.1 — maandgemiddelde buitentemperatuur, ventilatieve-koeling-
//!   temperatuur, windsnelheid en preheat-WTW-temperatuur per maand.
//! - Tabel 17.2 — maandgemiddelde totale opvallende zonnestraling `I_sol;mi`
//!   in W/m², per combinatie van hellingshoek β (0°, 30°, 45°, 60°, 90°,
//!   135°, 180°) en oriëntatie γ (N, NO, O, ZO, Z, ZW, W, NW).
//!
//! Voor D3 is alleen het referentieklimaat De Bilt geïmplementeerd. Regionale
//! varianten (KNMI-klimaatzones) kunnen later worden toegevoegd zonder
//! breaking change op de module-API.
//!
//! Zie [`NTA_8800_2025_PARAG17`](crate::references::NTA_8800_2025_PARAG17).

pub mod de_bilt;

pub use de_bilt::{
    de_bilt_climate_data, DE_BILT_COOLING_REFERENCE_TEMPERATURE, DE_BILT_MONTH_LENGTHS_HOURS,
    DE_BILT_OUTDOOR_TEMPERATURE, DE_BILT_SOLAR_IRRADIATION,
    DE_BILT_SOLAR_IRRADIATION_HORIZONTAL_W_PER_M2, DE_BILT_SOLAR_IRRADIATION_VERTICAL_W_PER_M2,
    DE_BILT_WIND_SPEED, DE_BILT_WTW_PREHEAT_TEMPERATURE,
};
