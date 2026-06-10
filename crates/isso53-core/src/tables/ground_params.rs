//! Parameters voor de equivalente warmtedoorgangscoëfficiënt U_equiv,k.
//!
//! Bron: ISSO 53 (2016) formule 4.24 + tabel 4.3, PDF p.44.
//!
//! De norm-vorm van formule 4.24 (visueel geverifieerd tegen de bron-PDF,
//! PM-verificatie 2026-06-10 — de formule staat als gerenderde afbeelding in
//! de PDF, de tekstlaag bevat alleen het formulenummer):
//!
//! ```text
//! U_equiv,k = a / ( b + (c₁ + B')^n₁ + (c₂ + z)^n₂ + (c₃ + U_k + ΔU_TB)^n₃ ) + d
//! ```
//!
//! De c-parameters zijn dus **addenden binnen de machten**, `b` is een
//! somterm in de noemer en `d` staat buiten de breuk. De **formule zelf**
//! hoort thuis in `calc/ground.rs` — deze module bevat alleen de
//! coëfficiënten uit tabel 4.3.
//!
//! Randvoorwaarden uit ISSO 53 §4.6 (PDF p.43-44):
//! - hulpwaarde `B' = 2·A_vl / O`, geclamped `2 ≤ B' ≤ 50`;
//! - vloerdiepte onder maaiveld `0 ≤ z ≤ 5` m (z > 5 → z = 5);
//! - grondwaterfactor `f_gw = 1` (grondwater ≥ 1 m onder vloer) of `1,15`;
//! - bij wanden heeft B' geen invloed (c1 = n1 = 0 → `(0 + B')^0 = 1`),
//!   maar B' mag rekenkundig niet 0 zijn (norm-voetnoot 1 bij tabel 4.3);
//! - bij meerdere vloer-U's in de beganegrondvloer: oppervlaktegewogen
//!   gemiddelde U-waarde vóór toepassing van de formule (norm-opmerking).
//!
//! IJkpunten uit de norm-voorbeelden (zie tests in `calc/ground.rs`):
//! - schilvoorbeeld PDF p.59: B' = 14,29, z = 0, U_k+ΔU_TB = 0,36954
//!   → U_equiv ≈ 0,181 (norm: 0,18);
//! - detailvoorbeeld PDF p.65: B' = 12,07, z = 0, U_k+ΔU_TB = 0,31
//!   → U_equiv ≈ 0,177 (norm: 0,177).

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
    /// Parameter a (teller van de breuk).
    pub a: f64,
    /// Parameter b (somterm in de noemer).
    pub b: f64,
    /// Parameter c1 (addend bij B'). Voor wanden 0 (B' heeft geen invloed).
    pub c1: f64,
    /// Parameter c2 (addend bij z).
    pub c2: f64,
    /// Parameter c3 (addend bij U_k + ΔU_TB).
    pub c3: f64,
    /// Parameter n1 (exponent op (c1 + B')). Voor wanden 0 → term = 1.
    pub n1: f64,
    /// Parameter n2 (exponent op (c2 + z)).
    pub n2: f64,
    /// Parameter n3 (exponent op (c3 + U_k + ΔU_TB)).
    pub n3: f64,
    /// Parameter d (term buiten de breuk).
    pub d: f64,
}

/// Parameters voor vloeren in contact met de grond.
/// ISSO 53 tabel 4.3 (PDF p.44), letterlijk overgenomen.
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
/// ISSO 53 tabel 4.3 (PDF p.44), letterlijk overgenomen.
///
/// Voor wanden zijn c1 en n1 gelijk aan 0: B' heeft geen invloed op het
/// warmteverlies door wanden (de term wordt `(0 + B')^0 = 1`). B' mag
/// rekenkundig echter niet 0 zijn (voetnoot 1 bij tabel 4.3).
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
/// ISSO 53 formule 4.24 (PDF p.44): `0 ≤ z ≤ 5` m; indien z > 5 m dan z = 5 m.
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

    /// Tabel 4.3 (PDF p.44), rij "Vloer" — letterlijke norm-waarden.
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

    /// Tabel 4.3 (PDF p.44), rij "Wand" — letterlijke norm-waarden.
    #[test]
    fn test_wall_params() {
        let p = ground_params(GroundSurfaceKind::Wall);
        assert_eq!(p.a, 0.799);
        assert_eq!(p.b, -6.7951);
        // B' heeft geen invloed bij wanden: c1 = n1 = 0 → (0+B')^0 = 1.
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
