//! NTA 8800:2025+C1:2026 §11.2 norm-tabellen voor het luchtstroommodel.
//!
//! Deze module bevat de **data en lookup-functies** uit de norm-paragrafen die
//! het massabalans-/drukmodel van §11.2.1 voeden. De rekenlogica zelf
//! (iteratieve `p_z;ref`-oplosroutine, C2.2) leeft in [`crate::calc`].
//!
//! | Tabel | Norm-§ | Inhoud |
//! |---|---|---|
//! | 11.2 | §11.2.1.3 | Stromingsexponenten `n` per stroomtype |
//! | 11.3 | §11.2.1.4 | Dimensieloze winddrukcoëfficiënten `C_p` per hoogteklasse |
//! | 11.13 | §11.2.5.1 | Bouwjaarcorrectiefactor `f_j` |
//! | 11.14 | §11.2.5.2 | Rekenwaarde `q_v10;spec;reken` + gebouwtype-correctie `f_type` |
//!
//! Norm-bron: NTA 8800:2025+C1:2026 PDF p. 439-440 (tabel 11.2/11.3) en
//! p. 485-489 (§11.2.5, tabel 11.13/11.14). NEN-licentie 3BM, intern gebruik.
//!
//! # Scope-grens C2
//!
//! C2 modelleert **één luchtstroomzone met gebouwhoogte < 15 m** — alleen de
//! hoogteklasse `Laag` uit tabel 11.3 is daarmee relevant. De klassen `Middel`
//! en `Hoog` zijn volledigheidshalve opgenomen maar buiten C2-scope (hoogbouw
//! met meerdere luchtstroomzones is V2).

use crate::model::BuildingLeakageType;

// ===========================================================================
// Tabel 11.2 — Stromingsexponent (n)
// ===========================================================================

/// Stromingsexponent `n_lea` voor **lekverliezen** (infiltratie).
///
/// NTA 8800:2025+C1:2026 tabel 11.2 (§11.2.1.3, PDF p. 439), rij
/// "Lekverliezen". Gebruikt in formule (11.84)/(11.85) om het 10 Pa-debiet
/// naar het 1 Pa-referentiedebiet om te rekenen.
pub const FLOW_EXPONENT_LEAKAGE: f64 = 0.67;

/// Stromingsexponent `n_vent` voor **ventilatietoevoervoorzieningen**
/// (regelbare roosters / toevoeropeningen).
///
/// NTA 8800:2025+C1:2026 tabel 11.2 (§11.2.1.3, PDF p. 439), rij
/// "Ventilatietoevoervoorzieningen".
pub const FLOW_EXPONENT_VENTILATION: f64 = 0.5;

/// Stromingsexponent `n_argI` voor **verplichte spuivoorzieningen**.
///
/// NTA 8800:2025+C1:2026 tabel 11.2 (§11.2.1.3, PDF p. 439), rij
/// "Verplichte spuivoorzieningen". Spui is V2-scope; de constante is hier
/// alleen voor norm-volledigheid.
pub const FLOW_EXPONENT_PURGE: f64 = 0.5;

/// Stromingsexponent `n_comb` voor **open verbrandingstoestellen**.
///
/// NTA 8800:2025+C1:2026 tabel 11.2 (§11.2.1.3, PDF p. 439), rij
/// "Open verbrandingstoestellen". Verbrandingslucht is V2-scope.
pub const FLOW_EXPONENT_COMBUSTION: f64 = 0.5;

// ===========================================================================
// Tabel 11.3 — Dimensieloze winddrukcoëfficiënten (C_p)
// ===========================================================================

/// Hoogteklasse van de luchtstroom op de gevel volgens NTA 8800 tabel 11.3.
///
/// De klasse bepaalt de winddrukcoëfficiënten `C_p` per geveloriëntatie.
///
/// NTA 8800:2025+C1:2026 tabel 11.3 (§11.2.1.4, PDF p. 440).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightClass {
    /// Laag — `h_path < 15 m`. **Enige C2-relevante klasse** (één
    /// luchtstroomzone, laagbouw).
    Low,
    /// Middel — `15 m ≤ h_path < 50 m`. V2-scope (hoogbouw multi-zone).
    Medium,
    /// Hoog — `h_path ≥ 50 m`. V2-scope (hoogbouw multi-zone).
    High,
}

