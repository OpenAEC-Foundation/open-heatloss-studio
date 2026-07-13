//! TO-juli orchestrator — volledige NTA 8800 H.10 keten op een [`ProjectV2`].
//!
//! Combineert [`nta8800_view`] (geometrie-mapper) + `nta8800-demand` (H.7)
//! + `nta8800-cooling` (H.10) tot één publieke functie die uit de gedeelde
//! geometrie + cooling-system inputs een [`TojuliResult`] berekent met
//! maandelijkse Q_C;use en jaarsom.
//!
//! ## Scope
//!
//! Transmissie loopt via `nta8800-transmission::calculate_transmission`
//! (Σ A·U + b-factoren), ventilatie via
//! `nta8800-ventilation::calculate_ventilation` — de echte engines, geen
//! synthese meer. Het ventilatie-warmteverlies Q_V (inclusief WTW-recovery)
//! komt uit de engine-output; de losse H_V die `calculate_demand` voedt
//! voor de tijdconstante τ wordt systeem-bewust afgeleid via
//! `system_total_airflow`. Als geen luchtdebieten bekend zijn valt de keten
//! terug op de norm-conforme benodigde luchtvolumestroom van buitenlucht
//! `q_V;ODA;req` (NTA 8800 §11.2.2 — functie van gebruiksfunctie + A_g),
//! géén handmatige ach-schatting.
//!
//! ## V2 / vervolg
//!
//! - Volledige §11.2.1.5-massabalans bij gebalanceerde systemen (D/E)
//! - Schaduw-factor uit BuildingPart-overstek/luifel-modellering
//! - Multi-rekenzone splitsing
//! - EP-bijdrage berekening (energieprestatie-index)

use nta8800_cooling::{
    calculate_cooling, CoolingDistribution, CoolingEmission, CoolingResult, CoolingSystem,
};
use nta8800_demand::calc::calculate_demand_with_cooling_ht;
use nta8800_demand::model::{
    InternalGains, ThermalMassInput,
    setpoints::{CoolingSetpoint, HeatingSetpoint},
};
use nta8800_demand::result::DemandResult;
use nta8800_model::geometry::ThermalBridgeCategory;
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::units::{Energy, Temperature};
use nta8800_model::ThermalBridgeLinear;
use nta8800_tables::climate::de_bilt_climate_data;
use nta8800_transmission::{
    calculate_transmission,
    BoundaryType as TransmissionBoundaryType, TransmissionElement,
};
use nta8800_ventilation::{
    calculate_ventilation, calculate_ventilation_with_pressure_model, system_total_airflow,
    AIR_VOLUMETRIC_HEAT_J_PER_M3_K, VentilationResult,
    model::{AirFlow, BuildingLeakageType, BuildingPressureContext, VentilationSystem, WtwSpecification},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Forfaitair specifiek ventilator vermogen (NTA 8800 tabel 11.23, modern DC-unit).
const VENTILATION_FAN_SFP_W_PER_M3H: f64 = 0.125;

// --- NTA 8800 §11.2.2 forfaitaire ventilatie-parameters ---------------------
//
// Wanneer geen luchtdebieten zijn opgegeven valt de TO-juli-keten terug op de
// norm-bepaalde benodigde luchtvolumestroom van buitenlucht `q_V;ODA;req`
// (§11.2.2.1, formule 11.22) — een traceerbare norm-bepaling, géén handmatige
// ach-schatting. De keten luidt:
//   q_usi;spec  → tabel 11.8 per gebruiksfunctie          [dm³/(s·m²)]
//   f_τ         → tabel 11.8 bezettingstijd-correctie     [-]
//   (11.56)     q_V;ODA;req;des;reken
//                 = f_lea;du · f_lea;ahu · f_τ · (q_usi;spec · A_g) · 3,6
//   (11.63)     woning-ondergrens (q_usi;spec · A_g) ≥ 35 dm³/s
//   (11.57)     installatie onbekend → q_V;ODA;req;des = q_V;ODA;req;des;reken
//   (11.22)     q_V;ODA;req = f_ctrl · f_sys · q_V;ODA;req;des / (ε_V · f_prac;req)

/// NTA 8800 §11.2.2.1.1, formule (11.22): praktijkprestatiefactor voor het
/// vereiste toevoerdebiet `f_prac;req`. Norm-vaste waarde.
const NTA_F_PRAC_REQ: f64 = 0.95;

/// NTA 8800 §11.2.2.1.1, formule (11.22): ventilatie-efficiëntie `ε_V`.
/// Norm-vaste waarde 1.
const NTA_EPSILON_V: f64 = 1.0;

/// NTA 8800 tabel 11.9: correctiefactor luchtlekken ventilatiekanalen
/// `f_lea;du` voor luchtdichtheidsklasse "Onbekend" — de norm-conforme default
/// wanneer geen kanaalspecificatie bekend is.
const NTA_F_LEA_DU_UNKNOWN: f64 = 1.10;

/// NTA 8800 §11.2.2.5.2: correctiefactor luchtlekken AHU `f_lea;ahu` bij het
/// ontbreken van een AHU (`f_lea;ahu = 1,0`).
const NTA_F_LEA_AHU_NONE: f64 = 1.0;

/// NTA 8800 §11.2.2.4.1, formule (11.56)/(11.55): omrekenfactor 3,6 van de
/// specifieke ventilatiecapaciteit (dm³/s) naar de luchtvolumestroom (m³/h).
const NTA_DM3S_TO_M3H: f64 = 3.6;

/// NTA 8800 §11.2.2.5.1, formule (11.63): woning-ondergrens voor de absolute
/// ventilatiecapaciteit `(q_usi;spec · A_g) ≥ 35 dm³/s`.
const NTA_WONING_MIN_CAPACITY_DM3S: f64 = 35.0;

/// Forfaitaire vrije verdiepingshoogte [m] om uit een rekenzone-volume het
/// gebruiksoppervlak `A_g` te schatten wanneer `gross_floor_area_m2` ontbreekt
/// (`A_g ≈ V / h`). Komt overeen met de in `nta8800_view` gehanteerde
/// standaard-verdiepingshoogte voor woningen.
const NTA_DEFAULT_ZONE_HEIGHT_M: f64 = 2.7;

/// Forfaitaire bruto verdiepingshoogte [m] voor de afleiding van de
/// gebouwhoogte uit `num_storeys`.
///
/// De gebouwhoogte voedt het NTA 8800 §11.2.1 drukmodel (winddruk-hoogteklasse
/// tabel 11.3 + C2-scopegrens van 15 m). Het projectmodel kent geen expliciet
/// gebouwhoogte-veld; `Space::height_m` is de *binnenwerkse* verdiepingshoogte.
/// We tellen daar een forfaitaire vloer-/plafonddikte bij op zodat de
/// *bruto* verdiepingshoogte ontstaat — `2,7 m` binnenwerks + `0,3 m`
/// constructie ≈ `3,0 m` bruto, een gangbare aanname voor Nederlandse
/// woningbouw. Dit is een **gedocumenteerde forfaitaire aanname**, geen
/// norm-waarde: NTA 8800 schrijft geen vaste verdiepingshoogte voor.
const FORFAIT_GROSS_STOREY_HEIGHT_M: f64 = 3.0;

/// Forfaitaire constructie-dikte [m] (vloer + plafondopbouw) die bij de
/// binnenwerkse `Space::height_m` wordt opgeteld om de bruto verdiepingshoogte
/// te krijgen. Gedocumenteerde aanname (zie [`FORFAIT_GROSS_STOREY_HEIGHT_M`]).
const FORFAIT_STOREY_CONSTRUCTION_THICKNESS_M: f64 = 0.3;

use crate::geometry::{BoundaryKind, SharedGeometry};
use crate::nta8800_view::geometry_to_nta8800;
use crate::project::ProjectV2;
use crate::shared::{
    BuildingTypeShared, HeatRecovery, ResidentialType, SharedProject, VentilationSystemKind,
};

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
/// Converteert elke Construction naar TransmissionElement's conform NTA 8800
/// §8.2.1 formule (8.1): `H_D = Σ(A_T;i · U_C;i)` over **alle** vlakdelen. Per
/// constructie ontstaat daarom:
///
/// - het **opake deel** — `A_opaak = A_bruto − Σ A_raam` bij de opake U-waarde;
/// - **elk raam/deur** apart — de kozijn-`area_m2` bij de kozijn-`u_value`.
///
/// De ramen transmitteren dus op hun eigen (samengestelde) U (`U_window`), niet
/// langer op de opake U over het volledige bruto vlak (dat was de F6-fase-2-
/// vereenvoudiging; zie [`crate::beng::geometry_bridge`]-moduledoc). Het opake
/// oppervlak wordt met het raamoppervlak verminderd zodat er geen dubbeltelling
/// ontstaat — precies Uniecs decompositie (opaak `CONSTRD_OPP` + kozijnmerken).
///
/// Een opake rest ≤ 0 (volledig beglaasde pui) levert geen opaak element op
/// (`calculate_transmission` weigert niet-positieve oppervlakten); de ramen
/// dragen dan de volledige transmissie.
fn build_transmission_elements(geometry: &SharedGeometry) -> Vec<TransmissionElement> {
    /// Onder deze opake rest [m²] wordt geen apart opaak element geëmitteerd
    /// (numerieke ruis / volledig beglaasd vlak).
    const MIN_OPAQUE_AREA_M2: f64 = 1e-9;

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
                construction.adjacent_space_id.as_ref(),
            );

            // Raam-/deuroppervlak dat van het opake deel wordt afgetrokken en als
            // eigen element(en) op de kozijn-U wordt geteld (formule 8.1).
            let openings_area: f64 = construction.openings.iter().map(|o| o.area_m2).sum();
            let opaque_area = construction.area_m2 - openings_area;

            if opaque_area > MIN_OPAQUE_AREA_M2 {
                elements.push(TransmissionElement {
                    id: format!("{}_{}", space.id, construction.id),
                    area: opaque_area,
                    u_value: construction.u_value,
                    boundary_type: boundary_type.clone(),
                    construction_id: Some(construction.id.clone()),
                });
            }

            for opening in &construction.openings {
                if opening.area_m2 <= MIN_OPAQUE_AREA_M2 {
                    continue;
                }
                elements.push(TransmissionElement {
                    id: format!("{}_{}_opening_{}", space.id, construction.id, opening.id),
                    area: opening.area_m2,
                    u_value: opening.u_value,
                    boundary_type: boundary_type.clone(),
                    construction_id: Some(construction.id.clone()),
                });
            }
        }
    }

    elements
}

