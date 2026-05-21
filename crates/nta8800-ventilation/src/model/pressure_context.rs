//! [`BuildingPressureContext`] — invoer voor het NTA 8800 §11.2.1 drukmodel.
//!
//! Dit struct is een **aanvullende** invoer naast [`crate::AirFlow`]: het levert
//! de gebouw-eigenschappen die de iteratieve massabalans-/druk-oplosroutine
//! (`p_z;ref`, §11.2.1.5/§11.2.1.6 — C2.2) nodig heeft. `AirFlow` (3 scalars)
//! blijft de bestaande publieke input-API; `BuildingPressureContext` vervangt
//! die niet.
//!
//! # C2-scope
//!
//! C2 modelleert **één luchtstroomzone met gebouwhoogte `< 15 m`** (laagbouw,
//! hoogteklasse `Laag` uit NTA 8800 tabel 11.3). Hoogbouw met meerdere
//! luchtstroomzones — waarvoor de hoogteklassen `Middel`/`Hoog` en een
//! per-zone-druk-oplossing nodig zijn — is **V2-scope**.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Gebouwtype-classificatie voor de gebouwtype-correctie uit NTA 8800
/// tabel 11.14 (§11.2.5.2).
///
/// De variant codeert zowel de **gebouwcategorie** (die de rekenwaarde
/// `q_v10;spec;reken` bepaalt) als de **uitvoeringsvariant** (die de
/// correctiefactor `f_type` bepaalt). De bijbehorende lookups staan in
/// [`crate::tables::specific_air_permeability_calc`] en
/// [`crate::tables::building_type_correction_factor`].
///
/// NTA 8800:2025+C1:2026 tabel 11.14, PDF p. 487-489.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildingLeakageType {
    // --- Grondgebonden gebouwen (q_v10;spec;reken = 1,0) ---
    /// Grondgebonden, tussenligging met kap. `f_type = 1,0`.
    GroundBoundTerracedPitchedRoof,
    /// Grondgebonden, kop-/eind-/hoekligging met kap. `f_type = 1,2`.
    GroundBoundEndPitchedRoof,
    /// Grondgebonden, vrijstaand gebouw met hellend dak. `f_type = 1,4`.
    GroundBoundDetachedPitchedRoof,
    /// Grondgebonden, vrijstaand gebouw met deels plat dak. `f_type = 1,2`.
    GroundBoundDetachedPartlyFlatRoof,

    // --- Eengezins plat dak + overige enkellaagse (q_v10;spec;reken = 0,7) ---
    /// Eengezins plat dak / overige enkellaagse, tussenligging. `f_type = 1,0`.
    SingleStoreyFlatRoofTerraced,
    /// Eengezins plat dak / overige enkellaagse, kop-/eind-/hoekligging.
    /// `f_type = 1,2`.
    SingleStoreyFlatRoofEnd,
    /// Eengezins plat dak / overige enkellaagse, vrijstaand gebouw met plat
    /// dak. `f_type = 1,4`.
    SingleStoreyFlatRoofDetached,

    // --- Meerlaagse gebouwen (q_v10;spec;reken = 0,5) ---
    /// Meerlaags, tussenligging op onderste of tussenverdieping.
    /// `f_type = 1,0`.
    MultiStoreyLowerTerraced,
    /// Meerlaags, kop-/eind-/hoekligging op onderste of tussenverdieping.
    /// `f_type = 1,3`.
    MultiStoreyLowerEnd,
    /// Meerlaags, tussenligging op bovenste verdieping. `f_type = 1,2`.
    MultiStoreyTopTerraced,
    /// Meerlaags, kop-/eind-/hoekligging op bovenste verdieping.
    /// `f_type = 1,4`.
    MultiStoreyTopEnd,

    // --- Footnote a: forfaitaire combinatiewaarden meerlaags gebouw ---
    /// Meerlaags gebouw als geheel (footnote a). `f_type = 1,2`.
    MultiStoreyWholeBuilding,
    /// Gehele bovenste gebouwlaag van een meerlaags gebouw (footnote a).
    /// `f_type = 1,3`.
    MultiStoreyTopLayer,
    /// Volledige tussengelegen gebouwlaag van een meerlaags gebouw
    /// (footnote a). `f_type = 1,2`.
    MultiStoreyIntermediateLayer,
    /// Hele onderste gebouwlaag van een meerlaags gebouw (footnote a).
    /// `f_type = 1,1`.
    MultiStoreyBottomLayer,
}

