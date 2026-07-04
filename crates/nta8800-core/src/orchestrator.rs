//! Keten-orchestratie: [`crate::model::Project`] → [`crate::result::Nta8800Result`].
//!
//! Pipeline (zie [`crate::formulas`] voor de hoofdstuk-referenties):
//!
//! 1. Rekenzone + transmissie-elementen + vensters uit het invoer-model
//! 2. Transmissie H.8 (`nta8800-transmission`)
//! 3. Ventilatie H.11 (`nta8800-ventilation`) — met §11.2.2 `q_V;ODA;req`
//!    forfait wanneer luchtdebieten ontbreken
//! 4. Behoefte H.7 (`nta8800-demand`)
//! 5. Diensten: verwarming H.9, koeling H.10, tapwater H.13, verlichting
//!    H.14 (alleen utiliteit), PV H.16
//! 6. EP-score H.5 (`nta8800-ep`) → energielabel
//!
//! De orchestratie-aanpak (rekenzone-synthese, forfait-fallbacks, H_V-afleiding
//! voor de tijdconstante) volgt het gevalideerde patroon van de TO-juli-keten
//! in `openaec-project-shared::tojuli`, uitgebreid van koeling-only naar alle
//! diensten.

use std::collections::HashMap;

use nta8800_cooling::calculate_cooling;
use nta8800_demand::calc::calculate_demand;
use nta8800_demand::model::{
    setpoints::{CoolingSetpoint, HeatingSetpoint},
    InternalGains, ThermalMassInput,
};
use nta8800_dhw::calc::calculate_dhw;
use nta8800_dhw::model::{DhwDemand, DhwDistribution};
use nta8800_ep::{calculate_ep_score, BuildingArea, EnergyCarrier as EpCarrier, EpInputs};
use nta8800_heating::calc::calculate_heating;
use nta8800_heating::model::{ControlFactor, DistributionSystem};
use nta8800_lighting::calc::calculate_lighting;
use nta8800_model::geometry::window::Window as N8Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_pv::{calculate_pv_yield, PvLocation};
use nta8800_tables::climate::de_bilt_climate_data;
use nta8800_transmission::{
    calculate_transmission, BoundaryType as TransmissionBoundaryType, TransmissionElement,
};
use nta8800_ventilation::{
    calculate_ventilation,
    model::{AirFlow, VentilationSystem, WtwSpecification},
    system_total_airflow, AIR_VOLUMETRIC_HEAT_J_PER_M3_K,
};

use crate::error::CoreResult;
use crate::model::{
    Boundary, Project, ThermalMassClass, VentilationSystemInput, DEFAULT_STOREY_HEIGHT_M,
};
use crate::result::{
    DemandSummary, EpSummary, Nta8800Result, PerServicePrimary, PvSummary, ServiceSummary,
    VentilationSummary,
};

/// Forfaitair specifiek ventilator-vermogen (NTA 8800 tabel 11.23, moderne
/// DC-unit) in W per (m³/h) — zelfde waarde als de TO-juli-keten hanteert.
const VENTILATION_FAN_SFP_W_PER_M3H: f64 = 0.125;

/// NTA 8800 §8.3.1 forfaitair minimum voor de grond-gebonden conductance
/// `H_g;an` van een woning zonder gedetailleerde grond-invoer, in W/K.
/// Alleen toegepast wanneer het project minstens één `Ground`-element heeft.
const FORFAIT_H_G_AN_W_PER_K: f64 = 10.0;

/// Default b-factor voor onverwarmde aangrenzende ruimtes (NTA 8800 §8.4.1).
const DEFAULT_UNHEATED_B_FACTOR: f64 = 0.5;

/// De Bilt referentie-locatie (breedte-/lengtegraad) voor de PV-keten.
const DE_BILT_LAT: f64 = 52.1;
/// Lengtegraad De Bilt.
const DE_BILT_LON: f64 = 5.2;

