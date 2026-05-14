//! TO-juli orchestrator — volledige NTA 8800 H.10 keten op een [`ProjectV2`].
//!
//! Combineert [`nta8800_view`] (geometrie-mapper) + `nta8800-demand` (H.7)
//! + `nta8800-cooling` (H.10) tot één publieke functie die uit de gedeelde
//! geometrie + cooling-system inputs een [`TojuliResult`] berekent met
//! maandelijkse Q_C;use en jaarsom.
//!
//! ## V1 scope
//!
//! Transmissie en ventilatie worden in V1 **gesynthesizeerd** uit de
//! geometry-mapper (Σ A·U voor H_T) en een eenvoudig ach-model (0.5 ach
//! woning / 1.0 ach utiliteit voor H_V). Dat is genoeg om de demand-keten
//! te voeden zonder dat de volledige `nta8800-transmission` /
//! `nta8800-ventilation` integratie nodig is — die landen in F7.2.
//!
//! ## V2 / vervolg
//!
//! - Echte transmissie via `nta8800-transmission::calculate_transmission`
//! - Echte ventilatie via `nta8800-ventilation::calculate_ventilation`
//!   met WTW + n_air uit `TojuliInputs`
//! - Schaduw-factor uit BuildingPart-overstek/luifel-modellering
//! - Multi-rekenzone splitsing
//! - EP-bijdrage berekening (energieprestatie-index)

use nta8800_cooling::{
    calculate_cooling, CoolingDistribution, CoolingEmission, CoolingResult, CoolingSystem,
};
use nta8800_demand::calc::calculate_demand;
use nta8800_demand::model::{
    InternalGains, ThermalMassInput,
    setpoints::{CoolingSetpoint, HeatingSetpoint},
};
use nta8800_demand::result::DemandResult;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{Energy, Temperature};
use nta8800_tables::climate::de_bilt_climate_data;
use nta8800_transmission::result::{TransmissionBreakdown, TransmissionResult};
use nta8800_ventilation::result::VentilationResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::geometry::BoundaryKind;
use crate::nta8800_view::{Nta8800View, geometry_to_nta8800};
use crate::project::ProjectV2;
use crate::shared::BuildingTypeShared;

/// TO-juli specifieke inputs voor de volledige H.10-berekening.
///
/// Aanvulling op `Calcs::tojuli` — bevat de cooling-installatie en
/// gebruikersgedrag. Geometrie wordt uit `ProjectV2.geometry` gehaald.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TojuliFullInputs {
    /// Type koelopwekker (compressie/absorptie/vrije koeling) met COP.
    pub system: CoolingSystem,
    /// Distributie-rendement η_dist;C.
    pub distribution: CoolingDistribution,
    /// Emissie + regelfactor.
    pub emission: CoolingEmission,
    /// Schaduw-factor F_sh (0..=1). 1.0 = geen schaduw. V2: per-construction.
    #[serde(default = "default_shading")]
    pub shading_factor: f64,
    /// Ventilatievoud n_air (ach, 1/h). 0 = auto uit gebouwtype.
    #[serde(default)]
    pub air_change_rate_per_h: f64,
    /// Verwarmings-setpoint °C (constant alle maanden).
    #[serde(default = "default_heating_setpoint")]
    pub heating_setpoint_c: f64,
    /// Koel-setpoint °C (constant alle maanden).
    #[serde(default = "default_cooling_setpoint")]
    pub cooling_setpoint_c: f64,
}

fn default_shading() -> f64 {
    1.0
}
fn default_heating_setpoint() -> f64 {
    20.0
}
fn default_cooling_setpoint() -> f64 {
    24.0
}