/// Dimensieloze winddrukcoëfficiënten `C_p` voor één hoogteklasse uit
/// NTA 8800 tabel 11.3.
///
/// Per gevel-/vlakpositie. De norm geeft `roof` en `floor` alleen voor de
/// klasse `Laag`; voor `Middel` en `Hoog` ontbreekt de vloer-coëfficiënt
/// (de tabel toont daar `–`) — vandaar `Option<f64>` voor `floor`.
///
/// NTA 8800:2025+C1:2026 tabel 11.3 (§11.2.1.4, PDF p. 440).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindPressureCoefficients {
    /// `C_p` loefzijde (windward) — de naar de wind gekeerde gevel.
    pub windward: f64,
    /// `C_p` lijzijde (leeward) — de van de wind afgekeerde gevel.
    pub leeward: f64,
    /// `C_p` dak.
    pub roof: f64,
    /// `C_p` vloer. `None` voor `Middel`/`Hoog` (norm geeft daar `–`).
    pub floor: Option<f64>,
}

/// Winddrukcoëfficiënten `C_p` voor de hoogteklasse `Laag` (`h < 15 m`).
///
/// NTA 8800:2025+C1:2026 tabel 11.3, rij "Laag h_path < 15 m":
/// loef +0,25, lij −0,50, dak −0,60, vloer −0,20.
///
/// **Dit is de enige C2-scope hoogteklasse.**
pub const WIND_PRESSURE_LOW: WindPressureCoefficients = WindPressureCoefficients {
    windward: 0.25,
    leeward: -0.50,
    roof: -0.60,
    floor: Some(-0.20),
};

/// Winddrukcoëfficiënten `C_p` voor de hoogteklasse `Middel`
/// (`15 m ≤ h < 50 m`).
///
/// NTA 8800:2025+C1:2026 tabel 11.3, rij "Middel 15 ≤ h_path < 50 m":
/// loef +0,45, lij −0,50, dak −0,60, vloer `–` (niet gegeven).
///
/// V2-scope — buiten C2.
pub const WIND_PRESSURE_MEDIUM: WindPressureCoefficients = WindPressureCoefficients {
    windward: 0.45,
    leeward: -0.50,
    roof: -0.60,
    floor: None,
};

/// Winddrukcoëfficiënten `C_p` voor de hoogteklasse `Hoog` (`h ≥ 50 m`).
///
/// NTA 8800:2025+C1:2026 tabel 11.3, rij "Hoog h_path ≥ 50 m":
/// loef +0,80, lij −0,70, dak −0,70, vloer `–` (niet gegeven).
///
/// V2-scope — buiten C2.
pub const WIND_PRESSURE_HIGH: WindPressureCoefficients = WindPressureCoefficients {
    windward: 0.80,
    leeward: -0.70,
    roof: -0.70,
    floor: None,
};

/// Grenshoogte (m) tussen klasse `Laag` en `Middel` — NTA 8800 tabel 11.3.
///
/// `h_path < 15 m` is klasse `Laag`. C2 is beperkt tot deze grens.
pub const HEIGHT_CLASS_LOW_MAX_M: f64 = 15.0;

/// Grenshoogte (m) tussen klasse `Middel` en `Hoog` — NTA 8800 tabel 11.3.
///
/// `15 m ≤ h_path < 50 m` is klasse `Middel`.
pub const HEIGHT_CLASS_MEDIUM_MAX_M: f64 = 50.0;