/// Voer de volledige NTA 8800 keten uit op een gevalideerd project.
///
/// # Errors
///
/// [`crate::CoreError`] — façade-validatie of een fout uit één van de
/// onderliggende reken-crates.
#[allow(clippy::too_many_lines)]
pub fn calculate(project: &Project) -> CoreResult<Nta8800Result> {
    project.validate()?;

    let climate = de_bilt_climate_data();
    let usage = project.building.usage_function;
    let a_g = project.building.floor_area_m2;
    let volume = project.effective_volume_m3();

    // ---- 1. Rekenzone ----
    let zone = Rekenzone {
        id: "rz1".into(),
        name: if project.info.name.is_empty() {
            "hoofdzone".into()
        } else {
            project.info.name.clone()
        },
        gebouw_id: "g1".into(),
        floor_area: a_g,
        volume,
        efr_ids: Vec::new(),
        constructions: Vec::new(),
        windows: Vec::new(),
        openings: Vec::new(),
        thermal_bridges_linear: Vec::new(),
        thermal_bridges_point: Vec::new(),
    };

    // ---- 2. Transmissie-elementen + vensters ----
    let mut elements: Vec<TransmissionElement> = Vec::new();
    let mut n8_windows: Vec<N8Window> = Vec::new();
    let mut has_ground = false;
    let mut unheated_b: HashMap<String, f64> = HashMap::new();

    for el in &project.envelope {
        let boundary_type = map_boundary(&el.boundary);
        match &el.boundary {
            Boundary::Ground => has_ground = true,
            Boundary::UnheatedSpace { id } => {
                let key = id.clone().unwrap_or_else(|| "default_unheated".to_string());
                unheated_b.entry(key).or_insert(DEFAULT_UNHEATED_B_FACTOR);
            }
            Boundary::Exterior => {}
        }

        // Opaak deel = bruto-oppervlak minus vensters.
        let window_area: f64 = el.windows.iter().map(|w| w.area_m2).sum();
        let opaque_area = (el.area_m2 - window_area).max(0.0);
        if opaque_area > 0.0 {
            elements.push(TransmissionElement {
                id: format!("{}_opaque", el.id),
                area: opaque_area,
                u_value: el.u_value,
                boundary_type: map_boundary(&el.boundary),
                construction_id: Some(el.id.clone()),
            });
        }

        // Vensters: eigen transmissie-element met venster-U; zoninstraling
        // alleen voor exterior-vlakken.
        for w in &el.windows {
            elements.push(TransmissionElement {
                id: w.id.clone(),
                area: w.area_m2,
                u_value: w.u_value,
                boundary_type: boundary_type.clone(),
                construction_id: Some(el.id.clone()),
            });
            if matches!(el.boundary, Boundary::Exterior) {
                let orientation = el
                    .orientation_deg
                    .map_or(Orientation::Horizontaal, orientation_from_degrees);
                let tilt = Tilt::new(el.tilt_deg.unwrap_or(90.0))?;
                n8_windows.push(N8Window {
                    id: w.id.clone(),
                    construction_id: el.id.clone(),
                    area: w.area_m2,
                    orientation,
                    tilt,
                    u_value: w.u_value,
                    g_value: w.g_value,
                    frame_fraction: w.frame_fraction,
                });
            }
        }
    }
    unheated_b
        .entry("default_unheated".to_string())
        .or_insert(DEFAULT_UNHEATED_B_FACTOR);

    // ---- 3. Transmissie H.8 ----
    let indoor_temperature =
        MonthlyProfile::from_constant(project.conditions.heating_setpoint_c);
    let thermal_bridges_linear = Vec::new(); // Forfaitair 0 (NTA §7.3.3)
    let thermal_bridges_point = Vec::new(); // Forfaitair 0 (NTA §7.3.3)
    let h_g_an = if has_ground { FORFAIT_H_G_AN_W_PER_K } else { 0.0 };
    let adjacent_zone_temperatures: HashMap<String, MonthlyProfile<f64>> = HashMap::new();

    let transmission = calculate_transmission(
        &zone,
        &elements,
        &thermal_bridges_linear,
        &thermal_bridges_point,
        &indoor_temperature,
        &climate,
        h_g_an,
        &unheated_b,
        &adjacent_zone_temperatures,
    )?;
    let h_tr = transmission.h_d + transmission.h_u + transmission.h_g_an + transmission.h_a;

    // ---- 4. Ventilatie H.11 ----
    let (system, mut flow, wtw) = map_ventilation(project);

    // §11.2.2 forfait wanneer debieten ontbreken (zelfde beslislogica als de
    // TO-juli-keten): systeem A / onbekend zonder infiltratie-invoer krijgt
    // q_V;ODA;req als natuurlijke toevoer; mech-systemen zonder debiet
    // krijgen q_V;ODA;req als mechanisch debiet.
    let q_oda_req = q_v_oda_req_m3_per_h(usage, a_g.max(volume / DEFAULT_STOREY_HEIGHT_M));
    match system {
        VentilationSystem::A => {
            if project.ventilation.infiltration_m3_per_h.is_none() {
                flow.infiltration = q_oda_req;
            }
        }
        VentilationSystem::B | VentilationSystem::D { .. } | VentilationSystem::E => {
            if project.ventilation.mechanical_supply_m3_per_h.is_none()
                && flow.mechanical_supply == 0.0
            {
                flow.mechanical_supply = q_oda_req;
            }
        }
        VentilationSystem::C => {
            if project.ventilation.mechanical_exhaust_m3_per_h.is_none()
                && flow.mechanical_exhaust == 0.0
            {
                flow.mechanical_exhaust = q_oda_req;
            }
        }
    }

    let ventilation = calculate_ventilation(
        &zone,
        &system,
        &flow,
        wtw.as_ref(),
        &indoor_temperature,
        &climate,
    )?;

    // Ventilator-hulpenergie voor systeem B/C — workaround voor een gap in
    // `nta8800-ventilation` V1: de fan-SFP komt daar uitsluitend uit de
    // `WtwSpecification`, die alleen bij gebalanceerde systemen (D/E) is
    // toegestaan. Systeem B/C hebben echter wél een ventilator
    // (f_systype = 1, NTA 8800 §11.4.3.3 formule 11.142). We berekenen
    // W_fan hier met dezelfde norm-formule en het tabel-11.23 forfait-SFP.
    let w_fan_supplement_mj: f64 = match system {
        VentilationSystem::B | VentilationSystem::C => {
            let q_v_mech = match system {
                VentilationSystem::B => flow.mechanical_supply,
                _ => flow.mechanical_exhaust,
            };
            // P_eff = f_SFP × f_systype × q_V; jaar = P × 8760 h × 3600 / 10⁶
            let p_eff_w = VENTILATION_FAN_SFP_W_PER_M3H * 1.0 * q_v_mech;
            p_eff_w * 8760.0 * 3600.0 / 1.0e6
        }
        _ => 0.0,
    };
    let annual_w_fan_total = ventilation.annual_w_fan + w_fan_supplement_mj;

    // H_V (W/K) voor de tijdconstante τ — systeem-bewuste q_V;tot × ρ·c/3600,
    // met WTW-reductiefactor (1 − η). Q_V zelf komt uit de engine-output.
    let q_v_total = system_total_airflow(system, &flow);
    let wtw_factor = wtw.as_ref().map_or(1.0, |w| 1.0 - w.efficiency);
    let h_ve = q_v_total * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0 * wtw_factor;

    // ---- 5. Behoefte H.7 ----
    let internal_gains = InternalGains::forfaitair(usage);
    let thermal_mass = match project.building.thermal_mass {
        ThermalMassClass::Light => ThermalMassInput::light_woning(),
        ThermalMassClass::Heavy => ThermalMassInput::zwaar_massief(),
    };
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(
        project.conditions.heating_setpoint_c,
    ));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(
        project.conditions.cooling_setpoint_c,
    ));
    let window_refs: Vec<&N8Window> = n8_windows.iter().collect();

    let demand = calculate_demand(
        &zone,
        &transmission,
        &ventilation,
        h_ve,
        &window_refs,
        &climate,
        heating_sp,
        cooling_sp,
        &internal_gains,
        thermal_mass,
        project.conditions.shading_factor,
    )?;

    // ---- 6a. Verwarming H.9 ----
    let distribution = DistributionSystem::custom(project.heating.distribution_efficiency)?;
    let control = ControlFactor::custom(project.heating.control_factor)?;
    let heating = calculate_heating(
        &demand,
        project.heating.emission,
        &distribution,
        &project.heating.generation,
        control,
    )?;

    // ---- 6b. Koeling H.10 (optioneel) ----
    let cooling = match &project.cooling {
        Some(c) => Some(calculate_cooling(
            &zone,
            &demand,
            &c.system,
            &c.distribution,
            &c.emission,
        )?),
        None => None,
    };

    // ---- 6c. Warm tapwater H.13 ----
    // Expliciete jaarbehoefte gaat vóór het forfait; verplicht voor
    // functies zonder tabel-13.1-forfait (industriefunctie).
    let dhw_demand = match project.dhw.annual_demand_kwh {
        Some(kwh) => DhwDemand::from_annual_even(kwh * 3.6),
        None => match usage {
            UsageFunction::Woonfunctie => DhwDemand::forfaitair_woningbouw(a_g)?,
            other => DhwDemand::forfaitair_utiliteit(other, a_g)?,
        },
    };
    let dhw_distribution = DhwDistribution {
        efficiency: project.dhw.distribution_efficiency,
    };
    let dhw = calculate_dhw(
        &zone,
        &dhw_demand,
        &project.dhw.emission,
        &dhw_distribution,
        &project.dhw.generation,
        project.dhw.shower_heat_recovery.as_ref(),
    )?;

    // ---- 6d. Verlichting H.14 (alleen utiliteit) ----
    let lighting = if usage == UsageFunction::Woonfunctie {
        None
    } else {
        match &project.lighting {
            Some(sys) => Some(calculate_lighting(&zone, sys)?),
            None => None,
        }
    };

    // ---- 6e. PV H.16 (optioneel) ----
    let pv = match project.pv.as_ref().filter(|v| !v.is_empty()) {
        Some(systems) => {
            let location = PvLocation::new(DE_BILT_LAT, DE_BILT_LON)?;
            Some(calculate_pv_yield(systems, &location, &climate)?)
        }
        None => None,
    };

    // ---- 7. EP-score H.5 ----
    let mut ep_heating = HashMap::new();
    ep_heating.insert(
        map_heating_carrier(heating.energy_carrier),
        heating.annual_q_h_use,
    );
    let mut ep_cooling = HashMap::new();
    if let Some(c) = &cooling {
        ep_cooling.insert(map_cooling_carrier(c.energy_carrier), c.annual_q_c_use);
    }
    let mut ep_dhw = HashMap::new();
    ep_dhw.insert(map_dhw_carrier(dhw.energy_carrier), dhw.annual_q_w_use);
    let mut ep_lighting = HashMap::new();
    if let Some(l) = &lighting {
        ep_lighting.insert(EpCarrier::Elektriciteit, l.annual_w_l_use);
    }
    let mut ep_vent_aux = HashMap::new();
    if annual_w_fan_total > 0.0 {
        ep_vent_aux.insert(EpCarrier::Elektriciteit, annual_w_fan_total);
    }

    let ep_inputs = EpInputs {
        heating: ep_heating,
        cooling: ep_cooling,
        dhw: ep_dhw,
        lighting: ep_lighting,
        ventilation_aux: ep_vent_aux,
        automation: HashMap::new(), // V1: geen BACS-energie
        pv_yield: pv.as_ref().map_or(0.0, |p| p.annual_yield_mj),
        building_area: BuildingArea { a_g },
    };
    let ep = calculate_ep_score(&ep_inputs, usage)?;

    // ---- 8. Resultaat samenstellen ----
    Ok(Nta8800Result {
        demand: DemandSummary {
            monthly_q_h_nd_mj: monthly_to_array(&demand.monthly_heating_demand),
            monthly_q_c_nd_mj: monthly_to_array(&demand.monthly_cooling_demand),
            annual_q_h_nd_mj: demand.annual_heating_demand,
            annual_q_c_nd_mj: demand.annual_cooling_demand,
            h_tr_w_per_k: h_tr,
            h_ve_w_per_k: h_ve,
            tau_hours: demand.breakdown.time_constant_hours,
        },
        heating: ServiceSummary {
            energy_carrier: heating_carrier_name(heating.energy_carrier).to_string(),
            monthly_use_mj: monthly_to_array(&heating.monthly_q_h_use),
            annual_use_mj: heating.annual_q_h_use,
            annual_use_kwh: heating.annual_q_h_use / 3.6,
            total_efficiency: Some(heating.breakdown.total_efficiency),
        },
        cooling: cooling.map(|c| ServiceSummary {
            energy_carrier: cooling_carrier_name(c.energy_carrier).to_string(),
            monthly_use_mj: monthly_to_array(&c.monthly_q_c_use),
            annual_use_mj: c.annual_q_c_use,
            annual_use_kwh: c.annual_q_c_use / 3.6,
            total_efficiency: None,
        }),
        dhw: ServiceSummary {
            energy_carrier: dhw_carrier_name(dhw.energy_carrier).to_string(),
            monthly_use_mj: monthly_to_array(&dhw.monthly_q_w_use),
            annual_use_mj: dhw.annual_q_w_use,
            annual_use_kwh: dhw.annual_q_w_use / 3.6,
            total_efficiency: None,
        },
        lighting: lighting.map(|l| ServiceSummary {
            energy_carrier: "elektriciteit".to_string(),
            monthly_use_mj: monthly_to_array(&l.monthly_w_l_use),
            annual_use_mj: l.annual_w_l_use,
            annual_use_kwh: l.annual_w_l_use / 3.6,
            total_efficiency: None,
        }),
        ventilation: VentilationSummary {
            annual_q_v_mj: ventilation.annual_q_v,
            annual_w_fan_mj: annual_w_fan_total,
            annual_wtw_recovery_mj: ventilation.annual_wtw_recovery,
        },
        pv: pv.map(|p| PvSummary {
            monthly_yield_mj: monthly_to_array(&p.monthly_yield_mj),
            annual_yield_mj: p.annual_yield_mj,
            annual_yield_kwh: p.annual_yield_mj / 3.6,
        }),
        ep: EpSummary {
            label: ep.ep_label.as_str().to_string(),
            primary_energy_mj: ep.ep_total_mj,
            primary_energy_mj_per_m2: ep.ep_total_mj_per_m2,
            primary_energy_kwh_per_m2: ep.ep_total_mj_per_m2 / 3.6,
            renewable_share: ep.ep_renewable_share,
            co2_kg_per_m2: ep.ep_co2_kg_per_m2,
            per_service_primary_mj: PerServicePrimary {
                heating: ep.breakdown.heating.primary_energy_mj,
                cooling: ep.breakdown.cooling.primary_energy_mj,
                dhw: ep.breakdown.dhw.primary_energy_mj,
                lighting: ep.breakdown.lighting.primary_energy_mj,
                ventilation_aux: ep.breakdown.ventilation_aux.primary_energy_mj,
                pv: ep.breakdown.pv.primary_energy_mj,
            },
        },
        beng: build_beng_summary(
            usage,
            a_g,
            demand.annual_heating_demand,
            demand.annual_cooling_demand,
            ep.ep_total_mj_per_m2,
            ep.ep_renewable_share,
        ),
    })
}

