//! Heating-up allowance (opwarmtoeslag) calculation.
//! ISSO 51:2023 §2.5.8 (Formule 2.45) + §4.3 (Formule 4.15).
//!
//! De opwarmtoeslag compenseert het extra vermogen dat nodig is om een ruimte
//! na nachtverlaging weer op ontwerptemperatuur te brengen. De 2023-norm
//! berekent dit als `Φ_hu = P × A_g` (vloeroppervlak), met `P` [W/m²] uit
//! Tabel 2.10 — niet meer via het 2017-model `f_RH × ΣA_metselwerk`.
//!
//! Scope (Ronde 5): **nieuwbouw**. Afkoeling = 2 K (woning ná 2015), resp.
//! 1 K bij `Ū ≤ 0,50`. Bestaande-bouw afkoeling (Afb. 2.7-grafiek) en de
//! kamerthermostaat-y-methode (§4.3.3) zijn gemarkeerde follow-ups.

use crate::error::{Isso51Error, Result};
use crate::model::building::Building;
use crate::model::enums::{HeatingControlType, ThermalMass};
use crate::tables::heating_up as table;

/// Afkoeling [K] na 8 uur nachtverlaging voor een nieuwbouwwoning, gegeven Ū.
///
/// ISSO 51:2023 Afb. 2.6 + p.44:
/// - nieuwbouw (woning ná 2015) → **2 K**;
/// - `Ū ≤ 0,50 W/(m²·K)` → **1 K** (overschrijft; zeer goed geïsoleerd).
///
/// # Arguments
/// * `u_bar` - Oppervlakte-gewogen gemiddelde U-waarde Ū [W/(m²·K)].
pub fn newbuild_cooling_k(u_bar: f64) -> f64 {
    if u_bar <= 0.50 {
        1.0
    } else {
        2.0
    }
}

/// Bepaal de gebouwzwaarte voor Tabel 2.10 uit `building.c_eff`.
///
/// `c_eff ≤ 70 Wh/K` → `ZL+L+M` ([`ThermalMass::Light`]), anders → `Z`
/// ([`ThermalMass::Heavy`]). Bij ontbrekende `c_eff` → conservatief `Z`
/// (default van [`ThermalMass`]).
pub fn building_thermal_mass(building: &Building) -> ThermalMass {
    match building.c_eff {
        Some(c) => ThermalMass::from_c_eff(c),
        None => ThermalMass::default(),
    }
}

