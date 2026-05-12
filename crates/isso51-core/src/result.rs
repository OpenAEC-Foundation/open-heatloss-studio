//! Result types for ISSO 51 heat loss calculations.
//!
//! These types represent the output of the calculation engine.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Complete calculation result for an entire project/dwelling.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectResult {
    /// Results per room.
    pub rooms: Vec<RoomResult>,

    /// Building-level summary.
    pub summary: BuildingSummary,
}

/// Calculation result for a single room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoomResult {
    /// Room ID (matches input Room.id).
    pub room_id: String,

    /// Room name.
    pub room_name: String,

    /// Design indoor temperature θ_i in °C.
    pub theta_i: f64,

    /// Transmission heat loss breakdown.
    pub transmission: TransmissionResult,

    /// Infiltration heat loss.
    pub infiltration: InfiltrationResult,

    /// Ventilation heat loss.
    pub ventilation: VentilationResult,

    /// Heating-up allowance (opwarmtoeslag).
    pub heating_up: HeatingUpResult,

    /// System losses (floor/wall/ceiling heating).
    pub system_losses: SystemLossResult,

    /// Total heat loss for this room in W.
    /// Φ_HL,i = Φ_basis + Φ_extra
    pub total_heat_loss: f64,

    /// Basis heat loss (always occurring) in W.
    /// Φ_basis = Φ_T,exterior + Φ_T,unheated + Φ_T,ground + Φ_infiltration + Φ_system
    pub basis_heat_loss: f64,

    /// Extra heat loss (quadratic sum of non-simultaneous) in W.
    /// Φ_extra = √(Φ_vent² + Φ_T,adj² + Φ_hu²)
    pub extra_heat_loss: f64,
}

