//! Rekenkundige kern: luchtstromen, warmteverlies, WTW-terugwinning, fan-energie.
//!
//! Alle publieke reken-entry's in deze module nemen input in m³/h (NTA 8800
//! eenheid) en produceren energie in MJ.

pub mod fan_energy;
pub mod infiltration;
pub mod monthly_heat_loss;
pub mod pressure_solver;
pub mod wtw_recovery;

use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{Energy, Temperature};
use nta8800_model::{ClimateData, Rekenzone};
use nta8800_tables::climate::de_bilt::DE_BILT_MONTH_LENGTHS_HOURS;

/// Forfaitair specifiek ventilator-vermogen `f_SFP` in W/(m³/h) voor
/// mechanische systemen zonder expliciete specificatie (NTA 8800
/// tabel 11.23, moderne DC-unit). Zonder dit forfait zouden systeem
/// B/C (en D/E zonder WTW-spec) een ventilator-energie van 0 krijgen,
/// terwijl `f_systype` (§11.4.3.3) voor die systemen wel degelijk een
/// draaiende ventilator voorschrijft.
pub const FORFAIT_FAN_SFP_W_PER_M3H: f64 = 0.125;

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
        // f_SFP: uit de WTW-spec wanneer aanwezig; anders het tabel-11.23
        // forfait voor elk systeem met een ventilator (f_systype > 0).
        // Systeem A (f_systype = 0) nult de term sowieso uit.
        let f_sfp = effective_wtw.map_or_else(
            || {
                if system.f_systype() > 0.0 {
                    FORFAIT_FAN_SFP_W_PER_M3H
                } else {
                    0.0
                }
            },
            |wtw| wtw.fan_sfp,
        );
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