impl HeightClass {
    /// Bepaal de hoogteklasse uit de gebouwhoogte (m) volgens NTA 8800
    /// tabel 11.3.
    ///
    /// - `h < 15`  → [`HeightClass::Low`]
    /// - `15 ≤ h < 50` → [`HeightClass::Medium`]
    /// - `h ≥ 50`  → [`HeightClass::High`]
    #[must_use]
    pub fn from_height(height_m: f64) -> Self {
        if height_m < HEIGHT_CLASS_LOW_MAX_M {
            Self::Low
        } else if height_m < HEIGHT_CLASS_MEDIUM_MAX_M {
            Self::Medium
        } else {
            Self::High
        }
    }

    /// Winddrukcoëfficiënten `C_p` voor deze hoogteklasse — NTA 8800 tabel 11.3.
    #[must_use]
    pub const fn wind_pressure_coefficients(self) -> WindPressureCoefficients {
        match self {
            Self::Low => WIND_PRESSURE_LOW,
            Self::Medium => WIND_PRESSURE_MEDIUM,
            Self::High => WIND_PRESSURE_HIGH,
        }
    }
}

// ===========================================================================
// Tabel 11.13 — Bouwjaarcorrectiefactor (f_j)
// ===========================================================================

/// Bouwjaarcorrectiefactor `f_j` voor de rekenwaarde van de specifieke
/// luchtdoorlatendheid `q_v10;spec;reken` — NTA 8800 tabel 11.13.
///
/// De bouwkwaliteit (en daarmee de luchtdichtheid) is over de jaren
/// verbeterd; de factor corrigeert de forfaitaire rekenwaarde naar het
/// bouw- of (volledige) renovatiejaar `j`.
///
/// NTA 8800:2025+C1:2026 tabel 11.13 (§11.2.5.1, PDF p. 486):
///
/// | Bouwjaar/renovatiejaar | f_j |
/// |---|---|
/// | `j < 1970`            | 3,0 |
/// | `1970 ≤ j < 1980`     | 2,5 |
/// | `1980 ≤ j < 1990`     | 2,0 |
/// | `1990 ≤ j < 2000`     | 1,5 |
/// | `2000 ≤ j < 2010`     | 1,0 |
/// | `j ≥ 2010`            | 0,7 |
///
/// # Argument
///
/// - `build_year`: bouwjaar of (bij nagenoeg volledige renovatie)
///   renovatiejaar `j`.
///
/// # Norm-noot
///
/// Het renovatiejaar mag alleen worden aangehouden bij (nagenoeg) volledige
/// renovatie; het verbeteren van een enkel aspect (bv. kierdichting) is
/// onvoldoende (NTA 8800 §11.2.5.1, PDF p. 487).
#[must_use]
pub fn build_year_correction_factor(build_year: u32) -> f64 {
    // NTA 8800 tabel 11.13 — klassengrenzen exact zoals de norm.
    match build_year {
        y if y < 1970 => 3.0,
        1970..=1979 => 2.5,
        1980..=1989 => 2.0,
        1990..=1999 => 1.5,
        2000..=2009 => 1.0,
        _ => 0.7, // j ≥ 2010
    }
}

// ===========================================================================
// Tabel 11.14 — Rekenwaarde q_v10;spec;reken + gebouwtype-correctie f_type
// ===========================================================================