/// Resultaat van de volledige TO-juli keten.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TojuliResult {
    /// Maandelijkse koudebehoefte Q_C;nd in MJ (uit demand-pipeline).
    pub monthly_q_c_nd_mj: MonthlyProfile<Energy>,
    /// Maandelijkse koel-energie Q_C;use in MJ (na η_em·η_dist·COP).
    pub monthly_q_c_use_mj: MonthlyProfile<Energy>,
    /// Jaarsom Q_C;use in MJ.
    pub annual_q_c_use_mj: Energy,
    /// Jaarsom Q_C;use in kWh.
    pub annual_q_c_use_kwh: f64,
    /// Maandelijkse warmtebehoefte Q_H;nd in MJ (bijproduct demand-keten).
    pub monthly_q_h_nd_mj: MonthlyProfile<Energy>,
    /// H_T (W/K) gebruikt voor demand — Σ A·U op exterior/ground/adjacent.
    pub transmission_h_t_w_per_k: f64,
    /// H_V (W/K) gebruikt voor demand — synthetisch uit ach + volume.
    pub ventilation_h_v_w_per_k: f64,
    /// Maandelijkse buitenluchttemperatuur (input De Bilt, voor UI-context).
    pub monthly_theta_e_c: MonthlyProfile<Temperature>,
    /// Tijdconstante τ_zone in uren.
    pub tau_hours: f64,
}

