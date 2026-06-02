//! Ventilation heat loss for ISSO 53 (§4.7.2).

use crate::error::Result;
use crate::formulas::RHO_CP_AIR;
use crate::model::{Room, VentilationConfig};
use crate::tables::ventilation_requirements::{requirement, ventilation_rate_per_person};
use crate::model::enums::VentilatieBouwfase;
use crate::tables::temperature::resolve_theta_i;

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
pub fn calculate_ventilation(
    room: &Room,
    config: &VentilationConfig,
    theta_i: f64,
    theta_e: f64,
) -> Result<VentilationResult> {
    // Calculate ventilation flow rate q_v in m³/s
    let q_v = calculate_ventilation_flow_rate(room)?;

    // Calculate temperature reduction factor f_v
    let f_v = calculate_f_v(config, theta_i, theta_e)?;

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
) -> Result<f64> {
    let result = calculate_ventilation(room, ventilation, theta_i, theta_e)?;
    Ok(result.h_v)
}

/// Calculate the ventilation heat loss Φ_vent for a room.
/// ISSO 53 formule 4.35, PDF p.47: Φ_vent = H_v × (θ_i - θ_e)
pub fn calculate_phi_vent(
    room: &Room,
    ventilation: &VentilationConfig,
    theta_e: f64,
) -> Result<f64> {
    let theta_i = resolve_theta_i(room, theta_e);

    let result = calculate_ventilation(room, ventilation, theta_i, theta_e)?;
    Ok(result.phi_vent)
}

/// Calculate ventilation flow rate q_v in m³/s based on room occupancy and requirements.
fn calculate_ventilation_flow_rate(room: &Room) -> Result<f64> {
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
        .unwrap_or(0.05); // Default density
    let area_based = room.floor_area * density;
    let people = match room.bezetting.personen {
        Some(explicit) => explicit.max(area_based),
        None => area_based,
    };

    // Get ventilation rate per person in dm³/s
    let dm3_s_per_person = ventilation_rate_per_person(req, VentilatieBouwfase::Nieuwbouw)
        .unwrap_or(6.5); // Default rate

    // Convert to m³/s: q_v = (people × dm³/s per person) / 1000
    let q_v = people * dm3_s_per_person / 1000.0;

    Ok(q_v)
}