/// Stel de BENG 1/2/3-samenvatting op.
///
/// - BENG 1 = `(Q_H;nd + Q_C;nd) / A_g` in kWh/(m²·jaar)
/// - BENG 2 = `E_P;tot / A_g` in kWh/(m²·jaar)
/// - BENG 3 = hernieuwbaar aandeel × 100 in %
///
/// De indicatieve grenzen per gebruiksfunctie zijn consistent met de
/// OpenAEC open-energy-studio referentie-implementatie; de formele
/// compactheids-correctie op BENG 1 is V2 (zie [`crate::result::BengSummary`]).
fn build_beng_summary(
    usage: UsageFunction,
    a_g: f64,
    annual_q_h_nd_mj: f64,
    annual_q_c_nd_mj: f64,
    ep_total_mj_per_m2: f64,
    renewable_share: f64,
) -> crate::result::BengSummary {
    let (beng1_limit, beng2_limit, beng3_limit) = beng_limits(usage);
    let beng1 = (annual_q_h_nd_mj + annual_q_c_nd_mj) / 3.6 / a_g;
    let beng2 = ep_total_mj_per_m2 / 3.6;
    let beng3 = renewable_share * 100.0;
    crate::result::BengSummary {
        beng1_kwh_per_m2: beng1,
        beng2_kwh_per_m2: beng2,
        beng3_pct: beng3,
        beng1_limit,
        beng2_limit,
        beng3_limit,
        beng1_pass: beng1 <= beng1_limit,
        beng2_pass: beng2 <= beng2_limit,
        beng3_pass: beng3 >= beng3_limit,
    }
}

