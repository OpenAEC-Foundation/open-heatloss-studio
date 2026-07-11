//! Ventilation heat loss for ISSO 53 (§4.7.2).

use crate::error::Result;
use crate::formulas::RHO_CP_AIR;
use crate::model::{Room, VentilationConfig};
use crate::model::enums::{HeatingSystem, VentilatieBouwfase};
use crate::tables::ventilation_requirements::{requirement, ventilation_rate_per_person};
use crate::tables::temperature::resolve_theta_i;
use crate::tables::temperature_stratification::delta_theta_v;
use crate::calc::rc_high::room_rc_high;

/// Fallback-bezettingsdichtheid in personen/m² wanneer noch de ruimte-bezetting
/// (`personen_per_m2_default`) noch de tabel 4.10-eis (`req.personen_per_m2`)
/// een waarde levert. Komt overeen met de laagste tabel 4.11-richtwaarde
/// (kantoor/vergaderen, ISSO 53 PDF p.51). Bewust benoemd i.p.v. magic getal;
/// in de praktijk zelden bereikt omdat functies zonder richtwaarde via de
/// `None`-gate al q_v = 0 opleveren.
const DEFAULT_OCCUPANCY_DENSITY_PERS_M2: f64 = 0.05;

/// Fallback-ventilatiedebiet in dm³/s per persoon wanneer de tabel 4.10-regel
/// geen bouwfase-specifieke waarde geeft. Komt overeen met de meest
/// voorkomende tabel 4.10-eis voor verblijfs-/kantoor-/vergaderruimten
/// (6,5 dm³/s·pp, ISSO 53 PDF p.48-50). Benoemd i.p.v. magic getal.
const DEFAULT_VENTILATION_RATE_DM3_S_PP: f64 = 6.5;

/// Results from ventilation calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct VentilationResult {
    /// Total ventilation heat loss Φ_vent in W.
    pub phi_vent: f64,
    /// Ventilation heat loss coefficient H_v in W/K.
    pub h_v: f64,
    /// Ventilation flow rate q_v in m³/s.
    pub q_v: f64,
    /// Temperature reduction factor f_v (dimensionless).
    pub f_v: f64,
}

/// Calculate ventilation heat loss for a room.
/// ISSO 53 formules 4.35-4.39, PDF p.47-50.
///
/// `heating_system` bepaalt — samen met de R_c-kolomkeuze uit de uitwendige
/// scheidingsconstructies van de ruimte — de gelaagdheidscorrectie Δθ_v
/// (tabel 2.3) die in de f_v-berekening (form. 4.39) ingaat.
pub fn calculate_ventilation(
    room: &Room,
    config: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
    heating_system: HeatingSystem,
) -> Result<VentilationResult> {
    // Calculate ventilation flow rate q_v in m³/s
    let q_v = calculate_ventilation_flow_rate(room, config.bouwfase)?;

    // Δθ_v-kolomkeuze: oppervlakte-gewogen R_c van de uitwendige
    // scheidingsconstructies (tabel 2.3 voetnoot 4).
    let rc_high = room_rc_high(room);
    let delta_theta_v_value = delta_theta_v(heating_system, rc_high);

    // Calculate temperature reduction factor f_v
    let f_v = calculate_f_v(config, theta_i, theta_e, delta_theta_v_value)?;

    // Calculate H_v: formule 4.37 - H_v = q_v × 1200 × f_v
    let h_v = q_v * RHO_CP_AIR * f_v;

    // Calculate Φ_vent: formule 4.35 - Φ_vent = H_v × (θ_i - θ_e)
    let phi_vent = h_v * (theta_i - theta_e);

    Ok(VentilationResult {
        phi_vent,
        h_v,
        q_v,
        f_v,
    })
}

