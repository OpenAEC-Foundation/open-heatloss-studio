//! Rekenkundige kern: luchtstromen, warmteverlies, WTW-terugwinning, fan-energie.
//!
//! Alle publieke reken-entry's in deze module nemen input in m³/h (NTA 8800
//! eenheid) en produceren energie in MJ.

pub mod fan_energy;
pub mod infiltration;
pub mod monthly_heat_loss;
pub mod wtw_recovery;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{Energy, Temperature};
use nta8800_model::{ClimateData, Rekenzone};
use nta8800_tables::climate::de_bilt::DE_BILT_MONTH_LENGTHS_HOURS;

use crate::errors::VentilationError;
use crate::model::{AirFlow, VentilationSystem, WtwSpecification};
use crate::result::VentilationResult;

/// Luchtdichtheid ρ_a in kg/m³ — NTA 8800 formule (11.106) constante.
pub const AIR_DENSITY_KG_PER_M3: f64 = 1.205;

/// Specifieke warmtecapaciteit c_a in J/(kg·K) — NTA 8800 formule (11.106).
pub const AIR_SPECIFIC_HEAT_J_PER_KG_K: f64 = 1006.0;

/// Volumetrische warmtecapaciteit ρ_a·c_a in J/(m³·K).
///
/// Evalueert naar ≈ **1212,23 J/(m³·K)** — NTA 8800 hanteert deze waarde via
/// de losse factoren in formule (11.106). Wijkt licht af van de veelgebruikte
/// benadering 1 200 J/(m³·K) uit oudere literatuur; wij volgen de norm
/// exact om audit-traceability te behouden.
pub const AIR_VOLUMETRIC_HEAT_J_PER_M3_K: f64 =
    AIR_DENSITY_KG_PER_M3 * AIR_SPECIFIC_HEAT_J_PER_KG_K;