/// Indicatieve BENG-grenzen `(beng1 ≤, beng2 ≤, beng3 ≥)` per
/// gebruiksfunctie — waarden consistent met de open-energy-studio
/// referentie-tabel (woningen 70/25/50, kantoor 50/40/50, onderwijs
/// 70/60/50, gezondheidszorg 120/80/50, winkel 70/60/50, industrie en
/// overig 100/80/50).
fn beng_limits(usage: UsageFunction) -> (f64, f64, f64) {
    use UsageFunction as UF;
    match usage {
        UF::Woonfunctie => (70.0, 25.0, 50.0),
        UF::Kantoorfunctie => (50.0, 40.0, 50.0),
        UF::Onderwijsfunctie => (70.0, 60.0, 50.0),
        UF::Gezondheidszorgfunctie => (120.0, 80.0, 50.0),
        UF::Winkelfunctie | UF::Bijeenkomstfunctie => (70.0, 60.0, 50.0),
        UF::Logiesfunctie | UF::Celfunctie | UF::Sportfunctie => (100.0, 80.0, 50.0),
        UF::Industriefunctie | UF::OverigeGebruiksfunctie => (100.0, 80.0, 50.0),
    }
}

// ---------------------------------------------------------------------------
// Mappers
// ---------------------------------------------------------------------------

