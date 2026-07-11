//! TOjuli-oververhittingsindicator — NTA 8800:2025+C1:2026 §5.7.
//!
//! §5.7 ("Grenswaarde oververhitting", PDF p. 113-119) geeft een
//! *vereenvoudigde* methode om het risico op te hoge zomertemperaturen te
//! toetsen zonder een volledige uur-voor-uur GTO-berekening. De methode
//! bepaalt **per rekenzone en per oriëntatie** een indicator `TOjuli;or,zi`
//! (formule 5.40) op basis van de koudebehoefte van de maand juli, gedeeld
//! door de warmteoverdrachtcoëfficiënten van die oriëntatie maal de
//! maandlengte.
//!
//! ## Eenheid
//!
//! `TOjuli;or,zi` heeft **de eenheid K** (kelvin) — het is een geschatte
//! temperatuurstijging, niet dimensieloos (§5.7.2, definitie bij formule
//! 5.40, p. 115). Numeriek is `Q·1000` [Wh] gedeeld door `H·t` [Wh/K] → K.
//! De Bbl-grenswaarde 1,20 ([`crate::beng::TOJULI_LIMIT`]) is dus 1,20 K.
//!
//! ## Toepasselijkheid (§5.7.1 / §5.7.2, p. 113-115)
//!
//! - De berekening is **alleen** vereist voor rekenzones **zonder** actief
//!   koelsysteem van voldoende capaciteit. Is zo'n systeem aanwezig, dan mag
//!   voor alle oriëntaties `TOjuli;or,zi = 0` worden gehanteerd en wordt de
//!   zone geacht te voldoen ([`TojuliZoneResult::actively_cooled`]).
//! - De acht oriëntaties zijn N, NO, O, ZO, Z, ZW, W, NW (§5.7.2 stap A).
//!   Horizontale/overige vlakken worden naar rato over de oriëntaties
//!   verdeeld — die verdeling gebeurt **buiten** deze module (F2), die levert
//!   de reeds-verdeelde per-oriëntatie-termen aan.
//! - Oriëntaties met `AT;or,zi ≤ 3 m²` blijven buiten beschouwing
//!   ([`AT_MIN_M2`]; §5.7.2 stap A + OPMERKING 3, p. 119).
//! - De uitkomst wordt geklemd op minimaal 0 en **naar boven afgerond op een
//!   veelvoud van 0,01** (p. 119).
//!
//! ## Relatie met GTO (§5.7.1, p. 114 + Bbl art. 4.149b)
//!
//! De TOjuli-indicator is een *vereenvoudigde* toets. Wordt de TOjuli-eis
//! (lid 1: ≤ 1,20 per rekenzone en oriëntatie) niet gehaald, dan mag conform
//! de regelgeving alsnog met een **GTO-uurberekening** worden aangetoond dat
//! het aantal gewogen temperatuuroverschrijdingsuren onder de grens
//! ([`crate::beng::GTO_LIMIT_HOURS`], 450 h) blijft. Die uurmethode zelf
//! staat **niet** in NTA 8800 §5.7 maar in de Regeling Bouwbesluit bijlage
//! VII / Omgevingsregeling bijlage XVI, en is hier **bewust niet**
//! geïmplementeerd. Deze module signaleert enkel via
//! [`TojuliZoneResult::requires_gto_hourly_check`] dat die vervolgroute nodig
//! is.
//!
//! ## Bewust niet geïmplementeerd
//!
//! - De opbouw van de per-oriëntatie-termen zelf (stappen A/B en 1-5 van
//!   §5.7.2: opdelen van `QC;nd;juli`, `HC;D;juli`, `Hgr;an;juli`,
//!   `HC;ve;juli` en de zonwinst per oriëntatie, gewogen naar `AT;or,zi`).
//!   Dat is ketenwerk over demand/transmission/ventilation (F2); deze module
//!   consumeert de uitkomst als kale parameters.
//! - De booster-warmtepomp-verdeling (formules 5.41a-5.41c). `QC;HP;juli;or,zi`
//!   wordt als kale parameter aangeleverd; zonder booster-WP is die 0.
//! - De GTO-uurberekening (zie hierboven).

