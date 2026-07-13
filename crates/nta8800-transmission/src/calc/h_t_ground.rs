//! H_g;an — warmteverlies via de grond (§8.3).
//!
//! Twee paden:
//!
//! 1. **Forfaitair** ([`conductance_via_ground`]) — bijlage I.2.3-fallback: de
//!    consumer levert de jaargemiddelde warmteoverdrachtcoëfficiënt naar de grond
//!    `h_g_an` (W/K) als één getal voor de gehele zone. Gebruikt wanneer de
//!    vloerconstructie-opbouw of de perimeter onbekend is.
//! 2. **P/A-grondmodel** ([`slab_on_ground_conductance`]) — de stationaire
//!    NEN-EN-ISO 13370 bepaling voor een vloer direct op de grond (vloer op
//!    staal), §8.3.2.2–§8.3.4.1: karakteristieke vloerbreedte
//!    `B'_f = A_f / (0,5·P)` (formule 8.30) → equivalente dikte `d_f;equi`
//!    (formule 8.32) → `U_fl` (formule 8.40/8.41) → `H_g = A_fl · U_fl`
//!    (formule 8.36, zonder de aparte `ψ_gr`-vloerrandterm — die reist in deze
//!    keten via de lineaire koudebruggen). Vervangt het forfaitaire `h_g;an`
//!    zodra de perimeter `P` bekend is.
//!
//! De maandelijkse faseverschuiving uit bijlage D wordt (nog) niet toegepast:
//! deze keten gebruikt de stationaire `H_g` als de jaargemiddelde
//! `H_g;an`, zoals de forfait-tak. Dat blijft de gedocumenteerde
//! vereenvoudiging t.o.v. de volledige bijlage-D-periodiek.
//!
//! Let op: formule (7.14) vermenigvuldigt `H_g;an` met **`θ_e;avg;an`**
//! (jaargemiddelde), niet met `θ_e;avg;mi` (maandgemiddelde). De module
//! [`super`] doet deze uitsplitsing aan de caller-zijde.

use crate::model::{BoundaryType, TransmissionElement};

/// Warmtegeleidingscoëfficiënt van de ondergrond `λ_gr` in W/(m·K).
/// NTA 8800:2025+C1:2026 §8.3.2.4.1, formule (8.35): vaste rekenwaarde 2,0.
pub const LAMBDA_GROUND_W_PER_MK: f64 = 2.0;

/// Warmteovergangsweerstand aan de uitwendige (grond-)zijde `R_se` in (m²·K)/W.
/// NTA 8800 §8.3.2.3 OPMERKING 8: voor een aan grond grenzende constructie wordt
/// de buitenlucht-waarde 0,04 aangehouden (de weerstand wordt omgezet naar een
/// aan grond equivalente constructie die bij het maaiveld aan buitenlucht grenst).
pub const R_SE_GROUND_M2K_PER_W: f64 = 0.04;

/// Volledige wanddikte `d_bw` in m die in de equivalente-diktebepaling wordt
/// aangehouden. NTA 8800 §8.3.2.3: "waarvoor de waarde 0,5 m wordt aangehouden".
pub const WALL_THICKNESS_M: f64 = 0.5;

