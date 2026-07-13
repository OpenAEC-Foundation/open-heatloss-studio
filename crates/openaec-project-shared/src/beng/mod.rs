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
//!   actieve koeling bepaalt de keten TOjuli **per oriëntatie** (formule 5.40 op
//!   de acht kompasrichtingen, Stap A/B) en levert een pass/fail. De teller
//!   `Q_C;nd;juli;or` is daarbij een gedocumenteerde benadering (zonwinst-gewogen
//!   verdeling van de whole-zone julikoudebehoefte i.p.v. een norm-exacte
//!   per-oriëntatie §7.2.2-julibalans — F3d); zie
//!   [`build_tojuli_orientation_inputs`].
//! - **Warmtepomp-SCOP's, koel-SEER, forfait-η's** komen uit
//!   [`mapping`]-defaults zolang de UI geen kentallen levert.
//! - **Lichte-bouwwijze-toeslag** (Bbl art. 4.149 lid 4) wordt niet automatisch
//!   toegepast: het invoer-DTO codeert de interne warmtecapaciteit nog niet.
//! - **Verlichting** telt 0 voor de woonfunctie (correct voor de nEP-indicator);
//!   utiliteitsverlichting vereist een invoerblok dat F5-scope is.

pub mod dynamics;
pub mod geometry_bridge;
pub mod mapping;
pub mod zeb;

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use nta8800_automation::{calculate_automation_factors, AutomationFactors};
use nta8800_cooling::{CoolingDistribution, CoolingEmission, CoolingSystem};
use nta8800_dhw::model::{DhwDemand, DhwDistribution, DhwEmission};
use nta8800_dhw::calculate_dhw;
use nta8800_ep::{
    beng::TOJULI_LIMIT, calculate_ep_score, tojuli_zone, BengAssessment, BengIndicators,
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

use crate::energy::{
    DhwGeneratorType, EnergyInput, HeatGeneratorType, ValueSource, ValueSourceKind,
};
use crate::geometry::{BoundaryKind, SharedGeometry};
use crate::nta8800_view::{geometry_to_nta8800, map_usage_function, orientation_from_degrees};
use crate::project::ProjectV2;
use crate::shared::{HeatRecovery, VentilationSystemKind};
use crate::tojuli::{compute_tojuli_full, TojuliFullInputs, TojuliResult};

use mapping::{
    cooling_carrier_to_ep, dhw_carrier_to_ep, heating_carrier_to_ep, map_automation, map_cooling,
    map_dhw_generation, map_dwtw, map_heating, map_pv, map_ventilation, DEFAULT_COOLING_SEER,
};

/// Omrekenfactor MJ → kWh (1 kWh = 3,6 MJ).
const MJ_PER_KWH: f64 = 3.6;

/// Rekenwaarde voor de lengte van de maand juli `tjuli` [h] in formule (5.40).
/// NTA 8800:2025+C1:2026 §17.2, tabel 17.1 (p. 690) geeft voor juli
/// `t_mi = 744 h` (31 d × 24 h) — dit is dus de **norm-exacte** waarde, niet
/// enkel een benadering. Identiek aan `DE_BILT_MONTH_LENGTHS_HOURS[Juli]`
/// (`nta8800-tables`), dus consistent met de demand-maandlengtes.
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
    /// Norm-conforme per-oriëntatie-bepaling (§5.7.2, formule 5.40 op de acht
    /// kompasrichtingen; maatgevend = max). De H-noemer is uit de geometrie +
    /// whole-zone `TojuliResult` gebouwd; de teller `Q_C;nd;juli;or` is een
    /// gedocumenteerde zonwinst-gewogen benadering (norm-exacte per-oriëntatie
    /// §7.2.2-julibalans = F3d). Levert een pass/fail (`pass = Some(..)`).
    PerOrientation,
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
    /// Voldoet de zone? `Some(true/false)` voor beide methoden (actief-gekoeld
    /// én de per-oriëntatie-toets). `None` blijft gereserveerd voor toekomstige
    /// niet-toetsbare gevallen.
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

/// Deelsysteem waarop een bronregistratie ([`ValueSourceReport`]) betrekking
/// heeft. Snake_case-serde spiegelt de deelsysteem-korrel van
/// [`crate::energy::EnergyInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BengSubsystem {
    /// Verwarming (H.9).
    Heating,
    /// Warm tapwater (H.13).
    Dhw,
    /// Douchewater-warmteterugwinning (bijlage U).
    Dwtw,
    /// Ventilatie (H.11).
    Ventilation,
    /// Koeling (H.10).
    Cooling,
    /// PV-veld (H.16).
    Pv,
}