use crate::beng::{GTO_LIMIT_HOURS, TOJULI_LIMIT};
use nta8800_model::location::Orientation;
use serde::{Deserialize, Serialize};

/// Omrekenfactor kWh → Wh in formule (5.40): de koudebehoefte in kWh wordt
/// met 1000 vermenigvuldigd zodat de eenheid met `H·t` [Wh/K] tot K leidt.
/// NTA 8800 §5.7.2, formule (5.40).
const KWH_TO_WH: f64 = 1_000.0;

/// `AT;or,zi`-drempel [m²]. Oriëntaties met een geprojecteerd gevel-/dak-
/// oppervlak ≤ deze waarde blijven buiten beschouwing: voor zulke oriëntaties
/// wordt `TOjuli;or,zi` niet bepaald.
///
/// NTA 8800 §5.7.2 stap A (p. 116) + OPMERKING 3 (p. 119): "Normatief is
/// bepaald dat deze een maximale afmeting mogen hebben van 3 m² per oriëntatie
/// per rekenzone voor alle constructies met eenzelfde oriëntatie."
pub const AT_MIN_M2: f64 = 3.0;

/// Afrondingsstap voor de TOjuli-indicator: naar boven op een veelvoud van
/// 0,01. NTA 8800 §5.7.2, p. 119: "Rond de TOjuli-indicator naar boven af op
/// een veelvoud van 0,01."
const TOJULI_ROUND_STEP: f64 = 0.01;

/// Foutmarge voor de naar-boven-afronding, uitgedrukt in stappen van
/// [`TOJULI_ROUND_STEP`]. Vangt drijvende-komma-artefacten op: een waarde die
/// wiskundig exact op een veelvoud van 0,01 ligt maar door de deling net
/// erboven wordt opgeslagen, zou anders ten onrechte een stap omhoog gaan.
/// 1e-6 stap ≙ 1e-8 in K — vier grootteordes kleiner dan de afrondingsstap,
/// dus een echte overschrijding wordt nooit weg-afgerond.
const TOJULI_ROUND_EPS_STEPS: f64 = 1e-6;

/// Rond een (reeds op ≥ 0 geklemde) TOjuli-waarde naar boven af op een
/// veelvoud van [`TOJULI_ROUND_STEP`]. Zie §5.7.2, p. 119.
fn round_up_tojuli(value: f64) -> f64 {
    let steps = value / TOJULI_ROUND_STEP;
    (steps - TOJULI_ROUND_EPS_STEPS).ceil() * TOJULI_ROUND_STEP
}

/// Invoer voor de TOjuli-berekening van **één oriëntatie** binnen een
/// rekenzone.
///
/// Alle termen zijn de reeds per oriëntatie opgedeelde grootheden uit §5.7.2
/// (stappen A/B en 1-5). Deze module deelt niet zelf op; de keten (F2) levert
/// de opgedeelde waarden aan. Symbolen tussen haakjes zijn de norm-symbolen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TojuliOrientationInput {
    /// De oriëntatie waarop deze invoer betrekking heeft (N, NO, …, NW).
    /// [`Orientation::Horizontaal`] hoort hier niet: horizontale vlakken zijn
    /// in de keten al naar rato over de acht kompas-oriëntaties verdeeld.
    pub orientation: Orientation,

    /// `AT;or,zi` — som van de geprojecteerde oppervlakten van de uitwendige
    /// scheidingsconstructies voor deze oriëntatie [m²]. NTA 8800 formule
    /// (5.41). Bepaalt via [`AT_MIN_M2`] of de oriëntatie wordt beoordeeld.
    pub a_t_m2: f64,

    /// `QC;nd;juli;or,zi` — koudebehoefte voor de maand juli voor deze
    /// oriëntatie [kWh]. NTA 8800 §5.7.2 stap B (volgens 7.2.2).
    pub q_c_nd_juli_kwh: f64,

    /// `QC;HP;juli;or,zi` — door de booster-warmtepomp aan het
    /// koudedistributiesysteem onttrokken energie voor deze oriëntatie [kWh].
    /// NTA 8800 formule (5.41c). **0 indien geen booster-warmtepomp aanwezig.**
    pub q_c_hp_juli_kwh: f64,

    /// `HC;D;juli;or,zi` — directe warmteoverdrachtcoëfficiënt door transmissie
    /// (excl. beganegrondvloer) voor deze oriëntatie [W/K]. NTA 8800 §5.7.2
    /// stap 5.
    pub h_c_d_juli_w_per_k: f64,

    /// `Hgr;an;juli;or,zi` — warmteoverdrachtcoëfficiënt door transmissie voor
    /// gebouwelementen in thermisch contact met de grond, voor deze oriëntatie
    /// [W/K]. NTA 8800 §5.7.2 (volgens 8.3), gewogen naar `AT;or,zi`.
    pub h_gr_an_juli_w_per_k: f64,

    /// `HC;ve;juli;or,zi` — warmteoverdrachtcoëfficiënt door ventilatie voor
    /// deze oriëntatie [W/K]. NTA 8800 §5.7.2 (volgens 7.4.3), gewogen naar
    /// `AT;or,zi`.
    pub h_c_ve_juli_w_per_k: f64,
}

