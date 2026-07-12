//! NTA 8800:2025+C1:2026 §11.2.1.5/§11.2.1.6 — norm-exacte massabalans- en
//! `p_z;ref`-drukoplosroutine.
//!
//! Deze module lost de **interne referentiedruk** `p_z;ref` (Pa) van één
//! luchtstroomzone op door de massabalans uit NTA 8800 formule (11.5) op te
//! lossen volgens de in §11.2.1.6 voorgeschreven 12-stappen-routine. Met de
//! gevonden `p_z;ref` worden vervolgens de effectieve in- en uitgaande
//! luchtvolumestromen per stroomtype bepaald (§11.2.1.7, formules
//! (11.19)-(11.21)).
//!
//! De module is **puur** en **geïsoleerd**: hij wordt (nog) door niets in deze
//! crate aangeroepen behalve zijn eigen tests. De inbedding in
//! [`crate::calculate_ventilation`] en de TO-juli-keten is C2.3-scope.
//!
//! # Norm-keten
//!
//! | Formule | Onderdeel | Functie |
//! |---|---|---|
//! | (11.1)  | Externe druk `p_e;path;i` | [`external_pressure`] |
//! | (11.4)  | Luchtdichtheid `ρ_T` | [`air_density`] |
//! | (11.2)/(11.3) | Massastroom per opening + mechanisch | [`mass_balance_sum`] |
//! | (11.5)  | Massabalans `Σq_m = 0` | [`mass_balance_sum`] |
//! | (11.11)-(11.13) | Drukverschil `Δp_path;i` | [`pressure_difference`] |
//! | (11.14) | Nauwkeurigheidscriterium `x` | [`accuracy_threshold`] |
//! | (11.15)-(11.18) | Iteratieve `p_z;ref`-routine | [`solve_p_z_ref`] |
//! | (11.19)-(11.21) | Effectieve luchtvolumestromen | [`solve_zone_airflow`] |
//!
//! Norm-bron: NTA 8800:2025+C1:2026 PDF p. 439-447. NEN-licentie 3BM, intern
//! gebruik.
//!
//! # C2-scope: één luchtstroomzone, gebouwhoogte `< 15 m`
//!
//! De solver modelleert **één luchtstroomzone**. Tabel 11.1 splitst een
//! rekenzone met `H ≥ 15 m` op in 2 of 3 luchtstroomzones, elk met een eigen
//! massabalans — dat is V2-scope. Roept een consumer de solver aan met een
//! [`BuildingPressureContext`] buiten C2-scope, dan geeft
//! [`solve_zone_airflow`] een [`VentilationError::PressureSolverDidNotConverge`]
//! noch een stille 1-zone-benadering: de scope-toets hoort vóór de aanroep te
//! gebeuren (zie [`BuildingPressureContext::within_c2_scope`]). Voor robuustheid
//! valt de solver bij `H ≥ 15 m` terug op een **gedocumenteerde
//! 1-zone-benadering** (hele gebouwhoogte als één zone) — zie
//! [`build_openings`].

use nta8800_model::time::Month;
use nta8800_tables::climate::de_bilt::DE_BILT_WIND_SPEED;

use crate::calc::infiltration::q_v1_lea_ref;
use crate::errors::VentilationError;
use crate::model::{AirFlow, BuildingPressureContext, VentilationSystem, WtwSpecification};
use crate::tables::{HeightClass, FLOW_EXPONENT_LEAKAGE, FLOW_EXPONENT_VENTILATION};

// ===========================================================================
// Norm-constanten — NTA 8800 formules (11.1)/(11.4)/(11.12)
// ===========================================================================

/// Referentieluchtdichtheid `ρ_a;ref` — dichtheid van droge lucht op zeeniveau
/// bij 293 K, in kg/m³.
///
/// NTA 8800:2025+C1:2026 formule (11.1)/(11.4), PDF p. 439-441.
pub const RHO_A_REF_KG_PER_M3: f64 = 1.205;

/// Referentietemperatuur `T_ref` / referentiebuitentemperatuur `T_e;ref`
/// — 293 K.
///
/// In de norm zijn dit twee symbolen met dezelfde getalswaarde: `T_ref`
/// (formule (11.4), dichtheidsreferentie) en `T_e;ref` (formules (11.1),
/// (11.12), (11.13), referentiebuitentemperatuur). Beide zijn 293 K.
///
/// NTA 8800:2025+C1:2026 formules (11.1)/(11.4)/(11.12), PDF p. 439-443.
pub const T_REF_K: f64 = 293.0;

/// Zwaartekrachtversnelling `g`, in m/s².
///
/// NTA 8800:2025+C1:2026 formule (11.1)/(11.12), PDF p. 439/443.
pub const GRAVITY_M_PER_S2: f64 = 9.81;

/// Omrekenoffset °C → K: `T[K] = ϑ[°C] + 273`.
///
/// De norm hanteert exact `273` (niet 273,15) — zie formule (11.1)
/// (`ϑ_e;avg;mi + 273`) en de definitie `T_e = ϑ_e;avg;mi + 273` bij
/// formule (11.2)/(11.3).
const CELSIUS_TO_KELVIN_OFFSET: f64 = 273.0;

// ===========================================================================
// Iteratie-parameters — NTA 8800 §11.2.1.6 routine
// ===========================================================================

/// Stapgrootte voor de bracket-zoektocht, in Pa.
///
/// NTA 8800 §11.2.1.6 routine stap 4 (`p_z;ref;b = p_z;ref;a + 2 Pa`) en
/// stap 7 (formule (11.17), `± 2 Pa`).
const BRACKET_STEP_PA: f64 = 2.0;

/// Tabel 11.1 — aandeel van de lek-`C_lea` op de **loef-** resp.
/// **lijzijde**-opening (rij "overige rekenzones met `H < 15 m`").
const LEA_FACADE_SHARE: f64 = 0.4;

/// Tabel 11.1 — aandeel van de lek-`C_lea` op de **dak**-opening
/// (rij "overige rekenzones met `H < 15 m`").
const LEA_ROOF_SHARE: f64 = 0.2;

/// Tabel 11.1 — aandeel van een natuurlijke ventilatie-`C` op de **loef-**
/// resp. **lijzijde**-opening.
const VENT_FACADE_SHARE: f64 = 0.5;

/// Harde iteratie-cap voor de volledige `p_z;ref`-routine (bracket-fase +
/// bisectie).
///
/// De norm geeft geen expliciete cap — de routine eindigt op het
/// nauwkeurigheidscriterium (11.14). Deze cap is een **defensieve
/// terminatie**: hij voorkomt een oneindige lus bij numeriek pathologische
/// invoer (bv. een opening-set zonder enige conductantie). Bij overschrijding
/// → [`VentilationError::PressureSolverDidNotConverge`].
///
/// 200 iteraties is ruim: een bisectie halveert het bracket-interval per stap,
/// dus na ~50 stappen is `p_z;ref` tot machineprecisie bepaald. De extra marge
/// dekt een lange bracket-zoekfase.
const MAX_SOLVER_ITERATIONS: u32 = 200;

// ===========================================================================
// Opening — één forfaitaire opening uit NTA 8800 tabel 11.1
// ===========================================================================

/// Eén forfaitaire opening in de gebouwschil — een luchtstroompad uit
/// NTA 8800 tabel 11.1.
///
/// De solver modelleert de gebouwschil als een set [`Opening`]s. Elke opening
/// draagt via formule (11.19) (`q_V = C · |Δp|^n`) bij aan de massabalans
/// (11.5). De `wind_pressure_coeff` en `height_m` bepalen via formule (11.1)
/// de externe druk; `conductance_c` en `flow_exponent_n` de luchtstroom bij
/// een gegeven drukverschil.
///
/// # Eenheden
///
/// | Veld | Eenheid |
/// |---|---|
/// | `height_m` (`H_path;i`) | m |
/// | `wind_pressure_coeff` (`C_p;i`) | dimensieloos |
/// | `conductance_c` (`C_path;i`) | m³/(h·Paⁿ) |
/// | `flow_exponent_n` (`n_i`) | dimensieloos |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opening {
    /// Hoogte `H_path;i` van de opening boven maaiveld, in m
    /// (NTA 8800 tabel 11.1).
    pub height_m: f64,

    /// Dimensieloze winddrukcoëfficiënt `C_p;i` (NTA 8800 tabel 11.3).
    pub wind_pressure_coeff: f64,

    /// Luchtdoorlatendheidscoëfficiënt `C_path;i`, in m³/(h·Paⁿ)
    /// (NTA 8800 tabel 11.1 / §11.2.2.2).
    pub conductance_c: f64,

    /// Stromingsexponent `n_i` (NTA 8800 tabel 11.2 — `n_lea = 0,67` voor
    /// lekverliezen, `n_vent = 0,5` voor ventilatievoorzieningen).
    pub flow_exponent_n: f64,
}

impl Opening {
    /// Bouw een [`Opening`] zonder validatie — een pure constructor.
    #[must_use]
    pub const fn new(
        height_m: f64,
        wind_pressure_coeff: f64,
        conductance_c: f64,
        flow_exponent_n: f64,
    ) -> Self {
        Self {
            height_m,
            wind_pressure_coeff,
            conductance_c,
            flow_exponent_n,
        }
    }
}

// ===========================================================================
// Formule (11.4) — luchtdichtheid
// ===========================================================================