/// Bereken de opwarmtoeslag `Φ_hu` voor één vertrek volgens ISSO 51:2023.
///
/// Implementeert `Φ_hu,i = P × A_g` (Formule 4.15) met `P` uit Tabel 2.10,
/// inclusief de regeltype-takken uit §4.3 (nieuwbouw-scope):
/// - `PerZone` (§4.3.1) → `P × A_g`;
/// - `SelfLearning` (§4.3.2) → `0`;
/// - vloerverwarming in alle vertrekken (`all_floor_heating == true`) → `0`
///   (p.70). Wordt door de aanroeper afgeleid uit `room.heating_system`;
///   `building.all_floor_heating` is hiervoor gedeprecateerd en genegeerd;
/// - `RoomThermostat` (§4.3.3, bestaande bouw, buiten scope) → **harde error**.
///
/// De afkoeling (2 K / 1 K) en zwaarte worden door de aanroeper bepaald
/// (gebouwbreed constant) en hier alleen toegepast op `floor_area`.
///
/// # Arguments
/// * `building` - Gebouw-config (regeltype, opwarmtijd, nachtverlaging).
/// * `floor_area` - `A_g` voor dit vertrek/verblijfsgebied [m²].
/// * `cooling_k` - Afkoeling [K] (uit [`newbuild_cooling_k`]).
/// * `mass` - Gebouwzwaarte (uit [`building_thermal_mass`]).
/// * `all_floor_heating` - Of ELK vertrek vloerverwarming heeft (gebouwbreed
///   afgeleid uit `room.heating_system`). Zo ja → `Φ_hu = 0` (p.70). Vervangt
///   het gedeprecateerde `building.all_floor_heating`-vlag.
///
/// # Returns
/// `Ok((Φ_hu [W], P [W/m²]))` — `0` als de tak geen toeslag geeft.
///
/// # Errors
/// [`Isso51Error::InvalidInput`] in twee gevallen — beide bewust een expliciete
/// error i.p.v. een gegokte fallback, zodat een third-party client geen
/// stilzwijgend niet-norm-conforme waarde krijgt:
/// 1. `built_after_2015 == false`: de bestaande-bouw afkoeling (Afb. 2.7) is
///    niet geïmplementeerd; de nieuwbouw-P-waarden (2 K/1 K) zouden anders
///    onterecht worden toegepast. Wordt **niet** geraakt als Φ_hu sowieso 0 is
///    (geen nachtverlaging) — de guard staat ná de night-setback-short-circuit.
/// 2. `heating_control_type == RoomThermostat`: de y-procentmethode (§4.3.3) is
///    een bestaande-bouw-methode buiten de nieuwbouw-scope en is niet
///    geïmplementeerd.
pub fn calculate_heating_up(
    building: &Building,
    floor_area: f64,
    cooling_k: f64,
    mass: ThermalMass,
    all_floor_heating: bool,
) -> Result<(f64, f64)> {
    // Geen nachtverlaging → geen opwarmtoeslag (p.69). MOET vóór de
    // bestaande-bouw-guard: een project zonder nachtverlaging heeft Φ_hu = 0
    // ongeacht bouwjaar, dus daar hoort géén error (Vabi-fixtures hebben
    // night_setback=false en zouden anders breken).
    if !building.has_night_setback || building.warmup_time < 0.01 {
        return Ok((0.0, 0.0));
    }

    // Bestaande bouw (woning vóór 2015) valt buiten de nieuwbouw-scope: de
    // afkoeling komt dan uit de Afb. 2.7-grafiek (nog niet geïmplementeerd),
    // niet uit de vaste 2 K/1 K. Harde error i.p.v. stilzwijgend een
    // nieuwbouw-P toepassen. Zie newbuild_cooling_k + Ronde 5-scope.
    if !building.built_after_2015 {
        return Err(Isso51Error::InvalidInput(
            "Bestaande-bouw afkoeling (ISSO 51 Afb. 2.7) is niet geïmplementeerd in de \
             nieuwbouw-scope. Zet built_after_2015=true (regeling per verblijfsgebied/zelflerend) \
             of wacht op de bestaande-bouw follow-up (§4.3.3 + Afb 2.7)."
                .to_string(),
        ));
    }

    // Vloerverwarming in alle vertrekken → traag systeem, nachtverlaging niet
    // zinvol → Φ_hu = 0 (p.70). Afgeleid uit `room.heating_system` door de
    // aanroeper; `building.all_floor_heating` is hiervoor gedeprecateerd.
    if all_floor_heating {
        return Ok((0.0, 0.0));
    }

    match building.heating_control_type {
        // §4.3.1 — regeling per verblijfsgebied: Φ_hu = P × A_g (Formule 4.15).
        HeatingControlType::PerZone => {
            let p = table::specific_heating_up_allowance(cooling_k, mass, building.warmup_time);
            let phi_hu = (p * floor_area).max(0.0);
            Ok((phi_hu, p))
        }
        // §4.3.2 — zelflerende regeling: Φ_hu = 0 (p.70).
        HeatingControlType::SelfLearning => Ok((0.0, 0.0)),
        // §4.3.3 — kamerthermostaat (y-procentmethode 4.16/4.17).
        // TODO Ronde 5-vervolg: §4.3.3 kamerthermostaat y-methode (4.16/4.17,
        // bestaande bouw). De y-procentmethode verdeelt Φ_hu over de vertrekken
        // naar rato van een hoofdruimte-percentage en hoort bij de bestaande-
        // bouw afkoeling (Afb. 2.7). Geen benadering/fallback: expliciete error
        // zodat een caller die dit regeltype kiest het bewust moet oplossen.
        HeatingControlType::RoomThermostat => Err(Isso51Error::InvalidInput(
            "Kamerthermostaat-regeling (ISSO 51 §4.3.3, y-procentmethode) is niet \
             geïmplementeerd in de nieuwbouw-scope. Kies regeling per verblijfsgebied \
             (§4.3.1) of zelflerende regeling (§4.3.2)."
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::building::Building;
    use crate::model::enums::{
        AggregationMethod, BuildingType, InfiltrationMethod, SecurityClass,
    };

    /// Minimale nieuwbouw-building met nachtverlaging aan, opwarmtijd 2 h.
    fn newbuild_building() -> Building {
        Building {
            building_type: BuildingType::Terraced,
            qv10: 100.0,
            total_floor_area: 100.0,
            security_class: SecurityClass::B,
            has_night_setback: true,
            warmup_time: 2.0,
            building_height: None,
            num_floors: 1,
            infiltration_method: InfiltrationMethod::PerExteriorArea,
            dwelling_class: None,
            construction_variant: None,
            construction_year: None,
            aggregation_method: AggregationMethod::default(),
            heating_control_type: HeatingControlType::PerZone,
            c_eff: None,
            built_after_2015: true,
            all_floor_heating: false,
        }
    }

    #[test]
    fn test_per_a_g_core_2k_heavy() {
        // Nieuwbouw, afkoeling 2 K, zwaar (default), opwarmtijd 2 h.
        // P = 22 W/m² (Tabel 2.10, 2K/Z/2h). A_g = 20 m² → Φ_hu = 440 W.
        let b = newbuild_building();
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert_eq!(p, 22.0);
        assert!((phi_hu - 440.0).abs() < 1e-9, "Φ_hu = {phi_hu}");
    }

    #[test]
    fn test_per_a_g_core_2k_light() {
        // 2K/ZL+L+M/2h → P = 13. A_g = 20 → Φ_hu = 260.
        let mut b = newbuild_building();
        b.c_eff = Some(50.0); // ≤ 70 → Light
        let mass = building_thermal_mass(&b);
        assert_eq!(mass, ThermalMass::Light);
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, 2.0, mass, false).unwrap();
        assert_eq!(p, 13.0);
        assert!((phi_hu - 260.0).abs() < 1e-9, "Φ_hu = {phi_hu}");
    }

    #[test]
    fn test_u_bar_clamp_to_1k() {
        // Ū ≤ 0,5 → afkoeling 1 K (overschrijft de 2 K nieuwbouw-default).
        assert_eq!(newbuild_cooling_k(0.50), 1.0);
        assert_eq!(newbuild_cooling_k(0.30), 1.0);
        // Ū > 0,5 → 2 K.
        assert_eq!(newbuild_cooling_k(0.51), 2.0);
        assert_eq!(newbuild_cooling_k(1.20), 2.0);

        // 1K/ZL+L+M/2h → P = 7. A_g = 20 → Φ_hu = 140.
        let mut b = newbuild_building();
        b.c_eff = Some(50.0);
        let mass = building_thermal_mass(&b);
        let cooling = newbuild_cooling_k(0.40);
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, cooling, mass, false).unwrap();
        assert_eq!(p, 7.0);
        assert!((phi_hu - 140.0).abs() < 1e-9, "Φ_hu = {phi_hu}");
    }

    #[test]
    fn test_self_learning_is_zero() {
        // §4.3.2 zelflerende regeling → Φ_hu = 0 ongeacht A_g / P.
        let mut b = newbuild_building();
        b.heating_control_type = HeatingControlType::SelfLearning;
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert_eq!(phi_hu, 0.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_no_night_setback_is_zero() {
        // Geen nachtverlaging → Φ_hu = 0 (golden-fixture-pad).
        let mut b = newbuild_building();
        b.has_night_setback = false;
        let (phi_hu, _) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert_eq!(phi_hu, 0.0);
    }

    #[test]
    fn test_all_floor_heating_is_zero() {
        // Vloerverwarming in alle vertrekken (all_floor_heating=true, afgeleid
        // uit room.heating_system) → Φ_hu = 0 (p.70). Het gedeprecateerde
        // building.all_floor_heating-vlag speelt geen rol meer.
        let b = newbuild_building();
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, true).unwrap();
        assert_eq!(phi_hu, 0.0);
        assert_eq!(p, 0.0);

        // Tegenproef: zelfde building, all_floor_heating=false → Φ_hu > 0.
        let (phi_hu_nonzero, _) =
            calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert!(phi_hu_nonzero > 0.0, "Φ_hu moet > 0 zonder all_floor_heating");
    }

    #[test]
    fn test_legacy_building_flag_is_ignored() {
        // Het gedeprecateerde building.all_floor_heating-vlag mag de berekening
        // NIET meer beïnvloeden: ook met building.all_floor_heating=true blijft
        // Φ_hu > 0 zolang het afgeleide all_floor_heating-argument false is.
        let mut b = newbuild_building();
        b.all_floor_heating = true; // legacy vlag — moet genegeerd worden
        let (phi_hu, _) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert!(
            phi_hu > 0.0,
            "legacy building.all_floor_heating mag Φ_hu niet meer op 0 forceren"
        );
    }

    #[test]
    fn test_existing_building_with_setback_errors() {
        // Bestaande bouw (vóór 2015) + nachtverlaging aan → expliciete error:
        // de Afb. 2.7-afkoeling is niet geïmplementeerd, dus geen stilzwijgende
        // nieuwbouw-P. (FIX A — stille-fout-gate.)
        let mut b = newbuild_building();
        b.built_after_2015 = false;
        let err = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap_err();
        assert!(
            matches!(err, Isso51Error::InvalidInput(_)),
            "verwacht InvalidInput, kreeg {err:?}"
        );
        let msg = err.to_string();
        assert!(msg.contains("Afb. 2.7"), "error mist Afb. 2.7-context: {msg}");
        assert!(
            msg.contains("built_after_2015"),
            "error mist veld-hint built_after_2015: {msg}"
        );
    }

    #[test]
    fn test_existing_building_without_setback_is_ok_zero() {
        // Bestaande bouw ZONDER nachtverlaging → Φ_hu = 0, GÉÉN error. De
        // night-setback-short-circuit staat vóór de bestaande-bouw-guard, zodat
        // de Vabi-fixtures (night_setback=false) niet breken.
        let mut b = newbuild_building();
        b.built_after_2015 = false;
        b.has_night_setback = false;
        let (phi_hu, p) = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap();
        assert_eq!(phi_hu, 0.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_room_thermostat_errors() {
        // §4.3.3 kamerthermostaat = bestaande bouw, buiten nieuwbouw-scope →
        // expliciete error, GEEN gegokte fallback (third-party stille-fout-gate).
        let mut b = newbuild_building();
        b.heating_control_type = HeatingControlType::RoomThermostat;
        let err = calculate_heating_up(&b, 20.0, 2.0, ThermalMass::Heavy, false).unwrap_err();
        assert!(
            matches!(err, Isso51Error::InvalidInput(_)),
            "verwacht InvalidInput, kreeg {err:?}"
        );
        // Boodschap moet de §4.3.3-context + de twee in-scope alternatieven noemen.
        let msg = err.to_string();
        assert!(msg.contains("§4.3.3"), "error mist §4.3.3-context: {msg}");
        assert!(msg.contains("§4.3.1"), "error mist §4.3.1-alternatief: {msg}");
        assert!(msg.contains("§4.3.2"), "error mist §4.3.2-alternatief: {msg}");
    }
}