/// TOjuli-resultaat voor **één oriëntatie**.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TojuliOrientationResult {
    /// De beschouwde oriëntatie.
    pub orientation: Orientation,

    /// `AT;or,zi` [m²] — meegenomen zodat de reden voor `tojuli_k == None`
    /// (≤ [`AT_MIN_M2`]) traceerbaar is.
    pub a_t_m2: f64,

    /// `TOjuli;or,zi` [K], geklemd op ≥ 0 en naar boven afgerond op 0,01.
    /// `None` betekent dat deze oriëntatie **buiten beschouwing** blijft omdat
    /// `AT;or,zi ≤` [`AT_MIN_M2`] (§5.7.2 stap A).
    pub tojuli_k: Option<f64>,
}

/// TOjuli-toetsresultaat voor **één rekenzone**: de indicator per oriëntatie
/// plus de maatgevende waarde en de pass/fail-uitkomst.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TojuliZoneResult {
    /// Resultaat per aangeleverde oriëntatie (zelfde volgorde als de invoer).
    pub per_orientation: Vec<TojuliOrientationResult>,

    /// Maatgevende (hoogste) `TOjuli;or,zi` [K] over de **beoordeelde**
    /// oriëntaties. `0.0` als geen enkele oriëntatie beoordeeld is (alle
    /// `AT ≤ 3 m²`) of als de rekenzone actief gekoeld is.
    pub max_tojuli_k: f64,

    /// De TOjuli-grenswaarde [K] waartegen getoetst is
    /// ([`crate::beng::TOJULI_LIMIT`], Bbl art. 4.149b lid 1).
    pub limit_k: f64,

    /// `true` als de TOjuli-eis wordt gehaald: `max_tojuli_k ≤ limit_k` voor
    /// elke beoordeelde oriëntatie.
    pub pass: bool,

    /// Is de rekenzone actief gekoeld? Zo ja, dan is de zone geacht te voldoen
    /// (§5.7.2) en zijn alle `TOjuli;or,zi = 0`.
    pub actively_cooled: bool,

    /// `true` zodra `pass == false`: de vereenvoudigde TOjuli-eis (Bbl lid 1)
    /// wordt niet gehaald, dus moet via een **GTO-uurberekening** worden
    /// aangetoond dat het aantal gewogen temperatuuroverschrijdingsuren onder
    /// [`gto_limit_hours`](Self::gto_limit_hours) blijft (Bbl lid 2). Die
    /// uurmethode staat niet in NTA 8800 §5.7 en is hier niet geïmplementeerd.
    pub requires_gto_hourly_check: bool,

    /// De GTO-uurgrens [h] van de vervolgroute
    /// ([`crate::beng::GTO_LIMIT_HOURS`], Bbl art. 4.149b lid 2). Louter
    /// informatief; deze module berekent geen GTO-uren.
    pub gto_limit_hours: f64,
}