/// Calculate the specific ventilation heat loss H_v for a room.
/// ISSO 53 formule 4.37, PDF p.47: H_v = q_v × 1200 × f_v
pub fn calculate_h_v(
    room: &Room,
    ventilation: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
    heating_system: HeatingSystem,
) -> Result<f64> {
    let result = calculate_ventilation(room, ventilation, theta_i, theta_e, heating_system)?;
    Ok(result.h_v)
}

/// Calculate the ventilation heat loss Φ_vent for a room.
/// ISSO 53 formule 4.35, PDF p.47: Φ_vent = H_v × (θ_i - θ_e)
pub fn calculate_phi_vent(
    room: &Room,
    ventilation: &VentilationConfig,
    theta_e: f64,
    heating_system: HeatingSystem,
) -> Result<f64> {
    let theta_i = resolve_theta_i(room, theta_e);

    let result = calculate_ventilation(room, ventilation, theta_i, theta_e, heating_system)?;
    Ok(result.phi_vent)
}

/// Calculate ventilation flow rate q_v in m³/s based on room occupancy and requirements.
///
/// `bouwfase` (tabel 4.10) bepaalt het ventilatiedebiet per persoon:
/// `Nieuwbouw` (strenger) vs `Bestaand` (soepeler). Vóór D2 stond dit
/// hardcoded op `Nieuwbouw`, wat bestaande-bouw-projecten ~+89% Φ_V gaf.
fn calculate_ventilation_flow_rate(room: &Room, bouwfase: VentilatieBouwfase) -> Result<f64> {
    // Fase 3 (uitvoering): een vastgestelde toevoer-q_v overrulet alles.
    // Wordt direct als q_v gebruikt; negeert de has_mechanical_supply-gate,
    // de requirement-lookup én de personen-/bezetting-afleiding.
    // Negatieve waarden defensief clampen op 0.
    if let Some(v) = room.ventilation_q_v_established {
        return Ok(v.max(0.0));
    }

    // In ISSO 53 telt alleen de toevoer van verse buitenlucht mee voor het
    // ventilatiewarmteverlies. Een ruimte zonder mechanische toevoer
    // (`Some(false)`) levert dus q_v = 0. `None` (oudere fixtures zonder veld)
    // → geen gate, bestaande berekening ongewijzigd.
    if room.has_mechanical_supply == Some(false) {
        return Ok(0.0);
    }

    // Get ventilation requirement for this room type.
    // Ruimtetypen zonder personen-gebaseerde eis in tabel 4.10
    // (berg-/technische/verkeers-/sanitaire ruimten) leveren `None` →
    // geen ventilatie-eis, dus q_v = 0 (geen crash van de berekening).
    let req = match requirement(room.gebruiks_functie, room.ruimte_type) {
        Some(req) => req,
        None => return Ok(0.0),
    };

    // Calculate number of people: maximum of (area-based occupancy, explicit input).
    // An explicit value raises the count above the area-based default but never lowers it.
    let density = room.bezetting.personen_per_m2_default
        .or(req.personen_per_m2)
        .unwrap_or(DEFAULT_OCCUPANCY_DENSITY_PERS_M2);
    let area_based = room.floor_area * density;
    let people = match room.bezetting.personen {
        Some(explicit) => explicit.max(area_based),
        None => area_based,
    };

    // Get ventilation rate per person in dm³/s (bouwfase-afhankelijk, D2).
    let dm3_s_per_person = ventilation_rate_per_person(req, bouwfase)
        .unwrap_or(DEFAULT_VENTILATION_RATE_DM3_S_PP);

    // Convert to m³/s: q_v = (people × dm³/s per person) / 1000
    let q_v = people * dm3_s_per_person / 1000.0;

    Ok(q_v)
}

