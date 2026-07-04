//! # nta8800-core
//!
//! Unified façade over de NTA 8800:2025+C1:2026 reken-crates — één
//! JSON-in/JSON-uit entry-point die de volledige energieprestatie-keten
//! orkestreert:
//!
//! ```text
//! Project (JSON)
//!   → transmissie (H.8) + ventilatie (H.11)
//!   → warmte-/koudebehoefte (H.7)
//!   → diensten: verwarming (H.9), koeling (H.10), tapwater (H.13),
//!     verlichting (H.14, utiliteit), PV (H.16)
//!   → EP-score + energielabel (H.5)
//! Nta8800Result (JSON)
//! ```
//!
//! Volgt het isso51-core / isso53-core API-patroon:
//!
//! - [`calculate_from_json`] — JSON in → JSON uit
//! - [`calculate`] — typed API
//! - [`project_schema`] — JSON-schema van het invoer-model (schemars)
//!
//! ## V1 scope
//!
//! - Eén rekenzone voor het hele gebouw (multi-zone is V2)
//! - De Bilt referentie-klimaat (NEN 5060 via `nta8800-tables`)
//! - Verlichting alleen voor utiliteitsfuncties (H.14 kent geen
//!   woonfunctie-forfait)
//! - Gebouwautomatisering (H.15) niet in de EP-optelling (V2)
//! - Bevochtiging (`nta8800-humidity`) buiten scope (V2)
//! - Ventilatie via de systeem-bewuste heuristiek; het §11.2.1 drukmodel
//!   (massabalans) is V2
//!
//! ## Voorbeeld
//!
//! ```no_run
//! let json = std::fs::read_to_string("examples/minimal.json").unwrap();
//! let result = nta8800_core::calculate_from_json(&json).unwrap();
//! println!("{result}");
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Doc-comments gebruiken NTA 8800-symbolen (Q_H;nd, q_V;ODA;req, η, etc.)
// die de backtick-heuristiek als false positive oppikt — consistent met de
// andere nta8800-crates.
#![allow(clippy::doc_markdown)]

pub mod error;
pub mod formulas;
pub mod model;
pub mod orchestrator;
pub mod result;

pub use error::{CoreError, CoreResult};
pub use model::Project;
pub use result::Nta8800Result;

/// Voer de volledige NTA 8800 keten uit op een JSON-projectbeschrijving.
///
/// # Errors
///
/// [`CoreError::Json`] bij ongeldig JSON; verder alle keten-fouten uit
/// [`calculate`].
pub fn calculate_from_json(input_json: &str) -> CoreResult<String> {
    let project: Project = serde_json::from_str(input_json)?;
    let result = calculate(&project)?;
    Ok(serde_json::to_string_pretty(&result)?)
}

/// Typed entry-point: [`Project`] → [`Nta8800Result`].
///
/// # Errors
///
/// Zie [`CoreError`].
pub fn calculate(project: &Project) -> CoreResult<Nta8800Result> {
    orchestrator::calculate(project)
}

