//! TO-juli (NTA 8800 bijlage AA) simplified cooling handlers.
//!
//! V1 endpoint: levert vereenvoudigde koelbehoefte- en minimum-koelcapaciteit
//! resultaten op basis van twee compacte input-structs. De Rekenzone / EFR /
//! Window / Climate parameters van de onderliggende `calculate_simplified_cooling`
//! zijn in V1 unused — V2 (project-integratie) maakt ze actief.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use nta8800_cooling::{
    calculate_simplified_cooling, SimplifiedAreaInput, SimplifiedCoolingResult,
    SimplifiedLoadInput,
};
use nta8800_tables::climate::de_bilt_climate_data;
use openaec_project_shared::{
    compute_tojuli_full, ProjectV2, TojuliFullInputs, TojuliResult,
};
use serde::{Deserialize, Serialize};

/// Request body voor POST /cooling/simplified.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimplifiedCoolingRequest {
    /// Σ(A_vg;woon;zi) — totale oppervlakte verblijfsruimten in gebruik als
    /// woonkamer/keuken/eetkamer, in m².
    pub living_area_m2: f64,
    /// Σ(A_vg;overig;zi) — totale oppervlakte overige verblijfsruimten, in m².
    pub other_area_m2: f64,
    /// N_woon;zi — aantal woonfuncties in de rekenzone.
    pub dwelling_count: u32,
    /// P_p;woon;zi — gemiddeld aantal bewoners per woonfunctie.
    pub persons_per_dwelling: f64,
    /// Infiltratie-luchtvolumestroom q_v;C;eff;lea in juli, in m³/h.
    pub infiltration_m3_per_h: f64,
    /// Natuurlijke ventilatie-toevoer q_v;C;eff;vent in juli, in m³/h.
    pub natural_ventilation_m3_per_h: f64,
    /// Mechanische toevoer q_v;C;SUP;eff in juli, in m³/h.
    pub mechanical_supply_m3_per_h: f64,
    /// Tijdstip van maximale koellast (9..21 h), drijft θ_e uit tabel AA.1.
    pub peak_hour: u8,
    /// Bouwjaar voor f_iso uit tabel AA.2.
    pub construction_year: u32,
    /// Binnenwerkse oppervlakte ondoorzichtig buitenwand + dak, in m².
    pub opaque_area_m2: f64,
    /// P_sol;zi — zoninstraling via transparante delen (AA.6), in W.
    pub solar_load_w: f64,
    /// P_gl;zi — transmissie via transparante delen (AA.7), in W.
    pub glazing_transmission_w: f64,
}

/// POST /cooling/simplified — vereenvoudigde koelbehoefte (TO-juli) per zone.
///
/// Roept `nta8800_cooling::calculate_simplified_cooling` aan met de twee
/// gebruikte input-structs en placeholders voor de V2-parameters. Output is
/// per-zone piekkoellast en minimum-koelcapaciteit.
pub async fn simplified_cooling(
    Json(req): Json<SimplifiedCoolingRequest>,
) -> impl IntoResponse {
    let area = SimplifiedAreaInput {
        living_area_m2: req.living_area_m2,
        other_area_m2: req.other_area_m2,
        dwelling_count: req.dwelling_count,
        persons_per_dwelling: req.persons_per_dwelling,
    };
    let load = SimplifiedLoadInput {
        infiltration_m3_per_h: req.infiltration_m3_per_h,
        natural_ventilation_m3_per_h: req.natural_ventilation_m3_per_h,
        mechanical_supply_m3_per_h: req.mechanical_supply_m3_per_h,
        peak_hour: req.peak_hour,
        construction_year: req.construction_year,
        opaque_area_m2: req.opaque_area_m2,
        solar_load_w: req.solar_load_w,
        glazing_transmission_w: req.glazing_transmission_w,
    };
    // NEN 5060 referentieklimaat De Bilt (NTA 8800 tabel 17.1 + 17.2).
    // Voor de bijlage AA quick-check is alleen θ_e in juli relevant — die
    // wordt uit tabel AA.1 (per peak_hour) gehaald, niet uit ClimateData.
    // Maar het zit nu in plaats van een handmatige stub, klaar voor F4+ waar
    // het maandprofiel wel actief wordt gebruikt.
    let climate = de_bilt_climate_data();

    let result = tokio::task::spawn_blocking(move || {
        calculate_simplified_cooling(&[], &[], &climate, &[], &area, &load)
    })
    .await;

    match result {
        Ok(Ok(r)) => Json::<SimplifiedCoolingResult>(r).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "cooling_calc_error",
                "detail": e.to_string()
            })),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "detail": join_err.to_string()
            })),
        )
            .into_response(),
    }
}

/// Request body voor POST /tojuli/calculate — volledige NTA 8800 H.10 keten.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TojuliCalculateRequest {
    /// Drielagig project (shared + geometry + calcs.tojuli optioneel).
    pub project: ProjectV2,
    /// TO-juli specifieke inputs: cooling system, distribution, emission, etc.
    pub inputs: TojuliFullInputs,
}

/// POST /tojuli/calculate — volledige TO-juli H.10 keten (woning + utiliteit).
///
/// Roept `openaec_project_shared::compute_tojuli_full` aan op blocking thread.
/// Levert maandelijkse Q_C;use + jaarsom + intermediates.
pub async fn tojuli_calculate(
    Json(req): Json<TojuliCalculateRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        compute_tojuli_full(&req.project, &req.inputs)
    })
    .await;

    match result {
        Ok(Ok(r)) => Json::<TojuliResult>(r).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "tojuli_calc_error",
                "detail": e.to_string()
            })),
        )
            .into_response(),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "detail": join_err.to_string()
            })),
        )
            .into_response(),
    }
}