/// Luchtdichtheid `ρ_T` bij temperatuur `T` (K), volgens NTA 8800
/// formule (11.4).
///
/// # Norm-afleiding — formule (11.4)
///
/// ```text
/// ρ_T = ρ_a;ref · T_ref / T
/// ```
///
/// waarin `ρ_a;ref = 1,205 kg/m³` (293 K) en `T_ref = 293 K`. De ideale-gaswet
/// bij constante druk: dichtheid is omgekeerd evenredig met absolute
/// temperatuur.
///
/// # Parameters
///
/// - `temp_k`: luchttemperatuur `T`, in K. Moet `> 0` zijn.
///
/// # Resultaat
///
/// `ρ_T` in kg/m³.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.4), §11.2.1.5, PDF p. 441.
#[must_use]
pub fn air_density(temp_k: f64) -> f64 {
    // Formule (11.4): ρ_T = ρ_a;ref · T_ref / T.
    RHO_A_REF_KG_PER_M3 * T_REF_K / temp_k
}

/// Omrekening °C → K met de norm-offset `273` (NTA 8800 formule (11.1)).
#[must_use]
fn celsius_to_kelvin(theta_c: f64) -> f64 {
    theta_c + CELSIUS_TO_KELVIN_OFFSET
}

// ===========================================================================
// Formule (11.1) — externe druk bij luchtvolumestroom
// ===========================================================================

/// Externe druk `p_e;path;i` bij luchtvolumestroom `i` in maand `mi`, in Pa,
/// volgens NTA 8800 formule (11.1).
///
/// # Norm-afleiding — formule (11.1)
///
/// ```text
/// p_e;path;i,zi,mi = ρ_a;ref · (T_e;ref / (ϑ_e;avg;mi + 273))
///                  · (0,5 · C_p;i · u_site;mi² − H_path;i · g)
/// ```
///
/// waarin:
/// - `ρ_a;ref = 1,205 kg/m³` ([`RHO_A_REF_KG_PER_M3`]);
/// - `T_e;ref = 293 K` ([`T_REF_K`]);
/// - `ϑ_e;avg;mi` — maandgemiddelde buitentemperatuur, in °C (tabel 17.1);
/// - `C_p;i` — winddrukcoëfficiënt van de opening (tabel 11.3,
///   [`Opening::wind_pressure_coeff`]);
/// - `u_site;mi` — windsnelheid op locatie, in m/s (tabel 17.1,
///   [`DE_BILT_WIND_SPEED`]);
/// - `H_path;i` — hoogte van de opening boven maaiveld, in m (tabel 11.1);
/// - `g = 9,81 m/s²` ([`GRAVITY_M_PER_S2`]).
///
/// De factor `0,5 · C_p;i · u_site;mi²` is de winddrukterm; `−H_path;i · g`
/// de hydrostatische (stack-) term. De voorfactor `T_e;ref / (ϑ_e + 273)`
/// schaalt naar de actuele buitenluchtdichtheid.
///
/// # Parameters
///
/// - `opening`: de forfaitaire opening (levert `C_p;i` en `H_path;i`).
/// - `month`: de maand `mi` — bepaalt `u_site;mi` uit [`DE_BILT_WIND_SPEED`].
/// - `theta_e_c`: maandgemiddelde buitentemperatuur `ϑ_e;avg;mi`, in °C.
///
/// # Resultaat
///
/// `p_e;path;i` in Pa.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.1), §11.2.1.4, PDF p. 439.
#[must_use]
pub fn external_pressure(opening: &Opening, month: Month, theta_e_c: f64) -> f64 {
    let u_site = DE_BILT_WIND_SPEED[month]; // m/s, tabel 17.1
    // Dimensieloze dichtheidsverhouding T_e;ref / (ϑ_e;avg;mi + 273).
    let density_ratio = T_REF_K / celsius_to_kelvin(theta_e_c);
    // Winddrukterm 0,5·C_p·u² + hydrostatische term −H·g.
    let wind_term = 0.5 * opening.wind_pressure_coeff * u_site * u_site;
    let stack_term = -opening.height_m * GRAVITY_M_PER_S2;
    // Formule (11.1).
    RHO_A_REF_KG_PER_M3 * density_ratio * (wind_term + stack_term)
}

// ===========================================================================
// Formules (11.11)-(11.13) — drukverschil over een opening
// ===========================================================================

/// Hydrostatische interne-drukcorrectie `ρ_a;ref · H_path;i · g · (T_e;ref / T_i)`
/// uit NTA 8800 formules (11.12)/(11.13), in Pa.
#[must_use]
fn internal_stack_term(height_m: f64, t_int_k: f64) -> f64 {
    RHO_A_REF_KG_PER_M3 * height_m * GRAVITY_M_PER_S2 * (T_REF_K / t_int_k)
}

/// Drukverschil `Δp_path;i` over een opening, in Pa, volgens NTA 8800
/// formules (11.11)/(11.12).
///
/// # Norm-afleiding — formules (11.11) + (11.12)
///
/// ```text
/// Δp_path;i = p_e;path;i − p_z;path;i                                  (11.11)
/// p_z;path;i = p_z;ref − ρ_a;ref · H_path;i · g · (T_e;ref / T_int;set) (11.12)
/// ```
///
/// gecombineerd:
///
/// ```text
/// Δp_path;i = p_e;path;i − p_z;ref + ρ_a;ref · H_path;i · g · (T_e;ref / T_i)
/// ```
///
/// Een **positief** `Δp_path;i` betekent een **ingaande** luchtstroom
/// (buitendruk hoger dan binnendruk ter plaatse van de opening); een
/// **negatief** `Δp_path;i` een **uitgaande** luchtstroom (§11.2.1.6
/// opmerking 1).
///
/// # Parameters
///
/// - `opening`: de opening (levert `H_path;i` en `C_p;i`).
/// - `month`, `theta_e_c`: voor de externe druk (11.1).
/// - `p_z_ref`: de interne referentiedruk `p_z;ref`, in Pa.
/// - `t_int_k`: binnentemperatuur `T_int;set;zi`, in K.
///
/// Referentie: NTA 8800:2025+C1:2026 formules (11.11)/(11.12), §11.2.1.6,
/// PDF p. 443.
#[must_use]
pub fn pressure_difference(
    opening: &Opening,
    month: Month,
    theta_e_c: f64,
    p_z_ref: f64,
    t_int_k: f64,
) -> f64 {
    let p_e = external_pressure(opening, month, theta_e_c);
    // p_z;path;i = p_z;ref − stack-term (formule (11.12)).
    let p_z_path = p_z_ref - internal_stack_term(opening.height_m, t_int_k);
    // Δp_path;i = p_e − p_z;path;i (formule (11.11)).
    p_e - p_z_path
}

// ===========================================================================
// Tabel 11.1 + §11.2.2.2 — forfaitaire opening-set
// ===========================================================================