/// Eén doorgegeven bronregistratie voor de rapportageketen (F4c-dossierplicht).
///
/// Wordt afgeleid uit de [`ValueSource`]-velden op de installatie-invoer en
/// bevat uitsluitend **niet-forfaitaire** bronnen (een expliciet forfait is de
/// norm-default en levert geen dossierstuk op). Puur metadata — parallel aan de
/// menselijk-leesbare regels in [`BengResult::notes`], maar gestructureerd zodat
/// de rapport-PDF-keten de herkomst kan renderen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValueSourceReport {
    /// Deelsysteem waarop de bron betrekking heeft.
    pub system: BengSubsystem,
    /// Optioneel label om gelijksoortige bronnen te onderscheiden (bv. de naam
    /// of id van een PV-veld). `None` voor de enkelvoudige deelsystemen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Soort bron (kwaliteitsverklaring, gelijkwaardigheidsverklaring, …).
    pub kind: ValueSourceKind,
    /// Vrije referentie naar het brondocument (BCRG-attest, meetrapport, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
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
    /// Bronregistratie per deelsysteem (F4c-dossierplicht) — alleen niet-forfaitaire
    /// bronnen. Additief; leeg voor projecten zonder bronopgave. Puur metadata:
    /// deze lijst reist naar de rapportage-keten maar is niet in de berekening
    /// verwerkt.
    #[serde(default)]
    pub value_sources: Vec<ValueSourceReport>,
    /// Bijlage-AB ZEB-indicator `EweP,ZEB;Tot` — **losse, additieve** informatieve
    /// output naast BENG 1/2/3 (NTA 8800:2025+C1:2026 bijlage AB). Anders dan de
    /// volledig-salderende BENG 2 crediteert deze indicator PV maar deels
    /// (directgebruik-fractie, tabel AB.1) tegen `fP,ZEB;del;el = 1,35`; zie
    /// [`zeb`]. `None` als de indicator niet faithful bepaald kan worden (bv.
    /// stadswarmte-drager). Additief: oude JSON zonder dit veld deserialiseert
    /// (`default`), en een `None` serialiseert byte-identiek weg
    /// (`skip_serializing_if`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zeb_indicator: Option<zeb::ZebIndicator>,
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
    let mut notes: Vec<String> = Vec::new();

    // ---- F6: geometrie-bron kiezen ----
    // Een aanwezige, niet-lege `beng_geometry` (gevel-georiënteerd, buiten-opp per
    // gevel) wint van de ruimte-georiënteerde `SharedGeometry` — dat is de hele
    // F6-bedoeling. De brug zet alleen de invoer om; de gevalideerde demand-keten
    // draait ongewijzigd. Zonder beng_geometry blijft alles byte-identiek (geen
    // extra note, geen kloon).
    // C3: thermische massa (C_m) + interne warmtewinst (Φ_int) worden — alleen in
    // de bridged BENG-tak — uit de eerste rekenzone afgeleid en via de
    // TO-juli-inputs doorgegeven; `None` laat de defaults staan (additief).
    let mut beng_thermal_mass = None;
    let mut beng_internal_gains = None;

    let bridged;
    let project = match project.beng_geometry.as_ref() {
        Some(bg) if !bg.zones.is_empty() => {
            let geometry = geometry_bridge::beng_geometry_to_shared(bg, &project.geometry)?;
            let a_g_total: f64 = bg.zones.iter().map(|z| z.a_g_m2).sum();
            let n_gevels: usize = bg.zones.iter().map(|z| z.gevels.len()).sum();
            // C3a/C3b — afgeleid uit de eerste BENG-rekenzone (de keten rekent
            // V1 op één rekenzone, zie `view.rekenzones.first()` hieronder).
            let usage_for_dynamics = map_usage_function(&project.shared.building_type);
            if let Some(first_zone) = bg.zones.first() {
                // C3a — bouwwijze-codes → C_m (tabel 7.10). `None` bij onbekende/
                // ontbrekende code → default `light_woning()`.
                beng_thermal_mass = dynamics::derive_thermal_mass(first_zone, usage_for_dynamics);
                match &beng_thermal_mass {
                    Some(_) => notes.push(format!(
                        "Thermische massa (C3a): C_m afgeleid uit de bouwwijze-codes \
                         (vloer {}, wand {}) via NTA 8800 tabel 7.10/7.11/7.12; \
                         woningbouw-default kolom 'geen of open plafond' (voetnoot b).",
                        first_zone.bouwwijze_vloer.as_deref().unwrap_or("—"),
                        first_zone.bouwwijze_wand.as_deref().unwrap_or("—"),
                    )),
                    None => notes.push(
                        "Thermische massa (C3a): bouwwijze-code ontbreekt of is niet \
                         herkend (bv. 'eigen waarde - bijlage B') — terugval op de \
                         default lichte woning (D_m = 55)."
                            .into(),
                    ),
                }
                // C3b — interne warmtewinst woningbouw (formule 7.21). Alleen voor
                // de woonfunctie; utiliteit (formule 7.25) blijft forfaitair (F5).
                if usage_for_dynamics == UsageFunction::Woonfunctie {
                    // N_woon = 1: grondgebonden woning (§6.6.7); meervoudige
                    // woonfuncties (appartementgebouw) zijn V2.
                    let gains = dynamics::derive_internal_gains_woningbouw(first_zone.a_g_m2, 1.0);
                    let flux = gains.heat_flux_per_m2[Month::Juli];
                    beng_internal_gains = Some(gains);
                    notes.push(format!(
                        "Interne warmtewinst (C3b): woningbouw formule 7.21 \
                         (Φ_int = 180·N_woon·N_P;woon/A_g = {flux:.2} W/m², N_woon = 1) \
                         i.p.v. het forfait 3 W/m² (tabel 7.6)."
                    ));
                }
            }
            let mut p = project.clone();
            p.geometry = geometry;
            // A_g voor de ventilatie-forfaits (§11.2.2) volgt de BENG-rekenzone.
            p.shared.gross_floor_area_m2 = Some(a_g_total);
            bridged = p;
            notes.push(format!(
                "Geometrie-bron: gevel-georiënteerde BENG-geometrie via de F6-brug \
                 ({} rekenzone(s), {n_gevels} begrenzingsvlakken, buiten-oppervlak per \
                 gevel, NTA 8800 §6.2/§8.1); de ruimte-georiënteerde SharedGeometry is \
                 genegeerd. De raam-U (formule 8.1) en — bij direct grondcontact — het \
                 vloer-op-grond P/A-grondmodel (§8.3) worden sinds C1 in de \
                 demand-transmissie meegenomen.",
                bg.zones.len(),
            ));
            &bridged
        }
        _ => project,
    };

    let energy = project.energy.as_ref().ok_or(BengError::MissingEnergyInput)?;

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
    // Transparantie: welke infiltratiebron voedt de demand-berekening (§11.2.5).
    notes.push(infiltration_source_note(&effective.shared));
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
        // C3 — thermische massa + interne warmtewinst uit de BENG-rekenzone
        // (afgeleid in de brug hierboven); `None` laat de defaults staan.
        thermal_mass: beng_thermal_mass,
        internal_gains: beng_internal_gains,
    };
    let tj = compute_tojuli_full(&effective, &tojuli_inputs)?;
    let demand = demand_shell(&tj);

    // ---- Automation-factoren (toegepast op de dienst-energie) ----
    let factors = match &energy.automation {
        Some(a) => calculate_automation_factors(&map_automation(a), usage)?,
        None => AutomationFactors::unity(),
    };

    // ---- Bijlage-AB ZEB-indicator: maandelijkse dienst-eindenergie verzamelen ----
    // Losse, additieve informatieve output (zie `zeb`-moduledoc). We vouwen de
    // maandprofielen van elke dienst (mét BAC-factor, identiek aan de EP-score-
    // invoer) in twee accumulatoren: de EP-elektriciteitsvraag `EEPus;el` en de
    // primair-totale energie van de niet-elektrische dragers. `zeb_supported`
    // gaat op false bij een niet-ondersteunde drager (stadswarmte) → indicator
    // wordt dan weggelaten i.p.v. een verkeerde factor te fabriceren.
    let mut zeb_el_mj = [0.0_f64; 12];
    let mut zeb_nonel_primary_kwh = [0.0_f64; 12];
    let mut zeb_supported = true;

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
            let carrier = heating_carrier_to_ep(r.energy_carrier);
            zeb_supported &= zeb::fold_zeb_service(
                r.monthly_q_h_use.as_array(),
                factors.f_bac_heating,
                carrier,
                &mut zeb_el_mj,
                &mut zeb_nonel_primary_kwh,
            );
            (r.annual_q_h_use * factors.f_bac_heating, Some(carrier), ambient)
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
            let carrier = dhw_carrier_to_ep(r.energy_carrier);
            zeb_supported &= zeb::fold_zeb_service(
                r.monthly_q_w_use.as_array(),
                factors.f_bac_dhw,
                carrier,
                &mut zeb_el_mj,
                &mut zeb_nonel_primary_kwh,
            );
            (r.annual_q_w_use * factors.f_bac_dhw, Some(carrier), ambient)
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
            zeb_supported &= zeb::fold_zeb_service(
                tj.monthly_q_c_use_mj.as_array(),
                factors.f_bac_cooling,
                carrier,
                &mut zeb_el_mj,
                &mut zeb_nonel_primary_kwh,
            );
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
            // Ventilator-hulpenergie is per definitie elektrisch (§5.5.3).
            zeb_supported &= zeb::fold_zeb_service(
                vr.monthly_w_fan.as_array(),
                factors.f_bac_ventilation,
                EpCarrier::Elektriciteit,
                &mut zeb_el_mj,
                &mut zeb_nonel_primary_kwh,
            );
            vr.annual_w_fan * factors.f_bac_ventilation
        }
        None => 0.0,
    };

    // ---- PV H.16 ----
    let (pv_yield_mj, pv_monthly_mj) = if energy.pv.is_empty() {
        (0.0, [0.0_f64; 12])
    } else {
        let systems = map_pv(&energy.pv)?;
        let location = PvLocation::new(52.1, 5.2)?;
        let climate = de_bilt_climate_data();
        let pv = calculate_pv_yield(&systems, &location, &climate)?;
        (pv.annual_yield_mj, *pv.monthly_yield_mj.as_array())
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
    let tojuli = compute_tojuli_summary(&tj, &project.geometry, energy.cooling.is_some());
    if matches!(tojuli.method, TojuliMethod::PerOrientation) {
        notes.push(
            "TOjuli per oriëntatie (§5.7.2, formule 5.40 op de acht kompasrichtingen). De teller \
             Q_C;nd;juli;or is een gedocumenteerde benadering: de whole-zone julikoudebehoefte is \
             naar zonwinst-aandeel per oriëntatie verdeeld i.p.v. een norm-exacte per-oriëntatie \
             §7.2.2-julibalans (F3d)."
                .into(),
        );
    }
    if usage == UsageFunction::Woonfunctie && beng_thermal_mass.is_none() {
        notes.push(
            "Lichte-bouwwijze-toeslag (Bbl 4.149 lid 4) niet automatisch toegepast — geen \
             bouwwijze-code in de invoer, C_m valt terug op de default lichte woning (F3)."
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

    // ---- Bronregistratie (F4c) — puur metadata, geen invloed op het bovenstaande.
    // Doorgevoerd als gestructureerde lijst (voor de rapport-keten) én als
    // menselijk-leesbare notes (transparantie-huisregel: bronnen nooit verbergen).
    let value_sources = collect_value_sources(energy);
    for r in &value_sources {
        notes.push(source_note(r));
    }

    // ---- Bijlage-AB ZEB-indicator (informatief, losstaand van BENG 1/2/3) ----
    // Alleen berekend als alle dienst-dragers ZEB-ondersteund zijn (all-electric,
    // gas of biomassa); bij stadswarmte laten we hem weg. PV = hernieuwbare
    // productie op eigen perceel (Epr;el,ren;tot). Zie de `zeb`-moduledoc.
    let zeb_indicator = if zeb_supported {
        let to_kwh = |mj: [f64; 12]| mj.map(|x| x / MJ_PER_KWH);
        let z = zeb::compute_zeb_indicator(&zeb::ZebInputs {
            monthly_ep_electricity_kwh: to_kwh(zeb_el_mj),
            monthly_renewable_pv_kwh: to_kwh(pv_monthly_mj),
            monthly_nonelectric_primary_kwh: zeb_nonel_primary_kwh,
            usage,
            a_g_m2: a_g,
        });
        notes.push(format!(
            "ZEB-indicator (NTA 8800 bijlage AB, informatief): EweP;ZEB;Tot = {:.2} \
             kWh/(m²·jr), naast de norm-conforme BENG 2 = {:.2}. Anders dan BENG 2 \
             (volledige PV-saldering, fP;exp;el = 1,45) crediteert de ZEB-indicator PV \
             maar deels via het directgebruik-fractiemodel (tabel AB.1, {:.0}% zelfgebruik) \
             tegen fP,ZEB;del;el = 1,35 en fP,ZEB;exp;el,ren = 1. Geen batterij/WKK \
             gemodelleerd (V1).",
            z.ewep_zeb_tot_kwh_m2,
            beng2.value,
            z.self_use_fraction * 100.0,
        ));
        Some(z)
    } else {
        notes.push(
            "ZEB-indicator (bijlage AB) niet berekend: een dienst gebruikt stadswarmte/\
             -koude, waarvan de temperatuurafhankelijke fP,ZEB;weeg-factor (tabel AB.2) \
             nog niet is gemodelleerd (F5)."
                .into(),
        );
        None
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
        value_sources,
        zeb_indicator,
    })
}

// ---------------------------------------------------------------------------
// Bronregistratie (F4c)
// ---------------------------------------------------------------------------

/// Verzamel de niet-forfaitaire bronregistraties uit het [`EnergyInput`]-blok.
///
/// Elke deelsysteem-[`ValueSource`] met `kind != Forfait` levert één
/// [`ValueSourceReport`]; PV-velden krijgen een label (naam/id/volgnummer) om
/// meerdere velden te onderscheiden. Een expliciet forfait is de norm-default en
/// wordt niet in het dossier opgenomen.
fn collect_value_sources(energy: &EnergyInput) -> Vec<ValueSourceReport> {
    let mut out = Vec::new();
    push_source(
        &mut out,
        BengSubsystem::Heating,
        None,
        energy.heating.as_ref().and_then(|h| h.source.as_ref()),
    );
    push_source(
        &mut out,
        BengSubsystem::Dhw,
        None,
        energy.dhw.as_ref().and_then(|d| d.source.as_ref()),
    );
    push_source(
        &mut out,
        BengSubsystem::Dwtw,
        None,
        energy
            .dhw
            .as_ref()
            .and_then(|d| d.dwtw.as_ref())
            .and_then(|w| w.source.as_ref()),
    );
    push_source(
        &mut out,
        BengSubsystem::Ventilation,
        None,
        energy.ventilation.as_ref().and_then(|v| v.source.as_ref()),
    );
    push_source(
        &mut out,
        BengSubsystem::Cooling,
        None,
        energy.cooling.as_ref().and_then(|c| c.source.as_ref()),
    );
    for (i, pv) in energy.pv.iter().enumerate() {
        let label = pv
            .name
            .clone()
            .or_else(|| pv.id.clone())
            .or_else(|| Some(format!("PV-veld {}", i + 1)));
        push_source(&mut out, BengSubsystem::Pv, label, pv.source.as_ref());
    }
    out
}

/// Maximale lengte [tekens] van een bron-referentie in het resultaat.
///
/// Het `reference`-veld is vrije invoer en stroomt door naar `notes`, het
/// gestructureerde rapport-veld en straks het PDF-rapport. Om te voorkomen dat
/// een abusievelijk geplakte lap tekst die kanalen opblaast, wordt de referentie
/// bij het opnemen getrimd en op deze lengte afgekapt (op char-grens, niet
/// byte-grens). Puur een presentatie-cap: de ruwe invoer op het DTO blijft
/// ongewijzigd.
const MAX_REFERENCE_LEN: usize = 200;

/// Trim een referentie en kap af op [`MAX_REFERENCE_LEN`] tekens; `None` bij
/// leeg (na trimmen).
fn normalize_reference(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(MAX_REFERENCE_LEN).collect())
}

