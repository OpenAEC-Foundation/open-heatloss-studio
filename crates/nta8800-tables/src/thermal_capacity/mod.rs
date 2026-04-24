//! NTA 8800:2025+C1:2026 §7.7 — Effectieve interne warmtecapaciteit.
//!
//! Levert `D_m;int;eff;zi` (specifieke interne warmtecapaciteit) in
//! kJ/(m²·K) per combinatie van vloer-, wand- en plafondklasse conform
//! tabel 7.10. Formule 7.45 berekent `C_m;int;eff;zi` (effectieve interne
//! warmtecapaciteit van de rekenzone) in J/K uit `D_m × 1000 × A_g;zi`.
//!
//! De bouwwijze-classificaties voor vloeren (tabel 7.11) en wanden
//! (tabel 7.12) zijn als enum-varianten beschikbaar. De kolomsplitsing
//! voor het plafondtype uit tabel 7.10 (gesloten/verlaagd versus geen/open)
//! is gemodelleerd als aparte enum.
//!
//! # Voetnoten bij tabel 7.10
//!
//! - Voetnoot a: Bij utiliteitsbouw geldt default de kolom ‘gesloten of
//!   verlaagd plafond’, tenzij het verblijfsgebied een vrijhangend plafond
//!   heeft dat netto ≥ 15 % open is uitgevoerd, én voetnoot c niet van
//!   toepassing is.
//! - Voetnoot b: Bij woningbouw geldt default de kolom ‘geen of open
//!   plafond’, behalve in situaties waarin voetnoot c van toepassing is.
//! - Voetnoot c: Bij woningbouw — indien de bovenzijde van een vloer in een
//!   zwaardere categorie valt dan de onderzijde van de vloer erboven, moet
//!   de kolom ‘gesloten of verlaagd plafond’ worden gebruikt.
//!
//! De keuze voor de juiste kolom op basis van voetnoten a/b/c is bouwtype-
//! en projectspecifiek; deze module levert uitsluitend de lookup. De keuze
//! zelf hoort in een hogere laag (projectconfiguratie of rekenzone-logica).
//!
//! # Alternatieve detailberekening
//!
//! §7.7 staat als afwijking ook een berekening volgens bijlage B toe
//! (ρ·c·d·A per laag). Deze module implementeert die methode niet —
//! toekomstige uitbreiding.
//!
//! # Referenties
//!
//! - [`NTA_8800_2025_PARAG7_7`](crate::references::NTA_8800_2025_PARAG7_7)
//! - [`NTA_8800_2025_TABEL7_10`](crate::references::NTA_8800_2025_TABEL7_10)
//! - [`NTA_8800_2025_TABEL7_11`](crate::references::NTA_8800_2025_TABEL7_11)
//! - [`NTA_8800_2025_TABEL7_12`](crate::references::NTA_8800_2025_TABEL7_12)
//! - [`NTA_8800_2025_FORMULE7_45`](crate::references::NTA_8800_2025_FORMULE7_45)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Vloer-massaklasse conform NTA 8800:2025+C1:2026 tabel 7.11.
///
/// Bepaalt de thermische-massabijdrage van de vloerconstructie aan de
/// effectieve interne warmtecapaciteit van de rekenzone.
///
/// | Variant | Typische constructie |
/// |---|---|
/// | [`Self::Light`] | Houten vloeren; HSB-/SFB-vloeren; schuimbetonvloer; elke vloer die aan de **bovenzijde** is geïsoleerd |
/// | [`Self::Heavy`] | Staal-betonvloer; niet-massieve betonnen vloeren (kanaalplaat, cassette); hout-betonvloer; een lichte vloer afgewerkt met een cement-/anhydriet-dekvloer |
/// | [`Self::VeryHeavy`] | Massieve betonnen vloeren |
///
/// OPMERKING 3 van de norm (bij tabel 7.11): uitgangspunt is een dekvloer
/// met minimale dikte 60 mm; in de bestaande bouw is die dikte niet altijd
/// op te nemen, dus geldt geen harde minimumeis.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FloorMassClass {
    /// Licht — houten/HSB/SFB-vloeren, schuimbeton of elke bovenzijdig
    /// geïsoleerde vloer.
    Light,
    /// Zwaar — staal-beton, niet-massieve betonvloeren, hout-beton of een
    /// lichte vloer met cement-/anhydriet-dekvloer.
    Heavy,
    /// Zeer zwaar — massieve betonnen vloeren.
    VeryHeavy,
}