/// Forfaitair minimum voor de jaargemiddelde grond-warmteoverdrachtcoëfficiënt
/// `H_g;an` [W/K] (NTA 8800 §8.3.1-fallback via bijlage I.2.3) wanneer de
/// vloerconstructie-opbouw / perimeter onbekend is. Historische default voor
/// een gemiddelde woning zonder grondcontact-details.
const H_G_AN_FORFAIT_W_PER_K: f64 = 10.0;

/// Bepaal de jaargemiddelde grond-warmteoverdrachtcoëfficiënt `H_g;an` [W/K].
///
/// **P/A-grondmodel (§8.3.2.2–§8.3.4.1)** zodra *elke* grondconstructie
/// (`boundary = Ground`) een `ground_perimeter_m` draagt: dan levert
/// [`slab_on_ground_conductance`] per vloer `H_g = A_fl·U_fl` (via de
/// karakteristieke breedte `B'_f = A/(0,5·P)` en de equivalente dikte), en de
/// som is het zone-totaal. De aparte `ψ_gr`-vloerrandterm (formule 8.36) loopt
/// in deze keten via de lineaire koudebruggen (`build_thermal_bridges_linear`).
///
/// **Forfait-terugval** ([`H_G_AN_FORFAIT_W_PER_K`]): als er grondcontact is maar
/// niet elke grondvloer een perimeter draagt (bv. de ruimte-georiënteerde
/// ISSO 51-invoer zonder P), blijft het forfaitaire minimum gelden — zo wijzigt
/// het gedrag van bestaande, perimeter-loze geometrie niet. Zonder grondcontact
/// is de waarde irrelevant: [`h_t_ground::conductance_via_ground`] nuldt de
/// bijdrage dan alsnog.
fn build_ground_conductance(geometry: &SharedGeometry) -> f64 {
    let ground: Vec<&crate::geometry::Construction> = geometry
        .spaces
        .iter()
        .flat_map(|s| s.constructions.iter())
        .filter(|c| matches!(c.boundary, BoundaryKind::Ground))
        .collect();

    if ground.is_empty() {
        // Geen grondcontact — waarde wordt in de transmission-crate genuld.
        return H_G_AN_FORFAIT_W_PER_K;
    }

    // P/A-model alleen als élke grondvloer een perimeter heeft; anders forfait
    // (byte-identiek voor perimeter-loze bestaande geometrie).
    if ground.iter().all(|c| c.ground_perimeter_m.is_some()) {
        ground
            .iter()
            .map(|c| {
                let p = c.ground_perimeter_m.unwrap_or(0.0);
                nta8800_transmission::slab_on_ground_conductance(c.area_m2, p, c.u_value)
            })
            .sum()
    } else {
        H_G_AN_FORFAIT_W_PER_K
    }
}

