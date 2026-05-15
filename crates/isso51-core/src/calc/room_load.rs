//! Room-level heat loss orchestrator.
//! ISSO 51 Chapter 4 — combines all calculation modules for a single room.

use crate::error::Result;
use crate::model::building::Building;
use crate::model::climate::DesignConditions;
use crate::model::enums::{BoundaryType, InfiltrationMethod, VerticalPosition};
use crate::model::room::Room;
use crate::model::ventilation::VentilationConfig;
use crate::result::{
    HeatingUpResult, InfiltrationResult, RoomResult, SystemLossResult, TransmissionResult,
    VentilationResult,
};
use crate::formulas;
use crate::tables;

use super::{heating_up, infiltration, quadratic_sum, system_losses, transmission, ventilation};

/// Height above which rooms are considered "tall" (vides, double-height
/// spaces) for the ISSO 51 Table 2.12 voetnoot 2 Δθ₁ correction.
/// At or below this height the correction factor is 1.0 (tabulated
/// values are calibrated for standard room heights of ~2.6–3.0 m).
const HEIGHT_CORRECTION_THRESHOLD_M: f64 = 4.0;

/// Δθ₁ height correction factor per ISSO 51 Table 2.12 voetnoot 2:
/// *"Bij toepassing van vides etc. waardoor een grotere hoogte ontstaat
/// moet de waarde van Δθ₁ worden vermenigvuldigd met h/4 waarbij h de
/// totale hoogte [m] is."*
///
/// For rooms up to and including 4.0 m the factor is 1.0 (tabulated
/// values apply as-is). Above 4.0 m the factor scales linearly with
/// `h / 4`, so a 6 m vide gets factor 1.5 and an 8 m atrium gets 2.0.
///
/// # Arguments
/// * `height_m` - Room height in metres.
///
/// # Returns
/// Multiplier to apply to the tabulated Δθ₁ value.
fn height_factor(height_m: f64) -> f64 {
    if height_m > HEIGHT_CORRECTION_THRESHOLD_M {
        height_m / HEIGHT_CORRECTION_THRESHOLD_M
    } else {
        1.0
    }
}