/// Voeg een [`ValueSourceReport`] toe als de bron aanwezig én niet-forfaitair is.
fn push_source(
    out: &mut Vec<ValueSourceReport>,
    system: BengSubsystem,
    label: Option<String>,
    src: Option<&ValueSource>,
) {
    if let Some(s) = src {
        if s.kind != ValueSourceKind::Forfait {
            out.push(ValueSourceReport {
                system,
                label,
                kind: s.kind,
                // Getrimd + afgekapt: het rapport-veld en de note dragen de
                // genormaliseerde referentie (zie MAX_REFERENCE_LEN).
                reference: s.reference.as_deref().and_then(normalize_reference),
            });
        }
    }
}

/// Bouw de menselijk-leesbare note-regel voor één bronregistratie.
fn source_note(r: &ValueSourceReport) -> String {
    let system = match r.system {
        BengSubsystem::Heating => "Verwarming",
        BengSubsystem::Dhw => "Warm tapwater",
        BengSubsystem::Dwtw => "Douchewater-WTW",
        BengSubsystem::Ventilation => "Ventilatie",
        BengSubsystem::Cooling => "Koeling",
        BengSubsystem::Pv => "PV",
    };
    let kind = match r.kind {
        ValueSourceKind::Forfait => "norm-forfait",
        ValueSourceKind::Kwaliteitsverklaring => "gecontroleerde kwaliteitsverklaring (BCRG)",
        ValueSourceKind::Gelijkwaardigheidsverklaring => "gelijkwaardigheidsverklaring",
        ValueSourceKind::Meting => "meting",
        ValueSourceKind::Overig => "overige bron",
    };
    let label = r
        .label
        .as_deref()
        .map(|l| format!(" ({l})"))
        .unwrap_or_default();
    // `reference` is in push_source al getrimd + afgekapt (MAX_REFERENCE_LEN).
    let reference = r
        .reference
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!(", ref. {s}"))
        .unwrap_or_default();
    format!("Bron {system}{label}: prestatiewaarden volgens {kind}{reference}.")
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
    p.shared.q_v10_spec_dm3_s_m2 = v.q_v10_spec_dm3_s_m2;
    p
}