/// Bouw de lineaire-koudebruggenlijst voor `calculate_transmission` uit de
/// gedeelde geometrie.
///
/// Elke [`crate::geometry::ThermalBridge`] wordt 1-op-1 een
/// [`nta8800_model::ThermalBridgeLinear`] (`ψ`, `L`); de bijdrage aan `H_D` is
/// `Σ ψ·L` (formule 8.1). Categorie → [`ThermalBridgeCategory::Overig`] omdat
/// het gedeelde model (nog) geen bijlage-I-detailtype draagt — de categorie
/// stuurt alleen rapportage, niet de sommatie.
fn build_thermal_bridges_linear(geometry: &SharedGeometry) -> Vec<ThermalBridgeLinear> {
    geometry
        .thermal_bridges
        .iter()
        .map(|tb| ThermalBridgeLinear {
            id: tb.id.clone(),
            length: tb.length_m,
            psi: tb.psi_w_per_mk,
            category: ThermalBridgeCategory::Overig,
        })
        .collect()
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

/// Leid een forfaitaire **gebouwhoogte** [m] af voor het NTA 8800 §11.2.1
/// drukmodel.
///
/// Het projectmodel kent geen expliciet gebouwhoogte-veld (een schema-veld
/// toevoegen zou een frontend-migratie forceren — bewust niet gedaan). De
/// hoogte wordt daarom afgeleid uit de wél aanwezige velden, in deze
/// prioriteitsvolgorde:
///
/// 1. **`num_storeys`** — `num_storeys × 3,0 m` bruto verdiepingshoogte
///    ([`FORFAIT_GROSS_STOREY_HEIGHT_M`]). Dit is de meest betrouwbare bron.
/// 2. **Som van de space-hoogtes** — als `num_storeys` ontbreekt: de som van
///    `Space::height_m` (binnenwerks) over alle spaces, elk verhoogd met de
///    forfaitaire constructie-dikte ([`FORFAIT_STOREY_CONSTRUCTION_THICKNESS_M`]).
///    Let op: dit klopt alleen als de spaces verschillende verdiepingen zijn;
///    bij een meerdere-kamers-per-verdieping-model overschat dit. Het is een
///    bewuste, gedocumenteerde benadering — overschatting trekt het gebouw
///    eerder buiten C2-scope (`≥ 15 m`), wat een veilige terugval op de
///    heuristiek triggert i.p.v. een twijfelachtige massabalans.
/// 3. **Fallback** — geen van beide bekend: één bruto verdiepingshoogte
///    (`3,0 m`), de meest conservatieve aanname (laagbouw, binnen C2-scope).
///
/// De afgeleide hoogte is een **forfaitaire aanname**, geen norm-waarde.
fn derive_building_height_m(shared: &SharedProject, geometry: &SharedGeometry) -> f64 {
    // 1. num_storeys is de betrouwbaarste bron.
    if let Some(storeys) = shared.num_storeys {
        if storeys > 0 {
            return f64::from(storeys) * FORFAIT_GROSS_STOREY_HEIGHT_M;
        }
    }

    // 2. Som van de space-hoogtes (binnenwerks → bruto).
    let summed: f64 = geometry
        .spaces
        .iter()
        .map(|s| s.height_m + FORFAIT_STOREY_CONSTRUCTION_THICKNESS_M)
        .sum();
    if summed > 0.0 {
        return summed;
    }

    // 3. Conservatieve fallback: één bruto verdiepingshoogte (laagbouw).
    FORFAIT_GROSS_STOREY_HEIGHT_M
}

/// Map het project-`BuildingTypeShared` naar de NTA 8800 tabel-11.14
/// gebouwtype-classificatie [`BuildingLeakageType`].
///
/// Tabel 11.14 onderscheidt de gebouwcategorie (grondgebonden / eengezins plat
/// dak / meerlaags) én de uitvoeringsvariant (tussen-/kop-/vrijstaande
/// ligging). Het projectmodel codeert dat niet één-op-één; deze mapper maakt
/// een **gedocumenteerde, conservatieve** vertaling:
///
/// - **Woning** — het `ResidentialType` bepaalt de variant. Eén-/tweelaagse
///   grondgebonden woningen (vrijstaand, 2-onder-1-kap, tussen-/hoekwoning)
///   gaan naar de grondgebonden-met-kap-categorie; gestapelde/portiek-/
///   galerijwoningen naar de meerlaagse categorie. `num_storeys ≥ 3` schuift
///   een grondgebonden woning niet om — de woningvorm is leidend in tabel 11.14.
/// - **Utiliteit** — enkellaagse utiliteitsbouw (`num_storeys ≤ 1`) valt onder
///   de grondgebonden categorie als vrijstaand gebouw; meerlaagse
///   utiliteitsbouw onder de meerlaagse categorie, behandeld als het
///   gebouw-als-geheel (footnote a).
///
/// Bij twijfel kiest de mapper de **vrijstaande / hoogste-`f_type`-variant**
/// binnen een categorie: dat geeft een hogere forfaitaire luchtlekkage
/// (`f_type` 1,2-1,4 i.p.v. 1,0) en daarmee een conservatievere — niet te
/// optimistische — infiltratieschatting.
fn derive_building_leakage_type(shared: &SharedProject) -> BuildingLeakageType {
    use BuildingLeakageType as L;
    match &shared.building_type {
        BuildingTypeShared::Woning { subtype } => match subtype {
            // Grondgebonden eengezinswoningen met kap — variant uit de ligging.
            ResidentialType::Detached => L::GroundBoundDetachedPitchedRoof,
            ResidentialType::SemiDetached | ResidentialType::EndOfTerrace => {
                L::GroundBoundEndPitchedRoof
            }
            ResidentialType::Terraced => L::GroundBoundTerracedPitchedRoof,
            // Gestapelde / portiek- / galerijwoningen → meerlaagse categorie.
            // Zonder verdieping-positie nemen we het gebouw als geheel
            // (footnote a, f_type = 1,2) — conservatief t.o.v. de
            // tussenligging-variant (f_type = 1,0).
            ResidentialType::Porch | ResidentialType::Gallery | ResidentialType::Stacked => {
                L::MultiStoreyWholeBuilding
            }
        },
        BuildingTypeShared::Utiliteit { .. } => {
            // Enkellaagse utiliteitsbouw → grondgebonden categorie, vrijstaand
            // gebouw met (deels) plat dak (f_type = 1,2). Meerlaagse
            // utiliteitsbouw → meerlaagse categorie, gebouw als geheel.
            if shared.num_storeys.is_none_or(|n| n <= 1) {
                L::GroundBoundDetachedPartlyFlatRoof
            } else {
                L::MultiStoreyWholeBuilding
            }
        }
    }
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
    /// Globale schaduw-factor F_sh (0..=1), whole-zone override. 1.0 = geen
    /// globale schaduw. **Voorrang/samenspel:** dit is een grove blunt-factor
    /// die op *alle* ramen wordt toegepast; de norm-geforfaiteerde beweegbare
    /// zonwering per raam (NTA 8800 §7.6.6.1.4) staat op `Opening.movable_shading`
    /// in de geometrie en vermenigvuldigt hiermee. Laat op 1.0 staan tenzij een
    /// projectbrede handmatige reductie gewenst is.
    #[serde(default = "default_shading")]
    pub shading_factor: f64,
    /// Verwarmings-setpoint °C (constant alle maanden).
    #[serde(default = "default_heating_setpoint")]
    pub heating_setpoint_c: f64,
    /// Koel-setpoint °C (constant alle maanden).
    #[serde(default = "default_cooling_setpoint")]
    pub cooling_setpoint_c: f64,
    /// Effectieve interne warmtecapaciteit C_m (NTA 8800 §7.7, tabel 7.10). `None`
    /// → de default `ThermalMassInput::light_woning()` (D_m = 55). De BENG-brug
    /// (`compute_beng`) vult dit uit de bouwwijze-codes (C3a); standalone
    /// TO-juli-callers laten het `None` zodat het gedrag byte-identiek blijft.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal_mass: Option<ThermalMassInput>,
    /// Interne warmtewinst Φ_int (W/m² per maand). `None` → de forfaitaire
    /// tabel-7.6-default per gebruiksfunctie. De BENG-brug vult dit voor
    /// woningbouw met formule 7.21 (C3b); standalone callers laten het `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_gains: Option<InternalGains>,
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
    /// Jaarsom hernieuwbare omgevingskoude Q_C;gen;out;rencold in MJ (BENG 3,
    /// §5.6.2.2 formule 5.34) — > 0 alleen bij (vrije) koeling met EER ≥ 8.
    pub annual_rencold_mj: Energy,
    /// Maandelijkse warmtebehoefte Q_H;nd in MJ (bijproduct demand-keten).
    pub monthly_q_h_nd_mj: MonthlyProfile<Energy>,
    /// H_T (W/K) gebruikt voor demand — Σ A·U op exterior/ground/adjacent.
    pub transmission_h_t_w_per_k: f64,
    /// H_V (W/K) die de tijdconstante τ voedt — afgeleid uit de
    /// systeem-bewuste `q_V;tot` (engine-functie) × ρ_a·c_a/3600, mét
    /// WTW-reductiefactor (1 − η_hr) voor gebalanceerde systemen.
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
    /// Ongeldige `q_v10;spec`-invoer: een specifieke luchtdoorlatendheid is per
    /// definitie ≥ 0 en eindig (NTA 8800 §11.2.5). `Some(0.0)` is geldig
    /// (perfecte luchtdichtheid); een negatieve of niet-eindige waarde wordt
    /// hier expliciet geweigerd i.p.v. stil naar het forfait teruggeschoven.
    #[error("ongeldige q_v10;spec = {0} dm³/(s·m²): moet ≥ 0 en eindig zijn (NTA 8800 §11.2.5)")]
    InvalidQv10Spec(f64),
}

/// Hoofd-orchestrator: voer de volledige TO-juli H.10 berekening uit.
///
/// Pipeline:
/// 1. `geometry_to_nta8800` levert Rekenzone + EFR + Window + Construction
/// 2. Echte transmissie via `nta8800-transmission::calculate_transmission`
/// 3. Echte ventilatie via `nta8800-ventilation::calculate_ventilation`
///    (Q_V mét WTW-recovery); H_V voor τ afgeleid via `system_total_airflow`.
///    Voor system D/E/B/C zonder mech-debieten valt mech_supply/exhaust
///    terug op het NTA 8800 §11.2.2 forfait `q_V;ODA;req` zodat H_V > 0
///    blijft. Eerder voer voor de keten alleen voor `ventilation_system =
///    None / Natural`; deze uitbreiding dekt ook D/E/B/C met onbekende
///    debieten (typisch V1→V2-migratie-output).
/// 4. `calculate_demand` → Q_C;nd + Q_H;nd 12 maanden
/// 5. `calculate_cooling` → Q_C;use 12 maanden + jaarsom
///
/// # Errors
/// Zie [`TojuliError`].
pub fn compute_tojuli_full(
    project: &ProjectV2,
    inputs: &TojuliFullInputs,
) -> Result<TojuliResult, TojuliError> {
    // ---- 0. Invoervalidatie ----
    // q_v10;spec (F3d-9) op de invoergrens weigeren als hij negatief of
    // niet-eindig is: een negatieve waarde zou verderop stil door de
    // `c_lea > 0.0`-guard vallen (= gedrag alsof er geen meting is). `Some(0.0)`
    // is bewust géén fout (perfecte luchtdichtheid; de `or_else`-terugval naar
    // het forfait triggert alleen op `None`).
    if let Some(q) = project.shared.q_v10_spec_dm3_s_m2 {
        if q < 0.0 || !q.is_finite() {
            return Err(TojuliError::InvalidQv10Spec(q));
        }
    }

    // ---- 1. View ----
    let view = geometry_to_nta8800(&project.shared, &project.geometry)?;
    let zone = view.rekenzones.first().ok_or(TojuliError::EmptyProject)?;

    // ---- 2. Echte transmissie via nta8800-transmission ----
    let climate = de_bilt_climate_data();
    let elements = build_transmission_elements(&project.geometry);
    // Lineaire koudebruggen (§8.2.3, formule 8.1: Σ ψ·L) uit de gedeelde
    // geometrie. Vóór F3d-4 stond hier een harde `Vec::new()` — dat was hét
    // verliespunt waardoor de koudebrugtoeslag nooit in H_T (en dus in Q_H;nd)
    // terechtkwam. Zowel de TO-juli- als de BENG-keten lopen via deze functie,
    // dus deze propagatie dekt beide.
    let thermal_bridges_linear = build_thermal_bridges_linear(&project.geometry);
    let thermal_bridges_point = Vec::new(); // Punt-χ nog niet in het model (§8.2.1 OPM. 4)

    let indoor_temperature = MonthlyProfile::from_constant(inputs.heating_setpoint_c);
    // Koel-rekentemperatuur θ_int;set;C (woonfunctie 24 °C, NTA 8800 tabel 7.13):
    // de warmteoverdracht voor koeling (§7.3.2 formule 7.15, §7.4) rekent tegen
    // deze hógere setpoint dan de verwarmingsbalans, zodat de koudebalans (7.7)
    // de juiste — grotere — verliesterm ziet.
    let cooling_indoor_temperature = MonthlyProfile::from_constant(inputs.cooling_setpoint_c);

    // H_g;an (§8.3): P/A-grondmodel zodra elke grondvloer een perimeter draagt,
    // anders het forfaitaire minimum. Zie [`build_ground_conductance`].
    let h_g_an = build_ground_conductance(&project.geometry);

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

    // Transmissie voor koeling (§7.3.2, formule 7.15): identieke schil/conductanties,
    // maar tegen de koel-setpoint θ_int;set;C. Levert samen met de koel-ventilatie
    // de warmteoverdracht voor koeling `Q_C;ht` voor de koudebalans (7.7).
    let transmission_cooling = calculate_transmission(
        zone,
        &elements,
        &thermal_bridges_linear,
        &thermal_bridges_point,
        &cooling_indoor_temperature,
        &climate,
        h_g_an,
        &unheated_space_b_factors,
        &adjacent_zone_temperatures,
    ).map_err(TojuliError::Transmission)?;

    // ---- 3. Ventilation via nta8800-ventilation engine ----
    //
    // Norm-forfait i.p.v. handmatige ach-schatting: als geen luchtdebieten zijn
    // opgegeven valt de keten terug op de NTA 8800 §11.2.2 benodigde
    // luchtvolumestroom van buitenlucht `q_V;ODA;req` (functie van
    // gebruiksfunctie + gebruiksoppervlak A_g) — een traceerbare norm-bepaling.
    //
    // De fallback triggert in twee situaties:
    //  (a) géén systeemtype én géén mechanische debieten bekend (volledig
    //      ontbrekende ventilatie-configuratie); en
    //  (b) systeemtype A (natuurlijke ventilatie) zonder ingevoerd
    //      infiltratie-/natuurlijk-toevoerdebiet — QC-bevinding 5: zonder dit
    //      forfait levert systeem A een stille `flow.infiltration = 0` en
    //      daarmee `h_v = 0`. Voor systeem A is `q_V;ODA;eff = q_V;vent;in`
    //      (§11.2.2.2.1), dus de norm-bepaalde `q_V;ODA;req` is hier de
    //      correcte natuurlijke toevoer.
    //
    // A_g uit `shared.gross_floor_area_m2`; bij ontbreken afgeleid uit het
    // rekenzone-volume / verdiepingshoogte (`zone.volume / NTA_DEFAULT_ZONE_HEIGHT_M`).
    let usage_function_for_ventilation = view
        .efrs
        .first()
        .map(|e| e.usage_function)
        .unwrap_or(nta8800_model::zoning::UsageFunction::Woonfunctie);
    let effective_infiltration_m3_per_h = project.shared.infiltration_m3_per_h.or_else(|| {
        let no_mech_config = project.shared.mechanical_supply_m3_per_h.is_none()
            && project.shared.mechanical_exhaust_m3_per_h.is_none();
        // De Natural-tak triggert het forfait ook wanneer er wél
        // `mechanical_supply/exhaust_m3_per_h` zijn ingevuld maar
        // `infiltration_m3_per_h` leeg is. Dat is bewust: bij systeem A
        // (natuurlijke toe- én afvoer) ís de buitenluchttoevoer per definitie
        // de natuurlijke luchtvolumestroom — eventueel ingevoerde mechanische
        // debieten horen niet bij systeem A en mogen de toevoer niet bepalen
        // (§11.2.2.2.1: q_V;SUP;eff = q_V;ETA;eff = 0, q_V;ODA;eff =
        // q_V;vent;in). Het norm-forfait `q_V;ODA;req` is dan de juiste bron.
        let needs_fallback = (project.shared.ventilation_system.is_none() && no_mech_config)
            || matches!(
                project.shared.ventilation_system,
                Some(VentilationSystemKind::Natural)
            );
        if needs_fallback {
            // A_g: bij voorkeur expliciet uit shared, anders uit het zone-volume.
            let a_g = project
                .shared
                .gross_floor_area_m2
                .filter(|v| *v > 0.0)
                .unwrap_or_else(|| zone.volume / NTA_DEFAULT_ZONE_HEIGHT_M);
            Some(nta8800_q_v_oda_req_m3_per_h(
                usage_function_for_ventilation,
                a_g,
            ))
        } else {
            None
        }
    });

    let (system, mut flow, wtw) = map_ventilation_to_nta8800(
        project.shared.ventilation_system,
        project.shared.mechanical_supply_m3_per_h,
        project.shared.mechanical_exhaust_m3_per_h,
        effective_infiltration_m3_per_h,
        project.shared.heat_recovery.as_ref(),
    );

    // NTA 8800 §11.2.2 forfait-fallback voor mechanische debieten als het
    // systeem mech vereist maar geen debieten in shared zijn opgegeven.
    // Zonder dit valt `system_total_airflow(D|E|B)` terug op
    // `flow.mechanical_supply = 0` (en idem voor C: `flow.mechanical_exhaust = 0`)
    // → q_v_total = 0 → H_V = 0. Dat is fysisch onmogelijk voor een bewoond
    // gebouw met mechanische ventilatie. Concreet trigger-pad: V1→V2-migratie
    // zet `ventilation_system = MechBalanced` + `heat_recovery = Some(η)` maar
    // laat `mechanical_supply/exhaust_m3_per_h = None`. We vullen die met het
    // norm-forfait `q_V;ODA;req` zodat de bestaande keten (drukmodel +
    // heuristiek) een non-zero H_V oplevert.
    let needs_supply_forfait = matches!(
        system,
        VentilationSystem::B | VentilationSystem::D { .. } | VentilationSystem::E
    ) && project.shared.mechanical_supply_m3_per_h.is_none()
        && flow.mechanical_supply == 0.0;
    let needs_exhaust_forfait = matches!(system, VentilationSystem::C)
        && project.shared.mechanical_exhaust_m3_per_h.is_none()
        && flow.mechanical_exhaust == 0.0;

    if needs_supply_forfait || needs_exhaust_forfait {
        let a_g_for_forfait = project
            .shared
            .gross_floor_area_m2
            .filter(|v| *v > 0.0)
            .unwrap_or_else(|| zone.volume / NTA_DEFAULT_ZONE_HEIGHT_M);
        let q_oda_req = nta8800_q_v_oda_req_m3_per_h(
            usage_function_for_ventilation,
            a_g_for_forfait,
        );
        if needs_supply_forfait {
            flow.mechanical_supply = q_oda_req;
        }
        if needs_exhaust_forfait {
            flow.mechanical_exhaust = q_oda_req;
        }
    }

    // --- NTA 8800 §11.2.1 drukmodel: massabalans-context + scope-toets ---
    //
    // Het ventilatie-warmteverlies komt sinds C2.3 uit de norm-exacte
    // massabalans (§11.2.1.5/§11.2.1.6): per maand wordt de interne
    // referentiedruk `p_z;ref` opgelost en daaruit de effectieve
    // luchtvolumestroom afgeleid. Dat vereist een `BuildingPressureContext`
    // met gebouwhoogte, bouwjaar, A_g en het tabel-11.14-gebouwtype.
    //
    // De gebouwhoogte is forfaitair afgeleid (`derive_building_height_m` —
    // het projectmodel kent geen expliciet hoogte-veld). Het drukmodel wordt
    // alleen ingezet als aan TWEE voorwaarden is voldaan:
    //
    //  1. **C2-scope** — de afgeleide gebouwhoogte `< 15 m`. NTA 8800
    //     tabel 11.1 splitst een rekenzone met `H ≥ 15 m` op in meerdere
    //     luchtstroomzones, elk met een eigen massabalans — dat is V2-scope
    //     (multi-luchtstroomzone).
    //  2. **Bekend bouwjaar** — zonder `construction_year` levert
    //     `forfait_q_v10()` géén forfaitaire `q_v10;lea;ref` (tabel 11.13 `f_y`
    //     niet bepaalbaar) en dus géén lek-conductantie `C_lea`. De gebouwschil
    //     is dan effectief dicht: de massabalans (11.5) kan een
    //     temperatuur-asymmetrische of onbalans-mechanische configuratie niet
    //     sluiten en de `p_z;ref`-routine zou niet convergeren. NTA 8800
    //     §11.2.5 vereist in dat geval een luchtdoorlatendheidsmeting
    //     (NEN 2686:1988) — een meetwaarde-invoerpad is V2-scope.
    //
    // Valt het project buiten één van beide voorwaarden, dan valt de keten
    // terug op de bestaande `calculate_ventilation`-heuristiek i.p.v. een
    // niet-convergerende of twijfelachtige 1-zone-massabalans te forceren.
    let pressure_a_g = project
        .shared
        .gross_floor_area_m2
        .filter(|v| *v > 0.0)
        .unwrap_or_else(|| zone.volume / NTA_DEFAULT_ZONE_HEIGHT_M);
    let pressure_context = BuildingPressureContext::new(
        derive_building_height_m(&project.shared, &project.geometry),
        project.shared.construction_year,
        pressure_a_g,
        derive_building_leakage_type(&project.shared),
    )
    // F3d-9: een gemeten/verklaarde `q_v10;spec` (dm³/(s·m²) per A_g) vervangt
    // het forfait uit formule (11.86) in de infiltratie-C_lea (§11.2.5).
    .with_measured_q_v10_spec(project.shared.q_v10_spec_dm3_s_m2);
    // effective_q_v10() = meting > forfait: een gemeten q_v10;spec activeert het
    // drukmodel óók zonder bekend bouwjaar (de meting heeft geen f_y nodig).
    let use_pressure_model =
        pressure_context.within_c2_scope() && pressure_context.effective_q_v10().is_some();

    // De ventilatie-warmteoverdracht wordt tweemaal berekend tegen dezelfde
    // configuratie maar een **andere rekentemperatuur**: één keer op de
    // verwarmings-setpoint (θ_int;set;H, voedt Q_H;ht) en één keer op de
    // koel-setpoint (θ_int;set;C, voedt Q_C;ht — §7.4/formule 7.15). Beide takken
    // van de branch (§11.2.1-drukmodel vs `q_V;tot`-heuristiek) hangen alleen via
    // de meegegeven `indoor_temperature` van de setpoint af; de closure isoleert
    // die ene variabele zodat de systeem-/debiet-context identiek blijft.
    let compute_ventilation = |indoor: &MonthlyProfile<Temperature>| -> Result<VentilationResult, TojuliError> {
        if use_pressure_model {
            // C2-scope + bekend bouwjaar: norm-exacte §11.2.1.6 massabalans per maand.
            //
            // Het §11.2.1 drukmodel modelleert de natuurlijke ventilatie-openingen
            // (systeem A) via een conductantie `C_vent = q_V;ODA;req`
            // (`pressure_solver::build_openings`, §11.2.2.2.1). Die functie leest
            // `q_V;ODA;req` af uit de mechanische-debietvelden van [`AirFlow`] — bij
            // systeem A heeft die geen mechanisch debiet, dus zonder ingreep blijft
            // de natuurlijke vent-conductantie 0 en levert de massabalans alleen
            // infiltratie. De échte `q_V;ODA;req` is hierboven al norm-conform
            // bepaald (werkpakket B) en landt in `flow.infiltration`; we propageren
            // die naar de mechanische velden op een **lokale kopie** zodat
            // `build_openings` voor systeem A de natuurlijke toe- én
            // afvoer-conductantie krijgt. De heuristiek-terugval leest die velden
            // niet en moet de ongemoeide `flow` krijgen.
            let mut pressure_flow = flow;
            if matches!(system, VentilationSystem::A) {
                pressure_flow.mechanical_supply = pressure_flow.infiltration;
                pressure_flow.mechanical_exhaust = pressure_flow.infiltration;
            }
            calculate_ventilation_with_pressure_model(
                zone,
                &system,
                &pressure_flow,
                wtw.as_ref(),
                &pressure_context,
                indoor,
                &climate,
            )
            .map_err(TojuliError::Ventilation)
        } else {
            // H ≥ 15 m (multi-luchtstroomzone, V2) of onbekend bouwjaar (geen
            // forfaitaire C_lea): terugval op de systeem-bewuste
            // `q_V;tot`-heuristiek.
            calculate_ventilation(zone, &system, &flow, wtw.as_ref(), indoor, &climate)
                .map_err(TojuliError::Ventilation)
        }
    };
    let ventilation = compute_ventilation(&indoor_temperature)?;

    // H_V (W/K) voor de demand-calc — voedt uitsluitend de tijdconstante τ
    // (`time_constant_hours`); het ventilatie-warmteverlies Q_ht komt al uit
    // `ventilation.monthly_q_v` (engine-output, inclusief WTW-recovery én —
    // binnen C2-scope — de norm-exacte massabalans).
    //
    // Afleiding consistent met de ventilation-engine:
    //   1. q_V;tot via `system_total_airflow` — systeem-bewust, dezelfde bron
    //      van waarheid als `calculate_ventilation` zelf gebruikt. Voor de
    //      tijdconstante τ volstaat deze representatieve, druk-onafhankelijke
    //      `q_V;tot`: τ is een tweede-orde-grootheid (vormt alleen de
    //      maand-demping), terwijl het eerste-orde-warmteverlies Q_V al uit de
    //      massabalans-engine komt. De per-maand variërende massabalans-`q_V`
    //      tot één τ terugbrengen zou een aparte aanname vereisen — de
    //      heuristiek is hier de transparantere keuze.
    //   2. Norm-exacte volumetrische warmtecapaciteit ρ_a·c_a (≈1212,23
    //      J/(m³·K), NTA 8800 formule 11.106) gedeeld door 3600 → W/(m³/h·K).
    //   3. WTW-reductie: een gebalanceerd systeem met WTW levert de
    //      toevoerlucht op ϑ_sup = ϑ_e + η·(ϑ_i − ϑ_e), zodat de effectieve
    //      temperatuursprong ϑ_i − ϑ_sup = (ϑ_i − ϑ_e)·(1 − η). De effectieve
    //      ventilatie-warmteoverdracht — en daarmee H_V — schaalt dus met
    //      (1 − η). Volgt direct uit `wtw_recovery::apply_wtw`; geen aanname.
    let q_v_total = system_total_airflow(system, &flow);
    let h_v_per_m3h = AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
    let wtw_factor = wtw.as_ref().map_or(1.0, |w| 1.0 - w.efficiency);
    let h_v = q_v_total * h_v_per_m3h * wtw_factor;

    // ---- 5. Demand calc ----
    // Interne warmtewinst + thermische massa: een door de BENG-brug meegegeven
    // waarde (C3, uit bouwwijze-codes resp. formule 7.21) wint; anders de
    // forfaitaire tabel-7.6-flux resp. `light_woning()` (byte-identieke terugval
    // voor standalone TO-juli-callers).
    let internal_gains = inputs
        .internal_gains
        .clone()
        .unwrap_or_else(|| InternalGains::forfaitair(usage_function_for_ventilation));
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(inputs.heating_setpoint_c));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(inputs.cooling_setpoint_c));
    let thermal_mass = inputs
        .thermal_mass
        .unwrap_or_else(ThermalMassInput::light_woning);

    // Warmteoverdracht voor koeling `Q_C;ht;mi` = transmissie + ventilatie tegen de
    // koel-setpoint (§7.2.3). De koudebalans (7.7) rekent hiermee i.p.v. de
    // verwarmings-`Q_H;ht`, en de §7.2.2-poort (formule 7.6) gebruikt γ_C =
    // Q_C;gn / Q_C;ht. De verwarmingsbalans blijft ongewijzigd (`transmission`/
    // `ventilation` op θ_int;set;H).
    let ventilation_cooling = compute_ventilation(&cooling_indoor_temperature)?;
    let cooling_heat_transfer: MonthlyProfile<Energy> = MonthlyProfile::new(std::array::from_fn(
        |i| {
            let m = Month::all()[i];
            transmission_cooling.monthly_q_t[m] + ventilation_cooling.monthly_q_v[m]
        },
    ));

    let windows_refs: Vec<&nta8800_model::geometry::window::Window> = view.windows.iter().collect();
    let demand: DemandResult = calculate_demand_with_cooling_ht(
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
        Some(&cooling_heat_transfer),
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
        annual_rencold_mj: cooling.annual_rencold_mj,
        monthly_q_h_nd_mj: demand.monthly_heating_demand,
        transmission_h_t_w_per_k: transmission.h_d + transmission.h_u + transmission.h_g_an + transmission.h_a,
        ventilation_h_v_w_per_k: h_v,
        monthly_theta_e_c: climate.outdoor_temperature,
        tau_hours: demand.breakdown.time_constant_hours,
    })
}


