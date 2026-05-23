//! Specifieke infiltratie-luchtvolumestroom q_is.
//!
//! Bronnen:
//! - ISSO 53 (2016) tabel 4.5, PDF p.45-46 — q_is bij bekende q_v10,kar;
//! - ISSO 53 (2016) tabel 4.9, PDF p.47 — q_i,spec,reken (onbekende q_v10,kar).
//!
//! Tabel 4.5 koppelt de specifieke infiltratie q_is [m³/(s·m² gevelopp.)] aan
//! de luchtdichtheidsklasse q_v10,kar [dm³/(s·m² gebruiksopp.)] en de
//! gebouwhoogte h [m]. Tabel 4.9 geeft de rekenwaarde q_i,spec,reken die in
//! formule 4.31 wordt gebruikt wanneer q_v10,kar onbekend is.

use crate::model::enums::BuildingShape;

/// Luchtdichtheidsklasse voor q_v10,kar (ISSO 53 tabel 4.5, rijen).
///
/// q_v10,kar wordt uitgedrukt in dm³/(s·m² gebruiksoppervlak); dit is de
/// EPC-waarde van het gebouw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Qv10Class {
    /// q_v10,kar < 0,20.
    LessThan020,
    /// 0,20 ≤ q_v10,kar < 0,40.
    From020To040,
    /// 0,40 ≤ q_v10,kar < 0,60.
    From040To060,
    /// 0,60 ≤ q_v10,kar < 0,80.
    From060To080,
    /// 0,80 ≤ q_v10,kar ≤ 1,00.
    From080To100,
    /// q_v10,kar > 1,0.
    GreaterThan100,
}

impl Qv10Class {
    /// Bepaalt de luchtdichtheidsklasse voor een q_v10,kar-waarde in
    /// dm³/(s·m² gebruiksoppervlak). ISSO 53 tabel 4.5 (PDF p.45).
    pub fn from_value(qv10_kar: f64) -> Self {
        if qv10_kar < 0.20 {
            Qv10Class::LessThan020
        } else if qv10_kar < 0.40 {
            Qv10Class::From020To040
        } else if qv10_kar < 0.60 {
            Qv10Class::From040To060
        } else if qv10_kar < 0.80 {
            Qv10Class::From060To080
        } else if qv10_kar <= 1.00 {
            Qv10Class::From080To100
        } else {
            Qv10Class::GreaterThan100
        }
    }
}

/// Gebouwhoogteklasse voor q_is (ISSO 53 tabel 4.5, kolommen).
///
/// De gebouwhoogte h is gedefinieerd als de hoogte boven het maaiveld van de
/// bovenste verdiepingsvloer (voetnoot 1 tabel 4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingHeightClass {
    /// h ≤ 3 m.
    UpTo3,
    /// 3 < h ≤ 6 m.
    From3To6,
    /// 6 < h ≤ 20 m.
    From6To20,
    /// 20 < h ≤ 30 m.
    From20To30,
    /// h > 30 m.
    Above30,
}

impl BuildingHeightClass {
    /// Bepaalt de hoogteklasse voor een gebouwhoogte h in meters.
    /// ISSO 53 tabel 4.5 (PDF p.45).
    pub fn from_height(height_m: f64) -> Self {
        if height_m <= 3.0 {
            BuildingHeightClass::UpTo3
        } else if height_m <= 6.0 {
            BuildingHeightClass::From3To6
        } else if height_m <= 20.0 {
            BuildingHeightClass::From6To20
        } else if height_m <= 30.0 {
            BuildingHeightClass::From20To30
        } else {
            BuildingHeightClass::Above30
        }
    }

    /// Kolomindex (0-4) in [`QIS_TABLE_4_5`].
    fn index(self) -> usize {
        match self {
            BuildingHeightClass::UpTo3 => 0,
            BuildingHeightClass::From3To6 => 1,
            BuildingHeightClass::From6To20 => 2,
            BuildingHeightClass::From20To30 => 3,
            BuildingHeightClass::Above30 => 4,
        }
    }
}

/// Tabel 4.5 — specifieke infiltratie q_is in m³/(s·m² geveloppervlak).
/// ISSO 53 (PDF p.45-46).
///
/// Rij-index: [`Qv10Class`] in volgorde `<0,20`, `0,20-0,40`, `0,40-0,60`,
/// `0,60-0,80`, `0,80-1,00`, `>1,0`. Kolom-index: [`BuildingHeightClass`]
/// in volgorde `≤3`, `3-6`, `6-20`, `20-30`, `>30`.
pub const QIS_TABLE_4_5: [[f64; 5]; 6] = [
    // q_v10,kar < 0,20
    [0.00026, 0.00034, 0.00043, 0.00051, 0.00062],
    // 0,20 - 0,40
    [0.00039, 0.00050, 0.00063, 0.00077, 0.00092],
    // 0,40 - 0,60
    [0.00064, 0.00082, 0.00103, 0.00126, 0.00149],
    // 0,60 - 0,80
    [0.00088, 0.00111, 0.00140, 0.00172, 0.00200],
    // 0,80 - 1,00
    [0.00109, 0.00138, 0.00175, 0.00213, 0.00251],
    // q_v10,kar > 1,0
    [0.00118, 0.00151, 0.00189, 0.00232, 0.00273],
];