impl BuildingLeakageType {
    /// Forfaitaire default — grondgebonden tussenwoning met kap.
    ///
    /// De meest voorkomende Nederlandse woningvorm; bruikbaar als zinnige
    /// startwaarde wanneer het gebouwtype (nog) niet bekend is.
    #[must_use]
    pub const fn default_value() -> Self {
        Self::GroundBoundTerracedPitchedRoof
    }
}

impl Default for BuildingLeakageType {
    fn default() -> Self {
        Self::default_value()
    }
}

/// Maximale gebouwhoogte (m) binnen C2-scope — één luchtstroomzone.
///
/// Komt overeen met de bovengrens van hoogteklasse `Laag` uit NTA 8800
/// tabel 11.3 ([`crate::tables::HEIGHT_CLASS_LOW_MAX_M`]). Gebouwen met
/// `height_m ≥ 15` vereisen een multi-zone-aanpak (V2).
pub const C2_MAX_BUILDING_HEIGHT_M: f64 = 15.0;

/// Aanvullende invoer voor het NTA 8800 §11.2.1 druk-/massabalansmodel.
///
/// Levert de gebouw-eigenschappen voor de iteratieve `p_z;ref`-oplosroutine
/// (C2.2). Dit struct **vult** [`crate::AirFlow`] aan — het vervangt die
/// publieke input-API niet.
///
/// # C2-scope: één luchtstroomzone, `height_m < 15`
///
/// Het model rekent met één luchtstroomzone. Hoogbouw met meerdere
/// luchtstroomzones (en daarmee de hoogteklassen `Middel`/`Hoog` uit
/// tabel 11.3) is V2. [`BuildingPressureContext::within_c2_scope`] toetst de
/// hoogtegrens; consumers horen die check uit te voeren vóór ze het drukmodel
/// aanroepen.
///
/// # Eenheden
///
/// | Veld | Eenheid |
/// |---|---|
/// | `building_height_m` | m |
/// | `gross_floor_area_m2` (`A_g`) | m² |
/// | `build_year` | jaartal (geen eenheid) |
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BuildingPressureContext {
    /// Gebouwhoogte boven maaiveld, in m.
    ///
    /// Bepaalt de hoogteklasse voor de winddrukcoëfficiënten `C_p`
    /// (NTA 8800 tabel 11.3, via [`crate::tables::HeightClass::from_height`]).
    /// **C2-scope vereist `building_height_m < 15`** — zie
    /// [`Self::within_c2_scope`].
    pub building_height_m: f64,

    /// Bouwjaar (of, bij nagenoeg volledige renovatie, renovatiejaar) van het
    /// gebouw.
    ///
    /// `None` als onbekend — dan kan geen forfaitaire
    /// bouwjaarcorrectie `f_j` (NTA 8800 tabel 11.13) worden bepaald en moet
    /// het infiltratie-referentiedebiet uit een meting komen.
    pub build_year: Option<u32>,

    /// Gebruiksoppervlakte `A_g` van de rekenzone, in m².
    ///
    /// Schaalt het forfaitaire infiltratie-referentiedebiet `q_v1;lea;ref`
    /// (NTA 8800 formule (11.85)).
    pub gross_floor_area_m2: f64,

    /// Gebouwtype-classificatie voor de NTA 8800 tabel 11.14 correctie
    /// (`q_v10;spec;reken` + `f_type`).
    pub leakage_type: BuildingLeakageType,
}

