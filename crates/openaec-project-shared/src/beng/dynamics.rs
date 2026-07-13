//! Thermische dynamica van een BENG-rekenzone: effectieve interne
//! warmtecapaciteit (C3a) en interne warmtewinst woningbouw (C3b).
//!
//! Deze module leidt de twee aan de verwarming/koeling **gekoppelde** invoerposten
//! af uit een [`BengZone`], zodat de gevel-georiënteerde BENG-keten ze via
//! [`crate::TojuliFullInputs`] aan `compute_tojuli_full` kan doorgeven i.p.v. de
//! hardcoded defaults (`ThermalMassInput::light_woning()` / `InternalGains::forfaitair`).
//! Zie `docs/2026-07-13-c3-norm-analyse-massa-interne-winst.md` voor de
//! norm-verificatie.
//!
//! Beide afleidingen zijn **additief**: ze worden alleen in de bridged BENG-tak
//! aangeroepen (aanwezig `beng_geometry`-blok). Zonder dat blok blijven de
//! defaults staan (byte-identiek gedrag).

use nta8800_demand::model::{InternalGains, ThermalMassInput};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::UsageFunction;
use nta8800_tables::thermal_capacity::{CeilingType, FloorMassClass, WallMassClass};

use crate::beng_geometry::BengZone;

// ---------------------------------------------------------------------------
// C3a — bouwwijze → C_m (NTA 8800 §7.7, tabel 7.10/7.11/7.12)
// ---------------------------------------------------------------------------

/// Leid de vloer-massaklasse (tabel 7.11) af uit een Uniec `RZ_BOUWW_VL`-code.
///
/// Codes + labels uit de golden-capture (`uniec_fields_capture.json`, veld
/// `RZ_BOUWW_VL`; ruwe codes in de walk-dump `fields.json`). `CONSTRM_FL_21`
/// (zwaar) en `CONSTRM_FL_26` (zeer zwaar) zijn capture-bevestigd (Gouda resp.
/// Aalten); de overige suffixen volgen de +5-nummering van de optievolgorde. De
/// **klasse** is in alle gevallen eenduidig uit het norm-label. `eigen waarde`
/// (bijlage B) en onbekende codes → `None` (terugval op de default).
fn floor_mass_class(code: &str) -> Option<FloorMassClass> {
    match code {
        // hsb/sfb/schuimbeton/hout (licht) resp. geïsoleerd aan binnenzijde (licht)
        "CONSTRM_FL_11" | "CONSTRM_FL_16" => Some(FloorMassClass::Light),
        // staal-beton, hout-beton of niet-massief beton (zwaar) — confirmed (Gouda)
        "CONSTRM_FL_21" => Some(FloorMassClass::Heavy),
        // massief beton (zeer zwaar) — confirmed (Aalten)
        "CONSTRM_FL_26" => Some(FloorMassClass::VeryHeavy),
        _ => None,
    }
}

/// Leid de wand-massaklasse (tabel 7.12) af uit een Uniec `RZ_BOUWW_W`-code.
///
/// `CONSTRM_W_11` (licht) is capture-bevestigd (beide fixtures); de overige
/// suffixen volgen de +5-nummering van de optievolgorde, de klasse uit het
/// norm-label. `eigen waarde` (bijlage B) en onbekende codes → `None`.
fn wall_mass_class(code: &str) -> Option<WallMassClass> {
    match code {
        // hsb/sfb/staalskeletbouw (licht) resp. geïsoleerd aan binnenzijde (licht)
        "CONSTRM_W_11" | "CONSTRM_W_16" => Some(WallMassClass::Light),
        // dragend metselwerk (zwaar) resp. betonnen kolom-ligger skeletbouw (zwaar)
        "CONSTRM_W_21" | "CONSTRM_W_26" => Some(WallMassClass::Heavy),
        // betonnen wand-vloer skeletbouw (zeer zwaar)
        "CONSTRM_W_31" => Some(WallMassClass::VeryHeavy),
        _ => None,
    }
}

/// Plafondkolom-keuze voor tabel 7.10 (voetnoot a/b, p. 204).
///
/// - Woningbouw → *"geen of open plafond"* (voetnoot b).
/// - Utiliteitsbouw → *"gesloten of verlaagd plafond"* (voetnoot a).
///
/// Voetnoot c (woningbouw, bovenzijde vloer zwaarder dan onderzijde vloer erboven)
/// wordt niet geëvalueerd: het BENG-DTO codeert geen per-verdieping-vloerconstructie.
fn ceiling_type(usage: UsageFunction) -> CeilingType {
    match usage {
        UsageFunction::Woonfunctie => CeilingType::OpenOrNone,
        _ => CeilingType::ClosedOrSuspended,
    }
}