/// Calculate temperature reduction factor f_v.
/// ISSO 53 formules 4.38-4.39, PDF p.47-48.
///
/// `delta_theta_v` is de gelaagdheidscorrectie Δθ_v uit tabel 2.3 (0 voor de
/// meeste systemen; −1 of −0,5 K bij wand-/vloer-lt-/betonkern-verwarming).
fn calculate_f_v(
    config: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
    delta_theta_v: f64,
) -> Result<f64> {
    let delta_t = theta_i - theta_e;
    if delta_t.abs() < 0.001 {
        return Ok(0.0); // Avoid division by zero
    }

    if config.has_heat_recovery {
        // WTW: formule 4.38 — f_v is het deel van Δθ dat nog opgewarmd moet
        // worden ná WTW. Fysisch: f_v · (θ_i − θ_e) = (θ_i − θ_t).
        // TODO A7: form. 4.38 met Δθ_v (f_v = (θ_i + Δθ_v − θ_t)/(θ_i − θ_e))
        // zodra θ_t echt uit een toevoertemperatuur-/voorverwarming-model komt
        // (U5). Nu is θ_t hier de gemodelleerde ná-WTW-toevoertemperatuur en is
        // de Δθ_v-stratificatie van de toevoerlucht al impliciet; we laten Δθ_v
        // bewust buiten deze tak om geen dubbeltelling te introduceren.
        let theta_t = if let Some(supply_temp) = config.supply_temperature {
            supply_temp
        } else {
            let efficiency = config.heat_recovery_efficiency.unwrap_or(0.75);
            theta_e + efficiency * (theta_i - theta_e)
        };

        let f_v = (theta_i - theta_t) / delta_t;
        Ok(f_v.clamp(0.0, 1.0))
    } else if config.has_preheating {
        // Voorverwarming: formule 4.38 geldt ook (zelfde definitie θ_t = toevoertemp).
        // Bij luchtverwarming (θ_t > θ_i) → f_v = 0 per norm.
        // TODO A7: idem 4.38-Δθ_v-tak (zie WTW hierboven, U5).
        let theta_t = config.preheating_temperature.unwrap_or(theta_i);
        if theta_t > theta_i {
            Ok(0.0)
        } else {
            let f_v = (theta_i - theta_t) / delta_t;
            Ok(f_v.clamp(0.0, 1.0))
        }
    } else {
        // Natuurlijke toevoer / mechanische toevoer zonder voorverwarming.
        // Formule 4.39: f_v = (θ_i + Δθ_v − θ_e) / (θ_i − θ_e).
        // Δθ_v = 0 → f_v = 1,0 (ongewijzigd t.o.v. de oude hardcode). Bij
        // wand-/vloer-lt-/betonkern-verwarming verlaagt Δθ_v (−1/−0,5) de f_v.
        let f_v = (theta_i + delta_theta_v - theta_e) / delta_t;
        Ok(f_v.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        GebruiksFunctie, RuimteType, VentilationSystemType, ConstructionElement,
        BoundaryType, MaterialType, VerticalPosition, Bezetting
    };

    #[test]
    fn test_wtw_ventilation_efficiency_applied() {
        // ISSO 53 §4.7.2 formule 4.38: WTW reduces f_v based on supply temperature
        // Verify that η=85% efficiency gives expected ~85% reduction in ventilation loss

        // Create a larger room for clearer effect
        let mut room = create_test_room();
        room.floor_area = 100.0;
        room.bezetting.personen = Some(10.0);

        let config_no_wtw = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let config_with_wtw = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let no_wtw = calculate_ventilation(&room, &config_no_wtw, 20.0, -10.0, HeatingSystem::default()).unwrap();
        let with_wtw = calculate_ventilation(&room, &config_with_wtw, 20.0, -10.0, HeatingSystem::default()).unwrap();

        // With η=0.85, expected f_v ≈ 0.15 (1 - η)
        assert!(
            (with_wtw.f_v - 0.15).abs() < 0.02,
            "f_v with 85% WTW should be ~0.15, got {}",
            with_wtw.f_v
        );

        // WTW should provide ~85% reduction in ventilation loss
        let reduction = 1.0 - with_wtw.phi_vent / no_wtw.phi_vent;
        assert!(
            reduction > 0.80,
            "WTW reduction should be >80%, got {:.1}%",
            reduction * 100.0
        );

        // f_v without WTW should be 1.0
        assert!(
            (no_wtw.f_v - 1.0).abs() < 0.001,
            "f_v without WTW should be 1.0, got {}",
            no_wtw.f_v
        );
    }

    #[test]
    fn test_ventilation_calculation_basic() {
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default());
        assert!(result.is_ok());

        let ventilation = result.unwrap();
        assert!(ventilation.phi_vent > 0.0);
        assert!(ventilation.h_v > 0.0);
        assert!(ventilation.q_v > 0.0);
        assert!((ventilation.f_v - 1.0).abs() < 0.001); // Natural ventilation f_v = 1.0
    }

    #[test]
    fn test_ventilation_with_heat_recovery() {
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.8),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default());
        assert!(result.is_ok());

        let ventilation = result.unwrap();
        assert!(ventilation.phi_vent >= 0.0);
        assert!(ventilation.h_v >= 0.0);
        assert!(ventilation.f_v < 1.0); // Heat recovery reduces f_v
        assert!(ventilation.f_v >= 0.0);
        // With 80% efficiency, f_v should be approximately 0.2 (1-η)
        assert!((ventilation.f_v - 0.2).abs() < 0.02, "Expected f_v ≈ 0.2, got {}", ventilation.f_v);
    }

    #[test]
    fn test_ventilation_smoke() {
        // Kantoor (0,05 pers/m²), floor_area 25 m² → area_based = 1,25 personen.
        // Ingevoerd 1 persoon < area_based → max-semantiek kiest 1,25.
        // q_v = 1,25 × 6,5/1000 = 0,008125 m³/s; H_v = 0,008125 × 1200 × 1 = 9,75 W/K.
        // (Voorheen koos de override 1,0 pers → H_v = 7,8 W/K; aangepast voor max-logica.)
        let mut room = create_test_room();
        room.bezetting.personen = Some(1.0);

        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default());
        assert!(result.is_ok());
        let ventilation = result.unwrap();
        assert!((ventilation.h_v - 9.75).abs() < 0.1, "Expected ~9.75, got {}", ventilation.h_v);
        assert!((ventilation.f_v - 1.0).abs() < 0.001, "f_v should be 1.0 for natural ventilation");
    }

    #[test]
    fn test_people_count_is_max_of_explicit_and_area_based() {
        // Kantoor/Kantoorruimte → personen_per_m2 = 0,05.
        // Configuratie zonder WTW (f_v = 1), zodat H_v = q_v × 1200 lineair in people is.
        // q_v = people × 6,5/1000  →  H_v = people × 6,5 × 1,2 = people × 7,8.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        // floor_area 25 m² × 0,05 = 1,25 area-based personen.

        // 1) explicit > area_based → gebruikt explicit (10,0).
        let mut room_explicit_high = create_test_room();
        room_explicit_high.floor_area = 25.0;
        room_explicit_high.bezetting.personen = Some(10.0);
        let r = calculate_ventilation(&room_explicit_high, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert!((r.h_v - 10.0 * 7.8).abs() < 0.1, "explicit>area: expected H_v {}, got {}", 10.0 * 7.8, r.h_v);

        // 2) explicit < area_based → gebruikt area_based (1,25). NIEUW gedrag:
        //    voorheen werd hier de override 0,5 gekozen.
        let mut room_explicit_low = create_test_room();
        room_explicit_low.floor_area = 25.0;
        room_explicit_low.bezetting.personen = Some(0.5);
        let r = calculate_ventilation(&room_explicit_low, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert!((r.h_v - 1.25 * 7.8).abs() < 0.1, "explicit<area: expected area-based H_v {}, got {}", 1.25 * 7.8, r.h_v);

        // 3) personen = None → gebruikt area_based (1,25).
        let mut room_none = create_test_room();
        room_none.floor_area = 25.0;
        room_none.bezetting.personen = None;
        let r = calculate_ventilation(&room_none, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert!((r.h_v - 1.25 * 7.8).abs() < 0.1, "none: expected area-based H_v {}, got {}", 1.25 * 7.8, r.h_v);
    }

    #[test]
    fn test_ventilation_vabi_wtw_scenario() {
        // Vabi scenario: η=0.85, θ_i=20, θ_e=-10, expected f_v ≈ 0.15
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default());
        assert!(result.is_ok());

        let ventilation = result.unwrap();
        assert!((ventilation.f_v - 0.15).abs() < 0.02, "Expected f_v ≈ 0.15, got {}", ventilation.f_v);
    }

    #[test]
    fn test_room_without_ventilation_requirement_yields_zero() {
        // TechnischeRuimte / Bergruimte hebben geen personen-gebaseerde eis in
        // tabel 4.10 → requirement() == None. De berekening moet dan q_v = 0
        // teruggeven i.p.v. een Err (regressie: voorheen NotSupported-crash die
        // de hele projectberekening blokkeerde).
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        for ruimte in [RuimteType::TechnischeRuimte, RuimteType::Bergruimte] {
            let mut room = create_test_room();
            room.ruimte_type = ruimte;

            let result = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default());
            assert!(
                result.is_ok(),
                "{:?} zonder eis moet Ok teruggeven, kreeg {:?}",
                ruimte,
                result
            );
            let v = result.unwrap();
            assert_eq!(v.q_v, 0.0, "q_v moet 0 zijn voor {:?}", ruimte);
            assert_eq!(v.phi_vent, 0.0, "phi_vent moet 0 zijn voor {:?}", ruimte);
            assert_eq!(v.h_v, 0.0, "h_v moet 0 zijn voor {:?}", ruimte);
        }
    }

    #[test]
    fn test_office_calculation_unchanged_after_none_fix() {
        // Borg dat de None→0 fix de normale kantoor-berekening niet raakt.
        // Identiek aan test_ventilation_smoke: Kantoor, 25 m², 1 pers ingevoerd
        // → area-based 1,25 pers wint → H_v = 9,75 W/K.
        let mut room = create_test_room();
        room.bezetting.personen = Some(1.0);

        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert!((v.h_v - 9.75).abs() < 0.1, "Expected ~9.75, got {}", v.h_v);
        assert!(v.q_v > 0.0, "kantoor moet q_v > 0 hebben");
    }

    #[test]
    fn test_no_mechanical_supply_gates_ventilation_to_zero() {
        // ISSO 53: alleen toevoer telt mee. has_mechanical_supply == Some(false)
        // → q_v = 0 en dus phi_vent = 0, ondanks geldige eis + bezetting.
        let mut room = create_test_room();
        room.bezetting.personen = Some(10.0);
        room.has_mechanical_supply = Some(false);

        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(v.q_v, 0.0, "geen toevoer → q_v moet 0 zijn");
        assert_eq!(v.phi_vent, 0.0, "geen toevoer → phi_vent moet 0 zijn");
        assert_eq!(v.h_v, 0.0, "geen toevoer → h_v moet 0 zijn");
    }

    #[test]
    fn test_mechanical_supply_true_unchanged() {
        // has_mechanical_supply == Some(true) → identiek aan veld afwezig (None):
        // de gate grijpt niet in, q_v > 0.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room_supply = create_test_room();
        room_supply.bezetting.personen = Some(10.0);
        room_supply.has_mechanical_supply = Some(true);
        let v_supply = calculate_ventilation(&room_supply, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert!(v_supply.q_v > 0.0, "met toevoer (Some(true)) moet q_v > 0 zijn");

        // backward-compat: None geeft exact hetzelfde resultaat als Some(true).
        let mut room_none = create_test_room();
        room_none.bezetting.personen = Some(10.0);
        room_none.has_mechanical_supply = None;
        let v_none = calculate_ventilation(&room_none, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(
            v_supply.q_v, v_none.q_v,
            "Some(true) en None moeten identieke q_v geven"
        );
        assert_eq!(
            v_supply.phi_vent, v_none.phi_vent,
            "Some(true) en None moeten identieke phi_vent geven"
        );
    }

    #[test]
    fn test_established_q_v_is_used_directly() {
        // Fase 3: ventilation_q_v_established == Some(0.05) → q_v == 0.05 ongeacht
        // functie, bezetting of supply-gate.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.bezetting.personen = Some(10.0); // zou normaal hoge q_v geven
        room.ventilation_q_v_established = Some(0.05);

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(v.q_v, 0.05, "vastgestelde q_v moet direct gebruikt worden");
    }

    #[test]
    fn test_established_q_v_overrides_supply_gate() {
        // Vastgestelde q_v overrulet de has_mechanical_supply==Some(false)-gate.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.has_mechanical_supply = Some(false);
        room.ventilation_q_v_established = Some(0.03);

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(
            v.q_v, 0.03,
            "vastgestelde q_v moet de supply-gate overrulen"
        );
    }

    #[test]
    fn test_established_q_v_zero_yields_zero() {
        // Some(0.0) → q_v == 0 (expliciet geen toevoer).
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.bezetting.personen = Some(10.0);
        room.ventilation_q_v_established = Some(0.0);

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(v.q_v, 0.0, "Some(0.0) → q_v moet 0 zijn");
        assert_eq!(v.phi_vent, 0.0, "Some(0.0) → phi_vent moet 0 zijn");
    }

    #[test]
    fn test_established_q_v_negative_clamped_to_zero() {
        // Defensief: negatieve waarde wordt geclamped op 0.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.ventilation_q_v_established = Some(-0.02);

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        assert_eq!(v.q_v, 0.0, "negatieve vastgestelde q_v moet op 0 clampen");
    }

    /// M4b: reproduceert ISSO 53 voorbeeld 6.2 se-scenario — een gegeven
    /// qv=100 m3/h (27,778 dm3/s) overrulet de Bbl/bezetting-afleiding, ook
    /// als de ruimte een lage bezetting heeft die anders een veel lager qv
    /// zou opleveren. Confirmed: `ventilation_q_v_established` was al
    /// volledig geïmplementeerd vóór deze sessie (M4b behoefde geen
    /// engine-wijziging, alleen het invullen van dit fixture-veld).
    #[test]
    fn test_m4a_established_q_v_reproduces_voorbeeld_62_qv() {
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.8),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.floor_area = 18.7;
        room.bezetting.personen = Some(2.0); // zou zonder override een veel lager q_v geven
        room.ventilation_q_v_established = Some(100.0 / 3600.0); // 100 m3/h

        let v = calculate_ventilation(&room, &config, 20.0, -8.5, HeatingSystem::default()).unwrap();
        assert!(
            (v.q_v - 0.027778).abs() < 1e-5,
            "q_v moet de vastgestelde 100 m3/h zijn, niet de bezetting-afgeleide waarde, kreeg {}",
            v.q_v
        );
        // H_v = 27,778e-3 * 1200 * f_v(WTW 80%) = 27,778e-3*1200*0,2 = 6,6667.
        assert!((v.h_v - 6.6667).abs() < 0.001, "H_v verwacht ~6,6667, kreeg {}", v.h_v);
        // Phi_vent = H_v * (20 - (-8,5)) = 6,6667 * 28,5 = 190,0 W (publicatie).
        assert!((v.phi_vent - 190.0).abs() < 0.5, "Phi_vent verwacht ~190 W, kreeg {}", v.phi_vent);
    }

    #[test]
    fn test_established_q_v_none_unchanged() {
        // None → reguliere afleiding ongewijzigd; identiek aan de smoke-test.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.bezetting.personen = Some(1.0);
        room.ventilation_q_v_established = None;

        let v = calculate_ventilation(&room, &config, 20.0, -10.0, HeatingSystem::default()).unwrap();
        // Kantoor 25 m² → area-based 1,25 pers → H_v = 9,75 W/K.
        assert!((v.h_v - 9.75).abs() < 0.1, "None: verwacht ~9.75, kreeg {}", v.h_v);
    }

    /// A7: bij wandverwarming met R_c < 3,5 (default-room wand U=0,28 → R_c≈3,40)
    /// geldt Δθ_v = −1 K → form. 4.39 f_v = (20 − 1 + 10)/30 = 29/30 ≈ 0,9667,
    /// en het ventilatieverlies daalt ~3,3% t.o.v. f_v = 1,0.
    #[test]
    fn test_a7_wandverwarming_rc_low_reduces_f_v() {
        let mut room = create_test_room();
        room.bezetting.personen = Some(5.0);

        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        // Referentie: radiatoren-ht → Δθ_v = 0 → f_v = 1,0.
        let radi = calculate_ventilation(
            &room, &config, 20.0, -10.0,
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
        ).unwrap();
        assert!((radi.f_v - 1.0).abs() < 1e-9, "radi → f_v=1,0, kreeg {}", radi.f_v);

        // Wandverwarming, R_c < 3,5 → Δθ_v = −1 → f_v = 29/30.
        let wand = calculate_ventilation(
            &room, &config, 20.0, -10.0, HeatingSystem::Wandverwarming,
        ).unwrap();
        let expected_fv = 29.0 / 30.0;
        assert!((wand.f_v - expected_fv).abs() < 1e-9, "wand f_v verwacht {expected_fv}, kreeg {}", wand.f_v);

        // ~3,3% lager ventilatieverlies (q_v identiek).
        let reduction = 1.0 - wand.phi_vent / radi.phi_vent;
        assert!((reduction - (1.0 - expected_fv)).abs() < 1e-9);
        assert!(reduction > 0.03 && reduction < 0.035, "verlies ~3,3% lager, kreeg {:.4}", reduction);
    }

    /// A7-regressie: een systeem met Δθ_v = 0 (radiatoren) houdt exact f_v = 1,0,
    /// ongeacht de R_c-kolom. Borgt dat de 4.39-tak voor de meeste systemen
    /// bit-identiek is aan de oude hardcode.
    #[test]
    fn test_a7_zero_delta_v_keeps_f_v_one() {
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        for sys in [
            HeatingSystem::RadiatorenConvHtEnLuchtverwarming,
            HeatingSystem::LokaleVerwarming,
            HeatingSystem::Plafondverwarming,
            HeatingSystem::VloerverwarmingPlusHtRadi,
        ] {
            let v = calculate_ventilation(&room, &config, 20.0, -10.0, sys).unwrap();
            assert!((v.f_v - 1.0).abs() < 1e-9, "{sys:?} → f_v moet 1,0 zijn, kreeg {}", v.f_v);
        }
    }

    /// A7: R_c-kolomkeuze. Een goed geïsoleerde gevel (U=0,15 → R_c≈6,5 ≥ 3,5)
    /// kiest de −0,5 K-kolom → f_v = 29,5/30; een slechte gevel de −1 K-kolom.
    #[test]
    fn test_a7_rc_high_picks_half_kelvin_column() {
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room_high = create_test_room();
        room_high.constructions[0].u_value = 0.15; // R_c ≈ 6,5 ≥ 3,5
        room_high.bezetting.personen = Some(5.0);
        let v_high = calculate_ventilation(&room_high, &config, 20.0, -10.0, HeatingSystem::Vloerverwarming).unwrap();
        assert!((v_high.f_v - 29.5 / 30.0).abs() < 1e-9, "R_c≥3,5 → f_v=29,5/30, kreeg {}", v_high.f_v);

        let mut room_low = create_test_room();
        room_low.constructions[0].u_value = 0.40; // R_c ≈ 2,33 < 3,5
        room_low.bezetting.personen = Some(5.0);
        let v_low = calculate_ventilation(&room_low, &config, 20.0, -10.0, HeatingSystem::Vloerverwarming).unwrap();
        assert!((v_low.f_v - 29.0 / 30.0).abs() < 1e-9, "R_c<3,5 → f_v=29/30, kreeg {}", v_low.f_v);
    }

    /// D2: bouwfase ontkoppeld van de hardcoded `Nieuwbouw`. Voor Kantoorruimte
    /// is het tabel-4.10-debiet 6,5 dm³/s·pp nieuwbouw vs 3,44 dm³/s·pp bestaand
    /// → nieuwbouw geeft 6,5/3,44 ≈ 1,890× = +89% Φ_V t.o.v. bestaande bouw.
    /// Vóór D2 kreeg elke bestaande-bouw-ruimte stilzwijgend het nieuwbouw-debiet.
    #[test]
    fn test_d2_bouwfase_decouples_ventilation_rate() {
        let mut room = create_test_room();
        room.bezetting.personen = Some(5.0); // forceer personen-gedreven debiet

        let base = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            bouwfase: VentilatieBouwfase::Nieuwbouw,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let config_nieuw = VentilationConfig { bouwfase: VentilatieBouwfase::Nieuwbouw, ..base.clone() };
        let config_bestaand = VentilationConfig { bouwfase: VentilatieBouwfase::Bestaand, ..base };

        let nieuw = calculate_ventilation(&room, &config_nieuw, 20.0, -10.0, HeatingSystem::default()).unwrap();
        let bestaand = calculate_ventilation(&room, &config_bestaand, 20.0, -10.0, HeatingSystem::default()).unwrap();

        // f_v identiek (1,0) voor beide → ratio zit volledig in q_v.
        assert!((nieuw.f_v - 1.0).abs() < 1e-9);
        assert!((bestaand.f_v - 1.0).abs() < 1e-9);

        // q_v-ratio = 6,5 / 3,44 ≈ 1,8895.
        let ratio = nieuw.phi_vent / bestaand.phi_vent;
        let expected = 6.5 / 3.44;
        assert!(
            (ratio - expected).abs() < 1e-6,
            "nieuwbouw/bestaand Φ_V-ratio verwacht {expected:.4} (≈+89%), kreeg {ratio:.4}"
        );
        // Sanity: +89% afwijking, niet bit-identiek.
        assert!((ratio - 1.89).abs() < 0.01, "ratio moet ~1,89 zijn, kreeg {ratio}");
    }

    /// D2-regressie: de serde-default voor `bouwfase` is `Nieuwbouw`, dus een
    /// project zonder expliciet veld behoudt exact het oude (pre-D2) gedrag.
    #[test]
    fn test_d2_serde_default_is_nieuwbouw_backward_compat() {
        // JSON zonder bouwfase-veld → moet als Nieuwbouw deserialiseren.
        let json = r#"{"systemType":"systemB","hasHeatRecovery":false,"hasPreheating":false}"#;
        let config: VentilationConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.bouwfase,
            VentilatieBouwfase::Nieuwbouw,
            "ontbrekend bouwfase-veld moet op Nieuwbouw defaulten (backward-compat)"
        );
    }

    fn create_test_room() -> Room {
        Room {
            id: "test_room".to_string(),
            name: "Test Office".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Kantoorruimte,
            floor_area: 25.0,
            height: 3.0,
            custom_temperature: Some(20.0),
            constructions: vec![
                ConstructionElement {
                    id: "wall".to_string(),
                    description: "Wall".to_string(),
                    area: 20.0,
                    u_value: 0.28,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    unheated_space: None,
                }
            ],
            bezetting: Bezetting {
                personen: Some(2.0),
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
            has_mechanical_supply: None,
            ventilation_q_v_established: None,
        }
    }
}