/// Map het façade-boundary-type naar het transmissie-boundary-type.
fn map_boundary(boundary: &Boundary) -> TransmissionBoundaryType {
    match boundary {
        Boundary::Exterior => TransmissionBoundaryType::Outdoor,
        Boundary::Ground => TransmissionBoundaryType::Ground,
        Boundary::UnheatedSpace { id } => TransmissionBoundaryType::UnheatedSpace {
            id: id.clone().unwrap_or_else(|| "default_unheated".to_string()),
        },
    }
}

/// Map graden (0 = noord, kloksgewijs) naar de dichtstbijzijnde 45°-sector.
fn orientation_from_degrees(deg: f64) -> Orientation {
    let normalized = deg.rem_euclid(360.0);
    // rem_euclid garandeert [0, 360) → (normalized+22.5)/45 ∈ [0.5, 8.5),
    // floor ∈ {0..8}: cast is verlies-vrij.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let sector = ((normalized + 22.5) / 45.0).floor() as usize % 8;
    match sector {
        0 => Orientation::Noord,
        1 => Orientation::NoordOost,
        2 => Orientation::Oost,
        3 => Orientation::ZuidOost,
        4 => Orientation::Zuid,
        5 => Orientation::ZuidWest,
        6 => Orientation::West,
        _ => Orientation::NoordWest,
    }
}