/// Errors van de TO-juli orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum TojuliError {
    /// NTA 8800 model-validatie faalde (oriëntatie/tilt buiten bereik etc.).
    #[error("nta8800 model error: {0}")]
    Model(#[from] nta8800_model::ModelError),
    /// Demand-keten faalde.
    #[error("nta8800-demand error: {0}")]
    Demand(#[from] nta8800_demand::errors::DemandError),
    /// Cooling-keten faalde.
    #[error("nta8800-cooling error: {0}")]
    Cooling(#[from] nta8800_cooling::CoolingError),
    /// Project mist een rekenzone (lege geometrie + geen gross_floor_area).
    #[error("project levert geen rekenzone (lege geometrie)")]
    EmptyProject,
}

/// Hoofd-orchestrator: voer de volledige TO-juli H.10 berekening uit.
///
/// Pipeline:
/// 1. `geometry_to_nta8800` levert Rekenzone + EFR + Window + Construction
/// 2. H_T = Σ A·U op exterior/ground/unheated/adjacent_building constructions
/// 3. H_V uit `air_change_rate × volume × ρc_p` (default 0.5 of 1.0 ach)
/// 4. Synthetische `TransmissionResult` + `VentilationResult` (monthly Q uit H × Δθ × uren)
/// 5. `calculate_demand` → Q_C;nd 12 maanden
/// 6. `calculate_cooling` → Q_C;use 12 maanden + jaarsom
///
/// # Errors
/// Zie [`TojuliError`].
pub fn compute_tojuli_full(
    project: &ProjectV2,
    inputs: &TojuliFullInputs,
) -> Result<TojuliResult, TojuliError> {
    // ---- 1. View ----
    let view = geometry_to_nta8800(&project.shared, &project.geometry)?;
    let zone = view.rekenzones.first().ok_or(TojuliError::EmptyProject)?;

    // ---- 2. H_T uit Σ A·U ----
    let h_t = compute_h_t_from_geometry(&project.geometry, &view);

    // ---- 3. H_V uit ach × volume × ρ·c_p (0.34 W/(m³/h·K)) ----
    let ach = if inputs.air_change_rate_per_h > 0.0 {
        inputs.air_change_rate_per_h
    } else {
        default_ach(&project.shared.building_type)
    };
    let h_v = ach * zone.volume * 0.34;

    // ---- 4. Synthesize TransmissionResult + VentilationResult ----
    let climate = de_bilt_climate_data();
    let theta_i_winter = inputs.heating_setpoint_c;
    let monthly_q_t = build_monthly_q(h_t, &climate.outdoor_temperature, theta_i_winter);
    let monthly_q_v = build_monthly_q(h_v, &climate.outdoor_temperature, theta_i_winter);

    let annual_q_t: Energy = Month::all().iter().map(|m| monthly_q_t[*m]).sum();
    let annual_q_v: Energy = Month::all().iter().map(|m| monthly_q_v[*m]).sum();

    let transmission = TransmissionResult {
        monthly_q_t: monthly_q_t.clone(),
        annual_q_t,
        breakdown: TransmissionBreakdown {
            outdoor: monthly_q_t.clone(),
            unheated_space: MonthlyProfile::from_constant(0.0),
            ground: MonthlyProfile::from_constant(0.0),
            adjacent_zone: MonthlyProfile::from_constant(0.0),
            thermal_bridges: MonthlyProfile::from_constant(0.0),
        },
        h_d: h_t,
        h_u: 0.0,
        h_g_an: 0.0,
        h_a: 0.0,
    };

    let ventilation = VentilationResult {
        monthly_q_v: monthly_q_v.clone(),
        annual_q_v,
        monthly_w_fan: MonthlyProfile::from_constant(0.0),
        annual_w_fan: 0.0,
        monthly_wtw_recovery: MonthlyProfile::from_constant(0.0),
        annual_wtw_recovery: 0.0,
    };

    // ---- 5. Demand calc ----
    let usage_function = view
        .efrs
        .first()
        .map(|e| e.usage_function)
        .unwrap_or(nta8800_model::zoning::UsageFunction::Woonfunctie);
    let internal_gains = InternalGains::forfaitair(usage_function);
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(theta_i_winter));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(inputs.cooling_setpoint_c));
    let thermal_mass = ThermalMassInput::light_woning(); // default; F7.2 user-input

    let windows_refs: Vec<&nta8800_model::geometry::window::Window> = view.windows.iter().collect();
    let demand: DemandResult = calculate_demand(
        zone,
        &transmission,
        &ventilation,
        h_v,
        &windows_refs,
        &climate,
        heating_sp,
        cooling_sp,
        &internal_gains,
        thermal_mass,
        inputs.shading_factor,
    )?;

    // ---- 6. Cooling calc ----
    let cooling: CoolingResult = calculate_cooling(
        zone,
        &demand,
        &inputs.system,
        &inputs.distribution,
        &inputs.emission,
    )?;

    let annual_q_c_use_mj = cooling.annual_q_c_use;
    let annual_q_c_use_kwh = annual_q_c_use_mj / 3.6;

    Ok(TojuliResult {
        monthly_q_c_nd_mj: demand.monthly_cooling_demand.clone(),
        monthly_q_c_use_mj: cooling.monthly_q_c_use,
        annual_q_c_use_mj,
        annual_q_c_use_kwh,
        monthly_q_h_nd_mj: demand.monthly_heating_demand,
        transmission_h_t_w_per_k: h_t,
        ventilation_h_v_w_per_k: h_v,
        monthly_theta_e_c: climate.outdoor_temperature,
        tau_hours: demand.breakdown.time_constant_hours,
    })
}

/// Σ A·U voor constructies aan exterior / ground / unheated / adjacent_building.
/// Adjacent_room en open_water tellen niet mee (interne uitwisseling resp.
/// aparte boundary-modeling, V2).
fn compute_h_t_from_geometry(geometry: &crate::geometry::SharedGeometry, _view: &Nta8800View) -> f64 {
    let mut h_t = 0.0;
    for space in &geometry.spaces {
        for c in &space.constructions {
            let counts = matches!(
                c.boundary,
                BoundaryKind::Exterior
                    | BoundaryKind::Ground
                    | BoundaryKind::UnheatedSpace
                    | BoundaryKind::AdjacentBuilding
            );
            if counts {
                h_t += c.area_m2 * c.u_value;
            }
        }
    }
    h_t
}

/// Default ach (1/h) op basis van gebouwtype voor V1 stub.
fn default_ach(bt: &BuildingTypeShared) -> f64 {
    match bt {
        BuildingTypeShared::Woning { .. } => 0.5,
        BuildingTypeShared::Utiliteit { .. } => 1.0,
    }
}