/// Specifieke infiltratie q_is bij bekende q_v10,kar.
/// ISSO 53 tabel 4.5 (PDF p.45-46), eenheid m³/(s·m² geveloppervlak).
pub fn q_is_known(qv10_class: Qv10Class, height_class: BuildingHeightClass) -> f64 {
    let row = match qv10_class {
        Qv10Class::LessThan020 => 0,
        Qv10Class::From020To040 => 1,
        Qv10Class::From040To060 => 2,
        Qv10Class::From060To080 => 3,
        Qv10Class::From080To100 => 4,
        Qv10Class::GreaterThan100 => 5,
    };
    QIS_TABLE_4_5[row][height_class.index()]
}

/// Specifieke infiltratie q_is bij bekende q_v10,kar — convenience-variant
/// die direct op de ruwe waarden q_v10,kar [dm³/(s·m² gebruiksopp.)] en
/// gebouwhoogte h [m] keyt. ISSO 53 tabel 4.5 (PDF p.45-46).
pub fn q_is_known_from_values(qv10_kar: f64, height_m: f64) -> f64 {
    q_is_known(
        Qv10Class::from_value(qv10_kar),
        BuildingHeightClass::from_height(height_m),
    )
}

/// Rekenwaarde specifieke luchtvolumestroom infiltratie q_i,spec,reken.
/// ISSO 53 tabel 4.9 (PDF p.47), eenheid m³/(s·m²).
///
/// Gebruikt in formule 4.33 wanneer q_v10,kar onbekend is:
/// `q_i,spec = f_typ · f_jaar · q_i,spec,reken`.
pub fn q_i_spec_reken(shape: BuildingShape) -> f64 {
    match shape {
        BuildingShape::EenLaagMetKap => 0.0010,
        BuildingShape::EenLaagMetHalfPlatDak => 0.00085,
        BuildingShape::EenLaagMetPlatDak => 0.0007,
        BuildingShape::Meerlaags => 0.0005,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qv10_class_boundaries() {
        assert_eq!(Qv10Class::from_value(0.10), Qv10Class::LessThan020);
        assert_eq!(Qv10Class::from_value(0.20), Qv10Class::From020To040);
        assert_eq!(Qv10Class::from_value(0.39), Qv10Class::From020To040);
        assert_eq!(Qv10Class::from_value(0.40), Qv10Class::From040To060);
        assert_eq!(Qv10Class::from_value(0.60), Qv10Class::From060To080);
        assert_eq!(Qv10Class::from_value(0.80), Qv10Class::From080To100);
        assert_eq!(Qv10Class::from_value(1.00), Qv10Class::From080To100);
        assert_eq!(Qv10Class::from_value(1.01), Qv10Class::GreaterThan100);
    }

    #[test]
    fn test_height_class_boundaries() {
        assert_eq!(BuildingHeightClass::from_height(3.0), BuildingHeightClass::UpTo3);
        assert_eq!(BuildingHeightClass::from_height(3.1), BuildingHeightClass::From3To6);
        assert_eq!(BuildingHeightClass::from_height(6.0), BuildingHeightClass::From3To6);
        assert_eq!(
            BuildingHeightClass::from_height(20.0),
            BuildingHeightClass::From6To20
        );
        assert_eq!(
            BuildingHeightClass::from_height(30.0),
            BuildingHeightClass::From20To30
        );
        assert_eq!(BuildingHeightClass::from_height(31.0), BuildingHeightClass::Above30);
    }

    #[test]
    fn test_q_is_known_corners() {
        // Tabel 4.5 hoekwaarden (PDF p.45-46).
        assert_eq!(
            q_is_known(Qv10Class::LessThan020, BuildingHeightClass::UpTo3),
            0.00026
        );
        assert_eq!(
            q_is_known(Qv10Class::GreaterThan100, BuildingHeightClass::Above30),
            0.00273
        );
        assert_eq!(
            q_is_known(Qv10Class::From040To060, BuildingHeightClass::From6To20),
            0.00103
        );
    }

    #[test]
    fn test_q_is_known_from_values() {
        assert_eq!(q_is_known_from_values(0.5, 10.0), 0.00103);
        assert_eq!(q_is_known_from_values(0.05, 2.0), 0.00026);
    }

    #[test]
    fn test_q_i_spec_reken() {
        assert_eq!(q_i_spec_reken(BuildingShape::EenLaagMetKap), 0.0010);
        assert_eq!(q_i_spec_reken(BuildingShape::EenLaagMetHalfPlatDak), 0.00085);
        assert_eq!(q_i_spec_reken(BuildingShape::EenLaagMetPlatDak), 0.0007);
        assert_eq!(q_i_spec_reken(BuildingShape::Meerlaags), 0.0005);
    }
}