/// Berekent `TOjuli;or,zi` voor **één oriëntatie** volgens NTA 8800 formule
/// (5.40) (§5.7.2, p. 115):
///
/// ```text
///                (QC;nd;juli;or,zi − QC;HP;juli;or,zi) × 1000
/// TOjuli;or,zi = ────────────────────────────────────────────────────────
///                (HC;D;juli;or,zi + Hgr;an;juli;or,zi + HC;ve;juli;or,zi) × tjuli
/// ```
///
/// De uitkomst wordt geklemd op minimaal 0 ("waarbij de rekenwaarde van
/// TOjuli;or,zi minimaal de waarde 0 heeft") en naar boven afgerond op een
/// veelvoud van 0,01 (p. 119).
///
/// `t_juli_h` is `tjuli`, de rekenwaarde voor de lengte van de maand juli [h]
/// volgens §17.2; de keten levert die aan (deze module hardcodeert geen
/// maandlengte).
///
/// Retourneert `0.0` als de noemer niet-eindig (NaN/±∞) of `≤ 0` is, of als de
/// teller niet-eindig is — dat duidt op een fout in de keten-invoer en zou
/// anders stil NaN/∞ doorgeven (hardening in de stijl van
/// [`crate::beng::BengIndicators::beng1_from_demand`]).
#[must_use]
pub fn tojuli_orientation(input: &TojuliOrientationInput, t_juli_h: f64) -> f64 {
    let numerator = (input.q_c_nd_juli_kwh - input.q_c_hp_juli_kwh) * KWH_TO_WH;
    let h_sum =
        input.h_c_d_juli_w_per_k + input.h_gr_an_juli_w_per_k + input.h_c_ve_juli_w_per_k;
    let denominator = h_sum * t_juli_h;

    debug_assert!(
        denominator.is_finite(),
        "TOjuli-noemer moet eindig zijn, kreeg {denominator}"
    );
    debug_assert!(
        numerator.is_finite(),
        "TOjuli-teller moet eindig zijn, kreeg {numerator}"
    );

    if !denominator.is_finite() || denominator <= 0.0 || !numerator.is_finite() {
        return 0.0;
    }

    round_up_tojuli((numerator / denominator).max(0.0))
}