/// Wand-massaklasse conform NTA 8800:2025+C1:2026 tabel 7.12.
///
/// Bepaalt de thermische-massabijdrage van de wandconstructie aan de
/// effectieve interne warmtecapaciteit van de rekenzone.
///
/// | Variant | Typische constructie |
/// |---|---|
/// | [`Self::Light`] | HSB; SFB; staalskeletbouw; elke wand die aan de **binnenzijde** is geïsoleerd |
/// | [`Self::Heavy`] | Dragend metselwerk; betonnen kolom-ligger skeletbouw |
/// | [`Self::VeryHeavy`] | Betonnen wand-vloer skeletbouw |
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WallMassClass {
    /// Licht — HSB/SFB/staalskelet of elke binnenzijdig geïsoleerde wand.
    Light,
    /// Zwaar — dragend metselwerk of betonnen kolom-ligger skeletbouw.
    Heavy,
    /// Zeer zwaar — betonnen wand-vloer skeletbouw.
    VeryHeavy,
}

/// Plafondtype voor de kolomkeuze in tabel 7.10.
///
/// Een gesloten of verlaagd plafond dempt de thermische-massabijdrage van
/// de vloerconstructie erboven (die is via het plafond grotendeels
/// afgeschermd van de luchtmassa in het vertrek). Een geen/open plafond
/// geeft volledige zichtbaarheid van de vloer-onderzijde en daarmee een
/// hogere effectieve interne warmtecapaciteit.
///
/// De keuze tussen de varianten volgt uit voetnoten a/b/c van tabel 7.10
/// (zie module-doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CeilingType {
    /// Gesloten of verlaagd plafond — linkerkolom van tabel 7.10.
    ///
    /// Default bij utiliteitsbouw (voetnoot a), óf bij woningbouw wanneer
    /// voetnoot c van toepassing is.
    ClosedOrSuspended,
    /// Geen of open plafond — rechterkolom van tabel 7.10.
    ///
    /// Default bij woningbouw (voetnoot b), tenzij voetnoot c van
    /// toepassing is. Ook bij utiliteitsbouw wanneer een vrijhangend
    /// plafond ≥ 15 % netto open is uitgevoerd in het verblijfsgebied.
    OpenOrNone,
}

/// Specifieke effectieve interne warmtecapaciteit `D_m;int;eff;zi` in
/// kJ/(m²·K) conform NTA 8800:2025+C1:2026 tabel 7.10.
///
/// De tabel geeft 9 unieke floor×wall-combinaties, verdeeld over 4
/// waarde-groepen per plafondtype:
///
/// | Groep | Vloer | Wand | Gesloten/verlaagd | Geen/open |
/// |---|---|---|---:|---:|
/// | 1 | Licht | Licht | 55 | 80 |
/// | 2 | Licht / Zwaar / Zeer zwaar | Zwaar/Licht/Licht | 110 | 180 |
/// | 3 | Zwaar / Licht | Zwaar / Zeer zwaar | 180 | 360 |
/// | 4 | Zwaar / Zeer zwaar / Zeer zwaar | Zeer zwaar / Zwaar / Zeer zwaar | 250 | 450 |
///
/// # Parameters
///
/// - `floor`: vloer-massaklasse (tabel 7.11)
/// - `wall`: wand-massaklasse (tabel 7.12)
/// - `ceiling`: plafondtype (kolomkeuze tabel 7.10)
///
/// # Retour
///
/// `D_m;int;eff;zi` in kJ/(m²·K).
///
/// # Voorbeelden
///
/// ```
/// use nta8800_tables::thermal_capacity::{
///     specific_heat_capacity, CeilingType, FloorMassClass, WallMassClass,
/// };
///
/// // Houten vloer + HSB-wand + gesloten plafond → lichtste combinatie.
/// let d_m = specific_heat_capacity(
///     FloorMassClass::Light,
///     WallMassClass::Light,
///     CeilingType::ClosedOrSuspended,
/// );
/// assert!((d_m - 55.0).abs() < 1e-9);
///
/// // Massieve betonvloer + betonnen wand-vloer skelet + open plafond
/// // → zwaarste combinatie.
/// let d_m = specific_heat_capacity(
///     FloorMassClass::VeryHeavy,
///     WallMassClass::VeryHeavy,
///     CeilingType::OpenOrNone,
/// );
/// assert!((d_m - 450.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn specific_heat_capacity(
    floor: FloorMassClass,
    wall: WallMassClass,
    ceiling: CeilingType,
) -> f64 {
    use CeilingType::{ClosedOrSuspended, OpenOrNone};
    use FloorMassClass as F;
    use WallMassClass as W;

    // Eerst de floor×wall-combinatie afbeelden op de 4 waarde-groepen van
    // tabel 7.10; vervolgens kolomkeuze op plafondtype.
    let (closed, open) = match (floor, wall) {
        // Groep 1 — 55 / 80
        (F::Light, W::Light) => (55.0_f64, 80.0_f64),

        // Groep 2 — 110 / 180
        (F::Light, W::Heavy) | (F::Heavy | F::VeryHeavy, W::Light) => (110.0, 180.0),

        // Groep 3 — 180 / 360
        (F::Heavy, W::Heavy) | (F::Light, W::VeryHeavy) => (180.0, 360.0),

        // Groep 4 — 250 / 450
        (F::Heavy | F::VeryHeavy, W::VeryHeavy) | (F::VeryHeavy, W::Heavy) => (250.0, 450.0),
    };

    match ceiling {
        ClosedOrSuspended => closed,
        OpenOrNone => open,
    }
}

