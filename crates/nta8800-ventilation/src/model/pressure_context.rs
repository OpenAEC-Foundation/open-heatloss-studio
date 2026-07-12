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

    /// Gemeten of (onder kwaliteitsborging) verklaarde specifieke
    /// luchtdoorlatendheid `q_v10;lea;ref` bij Δp = 10 Pa, in **dm³/(s·m²)**
    /// per **gebruiksoppervlakte** `A_g` (niet schiloppervlak).
    ///
    /// Vervangt — indien aanwezig — de forfaitaire waarde uit formule (11.86).
    /// NTA 8800 §11.2.5 (PDF p. 485): *"Ingeval de specifieke luchtvolumestroom
    /// die wordt doorgelaten bij 10 Pa, q_v10;lea;ref, op basis van meting
    /// [NEN 2686:1988] is vastgesteld, wordt deze waarde voor de berekening van
    /// de luchtstroom door infiltratie gebruikt."* — en bij kwaliteitsborging
    /// *"moet die waarde worden gebruikt"*. Referentie-oppervlak: OPMERKING 2
    /// (PDF p. 486) — de meetwaarde `q_v10;gemeten` wordt gedeeld door de
    /// gebruiksoppervlakte, identiek aan de eenheid van de forfaitaire waarde
    /// die via formule (11.85) met `A_g` wordt geschaald.
    ///
    /// `None` → val terug op het forfait ([`Self::forfait_q_v10`], formule
    /// (11.86)). Zie [`Self::effective_q_v10`] voor de prioriteitsvolgorde.
    ///
    /// **Preconditie:** een specifieke luchtdoorlatendheid is ≥ 0 en eindig.
    /// `Some(0.0)` is geldig (perfecte luchtdichtheid → `C_lea = 0`). Deze crate
    /// is een pure rekenlaag en toetst dat niet; consumers weigeren een
    /// ongeldige waarde op de invoergrens (bv. `TojuliError::InvalidQv10Spec`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_q_v10_spec: Option<f64>,
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
            measured_q_v10_spec: None,
        }
    }

    /// Zet de gemeten/verklaarde specifieke luchtdoorlatendheid
    /// `q_v10;lea;ref` (dm³/(s·m²) per `A_g`) — builder-stijl zodat
    /// [`Self::new`] signatuur-stabiel blijft.
    ///
    /// `Some(v)` laat de meting het forfait overrulen (zie
    /// [`Self::effective_q_v10`]); `None` behoudt het forfait-pad.
    #[must_use]
    pub const fn with_measured_q_v10_spec(mut self, measured_q_v10_spec: Option<f64>) -> Self {
        self.measured_q_v10_spec = measured_q_v10_spec;
        self
    }

    /// Of dit gebouw binnen de C2-scope valt: één luchtstroomzone,
    /// `building_height_m < 15` (NTA 8800 tabel 11.3 hoogteklasse `Laag`).
    ///
    /// `false` betekent dat het volledige multi-zone-drukmodel (V2) nodig is.
    #[must_use]
    pub fn within_c2_scope(&self) -> bool {
        self.building_height_m < C2_MAX_BUILDING_HEIGHT_M
    }

    /// Forfaitaire specifieke luchtdoorlatendheid bij 10 Pa, `q_v10;lea;ref`
    /// (dm³/(s·m²)), volgens NTA 8800 formule (11.86) — met de
    /// `build_year`-`None`-check ingebouwd.
    ///
    /// Geeft `None` wanneer [`Self::build_year`] onbekend is: zonder bouwjaar
    /// kan de bouwjaarcorrectiefactor `f_y` (tabel 11.13) niet worden bepaald
    /// en is een forfaitaire waarde dus niet af te leiden — dan is een
    /// meetwaarde uit een luchtdoorlatendheidsmeting (NEN 2686:1988) vereist.
    ///
    /// Deze methode neemt de `unwrap()` van het bouwjaar weg bij de
    /// solver-call-sites: zij krijgen een `Option<f64>` in plaats van zelf
    /// `build_year` te moeten ontleden.
    ///
    /// Referentie: NTA 8800:2025+C1:2026 formule (11.86), §11.2.5.2,
    /// PDF p. 485-486.
    #[must_use]
    pub fn forfait_q_v10(&self) -> Option<f64> {
        self.build_year
            .map(|year| crate::calc::infiltration::q_v10_lea_ref_forfait(self.leakage_type, year))
    }

    /// De effectief te gebruiken specifieke luchtdoorlatendheid `q_v10;lea;ref`
    /// (dm³/(s·m²)) voor de infiltratie-`C_lea`, met de NTA 8800 §11.2.5
    /// prioriteitsvolgorde:
    ///
    /// 1. **Gemeten/verklaarde meetwaarde** ([`Self::measured_q_v10_spec`]) —
    ///    heeft voorrang; de norm schrijft haar gebruik voor zodra ze is
    ///    vastgesteld (meting NEN 2686:1988) of onder kwaliteitsborging is
    ///    vastgelegd (PDF p. 485).
    /// 2. **Forfait** ([`Self::forfait_q_v10`], formule (11.86)) — alleen
    ///    *"indien er geen meetwaarde beschikbaar is"* (PDF p. 485); vereist een
    ///    bekend bouwjaar voor `f_y` (tabel 11.13).
    ///
    /// Geeft `None` als beide ontbreken (geen meting én onbekend bouwjaar) —
    /// dan is er geen forfaitaire `C_lea` af te leiden.
    ///
    /// Merk op: de meetwaarde heeft géén bouwjaar nodig; een project met een
    /// gemeten `q_v10;spec` maar zonder `build_year` levert hier tóch een
    /// waarde.
    ///
    /// Referentie: NTA 8800:2025+C1:2026 §11.2.5, PDF p. 485-486.
    #[must_use]
    pub fn effective_q_v10(&self) -> Option<f64> {
        self.measured_q_v10_spec.or_else(|| self.forfait_q_v10())
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
    fn forfait_q_v10_known_build_year_matches_formula_11_86() {
        // Grondgebonden tussenwoning met kap, bouwjaar 2015:
        //   q_v10;spec;reken = 1,0 · f_type = 1,0 · f_y = 0,7 → 0,7 dm³/(s·m²).
        let ctx = BuildingPressureContext::new(
            8.0,
            Some(2015),
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        let q = ctx.forfait_q_v10().expect("bouwjaar bekend → Some");
        assert!((q - 0.7).abs() < 1e-12);
    }

    #[test]
    fn forfait_q_v10_unknown_build_year_is_none() {
        // Zonder bouwjaar kan f_y niet bepaald worden → geen forfait.
        let ctx = BuildingPressureContext::new(
            8.0,
            None,
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        assert_eq!(ctx.forfait_q_v10(), None);
    }

    #[test]
    fn effective_q_v10_measured_wins_over_forfait() {
        // Bouwjaar 2015 → forfait 0,7; meting 0,35 moet winnen (§11.2.5).
        let ctx = BuildingPressureContext::new(
            8.0,
            Some(2015),
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        )
        .with_measured_q_v10_spec(Some(0.35));
        assert_eq!(ctx.forfait_q_v10(), Some(0.7));
        assert_eq!(ctx.effective_q_v10(), Some(0.35));
    }

    #[test]
    fn effective_q_v10_measured_without_build_year() {
        // Geen bouwjaar → forfait None, maar een meting levert tóch een waarde.
        let ctx = BuildingPressureContext::new(
            8.0,
            None,
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        )
        .with_measured_q_v10_spec(Some(0.6));
        assert_eq!(ctx.forfait_q_v10(), None);
        assert_eq!(ctx.effective_q_v10(), Some(0.6));
    }

    #[test]
    fn effective_q_v10_falls_back_to_forfait() {
        // Geen meting → forfait-pad (bouwjaar bekend).
        let ctx = BuildingPressureContext::new(
            8.0,
            Some(2015),
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        assert_eq!(ctx.measured_q_v10_spec, None);
        assert_eq!(ctx.effective_q_v10(), ctx.forfait_q_v10());
        assert_eq!(ctx.effective_q_v10(), Some(0.7));
    }

    #[test]
    fn effective_q_v10_none_when_no_measurement_and_no_build_year() {
        let ctx = BuildingPressureContext::new(
            8.0,
            None,
            120.0,
            BuildingLeakageType::GroundBoundTerracedPitchedRoof,
        );
        assert_eq!(ctx.effective_q_v10(), None);
    }

    #[test]
    fn new_defaults_measured_q_v10_to_none() {
        let ctx = BuildingPressureContext::new(
            9.0,
            Some(2000),
            100.0,
            BuildingLeakageType::MultiStoreyLowerTerraced,
        );
        assert_eq!(ctx.measured_q_v10_spec, None);
    }

    #[test]
    fn leakage_type_serde_is_snake_case() {
        let json = serde_json::to_string(&BuildingLeakageType::MultiStoreyWholeBuilding).unwrap();
        assert_eq!(json, "\"multi_storey_whole_building\"");
        let back: BuildingLeakageType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, BuildingLeakageType::MultiStoreyWholeBuilding);
    }
}