/// Toetst de TOjuli-indicator voor **één rekenzone** over alle aangeleverde
/// oriëntaties (NTA 8800 §5.7.2 + Bbl art. 4.149b).
///
/// - `orientations`: de per-oriëntatie-invoer (doorgaans tot acht; N…NW).
/// - `t_juli_h`: `tjuli`, lengte van de maand juli [h] (§17.2).
/// - `actively_cooled`: is in deze rekenzone een actief koelsysteem van
///   voldoende capaciteit aanwezig? Zo ja, dan is de zone geacht te voldoen en
///   krijgt elke oriëntatie `TOjuli = 0` (§5.7.2).
///
/// De pass/fail is per oriëntatie (Bbl lid 1: "ten hoogste 1,20 voor iedere
/// rekenzone en oriëntatie"); de maatgevende oriëntatie is de hoogste. Wordt
/// de eis niet gehaald, dan markeert
/// [`TojuliZoneResult::requires_gto_hourly_check`] dat de GTO-uurroute nodig
/// is.
#[must_use]
pub fn tojuli_zone(
    orientations: &[TojuliOrientationInput],
    t_juli_h: f64,
    actively_cooled: bool,
) -> TojuliZoneResult {
    let mut per_orientation = Vec::with_capacity(orientations.len());
    let mut max_tojuli = 0.0_f64;

    for input in orientations {
        let tojuli_k = if actively_cooled {
            // §5.7.2: bij een actief koelsysteem mag TOjuli = 0 worden
            // gehanteerd voor alle oriëntaties.
            Some(0.0)
        } else if input.a_t_m2 <= AT_MIN_M2 {
            // §5.7.2 stap A: oriëntatie blijft buiten beschouwing.
            None
        } else {
            let value = tojuli_orientation(input, t_juli_h);
            max_tojuli = max_tojuli.max(value);
            Some(value)
        };

        per_orientation.push(TojuliOrientationResult {
            orientation: input.orientation,
            a_t_m2: input.a_t_m2,
            tojuli_k,
        });
    }

    let pass = max_tojuli <= TOJULI_LIMIT;

    TojuliZoneResult {
        per_orientation,
        max_tojuli_k: max_tojuli,
        limit_k: TOJULI_LIMIT,
        pass,
        actively_cooled,
        requires_gto_hourly_check: !pass,
        gto_limit_hours: GTO_LIMIT_HOURS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rekenwaarde voor de lengte van de maand juli [h] die in deze tests als
    /// `tjuli` wordt gebruikt: 31 dagen × 24 h = 744 h. In de echte keten komt
    /// `tjuli` uit §17.2; hier volstaat een vaste plausibele waarde om de
    /// formule-transcriptie te toetsen.
    const T_JULI_H: f64 = 744.0;

    /// Bouwt een oriëntatie-invoer met de gegeven koudebehoefte en H-som;
    /// booster-WP = 0, oppervlak ruim boven de 3 m²-drempel.
    fn input(q_c_nd_kwh: f64, h_sum_w_per_k: f64) -> TojuliOrientationInput {
        TojuliOrientationInput {
            orientation: Orientation::Zuid,
            a_t_m2: 20.0,
            q_c_nd_juli_kwh: q_c_nd_kwh,
            q_c_hp_juli_kwh: 0.0,
            // Verdeel de H-som over de drie termen; de formule telt ze op.
            h_c_d_juli_w_per_k: h_sum_w_per_k * 0.5,
            h_gr_an_juli_w_per_k: h_sum_w_per_k * 0.125,
            h_c_ve_juli_w_per_k: h_sum_w_per_k * 0.375,
        }
    }

    // -- Formule-transcriptie (5.40) ---------------------------------------

    #[test]
    fn formule_5_40_handafleiding() {
        // QC;nd = 100 kWh, geen booster-WP, H-som = 40 W/K, tjuli = 744 h.
        // TOjuli = (100 − 0)·1000 / (40·744) = 100 000 / 29 760
        //        = 3,36021… K → naar boven op 0,01 → 3,37 K.
        let v = tojuli_orientation(&input(100.0, 40.0), T_JULI_H);
        assert!((v - 3.37).abs() < 1e-9, "kreeg {v}, verwacht 3,37");
    }

    #[test]
    fn formule_5_40_booster_wp_verlaagt() {
        // QC;nd = 100, QC;HP = 40 → (100 − 40)·1000 / (40·744)
        //        = 60 000 / 29 760 = 2,01612… → 2,02 K.
        let mut inp = input(100.0, 40.0);
        inp.q_c_hp_juli_kwh = 40.0;
        let v = tojuli_orientation(&inp, T_JULI_H);
        assert!((v - 2.02).abs() < 1e-9, "kreeg {v}, verwacht 2,02");
    }

    #[test]
    fn h_termen_worden_gesommeerd() {
        // Drie losse H-termen 20 + 5 + 15 = 40 W/K moeten identiek zijn aan
        // één term van 40. QC;nd = 30 → 30 000 / (40·744) = 1,00806… → 1,01.
        let inp = TojuliOrientationInput {
            orientation: Orientation::West,
            a_t_m2: 10.0,
            q_c_nd_juli_kwh: 30.0,
            q_c_hp_juli_kwh: 0.0,
            h_c_d_juli_w_per_k: 20.0,
            h_gr_an_juli_w_per_k: 5.0,
            h_c_ve_juli_w_per_k: 15.0,
        };
        let v = tojuli_orientation(&inp, T_JULI_H);
        assert!((v - 1.01).abs() < 1e-9, "kreeg {v}, verwacht 1,01");
    }

    // -- Naar-boven-afronding op 0,01 (p. 119) -----------------------------

    #[test]
    fn afronding_altijd_naar_boven() {
        // Kies QC;nd zodat de rauwe waarde net boven een 0,01-veelvoud ligt.
        // 30 000 / 29 760 = 1,008064… → 1,01 (niet 1,00).
        let v = tojuli_orientation(&input(30.0, 40.0), T_JULI_H);
        assert!((v - 1.01).abs() < 1e-9, "kreeg {v}, verwacht 1,01");
    }

    #[test]
    fn afronding_exact_veelvoud_blijft_staan() {
        // Construeer een rauwe waarde die exact 1,20 is:
        // TOjuli = 1,20 ⇒ QC;nd = 1,20 · (40·744) / 1000 = 35,712 kWh.
        let inp = input(35.712, 40.0);
        let v = tojuli_orientation(&inp, T_JULI_H);
        // Mag door de epsilon-marge niet ten onrechte naar 1,21 springen.
        assert!((v - 1.20).abs() < 1e-9, "kreeg {v}, verwacht 1,20");
    }

    // -- Klemmen op minimaal 0 (definitie bij 5.40) ------------------------

    #[test]
    fn negatieve_uitkomst_klemt_op_nul() {
        // Booster-WP onttrekt meer dan de koudebehoefte → teller negatief.
        let mut inp = input(20.0, 40.0);
        inp.q_c_hp_juli_kwh = 50.0;
        let v = tojuli_orientation(&inp, T_JULI_H);
        assert!(v.abs() < 1e-12, "kreeg {v}, verwacht 0");
    }

    #[test]
    fn nul_koudebehoefte_geeft_nul() {
        let v = tojuli_orientation(&input(0.0, 40.0), T_JULI_H);
        assert!(v.abs() < 1e-12, "kreeg {v}, verwacht 0");
    }

    // -- Hardening / randgevallen ------------------------------------------

    #[test]
    fn nul_h_som_geeft_nul() {
        // Noemer 0 (geen warmteoverdracht) → 0,0-fallback i.p.v. ∞.
        let v = tojuli_orientation(&input(100.0, 0.0), T_JULI_H);
        assert!(v.abs() < 1e-12, "kreeg {v}, verwacht 0");
    }

    #[test]
    fn nul_maandlengte_geeft_nul() {
        let v = tojuli_orientation(&input(100.0, 40.0), 0.0);
        assert!(v.abs() < 1e-12, "kreeg {v}, verwacht 0");
    }

    #[test]
    fn negatieve_maandlengte_geeft_nul() {
        // Onzinnige negatieve tjuli → noemer < 0 → zelfde `denominator <= 0.0`-
        // fallbackpad als de nul-maandlengte, hier expliciet getoetst i.p.v.
        // afgeleid.
        let v = tojuli_orientation(&input(100.0, 40.0), -744.0);
        assert!(v.abs() < 1e-12, "kreeg {v}, verwacht 0");
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn niet_eindige_invoer_fallback() {
        // In release-builds (debug_assert uit) geven NaN/∞ de 0,0-fallback.
        let mut inp = input(f64::NAN, 40.0);
        assert_eq!(tojuli_orientation(&inp, T_JULI_H), 0.0);
        inp = input(100.0, f64::INFINITY);
        assert_eq!(tojuli_orientation(&inp, T_JULI_H), 0.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "TOjuli-teller moet eindig zijn")]
    fn nan_teller_debug_assert() {
        let _ = tojuli_orientation(&input(f64::NAN, 40.0), T_JULI_H);
    }

    // -- Zone-toetsing: pass/fail rond 1,20 K ------------------------------

    #[test]
    fn zone_pass_onder_grens() {
        // QC;nd = 30 → 1,01 K ≤ 1,20 → pass, geen GTO-route.
        let zone = tojuli_zone(&[input(30.0, 40.0)], T_JULI_H, false);
        assert!((zone.max_tojuli_k - 1.01).abs() < 1e-9);
        assert!(zone.pass);
        assert!(!zone.requires_gto_hourly_check);
        assert!((zone.limit_k - TOJULI_LIMIT).abs() < 1e-12);
    }

    #[test]
    fn zone_fail_boven_grens_vereist_gto() {
        // QC;nd = 100 → 3,37 K > 1,20 → fail → GTO-uurroute vereist.
        let zone = tojuli_zone(&[input(100.0, 40.0)], T_JULI_H, false);
        assert!(!zone.pass);
        assert!(zone.requires_gto_hourly_check);
        assert!((zone.gto_limit_hours - GTO_LIMIT_HOURS).abs() < 1e-12);
    }

    #[test]
    fn zone_grens_exact_op_120_is_pass() {
        // Rauw 1,20 K → afgerond 1,20 → 1,20 ≤ 1,20 → pass.
        let zone = tojuli_zone(&[input(35.712, 40.0)], T_JULI_H, false);
        assert!((zone.max_tojuli_k - 1.20).abs() < 1e-9);
        assert!(zone.pass);
    }

    #[test]
    fn zone_net_boven_120_is_fail() {
        // QC;nd = 35,8 → 35 800 / 29 760 = 1,20296… → 1,21 > 1,20 → fail.
        let zone = tojuli_zone(&[input(35.8, 40.0)], T_JULI_H, false);
        assert!((zone.max_tojuli_k - 1.21).abs() < 1e-9, "kreeg {}", zone.max_tojuli_k);
        assert!(!zone.pass);
    }

    #[test]
    fn zone_maatgevend_is_hoogste_orientatie() {
        // Drie oriëntaties; de hoogste (Z, 3,37) is maatgevend.
        let mut noord = input(10.0, 40.0);
        noord.orientation = Orientation::Noord;
        let mut oost = input(50.0, 40.0);
        oost.orientation = Orientation::Oost;
        let zuid = input(100.0, 40.0); // Zuid, 3,37
        let zone = tojuli_zone(&[noord, oost, zuid], T_JULI_H, false);
        assert!((zone.max_tojuli_k - 3.37).abs() < 1e-9);
        assert_eq!(zone.per_orientation.len(), 3);
        assert!(!zone.pass);
    }

    // -- Actief gekoeld: TOjuli n.v.t., zone voldoet -----------------------

    #[test]
    fn actief_gekoeld_alles_nul_en_pass() {
        // Zelfs met hoge koudebehoefte: actief gekoeld → alle TOjuli = 0.
        let zone = tojuli_zone(&[input(100.0, 40.0), input(200.0, 40.0)], T_JULI_H, true);
        assert!(zone.actively_cooled);
        assert!((zone.max_tojuli_k).abs() < 1e-12);
        assert!(zone.pass);
        assert!(!zone.requires_gto_hourly_check);
        for res in &zone.per_orientation {
            assert_eq!(res.tojuli_k, Some(0.0));
        }
    }

    // -- AT ≤ 3 m²: oriëntatie buiten beschouwing --------------------------

    #[test]
    fn kleine_orientatie_buiten_beschouwing() {
        // AT = 2,0 m² ≤ 3 → None; telt niet mee in het maximum.
        let mut klein = input(100.0, 40.0); // zou 3,37 geven als beoordeeld
        klein.a_t_m2 = 2.0;
        let zone = tojuli_zone(&[klein], T_JULI_H, false);
        assert_eq!(zone.per_orientation[0].tojuli_k, None);
        assert!(zone.max_tojuli_k.abs() < 1e-12);
        assert!(zone.pass);
    }

    #[test]
    fn drempel_exact_3m2_blijft_buiten_beschouwing() {
        // "≤ 3 m²" is inclusief: exact 3,0 → None.
        let mut op_drempel = input(100.0, 40.0);
        op_drempel.a_t_m2 = 3.0;
        let zone = tojuli_zone(&[op_drempel], T_JULI_H, false);
        assert_eq!(zone.per_orientation[0].tojuli_k, None);
    }

    #[test]
    fn net_boven_drempel_wordt_beoordeeld() {
        let mut boven = input(100.0, 40.0);
        boven.a_t_m2 = 3.0001;
        let zone = tojuli_zone(&[boven], T_JULI_H, false);
        assert!(matches!(zone.per_orientation[0].tojuli_k, Some(v) if (v - 3.37).abs() < 1e-9));
    }

    #[test]
    fn lege_zone_geeft_nul_en_pass() {
        let zone = tojuli_zone(&[], T_JULI_H, false);
        assert!(zone.max_tojuli_k.abs() < 1e-12);
        assert!(zone.pass);
        assert!(zone.per_orientation.is_empty());
    }
}