/// Effectieve interne warmtecapaciteit `C_m;int;eff;zi` voor een rekenzone,
/// in J/K, conform NTA 8800:2025+C1:2026 formule 7.45.
///
/// ```text
/// C_m;int;eff;zi = D_m;int;eff;zi × 1000 × A_g;zi
/// ```
///
/// De factor 1000 converteert `D_m` van kJ/(m²·K) naar J/(m²·K).
///
/// # Parameters
///
/// - `floor`, `wall`, `ceiling`: classificaties per tabel 7.11, 7.12 en
///   kolomkeuze 7.10
/// - `floor_area_m2`: `A_g;zi` — gebruiksoppervlakte van de rekenzone in m²
///   (bepaald volgens §6.6.4)
///
/// # Retour
///
/// `C_m;int;eff;zi` in J/K.
///
/// # Voorbeelden
///
/// ```
/// use nta8800_tables::thermal_capacity::{
///     zone_heat_capacity, CeilingType, FloorMassClass, WallMassClass,
/// };
///
/// // 100 m² lichte woning → 55 × 1000 × 100 = 5,5 MJ/K
/// let c_m = zone_heat_capacity(
///     FloorMassClass::Light,
///     WallMassClass::Light,
///     CeilingType::ClosedOrSuspended,
///     100.0,
/// );
/// assert!((c_m - 5_500_000.0).abs() < 1e-6);
/// ```
#[must_use]
pub fn zone_heat_capacity(
    floor: FloorMassClass,
    wall: WallMassClass,
    ceiling: CeilingType,
    floor_area_m2: f64,
) -> f64 {
    specific_heat_capacity(floor, wall, ceiling) * 1000.0 * floor_area_m2
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // Lookup-waarden uit tabel 7.10 — 18 cellen
    // -------------------------------------------------------------------

    // Groep 1 — Licht/Licht — 55 / 80
    #[test]
    fn tabel_7_10_light_light_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Light,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 55.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_light_light_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Light,
            CeilingType::OpenOrNone,
        );
        assert!((d - 80.0).abs() < 1e-9);
    }

    // Groep 2 — 110 / 180
    #[test]
    fn tabel_7_10_light_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Heavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 110.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_light_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Heavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 180.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_heavy_light_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::Light,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 110.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_heavy_light_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::Light,
            CeilingType::OpenOrNone,
        );
        assert!((d - 180.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_light_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::Light,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 110.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_light_open() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::Light,
            CeilingType::OpenOrNone,
        );
        assert!((d - 180.0).abs() < 1e-9);
    }

    // Groep 3 — 180 / 360
    #[test]
    fn tabel_7_10_heavy_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::Heavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 180.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_heavy_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::Heavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 360.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_light_very_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::VeryHeavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 180.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_light_very_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::VeryHeavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 360.0).abs() < 1e-9);
    }

    // Groep 4 — 250 / 450
    #[test]
    fn tabel_7_10_heavy_very_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::VeryHeavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 250.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_heavy_very_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::VeryHeavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 450.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::Heavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 250.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::Heavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 450.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_very_heavy_closed() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::VeryHeavy,
            CeilingType::ClosedOrSuspended,
        );
        assert!((d - 250.0).abs() < 1e-9);
    }

    #[test]
    fn tabel_7_10_very_heavy_very_heavy_open() {
        let d = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::VeryHeavy,
            CeilingType::OpenOrNone,
        );
        assert!((d - 450.0).abs() < 1e-9);
    }

    // -------------------------------------------------------------------
    // Ordening-properties
    // -------------------------------------------------------------------

    #[test]
    fn zeer_zware_combinatie_gt_lichte_combinatie() {
        let zwaar = specific_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::VeryHeavy,
            CeilingType::ClosedOrSuspended,
        );
        let licht = specific_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Light,
            CeilingType::ClosedOrSuspended,
        );
        assert!(zwaar > licht, "{zwaar} !> {licht}");
    }

    #[test]
    fn open_plafond_ge_gesloten_plafond_voor_alle_combinaties() {
        // Voor elke (floor, wall)-combinatie: open plafond geeft een
        // hogere (of in niet-occurerende gevallen: gelijke) D_m dan een
        // gesloten/verlaagd plafond, omdat open/geen plafond de
        // vloer-onderzijde volledig zichtbaar laat.
        let floors = [
            FloorMassClass::Light,
            FloorMassClass::Heavy,
            FloorMassClass::VeryHeavy,
        ];
        let walls = [
            WallMassClass::Light,
            WallMassClass::Heavy,
            WallMassClass::VeryHeavy,
        ];
        for &f in &floors {
            for &w in &walls {
                let open = specific_heat_capacity(f, w, CeilingType::OpenOrNone);
                let closed = specific_heat_capacity(f, w, CeilingType::ClosedOrSuspended);
                assert!(
                    open >= closed,
                    "floor={f:?} wall={w:?}: open ({open}) < closed ({closed})"
                );
            }
        }
    }

    // -------------------------------------------------------------------
    // Formule 7.45
    // -------------------------------------------------------------------

    #[test]
    fn formule_7_45_light_woning_100m2() {
        // D_m = 55 kJ/(m²·K), A_g = 100 m² → C_m = 55 × 1000 × 100 = 5,5 MJ/K
        let c_m = zone_heat_capacity(
            FloorMassClass::Light,
            WallMassClass::Light,
            CeilingType::ClosedOrSuspended,
            100.0,
        );
        assert!((c_m - 5_500_000.0).abs() < 1e-6);
    }

    #[test]
    fn formule_7_45_zeer_zwaar_utiliteit_250m2() {
        // D_m = 450 kJ/(m²·K), A_g = 250 m² → C_m = 450 × 1000 × 250 = 112,5 MJ/K
        let c_m = zone_heat_capacity(
            FloorMassClass::VeryHeavy,
            WallMassClass::VeryHeavy,
            CeilingType::OpenOrNone,
            250.0,
        );
        assert!((c_m - 112_500_000.0).abs() < 1e-3);
    }

    #[test]
    fn formule_7_45_nul_oppervlak_is_nul_capaciteit() {
        let c_m = zone_heat_capacity(
            FloorMassClass::Heavy,
            WallMassClass::Heavy,
            CeilingType::OpenOrNone,
            0.0,
        );
        assert!(c_m.abs() < 1e-9);
    }

    // -------------------------------------------------------------------
    // Serde round-trip
    // -------------------------------------------------------------------

    #[test]
    fn floor_mass_class_serde_round_trip() {
        for variant in [
            FloorMassClass::Light,
            FloorMassClass::Heavy,
            FloorMassClass::VeryHeavy,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let back: FloorMassClass = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, back, "round-trip mislukt voor {variant:?}: {json}");
        }
    }

    #[test]
    fn wall_mass_class_serde_round_trip() {
        for variant in [
            WallMassClass::Light,
            WallMassClass::Heavy,
            WallMassClass::VeryHeavy,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let back: WallMassClass = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, back, "round-trip mislukt voor {variant:?}: {json}");
        }
    }

    #[test]
    fn ceiling_type_serde_round_trip() {
        for variant in [CeilingType::ClosedOrSuspended, CeilingType::OpenOrNone] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let back: CeilingType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, back, "round-trip mislukt voor {variant:?}: {json}");
        }
    }

    #[test]
    fn serde_gebruikt_snake_case() {
        // Contract: externe JSON-representatie is snake_case zodat
        // consumenten (UI, schemas) een stabiele string hebben.
        assert_eq!(
            serde_json::to_string(&FloorMassClass::VeryHeavy).unwrap(),
            "\"very_heavy\""
        );
        assert_eq!(
            serde_json::to_string(&WallMassClass::VeryHeavy).unwrap(),
            "\"very_heavy\""
        );
        assert_eq!(
            serde_json::to_string(&CeilingType::ClosedOrSuspended).unwrap(),
            "\"closed_or_suspended\""
        );
        assert_eq!(
            serde_json::to_string(&CeilingType::OpenOrNone).unwrap(),
            "\"open_or_none\""
        );
    }
}
