//! BENG (NTA 8800 energieprestatie) berekening-handler.
//!
//! Publieke reken-route analoog aan `/tojuli/calculate`: draait de volledige
//! BENG 1/2/3 + TOjuli + label-keten (`openaec_project_shared::compute_beng`) op
//! een `ProjectV2`. In tegenstelling tot TO-juli kent BENG géén los `inputs`-blok
//! — alle installatie-invoer leeft in `ProjectV2::energy`. De request-envelope
//! houdt dezelfde `project`-wrapper aan voor frontend-pariteit en toekomstige
//! uitbreiding.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use openaec_project_shared::{compute_beng, BengError, BengResult, ProjectV2};
use serde::{Deserialize, Serialize};

/// Request body voor POST /beng/calculate.
///
/// Alleen een `project`-veld: de energie-/installatie-invoer die BENG nodig
/// heeft zit in `project.energy` (`Option<EnergyInput>`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BengCalculateRequest {
    /// Drielagig project (shared + geometry + energy).
    pub project: ProjectV2,
}

/// POST /beng/calculate — volledige BENG-keten op een ProjectV2.
///
/// Roept `openaec_project_shared::compute_beng` aan op een blocking thread.
/// Levert BENG 1/2/3 + limieten + pass/fail + TOjuli + label + service-breakdown.
///
/// Foutmapping:
/// - `MissingEnergyInput` / `EmptyProject` → 422 (client mist verplichte invoer),
/// - overige reken-keten-fouten → 400 (conform `/tojuli/calculate`),
/// - blocking-task join failure → 500.
pub async fn beng_calculate(Json(req): Json<BengCalculateRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || compute_beng(&req.project)).await;

    match result {
        Ok(Ok(r)) => Json::<BengResult>(r).into_response(),
        Ok(Err(e)) => {
            let status = match e {
                BengError::MissingEnergyInput | BengError::EmptyProject => {
                    StatusCode::UNPROCESSABLE_ENTITY
                }
                _ => StatusCode::BAD_REQUEST,
            };
            (
                status,
                Json(serde_json::json!({
                    "error": "beng_calc_error",
                    "detail": e.to_string()
                })),
            )
                .into_response()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use openaec_project_shared::energy::{
        CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput, EnergyInput,
        HeatEmissionType, HeatGeneratorType, HeatingInput, VentilationInput, VentilationSystemType,
    };
    use openaec_project_shared::geometry::{BoundaryKind, ConstructionKind};
    use openaec_project_shared::{
        BuildingTypeShared, Construction, Opening, OpeningKind, ResidentialType, SharedGeometry,
        Space,
    };

    /// Opake gevel met één raam. Spiegelt de bewezen `synthetic_rijtjeshuis`-
    /// fixture uit `openaec-project-shared::beng::tests`; hier bewust
    /// gedupliceerd omdat die test-fixture niet cross-crate geëxporteerd is en
    /// deze route-test alleen de HTTP-laag (status + envelope) verifieert.
    fn wall(id: &str, orientation_deg: f64, area_m2: f64, window_area: f64) -> Construction {
        Construction {
            id: id.into(),
            description: format!("gevel {id}"),
            kind: ConstructionKind::Wall,
            boundary: BoundaryKind::Exterior,
            area_m2,
            u_value: 0.21,
            orientation_deg: Some(orientation_deg),
            slope_deg: Some(90.0),
            openings: if window_area > 0.0 {
                vec![Opening {
                    id: format!("{id}-raam"),
                    kind: OpeningKind::Window,
                    area_m2: window_area,
                    u_value: 1.4,
                    g_value: Some(0.6),
                    frame_fraction: Some(0.25),
                    movable_shading: None,
                    obstruction: Default::default(),
                }]
            } else {
                vec![]
            },
            layers: vec![],
            adjacent_space_id: None,
            psi_thermal_bridge: None,
        }
    }

    fn opaque(id: &str, kind: ConstructionKind, boundary: BoundaryKind, area_m2: f64, u: f64) -> Construction {
        Construction {
            id: id.into(),
            description: id.into(),
            kind,
            boundary,
            area_m2,
            u_value: u,
            orientation_deg: None,
            slope_deg: None,
            openings: vec![],
            layers: vec![],
            adjacent_space_id: None,
            psi_thermal_bridge: None,
        }
    }

    /// All-electric rijtjeshuis (WP-bodem + WTW-D) — voldoende voor een
    /// niet-triviaal, `Ok`-producerend `compute_beng`-resultaat.
    fn valid_project() -> ProjectV2 {
        let mut p = ProjectV2::new("BENG route happy-path");
        p.shared.building_type = BuildingTypeShared::Woning {
            subtype: ResidentialType::Terraced,
        };
        p.shared.gross_floor_area_m2 = Some(87.0);
        p.shared.num_storeys = Some(2);
        p.shared.construction_year = Some(2022);

        p.geometry = SharedGeometry {
            spaces: vec![Space {
                id: "s1".into(),
                name: "Woning".into(),
                function: None,
                floor_area_m2: 87.0,
                height_m: 2.7,
                theta_i_winter_c: Some(20.0),
                theta_i_summer_c: Some(24.0),
                constructions: vec![
                    wall("gevel-zw", 225.0, 34.0, 12.0),
                    wall("gevel-no", 45.0, 34.0, 6.0),
                    opaque("dak", ConstructionKind::Roof, BoundaryKind::Exterior, 44.0, 0.16),
                    opaque("vloer", ConstructionKind::Floor, BoundaryKind::Ground, 44.0, 0.26),
                ],
            }],
            ..Default::default()
        };

        p.energy = Some(EnergyInput {
            heating: Some(HeatingInput {
                generator: HeatGeneratorType::HeatPumpGround,
                cop: Some(4.5),
                hr_class: None,
                district_factor: None,
                emission: Some(HeatEmissionType::FloorHeating),
                distribution_efficiency: None,
                control_factor: None,
                coverage_fraction: 1.0,
                source: None,
            }),
            dhw: Some(DhwInput {
                generator: DhwGeneratorType::HeatPump,
                efficiency: Some(2.8),
                dwtw: None,
                has_solar_boiler: false,
                solar_boiler_fraction: None,
                source: None,
            }),
            ventilation: Some(VentilationInput {
                system: VentilationSystemType::D,
                wtw_efficiency: Some(0.85),
                sfp_w_per_m3h: None,
                bypass_enabled: true,
                mechanical_supply_m3_per_h: Some(150.0),
                mechanical_exhaust_m3_per_h: Some(150.0),
                infiltration_m3_per_h: None,
                source: None,
            }),
            cooling: Some(CoolingInput {
                generator: CoolingGeneratorType::FreeCooling,
                seer: None,
                cop: None,
                free_cooling_fraction: Some(0.4),
                source: None,
            }),
            pv: vec![],
            automation: None,
        });

        p
    }

    #[tokio::test]
    async fn beng_calculate_happy_path_returns_200() {
        let req = BengCalculateRequest {
            project: valid_project(),
        };
        let resp = beng_calculate(Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn beng_calculate_missing_energy_returns_422() {
        let mut project = valid_project();
        project.energy = None;
        let resp = beng_calculate(Json(BengCalculateRequest { project }))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn request_envelope_round_trips() {
        let req = BengCalculateRequest {
            project: valid_project(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: BengCalculateRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(back.project.energy.is_some());
    }
}
