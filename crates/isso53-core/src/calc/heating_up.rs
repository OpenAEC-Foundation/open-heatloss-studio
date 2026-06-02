//! Toeslag voor bedrijfsbeperking (heating-up supplement) — ISSO 53 §4.8.
//!
//! Automatische berekening van de specifieke toeslag φ_hu,i [W/m²] conform
//! §4.8.1 (vrije afkoeling, tabel 4.13) en §4.8.2 (beperkte afkoeling,
//! tabel 4.14), met lineaire interpolatie over de opwarmtijd en de
//! maatgevende `max(doordeweeks, weekend)`-logica. De §4.8.3-reductie
//! (formule 4.45) verrekent het ventilatievermogen dat bij uitgeschakelde
//! mechanische toevoer beschikbaar komt voor opwarmen.
//!
//! Tabelbron: ISSO 53 (2016) §4.8, tabel 4.13/4.14, PDF p.53.

use crate::error::Result;
use crate::model::{CoolingRegime, HeatingUpConfig, Room};
use crate::tables::heating_up::{
    lookup_free_cooling, lookup_limited_cooling, AirChanges, BuildingWeight,
};
use crate::tables::thermal_mass::c_eff;
use crate::model::enums::ThermalMass;

/// Bepaal de maatgevende specifieke toeslag φ_hu,i [W/m²].
///
/// Maatgevend = `max(doordeweekse verlaging, weekendverlaging)` (§4.8).
/// Retourneert `None` als geen van beide combinaties in de norm gedefinieerd is.
fn specific_supplement(
    config: &HeatingUpConfig,
    weight: BuildingWeight,
) -> Option<f64> {
    let air: AirChanges = config.air_changes.into();

    let (weekday, weekend) = match config.regime {
        CoolingRegime::Free {
            setback_hours_weekday,
            setback_hours_weekend,
        } => (
            lookup_free_cooling(setback_hours_weekday, air, weight, config.warmup_hours_weekday),
            lookup_free_cooling(setback_hours_weekend, air, weight, config.warmup_hours_weekend),
        ),
        CoolingRegime::Limited {
            degrees_weekday,
            degrees_weekend,
        } => (
            lookup_limited_cooling(degrees_weekday, air, weight, config.warmup_hours_weekday),
            lookup_limited_cooling(degrees_weekend, air, weight, config.warmup_hours_weekend),
        ),
    };

    // max(dag, weekend); negeer ongedefinieerde kanten.
    match (weekday, weekend) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Calculate the heating-up supplement Φ_hu,i for a room (ISSO 53 §4.8).
///
/// Berekening:
/// 1. `φ_hu,i` [W/m²] — handmatige override `p_w_per_m2_override` indien gezet,
///    anders automatische tabel-lookup (4.13/4.14) met max(dag, weekend);
/// 2. `Φ_op = A_vl · φ_hu,i` (formule 4.43);
/// 3. §4.8.3 reductie: bij uitgeschakelde mechanische toevoer (`a = 1`)
///    `Φ_hu,i = Φ_op − H_v · (θ_i − θ_e)`, geclamp op ≥ 0 (formule 4.45).
///    Zonder mechanische toevoer-uitschakeling (`a = 0`): `Φ_hu,i = Φ_op`.
///
/// # Arguments
/// * `room` - The room to calculate for (levert `floor_area` = A_vl).
/// * `config` - Heating-up configuration (regime, opwarmtijden, override).
/// * `thermal_mass` - Thermische massa van het gebouw → zwaarteklasse (l/z).
/// * `h_v` - Specifiek ventilatiewarmteverlies H_v [W/K] van de ruimte (§4.7.2).
/// * `theta_i` - Ontwerpbinnentemperatuur θ_i [°C].
/// * `theta_e` - Ontwerpbuitentemperatuur θ_e [°C].
///
/// # Returns
/// Toe te rekenen toeslag voor bedrijfsbeperking Φ_hu,i in W (≥ 0).
pub fn calculate_heating_up(
    room: &Room,
    config: &HeatingUpConfig,
    thermal_mass: ThermalMass,
    h_v: f64,
    theta_i: f64,
    theta_e: f64,
) -> Result<f64> {
    if !config.setback_active {
        return Ok(0.0);
    }

    // Stap 1 — specifieke toeslag φ_hu,i [W/m²].
    let phi_specific = match config.p_w_per_m2_override {
        // Handmatige override heeft voorrang op de automatische tabel-lookup.
        Some(p) => p,
        None => {
            // Zwaarte gebouw uit c_eff (§4.8.1): c_eff ≤ 70 → l, anders z.
            // ThermalMass-mapping: Licht(15)+Gemiddeld(50) → l, Zwaar(75) → z.
            let weight = BuildingWeight::from_c_eff(c_eff(thermal_mass));
            // Geen gedefinieerde tabelwaarde (verlaging-uren ∉ tabeldomein of
            // graden ∉ {1..5}) → expliciete fout i.p.v. stille 0 W/m², die
            // een fout antwoord zonder waarschuwing zou opleveren.
            specific_supplement(config, weight).ok_or_else(|| {
                crate::error::Isso53Error::InvalidHeatingUpParameters(format!(
                    "geen tabelwaarde φ_hu,i (4.13/4.14) voor regime {:?}, \
                     opwarmtijden dag={} weekend={}, zwaarte {:?}",
                    config.regime,
                    config.warmup_hours_weekday,
                    config.warmup_hours_weekend,
                    weight
                ))
            })?
        }
    };

    // Stap 2 — Φ_op = A_vl · φ_hu,i (formule 4.43).
    let phi_op = phi_specific * room.floor_area;

    // Stap 3 — §4.8.3 (formule 4.45): bij uitschakelen mechanische toevoer
    // komt H_v·(θ_i−θ_e) beschikbaar voor opwarmen → aftrekken, clamp ≥ 0.
    let phi_hu = if config.mechanical_supply_off {
        (phi_op - h_v * (theta_i - theta_e)).max(0.0)
    } else {
        phi_op
    };

    Ok(phi_hu)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AirChangeRate, Bezetting, GebruiksFunctie, RuimteType};

    fn create_test_room(floor_area: f64) -> Room {
        Room {
            id: "test_room".to_string(),
            name: "Test Room".to_string(),
            gebruiks_functie: GebruiksFunctie::Kantoor,
            ruimte_type: RuimteType::Verblijfsruimte,
            floor_area,
            height: 3.0,
            custom_temperature: None,
            constructions: vec![],
            bezetting: Bezetting {
                personen: None,
                personen_per_m2_default: None,
            },
            infiltration_reduction_z: 1.0,
            has_mechanical_supply: None,
            ventilation_q_v_established: None,
        }
    }

    #[test]
    fn test_heating_up_inactive() {
        let room = create_test_room(25.0);
        let config = HeatingUpConfig {
            setback_active: false,
            p_w_per_m2_override: Some(10.0), // Should be ignored
            ..Default::default()
        };

        let result =
            calculate_heating_up(&room, &config, ThermalMass::Licht, 0.0, 20.0, -10.0).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_manual_override() {
        let room = create_test_room(25.0);
        let config = HeatingUpConfig {
            setback_active: true,
            p_w_per_m2_override: Some(10.0),
            mechanical_supply_off: false,
            ..Default::default()
        };

        let result =
            calculate_heating_up(&room, &config, ThermalMass::Licht, 0.0, 20.0, -10.0).unwrap();
        // Override: 10.0 W/m² × 25.0 m² = 250.0 W (geen §4.8.3 reductie).
        assert_eq!(result, 250.0);
    }

    /// VERPLICHTE REGRESSIE-GATE — uitgewerkt voorbeeld ISSO 53 PDF p.62/66.
    ///
    /// Inputs (p.62/66):
    /// - c_eff = 15,6 Wh/(m³·K) → licht (l);
    /// - 14 uur bedrijfsbeperking doordeweeks, 62 uur (weekend);
    /// - n = 0,1 luchtwisselingen;
    /// - opwarmtijd doordeweeks 2 h, na weekend 4 h;
    /// - ventilatie uitgeschakeld → a = 1, H_v = 6,672 W/K, θ_i = 20, θ_e = -8,5;
    /// - A_vl = 5,8 × 3,5 = 20,3 m².
    ///
    /// Verwacht (p.66): φ doordeweeks 28 W/m², weekend 17 W/m², max = 28;
    /// Φ_op = 5,8 × 3,5 × 28 = 568 W; Φ_hu = 568 − 1 × 6,672 × 28,5 = 378 W.
    #[test]
    fn regression_isso53_example_p66() {
        let room = create_test_room(5.8 * 3.5); // A_vl = 20,3 m²
        let config = HeatingUpConfig {
            setback_active: true,
            p_w_per_m2_override: None,
            regime: CoolingRegime::Free {
                setback_hours_weekday: 14,
                setback_hours_weekend: 62,
            },
            air_changes: AirChangeRate::Low, // n = 0,1
            warmup_hours_weekday: 2.0,
            warmup_hours_weekend: 4.0,
            mechanical_supply_off: true, // ventilatie uit → a = 1
        };

        let h_v = 6.672;
        let theta_i = 20.0;
        let theta_e = -8.5;

        // Specifieke toeslag-check (max(28, 17) = 28 W/m²).
        let weight = BuildingWeight::from_c_eff(15.6);
        assert_eq!(weight, BuildingWeight::Light);
        let phi_specific = specific_supplement(&config, weight).unwrap();
        assert!(
            (phi_specific - 28.0).abs() < 1e-9,
            "φ_hu,i moet 28 W/m² zijn (max van 28 doordeweeks en 17 weekend), kreeg {phi_specific}"
        );

        // Φ_op = 20,3 × 28 = 568,4 W.
        let phi_op = phi_specific * room.floor_area;
        assert!((phi_op - 568.4).abs() < 0.5, "Φ_op ≈ 568 W, kreeg {phi_op}");

        // Φ_hu,i = 568,4 − 6,672 × 28,5 = 378,2 W.
        let phi_hu =
            calculate_heating_up(&room, &config, ThermalMass::Licht, h_v, theta_i, theta_e)
                .unwrap();
        assert!(
            (phi_hu - 378.0).abs() < 1.0,
            "Φ_hu,i moet ≈ 378 W zijn (norm-voorbeeld p.66), kreeg {phi_hu}"
        );
    }

    #[test]
    fn test_invalid_params_return_err() {
        // B1-regressie: een verlaging-uren-waarde buiten het tabeldomein
        // (geldig is {8, 14, 62} voor vrije afkoeling) mag GEEN stille 0
        // opleveren maar een expliciete fout.
        let room = create_test_room(25.0);
        let config = HeatingUpConfig {
            setback_active: true,
            p_w_per_m2_override: None,
            regime: CoolingRegime::Free {
                setback_hours_weekday: 99, // ∉ {8,14,62} → geen tabelkolom
                setback_hours_weekend: 99,
            },
            air_changes: AirChangeRate::Low,
            warmup_hours_weekday: 2.0,
            warmup_hours_weekend: 4.0,
            mechanical_supply_off: false,
        };

        let result =
            calculate_heating_up(&room, &config, ThermalMass::Licht, 0.0, 20.0, -10.0);
        assert!(
            matches!(
                result,
                Err(crate::error::Isso53Error::InvalidHeatingUpParameters(_))
            ),
            "ongeldige opwarmtoeslag-invoer moet een Err geven, kreeg {result:?}"
        );
    }

    #[test]
    fn test_valid_params_still_compute() {
        // B1-regressie: het VALIDE pad moet exact dezelfde waarde geven als
        // vóór de fix. Hergebruik de geldige case uit test_heavy_building.
        let room = create_test_room(10.0);
        let config = HeatingUpConfig {
            setback_active: true,
            regime: CoolingRegime::Free {
                setback_hours_weekday: 14,
                setback_hours_weekend: 62,
            },
            warmup_hours_weekday: 2.0,
            warmup_hours_weekend: 12.0,
            mechanical_supply_off: false,
            ..Default::default()
        };
        let result =
            calculate_heating_up(&room, &config, ThermalMass::Zwaar, 0.0, 20.0, -10.0).unwrap();
        // max(18, 31) = 31 W/m² × 10 m² = 310 W.
        assert!((result - 310.0).abs() < 1e-9, "kreeg {result}");
    }

    #[test]
    fn test_clamp_negative_to_zero() {
        // Grote H_v zodat de §4.8.3 reductie Φ_op overschrijdt → clamp op 0.
        let room = create_test_room(20.0);
        let config = HeatingUpConfig {
            setback_active: true,
            mechanical_supply_off: true,
            ..Default::default()
        };
        // Φ_op klein, reductie groot → negatief vóór clamp.
        let result =
            calculate_heating_up(&room, &config, ThermalMass::Zwaar, 1000.0, 20.0, -10.0).unwrap();
        assert_eq!(result, 0.0, "Φ_hu,i moet geclamp worden op ≥ 0");
    }

    #[test]
    fn test_heavy_building_uses_z_column() {
        // Zwaar gebouw (c_eff=75 > 70) → z-kolom. 14 uur/0,1/z/2h = 18 W/m².
        let room = create_test_room(10.0);
        let config = HeatingUpConfig {
            setback_active: true,
            regime: CoolingRegime::Free {
                setback_hours_weekday: 14,
                setback_hours_weekend: 62,
            },
            warmup_hours_weekday: 2.0,
            warmup_hours_weekend: 12.0, // weekend 62/0,1/z/12h = 31
            mechanical_supply_off: false,
            ..Default::default()
        };
        // max(18 doordeweeks, 31 weekend) = 31 W/m² × 10 = 310 W.
        let result =
            calculate_heating_up(&room, &config, ThermalMass::Zwaar, 0.0, 20.0, -10.0).unwrap();
        assert!((result - 310.0).abs() < 1e-9, "kreeg {result}");
    }
}
