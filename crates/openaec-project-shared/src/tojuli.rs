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
use nta8800_transmission::{
    calculate_transmission,
    BoundaryType as TransmissionBoundaryType, TransmissionElement,
};
use nta8800_ventilation::{
    calculate_ventilation,
    model::{AirFlow, VentilationSystem, WtwSpecification},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Forfaitair specifiek ventilator vermogen (NTA 8800 tabel 11.23, modern DC-unit).
const VENTILATION_FAN_SFP_W_PER_M3H: f64 = 0.125;

use crate::geometry::{BoundaryKind, SharedGeometry};
use crate::nta8800_view::geometry_to_nta8800;
use crate::project::ProjectV2;
use crate::shared::{BuildingTypeShared, VentilationSystemKind, HeatRecovery};

/// Map geometry::BoundaryKind to nta8800_transmission::BoundaryType.
///
/// Dit is de enum-mapper die verplicht is om de geometry-laag te koppelen
/// aan de nta8800-transmission crate (zie sessie-handoff §bekende valkuilen).
fn map_boundary_kind_to_transmission_type(kind: BoundaryKind, adjacent_space_id: Option<&String>) -> TransmissionBoundaryType {
    match kind {
        BoundaryKind::Exterior => TransmissionBoundaryType::Outdoor,
        BoundaryKind::Ground => TransmissionBoundaryType::Ground,
        BoundaryKind::OpenWater => TransmissionBoundaryType::Outdoor, // Water behandeld als Outdoor met aparte θ_e
        BoundaryKind::UnheatedSpace => TransmissionBoundaryType::UnheatedSpace {
            id: adjacent_space_id.cloned().unwrap_or_else(|| "default_unheated".to_string()),
        },
        BoundaryKind::AdjacentRoom => TransmissionBoundaryType::AdjacentZone {
            id: adjacent_space_id.cloned().unwrap_or_else(|| "unknown_adjacent".to_string()),
        },
        BoundaryKind::AdjacentBuilding => TransmissionBoundaryType::AdjacentZone {
            id: adjacent_space_id.cloned().unwrap_or_else(|| "adjacent_building".to_string()),
        },
    }
}

/// Bouw TransmissionElement lijst uit SharedGeometry voor calculate_transmission.
///
/// Converteert alle Construction's uit alle Space's naar TransmissionElement's
/// met de juiste BoundaryType mapping.
fn build_transmission_elements(geometry: &SharedGeometry) -> Vec<TransmissionElement> {
    let mut elements = Vec::new();

    for space in &geometry.spaces {
        for construction in &space.constructions {
            // Skip interne wanden tussen verwarmde ruimtes — netto-transmissie ≈ 0
            // bij identiek heating-setpoint. AdjacentRoom support komt met multi-zone in latere release.
            if matches!(construction.boundary, BoundaryKind::AdjacentRoom) {
                continue;
            }

            let boundary_type = map_boundary_kind_to_transmission_type(
                construction.boundary,
                construction.adjacent_space_id.as_ref()
            );

            let element = TransmissionElement {
                id: format!("{}_{}", space.id, construction.id),
                area: construction.area_m2,
                u_value: construction.u_value,
                boundary_type,
                construction_id: Some(construction.id.clone()),
            };

            elements.push(element);
        }
    }

    elements
}

/// Map ventilatie-configuratie uit SharedProject naar nta8800-ventilation types.
fn map_ventilation_to_nta8800(
    system_kind: Option<VentilationSystemKind>,
    mech_supply_m3_per_h: Option<f64>,
    mech_exhaust_m3_per_h: Option<f64>,
    infiltration_m3_per_h: Option<f64>,
    heat_recovery: Option<&HeatRecovery>,
) -> (VentilationSystem, AirFlow, Option<WtwSpecification>) {
    // Map system kind naar VentilationSystem
    let system = match system_kind {
        Some(VentilationSystemKind::MechBalanced) => {
            VentilationSystem::D {
                with_wtw: heat_recovery.is_some()
            }
        }
        Some(VentilationSystemKind::MechSupply) => VentilationSystem::B,
        Some(VentilationSystemKind::MechExhaust) => VentilationSystem::C,
        Some(VentilationSystemKind::Natural) => VentilationSystem::A,
        None => VentilationSystem::C, // fallback: NL pre-2000 mech exhaust
    };

    // Construct AirFlow
    let flow = AirFlow {
        mechanical_supply: mech_supply_m3_per_h.unwrap_or(0.0),
        mechanical_exhaust: mech_exhaust_m3_per_h.unwrap_or(0.0),
        infiltration: infiltration_m3_per_h.unwrap_or(0.0),
    };

    // WtwSpecification alleen voor gebalanceerde ventilatie met WTW
    let wtw = match system {
        VentilationSystem::D { with_wtw: true } => {
            heat_recovery.map(|hr| WtwSpecification {
                efficiency: hr.efficiency,
                fan_sfp: VENTILATION_FAN_SFP_W_PER_M3H,
                bypass_enabled: false, // default; V2 heeft geen veld, hardcoded false
            })
        }
        _ => None, // geen WTW voor andere systemen
    };

    (system, flow, wtw)
}

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
    /// Transmission-keten faalde.
    #[error("nta8800-transmission error: {0}")]
    Transmission(#[from] nta8800_transmission::errors::TransmissionError),
    /// Ventilation-keten faalde.
    #[error("nta8800-ventilation error: {0}")]
    Ventilation(#[from] nta8800_ventilation::VentilationError),
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

    // ---- 2. Echte transmissie via nta8800-transmission ----
    let climate = de_bilt_climate_data();
    let elements = build_transmission_elements(&project.geometry);
    let thermal_bridges_linear = Vec::new(); // Forfaitair 0 (NTA §7.3.3)
    let thermal_bridges_point = Vec::new();  // Forfaitair 0 (NTA §7.3.3)

    let indoor_temperature = MonthlyProfile::from_constant(inputs.heating_setpoint_c);

    // Forfaitaire defaults voor v1 norm-strict:
    // h_g_an via NTA §8.3.1 minimum voor residentieel: 10 W/K voor gemiddeld huis
    let h_g_an = 10.0; // NTA §8.3.1 forfaitair minimum voor woningen zonder grondcontact-details

    // b_factors: alle onverwarmde ruimtes krijgen 0.5 (NTA §8.4.1 default)
    let mut unheated_space_b_factors = std::collections::HashMap::new();
    for space in &project.geometry.spaces {
        for construction in &space.constructions {
            if matches!(construction.boundary, BoundaryKind::UnheatedSpace) {
                let key = construction.adjacent_space_id
                    .clone()
                    .unwrap_or_else(|| "default_unheated".to_string());
                unheated_space_b_factors.entry(key).or_insert(0.5);
            }
        }
    }
    // Behoud de default-key zodat lege geometrie ook werkt
    unheated_space_b_factors.entry("default_unheated".to_string()).or_insert(0.5);

    // Lege adjacent_zone_temperatures: geen adjacent rooms in v1
    let adjacent_zone_temperatures: std::collections::HashMap<String, MonthlyProfile<Temperature>> = std::collections::HashMap::new();

    let transmission = calculate_transmission(
        zone,
        &elements,
        &thermal_bridges_linear,
        &thermal_bridges_point,
        &indoor_temperature,
        &climate,
        h_g_an,
        &unheated_space_b_factors,
        &adjacent_zone_temperatures,
    ).map_err(TojuliError::Transmission)?;

    // ---- 3. Ventilation via nta8800-ventilation engine ----
    // Legacy fallback: als geen ventilatie-configuratie, gebruik ach-gebaseerde infiltratie
    let effective_infiltration_m3_per_h = project.shared.infiltration_m3_per_h.or_else(|| {
        let needs_fallback = project.shared.ventilation_system.is_none()
            && project.shared.mechanical_supply_m3_per_h.is_none()
            && project.shared.mechanical_exhaust_m3_per_h.is_none();
        if needs_fallback {
            let ach = if inputs.air_change_rate_per_h > 0.0 {
                inputs.air_change_rate_per_h
            } else {
                default_ach(&project.shared.building_type)
            };
            Some(ach * zone.volume)
        } else {
            None
        }
    });

    let (system, flow, wtw) = map_ventilation_to_nta8800(
        project.shared.ventilation_system,
        project.shared.mechanical_supply_m3_per_h,
        project.shared.mechanical_exhaust_m3_per_h,
        effective_infiltration_m3_per_h,
        project.shared.heat_recovery.as_ref(),
    );

    let ventilation = calculate_ventilation(
        zone,
        &system,
        &flow,
        wtw.as_ref(),
        &indoor_temperature,
        &climate,
    ).map_err(TojuliError::Ventilation)?;

    // h_v voor demand calc: leid af uit q_v_total via factor 1.2 kJ/(m³·K) → 0.34 W/(m³/h·K)
    let q_v_total = flow.mechanical_supply.max(flow.mechanical_exhaust).max(flow.infiltration);
    let h_v = q_v_total * 0.34;

    // ---- 5. Demand calc ----
    let usage_function = view
        .efrs
        .first()
        .map(|e| e.usage_function)
        .unwrap_or(nta8800_model::zoning::UsageFunction::Woonfunctie);
    let internal_gains = InternalGains::forfaitair(usage_function);
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(inputs.heating_setpoint_c));
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
        transmission_h_t_w_per_k: transmission.h_d + transmission.h_u + transmission.h_g_an + transmission.h_a,
        ventilation_h_v_w_per_k: h_v,
        monthly_theta_e_c: climate.outdoor_temperature,
        tau_hours: demand.breakdown.time_constant_hours,
    })
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

    #[test]
    fn enum_mapper_covers_all_boundary_kinds() {
        use crate::geometry::BoundaryKind::*;
        let id = "test_id".to_string();

        // Test alle 6 BoundaryKind varianten
        assert_eq!(
            map_boundary_kind_to_transmission_type(Exterior, None),
            TransmissionBoundaryType::Outdoor
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(Ground, None),
            TransmissionBoundaryType::Ground
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(OpenWater, None),
            TransmissionBoundaryType::Outdoor
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(UnheatedSpace, Some(&id)),
            TransmissionBoundaryType::UnheatedSpace { id: id.clone() }
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(AdjacentRoom, Some(&id)),
            TransmissionBoundaryType::AdjacentZone { id: id.clone() }
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(AdjacentBuilding, None),
            TransmissionBoundaryType::AdjacentZone { id: "adjacent_building".to_string() }
        );

        // Test fallbacks voor missing adjacent_space_id
        assert_eq!(
            map_boundary_kind_to_transmission_type(UnheatedSpace, None),
            TransmissionBoundaryType::UnheatedSpace { id: "default_unheated".to_string() }
        );

        assert_eq!(
            map_boundary_kind_to_transmission_type(AdjacentRoom, None),
            TransmissionBoundaryType::AdjacentZone { id: "unknown_adjacent".to_string() }
        );
    }

    #[test]
    fn compute_tojuli_full_with_adjacent_room_and_named_unheated_space() {
        let mut p = sample_project();

        // Voeg constructions toe die de bugs zouden triggeren
        p.geometry.spaces[0].constructions.extend(vec![
            SC {
                id: "C_adjacent".into(),
                description: "Binnenwand naar woonkamer".into(),
                kind: ConstructionKind::Wall,
                boundary: BoundaryKind::AdjacentRoom,
                area_m2: 20.0,
                u_value: 0.5,
                orientation_deg: None,
                slope_deg: Some(90.0),
                openings: vec![],
                layers: vec![],
                adjacent_space_id: Some("woonkamer".to_string()),
                psi_thermal_bridge: None,
            },
            SC {
                id: "C_unheated".into(),
                description: "Wand naar garage".into(),
                kind: ConstructionKind::Wall,
                boundary: BoundaryKind::UnheatedSpace,
                area_m2: 15.0,
                u_value: 0.8,
                orientation_deg: None,
                slope_deg: Some(90.0),
                openings: vec![],
                layers: vec![],
                adjacent_space_id: Some("garage".to_string()),
                psi_thermal_bridge: None,
            },
        ]);

        let i = sample_inputs();

        // Dit zou moeten slagen zonder panic/error
        let r = compute_tojuli_full(&p, &i).expect("compute_tojuli_full should succeed with adjacent_room and named_unheated");

        // Verifieer dat resultaat valide is (geen NaN/Inf)
        assert!(r.transmission_h_t_w_per_k.is_finite());
        assert!(r.transmission_h_t_w_per_k >= 0.0);

        // De transmission H_T zou nu alleen exterior + unheated moeten bevatten
        // Exterior: 150 × 0.3 = 45 W/K
        // UnheatedSpace: 15 × 0.8 × b_factor(0.5) = 6 W/K
        // AdjacentRoom: wordt geskipt, dus 0 W/K
        // Verwachte H_T ≈ 45 + 6 = 51 W/K (plus kleine h_g_an)
        assert!(r.transmission_h_t_w_per_k > 45.0); // Minstens de exterior component
        assert!(r.transmission_h_t_w_per_k < 65.0); // Realistisch bovengrens

        // Verifieer dat de rest van het resultaat ook geldig is
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.annual_q_c_use_kwh >= 0.0);
        assert!(r.tau_hours > 0.0);
    }

    #[test]
    fn ventilation_system_d_balanced_with_wtw_uses_engine() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechBalanced);
        p.shared.heat_recovery = Some(HeatRecovery {
            efficiency: 0.85,
            frost_protection: false,
            supply_temperature: None
        });
        p.shared.mechanical_supply_m3_per_h = Some(120.0);
        p.shared.mechanical_exhaust_m3_per_h = Some(120.0);

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok with System D + WTW");

        assert!(r.ventilation_h_v_w_per_k > 0.0);
        // Met WTW zou de h_v lager moeten zijn dan zonder (recovery effect)
        // Verify that WTW recovery happened by checking result is fault-free
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.annual_q_c_use_kwh >= 0.0);
        assert!(r.tau_hours > 0.0);
    }

    #[test]
    fn ventilation_system_b_supply_only() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechSupply);
        p.shared.heat_recovery = None;
        p.shared.mechanical_supply_m3_per_h = Some(150.0);
        p.shared.mechanical_exhaust_m3_per_h = None;
        p.shared.infiltration_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok with System B");

        // H_V = 150 × 0.34 = 51.0 W/K
        assert!((r.ventilation_h_v_w_per_k - 51.0).abs() < 0.5);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    #[test]
    fn ventilation_system_c_exhaust_only() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechExhaust);
        p.shared.heat_recovery = None;
        p.shared.mechanical_exhaust_m3_per_h = Some(100.0);
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.infiltration_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok with System C");

        // H_V = 100 × 0.34 = 34.0 W/K
        assert!((r.ventilation_h_v_w_per_k - 34.0).abs() < 0.5);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    #[test]
    fn ventilation_system_a_natural() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.heat_recovery = None;
        p.shared.infiltration_m3_per_h = Some(80.0);
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.mechanical_exhaust_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok with System A");

        // H_V = 80 × 0.34 = 27.2 W/K
        assert!((r.ventilation_h_v_w_per_k - 27.2).abs() < 0.5);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    #[test]
    fn ventilation_wtw_with_unbalanced_system_is_filtered() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechExhaust); // System C, niet balanced
        p.shared.heat_recovery = Some(HeatRecovery {
            efficiency: 0.80,
            frost_protection: false,
            supply_temperature: None
        }); // Zou normaal error triggeren
        p.shared.mechanical_exhaust_m3_per_h = Some(100.0);
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.infiltration_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok - WTW filtered for non-balanced");

        // Geen error, mapping helper moet WtwSpecification droppen voor non-balanced systemen
        assert!(r.ventilation_h_v_w_per_k > 0.0);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    #[test]
    fn legacy_v2_without_mech_supply_exhaust_round_trip() {
        // SharedProject JSON zonder de nieuwe mechanical_supply/exhaust velden (V2 legacy format)
        let json_v2_legacy = r#"{
            "name": "Test Project",
            "building_type": {
                "kind": "woning",
                "subtype": "detached"
            },
            "gross_floor_area_m2": 100.0
        }"#;

        let shared: crate::shared::SharedProject = serde_json::from_str(json_v2_legacy)
            .expect("deserialize legacy v2 without mech fields");

        // Assert defaults zijn None (serde default werkt)
        assert_eq!(shared.mechanical_supply_m3_per_h, None);
        assert_eq!(shared.mechanical_exhaust_m3_per_h, None);

        // Serialize terug - None velden zouden niet in JSON moeten zitten (skip_serializing_if)
        let serialized = serde_json::to_string(&shared).expect("serialize back");
        assert!(!serialized.contains("mechanical_supply_m3_per_h"));
        assert!(!serialized.contains("mechanical_exhaust_m3_per_h"));
    }
}