/// Rekenwaarde voor de specifieke luchtdoorlatendheid `q_v10;spec;reken`
/// per gebouwcategorie, in dm³/(s·m²) bij een uniform drukverschil van 10 Pa.
///
/// NTA 8800:2025+C1:2026 tabel 11.14 (§11.2.5.2, PDF p. 487-488), kolom
/// `q_v10;spec;calc`. De norm onderscheidt drie categorieën:
///
/// | Categorie | q_v10;spec;reken |
/// |---|---|
/// | Grondgebonden (eengezins met kap + enkellaagse utiliteitsbouw) | 1,0 |
/// | Eengezins met plat dak + overige enkellaagse utiliteitsbouw    | 0,7 |
/// | Meerlaagse gebouwen (etages, flat-/portiekwoningen)            | 0,5 |
///
/// # Argument
///
/// - `leakage_type`: het [`BuildingLeakageType`] dat de gebouwcategorie én
///   uitvoeringsvariant codeert.
// Eén match-arm per NTA 8800 tabel-11.14-categorie blijft expliciet voor
// audit-traceability — verschillende categorieën met dezelfde rekenwaarde
// worden bewust niet samengevoegd.
#[allow(clippy::match_same_arms)]
#[must_use]
pub fn specific_air_permeability_calc(leakage_type: BuildingLeakageType) -> f64 {
    use BuildingLeakageType as B;
    match leakage_type {
        // Categorie "Grondgebonden gebouwen" — q_v10;spec;reken = 1,0.
        B::GroundBoundTerracedPitchedRoof
        | B::GroundBoundEndPitchedRoof
        | B::GroundBoundDetachedPitchedRoof
        | B::GroundBoundDetachedPartlyFlatRoof => 1.0,
        // Categorie "Eengezins plat dak + overige enkellaagse" — 0,7.
        B::SingleStoreyFlatRoofTerraced
        | B::SingleStoreyFlatRoofEnd
        | B::SingleStoreyFlatRoofDetached => 0.7,
        // Categorie "Meerlaagse gebouwen" — 0,5.
        B::MultiStoreyLowerTerraced
        | B::MultiStoreyLowerEnd
        | B::MultiStoreyTopTerraced
        | B::MultiStoreyTopEnd => 0.5,
        // Forfaitaire combinatiewaarden voor een heel meerlaags gebouw
        // (footnote a) — q_v10;spec;reken blijft de meerlaagse 0,5.
        B::MultiStoreyWholeBuilding
        | B::MultiStoreyTopLayer
        | B::MultiStoreyIntermediateLayer
        | B::MultiStoreyBottomLayer => 0.5,
    }
}

/// Van de gebouwuitvoering afhankelijke correctiefactor `f_type` op de
/// rekenwaarde van de specifieke luchtdoorlatendheid.
///
/// NTA 8800:2025+C1:2026 tabel 11.14 (§11.2.5.2, PDF p. 487-489), kolom
/// `f_type`. Per gebouwcategorie en uitvoeringsvariant:
///
/// **Grondgebonden gebouwen** (`q_v10;spec;reken = 1,0`):
///
/// | Uitvoeringsvariant | f_type |
/// |---|---|
/// | Tussenligging met kap                | 1,0 |
/// | Kop-/eind-/hoekligging met kap        | 1,2 |
/// | Vrijstaand gebouw, hellend dak        | 1,4 |
/// | Vrijstaand gebouw, deels plat dak     | 1,2 |
///
/// **Eengezins plat dak + overige enkellaagse** (`q_v10;spec;reken = 0,7`):
///
/// | Uitvoeringsvariant | f_type |
/// |---|---|
/// | Tussenligging              | 1,0 |
/// | Kop-/eind-/hoekligging     | 1,2 |
/// | Vrijstaand gebouw, plat dak| 1,4 |
///
/// **Meerlaagse gebouwen** (`q_v10;spec;reken = 0,5`):
///
/// | Uitvoeringsvariant | f_type |
/// |---|---|
/// | Tussenligging op onderste/tussenverdieping        | 1,0 |
/// | Kop-/eind-/hoekligging op onderste/tussenverd.    | 1,3 |
/// | Tussenligging op bovenste verdieping              | 1,2 |
/// | Kop-/eind-/hoekligging op bovenste verdieping     | 1,4 |
///
/// **Footnote a — combinaties van eenheden in een meerlaags gebouw:**
///
/// | Beschouwd deel | f_type |
/// |---|---|
/// | Het gebouw als geheel              | 1,2 |
/// | De gehele bovenste gebouwlaag      | 1,3 |
/// | Een volledige tussengelegen laag   | 1,2 |
/// | De hele onderste gebouwlaag        | 1,1 |
// Eén match-arm per NTA 8800 tabel-11.14-rij blijft expliciet voor audit-
// traceability — verschillende uitvoeringsvarianten met dezelfde f_type
// (bv. 1,2) worden bewust niet samengevoegd tot één arm.
#[allow(clippy::match_same_arms)]
#[must_use]
pub fn building_type_correction_factor(leakage_type: BuildingLeakageType) -> f64 {
    use BuildingLeakageType as B;
    match leakage_type {
        // --- Grondgebonden gebouwen (q = 1,0) ---
        B::GroundBoundTerracedPitchedRoof => 1.0,
        B::GroundBoundEndPitchedRoof => 1.2,
        B::GroundBoundDetachedPitchedRoof => 1.4,
        B::GroundBoundDetachedPartlyFlatRoof => 1.2,
        // --- Eengezins plat dak + overige enkellaagse (q = 0,7) ---
        B::SingleStoreyFlatRoofTerraced => 1.0,
        B::SingleStoreyFlatRoofEnd => 1.2,
        B::SingleStoreyFlatRoofDetached => 1.4,
        // --- Meerlaagse gebouwen (q = 0,5) ---
        B::MultiStoreyLowerTerraced => 1.0,
        B::MultiStoreyLowerEnd => 1.3,
        B::MultiStoreyTopTerraced => 1.2,
        B::MultiStoreyTopEnd => 1.4,
        // --- Footnote a — combinaties (PDF p. 488-489) ---
        B::MultiStoreyWholeBuilding => 1.2,
        B::MultiStoreyTopLayer => 1.3,
        B::MultiStoreyIntermediateLayer => 1.2,
        B::MultiStoreyBottomLayer => 1.1,
    }
}