/// Map de ventilatie-invoer naar engine-typen.
fn map_ventilation(
    project: &Project,
) -> (VentilationSystem, AirFlow, Option<WtwSpecification>) {
    let v = &project.ventilation;
    let (system, wtw) = match v.system {
        VentilationSystemInput::Natural => (VentilationSystem::A, None),
        VentilationSystemInput::MechanicalSupply => (VentilationSystem::B, None),
        VentilationSystemInput::MechanicalExhaust => (VentilationSystem::C, None),
        VentilationSystemInput::Balanced { wtw_efficiency } => {
            let with_wtw = wtw_efficiency.is_some();
            let wtw = wtw_efficiency.map(|eta| WtwSpecification {
                efficiency: eta,
                fan_sfp: VENTILATION_FAN_SFP_W_PER_M3H,
                bypass_enabled: false,
            });
            (VentilationSystem::D { with_wtw }, wtw)
        }
    };
    let flow = AirFlow {
        mechanical_supply: v.mechanical_supply_m3_per_h.unwrap_or(0.0),
        mechanical_exhaust: v.mechanical_exhaust_m3_per_h.unwrap_or(0.0),
        infiltration: v.infiltration_m3_per_h.unwrap_or(0.0),
    };
    (system, flow, wtw)
}

/// Zet een [`MonthlyProfile`] om naar een plat `[f64; 12]` array.
fn monthly_to_array(profile: &MonthlyProfile<f64>) -> [f64; 12] {
    let mut out = [0.0_f64; 12];
    for month in Month::all() {
        out[month.index()] = profile[month];
    }
    out
}