/// JSON-schema (schemars) van het [`Project`] invoer-model.
///
/// # Panics
///
/// Panics als schema-serialisatie faalt — kan alleen bij een programmeerfout
/// in de schema-derives, niet door user-invoer.
#[must_use]
pub fn project_schema() -> String {
    let schema = schemars::schema_for!(Project);
    serde_json::to_string_pretty(&schema).expect("schema serialisatie faalt nooit op eigen types")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Boundary, Building, Conditions, DhwInput, EnvelopeElement, HeatingInput, ProjectInfo,
        ThermalMassClass, VentilationInput, VentilationSystemInput, WindowElement,
    };
    use nta8800_dhw::model::{DhwEmission, DhwGenerationSystem};
    use nta8800_heating::model::{EmissionSystem, GenerationSystem, HRClass};
    use nta8800_model::zoning::UsageFunction;

    /// Referentie-woning: 120 m² vrijstaand, HR-107 ketel + radiator LT,
    /// systeem C ventilatie, HR-combi tapwater, geen koeling, geen PV.
    fn sample_project() -> Project {
        Project {
            info: ProjectInfo {
                name: "Testwoning 120 m²".into(),
                description: None,
            },
            building: Building {
                usage_function: UsageFunction::Woonfunctie,
                floor_area_m2: 120.0,
                volume_m3: Some(324.0),
                construction_year: Some(1995),
                thermal_mass: ThermalMassClass::Heavy,
            },
            envelope: vec![
                EnvelopeElement {
                    id: "gevel-zuid".into(),
                    description: "Zuidgevel".into(),
                    area_m2: 40.0,
                    u_value: 0.30,
                    boundary: Boundary::Exterior,
                    orientation_deg: Some(180.0),
                    tilt_deg: Some(90.0),
                    windows: vec![WindowElement {
                        id: "raam-zuid".into(),
                        area_m2: 8.0,
                        u_value: 1.4,
                        g_value: 0.6,
                        frame_fraction: 0.25,
                    }],
                },
                EnvelopeElement {
                    id: "gevel-noord".into(),
                    description: "Noordgevel".into(),
                    area_m2: 40.0,
                    u_value: 0.30,
                    boundary: Boundary::Exterior,
                    orientation_deg: Some(0.0),
                    tilt_deg: Some(90.0),
                    windows: vec![WindowElement {
                        id: "raam-noord".into(),
                        area_m2: 4.0,
                        u_value: 1.4,
                        g_value: 0.6,
                        frame_fraction: 0.25,
                    }],
                },
                EnvelopeElement {
                    id: "dak".into(),
                    description: "Hellend dak".into(),
                    area_m2: 70.0,
                    u_value: 0.20,
                    boundary: Boundary::Exterior,
                    orientation_deg: None,
                    tilt_deg: Some(45.0),
                    windows: vec![],
                },
                EnvelopeElement {
                    id: "bg-vloer".into(),
                    description: "Begane-grondvloer".into(),
                    area_m2: 60.0,
                    u_value: 0.25,
                    boundary: Boundary::Ground,
                    orientation_deg: None,
                    tilt_deg: Some(0.0),
                    windows: vec![],
                },
            ],
            ventilation: VentilationInput {
                system: VentilationSystemInput::MechanicalExhaust,
                mechanical_supply_m3_per_h: None,
                mechanical_exhaust_m3_per_h: None,
                infiltration_m3_per_h: None,
            },
            heating: HeatingInput {
                emission: EmissionSystem::RadiatorLowTemp,
                generation: GenerationSystem::HRBoiler {
                    class: HRClass::HR107,
                },
                distribution_efficiency: 0.95,
                control_factor: 0.97,
            },
            cooling: None,
            dhw: DhwInput {
                generation: DhwGenerationSystem::HRCombiBoiler,
                emission: DhwEmission::WoningDefault,
                distribution_efficiency: 1.0,
                shower_heat_recovery: None,
                annual_demand_kwh: None,
            },
            lighting: None,
            pv: None,
            conditions: Conditions::default(),
        }
    }

    #[test]
    fn end_to_end_woning_levert_label_en_positieve_ketens() {
        let result = calculate(&sample_project()).expect("keten ok");

        // Behoefte: winter-warmtebehoefte moet positief zijn voor NL-klimaat.
        assert!(result.demand.annual_q_h_nd_mj > 0.0, "Q_H;nd > 0");
        assert!(result.demand.tau_hours > 0.0, "τ > 0");
        // H_tr = Σ A·U (opaque + ramen) + h_g_an:
        //   zuid: 32×0,30 + 8×1,4 = 9,6 + 11,2 = 20,8
        //   noord: 36×0,30 + 4×1,4 = 10,8 + 5,6 = 16,4
        //   dak: 70×0,20 = 14,0 → buiten-som 51,2
        //   vloer (ground) telt via H_g;an forfait 10,0
        assert!(
            result.demand.h_tr_w_per_k > 40.0 && result.demand.h_tr_w_per_k < 80.0,
            "H_tr plausibel (was {})",
            result.demand.h_tr_w_per_k
        );

        // Verwarming: gasketel-keten met alle rendementen ∈ (0, 1].
        assert_eq!(result.heating.energy_carrier, "aardgas");
        assert!(result.heating.annual_use_mj > result.demand.annual_q_h_nd_mj);
        let eta = result.heating.total_efficiency.expect("η bekend");
        assert!(eta > 0.0 && eta <= 1.0);

        // Tapwater: forfait 120 m² woning > 0.
        assert!(result.dhw.annual_use_mj > 0.0);
        assert_eq!(result.dhw.energy_carrier, "aardgas");

        // Geen koeling / verlichting / PV geconfigureerd.
        assert!(result.cooling.is_none());
        assert!(result.lighting.is_none());
        assert!(result.pv.is_none());

        // EP: label toegekend + specifieke primaire energie > 0.
        assert!(!result.ep.label.is_empty());
        assert!(result.ep.primary_energy_mj_per_m2 > 0.0);
        assert!(result.ep.per_service_primary_mj.heating > 0.0);
    }

    #[test]
    fn json_round_trip() {
        let project = sample_project();
        let json = serde_json::to_string(&project).expect("serialize");
        let output = calculate_from_json(&json).expect("keten via json ok");
        let parsed: Nta8800Result = serde_json::from_str(&output).expect("result parse");
        assert!(parsed.demand.annual_q_h_nd_mj > 0.0);
    }

    #[test]
    fn schema_bevat_project_velden() {
        let schema = project_schema();
        assert!(schema.contains("usage_function"));
        assert!(schema.contains("envelope"));
        assert!(schema.contains("heating"));
    }

    #[test]
    fn validatie_weigert_lege_envelope() {
        let mut p = sample_project();
        p.envelope.clear();
        let err = calculate(&p).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn pv_verhoogt_hernieuwbaar_aandeel() {
        use nta8800_pv::PvSystem;
        let base = calculate(&sample_project()).expect("basis ok");

        let mut with_pv = sample_project();
        with_pv.pv = Some(vec![PvSystem {
            peak_power_kwp: 4.0,
            tilt_degrees: 35.0,
            azimuth_degrees: 180.0,
            system_efficiency: 0.85,
            inverter_efficiency: 0.96,
            shadow_factor: 1.0,
        }]);
        let result = calculate(&with_pv).expect("pv ok");

        let pv = result.pv.expect("pv summary aanwezig");
        assert!(pv.annual_yield_mj > 0.0, "PV levert op");
        // nta8800-ep V1 hanteert f_prim = 0 voor hernieuwbare elektriciteit
        // ter plaatse: PV verlaagt dus NIET de primaire energie maar telt in
        // het hernieuwbaar aandeel (bijlage Z-semantiek van de sub-crate).
        assert!(
            result.ep.renewable_share > base.ep.renewable_share,
            "PV verhoogt het hernieuwbaar aandeel ({} → {})",
            base.ep.renewable_share,
            result.ep.renewable_share
        );
        // 4 kWp zuid, 35° → orde 3.000-6.000 kWh/jaar → 10.800-21.600 MJ.
        assert!(
            pv.annual_yield_mj > 8_000.0 && pv.annual_yield_mj < 25_000.0,
            "PV-opbrengst plausibel (was {} MJ)",
            pv.annual_yield_mj
        );
    }
}