impl BuildingPressureContext {
    /// Bouw een [`BuildingPressureContext`] zonder validatie.
    ///
    /// Plausibiliteitscontroles (hoogte, `A_g > 0`) gebeuren in de
    /// reken-entry van C2.2; dit is een pure constructor.
    #[must_use]
    pub const fn new(
        building_height_m: f64,
        build_year: Option<u32>,
        gross_floor_area_m2: f64,
        leakage_type: BuildingLeakageType,
    ) -> Self {
        Self {
            building_height_m,
            build_year,
            gross_floor_area_m2,
            leakage_type,
        }
    }

    /// Of dit gebouw binnen de C2-scope valt: één luchtstroomzone,
    /// `building_height_m < 15` (NTA 8800 tabel 11.3 hoogteklasse `Laag`).
    ///
    /// `false` betekent dat het volledige multi-zone-drukmodel (V2) nodig is.
    #[must_use]
    pub fn within_c2_scope(&self) -> bool {
        self.building_height_m < C2_MAX_BUILDING_HEIGHT_M
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_leakage_type_is_terraced_pitched_roof() {
        assert_eq!(
            BuildingLeakageType::default(),
            BuildingLeakageType::GroundBoundTerracedPitchedRoof
        );
        assert_eq!(
            BuildingLeakageType::default_value(),
            BuildingLeakageType::GroundBoundTerracedPitchedRoof
        );
    }

    #[test]
    fn within_c2_scope_respects_15m_boundary() {
        // < 15 m → binnen C2-scope.
        let low = BuildingPressureContext::new(
            9.0,
            Some(2015),
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        assert!(low.within_c2_scope());

        // ≥ 15 m → buiten C2-scope (multi-zone, V2).
        let high = BuildingPressureContext::new(
            15.0,
            Some(2015),
            120.0,
            BuildingLeakageType::MultiStoreyWholeBuilding,
        );
        assert!(!high.within_c2_scope());

        let very_high = BuildingPressureContext::new(
            45.0,
            None,
            500.0,
            BuildingLeakageType::MultiStoreyTopLayer,
        );
        assert!(!very_high.within_c2_scope());
    }

    #[test]
    fn constructor_stores_all_fields() {
        let ctx = BuildingPressureContext::new(
            8.5,
            Some(1985),
            95.0,
            BuildingLeakageType::SingleStoreyFlatRoofEnd,
        );
        assert!((ctx.building_height_m - 8.5).abs() < 1e-9);
        assert_eq!(ctx.build_year, Some(1985));
        assert!((ctx.gross_floor_area_m2 - 95.0).abs() < 1e-9);
        assert_eq!(ctx.leakage_type, BuildingLeakageType::SingleStoreyFlatRoofEnd);
    }

    #[test]
    fn serde_round_trip_pressure_context() {
        let ctx = BuildingPressureContext::new(
            11.2,
            Some(1972),
            140.0,
            BuildingLeakageType::MultiStoreyLowerEnd,
        );
        let json = serde_json::to_string(&ctx).unwrap();
        let back: BuildingPressureContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, back);
    }

    #[test]
    fn serde_round_trip_pressure_context_unknown_build_year() {
        let ctx = BuildingPressureContext::new(
            7.0,
            None,
            80.0,
            BuildingLeakageType::GroundBoundDetachedPitchedRoof,
        );
        let json = serde_json::to_string(&ctx).unwrap();
        let back: BuildingPressureContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, back);
        assert_eq!(back.build_year, None);
    }

    #[test]
    fn leakage_type_serde_is_snake_case() {
        let json = serde_json::to_string(&BuildingLeakageType::MultiStoreyWholeBuilding).unwrap();
        assert_eq!(json, "\"multi_storey_whole_building\"");
        let back: BuildingLeakageType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, BuildingLeakageType::MultiStoreyWholeBuilding);
    }
}