/// Bereken ventilatie-warmteverliezen en ventilator-energie met de
/// **norm-exacte massabalans** uit NTA 8800 §11.2.1.5/§11.2.1.6 i.p.v. de
/// [`system_total_airflow`]-heuristiek.
///
/// Deze variant lost per maand de interne referentiedruk `p_z;ref` op
/// ([`pressure_solver::solve_zone_airflow`]) en gebruikt de daaruit volgende
/// effectieve luchtvolumestroom — de som van mechanische toevoer, ingaande
/// natuurlijke ventilatie en ingaande infiltratie ([`ZoneAirflowSolution`]) —
/// als `q_V;tot` voor het ventilatie-warmteverlies (formule (11.106)). Bij een
/// gebalanceerd systeem (D/E) wordt zo de mechanische onbalans (supply ≠
/// exhaust) en de stack-/wind-gedreven infiltratie norm-exact verrekend i.p.v.
/// `q_V;tot = q_V;SUP;eff` te benaderen.
///
/// De bestaande [`calculate_ventilation`] (oude signatuur, heuristiek) blijft
/// ongewijzigd bestaan voor backward-compat en voor consumers die geen
/// [`BuildingPressureContext`] kunnen leveren.
///
/// # C2-scope
///
/// `ctx.building_height_m` moet `< 15 m` zijn (één luchtstroomzone). De
/// scope-toets ([`BuildingPressureContext::within_c2_scope`]) hoort vóór de
/// aanroep te gebeuren; een gebouw `≥ 15 m` levert nog steeds een resultaat
/// (gedocumenteerde 1-zone-benadering, zie [`pressure_solver::build_openings`])
/// maar dat is geen norm-conforme multi-zone-berekening.
///
/// # Verschil met `calculate_ventilation`
///
/// | Aspect | `calculate_ventilation` | deze functie |
/// |---|---|---|
/// | `q_V;tot` | `system_total_airflow`-heuristiek | massabalans `p_z;ref` (§11.2.1.6) |
/// | Infiltratie | losse `flow.infiltration`-scalar | stack/wind-gedreven uit drukmodel |
/// | Onbalans D/E | genegeerd (`q_V;tot = supply`) | norm-exact verrekend |
/// | Ventilator-energie | gelijk | gelijk (drukonafhankelijk debiet) |
/// | WTW-recovery | gelijk | gelijk |
///
/// # WTW-toevoertemperatuur
///
/// De massabalans gebruikt — net als de geïsoleerde solver — de luchtdichtheid
/// bij de buitentemperatuur voor de mechanische toevoer. De WTW verwarmt de
/// toevoerlucht (en verlaagt dus de dichtheid van `q_V;SUP;eff`); dat effect op
/// `p_z;ref` is een tweede-orde-verfijning en blijft V2-scope. De `wtw`-
/// parameter wordt — net als in [`calculate_ventilation`] — wél gebruikt voor
/// de WTW-recovery en de toevoertemperatuur ϑ_sup in het warmteverlies.
///
/// # Systeem A — caller-contract
///
/// Bij systeem A (natuurlijke ventilatie) modelleert het drukmodel de
/// natuurlijke ventilatie-openingen via een conductantie `C_vent = q_V;ODA;req`
/// ([`pressure_solver::build_openings`], §11.2.2.2.1). `build_openings` leest
/// die `q_V;ODA;req` af uit de **mechanische** debietvelden van [`AirFlow`]
/// (`flow.mechanical_supply` / `flow.mechanical_exhaust`) — systeem A heeft per
/// definitie geen mechanisch debiet, dus die velden zijn hier het transport-
/// kanaal voor de norm-bepaalde natuurlijke toevoer.
///
/// **Contract:** de caller MOET voor systeem A `flow.mechanical_supply` én
/// `flow.mechanical_exhaust` vullen met de norm-conforme `q_V;ODA;req`
/// (NTA 8800 §11.2.2). Doet de caller dat niet, dan krijgt `build_openings`
/// een `C_vent = 0`-openings-set: de natuurlijke ventilatie-conductantie
/// vervalt en de massabalans levert dan alléén infiltratie via de
/// lek-conductantie.
///
/// `0` is een **geldige** waarde voor die velden: dat representeert een
/// volledig dichte schil of een nog niet uitgewerkt ventilatie-ontwerp —
/// de massabalans loopt dan zuiver op de lek-conductantie `C_lea`. Er is
/// daarom bewust géén `debug_assert!` op `mechanical_supply > 0`; een
/// nul-waarde mag niet paniekeren.
///
/// # Errors
///
/// Naast de errors van [`calculate_ventilation`]:
/// - [`VentilationError::PressureSolverDidNotConverge`] als de iteratieve
///   `p_z;ref`-routine niet binnen de cap convergeert.
///
/// Referentie: NTA 8800:2025+C1:2026 §11.2.1.5/§11.2.1.6/§11.2.1.7,
/// PDF p. 440-447.
#[allow(clippy::too_many_arguments)]
pub fn calculate_ventilation_with_pressure_model(
    _zone: &Rekenzone,
    system: &VentilationSystem,
    flow: &AirFlow,
    wtw: Option<&WtwSpecification>,
    pressure_context: &crate::model::BuildingPressureContext,
    indoor_temperature: &MonthlyProfile<Temperature>,
    climate: &ClimateData,
) -> Result<VentilationResult, VentilationError> {
    // --- input-validatie (gelijk aan calculate_ventilation) --------------
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

    // Effectieve WTW — alleen D/E mét `with_wtw = true` + aangeleverde spec.
    let effective_wtw = match system {
        VentilationSystem::D { with_wtw: true } | VentilationSystem::E => wtw,
        _ => None,
    };

    // --- per-maand-berekening met massabalans ----------------------------
    let mut monthly_q_v = [0.0_f64; 12];
    let mut monthly_w_fan = [0.0_f64; 12];
    let mut monthly_wtw_recovery = [0.0_f64; 12];

    // Mechanische stroom voor ventilator-energie — drukonafhankelijk, dus
    // identiek aan calculate_ventilation (W_fan rekent met q_V;ODA;req, niet
    // met de massabalans-`q_V;tot`).
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

        // --- Massabalans (§11.2.1.6) → q_V;tot voor deze maand -----------
        // De solver levert de effectieve in-/uitgaande debieten; `q_V;tot`
        // voor het warmteverlies is de totale verse-lucht-toevoer over de
        // gebouwschil (mechanische toevoer + ingaande natuurlijke ventilatie
        // + ingaande infiltratie, formule (11.20)).
        let solution = pressure_solver::solve_zone_airflow(
            *system,
            flow,
            wtw,
            pressure_context,
            theta_e,
            theta_i,
            month,
        )?;
        let q_v_total = solution.total_inflow();

        // --- WTW toevoertemperatuur + recovery ---------------------------
        let (theta_supply, wtw_recovered_mj) = if let Some(wtw) = effective_wtw {
            wtw_recovery::apply_wtw(theta_e, theta_i, wtw.efficiency, q_v_total, t_mi)
        } else {
            (theta_e, 0.0)
        };

        // --- Q_V ventilatie-warmteverlies (formule (11.106)) -------------
        let q_v_mj = monthly_heat_loss::heat_loss_mj(q_v_total, theta_i, theta_supply, t_mi);

        // --- W_fan ventilator-energie (forfaitair, formule (11.142)) -----
        // f_SFP: uit de WTW-spec wanneer aanwezig; anders het tabel-11.23
        // forfait voor elk systeem met een ventilator (f_systype > 0).
        // Systeem A (f_systype = 0) nult de term sowieso uit.
        let f_sfp = effective_wtw.map_or_else(
            || {
                if system.f_systype() > 0.0 {
                    FORFAIT_FAN_SFP_W_PER_M3H
                } else {
                    0.0
                }
            },
            |wtw| wtw.fan_sfp,
        );
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
/// Dit is de canonieke systeem-bewuste bepaling van de zone-ventilatiestroom:
/// - A → infiltratie (q_V;lea)
/// - B → max(mech. toevoer, infiltratie)
/// - C → max(mech. afvoer, infiltratie)
/// - D/E → mech. toevoer
///
/// Consumers buiten deze crate (o.a. `openaec-project-shared::tojuli`, voor
/// de afleiding van H_V t.b.v. de tijdconstante τ) horen deze functie te
/// hergebruiken in plaats van een eigen `max()`-heuristiek — één bron van
/// waarheid.
///
/// Pragmatische heuristiek — de norm-exacte massabalans uit §11.2.1.5/§11.2.1.6
/// (de iteratieve `p_z;ref`-oplosroutine) leeft in
/// [`pressure_solver::solve_zone_airflow`] en wordt door
/// [`calculate_ventilation_with_pressure_model`] gebruikt. Deze functie blijft
/// als snelle, schema-vrije heuristiek bestaan voor consumers die geen
/// [`crate::BuildingPressureContext`] kunnen leveren, en voor gebouwen buiten
/// C2-scope (`H ≥ 15 m`, multi-luchtstroomzone — V2).
#[must_use]
pub fn system_total_airflow(system: VentilationSystem, flow: &AirFlow) -> f64 {
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