/// Leid de [`ThermalMassInput`] (C_m via tabel 7.10) af uit de bouwwijze-codes van
/// een [`BengZone`]. `None` als een van beide codes ontbreekt of niet herkend
/// wordt (dan valt de keten terug op `ThermalMassInput::light_woning()`).
#[must_use]
pub fn derive_thermal_mass(zone: &BengZone, usage: UsageFunction) -> Option<ThermalMassInput> {
    let floor = floor_mass_class(zone.bouwwijze_vloer.as_deref()?)?;
    let wall = wall_mass_class(zone.bouwwijze_wand.as_deref()?)?;
    Some(ThermalMassInput::new(floor, wall, ceiling_type(usage)))
}

// ---------------------------------------------------------------------------
// C3b — interne warmtewinst woningbouw (NTA 8800 §7.5.2.1, formule 7.21)
// ---------------------------------------------------------------------------

/// Gemiddeld aantal bewoners per woonfunctie `N_P;woon;zi` volgens de gemiddelde
/// gebruiksoppervlakte per woning `x = A_g/N_woon` (NTA 8800 formules 7.22–7.24,
/// p. 176-177):
///
/// - `x ≤ 30 m²` → 1                                    (7.22)
/// - `30 < x ≤ 100 m²` → 2,28 − 1,28/70 · (100 − x)     (7.23)
/// - `x > 100 m²` → 1,28 + 0,01 · x                     (7.24)
#[must_use]
pub fn n_p_woon(area_per_dwelling_m2: f64) -> f64 {
    let x = area_per_dwelling_m2;
    if x <= 30.0 {
        1.0
    } else if x <= 100.0 {
        2.28 - (1.28 / 70.0) * (100.0 - x)
    } else {
        1.28 + 0.01 * x
    }
}

/// Leid de interne warmtewinst voor woningbouw af uit A_g en het aantal
/// woonfuncties (NTA 8800 §7.5.2.1, formule 7.21).
///
/// Formule 7.21 geeft `Q_int;mi = 180 · N_woon · N_P · 0,001 · t_mi` [kWh]. De
/// demand-crate rekent `Q_int;mi = Φ_int · A_g · t_mi · 0,0036` [MJ]; gelijkstellen
/// (kWh → MJ ×3,6) levert een **constante** flux
/// `Φ_int = 180 · N_woon · N_P / A_g` [W/m²] die formule 7.21 exact reproduceert
/// door de gevalideerde maandbalans (de maandlengte `t_mi` valt weg). Dezelfde
/// winst voedt de verwarmings- én de koudebalans (formule 7.21 is `Q_H/C;int`).
///
/// `n_woon` = aantal woonfuncties in de rekenzone (§6.6.7); voor een
/// grondgebonden woning = 1.
///
/// # Panics
///
/// Nooit in de praktijk: de flux is per constructie eindig en ≥ 0 (`a_g_m2 > 0` is
/// door [`BengZone`]-validatie gegarandeerd), dus [`InternalGains::new`] slaagt.
#[must_use]
pub fn derive_internal_gains_woningbouw(a_g_m2: f64, n_woon: f64) -> InternalGains {
    let n_p = n_p_woon(a_g_m2 / n_woon);
    let flux_w_per_m2 = 180.0 * n_woon * n_p / a_g_m2;
    InternalGains::new(MonthlyProfile::from_constant(flux_w_per_m2))
        .expect("Φ_int = 180·N_woon·N_P/A_g is eindig en ≥ 0")
}

#[cfg(test)]
mod tests {
    use super::*;
    use nta8800_model::time::Month;
    use nta8800_tables::thermal_capacity::specific_heat_capacity;

    fn zone_with(floor: Option<&str>, wall: Option<&str>) -> BengZone {
        BengZone {
            id: "rz".into(),
            naam: "woning".into(),
            a_g_m2: 67.0,
            bouwwijze_vloer: floor.map(String::from),
            bouwwijze_wand: wall.map(String::from),
            woningtype: None,
            gevels: Vec::new(),
        }
    }