/// Bouw de forfaitaire opening-set voor één luchtstroomzone uit NTA 8800
/// tabel 11.1 + §11.2.2.2.
///
/// # Lek-openingen — tabel 11.1
///
/// Voor de C2-doelgroep ("overige rekenzones met `H < 15 m`", d.w.z.
/// bouwjaar ≥ 1992 of een begane-grondvloer die geen kruipruimte begrenst)
/// verdeelt tabel 11.1 de lek-luchtdoorlatendheidscoëfficiënt `C_lea` over
/// **drie** openingen:
///
/// | Opening | `H_path` | `C_path` | `C_p` (tabel 11.3, Laag) |
/// |---|---|---|---|
/// | Loefzijde | `0,5·H` | `0,4·C_lea` | `+0,25` |
/// | Lijzijde  | `0,5·H` | `0,4·C_lea` | `−0,50` |
/// | Dak       | `H`     | `0,2·C_lea` | `−0,60` |
///
/// De vloer-lekfactor (`0,15·C_lea` resp. de aparte pre-1992-kruipruimterij)
/// is **niet** meegenomen: voor gebouwen van na 1992 zijn de lekverliezen via
/// de begane-grondvloer per norm-noot (PDF p. 438) verwaarloosbaar. De
/// pre-1992-kruipruimtevariant is V2-scope.
///
/// `C_lea` volgt uit het infiltratie-referentiedebiet `q_v1;lea;ref`
/// (formule (11.85), [`q_v1_lea_ref`]): omdat `q_v1;lea;ref` per definitie
/// het debiet bij `Δp = 1 Pa` is, geldt via formule (11.84)
/// `C_lea = q_v1;lea;ref / 1^n_lea = q_v1;lea;ref`.
///
/// # Natuurlijke ventilatie-openingen — §11.2.2.2
///
/// Afhankelijk van het ventilatiesysteem komen er natuurlijke
/// ventilatie-openingen bij. Tabel 11.1 verdeelt elke natuurlijke
/// ventilatiestroom over een **loef-** en **lijzijde**-opening op `0,5·H`,
/// elk met `0,5·C`. De conductantie `C` volgt uit formules (11.25)-(11.28):
/// `C = q_V;ODA;req / 1^n_vent = q_V;ODA;req` (debiet bij `Δp = 1 Pa`).
///
/// | Systeem (NTA 8800-symbool) | Natuurlijke vent-openingen |
/// |---|---|
/// | A — `NATURAL_OP`  | `C_vent;in` **én** `C_vent;out` (§11.2.2.2.1) |
/// | B — `SUPPLY_OP`   | alleen `C_vent;out` — `C_vent;in = 0` (§11.2.2.2.2) |
/// | C — `EXTRACT_OP`  | alleen `C_vent;in` — `C_vent;out = 0` (§11.2.2.2.3) |
/// | D/E — `BALANCED_OP` | geen — `C_vent;in = C_vent;out = 0` (§11.2.2.2.4) |
///
/// Voor de natuurlijke vent-conductantie geldt `q_V;ODA;req`. In de C2-aanpak
/// nemen we hiervoor de relevante mechanische-debietwaarde uit [`AirFlow`]
/// als forfaitaire `q_V;ODA;req`: bij systeem B/C is dat het enige mechanische
/// debiet, bij systeem A is er geen mechanisch debiet en gebruiken we de
/// `infiltration`-waarde niet (systeem A heeft geen ventilatievoorziening met
/// een `q_V;ODA;req` los van infiltratie — voor systeem A levert `flow` geen
/// natuurlijke vent-conductantie tenzij `mechanical_supply`/`exhaust` als
/// proxy voor de ontwerp-ventilatiestroom is gevuld).
///
/// # `H ≥ 15 m` — 1-zone-benadering
///
/// Tabel 11.1 splitst een rekenzone met `H ≥ 15 m` op in meerdere
/// luchtstroomzones. Dat is V2-scope. Valt `ctx.building_height_m ≥ 15`, dan
/// bouwt deze functie tóch één opening-set met de hele gebouwhoogte als één
/// zone (de "`H < 15 m`"-verdeling toegepast op de volledige `H`). Dit is een
/// **gedocumenteerde benadering**, geen norm-conforme multi-zone-berekening —
/// consumers horen [`BuildingPressureContext::within_c2_scope`] te toetsen
/// vóór ze de solver aanroepen.
///
/// # Parameters
///
/// - `system`: het ventilatiesysteem (bepaalt welke natuurlijke
///   vent-openingen erbij komen).
/// - `flow`: de luchtstromen — levert de forfaitaire `q_V;ODA;req` voor de
///   natuurlijke vent-conductantie.
/// - `ctx`: de gebouw-drukcontext — levert `H` en `C_lea` (via
///   [`BuildingPressureContext::forfait_q_v10`] of, zo nodig, een meetwaarde).
///
/// # Resultaat
///
/// De volledige opening-set voor de massabalans. Leeg alleen als zowel
/// `C_lea` als alle vent-conductanties nul zijn.
///
/// Referentie: NTA 8800:2025+C1:2026 tabel 11.1 (§11.2.1.2, PDF p. 430-431) +
/// §11.2.2.2 (PDF p. 451-454).
#[must_use]
pub fn build_openings(
    system: VentilationSystem,
    flow: &AirFlow,
    ctx: &BuildingPressureContext,
) -> Vec<Opening> {
    // --- C_lea uit het forfaitaire infiltratie-referentiedebiet -----------
    // q_v1;lea;ref is per definitie het lek-debiet bij Δp = 1 Pa; formule
    // (11.84) geeft C_lea = q_v1;lea;ref / Δp^n_lea = q_v1;lea;ref bij 1 Pa.
    // Prioriteit (§11.2.5): een gemeten/verklaarde `q_v10;lea;ref` wint van het
    // forfait; zonder meting én zonder bekend bouwjaar (effective_q_v10 → None)
    // is er geen forfaitaire C_lea — dan blijft de lek-bijdrage 0.
    let c_lea = ctx
        .effective_q_v10()
        .map_or(0.0, |q_v10| q_v1_lea_ref(q_v10, ctx.gross_floor_area_m2));

    // C2-scope: H < 15 m → één luchtstroomzone met de hele gebouwhoogte.
    // H ≥ 15 m → 1-zone-benadering (zie doc-comment); de hoogteklasse
    // bepaalt de winddrukcoëfficiënten C_p uit tabel 11.3.
    let cp = HeightClass::from_height(ctx.building_height_m).wind_pressure_coefficients();
    let h = ctx.building_height_m;
    let half_h = 0.5 * h;

    let mut openings = Vec::new();

    // --- Lek-openingen (tabel 11.1, "overige rekenzones met H < 15 m") ----
    // Loef 0,4·C_lea op 0,5·H; lij 0,4·C_lea op 0,5·H; dak 0,2·C_lea op H.
    // Vloer-lekfactor weggelaten (post-1992 verwaarloosbaar, PDF p. 438).
    if c_lea > 0.0 {
        openings.push(Opening::new(
            half_h,
            cp.windward,
            LEA_FACADE_SHARE * c_lea,
            FLOW_EXPONENT_LEAKAGE,
        ));
        openings.push(Opening::new(
            half_h,
            cp.leeward,
            LEA_FACADE_SHARE * c_lea,
            FLOW_EXPONENT_LEAKAGE,
        ));
        openings.push(Opening::new(
            h,
            cp.roof,
            LEA_ROOF_SHARE * c_lea,
            FLOW_EXPONENT_LEAKAGE,
        ));
    }

    // --- Natuurlijke ventilatie-openingen (§11.2.2.2) ---------------------
    // Tabel 11.1 verdeelt elke natuurlijke vent-stroom over loef + lij op
    // 0,5·H, elk met 0,5·C. C = q_V;ODA;req (formules (11.25)-(11.28)).
    let mut push_natural_vent = |conductance: f64| {
        if conductance > 0.0 {
            openings.push(Opening::new(
                half_h,
                cp.windward,
                VENT_FACADE_SHARE * conductance,
                FLOW_EXPONENT_VENTILATION,
            ));
            openings.push(Opening::new(
                half_h,
                cp.leeward,
                VENT_FACADE_SHARE * conductance,
                FLOW_EXPONENT_VENTILATION,
            ));
        }
    };

    match system {
        // §11.2.2.2.1 NATURAL_OP — natuurlijke toe- én afvoer.
        // C_vent;in en C_vent;out beide = q_V;ODA;req. In de C2-aanpak nemen
        // we de aanwezige mechanische-debietvelden als forfaitaire
        // q_V;ODA;req-proxy (systeem A heeft geen mechanisch debiet → 0).
        VentilationSystem::A => {
            push_natural_vent(flow.mechanical_supply);
            push_natural_vent(flow.mechanical_exhaust);
        }
        // §11.2.2.2.2 SUPPLY_OP — alleen natuurlijke afvoer (C_vent;in = 0).
        // q_V;ODA;req = de mechanische toevoer (q_V;SUP;eff = q_V;ODA;req).
        VentilationSystem::B => {
            push_natural_vent(flow.mechanical_supply);
        }
        // §11.2.2.2.3 EXTRACT_OP — alleen natuurlijke toevoer (C_vent;out = 0).
        // q_V;ODA;req = de mechanische afvoer.
        VentilationSystem::C => {
            push_natural_vent(flow.mechanical_exhaust);
        }
        // §11.2.2.2.4 BALANCED_OP — geen natuurlijke vent-openingen.
        VentilationSystem::D { .. } | VentilationSystem::E => {}
    }

    openings
}

// ===========================================================================
// Mechanische massastromen — NTA 8800 formules (11.2)/(11.3) + §11.2.2.2
// ===========================================================================

/// De mechanische bijdrage aan de massabalans (11.5), in kg/h.
///
/// De mechanische toe- en afvoer zijn **drukonafhankelijk** (vaste debieten),
/// dus hun massabijdrage hangt niet van `p_z;ref` af. Per §11.2.1.6 + tabel
/// 11.4:
/// - toevoer `q_m;V;SUP;dis` — ingaand, `+ρ_a;e · q_V;SUP;eff` (dichtheid bij
///   buitentemperatuur, formule (11.2));
/// - afvoer `q_m;V;ETA;dis` — uitgaand, `−ρ_a;zi · q_V;ETA;eff` (dichtheid bij
///   binnentemperatuur, formule (11.3); `q_V;ETA;eff` is per §11.2.2.2.4
///   negatief, hier als magnitude met expliciet minteken).
#[derive(Debug, Clone, Copy)]
struct MechanicalMassFlow {
    /// `+ρ_a;e · q_V;SUP;eff` − ingaande mechanische toevoer, in kg/h.
    supply_in: f64,
    /// `−ρ_a;zi · q_V;ETA;eff` − uitgaande mechanische afvoer, in kg/h.
    exhaust_out: f64,
}

impl MechanicalMassFlow {
    /// Som van de mechanische massastromen, in kg/h.
    fn sum(self) -> f64 {
        self.supply_in + self.exhaust_out
    }
}

/// Bepaal de mechanische massastromen uit het systeemtype + de luchtstromen.
///
/// Per §11.2.2.2 levert elk systeem een mechanische toevoer (`q_V;SUP;eff`)
/// en/of afvoer (`q_V;ETA;eff`):
/// - A — geen mechanische stromen;
/// - B — alleen toevoer (`q_V;SUP;eff = mechanical_supply`);
/// - C — alleen afvoer (`q_V;ETA;eff = mechanical_exhaust`);
/// - D/E — toevoer **en** afvoer.
fn mechanical_mass_flow(
    system: VentilationSystem,
    flow: &AirFlow,
    rho_e: f64,
    rho_i: f64,
) -> MechanicalMassFlow {
    let (q_supply, q_exhaust) = match system {
        VentilationSystem::A => (0.0, 0.0),
        VentilationSystem::B => (flow.mechanical_supply, 0.0),
        VentilationSystem::C => (0.0, flow.mechanical_exhaust),
        VentilationSystem::D { .. } | VentilationSystem::E => {
            (flow.mechanical_supply, flow.mechanical_exhaust)
        }
    };
    MechanicalMassFlow {
        // Formule (11.2): ingaande toevoer bij buitenluchtdichtheid.
        supply_in: rho_e * q_supply,
        // Formule (11.3): uitgaande afvoer bij binnenluchtdichtheid (−).
        exhaust_out: -rho_i * q_exhaust,
    }
}

// ===========================================================================
// Formule (11.5) — massabalans
// ===========================================================================