/// Calculate temperature reduction factor f_v.
/// ISSO 53 formules 4.38-4.39, PDF p.47-48.
fn calculate_f_v(config: &VentilationConfig, theta_i: f64, theta_e: f64) -> Result<f64> {
    if (theta_i - theta_e).abs() < 0.001 {
        return Ok(0.0); // Avoid division by zero
    }

    if config.has_heat_recovery {
        // WTW: formule 4.38 — f_v is het deel van Δθ dat nog opgewarmd moet
        // worden ná WTW. Fysisch: f_v · (θ_i − θ_e) = (θ_i − θ_t).
        let theta_t = if let Some(supply_temp) = config.supply_temperature {
            supply_temp
        } else {
            let efficiency = config.heat_recovery_efficiency.unwrap_or(0.75);
            theta_e + efficiency * (theta_i - theta_e)
        };

        let f_v = (theta_i - theta_t) / (theta_i - theta_e);
        Ok(f_v.clamp(0.0, 1.0))
    } else if config.has_preheating {
        // Voorverwarming: formule 4.38 geldt ook (zelfde definitie θ_t = toevoertemp).
        // Bij luchtverwarming (θ_t > θ_i) → f_v = 0 per norm.
        let theta_t = config.preheating_temperature.unwrap_or(theta_i);
        if theta_t > theta_i {
            Ok(0.0)
        } else {
            let f_v = (theta_i - theta_t) / (theta_i - theta_e);
            Ok(f_v.clamp(0.0, 1.0))
        }
    } else {
        // Natural ventilation - f_v = 1.0
        Ok(1.0)
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
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let config_with_wtw = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let no_wtw = calculate_ventilation(&room, &config_no_wtw, 20.0, -10.0).unwrap();
        let with_wtw = calculate_ventilation(&room, &config_with_wtw, 20.0, -10.0).unwrap();

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
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
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
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.8),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
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
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
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
        let r = calculate_ventilation(&room_explicit_high, &config, 20.0, -10.0).unwrap();
        assert!((r.h_v - 10.0 * 7.8).abs() < 0.1, "explicit>area: expected H_v {}, got {}", 10.0 * 7.8, r.h_v);

        // 2) explicit < area_based → gebruikt area_based (1,25). NIEUW gedrag:
        //    voorheen werd hier de override 0,5 gekozen.
        let mut room_explicit_low = create_test_room();
        room_explicit_low.floor_area = 25.0;
        room_explicit_low.bezetting.personen = Some(0.5);
        let r = calculate_ventilation(&room_explicit_low, &config, 20.0, -10.0).unwrap();
        assert!((r.h_v - 1.25 * 7.8).abs() < 0.1, "explicit<area: expected area-based H_v {}, got {}", 1.25 * 7.8, r.h_v);

        // 3) personen = None → gebruikt area_based (1,25).
        let mut room_none = create_test_room();
        room_none.floor_area = 25.0;
        room_none.bezetting.personen = None;
        let r = calculate_ventilation(&room_none, &config, 20.0, -10.0).unwrap();
        assert!((r.h_v - 1.25 * 7.8).abs() < 0.1, "none: expected area-based H_v {}, got {}", 1.25 * 7.8, r.h_v);
    }

    #[test]
    fn test_ventilation_vabi_wtw_scenario() {
        // Vabi scenario: η=0.85, θ_i=20, θ_e=-10, expected f_v ≈ 0.15
        let room = create_test_room();
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemD,
            has_heat_recovery: true,
            heat_recovery_efficiency: Some(0.85),
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let result = calculate_ventilation(&room, &config, 20.0, -10.0);
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

            let result = calculate_ventilation(&room, &config, 20.0, -10.0);
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
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
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
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
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
        let v_supply = calculate_ventilation(&room_supply, &config, 20.0, -10.0).unwrap();
        assert!(v_supply.q_v > 0.0, "met toevoer (Some(true)) moet q_v > 0 zijn");

        // backward-compat: None geeft exact hetzelfde resultaat als Some(true).
        let mut room_none = create_test_room();
        room_none.bezetting.personen = Some(10.0);
        room_none.has_mechanical_supply = None;
        let v_none = calculate_ventilation(&room_none, &config, 20.0, -10.0).unwrap();
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

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
        assert_eq!(v.q_v, 0.05, "vastgestelde q_v moet direct gebruikt worden");
    }

    #[test]
    fn test_established_q_v_overrides_supply_gate() {
        // Vastgestelde q_v overrulet de has_mechanical_supply==Some(false)-gate.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
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

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
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

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
        assert_eq!(v.q_v, 0.0, "Some(0.0) → q_v moet 0 zijn");
        assert_eq!(v.phi_vent, 0.0, "Some(0.0) → phi_vent moet 0 zijn");
    }

    #[test]
    fn test_established_q_v_negative_clamped_to_zero() {
        // Defensief: negatieve waarde wordt geclamped op 0.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        };

        let mut room = create_test_room();
        room.ventilation_q_v_established = Some(-0.02);

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
        assert_eq!(v.q_v, 0.0, "negatieve vastgestelde q_v moet op 0 clampen");
    }

    #[test]
    fn test_established_q_v_none_unchanged() {
        // None → reguliere afleiding ongewijzigd; identiek aan de smoke-test.
        let config = VentilationConfig {
            system_type: VentilationSystemType::SystemB,
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

        let v = calculate_ventilation(&room, &config, 20.0, -10.0).unwrap();
        // Kantoor 25 m² → area-based 1,25 pers → H_v = 9,75 W/K.
        assert!((v.h_v - 9.75).abs() < 0.1, "None: verwacht ~9.75, kreeg {}", v.h_v);
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