/// NTA 8800 tabel 11.8 — aan de gebruiksfunctie gerelateerde specifieke
/// ventilatiecapaciteit `q_usi;spec` [dm³/(s·m²)] én de bezettingstijd-
/// correctiefactor `f_τ` [-].
///
/// Voor utiliteitsfuncties geeft de tabel een vaste `f_τ`; voor de woonfunctie
/// is `f_τ` oppervlakte-afhankelijk: `f_τ = min[(0,38 + A_g · 0,006); 0,8]`.
/// Daarom retourneert deze functie voor de woonfunctie de reeds met `A_g`
/// uitgerekende `f_τ`.
///
/// Bronwaarden (tabel 11.8, kolom `q_usi;spec` en kolom `f_τ`):
/// - Woonfunctie:           q_usi;spec = 0,50  ; f_τ = min[(0,38+A_g·0,006);0,8]
/// - Bijeenkomstfunctie:    q_usi;spec = 1,71  ; f_τ = 0,15  (andere bijeenkomst)
/// - Celfunctie:            q_usi;spec = 0,84  ; f_τ = 0,80
/// - Gezondheidszorg:       q_usi;spec = 1,11  ; f_τ = 0,30  (ander verblijfsgebied)
/// - Industriefunctie:      q_usi;spec = 1,11  ; f_τ = 0,30  (zie OPMERKING hieronder)
/// - Kantoorfunctie:        q_usi;spec = 1,11  ; f_τ = 0,30
/// - Logiesfunctie:         q_usi;spec = 0,84  ; f_τ = 0,40
/// - Onderwijsfunctie:      q_usi;spec = 3,64  ; f_τ = 0,30
/// - Sportfunctie:          q_usi;spec = 0,46  ; f_τ = 0,30
/// - Winkelfunctie:         q_usi;spec = 0,28  ; f_τ = 0,40
///
/// OPMERKING — Industriefunctie: tabel 11.8 geeft voor deze functie alleen een
/// specifieke capaciteit volgens bouwregelgeving (6,5 dm³/s/pers.) en géén
/// kolomwaarden `q_usi;spec`/`f_τ`. Bij gebrek aan een norm-waarde wordt hier
/// de kantoor-rij aangehouden (conservatieve, traceerbare keuze in plaats van
/// een verzonnen getal). Voor industriële projecten met afwijkende
/// procesventilatie moeten de werkelijke debieten worden ingevoerd.
fn nta8800_usi_spec_and_f_tau(
    usage: nta8800_model::zoning::UsageFunction,
    gross_floor_area_m2: f64,
) -> (f64, f64) {
    use nta8800_model::zoning::UsageFunction as UF;
    match usage {
        UF::Woonfunctie => {
            // Tabel 11.8: f_τ = min[(0,38 + A_g · 0,006); 0,8].
            let f_tau = (0.38 + gross_floor_area_m2 * 0.006).min(0.8);
            (0.50, f_tau)
        }
        UF::Bijeenkomstfunctie => (1.71, 0.15),
        UF::Celfunctie => (0.84, 0.80),
        UF::Gezondheidszorgfunctie => (1.11, 0.30),
        // Industriefunctie: geen kolomwaarde in tabel 11.8 → kantoor-rij (zie doc).
        UF::Industriefunctie => (1.11, 0.30),
        UF::Kantoorfunctie => (1.11, 0.30),
        UF::Logiesfunctie => (0.84, 0.40),
        UF::Onderwijsfunctie => (3.64, 0.30),
        UF::Sportfunctie => (0.46, 0.30),
        UF::Winkelfunctie => (0.28, 0.40),
        // OverigeGebruiksfunctie: tabel 11.8 kent deze niet → kantoor-rij als
        // norm-traceerbare default; afwijkende functies vereisen debiet-invoer.
        UF::OverigeGebruiksfunctie => (1.11, 0.30),
    }
}