fn map_heating_carrier(c: nta8800_heating::model::EnergyCarrier) -> EpCarrier {
    use nta8800_heating::model::EnergyCarrier as HC;
    match c {
        HC::Gas => EpCarrier::Aardgas,
        HC::Electricity => EpCarrier::Elektriciteit,
        HC::DistrictHeat => EpCarrier::Stadswarmte,
    }
}

fn heating_carrier_name(c: nta8800_heating::model::EnergyCarrier) -> &'static str {
    use nta8800_heating::model::EnergyCarrier as HC;
    match c {
        HC::Gas => "aardgas",
        HC::Electricity => "elektriciteit",
        HC::DistrictHeat => "stadswarmte",
    }
}

fn map_dhw_carrier(c: nta8800_dhw::model::EnergyCarrier) -> EpCarrier {
    use nta8800_dhw::model::EnergyCarrier as DC;
    match c {
        DC::Gas => EpCarrier::Aardgas,
        DC::Electricity => EpCarrier::Elektriciteit,
        DC::DistrictHeat => EpCarrier::Stadswarmte,
    }
}

fn dhw_carrier_name(c: nta8800_dhw::model::EnergyCarrier) -> &'static str {
    use nta8800_dhw::model::EnergyCarrier as DC;
    match c {
        DC::Gas => "aardgas",
        DC::Electricity => "elektriciteit",
        DC::DistrictHeat => "stadswarmte",
    }
}

fn map_cooling_carrier(c: nta8800_cooling::EnergyCarrier) -> EpCarrier {
    use nta8800_cooling::EnergyCarrier as CC;
    match c {
        CC::Electricity => EpCarrier::Elektriciteit,
        CC::Gas => EpCarrier::Aardgas,
        // EP kent geen aparte stadskoude-drager; stadswarmte is de dichtst-
        // bijzijnde primaire-energie-classificatie (extern warmte-/koudenet).
        CC::DistrictCold => EpCarrier::Stadswarmte,
    }
}

fn cooling_carrier_name(c: nta8800_cooling::EnergyCarrier) -> &'static str {
    use nta8800_cooling::EnergyCarrier as CC;
    match c {
        CC::Electricity => "elektriciteit",
        CC::Gas => "aardgas",
        CC::DistrictCold => "stadskoude",
    }
}

// ---------------------------------------------------------------------------
// NTA 8800 §11.2.2 forfait — q_V;ODA;req
// ---------------------------------------------------------------------------

/// NTA 8800 §11.2.2.1.1, formule (11.22): praktijkprestatiefactor `f_prac;req`.
const NTA_F_PRAC_REQ: f64 = 0.95;
/// NTA 8800 tabel 11.9: `f_lea;du` voor luchtdichtheidsklasse "Onbekend".
const NTA_F_LEA_DU_UNKNOWN: f64 = 1.10;
/// §11.2.2.4.1: omrekenfactor dm³/s → m³/h.
const NTA_DM3S_TO_M3H: f64 = 3.6;
/// §11.2.2.5.1, formule (11.63): woning-ondergrens `(q_usi;spec · A_g) ≥ 35 dm³/s`.
const NTA_WONING_MIN_CAPACITY_DM3S: f64 = 35.0;