/// Stationaire grond-warmteverliescoëfficiënt `H_g` in W/K van een vloer direct
/// op de grond (vloer op staal, `z = 0`), volgens NEN-EN-ISO 13370 zoals
/// overgenomen in NTA 8800:2025+C1:2026 §8.3.2.2–§8.3.4.1.
///
/// # Argumenten
///
/// - `floor_area_m2` — `A_fl`, de (bruto) vloeroppervlakte in m² (formule 8.36).
/// - `perimeter_m` — `P`, de blootgestelde perimeter in m (formule 8.30/8.31):
///   de som van de randlengtes die grenzen aan buitenlucht of aan een
///   onverwarmde ruimte buiten de thermische schil.
/// - `floor_u_value` — de warmtedoorgangscoëfficiënt van de vloerconstructie
///   zoals in deze keten opgeslagen, d.w.z. `1/(R_si + R_c)` (R_se = 0 voor
///   grondcontact). De reciproke `1/U` = `R_si + R_c` voedt de equivalente
///   dikte; deze functie telt zelf `R_se = 0,04` erbij (§8.3.2.3 OPMERKING 8).
///
/// # Formules
///
/// - `B'_f = A_fl / (0,5·P)` (8.30);
/// - `d_f;equi = d_bw + λ_gr·(R_si + R_c + R_se)` (8.32), met
///   `R_si + R_c = 1/floor_u_value` en `R_se = 0,04`;
/// - ongeïsoleerd/matig (`d_f;equi < B'_f`, 8.40):
///   `U_fl = 2·λ_gr/(π·B'_f + d_f;equi) · ln(π·B'_f/d_f;equi + 1)`;
/// - goed geïsoleerd (`d_f;equi ≥ B'_f`, 8.41):
///   `U_fl = λ_gr/(0,457·B'_f + d_f;equi)`;
/// - `H_g = A_fl · U_fl` (8.36).
///
/// Retourneert `0,0` bij een niet-positieve oppervlakte, perimeter of U-waarde
/// (geen grondcontact / ongeldige invoer) i.p.v. te panieken.
#[must_use]
pub fn slab_on_ground_conductance(floor_area_m2: f64, perimeter_m: f64, floor_u_value: f64) -> f64 {
    // `> 0.0` sluit NaN en niet-positieve waarden al uit; boven-oneindig is
    // fysiek onmogelijk hier, dus een expliciete finite-check is overbodig.
    if !(floor_area_m2 > 0.0 && perimeter_m > 0.0 && floor_u_value > 0.0) {
        return 0.0;
    }

    // B'_f — karakteristieke vloerbreedte (8.30).
    let b_prime = floor_area_m2 / (0.5 * perimeter_m);

    // d_f;equi — equivalente dikte (8.32). 1/U = R_si + R_c (R_se = 0 in de
    // opgeslagen U); tel R_se = 0,04 erbij voor (R_si + R_c + R_se).
    let r_si_plus_rc = 1.0 / floor_u_value;
    let d_equi =
        WALL_THICKNESS_M + LAMBDA_GROUND_W_PER_MK * (r_si_plus_rc + R_SE_GROUND_M2K_PER_W);

    // U_fl — vloer op staal, z = 0 (8.40/8.41).
    let u_fl = if d_equi < b_prime {
        (2.0 * LAMBDA_GROUND_W_PER_MK / (std::f64::consts::PI * b_prime + d_equi))
            * (std::f64::consts::PI * b_prime / d_equi + 1.0).ln()
    } else {
        LAMBDA_GROUND_W_PER_MK / (0.457 * b_prime + d_equi)
    };

    floor_area_m2 * u_fl
}