/// Bereken ventilatie-warmteverliezen en ventilator-energie voor één
/// [`Rekenzone`] conform NTA 8800:2025+C1:2026 H.11.
///
/// De maandduren `t_mi` komen uit `DE_BILT_MONTH_LENGTHS_HOURS` (NTA 8800
/// tabel 17.1). Bij latere klimaatzone-uitbreiding kunnen ze uit `climate`
/// worden afgeleid.
///
/// # Eenheden
///
/// | Grootheid | Input | Output |
/// |---|---|---|
/// | Luchtstromen | m³/h | — |
/// | Temperaturen | °C | — |
/// | Q_V (ventilatie-warmteverlies) | — | MJ |
/// | W_fan (ventilator-energie) | — | MJ elektrisch |
///
/// # Referenties
///
/// - Formule (11.106): `P = q · ρ_a · c_a · ΔT / 3600`
/// - Formule (11.107): WTW-temperatuursprong
/// - Formule (11.142): forfaitair ventilatorvermogen
///
/// # Errors
///
/// - [`VentilationError::InvalidWtwEfficiency`] bij η_hr buiten `[0, 1]`
/// - [`VentilationError::InvalidFanSfp`] bij f_SFP < 0
/// - [`VentilationError::NegativeAirFlow`] bij een negatieve luchtstroom
/// - [`VentilationError::WtwWithoutBalancedSystem`] als WTW opgegeven bij A/B/C
pub fn calculate_ventilation(
    _zone: &Rekenzone,
    system: &VentilationSystem,
    flow: &AirFlow,
    wtw: Option<&WtwSpecification>,
    indoor_temperature: &MonthlyProfile<Temperature>,
    climate: &ClimateData,
) -> Result<VentilationResult, VentilationError> {
    // --- input-validatie -------------------------------------------------
    if flow.mechanical_supply < 0.0 {
        return Err(VentilationError::NegativeAirFlow {
            name: "mechanical_supply",
            value: flow.mechanical_supply,
        });
    }
    if flow.mechanical_exhaust < 0.0 {
        return Err(VentilationError::NegativeAirFlow {
            name: "mechanical_exhaust",
            value: flow.mechanical_exhaust,
        });
    }
    if flow.infiltration < 0.0 {
        return Err(VentilationError::NegativeAirFlow {
            name: "infiltration",
            value: flow.infiltration,
        });
    }

    if let Some(wtw) = wtw {
        if !(0.0..=1.0).contains(&wtw.efficiency) {
            return Err(VentilationError::InvalidWtwEfficiency(wtw.efficiency));
        }
        if wtw.fan_sfp < 0.0 {
            return Err(VentilationError::InvalidFanSfp(wtw.fan_sfp));
        }
        if !system.is_balanced() {
            return Err(VentilationError::WtwWithoutBalancedSystem);
        }
    }

    // Effectieve WTW — alleen als het systeem D/E is én `with_wtw = true`
    // én een WTW-specificatie is aangeleverd.
    let effective_wtw = match system {
        VentilationSystem::D { with_wtw: true } | VentilationSystem::E => wtw,
        _ => None,
    };

    // --- per-maand-berekening -------------------------------------------
    let mut monthly_q_v = [0.0_f64; 12];
    let mut monthly_w_fan = [0.0_f64; 12];
    let mut monthly_wtw_recovery = [0.0_f64; 12];

    // Totale ventilatiestroom q_V;tot afhankelijk van systeem-topologie:
    // - Systeem A: alleen infiltratie (q_V;lea)
    // - Systeem B: max(mech_supply, infiltratie) — aanvoer dominant
    // - Systeem C: max(mech_exhaust, infiltratie) — afvoer dominant
    // - Systeem D/E: mech_supply (balans → supply bepaalt binnenkomende lucht)
    let q_v_total = system_total_airflow(*system, flow);

    // Mechanische stroom voor ventilator-energie berekening.
    // Per systeem-type:
    // - A → geen ventilator
    // - B → supply fan actief (mech_supply)
    // - C → extract fan actief (mech_exhaust)
    // - D/E → supply fan actief; exhaust wordt via f_systype=2 in
    //         ventilator-energie meegeteld. De norm gebruikt q_V;ODA;req
    //         (toevoerstroom) als basis voor W_fan — exhaust-fan energie
    //         zit in de f_systype=2 factor, niet in een apart debiet.
    let q_v_mech = match system {
        VentilationSystem::A => 0.0,
        VentilationSystem::B | VentilationSystem::D { .. } | VentilationSystem::E => {
            flow.mechanical_supply
        }
        VentilationSystem::C => flow.mechanical_exhaust,
    };

    for month in Month::all() {
        let theta_e = climate.outdoor_temperature[month];
        let theta_i = indoor_temperature[month];
        let t_mi = DE_BILT_MONTH_LENGTHS_HOURS[month]; // h

        // --- WTW toevoertemperatuur + recovery ------------------------
        let (theta_supply, wtw_recovered_mj) = if let Some(wtw) = effective_wtw {
            let (theta_sup, q_recovered) =
                wtw_recovery::apply_wtw(theta_e, theta_i, wtw.efficiency, q_v_total, t_mi);
            (theta_sup, q_recovered)
        } else {
            (theta_e, 0.0)
        };

        // --- Q_V ventilatie-warmteverlies ------------------------------
        let q_v_mj = monthly_heat_loss::heat_loss_mj(q_v_total, theta_i, theta_supply, t_mi);

        // --- W_fan ventilator-energie ----------------------------------
        let f_sfp = effective_wtw.map_or(0.0, |wtw| wtw.fan_sfp);
        let w_fan_mj = fan_energy::fan_energy_mj(f_sfp, system.f_systype(), q_v_mech, t_mi);

        monthly_q_v[month.index()] = q_v_mj;
        monthly_w_fan[month.index()] = w_fan_mj;
        monthly_wtw_recovery[month.index()] = wtw_recovered_mj;
    }

    let monthly_q_v = MonthlyProfile::new(monthly_q_v);
    let monthly_w_fan = MonthlyProfile::new(monthly_w_fan);
    let monthly_wtw_recovery = MonthlyProfile::new(monthly_wtw_recovery);

    Ok(VentilationResult {
        annual_q_v: sum_monthly(&monthly_q_v),
        annual_w_fan: sum_monthly(&monthly_w_fan),
        annual_wtw_recovery: sum_monthly(&monthly_wtw_recovery),
        monthly_q_v,
        monthly_w_fan,
        monthly_wtw_recovery,
    })
}

/// Bepaal de totale ventilatiestroom `q_V;tot` (m³/h) op basis van het
/// systeem-type en de opgegeven luchtstromen.
///
/// Pragmatische V1-heuristiek — de volledige massabalans uit §11.2.1.5
/// (stap 4, `p_z;ref` oplossen) is V2-scope.
fn system_total_airflow(system: VentilationSystem, flow: &AirFlow) -> f64 {
    match system {
        VentilationSystem::A => flow.infiltration,
        VentilationSystem::B => flow.mechanical_supply.max(flow.infiltration),
        VentilationSystem::C => flow.mechanical_exhaust.max(flow.infiltration),
        // Balans: mech. supply drijft de stroom in de rekenzone;
        // infiltratie telt niet dubbel op (over-/onderdruk is klein).
        VentilationSystem::D { .. } | VentilationSystem::E => flow.mechanical_supply,
    }
}

fn sum_monthly(p: &MonthlyProfile<Energy>) -> Energy {
    p.as_array().iter().sum()
}