/// Menselijk-leesbare herkomst-note van de gebruikte infiltratiebron voor
/// [`BengResult::notes`] (transparantie-huisregel: bronnen nooit verbergen).
///
/// De prioriteitsvolgorde spiegelt
/// `nta8800_ventilation::BuildingPressureContext::effective_q_v10` + de
/// bestaande `flow.infiltration`-betekenis:
///
/// 1. **Gemeten/verklaarde `q_v10;spec`** — vervangt het forfait in het
///    §11.2.1 drukmodel (NTA 8800 §11.2.5).
/// 2. **Forfait** — bouwjaar-/gebouwtype-pad (formule (11.86)); alleen gemeld
///    als er een bouwjaar is (anders geen forfaitaire `C_lea`).
///
/// Een expliciet **absoluut** infiltratiedebiet (`infiltration_m3_per_h`) stuurt
/// een andere grootheid (`flow.infiltration`) en wordt, indien aanwezig, apart
/// vermeld.
fn infiltration_source_note(shared: &crate::shared::SharedProject) -> String {
    let mut note = String::from("Infiltratie-bron: ");
    match shared.q_v10_spec_dm3_s_m2 {
        Some(q) => {
            note.push_str(&format!(
                "gemeten/verklaarde q_v10;spec = {q} dm³/(s·m²) per A_g \
                 (NTA 8800 §11.2.5, vervangt forfait)"
            ));
        }
        None if shared.construction_year.is_some() => {
            note.push_str("forfait uit bouwjaar + gebouwtype (NTA 8800 formule 11.86)");
        }
        None => {
            note.push_str(
                "geen q_v10;spec en geen bouwjaar — geen forfaitaire C_lea, \
                 lek-infiltratie via drukmodel = 0 (NTA 8800 §11.2.5)",
            );
        }
    }
    if let Some(q) = shared.infiltration_m3_per_h {
        note.push_str(&format!("; expliciet absoluut debiet = {q} m³/h"));
    }
    note
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

/// De acht kompas-oriëntaties waarover NTA 8800 §5.7.2 (Stap A, p. 116) de
/// TOjuli-indicator opdeelt. [`Orientation::Horizontaal`] hoort hier **niet**
/// bij: horizontale vlakken worden naar rato over deze acht verdeeld (Stap 3/4).
const TOJULI_ORIENTATIONS: [Orientation; 8] = [
    Orientation::Noord,
    Orientation::NoordOost,
    Orientation::Oost,
    Orientation::ZuidOost,
    Orientation::Zuid,
    Orientation::ZuidWest,
    Orientation::West,
    Orientation::NoordWest,
];

/// Hellingsdrempel [°] waaronder een uitwendige constructie als **horizontaal**
/// geldt voor de §5.7.2-opdeling. NTA 8800 §7.6.6.4 (Vormfactor, p. 203)
/// definieert een "horizontale constructie" als een vlak "waarvan de
/// hellingshoek met de horizontaal kleiner is dan of gelijk is aan 5°". Het
/// projectveld `slope_deg` is de helling t.o.v. horizontaal (0° = plat,
/// 90° = verticaal), dus de drempel is direct toepasbaar.
const HORIZONTAL_TILT_MAX_DEG: f64 = 5.0;

/// Bepaal in welke §5.7.2-bucket een uitwendige constructie valt:
///
/// - `Some(or)` — **oriëntatiegebonden** (Stap 2/5, p. 117): het element heeft
///   een azimuth én een helling > 5° t.o.v. horizontaal (verticale gevel *óf*
///   hellend dakvlak). Telt mee in de oriëntatie-`or`-sommen van `A_T`, `H_C;D`
///   en zonwinst.
/// - `None` — **overig/horizontaal** (Stap 3/4, p. 117): (bijna-)plat vlak
///   (helling ≤ 5°, §7.6.6.4) of zonder azimuth. Wordt naar rato van `A_T;or`
///   over de oriëntaties verdeeld.
///
/// Norm-uitgangspunt (§5.7.2 Stap A, p. 116): alleen *horizontale* elementen
/// vallen buiten `A_T;or`; een **hellend dakvlak met azimuth is dus
/// oriëntatiegebonden**, niet pro-rata. De klassering hangt daarom aan de
/// **helling** (`slope_deg`), niet aan `kind` — een zuidgericht dakvlak draagt
/// bij aan het oververhittingsrisico van de zuid-oriëntatie.
fn tojuli_orientation_bucket(construction: &crate::geometry::Construction) -> Option<Orientation> {
    let deg = construction.orientation_deg?;
    // Bijna-horizontaal (plat dak/vloer) → overig, ongeacht een eventuele azimuth.
    if construction.slope_deg.is_some_and(|s| s <= HORIZONTAL_TILT_MAX_DEG) {
        return None;
    }
    Some(orientation_from_degrees(deg))
}

/// TOjuli-samenvatting. Bij een actief gekoelde zone (§5.7.2) is `TOjuli = 0`;
/// anders de norm-conforme per-oriëntatie-bepaling (formule 5.40 over de acht
/// kompasrichtingen) via [`build_tojuli_orientation_inputs`] + [`tojuli_zone`],
/// die de maatgevende (hoogste) waarde en de pass/fail levert.
fn compute_tojuli_summary(
    tj: &TojuliResult,
    geometry: &SharedGeometry,
    actively_cooled: bool,
) -> TojuliBengSummary {
    if actively_cooled {
        return TojuliBengSummary {
            max_tojuli_k: 0.0,
            limit_k: TOJULI_LIMIT,
            actively_cooled: true,
            pass: Some(true),
            method: TojuliMethod::ActivelyCooled,
        };
    }

    let inputs = build_tojuli_orientation_inputs(geometry, tj);
    let zone = tojuli_zone(&inputs, T_JULI_H, false);
    TojuliBengSummary {
        max_tojuli_k: zone.max_tojuli_k,
        limit_k: zone.limit_k,
        actively_cooled: false,
        pass: Some(zone.pass),
        method: TojuliMethod::PerOrientation,
    }
}

/// Bouw de per-oriëntatie-invoer voor de TOjuli-toets (§5.7.2, Stap A/B +
/// Stap 1-5) uit de projectgeometrie en de whole-zone [`TojuliResult`].
///
/// **Noemer (norm-conform):**
/// - `H_C;D;juli;or` = Σ exterieur-**verticale** constructies op oriëntatie `or`
///   (`A·U` + ramen/deuren `A·U`, Stap 2/5) + het exterieur-**horizontale** deel
///   (daken) naar rato van `A_T;or` verdeeld (Stap 3/4).
/// - `H_gr;an;juli;or` = Σ grond-constructies (`A·U`, gedocumenteerde
///   screening-vereenvoudiging i.p.v. het §8.3-grondmodel) naar rato van `A_T;or`.
/// - `H_C;ve;juli;or` = whole-zone `tj.ventilation_h_v_w_per_k` (incl. WTW) naar
///   rato van `A_T;or`.
///
/// **Teller (gedocumenteerde benadering — F3d-restant):** de whole-zone
/// julikoudebehoefte `tj.monthly_q_c_nd_mj[Juli]` wordt over de oriëntaties
/// verdeeld naar het **toegelaten zonwinst-aandeel**
/// `S_or = Σ ramen[or] (A_glas·g·I_juli(or))` (met horizontale ramen naar rato
/// van `A_T;or`). Dit is de fysisch dominante oriëntatie-driver van de
/// julikoudebehoefte; de norm-exacte per-oriëntatie §7.2.2-julibalans is F3d.
/// Bij `S_total = 0` (raamloze zone) valt de verdeling terug op de `A_T;or`-
/// fractie (julikoudebehoefte ≈ 0 → TOjuli ≈ 0).
///
/// `A_T;or` (Stap A) omvat de geprojecteerde geveloppervlakte per oriëntatie
/// (opake wand + openingen); oriëntaties met `A_T;or ≤ 3 m²` filtert
/// [`tojuli_zone`] uit (Stap A, p. 116).
fn build_tojuli_orientation_inputs(
    geometry: &SharedGeometry,
    tj: &TojuliResult,
) -> Vec<TojuliOrientationInput> {
    let climate = de_bilt_climate_data();
    let i_juli = |or: Orientation| -> f64 {
        climate
            .solar_irradiation
            .get(&or)
            .map_or(0.0, |p| p[Month::Juli])
    };

    // Per-oriëntatie accumulatoren (index = positie in TOJULI_ORIENTATIONS).
    let mut a_t = [0.0_f64; 8];
    let mut h_cd_vert = [0.0_f64; 8];
    let mut s_solar = [0.0_f64; 8];
    // Whole-zone horizontale/grond-termen (pro-rata over oriëntaties verdeeld).
    let mut h_cd_hor_total = 0.0_f64;
    let mut h_gr_total = 0.0_f64;
    let mut s_hor_total = 0.0_f64;

    let orientation_index = |or: Orientation| TOJULI_ORIENTATIONS.iter().position(|o| *o == or);

    for construction in geometry.spaces.iter().flat_map(|s| s.constructions.iter()) {
        // Openingen-bijdragen (ramen + deuren): projected area, transmissie, en
        // — alleen ramen met g-waarde — zonwinst-proxy.
        let opening_area: f64 = construction.openings.iter().map(|o| o.area_m2).sum();
        let opening_h: f64 = construction
            .openings
            .iter()
            .map(|o| o.area_m2 * o.u_value)
            .sum();

        match construction.boundary {
            BoundaryKind::Ground => {
                // Begane-grondvloer/grondwand → H_gr;an-term (A·U-screening).
                h_gr_total += construction.area_m2 * construction.u_value;
            }
            BoundaryKind::Exterior => {
                let h_element = construction.area_m2 * construction.u_value + opening_h;
                let a_element = construction.area_m2 + opening_area;
                match tojuli_orientation_bucket(construction) {
                    // Oriëntatiegebonden: verticale gevel óf hellend dakvlak met
                    // azimuth (helling > 5°). §5.7.2 Stap 2/5.
                    Some(or) => {
                        if let Some(idx) = orientation_index(or) {
                            a_t[idx] += a_element;
                            h_cd_vert[idx] += h_element;
                            s_solar[idx] += window_solar_gain(construction, i_juli(or));
                        }
                    }
                    // Overig/horizontaal (plat vlak ≤ 5° of geen azimuth) → naar
                    // rato van A_T;or verdeeld. §5.7.2 Stap 3/4.
                    None => {
                        h_cd_hor_total += h_element;
                        s_hor_total += window_solar_gain(construction, i_juli(Orientation::Horizontaal));
                    }
                }
            }
            // AOR/AVR/aangrenzend gebouw/open water tellen niet mee in de
            // TOjuli-noemer: §5.7.2 rekent H_C;D als directe transmissie naar
            // buitenlucht en A_T uitsluitend voor uitwendige scheidingsconstructies.
            BoundaryKind::UnheatedSpace
            | BoundaryKind::AdjacentRoom
            | BoundaryKind::AdjacentBuilding
            | BoundaryKind::OpenWater => {}
        }
    }

    let a_t_total: f64 = a_t.iter().sum();
    if a_t_total <= 0.0 {
        // Geen uitwendige gevel-oriëntaties → geen te toetsen oriëntatie.
        return Vec::new();
    }

    let h_ve_total = tj.ventilation_h_v_w_per_k;
    let q_c_juli_total_kwh = tj.monthly_q_c_nd_mj[Month::Juli] / MJ_PER_KWH;

    // Zonwinst-gewogen verdeelsleutel voor de teller (met horizontale ramen
    // naar rato van A_T;or). Terugval op de A_T-fractie als er geen zonwinst is.
    let s_total: f64 = s_solar.iter().sum::<f64>() + s_hor_total;

    let mut inputs = Vec::new();
    for (idx, &or) in TOJULI_ORIENTATIONS.iter().enumerate() {
        if a_t[idx] <= 0.0 {
            continue;
        }
        let frac = a_t[idx] / a_t_total;
        let f_c = if s_total > 0.0 {
            (s_solar[idx] + frac * s_hor_total) / s_total
        } else {
            frac
        };
        inputs.push(TojuliOrientationInput {
            orientation: or,
            a_t_m2: a_t[idx],
            q_c_nd_juli_kwh: q_c_juli_total_kwh * f_c,
            q_c_hp_juli_kwh: 0.0,
            h_c_d_juli_w_per_k: h_cd_vert[idx] + frac * h_cd_hor_total,
            h_gr_an_juli_w_per_k: frac * h_gr_total,
            h_c_ve_juli_w_per_k: frac * h_ve_total,
        });
    }
    inputs
}

/// Zonwinst-proxy [MJ] van de ramen in één constructie voor de maand juli:
/// `Σ ramen (A_glas · g · I_juli)` met `A_glas = A_raam·(1 − kozijnfractie)`.
/// Alleen openingen met een g-waarde (ramen) dragen bij; deuren (g = `None`)
/// leveren geen zontoetreding. `solar_juli` is `I_juli` [MJ/m²] voor de
/// oriëntatie van de constructie.
fn window_solar_gain(construction: &crate::geometry::Construction, solar_juli: f64) -> f64 {
    construction
        .openings
        .iter()
        .filter_map(|o| o.g_value.map(|g| (o, g)))
        .map(|(o, g)| {
            let frame = o.frame_fraction.unwrap_or(0.25);
            o.area_m2 * (1.0 - frame).max(0.0) * g * solar_juli
        })
        .sum()
}

#[cfg(test)]
mod tests;