/// Norm-conforme forfaitaire benodigde luchtvolumestroom van buitenlucht
/// `q_V;ODA;req` [m³/h] volgens NTA 8800:2025+C1:2026 §11.2.2.
///
/// Vervangt de oude handmatige ach-schatting: een ventilatievoud is
/// norm-technisch niet traceerbaar. Wanneer geen luchtdebieten zijn ingevoerd
/// rekent de norm de benodigde buitenluchtstroom uit de gebruiksfunctie en het
/// gebruiksoppervlak `A_g`.
///
/// Keten:
/// 1. `q_usi;spec` + `f_τ` uit tabel 11.8 (`nta8800_usi_spec_and_f_tau`).
/// 2. Woning-ondergrens (11.63): `(q_usi;spec · A_g) ≥ 35 dm³/s`.
/// 3. (11.56): `q_V;ODA;req;des;reken =
///        f_lea;du · f_lea;ahu · f_τ · (q_usi;spec · A_g) · 3,6` [m³/h].
///    `f_lea;du` = 1,10 (kanaal-luchtdichtheidsklasse "Onbekend", tabel 11.9);
///    `f_lea;ahu` = 1,0 (geen AHU, §11.2.2.5.2).
/// 4. (11.57): geïnstalleerde capaciteit onbekend → `q_V;ODA;req;des` =
///    `q_V;ODA;req;des;reken`.
/// 5. (11.22): `q_V;ODA;req = f_ctrl · f_sys · q_V;ODA;req;des /
///        (ε_V · f_prac;req)` met `ε_V = 1` en `f_prac;req = 0,95`.
///    De systeem-correctiefactoren `f_ctrl` en `f_sys` (§11.2.2.3) hangen van
///    het ventilatie-systeemtype af. In de forfait-tak is per definitie geen
///    systeem opgegeven; de norm-neutrale keuze is `f_ctrl = f_sys = 1` (geen
///    vraag-/systeemsturing). Zodra wél een systeem of debieten bekend zijn
///    loopt de berekening via de `nta8800-ventilation`-engine en niet via dit
///    forfait.
fn nta8800_q_v_oda_req_m3_per_h(
    usage: nta8800_model::zoning::UsageFunction,
    gross_floor_area_m2: f64,
) -> f64 {
    let a_g = gross_floor_area_m2.max(0.0);
    let (q_usi_spec, f_tau) = nta8800_usi_spec_and_f_tau(usage, a_g);

    // (11.63) — woning-ondergrens op de absolute capaciteit (q_usi;spec · A_g).
    let mut capacity_dm3s = q_usi_spec * a_g;
    if matches!(usage, nta8800_model::zoning::UsageFunction::Woonfunctie) {
        capacity_dm3s = capacity_dm3s.max(NTA_WONING_MIN_CAPACITY_DM3S);
    }

    // (11.56) — q_V;ODA;req;des;reken in m³/h.
    let q_des_reken_m3h =
        NTA_F_LEA_DU_UNKNOWN * NTA_F_LEA_AHU_NONE * f_tau * capacity_dm3s * NTA_DM3S_TO_M3H;

    // (11.57) — geïnstalleerde capaciteit onbekend → q_V;ODA;req;des = reken.
    let q_des_m3h = q_des_reken_m3h;

    // (11.22) — f_ctrl = f_sys = 1 in de forfait-tak (geen systeem opgegeven).
    let f_ctrl = 1.0;
    let f_sys = 1.0;
    f_ctrl * f_sys * q_des_m3h / (NTA_EPSILON_V * NTA_F_PRAC_REQ)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{
        Construction as SC, ConstructionKind, OpeningKind, SharedGeometry, Space,
    };
    use crate::shared::{BuildingTypeShared, ResidentialType};

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
                        movable_shading: None,
                        obstruction: Default::default(),
                    }],
                    layers: vec![],
                    adjacent_space_id: None,
                    psi_thermal_bridge: None,
                    ground_perimeter_m: None,
                }],
            }],
            ..Default::default()
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
            heating_setpoint_c: 20.0,
            cooling_setpoint_c: 24.0,
            thermal_mass: None,
            internal_gains: None,
        }
    }

    /// Norm-referentie voor de tests die op de §11.2.2-forfait-tak leunen.
    ///
    /// Reproduceert `nta8800_q_v_oda_req_m3_per_h` voor een woonfunctie met
    /// gebruiksoppervlak `a_g` zodat de verwachte H_V-waarden traceerbaar uit
    /// de norm-keten zijn afgeleid (NTA 8800 §11.2.2, formules 11.56/11.57/
    /// 11.22 + tabel 11.8 + ondergrens 11.63):
    ///   q_usi;spec = 0,50 dm³/(s·m²)  (tabel 11.8, woonfunctie)
    ///   f_τ        = min[(0,38 + A_g · 0,006); 0,8]
    ///   capacity   = max(q_usi;spec · A_g; 35) dm³/s   (11.63)
    ///   q_des      = 1,10 · 1,0 · f_τ · capacity · 3,6  (11.56/11.57)
    ///   q_oda;req  = q_des / (1 · 0,95)                 (11.22)
    fn expected_q_v_oda_req_woning(a_g: f64) -> f64 {
        let f_tau = (0.38 + a_g * 0.006).min(0.8);
        let capacity = (0.50 * a_g).max(35.0);
        let q_des = 1.10 * 1.0 * f_tau * capacity * 3.6;
        q_des / (1.0 * 0.95)
    }

    #[test]
    fn thermal_bridges_raise_h_t_and_heating_demand() {
        use crate::geometry::ThermalBridge;
        let i = sample_inputs();

        // Basis (geen koudebruggen), NTA 8800 formule (8.1) met aparte raam-U:
        //   opaak = (150 − 20) × 0,3 = 39,0 W/K
        //   raam  =        20  × 1,4 = 28,0 W/K
        //   H_T = 67,0 W/K.
        let base = compute_tojuli_full(&sample_project(), &i).expect("base ok");
        assert!((base.transmission_h_t_w_per_k - 67.0).abs() < 1e-6);

        // Mét koudebruggen: Σ ψ·L = 0,10·20 + 0,05·30 = 3,5 W/K → H_T = 70,5.
        let mut p = sample_project();
        p.geometry.thermal_bridges = vec![
            ThermalBridge {
                id: "tb-vloer".into(),
                description: "gevel-vloer".into(),
                psi_w_per_mk: 0.10,
                length_m: 20.0,
            },
            ThermalBridge {
                id: "tb-dak".into(),
                description: "gevel-dak".into(),
                psi_w_per_mk: 0.05,
                length_m: 30.0,
            },
        ];
        let with_tb = compute_tojuli_full(&p, &i).expect("with_tb ok");

        assert!(
            (with_tb.transmission_h_t_w_per_k - (67.0 + 3.5)).abs() < 1e-6,
            "H_T met koudebruggen = {}",
            with_tb.transmission_h_t_w_per_k
        );
        // Hogere H_T → hogere warmtebehoefte Q_H;nd (jaarsom).
        let q_h_base: f64 = base.monthly_q_h_nd_mj.as_array().iter().sum();
        let q_h_tb: f64 = with_tb.monthly_q_h_nd_mj.as_array().iter().sum();
        assert!(q_h_tb > q_h_base, "Q_H;nd tb {q_h_tb} moet > basis {q_h_base}");
    }

    #[test]
    fn end_to_end_woning_120m2() {
        let p = sample_project();
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute_tojuli_full ok");
        // NTA 8800 formule (8.1): opaak (150−20)·0,3 + raam 20·1,4 = 39 + 28 = 67 W/K.
        assert!((r.transmission_h_t_w_per_k - 67.0).abs() < 1e-6);
        // Geen ventilatie-config → systeem C-fallback. De infiltratie komt nu
        // uit het NTA 8800 §11.2.2-forfait `q_V;ODA;req` i.p.v. een handmatige
        // ach. A_g = 120 m² (woonfunctie):
        //   f_τ      = min[(0,38 + 120·0,006); 0,8] = min[1,10; 0,8] = 0,80
        //   capacity = max(0,50·120; 35) = max(60; 35) = 60 dm³/s
        //   q_des    = 1,10·1,0·0,80·60·3,6 = 190,08 m³/h
        //   q_oda    = 190,08 / 0,95 = 200,08 m³/h
        // Systeem C: q_V;tot = max(exhaust 0, infil 200,08) = 200,08 m³/h.
        // H_V = q_V;tot × (1212.23/3600) (geen WTW).
        let expected_q_v = expected_q_v_oda_req_woning(120.0);
        let expected_h_v = expected_q_v * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6,
            "verwachte H_V {expected_h_v}, gemeten {}",
            r.ventilation_h_v_w_per_k
        );
        // Q_C;use jaar > 0 (woning heeft ramen, krijgt zonbelasting in zomer)
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.annual_q_c_use_kwh >= 0.0);
        assert!(r.tau_hours > 0.0);
    }

    #[test]
    fn systemd_with_wtw_no_mech_debieten_uses_forfait() {
        // Weesp-scenario: V1→V2-migratie heeft ventilation_system=MechBalanced
        // en heat_recovery=Some(0.85) gezet, maar mech_supply/exhaust ontbreken.
        // Zonder de §11.2.2-forfait zou system_total_airflow(D) = 0 → H_V = 0.
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechBalanced);
        p.shared.heat_recovery = Some(HeatRecovery {
            efficiency: 0.85,
            frost_protection: false,
            supply_temperature: None,
        });
        // mech_supply / mech_exhaust / infiltration blijven None
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok");

        // Met forfait q_V;ODA;req voor woning 120 m² (zie expected_q_v_oda_req_woning)
        // en WTW η=0.85 reductie: H_V = q_oda × (1212/3600) × (1 - 0.85)
        let expected_q_v = expected_q_v_oda_req_woning(120.0);
        let expected_h_v = expected_q_v * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0 * (1.0 - 0.85);
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 0.5,
            "verwachte H_V {expected_h_v} W/K, gemeten {} W/K",
            r.ventilation_h_v_w_per_k
        );
        assert!(r.ventilation_h_v_w_per_k > 0.0, "H_V moet > 0 zijn voor D+WTW");
    }

    #[test]
    fn empty_geometry_uses_gross_area_fallback() {
        let mut p = ProjectV2::new("Empty");
        p.shared.gross_floor_area_m2 = Some(100.0);
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute ok");
        // H_T = 0 want geen constructies
        assert_eq!(r.transmission_h_t_w_per_k, 0.0);
        // Geen ventilatie-config → §11.2.2-forfait `q_V;ODA;req` op A_g = 100 m²
        // (woonfunctie, default-gebouwtype). H_V volgt uit q_V;tot × ρ_a·c_a.
        let expected_q_v = expected_q_v_oda_req_woning(100.0);
        let expected_h_v = expected_q_v * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6,
            "verwachte H_V {expected_h_v}, gemeten {}",
            r.ventilation_h_v_w_per_k
        );
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
                ground_perimeter_m: None,
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
                ground_perimeter_m: None,
            },
        ]);

        let i = sample_inputs();

        // Dit zou moeten slagen zonder panic/error
        let r = compute_tojuli_full(&p, &i).expect("compute_tojuli_full should succeed with adjacent_room and named_unheated");

        // Verifieer dat resultaat valide is (geen NaN/Inf)
        assert!(r.transmission_h_t_w_per_k.is_finite());
        assert!(r.transmission_h_t_w_per_k >= 0.0);

        // De transmission H_T zou nu alleen exterior + unheated moeten bevatten
        // Exterior (formule 8.1, aparte raam-U): opaak (150−20)·0,3 + raam 20·1,4
        //   = 39 + 28 = 67 W/K
        // UnheatedSpace: 15 × 0.8 × b_factor(0.5) = 6 W/K
        // AdjacentRoom: wordt geskipt, dus 0 W/K
        // Verwachte H_T ≈ 67 + 6 = 73 W/K (geen grondcontact → h_g_an genuld)
        assert!(r.transmission_h_t_w_per_k > 67.0); // Minstens de exterior component
        assert!(r.transmission_h_t_w_per_k < 80.0); // Realistisch bovengrens

        // Verifieer dat de rest van het resultaat ook geldig is
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.annual_q_c_use_kwh >= 0.0);
        assert!(r.tau_hours > 0.0);
    }

    #[test]
    fn ventilation_system_d_balanced_with_wtw_uses_engine() {
        // Bouw twee identieke System D-projecten — één mét WTW, één zonder —
        // en verifieer dat de WTW-reductie (1 − η) de H_V die τ voedt verlaagt.
        let mut p_wtw = sample_project();
        p_wtw.shared.ventilation_system = Some(VentilationSystemKind::MechBalanced);
        p_wtw.shared.heat_recovery = Some(HeatRecovery {
            efficiency: 0.85,
            frost_protection: false,
            supply_temperature: None,
        });
        p_wtw.shared.mechanical_supply_m3_per_h = Some(120.0);
        p_wtw.shared.mechanical_exhaust_m3_per_h = Some(120.0);

        // Ceteris paribus: zelfde geometrie + debieten, alleen WTW eraf.
        let mut p_no_wtw = p_wtw.clone();
        p_no_wtw.shared.heat_recovery = None;

        let i = sample_inputs();
        let r_wtw = compute_tojuli_full(&p_wtw, &i).expect("compute ok with System D + WTW");
        let r_no_wtw =
            compute_tojuli_full(&p_no_wtw, &i).expect("compute ok with System D without WTW");

        assert!(r_wtw.ventilation_h_v_w_per_k > 0.0);
        assert!(r_no_wtw.ventilation_h_v_w_per_k > 0.0);

        // Met WTW is de effectieve ventilatie-warmteoverdracht lager:
        // H_V;wtw = H_V;no_wtw × (1 − η).
        assert!(
            r_wtw.ventilation_h_v_w_per_k < r_no_wtw.ventilation_h_v_w_per_k,
            "WTW moet H_V verlagen: wtw={}, no_wtw={}",
            r_wtw.ventilation_h_v_w_per_k,
            r_no_wtw.ventilation_h_v_w_per_k
        );
        // Exacte reductiefactor: η = 0.85 → factor 0.15.
        let expected_ratio = 1.0 - 0.85;
        let actual_ratio = r_wtw.ventilation_h_v_w_per_k / r_no_wtw.ventilation_h_v_w_per_k;
        assert!(
            (actual_ratio - expected_ratio).abs() < 1e-9,
            "verwachte (1−η)-reductie {expected_ratio}, gemeten {actual_ratio}"
        );

        assert!(r_wtw.annual_q_c_use_mj >= 0.0);
        assert!(r_wtw.annual_q_c_use_kwh >= 0.0);
        assert!(r_wtw.tau_hours > 0.0);
    }

    // --- C2.3 — wiring van het §11.2.1 drukmodel in de tojuli-pipeline ----

    #[test]
    fn derive_building_height_prefers_num_storeys() {
        // num_storeys is de betrouwbaarste bron: storeys × 3,0 m bruto.
        let mut p = sample_project();
        p.shared.num_storeys = Some(2);
        let h = derive_building_height_m(&p.shared, &p.geometry);
        assert!((h - 6.0).abs() < 1e-9, "2 verdiepingen → 6,0 m, gemeten {h}");
    }

    #[test]
    fn derive_building_height_falls_back_to_space_sum() {
        // Zonder num_storeys: som van space-hoogtes (binnenwerks + 0,3 m).
        let mut p = sample_project();
        p.shared.num_storeys = None;
        // sample_project heeft één space met height_m = 2,7 → 2,7 + 0,3 = 3,0.
        let h = derive_building_height_m(&p.shared, &p.geometry);
        assert!((h - 3.0).abs() < 1e-9, "één space van 2,7 m → 3,0 m, gemeten {h}");
    }

    #[test]
    fn derive_building_leakage_type_maps_residential_subtypes() {
        // Vrijstaand → grondgebonden vrijstaand hellend dak.
        let mut p = sample_project();
        p.shared.building_type = BuildingTypeShared::Woning {
            subtype: ResidentialType::Detached,
        };
        assert_eq!(
            derive_building_leakage_type(&p.shared),
            BuildingLeakageType::GroundBoundDetachedPitchedRoof
        );
        // Tussenwoning → grondgebonden tussenligging.
        p.shared.building_type = BuildingTypeShared::Woning {
            subtype: ResidentialType::Terraced,
        };
        assert_eq!(
            derive_building_leakage_type(&p.shared),
            BuildingLeakageType::GroundBoundTerracedPitchedRoof
        );
        // Gestapelde woning → meerlaags, gebouw als geheel.
        p.shared.building_type = BuildingTypeShared::Woning {
            subtype: ResidentialType::Stacked,
        };
        assert_eq!(
            derive_building_leakage_type(&p.shared),
            BuildingLeakageType::MultiStoreyWholeBuilding
        );
    }

    /// Met een bekend bouwjaar én C2-scope-hoogte loopt de tojuli-keten via het
    /// norm-exacte §11.2.1.6 drukmodel — niet via de heuristiek. De berekening
    /// moet convergeren en een plausibel resultaat geven.
    #[test]
    fn pressure_model_runs_for_known_build_year_in_c2_scope() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechBalanced);
        p.shared.construction_year = Some(2015); // bekend bouwjaar → C_lea forfait
        p.shared.num_storeys = Some(2); // 6,0 m → binnen C2-scope
        p.shared.mechanical_supply_m3_per_h = Some(150.0);
        p.shared.mechanical_exhaust_m3_per_h = Some(150.0);

        let i = sample_inputs();
        // Convergeert (geen PressureSolverDidNotConverge) → het drukmodel is
        // daadwerkelijk gebruikt en heeft een sluitende massabalans gevonden.
        let r = compute_tojuli_full(&p, &i)
            .expect("drukmodel moet convergeren binnen C2-scope met bekend bouwjaar");
        assert!(r.ventilation_h_v_w_per_k > 0.0);
        assert!(r.tau_hours > 0.0);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    /// Onbekend bouwjaar → geen forfaitaire `C_lea` → het drukmodel zou niet
    /// convergeren. De keten moet dan netjes terugvallen op de heuristiek
    /// i.p.v. te paniekeren met een `PressureSolverDidNotConverge`.
    #[test]
    fn unknown_build_year_falls_back_to_heuristic_without_panic() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechBalanced);
        p.shared.construction_year = None; // geen bouwjaar → geen C_lea
        p.shared.num_storeys = Some(2);
        p.shared.mechanical_supply_m3_per_h = Some(150.0);
        p.shared.mechanical_exhaust_m3_per_h = Some(150.0);

        let i = sample_inputs();
        // Mag NIET met PressureSolverDidNotConverge falen — terugval heuristiek.
        let r = compute_tojuli_full(&p, &i)
            .expect("onbekend bouwjaar moet terugvallen op heuristiek, niet paniekeren");
        // Heuristiek-H_V voor System D (geen WTW): q_V;tot = supply = 150 m³/h.
        let expected_h_v = 150.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6,
            "verwachte heuristiek-H_V {expected_h_v}, gemeten {}",
            r.ventilation_h_v_w_per_k
        );
    }

    /// Een gebouw ≥ 15 m (multi-luchtstroomzone, V2-scope) valt terug op de
    /// heuristiek — geen 1-zone-massabalans improviseren.
    #[test]
    fn building_above_15m_falls_back_to_heuristic() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::MechExhaust);
        p.shared.construction_year = Some(2015); // bekend bouwjaar
        p.shared.num_storeys = Some(6); // 6 × 3,0 = 18 m ≥ 15 m → buiten C2-scope
        p.shared.mechanical_exhaust_m3_per_h = Some(100.0);
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.infiltration_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("hoogbouw moet via heuristiek lopen");
        // Systeem C-heuristiek: q_V;tot = max(exhaust 100, infil 0) = 100 m³/h.
        let expected_h_v = 100.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6,
            "verwachte heuristiek-H_V {expected_h_v}, gemeten {}",
            r.ventilation_h_v_w_per_k
        );
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

        // Systeem B: q_V;tot = max(supply 150, infil 0) = 150 m³/h
        // H_V = 150 × (1212.23/3600) ≈ 50.51 W/K (geen WTW)
        let expected_h_v = 150.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!((r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6);
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

        // Systeem C: q_V;tot = max(exhaust 100, infil 0) = 100 m³/h
        // H_V = 100 × (1212.23/3600) ≈ 33.67 W/K (geen WTW)
        let expected_h_v = 100.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!((r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6);
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

        // Systeem A: q_V;tot = infiltratie = 80 m³/h
        // H_V = 80 × (1212.23/3600) ≈ 26.94 W/K (geen WTW)
        let expected_h_v = 80.0 * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!((r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6);
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    /// QC-bevinding 5: systeem A (natuurlijke ventilatie) zonder ingevoerd
    /// infiltratie-/toevoerdebiet mag geen stille `h_v = 0` opleveren. De
    /// §11.2.2-forfait-tak moet ook voor systeem A triggeren, zodat de
    /// natuurlijke toevoer `q_V;ODA;eff = q_V;ODA;req > 0` gegarandeerd is.
    #[test]
    fn ventilation_system_a_without_infiltration_uses_norm_forfait() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.heat_recovery = None;
        p.shared.infiltration_m3_per_h = None; // geen debiet ingevoerd
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.mechanical_exhaust_m3_per_h = None;

        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i)
            .expect("compute ok with System A zonder infiltratie");

        // Systeem A zonder debiet → §11.2.2-forfait op A_g = 120 m² (woonfunctie).
        // q_V;tot = q_V;ODA;req (natuurlijke toevoer), H_V = q_V;tot × ρ_a·c_a.
        let expected_q_v = expected_q_v_oda_req_woning(120.0);
        let expected_h_v = expected_q_v * AIR_VOLUMETRIC_HEAT_J_PER_M3_K / 3600.0;
        assert!(
            (r.ventilation_h_v_w_per_k - expected_h_v).abs() < 1e-6,
            "verwachte H_V {expected_h_v}, gemeten {}",
            r.ventilation_h_v_w_per_k
        );
        // Kern van de bevinding: geen stille h_v = 0 meer.
        assert!(
            r.ventilation_h_v_w_per_k > 0.0,
            "systeem A zonder infiltratie moet H_V > 0 hebben, niet stil 0"
        );
        assert!(r.annual_q_c_use_mj >= 0.0);
    }

    /// QC-bevinding C2.3-3: systeem A met een expliciet ingevoerd
    /// nul-infiltratiedebiet (`infiltration_m3_per_h = Some(0.0)`). Een
    /// expliciete `Some(0.0)` schakelt de §11.2.2-forfait-terugval uit (die
    /// vuurt alleen op `None`), zodat `flow.infiltration` daadwerkelijk `0`
    /// blijft. De systeem-A-propagatie zet dan `mechanical_supply =
    /// mechanical_exhaust = 0` — precies het caller-contract-scenario uit
    /// bevinding 1 & 2: een volledig dichte schil zonder ventilatie-ontwerp.
    ///
    /// Kern: de keten moet netjes doorlopen — drukmodel met enkel
    /// lek-conductantie `C_lea`, óf heuristiek-terugval — zonder paniek. Er is
    /// bewust géén `debug_assert!` op `mechanical_supply > 0`, dus een
    /// nul-debiet mag de pipeline niet laten crashen.
    #[test]
    fn system_a_with_zero_infiltration_runs_without_panic() {
        // Valide 120 m²-geometrie (anders weigert de demand-keten met
        // InvalidFloorArea) — de test draait om de ventilatie-propagatie,
        // niet om een lege geometrie.
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.heat_recovery = None;
        // Expliciete Some(0.0): zet de §11.2.2-forfait-terugval uit (vuurt enkel
        // op None) → flow.infiltration blijft echt 0 → propagatie zet
        // mechanical_supply = mechanical_exhaust = 0.
        p.shared.infiltration_m3_per_h = Some(0.0);
        p.shared.mechanical_supply_m3_per_h = None;
        p.shared.mechanical_exhaust_m3_per_h = None;
        p.shared.construction_year = Some(2015); // bekend bouwjaar → drukmodel-pad
        p.shared.num_storeys = Some(1); // 3,0 m → binnen C2-scope

        let i = sample_inputs();
        // Mag NIET paniekeren — drukmodel met enkel lek-conductantie of
        // heuristiek-terugval moet de massabalans sluiten.
        let r = compute_tojuli_full(&p, &i)
            .expect("systeem A met nul-infiltratie moet doorlopen zonder paniek");
        assert!(r.ventilation_h_v_w_per_k.is_finite());
        assert!(r.ventilation_h_v_w_per_k >= 0.0);
        assert!(r.annual_q_c_use_mj >= 0.0);
        assert!(r.tau_hours > 0.0);
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

    /// Norm-referentietest met een HARDGECODEERDE verwachtingswaarde — bewust
    /// NIET via `expected_q_v_oda_req_woning` (die helper is een inlining van
    /// de productie-formule, dus alleen een consistentie-check). Deze test
    /// verifieert de norm-correctheid zelf.
    ///
    /// Verwachte waarde, stap-voor-stap geverifieerd tegen NTA 8800:2025+C1:2026
    /// (woonfunctie, A_g = 120 m²; formules 11.22 + 11.56 + 11.57 + 11.63;
    /// tabel 11.8 + tabel 11.9):
    ///   q_usi;spec = 0,50 dm³/(s·m²)                       (tabel 11.8)
    ///   f_τ        = min[(0,38 + 120·0,006); 0,8]
    ///              = min[1,10; 0,8] = 0,80                 (tabel 11.8)
    ///   capaciteit = max(q_usi;spec·A_g; 35)
    ///              = max(0,50·120; 35) = max(60; 35) = 60 dm³/s   (11.63)
    ///   q_des;reken (11.56)
    ///              = f_lea;du · f_lea;ahu · f_τ · capaciteit · 3,6
    ///              = 1,10 · 1,0 · 0,80 · 60 · 3,6 = 190,08 m³/h
    ///   q_des (11.57, installatie onbekend) = 190,08 m³/h
    ///   q_V;ODA;req (11.22)
    ///              = q_des / (ε_V · f_prac;req)
    ///              = 190,08 / (1,0 · 0,95) = 200,08 m³/h
    #[test]
    fn nta8800_q_v_oda_req_woning_120m2_matches_norm_literal() {
        let q = nta8800_q_v_oda_req_m3_per_h(
            nta8800_model::zoning::UsageFunction::Woonfunctie,
            120.0,
        );
        // Hardgecodeerde, PDF-geverifieerde verwachtingswaarde — 200,08 m³/h.
        assert!(
            (q - 200.08).abs() < 0.01,
            "NTA 8800 §11.2.2 forfait voor 120 m² woning moet 200,08 m³/h zijn, gemeten {q}"
        );
    }

    /// Edge-case: A_g = 0 (lege/ontbrekende gross floor area). `a_g.max(0.0)`
    /// laat dit pad uitvoerbaar, maar dan moet de woning-ondergrens (11.63)
    /// van 35 dm³/s correct intreden i.p.v. een capaciteit van 0.
    ///
    /// Verwachte waarde (woonfunctie, A_g = 0; zelfde norm-keten als hierboven):
    ///   f_τ        = min[(0,38 + 0·0,006); 0,8] = min[0,38; 0,8] = 0,38
    ///   capaciteit = max(0,50·0; 35) = max(0; 35) = 35 dm³/s     (11.63)
    ///   q_des;reken (11.56)
    ///              = 1,10 · 1,0 · 0,38 · 35 · 3,6 = 52,668 m³/h
    ///   q_des (11.57) = 52,668 m³/h
    ///   q_V;ODA;req (11.22)
    ///              = 52,668 / (1,0 · 0,95) = 55,440 m³/h
    #[test]
    fn nta8800_q_v_oda_req_woning_zero_area_triggers_minimum() {
        let q = nta8800_q_v_oda_req_m3_per_h(
            nta8800_model::zoning::UsageFunction::Woonfunctie,
            0.0,
        );
        // Hardgecodeerde verwachtingswaarde — woning-ondergrens (11.63) actief.
        assert!(
            (q - 55.44).abs() < 0.01,
            "bij A_g = 0 moet de woning-ondergrens (11.63) intreden → 55,44 m³/h, gemeten {q}"
        );
        // De ondergrens garandeert een strikt positieve toevoer.
        assert!(q > 0.0, "woning-ondergrens moet q_V;ODA;req > 0 garanderen");
    }

    // --- F3d-9: gemeten/verklaarde q_v10;spec (NTA 8800 §11.2.5) -----------

    /// Sommeer de jaarlijkse warmtevraag Q_H;nd (MJ) voor een systeem-A-project
    /// binnen C2-scope met een gegeven `q_v10;spec` (dm³/(s·m²) per A_g).
    ///
    /// Systeem A + `construction_year = 2015` + 1 bouwlaag (3,0 m) garandeert
    /// dat het §11.2.1 drukmodel wordt gebruikt (within_c2_scope &&
    /// effective_q_v10().is_some()), zodat de lek-`C_lea` daadwerkelijk uit
    /// `q_v10;spec` volgt.
    fn q_h_nd_for_q_v10_spec(q_v10_spec: Option<f64>, build_year: Option<u32>) -> f64 {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.heat_recovery = None;
        p.shared.construction_year = build_year;
        p.shared.num_storeys = Some(1); // 3,0 m → binnen C2-scope
        p.shared.q_v10_spec_dm3_s_m2 = q_v10_spec;
        let i = sample_inputs();
        let r = compute_tojuli_full(&p, &i).expect("compute_tojuli_full moet slagen");
        r.monthly_q_h_nd_mj.as_array().iter().sum::<f64>()
    }

    /// Een lekkere gemeten `q_v10;spec` levert een hogere Q_H;nd dan een
    /// luchtdichte waarde: de meting stuurt daadwerkelijk de lek-`C_lea` in het
    /// drukmodel (§11.2.5 → formule (11.85)/(11.86)).
    #[test]
    fn measured_q_v10_spec_scales_heating_demand() {
        let tight = q_h_nd_for_q_v10_spec(Some(0.3), Some(2015));
        let leaky = q_h_nd_for_q_v10_spec(Some(3.0), Some(2015));
        assert!(
            leaky > tight,
            "lekkere schil (q_v10=3,0) moet meer warmtevraag geven dan luchtdicht \
             (q_v10=0,3): leaky {leaky} vs tight {tight}"
        );
    }

    /// De meting wint van het forfait én maakt het bouwjaar irrelevant voor de
    /// lek-`C_lea`: dezelfde `q_v10;spec` levert een identieke Q_H;nd, of het
    /// bouwjaar nu 1965 (fors ander forfait) of onbekend is (§11.2.5).
    #[test]
    fn measured_q_v10_overrides_forfait_and_ignores_build_year() {
        let with_old_year = q_h_nd_for_q_v10_spec(Some(1.2), Some(1965));
        let without_year = q_h_nd_for_q_v10_spec(Some(1.2), None);
        assert!(
            (with_old_year - without_year).abs() < 1e-9,
            "gezette meting moet het bouwjaar irrelevant maken: {with_old_year} vs {without_year}"
        );
    }

    /// Drop-in-equivalentie: een gemeten `q_v10;spec` die exact gelijk is aan
    /// het forfait rekent byte-identiek aan het forfait-pad. Voor een
    /// vrijstaande woning (GroundBoundDetachedPitchedRoof) met bouwjaar 2015
    /// geldt forfait = q_spec(1,0) · f_type(1,4) · f_y(0,7) = 0,98 dm³/(s·m²).
    #[test]
    fn measured_equal_to_forfait_matches_forfait_path() {
        let forfait = q_h_nd_for_q_v10_spec(None, Some(2015));
        let measured_equal = q_h_nd_for_q_v10_spec(Some(0.98), Some(2015));
        assert!(
            (forfait - measured_equal).abs() < 1e-6,
            "meting == forfait (0,98) moet identiek rekenen aan het forfait-pad: \
             {forfait} vs {measured_equal}"
        );
    }

    /// Een negatieve `q_v10;spec` is fysisch onmogelijk en wordt op de
    /// invoergrens expliciet geweigerd — NIET stil naar het forfait
    /// teruggeschoven (zou anders door de `c_lea > 0.0`-guard vallen).
    #[test]
    fn negative_q_v10_spec_is_rejected() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.construction_year = Some(2015);
        p.shared.num_storeys = Some(1);
        p.shared.q_v10_spec_dm3_s_m2 = Some(-0.5);
        let i = sample_inputs();
        let err = compute_tojuli_full(&p, &i).expect_err("negatieve q_v10;spec moet falen");
        assert!(
            matches!(err, TojuliError::InvalidQv10Spec(v) if v == -0.5),
            "verwacht InvalidQv10Spec(-0.5), kreeg {err:?}"
        );
    }

    /// Een niet-eindige `q_v10;spec` (NaN/∞) wordt eveneens geweigerd.
    #[test]
    fn non_finite_q_v10_spec_is_rejected() {
        let mut p = sample_project();
        p.shared.ventilation_system = Some(VentilationSystemKind::Natural);
        p.shared.construction_year = Some(2015);
        p.shared.num_storeys = Some(1);
        p.shared.q_v10_spec_dm3_s_m2 = Some(f64::INFINITY);
        let i = sample_inputs();
        let err = compute_tojuli_full(&p, &i).expect_err("niet-eindige q_v10;spec moet falen");
        assert!(matches!(err, TojuliError::InvalidQv10Spec(_)), "kreeg {err:?}");
    }

    /// `Some(0.0)` is een geldige invoer (perfecte luchtdichtheid): de keten
    /// rekent zonder fout én triggert het forfait NIET — de lek-`C_lea` is
    /// nul, dus de warmtevraag ligt lager dan met het forfait-pad.
    #[test]
    fn zero_q_v10_spec_is_valid_and_bypasses_forfait() {
        let airtight = q_h_nd_for_q_v10_spec(Some(0.0), Some(2015));
        let forfait = q_h_nd_for_q_v10_spec(None, Some(2015));
        assert!(
            airtight < forfait,
            "perfecte luchtdichtheid (q_v10=0) moet minder warmtevraag geven dan het \
             forfait (0,98): airtight {airtight} vs forfait {forfait}"
        );
    }

    /// Regressie-pin: een project zónder `q_v10;spec` blijft byte-identiek
    /// (serde-default None, skip_serializing_if). Bestaande JSON's zonder het
    /// veld deserialiseren zonder waarde en serialiseren zonder de sleutel.
    #[test]
    fn legacy_v2_without_q_v10_spec_round_trip() {
        let json_v2_legacy = r#"{
            "name": "Test Project",
            "building_type": {
                "kind": "woning",
                "subtype": "detached"
            },
            "gross_floor_area_m2": 100.0
        }"#;
        let shared: crate::shared::SharedProject =
            serde_json::from_str(json_v2_legacy).expect("deserialize legacy v2 zonder q_v10;spec");
        assert_eq!(shared.q_v10_spec_dm3_s_m2, None);
        let serialized = serde_json::to_string(&shared).expect("serialize back");
        assert!(!serialized.contains("q_v10_spec_dm3_s_m2"));
    }
}