/// Breakdown of transmission heat losses for a room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransmissionResult {
    /// Specific heat loss to exterior H_T,ie in W/K.
    pub h_t_exterior: f64,

    /// Specific heat loss to adjacent rooms H_T,ia in W/K (intra-dwelling).
    ///
    /// **Design — zero-sum, géén bug:**
    /// Warmteoverdracht tussen binnenruimten binnen dezelfde woning. Positieve
    /// waardes in ruimte A (warmte die wegloopt naar een koelere buurkamer)
    /// worden per definitie gecompenseerd door negatieve waardes in ruimte B
    /// (warmte die binnenkomt). Over de hele woning gesommeerd is dit nul —
    /// conservation of energy — daarom telt dit veld **niet** mee in
    /// `BuildingSummary::total_envelope_loss` of `connection_capacity`.
    ///
    /// Wél gerapporteerd per-room zodat ingenieurs kunnen zien welk vertrek
    /// thermisch gekoppeld is aan welk ander vertrek (diagnose van over-
    /// of onderverwarmen van individuele ruimten).
    ///
    /// EN: Intra-dwelling heat transfer between interior rooms. Excluded
    /// from the building-level totals because positive values in one room
    /// are offset by negative values in its neighbour (energy conservation).
    /// Reported per-room for engineering diagnosis only.
    pub h_t_adjacent_rooms: f64,

    /// Specific heat loss via unheated spaces H_T,io in W/K.
    pub h_t_unheated: f64,

    /// Specific heat loss to neighboring buildings H_T,ib in W/K.
    pub h_t_adjacent_buildings: f64,

    /// Specific heat loss to ground H_T,ig in W/K.
    pub h_t_ground: f64,

    /// Specific heat loss to open water H_T,iw in W/K.
    /// Non-norm category (woonboot use case) — see `BoundaryType::Water`.
    /// `0.0` for projects without any water boundaries.
    #[serde(default)]
    pub h_t_water: f64,

    /// Total transmission heat loss Φ_T in W.
    pub phi_t: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Infiltration heat loss result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InfiltrationResult {
    /// Specific heat loss by infiltration H_i in W/K.
    pub h_i: f64,

    /// Infiltration fraction z_i.
    pub z_i: f64,

    /// Infiltration heat loss Φ_i in W.
    pub phi_i: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Ventilation heat loss result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VentilationResult {
    /// Specific heat loss by ventilation H_v in W/K.
    pub h_v: f64,

    /// Temperature correction factor f_v.
    pub f_v: f64,

    /// Ventilation volume flow q_v in dm³/s (effective, used in calculation).
    pub q_v: f64,

    /// BBL minimum ventilation rate in dm³/s.
    /// Always calculated from room function and floor area (BBL Afdeling 3.6).
    pub q_v_minimum: f64,

    /// Ventilation heat loss Φ_v in W.
    pub phi_v: f64,

    /// In-scope ventilation loss Φ_vent (after subtracting infiltration) in W.
    pub phi_vent: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Heating-up allowance result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeatingUpResult {
    /// Heating-up allowance Φ_hu in W.
    pub phi_hu: f64,

    /// Heating-up factor f_RH in W/m².
    pub f_rh: f64,

    /// Accumulating surface area in m².
    pub accumulating_area: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// System loss result (embedded heating).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemLossResult {
    /// Floor heating loss to ground/crawlspace Φ_verlies1 in W.
    pub phi_floor_loss: f64,

    /// Wall heating loss to exterior/adjacent Φ_verlies2 in W.
    pub phi_wall_loss: f64,

    /// Ceiling heating loss to exterior/adjacent Φ_verlies3 in W.
    pub phi_ceiling_loss: f64,

    /// Total system losses in W.
    pub phi_system_total: f64,

    /// ISSO 51 normreferenties gebruikt bij deze berekening.
    #[serde(skip_deserializing, default)]
    #[schemars(with = "Vec<String>")]
    pub norm_refs: Vec<&'static str>,
}

/// Building-level summary of heat losses.
///
/// Totalen op gebouw-niveau. Let op de bewuste weglating van intra-dwelling
/// transmissie (`TransmissionResult::h_t_adjacent_rooms`): warmte die
/// tussen binnenruimten heen-en-weer stroomt is zero-sum over de woning
/// en wordt dus niet opgeteld bij `total_envelope_loss`,
/// `total_neighbor_loss` of `connection_capacity`. Alleen netto transport
/// uit de woning (schil, buren, grond, water, ventilatie) telt mee in het
/// aansluitvermogen — conform ISSO 51 §4.
///
/// EN: Building totals. Intra-dwelling room-to-room transmission is
/// deliberately excluded because it sums to zero across the dwelling
/// (conservation of energy). Only net exports (envelope, neighbours,
/// ground, water, ventilation) contribute to the connection capacity.
///
/// **Aggregatie-formules op gebouwniveau (erratum 2023):**
/// - `Φ_vent = Φ_v − Φ_i` (formule 3.3): netto ventilatieverlies na aftrek infiltratie.
/// - `Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)` (formule 3.11): kwadratische sommatie
///   van niet-simultane bijdragen.
/// - `aansluitvermogen = Φ_basis + Φ_extra` waarbij Φ_basis lineair wordt opgeteld
///   (envelope + buren + grond + water + infiltratie + systeem).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BuildingSummary {
    /// Total transmission loss through building envelope in W.
    pub total_envelope_loss: f64,

    /// Total transmission loss to neighbors in W.
    pub total_neighbor_loss: f64,

    /// Total ventilation/infiltration loss in W.
    ///
    /// Lineaire som van Σ `room.ventilation.phi_v` (gross, vóór aftrek
    /// infiltratie). Bewaard voor backwards-compatible rapportage. Voor
    /// gebouwsom-conformiteit (formule 3.3) gebruik `phi_vent_building`.
    pub total_ventilation_loss: f64,

    /// Total heating-up allowance in W.
    pub total_heating_up: f64,

    /// Total system losses in W.
    pub total_system_losses: f64,

    /// Connection capacity (aansluitvermogen) of the dwelling in W.
    ///
    /// Definitie (erratum 2023): `Φ_basis_total + Φ_extra_quadratic` waarbij
    /// `Φ_extra_quadratic = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)` op gebouwniveau.
    /// Dit is een **breaking change** t.o.v. eerdere versies waarin deze
    /// waarde een lineaire optelsom van W/K-bijdragen was.
    pub connection_capacity: f64,

    /// Contribution to collective installation in W (if applicable).
    pub collective_contribution: f64,

    /// Σ van per-vertrek basis-verlies (inclusief systeemverliezen) in W.
    ///
    /// Basis omvat alle continue, simultane bijdragen op gebouwniveau:
    /// envelope + grond + water + infiltratie + systeemverliezen. Intra-woning
    /// transmissie (`h_t_adjacent_rooms`) is zero-sum en valt weg in de
    /// gebouwsom.
    #[serde(default)]
    pub phi_basis_total: f64,

    /// Φ_vent op gebouwniveau in W: `Σ Φ_v − Σ Φ_i` (erratum 2023, formule 3.3).
    ///
    /// Niet-negatief geclampt: als infiltratie groter is dan ventilatie
    /// (zeldzaam, alleen bij zeer lekke woning met balanced ventilation)
    /// wordt 0 gerapporteerd — een negatieve netto ventilatieverlies is
    /// fysisch niet zinvol in de aansluitvermogen-context.
    #[serde(default)]
    pub phi_vent_building: f64,

    /// Σ van per-vertrek transmissieverlies naar aangrenzende gebouwen in W.
    ///
    /// `Φ_T,iaBE = Σ (h_t_adjacent_buildings × Δθ)` per vertrek. Gaat als
    /// kwadratische component mee in `phi_extra_quadratic`.
    #[serde(default)]
    pub phi_t_iabe_building: f64,

    /// Σ van per-vertrek opwarmtoeslag Φ_hu in W.
    ///
    /// Gaat als kwadratische component mee in `phi_extra_quadratic`.
    #[serde(default)]
    pub phi_hu_building: f64,

    /// Kwadratische som van niet-simultane bijdragen op gebouwniveau in W.
    ///
    /// `√(phi_vent_building² + phi_t_iabe_building² + phi_hu_building²)`
    /// conform erratum 2023 formule 3.11. Telt op met `phi_basis_total`
    /// tot `connection_capacity`.
    #[serde(default)]
    pub phi_extra_quadratic: f64,
}