/// Som van de massastromen `Σq_m` van de massabalans (11.5), in kg/h, bij een
/// gegeven `p_z;ref`.
///
/// # Norm-afleiding — formule (11.5)
///
/// De massabalans sommeert álle mechanische en natuurlijke massastromen; de
/// oplossing `p_z;ref` is de waarde waarvoor `Σq_m = 0`. Deze functie geeft
/// `Σq_m` zelf — de routine in [`solve_p_z_ref`] zoekt het nulpunt.
///
/// Per opening (formules (11.2)/(11.3) + (11.19)):
/// - `q_V = C · |Δp|^n` — de luchtvolumestroom (11.19);
/// - bij **ingaande** stroom (`Δp > 0`): `+ρ_a;e · q_V` (11.2, dichtheid bij
///   buitentemperatuur);
/// - bij **uitgaande** stroom (`Δp < 0`): `−ρ_a;zi · q_V` (11.3, dichtheid bij
///   binnentemperatuur).
///
/// De mechanische toe-/afvoer is drukonafhankelijk en wordt onveranderd
/// opgeteld.
///
/// # Parameters
///
/// - `p_z_ref`: de te toetsen interne referentiedruk, in Pa.
/// - `openings`: de forfaitaire opening-set (uit [`build_openings`]).
/// - `mechanical`: de mechanische massastromen.
/// - `month`, `theta_e_c`: voor de externe druk (11.1).
/// - `rho_e`, `rho_i`: luchtdichtheid bij buiten- resp. binnentemperatuur,
///   in kg/m³ (formule (11.4)).
/// - `t_int_k`: binnentemperatuur `T_int;set;zi`, in K.
///
/// # Resultaat
///
/// `Σq_m` in kg/h. `0` betekent een gesloten massabalans.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.5), §11.2.1.6, PDF p. 441.
#[must_use]
#[allow(clippy::too_many_arguments)]
fn mass_balance_sum(
    p_z_ref: f64,
    openings: &[Opening],
    mechanical: MechanicalMassFlow,
    month: Month,
    theta_e_c: f64,
    rho_e: f64,
    rho_i: f64,
    t_int_k: f64,
) -> f64 {
    let mut sum = mechanical.sum();
    for opening in openings {
        let delta_p = pressure_difference(opening, month, theta_e_c, p_z_ref, t_int_k);
        // Formule (11.19): q_V = C · |Δp|^n.
        let q_v = opening.conductance_c * delta_p.abs().powf(opening.flow_exponent_n);
        if delta_p > 0.0 {
            // Ingaande stroom — formule (11.2), dichtheid bij T_e.
            sum += rho_e * q_v;
        } else {
            // Uitgaande stroom — formule (11.3), dichtheid bij T_i.
            sum -= rho_i * q_v;
        }
    }
    sum
}

// ===========================================================================
// Formule (11.14) — nauwkeurigheidscriterium
// ===========================================================================

/// Vereiste nauwkeurigheid `x` van de massabalans, in kg/h, volgens NTA 8800
/// formule (11.14).
///
/// # Norm-afleiding — formule (11.14)
///
/// ```text
/// q_V;nauwkeurigheid = q_V;ODA;req + q_V;comb;in − q_V;comb;out
///                    + q_V;argI;in + q_V;argII;in + q_V1;lea;ref
///
/// als q_V;nauwkeurigheid ≤ 1 000 m³/h:  x = 0,9 kg/h
/// als q_V;nauwkeurigheid > 1 000 m³/h:  x = ⌊q_V;nauwkeurigheid / 1 000⌋ · 0,9 kg/h
/// ```
///
/// De drempel schaalt mee met de totale verwerkte luchtvolumestroom: grotere
/// gebouwen mogen een evenredig groter absoluut massabalans-residu hebben.
/// Spui (`q_V;argI;in`), ventilatieve koeling (`q_V;argII;in`) en
/// verbrandingslucht (`q_V;comb`) zijn C2-buiten-scope (= 0), dus
/// `q_V;nauwkeurigheid = q_V;ODA;req + q_V1;lea;ref`.
///
/// # Parameters
///
/// - `q_v_oda_req`: benodigde luchtvolumestroom buitenlucht `q_V;ODA;req`,
///   in m³/h.
/// - `q_v1_lea_ref`: infiltratie-referentiedebiet `q_V1;lea;ref`, in m³/h.
///
/// # Resultaat
///
/// `x` in kg/h.
///
/// Referentie: NTA 8800:2025+C1:2026 formule (11.14), §11.2.1.6, PDF p. 444.
#[must_use]
pub fn accuracy_threshold(q_v_oda_req: f64, q_v1_lea_ref: f64) -> f64 {
    /// Norm-basisnauwkeurigheid uit formule (11.14), in kg/h.
    const BASE_ACCURACY_KG_PER_H: f64 = 0.9;
    /// Norm-schaaldrempel uit formule (11.14), in m³/h.
    const SCALING_THRESHOLD_M3_PER_H: f64 = 1000.0;

    // q_V;nauwkeurigheid — C2: alleen ODA-req + infiltratie-referentie.
    let q_v_accuracy = q_v_oda_req + q_v1_lea_ref;
    if q_v_accuracy <= SCALING_THRESHOLD_M3_PER_H {
        BASE_ACCURACY_KG_PER_H
    } else {
        // ⌊q_V;nauwkeurigheid / 1 000⌋ · 0,9 — naar beneden afgerond.
        (q_v_accuracy / SCALING_THRESHOLD_M3_PER_H).floor() * BASE_ACCURACY_KG_PER_H
    }
}

// ===========================================================================
// §11.2.1.6 routine — iteratieve p_z;ref-oplossing
// ===========================================================================