/// Calculate the complete heat loss for a single room.
///
/// # Arguments
/// * `room` - The room to calculate
/// * `all_rooms` - Full project room list, needed for adjacent-room temperature lookup
/// * `building` - Building-level properties
/// * `climate` - Design conditions (temperatures)
/// * `vent_config` - Ventilation system configuration
/// * `main_room_hu_pct` - Heating-up percentage from main room (None for main room)
/// * `use_high_delta_v` - Whether Ū > 0.5 (true) or Ū ≤ 0.5 (false) for Δθ_v selection
///
/// # Returns
/// Complete RoomResult with all heat loss components.
#[allow(clippy::too_many_arguments)]
pub fn calculate_room(
    room: &Room,
    all_rooms: &[Room],
    building: &Building,
    climate: &DesignConditions,
    vent_config: &VentilationConfig,
    main_room_hu_pct: Option<f64>,
    use_high_delta_v: bool,
) -> Result<RoomResult> {
    let theta_i = room.design_temperature();
    let theta_e = climate.theta_e;
    let theta_b = climate.theta_b_residential;
    let theta_water = climate.theta_water;
    let c_z = building.security_class.factor();

    // Get Δθ corrections from the heating system table
    let dt = tables::temperature::delta_theta(room.heating_system);
    let delta_1 = dt.delta_1 * height_factor(room.height);
    let delta_2 = dt.delta_2;

    // --- Transmission ---
    let h_t = transmission::calculate_all_h_t(
        &room.constructions,
        all_rooms,
        theta_i,
        theta_e,
        theta_b,
        theta_water,
        c_z,
        delta_1,
        delta_2,
    );
    let h_t_ie = h_t.h_t_ie;
    let h_t_ia = h_t.h_t_ia;
    let h_t_io = h_t.h_t_io;
    let h_t_ib = h_t.h_t_ib;
    let h_t_ig = h_t.h_t_ig;
    let h_t_iw = h_t.h_t_iw;

    let h_t_total = h_t_ie + h_t_ia + h_t_io + h_t_ib + h_t_ig + h_t_iw;
    let phi_t = transmission::phi_transmission(h_t_total, theta_i, theta_e);

    // --- Infiltration ---
    let q_i = match building.infiltration_method {
        InfiltrationMethod::PerExteriorArea => {
            // ISSO 51:2023 Table 4.3: q_i = qi_spec × ΣA_exterior
            let qi_spec = tables::infiltration::qi_spec_per_exterior_area(building.qv10);
            let total_exterior_area: f64 = room
                .constructions
                .iter()
                .filter(|c| c.boundary_type == BoundaryType::Exterior)
                .map(|c| c.area)
                .sum();
            infiltration::infiltration_flow_rate(qi_spec, total_exterior_area)
        }
        InfiltrationMethod::PerFloorArea => {
            // ISSO 51:2024 Table 2.8: q_i = qi_spec_floor × A_floor
            let qi_spec = tables::infiltration::qi_spec_per_floor_area(building.qv10);
            qi_spec * room.floor_area
        }
        // Norm-conforme infiltratie-keten (ISSO 51:2023 Tabel 2.8 +
        // NTA 8800 Tabel 11.13/11.14 + NEN 8088-1 Tabel 10 + power-law).
        // Zie `calc/infiltration.rs::qi_norm_method` voor de formule en
        // `docs/2026-05-12-vabi-infiltratie-keten-reproductie.md` voor de
        // Vabi-fit Δp = 3.14 Pa. Building-level `qi` wordt naar rato van
        // `A_g_room / A_g_total` aan de kamer toegekend.
        InfiltrationMethod::VabiCompat
        | InfiltrationMethod::Nta8800Strict
        | InfiltrationMethod::MeasuredQv10 => {
            let qi_building =
                infiltration::compute_norm_qi(building, vent_config.system_type)?;
            // Defensieve guards tegen division-by-zero en negatieve floor areas.
            let a_g_total = building.total_floor_area.max(0.0);
            if a_g_total > 0.0 && room.floor_area > 0.0 {
                qi_building * (room.floor_area / a_g_total)
            } else {
                0.0
            }
        }
    };
    let h_i = infiltration::h_infiltration(q_i);
    let z_i = 1.0; // Erratum: z_i tables removed, default to 1.0
    let phi_i = infiltration::phi_infiltration(h_i, z_i, theta_i, theta_e);

    // --- Ventilation ---
    let theta_t = if let Some(t) = room.supply_air_temperature {
        t
    } else {
        vent_config.effective_supply_temperature(theta_e)
    };

    // Determine Δθ_v (ventilation temperature correction) based on Ū
    let delta_v = if use_high_delta_v { dt.delta_v_high } else { dt.delta_v_low };

    let (h_v, fv, vent_norm_refs) =
        if room.fraction_outside_air < 1.0 && room.fraction_outside_air > 0.0 {
            // Mixed air supply (formule 4.7)
            let f_v1 = ventilation::f_v(theta_i, theta_e, theta_t, delta_v);
            let theta_a = room.internal_air_temperature.unwrap_or(theta_i);
            let f_v2 =
                ventilation::f_v_adjacent(theta_i, theta_e, theta_a, delta_v);
            let h = ventilation::h_ventilation_mixed(
                room.effective_ventilation_rate(),
                room.fraction_outside_air,
                f_v1,
                f_v2,
            );
            let fv_eff = if room.effective_ventilation_rate() > 0.0 {
                h / (1.2 * room.effective_ventilation_rate())
            } else {
                0.0
            };
            (
                h,
                fv_eff,
                vec![
                    formulas::ISSO_51_2023_FORMULE4_7_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE4_6A_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE4_6B_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE3_3_ERRATUM,
                ],
            )
        } else if room.fraction_outside_air == 0.0 {
            // All air from internal source
            let theta_a = room.internal_air_temperature.unwrap_or(theta_i);
            let fv =
                ventilation::f_v_adjacent(theta_i, theta_e, theta_a, delta_v);
            let h = ventilation::h_ventilation(room.effective_ventilation_rate(), fv);
            (
                h,
                fv,
                vec![
                    formulas::ISSO_51_2023_FORMULE4_3_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE4_6B_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE3_3_ERRATUM,
                ],
            )
        } else {
            // All air from outside
            let fv = ventilation::f_v(theta_i, theta_e, theta_t, delta_v);
            let h = ventilation::h_ventilation(room.effective_ventilation_rate(), fv);
            (
                h,
                fv,
                vec![
                    formulas::ISSO_51_2023_FORMULE4_3_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE4_6A_ERRATUM,
                    formulas::ISSO_51_2023_FORMULE3_3_ERRATUM,
                ],
            )
        };

    let phi_v = ventilation::phi_ventilation(h_v, theta_i, theta_e);

    // ISSO 51:2024 / Vabi: Φ_vent = Φ_v (ventilation loss, independent of infiltration)
    // Both Φ_i (in basis) and Φ_vent (in extra/quadratic) are counted separately,
    // because mechanical ventilation and infiltration are non-simultaneous events.
    let phi_vent = phi_v.max(0.0);

    // --- Heating-up allowance ---
    let is_main_room = main_room_hu_pct.is_none();
    let accumulating_area: f64 = room
        .constructions
        .iter()
        .filter(|c| {
            matches!(
                c.material_type,
                crate::model::enums::MaterialType::Masonry
            )
        })
        .map(|c| c.area)
        .sum();

    let (phi_hu, f_rh) = heating_up::calculate_heating_up(
        building.building_type,
        building.warmup_time,
        accumulating_area,
        phi_t,
        phi_v,
        is_main_room,
        main_room_hu_pct,
    );

    // --- System losses (ISSO 51 §2.9) ---
    // Scan for embedded heating elements facing exterior/ground/adjacent building.
    // R_c estimated from U-value: R_c = 1/U - R_si - R_se.
    let mut has_floor_heat = false;
    let mut rc_floor = f64::MAX;
    let mut has_wall_heat = false;
    let mut rc_wall = f64::MAX;
    let mut has_ceil_heat = false;
    let mut rc_ceil = f64::MAX;

    for c in &room.constructions {
        if !c.has_embedded_heating {
            continue;
        }
        // ISSO 51 §2.9.1: system losses apply for embedded heating in
        // constructions facing exterior, ground, adjacent buildings, or water.
        // Water boundaries (woonboot use case) behave like ground: the floor
        // heating loses heat downward through the insulated floor slab.
        let exterior_facing = matches!(
            c.boundary_type,
            BoundaryType::Exterior | BoundaryType::Ground | BoundaryType::AdjacentBuilding | BoundaryType::Water
        );
        if !exterior_facing {
            continue;
        }
        match c.vertical_position {
            VerticalPosition::Floor => {
                has_floor_heat = true;
                let r_se = if matches!(c.boundary_type, BoundaryType::Ground | BoundaryType::Water) { 0.0 } else { 0.04 };
                rc_floor = rc_floor.min((1.0 / c.u_value - 0.17 - r_se).max(0.0));
            }
            VerticalPosition::Wall => {
                has_wall_heat = true;
                rc_wall = rc_wall.min((1.0 / c.u_value - 0.17).max(0.0));
            }
            VerticalPosition::Ceiling => {
                has_ceil_heat = true;
                rc_ceil = rc_ceil.min((1.0 / c.u_value - 0.14).max(0.0));
            }
        }
    }

    let f_floor = if has_floor_heat { system_losses::floor_heating_loss_fraction(rc_floor) } else { 0.0 };
    let f_wall = if has_wall_heat { system_losses::wall_heating_loss_fraction(rc_wall) } else { 0.0 };
    let f_ceil = if has_ceil_heat { system_losses::ceiling_heating_loss_fraction(rc_ceil) } else { 0.0 };
    let f_sys_total = f_floor + f_wall + f_ceil;

    // --- Basis & extra heat loss (without system losses) ---
    let phi_t_exterior = h_t_ie * (theta_i - theta_e);
    let phi_t_adjacent = h_t_ia * (theta_i - theta_e);
    let phi_t_unheated = h_t_io * (theta_i - theta_e);
    let phi_t_ground = h_t_ig * (theta_i - theta_e);
    // Water boundaries count in the basis (continuous, non-simultaneous with
    // ventilation peaks). The water-side surface is clamped to θ_water in
    // `h_t_water_element`, so multiplying by (θ_i - θ_e) recovers the
    // physical A·U·(θ_i - θ_water) flow.
    let phi_t_water = h_t_iw * (theta_i - theta_e);
    let phi_basis_no_sys =
        phi_t_exterior + phi_t_adjacent + phi_t_unheated + phi_t_ground + phi_t_water + phi_i;

    let phi_t_adj_building = h_t_ib * (theta_i - theta_e);
    let phi_extra = quadratic_sum::quadratic_sum(phi_vent, phi_t_adj_building, phi_hu);

    // Algebraic solution for circular dependency:
    // Φ_system = f × Φ_HL,i and Φ_HL,i = Φ_basis_no_sys + Φ_system + Φ_extra
    // → Φ_HL,i = (Φ_basis_no_sys + Φ_extra) / (1 - f)
    let (phi_system, phi_floor_loss, phi_wall_loss, phi_ceiling_loss, phi_basis, total) =
        if f_sys_total > 0.0 && f_sys_total < 1.0 {
            let total = (phi_basis_no_sys + phi_extra) / (1.0 - f_sys_total);
            let fl = f_floor * total;
            let wl = f_wall * total;
            let cl = f_ceil * total;
            let phi_sys = fl + wl + cl;
            (phi_sys, fl, wl, cl, phi_basis_no_sys + phi_sys, total)
        } else {
            (0.0, 0.0, 0.0, 0.0, phi_basis_no_sys, phi_basis_no_sys + phi_extra)
        };

    // --- Total ---
    let total = if room.clamp_positive { total.max(0.0) } else { total };

    Ok(RoomResult {
        room_id: room.id.clone(),
        room_name: room.name.clone(),
        theta_i,
        transmission: TransmissionResult {
            h_t_exterior: h_t_ie,
            h_t_adjacent_rooms: h_t_ia,
            h_t_unheated: h_t_io,
            h_t_adjacent_buildings: h_t_ib,
            h_t_ground: h_t_ig,
            h_t_water: h_t_iw,
            phi_t,
            norm_refs: vec![
                formulas::ISSO_51_2023_FORMULE4_2,
                formulas::ISSO_51_2023_FORMULE4_3A,
                formulas::ISSO_51_2023_FORMULE4_6,
                formulas::ISSO_51_2023_FORMULE4_14,
                formulas::ISSO_51_2023_FORMULE4_18,
            ],
        },
        infiltration: InfiltrationResult {
            h_i,
            z_i,
            phi_i,
            norm_refs: vec![
                formulas::ISSO_51_2023_FORMULE4_1_ERRATUM,
                formulas::ISSO_51_2023_FORMULE_E5_ERRATUM,
            ],
        },
        ventilation: VentilationResult {
            h_v,
            f_v: fv,
            q_v: room.effective_ventilation_rate(),
            q_v_minimum: room.bbl_minimum_ventilation_rate(),
            phi_v,
            phi_vent,
            norm_refs: vent_norm_refs,
        },
        heating_up: HeatingUpResult {
            phi_hu,
            f_rh,
            accumulating_area,
            norm_refs: vec![
                formulas::ISSO_51_2023_PARAG4_3,
                formulas::ISSO_51_2023_TABEL4_6,
            ],
        },
        system_losses: SystemLossResult {
            phi_floor_loss,
            phi_wall_loss,
            phi_ceiling_loss,
            phi_system_total: phi_system,
            norm_refs: if phi_system > 0.0 {
                vec![
                    formulas::ISSO_51_2023_TABEL2_17,
                    formulas::ISSO_51_2023_TABEL2_18_ERRATUM,
                    formulas::ISSO_51_2023_PARAG2_9_1_ERRATUM,
                ]
            } else {
                vec![]
            },
        },
        total_heat_loss: total,
        basis_heat_loss: phi_basis,
        extra_heat_loss: phi_extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ISSO 51 Table 2.12 voetnoot 2 — Δθ₁ height correction for vides.
    //
    // Standard rooms (≤ 4 m) keep the tabulated value; taller rooms get
    // a linear h/4 multiplier. These regression tests cover the hotfix
    // from 2026-04-10 (commits 960a70f + 804fb30) that ship the
    // correction without prior unit coverage.

    /// Standard room height (2.6 m) must yield factor 1.0.
    #[test]
    fn test_height_factor_standard_room() {
        assert_eq!(height_factor(2.6), 1.0);
    }

    /// Break-even at exactly 4.0 m: the conditional is strictly `>`,
    /// so 4.0 m is still in the unit-factor regime.
    #[test]
    fn test_height_factor_break_even_at_4m() {
        assert_eq!(height_factor(4.0), 1.0);
    }

    /// 6.0 m vide → factor 1.5 (6 / 4).
    #[test]
    fn test_height_factor_6m_vide() {
        assert!(
            (height_factor(6.0) - 1.5).abs() < 1e-12,
            "height_factor(6.0) = {}, expected 1.5",
            height_factor(6.0)
        );
    }

    /// 8.0 m atrium → factor 2.0 (8 / 4).
    #[test]
    fn test_height_factor_8m_atrium() {
        assert!(
            (height_factor(8.0) - 2.0).abs() < 1e-12,
            "height_factor(8.0) = {}, expected 2.0",
            height_factor(8.0)
        );
    }

    /// Just above threshold (4.1 m) — regression guard for the strictly
    /// `>` condition. 4.1 / 4 = 1.025.
    #[test]
    fn test_height_factor_just_above_threshold() {
        let f = height_factor(4.1);
        assert!(
            (f - 1.025).abs() < 1e-12,
            "height_factor(4.1) = {f}, expected 1.025"
        );
    }

    /// Very low / sub-standard heights (1.5 m crawlspace) must also
    /// clamp to 1.0 — the correction never makes Δθ₁ smaller.
    #[test]
    fn test_height_factor_below_standard() {
        assert_eq!(height_factor(1.5), 1.0);
    }

    /// Tabulated Δθ₁ × height_factor must match the expected product
    /// for a RadiatorLt vide. This is the composition test that
    /// mirrors what `calculate_room` does in its hot path.
    #[test]
    fn test_height_factor_applied_to_radiator_lt_delta_1() {
        use crate::model::enums::HeatingSystem;

        let dt = tables::temperature::delta_theta(HeatingSystem::RadiatorLt);
        // RadiatorLt Δθ₁ = 2.0 per Table 2.12
        assert_eq!(dt.delta_1, 2.0);

        // 6 m vide → factor 1.5 → corrected Δθ₁ = 3.0
        let corrected = dt.delta_1 * height_factor(6.0);
        assert!(
            (corrected - 3.0).abs() < 1e-12,
            "corrected Δθ₁ = {corrected}, expected 3.0"
        );

        // 8 m atrium → factor 2.0 → corrected Δθ₁ = 4.0
        let corrected_8 = dt.delta_1 * height_factor(8.0);
        assert!(
            (corrected_8 - 4.0).abs() < 1e-12,
            "corrected Δθ₁ at 8 m = {corrected_8}, expected 4.0"
        );

        // Standard room → factor 1.0 → tabulated value passes through.
        let corrected_std = dt.delta_1 * height_factor(2.6);
        assert_eq!(corrected_std, 2.0);
    }
}
