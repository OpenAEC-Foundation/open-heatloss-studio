//! BENG-orchestrator — `ProjectV2` → BENG 1/2/3 + TOjuli + energielabel.
//!
//! [`compute_beng`] is de end-to-end NTA 8800 / BENG-keten op een
//! [`crate::ProjectV2`], gebouwd naar het patroon van
//! [`crate::compute_tojuli_full`]: geometrie → demand → diensten → EP-score →
//! BENG-toets. De keten-volgorde en carrier-mapping volgen de referentie-
//! orchestrator van Maarten Vroegindeweij
//! (`origin/claude/nta8800-core:crates/nta8800-core/src/orchestrator.rs`); zijn
//! invoermodel is niet overgenomen.
//!
//! ## Hergebruik van de TO-juli-keten
//!
//! De demand-tak (transmissie H.8 → ventilatie H.11 → maandbalans H.7, mét de
//! gesloten volume→H_ve→τ-keten en de §11.2.2-forfaits) is al gevalideerd in
//! [`crate::compute_tojuli_full`]. `compute_beng` roept die functie aan op een
//! *effectief* project waarin het [`crate::energy::EnergyInput`]-ventilatieblok
//! de `SharedProject`-ventilatievelden overschrijft (normatieve A-E-invoer wint
//! voor BENG), en gebruikt de teruggeleverde `Q_H;nd`/`Q_C;nd`-maandprofielen
//! als demand-invoer voor de dienst-crates. Zo wordt de subtiele forfait-/
//! drukmodel-logica niet gedupliceerd.
//!
//! ## Bekende vereenvoudigingen (F3-input)
//!
//! - **TOjuli** wordt bij een actief gekoelde zone op 0 gezet (§5.7.2); zonder
//!   actieve koeling levert de keten een *whole-zone screening*-indicator
//!   (geen norm-conforme per-oriëntatie §5.7.2-opdeling) met `pass = None`.
//! - **Warmtepomp-SCOP's, koel-SEER, forfait-η's** komen uit
//!   [`mapping`]-defaults zolang de UI geen kentallen levert.
//! - **Lichte-bouwwijze-toeslag** (Bbl art. 4.149 lid 4) wordt niet automatisch
//!   toegepast: het invoer-DTO codeert de interne warmtecapaciteit nog niet.
//! - **Verlichting** telt 0 voor de woonfunctie (correct voor de nEP-indicator);
//!   utiliteitsverlichting vereist een invoerblok dat F5-scope is.

pub mod mapping;

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_automation::{calculate_automation_factors, AutomationFactors};
use nta8800_cooling::{CoolingDistribution, CoolingEmission, CoolingSystem};
use nta8800_dhw::model::{DhwDemand, DhwDistribution, DhwEmission};
use nta8800_dhw::calculate_dhw;
use nta8800_ep::{
    beng::TOJULI_LIMIT, calculate_ep_score, tojuli_orientation, BengAssessment, BengIndicators,
    BengLimits, BuildingArea, EnergyCarrier as EpCarrier, EpInputs, TojuliOrientationInput,
};
use nta8800_heating::calculate_heating;
use nta8800_model::location::Orientation;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::Temperature;
use nta8800_model::zoning::UsageFunction;
use nta8800_pv::{calculate_pv_yield, PvLocation};
use nta8800_tables::climate::de_bilt_climate_data;
use nta8800_ventilation::calculate_ventilation;
use nta8800_demand::{DemandBreakdown, DemandResult};

use crate::energy::{DhwGeneratorType, HeatGeneratorType};
use crate::geometry::{BoundaryKind, SharedGeometry};
use crate::nta8800_view::{geometry_to_nta8800, map_usage_function};
use crate::project::ProjectV2;
use crate::shared::{HeatRecovery, VentilationSystemKind};
use crate::tojuli::{compute_tojuli_full, TojuliFullInputs, TojuliResult};

use mapping::{
    cooling_carrier_to_ep, dhw_carrier_to_ep, heating_carrier_to_ep, map_automation, map_cooling,
    map_dhw_generation, map_dwtw, map_heating, map_pv, map_ventilation, DEFAULT_COOLING_SEER,
};