/// `sign(x)` zoals NTA 8800 §11.2.1.6 stap 7 die definieert: `−1` als `x < 0`,
/// `+1` als `x ≥ 0`.
#[must_use]
fn norm_sign(x: f64) -> f64 {
    if x < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// Of twee getallen hetzelfde teken hebben volgens de `sign`-definitie van
/// NTA 8800 §11.2.1.6 stap 7 (`0` telt als positief).
///
/// Wordt apart van [`norm_sign`] gehouden omdat de routine `sign` op twee
/// manieren gebruikt: als **factor** in de formules (11.16)/(11.17) (vandaar
/// dat [`norm_sign`] een `f64` teruggeeft) én als **tekenvergelijking** in de
/// stappen 6/7/11 — een directe `==` op de `f64`-uitkomsten zou een
/// float-vergelijking zijn.
#[must_use]
fn same_norm_sign(a: f64, b: f64) -> bool {
    (a < 0.0) == (b < 0.0)
}

/// Los de interne referentiedruk `p_z;ref` (Pa) op met de iteratieve routine
/// uit NTA 8800 §11.2.1.6 (stappen 1-12).
///
/// # De norm-routine (PDF p. 444-446)
///
/// 1. **Initiële schatting (11.15):** `p_z;ref = p_z;path;gem + ρ_a;z · H_path;lea · g`,
///    met `p_z;path;gem` het gemiddelde van de minimale en maximale
///    externe druk `p_e;path;i` over alle openingen, en `H_path;lea` de hoogte
///    van de loef-lek-opening.
/// 2. Bereken `Σq_m` (formule (11.5)) met deze `p_z;ref`. Is `|Σq_m| ≤ x` →
///    klaar (stap 12).
/// 3. `p_z;ref;a := p_z;ref`, `q_m;som;a := Σq_m`.
/// 4. `p_z;ref;b := p_z;ref;a + 2 Pa`.
/// 5. Bereken `q_m;som;b` met `p_z;ref;b`. Is `|q_m;som;b| ≤ x` → klaar.
/// 6. Hebben `q_m;som;a` en `q_m;som;b` een verschillend teken → ga naar stap 8.
/// 7. **Bracket-zoektocht (11.16)/(11.17):** zelfde teken → bereken
///    `r = sign(p_z;ref;b − p_z;ref;a) · sign(q_m;som;b − q_m;som;a)`. Is
///    `|q_m;som;a| > |q_m;som;b|`, schuif dan het bracket op (a := b). Bereken
///    `p_z;ref;b = p_z;ref;a − 2 Pa · sign(q_m;som;a) · r` en ga terug naar
///    stap 5.
/// 8. **Bisectie (stappen 8-12, formule (11.18)):**
///    `p_z;ref;c = (p_z;ref;a + p_z;ref;b)/2`, bereken `q_m;som;c`. Is
///    `|q_m;som;c| ≤ x` → klaar. Anders vervang het bracket-eindpunt waarvan
///    het teken met `q_m;som;c` overeenkomt, en herhaal vanaf stap 8.
///
/// # Iteratie-cap
///
/// De routine eindigt op het nauwkeurigheidscriterium (11.14). Een harde cap
/// ([`MAX_SOLVER_ITERATIONS`]) voorkomt een oneindige lus bij numeriek
/// pathologische invoer; bij overschrijding →
/// [`VentilationError::PressureSolverDidNotConverge`].
///
/// # Parameters
///
/// - `openings`: de forfaitaire opening-set (uit [`build_openings`]).
/// - `mechanical`: de mechanische massastromen.
/// - `month`, `theta_e_c`: voor de externe druk (11.1).
/// - `rho_e`, `rho_i`: luchtdichtheid bij buiten- resp. binnentemperatuur.
/// - `t_int_k`: binnentemperatuur `T_int;set;zi`, in K.
/// - `accuracy_x`: de vereiste nauwkeurigheid `x` (uit [`accuracy_threshold`]).
///
/// # Resultaat
///
/// `p_z;ref` in Pa.
///
/// # Errors
///
/// [`VentilationError::PressureSolverDidNotConverge`] als de routine de cap
/// bereikt zonder `|Σq_m| ≤ x`.
///
/// Referentie: NTA 8800:2025+C1:2026 §11.2.1.6 routine, PDF p. 444-446.
#[allow(clippy::too_many_arguments)]
fn solve_p_z_ref(
    openings: &[Opening],
    mechanical: MechanicalMassFlow,
    month: Month,
    theta_e_c: f64,
    rho_e: f64,
    rho_i: f64,
    t_int_k: f64,
    accuracy_x: f64,
) -> Result<f64, VentilationError> {
    // Sluitende massabalans-evaluatie voor een gegeven p_z;ref.
    let eval = |p_z_ref: f64| {
        mass_balance_sum(
            p_z_ref, openings, mechanical, month, theta_e_c, rho_e, rho_i, t_int_k,
        )
    };

    // --- Stap 1 — initiële schatting (formule (11.15)) -------------------
    let p_z_ref_init = initial_estimate(openings, month, theta_e_c, rho_i);

    // --- Stap 2 — eerste massabalans-evaluatie ---------------------------
    let mut iterations: u32 = 1;
    let mut p_z_ref_a = p_z_ref_init;
    let mut q_m_som_a = eval(p_z_ref_a);
    if q_m_som_a.abs() <= accuracy_x {
        return Ok(p_z_ref_a); // stap 12
    }

    // --- Stap 4 — tweede bracket-punt ------------------------------------
    let mut p_z_ref_b = p_z_ref_a + BRACKET_STEP_PA;
    let mut q_m_som_b = eval(p_z_ref_b);
    iterations += 1;
    if q_m_som_b.abs() <= accuracy_x {
        return Ok(p_z_ref_b); // stap 5 → stap 12
    }

    // --- Stappen 5-7 — bracket-zoektocht tot een tekenwissel -------------
    // Herhaal tot q_m;som;a en q_m;som;b een verschillend teken hebben.
    while same_norm_sign(q_m_som_a, q_m_som_b) {
        if iterations >= MAX_SOLVER_ITERATIONS {
            return Err(VentilationError::PressureSolverDidNotConverge {
                iterations,
                residual: q_m_som_b.abs().min(q_m_som_a.abs()),
            });
        }
        // Stap 7 — formule (11.16): r = sign(Δp_z;ref) · sign(Δq_m;som).
        let r = norm_sign(p_z_ref_b - p_z_ref_a) * norm_sign(q_m_som_b - q_m_som_a);
        // Als |q_m;som;a| > |q_m;som;b|, schuif het bracket op (a := b).
        if q_m_som_a.abs() > q_m_som_b.abs() {
            p_z_ref_a = p_z_ref_b;
            q_m_som_a = q_m_som_b;
        }
        // Stap 7 — formule (11.17): p_z;ref;b = p_z;ref;a − 2 Pa · sign(q_m;som;a) · r.
        p_z_ref_b = p_z_ref_a - BRACKET_STEP_PA * norm_sign(q_m_som_a) * r;
        // Stap 5 — herbereken q_m;som;b.
        q_m_som_b = eval(p_z_ref_b);
        iterations += 1;
        if q_m_som_b.abs() <= accuracy_x {
            return Ok(p_z_ref_b); // stap 5 → stap 12
        }
    }

    // --- Stappen 8-12 — bisectie -----------------------------------------
    loop {
        if iterations >= MAX_SOLVER_ITERATIONS {
            return Err(VentilationError::PressureSolverDidNotConverge {
                iterations,
                residual: q_m_som_a.abs().min(q_m_som_b.abs()),
            });
        }
        // Stap 8 — formule (11.18): p_z;ref;c = (p_z;ref;a + p_z;ref;b)/2.
        // `f64::midpoint` is hier wiskundig identiek aan `(a + b)/2` maar
        // overflow-veilig — de norm-formule blijft (11.18).
        let p_z_ref_c = f64::midpoint(p_z_ref_a, p_z_ref_b);
        // Stap 9 — bereken q_m;som;c.
        let q_m_som_c = eval(p_z_ref_c);
        iterations += 1;
        // Stap 10 — nauwkeurigheidstoets.
        if q_m_som_c.abs() <= accuracy_x {
            return Ok(p_z_ref_c); // stap 12
        }
        // Stap 11 — vervang het bracket-eindpunt met gelijk teken aan c.
        if same_norm_sign(q_m_som_c, q_m_som_a) {
            p_z_ref_a = p_z_ref_c;
            q_m_som_a = q_m_som_c;
        } else {
            p_z_ref_b = p_z_ref_c;
            q_m_som_b = q_m_som_c;
        }
    }
}

/// Stap 1 van de §11.2.1.6 routine — de initiële schatting voor `p_z;ref`
/// (formule (11.15)).
///
/// ```text
/// p_z;ref = p_z;path;gem + ρ_a;z · H_path;lea · g                     (11.15)
/// ```
///
/// waarin `p_z;path;gem` het gemiddelde is van de minimale en maximale
/// externe druk `p_e;path;i` over alle openingen, en `H_path;lea` de hoogte
/// van de loef-lek-opening is.
///
/// Is de opening-set leeg, dan is er geen `p_e`-spreiding; de schatting valt
/// terug op `0 Pa` (de solver levert dan sowieso direct een gesloten balans
/// als er ook geen mechanische stromen zijn).
fn initial_estimate(openings: &[Opening], month: Month, theta_e_c: f64, rho_i: f64) -> f64 {
    if openings.is_empty() {
        return 0.0;
    }
    // Min/max van p_e;path;i over alle openingen.
    let mut p_e_min = f64::INFINITY;
    let mut p_e_max = f64::NEG_INFINITY;
    for opening in openings {
        let p_e = external_pressure(opening, month, theta_e_c);
        p_e_min = p_e_min.min(p_e);
        p_e_max = p_e_max.max(p_e);
    }
    let p_z_path_gem = f64::midpoint(p_e_min, p_e_max);
    // H_path;lea — hoogte van de loef-lek-opening (formule (11.15)).
    //
    // [`build_openings`] plaatst de lek-openingen (n_lea = 0,67) altijd vóór de
    // natuurlijke vent-openingen (n_vent = 0,5), dus de eerste lek-opening is
    // tevens de loef-lek-opening. Zoek die expliciet op via de stromingsexponent
    // i.p.v. blind `openings[0]` te nemen: bij een opening-set zonder forfaitaire
    // C_lea (onbekend bouwjaar → geen lek-openingen) zou `openings[0]` een
    // natuurlijke vent-opening zijn. De stack-correctie `ρ·H·g` blijft dan toch
    // op de juiste 0,5·H-hoogte van die opening — een gedocumenteerde benadering:
    // zonder lek-openingen bestaat er strikt genomen geen `H_path;lea`, en de
    // initiële schatting (11.15) hoeft alleen de bracket-zoektocht te starten,
    // niet exact te zijn. Bij een volledig lege set valt de functie hierboven
    // al terug op 0 Pa.
    let h_path_lea = openings
        .iter()
        .find(|o| (o.flow_exponent_n - FLOW_EXPONENT_LEAKAGE).abs() < f64::EPSILON)
        .map_or(openings[0].height_m, |lea| lea.height_m);
    // Formule (11.15).
    p_z_path_gem + rho_i * h_path_lea * GRAVITY_M_PER_S2
}

// ===========================================================================
// §11.2.1.7 — effectieve luchtvolumestromen
// ===========================================================================

/// Resultaat van de zone-massabalans: de opgeloste `p_z;ref` plus de
/// effectieve in- en uitgaande luchtvolumestromen per stroomtype.
///
/// Alle debieten in **m³/h** conform NTA 8800 §11.2 (de norm rekent in m³/h).
/// De effectieve infiltratie volgt formules (11.20)/(11.21) §11.2.1.7;
/// de mechanische debieten zijn de drukonafhankelijke `q_V;SUP;eff` /
/// `q_V;ETA;eff` uit §11.2.2.2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoneAirflowSolution {
    /// De opgeloste interne referentiedruk `p_z;ref`, in Pa.
    pub p_z_ref: f64,

    /// Effectieve **ingaande** infiltratie `Σq_V;eff;lea;in`, in m³/h —
    /// de som over alle lek-openingen met `Δp > 0` (formule (11.20)).
    pub leakage_in: f64,

    /// Effectieve **uitgaande** infiltratie `Σq_V;eff;lea;out`, in m³/h —
    /// de som over alle lek-openingen met `Δp < 0` (formule (11.21)).
    pub leakage_out: f64,

    /// Effectieve **ingaande** natuurlijke ventilatie `Σq_V;eff;vent;in`,
    /// in m³/h — de som over alle natuurlijke vent-openingen met `Δp > 0`.
    pub natural_vent_in: f64,

    /// Effectieve **uitgaande** natuurlijke ventilatie `Σq_V;eff;vent;out`,
    /// in m³/h — de som over alle natuurlijke vent-openingen met `Δp < 0`.
    pub natural_vent_out: f64,

    /// Effectieve mechanische toevoer `q_V;SUP;eff`, in m³/h
    /// (drukonafhankelijk, §11.2.2.2).
    pub mechanical_supply: f64,

    /// Effectieve mechanische afvoer `q_V;ETA;eff`, in m³/h
    /// (drukonafhankelijk, §11.2.2.2).
    pub mechanical_exhaust: f64,
}

impl ZoneAirflowSolution {
    /// Netto **ingaande** luchtvolumestroom over de gebouwschil, in m³/h —
    /// de som van alle stromen die de zone binnenkomen.
    ///
    /// Dit is de fysisch relevante "verse-lucht"-aanvoer: mechanische toevoer
    /// + ingaande natuurlijke ventilatie + ingaande infiltratie.
    #[must_use]
    pub fn total_inflow(&self) -> f64 {
        self.mechanical_supply + self.natural_vent_in + self.leakage_in
    }

    /// Netto **uitgaande** luchtvolumestroom over de gebouwschil, in m³/h.
    #[must_use]
    pub fn total_outflow(&self) -> f64 {
        self.mechanical_exhaust + self.natural_vent_out + self.leakage_out
    }
}