/// Tabel 11.8 — `q_usi;spec` (dm³/(s·m²)) en `f_τ` per gebruiksfunctie.
///
/// Voor de woonfunctie is `f_τ` oppervlakte-afhankelijk:
/// `f_τ = min[(0,38 + A_g · 0,006); 0,8]`. Voor functies zonder eigen
/// kolomwaarde (industrie, overige) geldt de kantoor-rij als traceerbare
/// conservatieve default — zelfde keuze als de TO-juli-keten.
fn usi_spec_and_f_tau(usage: UsageFunction, a_g: f64) -> (f64, f64) {
    use UsageFunction as UF;
    // Gezondheidszorg heeft in tabel 11.8 toevallig dezelfde rij-waarden als
    // kantoor, maar is een eigen tabel-regel — bewust een aparte arm zodat
    // een toekomstige norm-wijziging maar één plek raakt.
    #[allow(clippy::match_same_arms)]
    match usage {
        UF::Woonfunctie => {
            let f_tau = (0.38 + a_g * 0.006).min(0.8);
            (0.50, f_tau)
        }
        UF::Bijeenkomstfunctie => (1.71, 0.15),
        UF::Celfunctie => (0.84, 0.80),
        UF::Gezondheidszorgfunctie => (1.11, 0.30),
        UF::Industriefunctie | UF::Kantoorfunctie | UF::OverigeGebruiksfunctie => (1.11, 0.30),
        UF::Logiesfunctie => (0.84, 0.40),
        UF::Onderwijsfunctie => (3.64, 0.30),
        UF::Sportfunctie => (0.46, 0.30),
        UF::Winkelfunctie => (0.28, 0.40),
    }
}

/// Norm-forfait `q_V;ODA;req` in m³/h (§11.2.2, formules 11.22/11.56/11.57/
/// 11.63 + tabel 11.8) — de benodigde luchtvolumestroom van buitenlucht
/// wanneer geen luchtdebieten zijn ingevoerd.
fn q_v_oda_req_m3_per_h(usage: UsageFunction, a_g: f64) -> f64 {
    let a_g = a_g.max(0.0);
    let (q_usi_spec, f_tau) = usi_spec_and_f_tau(usage, a_g);

    // (11.63) — woning-ondergrens op de absolute capaciteit.
    let mut capacity_dm3s = q_usi_spec * a_g;
    if matches!(usage, UsageFunction::Woonfunctie) {
        capacity_dm3s = capacity_dm3s.max(NTA_WONING_MIN_CAPACITY_DM3S);
    }

    // (11.56)/(11.57) — geïnstalleerde capaciteit onbekend → reken-waarde.
    let q_des_m3h = NTA_F_LEA_DU_UNKNOWN * f_tau * capacity_dm3s * NTA_DM3S_TO_M3H;

    // (11.22) — f_ctrl = f_sys = 1 (forfait-tak), ε_V = 1.
    q_des_m3h / NTA_F_PRAC_REQ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_sectors() {
        assert_eq!(orientation_from_degrees(0.0), Orientation::Noord);
        assert_eq!(orientation_from_degrees(90.0), Orientation::Oost);
        assert_eq!(orientation_from_degrees(180.0), Orientation::Zuid);
        assert_eq!(orientation_from_degrees(270.0), Orientation::West);
        assert_eq!(orientation_from_degrees(359.0), Orientation::Noord);
        assert_eq!(orientation_from_degrees(-90.0), Orientation::West);
    }

    #[test]
    fn q_v_oda_req_woning_120m2() {
        // f_τ = min(0,38 + 120·0,006; 0,8) = 0,8; cap = max(60; 35) = 60
        // q_des = 1,10 · 0,8 · 60 · 3,6 = 190,08; /0,95 = 200,08
        let q = q_v_oda_req_m3_per_h(UsageFunction::Woonfunctie, 120.0);
        assert!((q - 200.084_210_526).abs() < 1e-6);
    }

    #[test]
    fn q_v_oda_req_woning_ondergrens() {
        // Kleine woning 40 m²: cap = max(20; 35) = 35 dm³/s.
        let q = q_v_oda_req_m3_per_h(UsageFunction::Woonfunctie, 40.0);
        let f_tau = 0.38 + 40.0 * 0.006;
        let expected = 1.10 * f_tau * 35.0 * 3.6 / 0.95;
        assert!((q - expected).abs() < 1e-9);
    }
}