/// Omrekenfactor MJ → kWh (1 kWh = 3,6 MJ).
const MJ_PER_KWH: f64 = 3.6;

/// Rekenwaarde voor de lengte van de maand juli `tjuli` [h] in de TOjuli-
/// screening. NTA 8800 §17.2 geeft de exacte waarde; hier 31 × 24 = 744 h als
/// gedocumenteerde benadering (F3: uit §17.2 halen, consistent met de
/// demand-maandlengtes).
const T_JULI_H: f64 = 744.0;

// ---------------------------------------------------------------------------
// Resultaat-typen
// ---------------------------------------------------------------------------

/// Eén BENG-indicator met (indien beschikbaar) grenswaarde en pass/fail.
///
/// `limit`/`pass` zijn `None` voor gebruiksfuncties waarvoor de grenswaarden
/// nog niet geverifieerd zijn (utiliteit — zie [`BengLimits::for_utiliteit`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct IndicatorReport {
    /// Berekende indicatorwaarde (BENG 1/2 in kWh/(m²·jr), BENG 3 in %).
    pub value: f64,
    /// Grenswaarde uit het Bbl (art. 4.149), of `None` als niet-geverifieerd.
    pub limit: Option<f64>,
    /// Voldoet de indicator? `None` als er geen grenswaarde is.
    pub pass: Option<bool>,
}

/// Methode waarmee de TOjuli-indicator is bepaald.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TojuliMethod {
    /// Actief gekoelde rekenzone: §5.7.2 stelt `TOjuli = 0` voor alle
    /// oriëntaties; de zone is geacht te voldoen.
    ActivelyCooled,
    /// Whole-zone screening: de gehele zone als één bucket door formule (5.40),
    /// **geen** norm-conforme per-oriëntatie §5.7.2-opdeling. Neigt de per-
    /// oriëntatie-piek te onderschatten → `pass` blijft `None`. F3-werk.
    WholeZoneScreening,
}

/// TOjuli-oververhittingssamenvatting voor de BENG-toets (§5.7 / Bbl 4.149b).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TojuliBengSummary {
    /// Maatgevende `TOjuli` [K].
    pub max_tojuli_k: f64,
    /// Grenswaarde [K] (Bbl art. 4.149b lid 1).
    pub limit_k: f64,
    /// Is de rekenzone actief gekoeld?
    pub actively_cooled: bool,
    /// Voldoet de zone? `None` bij de whole-zone screening (niet norm-conform).
    pub pass: Option<bool>,
    /// Gebruikte bepalingsmethode.
    pub method: TojuliMethod,
}

/// Primair energiegebruik per dienst in kWh/(m²·jr) — negatief voor PV
/// (netto opwekking).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceBreakdownKwhM2 {
    /// Verwarming.
    pub heating: f64,
    /// Koeling.
    pub cooling: f64,
    /// Warm tapwater.
    pub dhw: f64,
    /// Ventilator-hulpenergie.
    pub ventilation_aux: f64,
    /// Verlichting (0 voor de woonfunctie).
    pub lighting: f64,
    /// PV-opwekking (negatief).
    pub pv: f64,
}

/// Volledig BENG-resultaat voor een [`ProjectV2`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BengResult {
    /// BENG 1 — energiebehoefte (Q_H;nd + Q_C;nd)/A_g [kWh/(m²·jr)].
    pub beng1: IndicatorReport,
    /// BENG 2 — karakteristiek primair fossiel energiegebruik [kWh/(m²·jr)].
    pub beng2: IndicatorReport,
    /// BENG 3 — aandeel hernieuwbare energie [%].
    pub beng3: IndicatorReport,
    /// TOjuli-oververhitting.
    pub tojuli: TojuliBengSummary,
    /// Energielabel (A++++ t/m G).
    pub energy_label: String,
    /// Hernieuwbaar aandeel [0..=1].
    pub renewable_share: f64,
    /// CO₂-uitstoot [kg/(m²·jr)].
    pub co2_kg_per_m2: f64,
    /// Gebruiksoppervlak A_g [m²].
    pub a_g_m2: f64,
    /// Verliesoppervlak A_ls [m²] (thermische schil).
    pub a_ls_m2: f64,
    /// Vormfactor A_ls/A_g.
    pub als_ag_ratio: f64,
    /// Primair energiegebruik per dienst [kWh/(m²·jr)].
    pub service_breakdown_kwh_m2: ServiceBreakdownKwhM2,
    /// Bekende vereenvoudigingen/stubs die op dit resultaat van toepassing zijn.
    pub notes: Vec<String>,
}

