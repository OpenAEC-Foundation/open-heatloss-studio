//! Parameters voor de equivalente warmtedoorgangscoëfficiënt U_equiv,k.
//!
//! Bron: ISSO 53 (2016) tabel 4.3, PDF p.44.
//!
//! De parameters a, b, c1/c2/c3, n1/n2/n3 en d worden ingevuld in formule
//! 4.24 om U_equiv,k te bepalen voor vloeren en wanden in contact met de
//! grond. De **formule zelf** (4.24) hoort thuis in `calc/ground.rs` — deze
//! module bevat alleen de coëfficiënten.
//!
//! Randvoorwaarden uit ISSO 53 §4.6 (PDF p.43-44):
//! - hulpwaarde `B' = 2·A_vl / O`, geclamped `2 ≤ B' ≤ 50`;
//! - vloerdiepte onder maaiveld `0 ≤ z ≤ 5` m;
//! - grondwaterfactor `f_gw = 1` (grondwater ≥ 1 m onder vloer) of `1,15`;
//! - bij wanden heeft B' geen invloed (c1 = n1 = 0), maar B' mag rekenkundig
//!   niet 0 zijn.
//!
//! Cross-validatie formule 4.24: de letterlijke machtsstructuur kon niet
//! betrouwbaar uit de PDF-tekstlaag/OCR worden afgeleid (gerenderde
//! formule-afbeelding). De `calc/ground.rs`-implementatie is impliciet
//! geverifieerd via Vabi-fixtures (`tests/verification/isso53_*`) waar
//! ground-floor elementen voorkomen en `phi_t` binnen norm-tolerantie
//! matcht — zie sessie 8 cross-val (commit 0f4293a).

/// Vlaktype waarvoor de U_equiv-parameters gelden.
/// ISSO 53 tabel 4.3 (PDF p.44).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundSurfaceKind {
    /// Vloer in contact met de grond.
    Floor,
    /// Wand in contact met de grond.
    Wall,
}

/// Parameterset voor formule 4.24 (bepaling van U_equiv,k).
/// ISSO 53 tabel 4.3 (PDF p.44).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundParams {
    /// Parameter a.
    pub a: f64,
    /// Parameter b.
    pub b: f64,
    /// Parameter c1. Voor wanden 0 (B' heeft geen invloed).
    pub c1: f64,
    /// Parameter c2.
    pub c2: f64,
    /// Parameter c3.
    pub c3: f64,
    /// Parameter n1. Voor wanden 0 (B' heeft geen invloed).
    pub n1: f64,
    /// Parameter n2.
    pub n2: f64,
    /// Parameter n3.
    pub n3: f64,
    /// Parameter d.
    pub d: f64,
}

/// Parameters voor vloeren in contact met de grond.
/// ISSO 53 tabel 4.3 (PDF p.44).
pub const GROUND_PARAMS_FLOOR: GroundParams = GroundParams {
    a: 0.9671,
    b: -7.455,
    c1: 10.76,
    c2: 9.773,
    c3: 0.0265,
    n1: 0.5532,
    n2: 0.6027,
    n3: -0.9296,
    d: -0.0203,
};

/// Parameters voor wanden in contact met de grond.
/// ISSO 53 tabel 4.3 (PDF p.44).
///
/// Voor wanden zijn c1 en n1 gelijk aan 0: B' heeft geen invloed op het
/// warmteverlies door wanden. B' mag rekenkundig echter niet 0 zijn.
pub const GROUND_PARAMS_WALL: GroundParams = GroundParams {
    a: 0.799,
    b: -6.7951,
    c1: 0.0,
    c2: 26.586,
    c3: 0.1523,
    n1: 0.0,
    n2: 0.5012,
    n3: -0.1406,
    d: -1.074,
};

/// Ondergrens voor de equivalente warmtedoorgangscoëfficiënt U_equiv,k.
/// ISSO 53 §4.6 — `U_equiv,k ≥ 0,1 W/(m²·K)`.
pub const U_EQUIV_MIN: f64 = 0.1;

/// Minimale hulpwaarde B' (geometrische factor) — clamp-ondergrens.
/// ISSO 53 §4.6 (PDF p.43): `2 ≤ B' ≤ 50`.
pub const B_PRIME_MIN: f64 = 2.0;

/// Maximale hulpwaarde B' (geometrische factor) — clamp-bovengrens.
/// ISSO 53 §4.6 (PDF p.43): `2 ≤ B' ≤ 50`.
pub const B_PRIME_MAX: f64 = 50.0;

/// Maximale vloerdiepte z onder maaiveld — clamp-bovengrens.
/// ISSO 53 §4.6 (PDF p.44): `0 ≤ z ≤ 5` m.
pub const Z_DEPTH_MAX: f64 = 5.0;

/// Retourneert de parameterset voor formule 4.24 voor het gegeven vlaktype.
/// ISSO 53 tabel 4.3 (PDF p.44).
pub fn ground_params(kind: GroundSurfaceKind) -> GroundParams {
    match kind {
        GroundSurfaceKind::Floor => GROUND_PARAMS_FLOOR,
        GroundSurfaceKind::Wall => GROUND_PARAMS_WALL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_params() {
        let p = ground_params(GroundSurfaceKind::Floor);
        assert_eq!(p.a, 0.9671);
        assert_eq!(p.b, -7.455);
        assert_eq!(p.c1, 10.76);
        assert_eq!(p.c2, 9.773);
        assert_eq!(p.c3, 0.0265);
        assert_eq!(p.n1, 0.5532);
        assert_eq!(p.n2, 0.6027);
        assert_eq!(p.n3, -0.9296);
        assert_eq!(p.d, -0.0203);
    }

    #[test]
    fn test_wall_params() {
        let p = ground_params(GroundSurfaceKind::Wall);
        assert_eq!(p.a, 0.799);
        assert_eq!(p.b, -6.7951);
        // B' heeft geen invloed bij wanden: c1 = n1 = 0.
        assert_eq!(p.c1, 0.0);
        assert_eq!(p.n1, 0.0);
        assert_eq!(p.c2, 26.586);
        assert_eq!(p.c3, 0.1523);
        assert_eq!(p.n2, 0.5012);
        assert_eq!(p.n3, -0.1406);
        assert_eq!(p.d, -1.074);
    }

    #[test]
    fn test_clamp_constants() {
        assert_eq!(U_EQUIV_MIN, 0.1);
        assert_eq!(B_PRIME_MIN, 2.0);
        assert_eq!(B_PRIME_MAX, 50.0);
        assert_eq!(Z_DEPTH_MAX, 5.0);
    }
}