// ===========================================================================
// Tests — assert tegen norm-tabelwaarden
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn flow_exponents_match_table_11_2() {
        // NTA 8800 tabel 11.2: lek 0,67; vent/spui/comb 0,5.
        assert_relative_eq!(FLOW_EXPONENT_LEAKAGE, 0.67, epsilon = 1e-12);
        assert_relative_eq!(FLOW_EXPONENT_VENTILATION, 0.5, epsilon = 1e-12);
        assert_relative_eq!(FLOW_EXPONENT_PURGE, 0.5, epsilon = 1e-12);
        assert_relative_eq!(FLOW_EXPONENT_COMBUSTION, 0.5, epsilon = 1e-12);
    }

    #[test]
    fn height_class_boundaries_match_table_11_3() {
        // < 15 m → Laag; [15, 50) → Middel; ≥ 50 → Hoog.
        assert_eq!(HeightClass::from_height(0.0), HeightClass::Low);
        assert_eq!(HeightClass::from_height(14.99), HeightClass::Low);
        assert_eq!(HeightClass::from_height(15.0), HeightClass::Medium);
        assert_eq!(HeightClass::from_height(49.99), HeightClass::Medium);
        assert_eq!(HeightClass::from_height(50.0), HeightClass::High);
        assert_eq!(HeightClass::from_height(100.0), HeightClass::High);
    }

    #[test]
    fn wind_pressure_low_matches_table_11_3() {
        // NTA 8800 tabel 11.3, rij Laag: loef +0,25 / lij −0,50 / dak −0,60 /
        // vloer −0,20.
        let cp = HeightClass::Low.wind_pressure_coefficients();
        assert_relative_eq!(cp.windward, 0.25, epsilon = 1e-12);
        assert_relative_eq!(cp.leeward, -0.50, epsilon = 1e-12);
        assert_relative_eq!(cp.roof, -0.60, epsilon = 1e-12);
        assert_relative_eq!(cp.floor.unwrap(), -0.20, epsilon = 1e-12);
    }

    #[test]
    fn wind_pressure_medium_high_match_table_11_3() {
        // Middel: loef +0,45 / lij −0,50 / dak −0,60 / vloer geen waarde.
        let mid = HeightClass::Medium.wind_pressure_coefficients();
        assert_relative_eq!(mid.windward, 0.45, epsilon = 1e-12);
        assert_relative_eq!(mid.leeward, -0.50, epsilon = 1e-12);
        assert_relative_eq!(mid.roof, -0.60, epsilon = 1e-12);
        assert!(mid.floor.is_none(), "Norm geeft '–' voor vloer bij Middel");
        // Hoog: loef +0,80 / lij −0,70 / dak −0,70 / vloer geen waarde.
        let high = HeightClass::High.wind_pressure_coefficients();
        assert_relative_eq!(high.windward, 0.80, epsilon = 1e-12);
        assert_relative_eq!(high.leeward, -0.70, epsilon = 1e-12);
        assert_relative_eq!(high.roof, -0.70, epsilon = 1e-12);
        assert!(high.floor.is_none(), "Norm geeft '–' voor vloer bij Hoog");
    }

    #[test]
    fn build_year_correction_matches_table_11_13() {
        // NTA 8800 tabel 11.13 — controleer elke klasse + zijn grenzen.
        assert_relative_eq!(build_year_correction_factor(1900), 3.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1969), 3.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1970), 2.5, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1979), 2.5, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1980), 2.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1989), 2.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1990), 1.5, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(1999), 1.5, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(2000), 1.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(2009), 1.0, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(2010), 0.7, epsilon = 1e-12);
        assert_relative_eq!(build_year_correction_factor(2026), 0.7, epsilon = 1e-12);
    }

    #[test]
    fn specific_air_permeability_matches_table_11_14() {
        use BuildingLeakageType as B;
        // Grondgebonden categorie → 1,0.
        assert_relative_eq!(
            specific_air_permeability_calc(B::GroundBoundTerracedPitchedRoof),
            1.0,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            specific_air_permeability_calc(B::GroundBoundDetachedPitchedRoof),
            1.0,
            epsilon = 1e-12
        );
        // Eengezins plat dak categorie → 0,7.
        assert_relative_eq!(
            specific_air_permeability_calc(B::SingleStoreyFlatRoofTerraced),
            0.7,
            epsilon = 1e-12
        );
        // Meerlaagse categorie → 0,5.
        assert_relative_eq!(
            specific_air_permeability_calc(B::MultiStoreyLowerTerraced),
            0.5,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            specific_air_permeability_calc(B::MultiStoreyWholeBuilding),
            0.5,
            epsilon = 1e-12
        );
    }

    #[test]
    fn building_type_correction_matches_table_11_14() {
        use BuildingLeakageType as B;
        // Grondgebonden: tussen 1,0 / kop 1,2 / vrijstaand hellend 1,4 /
        // vrijstaand deels plat 1,2.
        assert_relative_eq!(
            building_type_correction_factor(B::GroundBoundTerracedPitchedRoof),
            1.0,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::GroundBoundEndPitchedRoof),
            1.2,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::GroundBoundDetachedPitchedRoof),
            1.4,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::GroundBoundDetachedPartlyFlatRoof),
            1.2,
            epsilon = 1e-12
        );
        // Eengezins plat dak: tussen 1,0 / kop 1,2 / vrijstaand plat 1,4.
        assert_relative_eq!(
            building_type_correction_factor(B::SingleStoreyFlatRoofTerraced),
            1.0,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::SingleStoreyFlatRoofEnd),
            1.2,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::SingleStoreyFlatRoofDetached),
            1.4,
            epsilon = 1e-12
        );
        // Meerlaags: tussen-onder 1,0 / kop-onder 1,3 / tussen-boven 1,2 /
        // kop-boven 1,4.
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyLowerTerraced),
            1.0,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyLowerEnd),
            1.3,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyTopTerraced),
            1.2,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyTopEnd),
            1.4,
            epsilon = 1e-12
        );
        // Footnote a — combinaties: geheel 1,2 / boven 1,3 / tussen 1,2 /
        // onder 1,1.
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyWholeBuilding),
            1.2,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyTopLayer),
            1.3,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyIntermediateLayer),
            1.2,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            building_type_correction_factor(B::MultiStoreyBottomLayer),
            1.1,
            epsilon = 1e-12
        );
    }
}