/// Fouten van de BENG-orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum BengError {
    /// Het project mist het `energy`-invoerblok.
    #[error("project mist het `energy`-invoerblok (nodig voor de BENG-keten)")]
    MissingEnergyInput,
    /// Lege geometrie én geen gebruiksoppervlak.
    #[error("project levert geen rekenzone / gebruiksoppervlak")]
    EmptyProject,
    /// NTA 8800 model-fout (view-mapping).
    #[error("nta8800 model: {0}")]
    Model(#[from] nta8800_model::ModelError),
    /// Fout uit de demand-/TO-juli-keten.
    #[error("demand/tojuli-keten: {0}")]
    Tojuli(#[from] crate::tojuli::TojuliError),
    /// Fout uit de verwarming-keten.
    #[error("verwarming: {0}")]
    Heating(#[from] nta8800_heating::HeatingError),
    /// Fout uit de tapwater-keten.
    #[error("tapwater: {0}")]
    Dhw(#[from] nta8800_dhw::DhwError),
    /// Fout uit de ventilatie-keten.
    #[error("ventilatie: {0}")]
    Ventilation(#[from] nta8800_ventilation::VentilationError),
    /// Fout uit de PV-keten.
    #[error("pv: {0}")]
    Pv(#[from] nta8800_pv::PvError),
    /// Fout uit de automatisering-keten.
    #[error("automatisering: {0}")]
    Automation(#[from] nta8800_automation::AutomationError),
    /// Fout uit de EP-score-keten.
    #[error("ep-score: {0}")]
    Ep(#[from] nta8800_ep::EpError),
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

/// Voer de volledige NTA 8800 / BENG-keten uit op een [`ProjectV2`].
///
/// Vereist een gevuld [`ProjectV2::energy`]-blok. De demand-tak loopt via
/// [`compute_tojuli_full`] (zie module-doc); de diensten (verwarming, koeling,
/// tapwater, ventilator-hulpenergie, PV) en de automation-factoren worden
/// samengevoegd tot [`EpInputs`] en door [`calculate_ep_score`] gehaald,
/// waarna BENG 1/2/3 + TOjuli + label volgen.
///
/// # Errors
///
/// [`BengError`] — ontbrekend `energy`-blok, lege geometrie, of een fout uit
/// één van de onderliggende reken-crates.
#[allow(clippy::too_many_lines)]
pub fn compute_beng(project: &ProjectV2) -> Result<BengResult, BengError> {
    let energy = project.energy.as_ref().ok_or(BengError::MissingEnergyInput)?;
    let mut notes: Vec<String> = Vec::new();

    // ---- Geometrie / zone ----
    let view = geometry_to_nta8800(&project.shared, &project.geometry)?;
    let zone = view.rekenzones.first().ok_or(BengError::EmptyProject)?.clone();
    let usage = map_usage_function(&project.shared.building_type);
    let a_g = if zone.floor_area > 0.0 {
        zone.floor_area
    } else {
        project.shared.gross_floor_area_m2.unwrap_or(0.0)
    };
    if a_g <= 0.0 {
        return Err(BengError::EmptyProject);
    }
    let a_ls = loss_surface_area_m2(&project.geometry);
    let als_ag = a_ls / a_g;

    // ---- Demand-tak via de gevalideerde TO-juli-keten ----
    let effective = effective_project_with_ventilation(project);
    let cooling_system = energy
        .cooling
        .as_ref()
        .map_or(
            CoolingSystem::CompressionCooling {
                scop_cooling: DEFAULT_COOLING_SEER,
            },
            map_cooling,
        );
    let tojuli_inputs = TojuliFullInputs {
        system: cooling_system,
        distribution: CoolingDistribution::default_insulated(),
        emission: CoolingEmission {
            efficiency: 0.95,
            regulation_factor: 0.95,
        },
        shading_factor: 1.0,
        heating_setpoint_c: 20.0,
        cooling_setpoint_c: 24.0,
    };
    let tj = compute_tojuli_full(&effective, &tojuli_inputs)?;
    let demand = demand_shell(&tj);

    // ---- Automation-factoren (toegepast op de dienst-energie) ----
    let factors = match &energy.automation {
        Some(a) => calculate_automation_factors(&map_automation(a), usage)?,
        None => AutomationFactors::unity(),
    };

    // ---- Verwarming H.9 ----
    let (heating_use_mj, heating_carrier, heating_ambient_mj) = match &energy.heating {
        Some(h) => {
            let m = map_heating(h);
            let r = calculate_heating(&demand, m.emission, &m.distribution, &m.generation, m.control)?;
            // Omgevingswarmte (§5.6.2.1, formule 5.31): alleen lucht-/bodem-WP
            // (bron < 20 °C, geen ventilatieretourlucht).
            let is_heat_pump = matches!(
                h.generator,
                HeatGeneratorType::HeatPumpAir | HeatGeneratorType::HeatPumpGround
            );
            let ambient = heat_pump_ambient_mj(
                is_heat_pump,
                r.annual_q_h_use,
                r.breakdown.generation_efficiency,
            );
            (
                r.annual_q_h_use * factors.f_bac_heating,
                Some(heating_carrier_to_ep(r.energy_carrier)),
                ambient,
            )
        }
        None => {
            notes.push("Geen verwarmingssysteem opgegeven — verwarming telt 0 mee in BENG 2.".into());
            (0.0, None, 0.0)
        }
    };

    // ---- Tapwater H.13 ----
    let (dhw_use_mj, dhw_carrier, dhw_ambient_mj) = match &energy.dhw {
        Some(d) => {
            let dhw_demand = match usage {
                UsageFunction::Woonfunctie => DhwDemand::forfaitair_woningbouw(a_g)?,
                other => DhwDemand::forfaitair_utiliteit(other, a_g)?,
            };
            let emission = if usage == UsageFunction::Woonfunctie {
                DhwEmission::WoningDefault
            } else {
                DhwEmission::UtiliteitKort
            };
            let generation = map_dhw_generation(d);
            let recovery = map_dwtw(d);
            let r = calculate_dhw(
                &zone,
                &dhw_demand,
                &emission,
                &DhwDistribution::default_individueel(),
                &generation,
                recovery.as_ref(),
            )?;
            // Omgevingswarmte tapwater-WP (§5.6.2.3, formule 5.36).
            let ambient = heat_pump_ambient_mj(
                matches!(d.generator, DhwGeneratorType::HeatPump),
                r.annual_q_w_use,
                r.breakdown.generation_efficiency,
            );
            (
                r.annual_q_w_use * factors.f_bac_dhw,
                Some(dhw_carrier_to_ep(r.energy_carrier)),
                ambient,
            )
        }
        None => {
            notes.push("Geen tapwatersysteem opgegeven — tapwater telt 0 mee in BENG 2.".into());
            (0.0, None, 0.0)
        }
    };

    // ---- Koeling H.10 (Q_C;use uit de TO-juli-keten; telt alleen bij een
    //      geïnstalleerd koelsysteem) ----
    let (cooling_use_mj, cooling_carrier) = match &energy.cooling {
        Some(c) => {
            let carrier = cooling_carrier_to_ep(map_cooling(c).energy_carrier());
            (tj.annual_q_c_use_mj * factors.f_bac_cooling, Some(carrier))
        }
        None => (0.0, None),
    };

    // ---- Ventilator-hulpenergie H.12 ----
    let vent_aux_mj = match &energy.ventilation {
        Some(v) => {
            let vm = map_ventilation(v, usage, a_g);
            let climate = de_bilt_climate_data();
            let indoor: MonthlyProfile<Temperature> = MonthlyProfile::from_constant(20.0);
            let vr = calculate_ventilation(&zone, &vm.system, &vm.flow, vm.wtw.as_ref(), &indoor, &climate)?;
            vr.annual_w_fan * factors.f_bac_ventilation
        }
        None => 0.0,
    };

    // ---- PV H.16 ----
    let pv_yield_mj = if energy.pv.is_empty() {
        0.0
    } else {
        let systems = map_pv(&energy.pv)?;
        let location = PvLocation::new(52.1, 5.2)?;
        let climate = de_bilt_climate_data();
        calculate_pv_yield(&systems, &location, &climate)?.annual_yield_mj
    };

    // Omgevingswarmte (renheat) van de warmtepomp-diensten — teller van BENG 3
    // (§5.6.2.1/§5.6.2.3). Omgevingskoude (rencold, §5.6.2.2 formule 5.34) komt
    // uit de koel-keten: de vrij-geleverde koude bij EER ≥ 8 (tabel 10.34), door
    // `calculate_cooling` bepaald en via `tj.annual_rencold_mj` doorgegeven.
    let renewable_ambient_heat_mj = heating_ambient_mj + dhw_ambient_mj;
    let renewable_ambient_cold_mj = if energy.cooling.is_some() {
        tj.annual_rencold_mj
    } else {
        0.0
    };

    // ---- EP-score H.5 ----
    let ep_inputs = EpInputs {
        heating: single_carrier_map(heating_carrier, heating_use_mj),
        cooling: single_carrier_map(cooling_carrier, cooling_use_mj),
        dhw: single_carrier_map(dhw_carrier, dhw_use_mj),
        lighting: HashMap::new(),
        ventilation_aux: single_carrier_map(Some(EpCarrier::Elektriciteit), vent_aux_mj),
        automation: HashMap::new(),
        pv_yield: pv_yield_mj,
        renewable_ambient_heat_mj,
        renewable_ambient_cold_mj,
        building_area: BuildingArea { a_g },
    };
    let ep = calculate_ep_score(&ep_inputs, usage)?;

    // ---- BENG 1/2/3-indicatoren + toets ----
    let annual_q_h_nd: f64 = demand.annual_heating_demand;
    let annual_q_c_nd: f64 = demand.annual_cooling_demand;
    let indicators = BengIndicators::from_chain(
        annual_q_h_nd,
        annual_q_c_nd,
        a_g,
        ep.ep_total_mj_per_m2,
        ep.ep_renewable_share,
    );

    let limits = match usage {
        UsageFunction::Woonfunctie => Some(BengLimits::for_woonfunctie(als_ag)),
        other => {
            let l = BengLimits::for_utiliteit(other);
            if l.is_none() {
                notes.push(
                    "Utiliteits-BENG-grenswaarden zijn nog niet geverifieerd (F5) — alleen \
                     indicatorwaarden gerapporteerd."
                        .into(),
                );
            }
            l
        }
    };
    let assessment = limits.map(|l| BengAssessment::assess(&indicators, &l));

    let beng1 = indicator_report(indicators.beng1_kwh_per_m2, assessment.map(|a| a.beng1));
    let beng2 = indicator_report(indicators.beng2_kwh_per_m2, assessment.map(|a| a.beng2));
    let beng3 = indicator_report(indicators.beng3_renewable_pct, assessment.map(|a| a.beng3));

    // ---- TOjuli ----
    let tojuli = compute_tojuli_summary(&tj, a_ls, energy.cooling.is_some());
    if matches!(tojuli.method, TojuliMethod::WholeZoneScreening) {
        notes.push(
            "TOjuli via whole-zone screening (formule 5.40 op de gehele zone) — de norm-conforme \
             per-oriëntatie §5.7.2-opdeling is F3; `pass` is daarom onbepaald."
                .into(),
        );
    }
    if usage == UsageFunction::Woonfunctie {
        notes.push(
            "Lichte-bouwwijze-toeslag (Bbl 4.149 lid 4) niet automatisch toegepast — het DTO \
             codeert de interne warmtecapaciteit nog niet (F3)."
                .into(),
        );
    }

    let service_breakdown_kwh_m2 = ServiceBreakdownKwhM2 {
        heating: primary_kwh_m2(ep.breakdown.heating.primary_energy_mj, a_g),
        cooling: primary_kwh_m2(ep.breakdown.cooling.primary_energy_mj, a_g),
        dhw: primary_kwh_m2(ep.breakdown.dhw.primary_energy_mj, a_g),
        ventilation_aux: primary_kwh_m2(ep.breakdown.ventilation_aux.primary_energy_mj, a_g),
        lighting: primary_kwh_m2(ep.breakdown.lighting.primary_energy_mj, a_g),
        pv: primary_kwh_m2(ep.breakdown.pv.primary_energy_mj, a_g),
    };

    Ok(BengResult {
        beng1,
        beng2,
        beng3,
        tojuli,
        energy_label: ep.ep_label.as_str().to_string(),
        renewable_share: ep.ep_renewable_share,
        co2_kg_per_m2: ep.ep_co2_kg_per_m2,
        a_g_m2: a_g,
        a_ls_m2: a_ls,
        als_ag_ratio: als_ag,
        service_breakdown_kwh_m2,
        notes,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Omgevingswarmte [MJ] van een warmtepomp-dienst voor de BENG 3-teller.
///
/// NTA 8800:2025+C1:2026 §5.6.2.1/§5.6.2.3 (formules 5.31/5.36):
/// `Q_hp;in = Q_gen;out × (1 − 1/COP)`. Met `Q_gen;out = Q_use × SCOP` (de
/// heating-/dhw-crates gebruiken de seizoens-COP als opwekkingsrendement) volgt
/// `Q_hp;in = Q_use × (SCOP − 1)` — de omgevingswarmte = geleverde warmte minus
/// elektrische input. Alleen voor warmtepompen met `SCOP > 1` en een bron < 20 °C
/// (lucht/bodem); andere opwekkers (weerstand, HR-ketel, stadswarmte) → 0
/// (formule 5.33).
fn heat_pump_ambient_mj(is_heat_pump: bool, q_use_mj: f64, scop: f64) -> f64 {
    if is_heat_pump && scop > 1.0 {
        q_use_mj * (scop - 1.0)
    } else {
        0.0
    }
}

/// Bouw een één-drager-map, of leeg als de drager `None` of de energie 0 is.
fn single_carrier_map(carrier: Option<EpCarrier>, energy_mj: f64) -> HashMap<EpCarrier, f64> {
    let mut map = HashMap::new();
    if let Some(c) = carrier {
        if energy_mj != 0.0 {
            map.insert(c, energy_mj);
        }
    }
    map
}

/// Zet een [`IndicatorReport`] samen uit de waarde + optionele toets.
fn indicator_report(
    value: f64,
    assessment: Option<nta8800_ep::IndicatorAssessment>,
) -> IndicatorReport {
    IndicatorReport {
        value,
        limit: assessment.map(|a| a.limit),
        pass: assessment.map(|a| a.pass),
    }
}

/// Primair energiegebruik [MJ] → [kWh/(m²·jr)].
fn primary_kwh_m2(primary_mj: f64, a_g: f64) -> f64 {
    primary_mj / a_g / MJ_PER_KWH
}

/// Verliesoppervlak A_ls [m²]: som van alle schil-constructies met een
/// warmteverlies-grens (buiten, grond, open water, onverwarmde ruimte).
/// Aangrenzende verwarmde ruimtes tellen niet mee (netto-transmissie ≈ 0).
fn loss_surface_area_m2(geometry: &SharedGeometry) -> f64 {
    geometry
        .spaces
        .iter()
        .flat_map(|s| s.constructions.iter())
        .filter(|c| {
            matches!(
                c.boundary,
                BoundaryKind::Exterior
                    | BoundaryKind::Ground
                    | BoundaryKind::OpenWater
                    | BoundaryKind::UnheatedSpace
            )
        })
        .map(|c| c.area_m2)
        .sum()
}

/// Kloon het project en laat het `energy`-ventilatieblok de `SharedProject`-
/// ventilatievelden overschrijven (normatieve A-E-invoer wint voor BENG). Bij
/// afwezig `energy.ventilation` blijft de projecteigen ventilatie staan.
fn effective_project_with_ventilation(project: &ProjectV2) -> ProjectV2 {
    let mut p = project.clone();
    let Some(energy) = project.energy.as_ref() else {
        return p;
    };
    let Some(v) = energy.ventilation.as_ref() else {
        return p;
    };
    use crate::energy::VentilationSystemType as VT;
    p.shared.ventilation_system = Some(match v.system {
        VT::A => VentilationSystemKind::Natural,
        VT::B => VentilationSystemKind::MechSupply,
        VT::C => VentilationSystemKind::MechExhaust,
        VT::D | VT::E => VentilationSystemKind::MechBalanced,
    });
    p.shared.heat_recovery = v.wtw_efficiency.map(|efficiency| HeatRecovery {
        efficiency,
        frost_protection: false,
        supply_temperature: None,
    });
    p.shared.mechanical_supply_m3_per_h = v.mechanical_supply_m3_per_h;
    p.shared.mechanical_exhaust_m3_per_h = v.mechanical_exhaust_m3_per_h;
    p.shared.infiltration_m3_per_h = v.infiltration_m3_per_h;
    p
}

/// Reconstrueer een [`DemandResult`] uit de TO-juli-keten-uitvoer voor de
/// dienst-crates. Alleen de demand-maandprofielen (Q_H;nd/Q_C;nd), de
/// jaarsommen en de tijdconstante τ zijn betekenisdragend; de diagnostische
/// sub-termen worden op nul gezet omdat de consumers ([`calculate_heating`])
/// ze niet lezen.
fn demand_shell(tj: &TojuliResult) -> DemandResult {
    let annual_h: f64 = tj.monthly_q_h_nd_mj.as_array().iter().sum();
    let annual_c: f64 = tj.monthly_q_c_nd_mj.as_array().iter().sum();
    DemandResult {
        monthly_heating_demand: tj.monthly_q_h_nd_mj.clone(),
        monthly_cooling_demand: tj.monthly_q_c_nd_mj.clone(),
        annual_heating_demand: annual_h,
        annual_cooling_demand: annual_c,
        breakdown: DemandBreakdown {
            monthly_q_ht: MonthlyProfile::from_constant(0.0),
            monthly_q_gn: MonthlyProfile::from_constant(0.0),
            monthly_q_sol: MonthlyProfile::from_constant(0.0),
            monthly_q_int: MonthlyProfile::from_constant(0.0),
            monthly_utilization_heating: MonthlyProfile::from_constant(0.0),
            monthly_utilization_cooling: MonthlyProfile::from_constant(0.0),
            time_constant_hours: tj.tau_hours,
        },
    }
}

/// TOjuli-samenvatting. Bij een actief gekoelde zone (§5.7.2) is `TOjuli = 0`;
/// anders een whole-zone screening via formule (5.40) op de gehele zone (geen
/// norm-conforme per-oriëntatie-opdeling → `pass = None`).
fn compute_tojuli_summary(tj: &TojuliResult, a_ls: f64, actively_cooled: bool) -> TojuliBengSummary {
    if actively_cooled {
        return TojuliBengSummary {
            max_tojuli_k: 0.0,
            limit_k: TOJULI_LIMIT,
            actively_cooled: true,
            pass: Some(true),
            method: TojuliMethod::ActivelyCooled,
        };
    }

    // Whole-zone screening: de gehele zone als één "oriëntatie"-bucket.
    let q_c_nd_juli_kwh = tj.monthly_q_c_nd_mj[Month::Juli] / MJ_PER_KWH;
    let input = TojuliOrientationInput {
        orientation: Orientation::Zuid,
        a_t_m2: a_ls.max(4.0),
        q_c_nd_juli_kwh,
        q_c_hp_juli_kwh: 0.0,
        h_c_d_juli_w_per_k: tj.transmission_h_t_w_per_k,
        h_gr_an_juli_w_per_k: 0.0,
        h_c_ve_juli_w_per_k: tj.ventilation_h_v_w_per_k,
    };
    let max_tojuli_k = tojuli_orientation(&input, T_JULI_H);
    TojuliBengSummary {
        max_tojuli_k,
        limit_k: TOJULI_LIMIT,
        actively_cooled: false,
        pass: None,
        method: TojuliMethod::WholeZoneScreening,
    }
}

#[cfg(test)]
mod tests;