/// Los de volledige zone-luchtstroom op: bepaal `p_z;ref` (§11.2.1.6) en de
/// effectieve in-/uitgaande debieten per stroomtype (§11.2.1.7).
///
/// # Norm-keten
///
/// 1. Bouw de forfaitaire opening-set (tabel 11.1 + §11.2.2.2,
///    [`build_openings`]).
/// 2. Bepaal de mechanische massastromen (§11.2.2.2).
/// 3. Bepaal de nauwkeurigheid `x` (formule (11.14)).
/// 4. Los `p_z;ref` op met de iteratieve routine (§11.2.1.6,
///    [`solve_p_z_ref`]).
/// 5. Bepaal per opening de effectieve luchtvolumestroom `q_V = C · |Δp|^n`
///    (formule (11.19)) en tel die op bij `…_in` (`Δp > 0`, formule (11.20))
///    of `…_out` (`Δp < 0`, formule (11.21)).
///
/// De `wtw`-parameter is op dit moment **niet** in de drukoplossing verwerkt:
/// de massabalans gebruikt de luchtdichtheid bij de buiten- en
/// binnentemperatuur (`theta_e`/`theta_i`), niet de WTW-toevoertemperatuur.
/// De WTW beïnvloedt de tóevoertemperatuur en daarmee de dichtheid van
/// `q_V;SUP;eff` — dat verfijnen is C2.3-scope (wiring + temperatuurketen).
/// De parameter staat in de signatuur zodat C2.3 hem zonder API-breuk kan
/// gaan gebruiken.
///
/// # Parameters
///
/// - `system`: het ventilatiesysteem.
/// - `flow`: de luchtstromen (mechanische debieten + `q_V;ODA;req`-proxy).
/// - `wtw`: de WTW-specificatie — nu nog ongebruikt (zie hierboven).
/// - `ctx`: de gebouw-drukcontext (`H`, `A_g`, bouwjaar, lek-type).
/// - `theta_e_c`: maandgemiddelde buitentemperatuur `ϑ_e;avg;mi`, in °C.
/// - `theta_i_c`: binnentemperatuur-setpoint `ϑ_int;set;stc`, in °C.
/// - `month`: de maand `mi` — bepaalt `u_site;mi`.
///
/// # Resultaat
///
/// Een [`ZoneAirflowSolution`] met `p_z;ref` + de effectieve debieten.
///
/// # Errors
///
/// [`VentilationError::PressureSolverDidNotConverge`] als de iteratieve
/// routine niet binnen de cap convergeert.
///
/// Referentie: NTA 8800:2025+C1:2026 §11.2.1.5/§11.2.1.6/§11.2.1.7,
/// PDF p. 440-447.
// theta_e_c/theta_i_c en t_e_k/t_i_k spiegelen bewust de norm-symbolen
// ϑ_e/ϑ_i resp. T_e/T_i — verschillende namen zouden de norm-traceability
// breken, dus de `similar_names`-heuristiek is hier een false-positief.
#[allow(clippy::too_many_arguments, clippy::similar_names)]
pub fn solve_zone_airflow(
    system: VentilationSystem,
    flow: &AirFlow,
    wtw: Option<&WtwSpecification>,
    ctx: &BuildingPressureContext,
    theta_e_c: f64,
    theta_i_c: f64,
    month: Month,
) -> Result<ZoneAirflowSolution, VentilationError> {
    // wtw is C2.3-scope (zie doc-comment) — nu expliciet ongebruikt.
    let _ = wtw;

    // --- Stap 1 — forfaitaire opening-set + lek-conductantie -------------
    let openings = build_openings(system, flow, ctx);

    // Luchtdichtheid bij buiten- en binnentemperatuur (formule (11.4)).
    let t_e_k = celsius_to_kelvin(theta_e_c);
    let t_i_k = celsius_to_kelvin(theta_i_c);
    let rho_e = air_density(t_e_k);
    let rho_i = air_density(t_i_k);

    // --- Stap 3 — mechanische massastromen (§11.2.2.2) -------------------
    let mechanical = mechanical_mass_flow(system, flow, rho_e, rho_i);

    // --- Nauwkeurigheid x (formule (11.14)) ------------------------------
    // q_V;ODA;req-proxy: de relevante mechanische ventilatiestroom.
    //
    // Systeem A heeft per definitie GEEN mechanisch debiet (§11.2.2.2.1:
    // q_V;SUP;eff = q_V;ETA;eff = 0). De `mechanical_supply`/`mechanical_exhaust`-
    // velden van [`AirFlow`] dienen voor systeem A uitsluitend als forfaitaire
    // q_V;ODA;req-proxy (zie [`build_openings`] §11.2.2.2.1): consumers die de
    // natuurlijke ventilatie-conductantie willen voeden, zetten daar de
    // ontwerp-ventilatiestroom in. De `max()` pakt de hoogste van de twee zodat
    // de nauwkeurigheidsterm (11.14) niet onderschat wordt.
    let q_v_oda_req = match system {
        VentilationSystem::A => flow.mechanical_supply.max(flow.mechanical_exhaust),
        VentilationSystem::B => flow.mechanical_supply,
        VentilationSystem::C => flow.mechanical_exhaust,
        VentilationSystem::D { .. } | VentilationSystem::E => {
            flow.mechanical_supply.max(flow.mechanical_exhaust)
        }
    };
    // q_V1;lea;ref voor de nauwkeurigheidsterm (formule (11.14)) — zelfde
    // effectieve q_v10;lea;ref-bron als C_lea (meting > forfait, §11.2.5).
    let q_v1_lea = ctx
        .effective_q_v10()
        .map_or(0.0, |q_v10| q_v1_lea_ref(q_v10, ctx.gross_floor_area_m2));
    let accuracy_x = accuracy_threshold(q_v_oda_req, q_v1_lea);

    // --- Stap 4 — los p_z;ref op (§11.2.1.6 routine) ---------------------
    let p_z_ref = solve_p_z_ref(
        &openings, mechanical, month, theta_e_c, rho_e, rho_i, t_i_k, accuracy_x,
    )?;

    // --- Stap 5/6 — effectieve luchtvolumestromen (§11.2.1.7) ------------
    let mut solution = ZoneAirflowSolution {
        p_z_ref,
        leakage_in: 0.0,
        leakage_out: 0.0,
        natural_vent_in: 0.0,
        natural_vent_out: 0.0,
        mechanical_supply: match system {
            VentilationSystem::A | VentilationSystem::C => 0.0,
            VentilationSystem::B | VentilationSystem::D { .. } | VentilationSystem::E => {
                flow.mechanical_supply
            }
        },
        mechanical_exhaust: match system {
            VentilationSystem::A | VentilationSystem::B => 0.0,
            VentilationSystem::C | VentilationSystem::D { .. } | VentilationSystem::E => {
                flow.mechanical_exhaust
            }
        },
    };

    for opening in &openings {
        let delta_p = pressure_difference(opening, month, theta_e_c, p_z_ref, t_i_k);
        // Formule (11.19)/(11.20)/(11.21): q_V = C · |Δp|^n.
        let q_v = opening.conductance_c * delta_p.abs().powf(opening.flow_exponent_n);
        // Onderscheid lek vs. natuurlijke vent via de stromingsexponent:
        // n_lea = 0,67 (lek), n_vent = 0,5 (ventilatievoorziening).
        let is_leakage =
            (opening.flow_exponent_n - FLOW_EXPONENT_LEAKAGE).abs() < f64::EPSILON;
        if delta_p > 0.0 {
            // Ingaande stroom (formule (11.20)).
            if is_leakage {
                solution.leakage_in += q_v;
            } else {
                solution.natural_vent_in += q_v;
            }
        } else {
            // Uitgaande stroom (formule (11.21)).
            if is_leakage {
                solution.leakage_out += q_v;
            } else {
                solution.natural_vent_out += q_v;
            }
        }
    }

    Ok(solution)
}