    #[test]
    fn aalten_codes_geven_d_m_180() {
        // CONSTRM_FL_26 (zeer zwaar) + CONSTRM_W_11 (licht) + woning (open) → 180.
        let m = derive_thermal_mass(
            &zone_with(Some("CONSTRM_FL_26"), Some("CONSTRM_W_11")),
            UsageFunction::Woonfunctie,
        )
        .expect("beide codes herkend");
        assert_eq!(m.floor, FloorMassClass::VeryHeavy);
        assert_eq!(m.wall, WallMassClass::Light);
        assert_eq!(m.ceiling, CeilingType::OpenOrNone);
        assert!((specific_heat_capacity(m.floor, m.wall, m.ceiling) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn gouda_codes_geven_d_m_180() {
        // CONSTRM_FL_21 (zwaar) + CONSTRM_W_11 (licht) + woning (open) → 180.
        let m = derive_thermal_mass(
            &zone_with(Some("CONSTRM_FL_21"), Some("CONSTRM_W_11")),
            UsageFunction::Woonfunctie,
        )
        .expect("beide codes herkend");
        assert_eq!(m.floor, FloorMassClass::Heavy);
        assert_eq!(m.wall, WallMassClass::Light);
        assert!((specific_heat_capacity(m.floor, m.wall, m.ceiling) - 180.0).abs() < 1e-9);
    }

    #[test]
    fn utiliteit_gebruikt_gesloten_plafond() {
        let m = derive_thermal_mass(
            &zone_with(Some("CONSTRM_FL_26"), Some("CONSTRM_W_11")),
            UsageFunction::Kantoorfunctie,
        )
        .expect("beide codes herkend");
        assert_eq!(m.ceiling, CeilingType::ClosedOrSuspended);
        // VeryHeavy/Light/gesloten → groep 2 gesloten = 110.
        assert!((specific_heat_capacity(m.floor, m.wall, m.ceiling) - 110.0).abs() < 1e-9);
    }

    #[test]
    fn ontbrekende_of_onbekende_code_geeft_none() {
        assert!(derive_thermal_mass(&zone_with(None, Some("CONSTRM_W_11")), UsageFunction::Woonfunctie).is_none());
        assert!(derive_thermal_mass(&zone_with(Some("CONSTRM_FL_26"), None), UsageFunction::Woonfunctie).is_none());
        // `eigen waarde` (bijlage B) → None.
        assert!(derive_thermal_mass(&zone_with(Some("CONSTRM_FL_31"), Some("CONSTRM_W_11")), UsageFunction::Woonfunctie).is_none());
    }

    #[test]
    fn n_p_woon_bandgrenzen() {
        // ≤30 → 1
        assert!((n_p_woon(30.0) - 1.0).abs() < 1e-12);
        // Aalten 67 (band 7.23): 2,28 − 1,28/70·(100−67) = 1,6766
        assert!((n_p_woon(67.0) - (2.28 - (1.28 / 70.0) * 33.0)).abs() < 1e-12);
        assert!((n_p_woon(67.0) - 1.676_571_4).abs() < 1e-6);
        // grens 100 (band 7.23): 2,28 − 0 = 2,28
        assert!((n_p_woon(100.0) - 2.28).abs() < 1e-12);
        // Gouda 133,06 (band 7.24): 1,28 + 0,01·133,06 = 2,6106
        assert!((n_p_woon(133.06) - (1.28 + 0.01 * 133.06)).abs() < 1e-12);
    }

    #[test]
    fn aalten_internal_gains_flux() {
        // Φ_int = 180·1·1,6766/67 = 4,504 W/m², constant over de maanden.
        let g = derive_internal_gains_woningbouw(67.0, 1.0);
        let expected = 180.0 * n_p_woon(67.0) / 67.0;
        assert!((g.heat_flux_per_m2[Month::Januari] - expected).abs() < 1e-9);
        assert!((g.heat_flux_per_m2[Month::Juli] - expected).abs() < 1e-9);
        assert!((expected - 4.504).abs() < 1e-2);
    }

    #[test]
    fn gouda_internal_gains_flux() {
        // Φ_int = 180·1·2,6106/133,06 = 3,531 W/m².
        let g = derive_internal_gains_woningbouw(133.06, 1.0);
        let expected = 180.0 * n_p_woon(133.06) / 133.06;
        assert!((g.heat_flux_per_m2[Month::Juli] - expected).abs() < 1e-9);
        assert!((expected - 3.531).abs() < 1e-2);
    }
}