/// Totale jaargemiddelde warmteoverdrachtcoëfficiënt via grond in W/K.
///
/// Deze helper bestaat om het contract tussen de zone-samenstelling (welke
/// elementen zijn `Ground`?) en de door de consumer aangeleverde `h_g_an`
/// expliciet te maken. De huidige V1-implementatie retourneert gewoon de
/// meegegeven `h_g_an` als er ten minste één [`BoundaryType::Ground`] element
/// is, anders 0.
///
/// Dit voorkomt dat consumers per ongeluk een restwaarde voor `h_g_an`
/// meegeven op zones zonder grondcontact (bovengelegen appartement, etc.).
#[must_use]
pub fn conductance_via_ground(elements: &[TransmissionElement], h_g_an: f64) -> f64 {
    let has_ground = elements
        .iter()
        .any(|el| matches!(el.boundary_type, BoundaryType::Ground));
    if has_ground {
        h_g_an
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground_el(id: &str, area: f64, u: f64) -> TransmissionElement {
        TransmissionElement {
            id: id.into(),
            area,
            u_value: u,
            boundary_type: BoundaryType::Ground,
            construction_id: None,
        }
    }

    #[test]
    fn zero_if_no_ground_elements() {
        let els = vec![TransmissionElement {
            id: "outdoor".into(),
            area: 10.0,
            u_value: 1.0,
            boundary_type: BoundaryType::Outdoor,
            construction_id: None,
        }];
        assert!((conductance_via_ground(&els, 25.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn returns_supplied_value_when_ground_present() {
        let els = vec![ground_el("floor", 60.0, 0.3)];
        assert!((conductance_via_ground(&els, 18.5) - 18.5).abs() < 1e-12);
    }

    #[test]
    fn multiple_ground_elements_still_return_single_hg_an() {
        // V1: h_g_an is reeds geaggregeerd voor de zone. Consumers die dit
        // splitsen per vloer, sommeren dat zelf op vóór aanroep.
        let els = vec![
            ground_el("floor-1", 60.0, 0.3),
            ground_el("floor-2", 20.0, 0.4),
        ];
        assert!((conductance_via_ground(&els, 22.0) - 22.0).abs() < 1e-12);
    }

    /// P/A-grondmodel reproduceert de handmatig nagerekende certified Aalten-vloer
    /// (§8.3.2.2–§8.3.4.1). A = 67,0 m², P = 32,92 m, Rc = 3,70 (U = 0,258398):
    /// B'_f = 67/(0,5·32,92) = 4,0705 m; d_f;equi = 0,5 + 2·(1/0,258398 + 0,04) =
    /// 8,320 m ≥ B'_f → goed geïsoleerd (8.41): U_fl = 2/(0,457·4,0705 + 8,320) =
    /// 0,19646; H_g = 67·0,19646 = 13,163 W/K.
    #[test]
    fn slab_on_ground_matches_hand_calc_aalten() {
        let u_floor = 1.0 / (0.17 + 3.70); // R_si(vloer) + R_c, R_se = 0 → 0,258398
        let h_g = slab_on_ground_conductance(67.0, 32.92, u_floor);
        assert!((h_g - 13.163).abs() < 0.01, "H_g = {h_g}, verwacht ~13,163");
    }

    /// Ongeïsoleerde vloer (`d_f;equi < B'_f`) volgt de logaritmische tak (8.40).
    #[test]
    fn slab_on_ground_uses_log_branch_when_poorly_insulated() {
        // R_c ~ 0 (ongeïsoleerd): d_equi = 0,5 + 2·(0,17 + 0,04) = 0,92 m,
        // ruime vloer B'_f groot → d_equi < B'_f.
        let u_floor = 1.0 / 0.17; // R_si only
        let h_g = slab_on_ground_conductance(100.0, 40.0, u_floor);
        // B'_f = 100/20 = 5,0; d_equi = 0,92 < 5,0 → log-tak.
        let b = 5.0_f64;
        let d = 0.5 + 2.0 * (0.17 + 0.04);
        let u_fl = (2.0 * 2.0 / (std::f64::consts::PI * b + d))
            * (std::f64::consts::PI * b / d + 1.0).ln();
        assert!((h_g - 100.0 * u_fl).abs() < 1e-9);
    }

    /// Ongeldige/afwezige invoer → 0 (geen grondcontact, geen paniek).
    #[test]
    fn slab_on_ground_zero_for_invalid_input() {
        assert_eq!(slab_on_ground_conductance(0.0, 30.0, 0.25), 0.0);
        assert_eq!(slab_on_ground_conductance(60.0, 0.0, 0.25), 0.0);
        assert_eq!(slab_on_ground_conductance(60.0, 30.0, 0.0), 0.0);
        assert_eq!(slab_on_ground_conductance(f64::NAN, 30.0, 0.25), 0.0);
    }
}