// ===========================================================================
// Tests — assert tegen norm-formules + handberekende referentiegevallen
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BuildingLeakageType;
    use approx::assert_relative_eq;

    /// Standaard C2-drukcontext: grondgebonden tussenwoning met kap, bouwjaar
    /// 2015, `H = 8 m`, `A_g = 120 m²`.
    fn sample_ctx() -> BuildingPressureContext {
        BuildingPressureContext::new(
            8.0,
            Some(2015),
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        )
    }

    // --- Formule (11.4) — luchtdichtheid --------------------------------

    #[test]
    fn air_density_at_reference_temp_is_rho_ref() {
        // Bij T = T_ref = 293 K geldt ρ_T = ρ_a;ref.
        assert_relative_eq!(air_density(T_REF_K), RHO_A_REF_KG_PER_M3, epsilon = 1e-12);
    }

    #[test]
    fn air_density_colder_air_is_denser() {
        // Koudere lucht (lagere T) → hogere dichtheid.
        let rho_cold = air_density(celsius_to_kelvin(-10.0)); // 263 K
        let rho_warm = air_density(celsius_to_kelvin(20.0)); // 293 K
        assert!(rho_cold > rho_warm);
        // Numerieke verankering: ρ(263 K) = 1,205·293/263 ≈ 1,34244 kg/m³.
        assert_relative_eq!(rho_cold, 1.205 * 293.0 / 263.0, epsilon = 1e-12);
    }

    // --- Formule (11.1) — externe druk ----------------------------------

    #[test]
    fn external_pressure_matches_formula_11_1() {
        // Loef-opening: C_p = +0,25, H = 4 m, januari (u = 3,04 m/s),
        // ϑ_e = 5 °C.
        let opening = Opening::new(4.0, 0.25, 100.0, FLOW_EXPONENT_LEAKAGE);
        let p_e = external_pressure(&opening, Month::Januari, 5.0);
        // Handberekening formule (11.1):
        //   density_ratio = 293 / (5 + 273) = 293/278
        //   wind_term     = 0,5 · 0,25 · 3,04²
        //   stack_term    = −4 · 9,81
        //   p_e = 1,205 · density_ratio · (wind_term + stack_term)
        let density_ratio = 293.0 / 278.0;
        let wind_term = 0.5 * 0.25 * 3.04 * 3.04;
        let stack_term = -4.0 * 9.81;
        let expected = 1.205 * density_ratio * (wind_term + stack_term);
        assert_relative_eq!(p_e, expected, epsilon = 1e-9);
    }

    #[test]
    fn external_pressure_windward_higher_than_leeward() {
        // Loef (C_p = +0,25) heeft een hogere winddrukbijdrage dan lij
        // (C_p = −0,50) bij gelijke hoogte.
        let loef = Opening::new(4.0, 0.25, 50.0, FLOW_EXPONENT_LEAKAGE);
        let lij = Opening::new(4.0, -0.50, 50.0, FLOW_EXPONENT_LEAKAGE);
        let p_e_loef = external_pressure(&loef, Month::Januari, 5.0);
        let p_e_lij = external_pressure(&lij, Month::Januari, 5.0);
        assert!(p_e_loef > p_e_lij);
    }

    // --- Formules (11.11)/(11.12) — drukverschil ------------------------

    #[test]
    fn pressure_difference_sign_flips_around_p_z_ref() {
        // Bij stijgende p_z;ref daalt Δp_path;i monotoon (Δp = p_e − p_z;ref + …).
        let opening = Opening::new(4.0, 0.25, 50.0, FLOW_EXPONENT_LEAKAGE);
        let t_i = celsius_to_kelvin(20.0);
        let dp_low = pressure_difference(&opening, Month::Januari, 5.0, -50.0, t_i);
        let dp_high = pressure_difference(&opening, Month::Januari, 5.0, 50.0, t_i);
        assert!(dp_low > dp_high);
        // Er moet ergens tussen −50 en +50 Pa een tekenwissel zitten.
        assert!(dp_low > 0.0 && dp_high < 0.0);
    }

    // --- Formule (11.14) — nauwkeurigheidscriterium ---------------------

    #[test]
    fn accuracy_threshold_below_1000_is_base() {
        // q_V;nauwkeurigheid ≤ 1 000 m³/h → x = 0,9 kg/h.
        assert_relative_eq!(accuracy_threshold(150.0, 64.0), 0.9, epsilon = 1e-12);
        assert_relative_eq!(accuracy_threshold(0.0, 0.0), 0.9, epsilon = 1e-12);
        // Exact op de drempel: 1 000 ≤ 1 000 → nog steeds 0,9.
        assert_relative_eq!(accuracy_threshold(1000.0, 0.0), 0.9, epsilon = 1e-12);
    }

    #[test]
    fn accuracy_threshold_above_1000_scales_floored() {
        // q_V;nauwkeurigheid = 2 500 → ⌊2 500/1 000⌋ · 0,9 = 2 · 0,9 = 1,8.
        assert_relative_eq!(accuracy_threshold(2000.0, 500.0), 1.8, epsilon = 1e-12);
        // q_V;nauwkeurigheid = 3 999 → ⌊3,999⌋ · 0,9 = 3 · 0,9 = 2,7.
        assert_relative_eq!(accuracy_threshold(3999.0, 0.0), 2.7, epsilon = 1e-12);
    }

    // --- build_openings — forfaitaire opening-set (tabel 11.1) ----------

    #[test]
    fn build_openings_balanced_system_has_only_leakage() {
        // Systeem D (BALANCED_OP): geen natuurlijke vent-openingen
        // (§11.2.2.2.4) → alleen de 3 lek-openingen (loef/lij/dak).
        let flow = AirFlow::new(150.0, 150.0, 0.0);
        let openings = build_openings(VentilationSystem::D { with_wtw: true }, &flow, &sample_ctx());
        assert_eq!(openings.len(), 3, "verwacht loef + lij + dak lek-openingen");
        for o in &openings {
            assert_relative_eq!(o.flow_exponent_n, FLOW_EXPONENT_LEAKAGE, epsilon = 1e-12);
        }
        // Loef + lij op 0,5·H = 4 m, dak op H = 8 m.
        assert_relative_eq!(openings[0].height_m, 4.0, epsilon = 1e-12);
        assert_relative_eq!(openings[1].height_m, 4.0, epsilon = 1e-12);
        assert_relative_eq!(openings[2].height_m, 8.0, epsilon = 1e-12);
        // Loef +0,25 / lij −0,50 / dak −0,60 (tabel 11.3, klasse Laag).
        assert_relative_eq!(openings[0].wind_pressure_coeff, 0.25, epsilon = 1e-12);
        assert_relative_eq!(openings[1].wind_pressure_coeff, -0.50, epsilon = 1e-12);
        assert_relative_eq!(openings[2].wind_pressure_coeff, -0.60, epsilon = 1e-12);
    }

    #[test]
    fn build_openings_leakage_conductance_splits_per_table_11_1() {
        // C_lea = q_v1;lea;ref (1 Pa). Loef/lij elk 0,4·C_lea, dak 0,2·C_lea.
        let flow = AirFlow::new(0.0, 0.0, 0.0);
        let ctx = sample_ctx();
        let openings = build_openings(VentilationSystem::D { with_wtw: false }, &flow, &ctx);
        // C_lea: q_v10 = 0,7 (forfait 2015), q_v1;lea;ref = 0,7·0,1^0,67·120·3,6.
        let c_lea = 0.7 * 0.1_f64.powf(0.67) * 120.0 * 3.6;
        assert_relative_eq!(openings[0].conductance_c, 0.4 * c_lea, epsilon = 1e-9);
        assert_relative_eq!(openings[1].conductance_c, 0.4 * c_lea, epsilon = 1e-9);
        assert_relative_eq!(openings[2].conductance_c, 0.2 * c_lea, epsilon = 1e-9);
    }

    #[test]
    fn build_openings_supply_system_adds_natural_exhaust() {
        // Systeem B (SUPPLY_OP): natuurlijke afvoer-openingen erbij
        // (§11.2.2.2.2) → 3 lek + 2 vent (loef + lij).
        let flow = AirFlow::new(120.0, 0.0, 0.0);
        let openings = build_openings(VentilationSystem::B, &flow, &sample_ctx());
        assert_eq!(openings.len(), 5);
        // De 2 vent-openingen hebben n_vent = 0,5.
        let vent_count = openings
            .iter()
            .filter(|o| (o.flow_exponent_n - FLOW_EXPONENT_VENTILATION).abs() < 1e-12)
            .count();
        assert_eq!(vent_count, 2);
    }

    #[test]
    fn build_openings_unknown_build_year_has_no_leakage() {
        // Onbekend bouwjaar → geen forfaitaire C_lea → geen lek-openingen.
        let flow = AirFlow::new(0.0, 0.0, 0.0);
        let ctx = BuildingPressureContext::new(
            8.0,
            None,
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        let openings = build_openings(VentilationSystem::D { with_wtw: false }, &flow, &ctx);
        assert!(openings.is_empty());
    }

    // --- solve_p_z_ref — handberekend referentiegeval -------------------

    #[test]
    fn solve_p_z_ref_symmetric_loef_lij_pair_matches_closed_form() {
        // HANDBEREKEND REFERENTIEGEVAL.
        //
        // Een symmetrisch loef/lij-paar op gelijke hoogte (H = 4 m), met
        // ϑ_e = ϑ_i = 10 °C zodat ρ_a;e = ρ_a;zi (formule (11.4)) en de
        // hydrostatische termen identiek zijn. Geen mechanische stromen.
        //
        // Massabalans (11.5) reduceert dan tot ρ·(q_in − q_out) = 0, dus
        // q_in = q_out. Met twee openingen van gelijke conductantie C en
        // gelijke hoogte H volgt C·|Δp_loef|^n = C·|Δp_lij|^n, waarbij loef
        // ingaand en lij uitgaand is → Δp_loef = −Δp_lij.
        //
        // Δp_path;i = p_e;i − p_z;ref + stack(H) (formule (11.11)/(11.12)).
        // Δp_loef = −Δp_lij  ⇒
        //   (p_e;loef − p_z;ref + s) = −(p_e;lij − p_z;ref + s)
        //   ⇒ p_e;loef + p_e;lij + 2s − 2·p_z;ref = 0
        //   ⇒ p_z;ref = (p_e;loef + p_e;lij)/2 + s
        // met s = ρ_a;ref · H · g · (T_e;ref / T_i).
        //
        // Numeriek (januari, u = 3,04 m/s, ϑ = 10 °C, C = 20 m³/(h·Pa⁰·⁶⁷)):
        //   p_e;loef = 1,205·(293/283)·(0,5·0,25·3,04² − 4·9,81)
        //   p_e;lij  = 1,205·(293/283)·(0,5·(−0,50)·3,04² − 4·9,81)
        //   s        = 1,205·4·9,81·(293/283)
        //   p_z;ref  = (p_e;loef + p_e;lij)/2 + s ≈ −0,72060 Pa
        //   (zie assert hieronder).
        let n = FLOW_EXPONENT_LEAKAGE;
        let loef = Opening::new(4.0, 0.25, 20.0, n);
        let lij = Opening::new(4.0, -0.50, 20.0, n);
        let openings = [loef, lij];
        let theta = 10.0_f64;
        let t_k = celsius_to_kelvin(theta);
        let rho = air_density(t_k);
        let mechanical = MechanicalMassFlow {
            supply_in: 0.0,
            exhaust_out: 0.0,
        };
        let solved = solve_p_z_ref(
            &openings,
            mechanical,
            Month::Januari,
            theta,
            rho,
            rho,
            t_k,
            // Strakke nauwkeurigheid om de bisectie diep te laten lopen.
            1e-9,
        )
        .expect("symmetrisch geval moet convergeren");

        // Gesloten-vorm referentie: p_z;ref = (p_e;loef + p_e;lij)/2 + s.
        let p_e_loef = external_pressure(&loef, Month::Januari, theta);
        let p_e_lij = external_pressure(&lij, Month::Januari, theta);
        let s = internal_stack_term(4.0, t_k);
        let closed_form = f64::midpoint(p_e_loef, p_e_lij) + s;
        assert_relative_eq!(solved, closed_form, epsilon = 1e-6);

        // Numerieke verankering op de afgeleide waarde.
        assert_relative_eq!(solved, -0.720_60, epsilon = 1e-3);

        // Verifieer dat de massabalans bij deze p_z;ref dicht is.
        let residual = mass_balance_sum(
            solved, &openings, mechanical, Month::Januari, theta, rho, rho, t_k,
        );
        assert!(residual.abs() < 1e-6, "residu {residual} te groot");
    }

    #[test]
    fn solve_p_z_ref_bracket_search_finds_sign_change() {
        // De bracket-zoektocht (stappen 5-7) moet bij een asymmetrische
        // opening-set tóch twee p_z;ref-waarden met tegengesteld
        // massabalans-teken vinden, ook al ligt het nulpunt ver van de
        // initiële schatting (11.15).
        let flow = AirFlow::new(0.0, 0.0, 0.0);
        let ctx = sample_ctx();
        let openings = build_openings(VentilationSystem::D { with_wtw: false }, &flow, &ctx);
        let theta_e = -5.0_f64; // strenge vorst → grote stack-asymmetrie
        let theta_i = 21.0_f64;
        let t_e = celsius_to_kelvin(theta_e);
        let t_i = celsius_to_kelvin(theta_i);
        let rho_e = air_density(t_e);
        let rho_i = air_density(t_i);
        let mechanical = MechanicalMassFlow {
            supply_in: 0.0,
            exhaust_out: 0.0,
        };
        let solved = solve_p_z_ref(
            &openings, mechanical, Month::Februari, theta_e, rho_e, rho_i, t_i, 0.9,
        )
        .expect("bracket-zoektocht moet een tekenwissel vinden");
        // De gevonden p_z;ref moet de massabalans sluiten binnen x = 0,9.
        let residual = mass_balance_sum(
            solved,
            &openings,
            mechanical,
            Month::Februari,
            theta_e,
            rho_e,
            rho_i,
            t_i,
        );
        assert!(residual.abs() <= 0.9, "residu {residual} > x = 0,9 kg/h");
        // En de massabalans wisselt van teken rond de oplossing — bewijs dat
        // er een echte bracket is.
        let lo = mass_balance_sum(
            solved - 5.0,
            &openings,
            mechanical,
            Month::Februari,
            theta_e,
            rho_e,
            rho_i,
            t_i,
        );
        let hi = mass_balance_sum(
            solved + 5.0,
            &openings,
            mechanical,
            Month::Februari,
            theta_e,
            rho_e,
            rho_i,
            t_i,
        );
        assert!(lo * hi < 0.0, "geen tekenwissel rond de oplossing");
    }

    #[test]
    fn solve_p_z_ref_does_not_converge_without_conductance() {
        // Pathologisch geval: geen openingen + een onbalans in de mechanische
        // stromen → de massabalans kan nooit sluiten, geen tekenwissel
        // bereikbaar. De solver moet netjes met de error-variant stoppen,
        // niet oneindig lussen.
        let openings: [Opening; 0] = [];
        // supply_in ≠ −exhaust_out → constante, niet-nul Σq_m.
        let mechanical = MechanicalMassFlow {
            supply_in: 120.0,
            exhaust_out: -100.0,
        };
        let err = solve_p_z_ref(
            &openings,
            mechanical,
            Month::Januari,
            5.0,
            air_density(celsius_to_kelvin(5.0)),
            air_density(celsius_to_kelvin(20.0)),
            celsius_to_kelvin(20.0),
            0.9,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            VentilationError::PressureSolverDidNotConverge { .. }
        ));
    }

    // --- solve_zone_airflow — top-level ---------------------------------

    #[test]
    fn solve_zone_airflow_balanced_imbalance_gives_net_infiltration() {
        // ONBALANS-GEVAL (a): gebalanceerd systeem D met supply ≠ exhaust.
        // De solver moet een netto infiltratie > 0 leveren: een onbalans in
        // de mechanische stromen wordt over de gebouwschil gecompenseerd.
        let flow = AirFlow::new(180.0, 120.0, 0.0); // supply > exhaust
        let solution = solve_zone_airflow(
            VentilationSystem::D { with_wtw: true },
            &flow,
            None,
            &sample_ctx(),
            5.0,
            20.0,
            Month::Januari,
        )
        .expect("balansgeval moet convergeren");
        // Netto luchtbalans over de schil moet de mechanische onbalans dekken.
        // supply > exhaust → de zone staat onder overdruk → netto infiltratie
        // verlaat de zone (leakage_out > leakage_in).
        assert!(
            solution.leakage_out > solution.leakage_in,
            "overdruk → netto uitstroom via lek verwacht: in={} out={}",
            solution.leakage_in,
            solution.leakage_out
        );
        // Er ís infiltratie (de schil is niet dicht).
        assert!(solution.leakage_in + solution.leakage_out > 0.0);
    }

    #[test]
    fn solve_zone_airflow_balanced_symmetric_is_pure_stack_wind() {
        // SYMMETRIE-GEVAL (b): gebalanceerd systeem D met supply == exhaust.
        // Geen mechanische onbalans → de lek-stromen worden puur door
        // stack/wind gedreven → ingaande en uitgaande infiltratie zijn
        // ongeveer in evenwicht.
        let flow = AirFlow::new(150.0, 150.0, 0.0);
        let solution = solve_zone_airflow(
            VentilationSystem::D { with_wtw: true },
            &flow,
            None,
            &sample_ctx(),
            5.0,
            20.0,
            Month::Januari,
        )
        .expect("symmetrisch balansgeval moet convergeren");
        // Bij mechanische balans drijft alleen stack/wind de infiltratie:
        // de massabalans van de lek-openingen sluit onderling. Met
        // ρ_a;e ≠ ρ_a;zi is q_V;in iets kleiner dan q_V;out (de zwaardere
        // koude buitenlucht draagt meer massa per m³), maar de afwijking is
        // klein t.o.v. een onbalansgeval.
        let total_leak = solution.leakage_in + solution.leakage_out;
        assert!(total_leak > 0.0, "stack/wind moet infiltratie aandrijven");
        let imbalance = (solution.leakage_in - solution.leakage_out).abs() / total_leak;
        assert!(
            imbalance < 0.20,
            "symmetrisch geval: lek-in/uit moet bijna in evenwicht zijn, \
             relatieve onbalans = {imbalance}"
        );
    }

    #[test]
    fn solve_zone_airflow_converges_within_iteration_cap() {
        // CONVERGENTIE-GEVAL (c): voor alle 12 maanden + de 4 systeemtypes
        // moet de solver binnen de iteratie-cap convergeren — geen
        // PressureSolverDidNotConverge.
        let ctx = sample_ctx();
        let systems = [
            VentilationSystem::A,
            VentilationSystem::B,
            VentilationSystem::C,
            VentilationSystem::D { with_wtw: true },
        ];
        for system in systems {
            let flow = AirFlow::new(150.0, 130.0, 0.0);
            for month in Month::all() {
                let result =
                    solve_zone_airflow(system, &flow, None, &ctx, 4.0, 20.0, month);
                assert!(
                    result.is_ok(),
                    "solver moet convergeren voor {system:?} in {month:?}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn solve_zone_airflow_natural_system_has_natural_vent_flow() {
        // Systeem A (NATURAL_OP) met een q_V;ODA;req-proxy → er moeten
        // effectieve natuurlijke ventilatiestromen ontstaan.
        let flow = AirFlow::new(100.0, 100.0, 0.0);
        let solution = solve_zone_airflow(
            VentilationSystem::A,
            &flow,
            None,
            &sample_ctx(),
            5.0,
            20.0,
            Month::Januari,
        )
        .expect("systeem A moet convergeren");
        // Natuurlijke ventilatie loopt zowel in als uit (loef/lij-paren).
        assert!(solution.natural_vent_in + solution.natural_vent_out > 0.0);
        // Systeem A heeft geen mechanische debieten.
        assert_relative_eq!(solution.mechanical_supply, 0.0, epsilon = 1e-12);
        assert_relative_eq!(solution.mechanical_exhaust, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn solve_zone_airflow_p_z_ref_closes_mass_balance() {
        // De opgeloste p_z;ref moet de massabalans (11.5) sluiten binnen de
        // norm-nauwkeurigheid x — onafhankelijk verifieerd via
        // mass_balance_sum.
        let flow = AirFlow::new(160.0, 140.0, 0.0);
        let ctx = sample_ctx();
        let system = VentilationSystem::D { with_wtw: false };
        let solution =
            solve_zone_airflow(system, &flow, None, &ctx, 3.0, 20.0, Month::December)
                .expect("moet convergeren");
        // Reconstrueer de massabalans-componenten.
        let openings = build_openings(system, &flow, &ctx);
        let rho_e = air_density(celsius_to_kelvin(3.0));
        let rho_i = air_density(celsius_to_kelvin(20.0));
        let mechanical = mechanical_mass_flow(system, &flow, rho_e, rho_i);
        let residual = mass_balance_sum(
            solution.p_z_ref,
            &openings,
            mechanical,
            Month::December,
            3.0,
            rho_e,
            rho_i,
            celsius_to_kelvin(20.0),
        );
        // x voor dit geval (q_V;ODA;req ≈ 160, q_v1;lea;ref ≈ 65) ≤ 1 000 → x = 0,9.
        assert!(residual.abs() <= 0.9, "massabalans-residu {residual} > 0,9");
    }

    #[test]
    fn zone_airflow_solution_inflow_outflow_helpers() {
        // total_inflow / total_outflow tellen de juiste componenten op.
        let sol = ZoneAirflowSolution {
            p_z_ref: -1.0,
            leakage_in: 10.0,
            leakage_out: 5.0,
            natural_vent_in: 20.0,
            natural_vent_out: 15.0,
            mechanical_supply: 100.0,
            mechanical_exhaust: 90.0,
        };
        assert_relative_eq!(sol.total_inflow(), 130.0, epsilon = 1e-12);
        assert_relative_eq!(sol.total_outflow(), 110.0, epsilon = 1e-12);
    }
}