/// Bouw maandelijkse Q [MJ] uit H [W/K] × Δθ [°C] × uren [h] / 1e6.
/// Δθ = theta_i_winter - theta_e[m] gewoon positief = warmteverlies.
fn build_monthly_q(
    h: f64,
    theta_e: &MonthlyProfile<Temperature>,
    theta_i: Temperature,
) -> MonthlyProfile<Energy> {
    let hours_per_month = [
        744.0_f64, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
    ];
    let months = Month::all();
    let mut values = [0.0_f64; 12];
    for (i, &m) in months.iter().enumerate() {
        let delta = (theta_i - theta_e[m]).max(0.0);
        values[i] = h * delta * hours_per_month[i] * 3600.0 / 1e6;
    }
    MonthlyProfile::new(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{
        Construction as SC, ConstructionKind, OpeningKind, SharedGeometry, Space,
    };
    use crate::shared::ResidentialType;

    fn sample_project() -> ProjectV2 {
        let mut p = ProjectV2::new("Test Woning");
        p.shared.gross_floor_area_m2 = Some(120.0);
        p.shared.building_type = BuildingTypeShared::Woning {
            subtype: ResidentialType::Detached,
        };
        p.geometry = SharedGeometry {
            spaces: vec![Space {
                id: "S1".into(),
                name: "Hele woning".into(),
                function: None,
                floor_area_m2: 120.0,
                height_m: 2.7,
                theta_i_winter_c: Some(20.0),
                theta_i_summer_c: Some(24.0),
                constructions: vec![SC {
                    id: "C1".into(),
                    description: "Gevels totaal".into(),
                    kind: ConstructionKind::Wall,
                    boundary: BoundaryKind::Exterior,
                    area_m2: 150.0,
                    u_value: 0.3,
                    orientation_deg: Some(180.0),
                    slope_deg: Some(90.0),
                    openings: vec![crate::geometry::Opening {
                        id: "W1".into(),
                        kind: OpeningKind::Window,
                        area_m2: 20.0,
                        u_value: 1.4,
                        g_value: Some(0.6),
                        frame_fraction: Some(0.2),
                    }],
                    layers: vec![],
                    adjacent_space_id: None,
                    psi_thermal_bridge: None,
                }],
            }],
        };
        p
    }

    fn sample_inputs() -> TojuliFullInputs {
        TojuliFullInputs {
            system: CoolingSystem::CompressionCooling { scop_cooling: 3.5 },
            distribution: CoolingDistribution::default_insulated(),
            emission: CoolingEmission {
                efficiency: 0.95,
                regulation_factor: 0.95,
            },
            shading_factor: 1.0,
            air_change_rate_per_h: 0.0, // auto
            heating_setpoint_c: 20.0,
            cooling_setpoint_c: 24.0,
        }
    }

    #[test]
    fn end_to_end_woning_120m2() {
        let p = sample_project();
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute_tojuli_full ok");
        // H_T = 150 × 0.3 = 45 W/K
        assert!((r.transmission_h_t_w_per_k - 45.0).abs() < 1e-6);
        // H_V default 0.5 ach × 120 × 2.7 × 0.34 = 55.08 W/K
        assert!((r.ventilation_h_v_w_per_k - 55.08).abs() < 0.1);
        // Q_C;use jaar > 0 (woning heeft ramen, krijgt zonbelasting in zomer)
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.annual_q_c_use_kwh >= 0.0);
        assert!(r.tau_hours > 0.0);
    }

    #[test]
    fn empty_geometry_uses_gross_area_fallback() {
        let mut p = ProjectV2::new("Empty");
        p.shared.gross_floor_area_m2 = Some(100.0);
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok");
        // H_T = 0 want geen constructies
        assert_eq!(r.transmission_h_t_w_per_k, 0.0);
        // H_V > 0 want volume = 100 × 2.7
        assert!(r.ventilation_h_v_w_per_k > 0.0);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }
}
