//! Room-level heat loss orchestrator.
//! ISSO 51 Chapter 4 — combines all calculation modules for a single room.

use crate::error::Result;
use crate::model::building::Building;
use crate::model::climate::DesignConditions;
use crate::model::enums::BoundaryType;
use crate::model::room::Room;
use crate::model::ventilation::VentilationConfig;
use crate::result::{
    HeatingUpResult, InfiltrationResult, RoomResult, SystemLossResult, TransmissionResult,
    VentilationResult,
};
use crate::formulas;
use crate::tables;

use super::{heating_up, infiltration, quadratic_sum, transmission, ventilation};

/// Calculate the complete heat loss for a single room.
///
/// # Arguments
/// * `room` - The room to calculate
/// * `building` - Building-level properties
/// * `climate` - Design conditions (temperatures)
/// * `vent_config` - Ventilation system configuration
/// * `main_room_hu_pct` - Heating-up percentage from main room (None for main room)
///
/// # Returns
/// Complete RoomResult with all heat loss components.
pub fn calculate_room(
    room: &Room,
    building: &Building,
    climate: &DesignConditions,
    vent_config: &VentilationConfig,
    main_room_hu_pct: Option<f64>,
) -> Result<RoomResult> {
    let theta_i = room.design_temperature();
    let theta_e = climate.theta_e;
    let theta_b = climate.theta_b_residential;
    let c_z = building.security_class.factor();

    // Get Δθ corrections from the heating system table
    let dt = tables::temperature::delta_theta(room.heating_system);
    let delta_1 = dt.delta_1;
    let delta_2 = dt.delta_2;

    // --- Transmission ---
    let (h_t_ie, h_t_ia, h_t_io, h_t_ib, h_t_ig) = transmission::calculate_all_h_t(
        &room.constructions,
        theta_i,
        theta_e,
        theta_b,
        c_z,
        delta_1,
        delta_2,
    );

    let h_t_total = h_t_ie + h_t_ia + h_t_io + h_t_ib + h_t_ig;
    let phi_t = transmission::phi_transmission(h_t_total, theta_i, theta_e);

    // --- Infiltration ---
    let qi_spec = tables::infiltration::qi_spec_per_exterior_area(building.qv10);
    let total_exterior_area: f64 = room
        .constructions
        .iter()
        .filter(|c| c.boundary_type == BoundaryType::Exterior)
        .map(|c| c.area)
        .sum();

    let q_i = infiltration::infiltration_flow_rate(qi_spec, total_exterior_area);
    let h_i = infiltration::h_infiltration(q_i);
    let z_i = 1.0; // Erratum: z_i tables removed, default to 1.0
    let phi_i = infiltration::phi_infiltration(h_i, z_i, theta_i, theta_e);

    // --- Ventilation ---
    let theta_t = if let Some(t) = room.supply_air_temperature {
        t
    } else {
        vent_config.effective_supply_temperature(theta_e)
    };

    // Determine Δθ_v (ventilation temperature correction)
    // For simplicity, use delta_v_high (Ū > 0.5) as default
    let delta_v = dt.delta_v_high;

    let (h_v, fv, vent_norm_refs) =
        if room.fraction_outside_air < 1.0 && room.fraction_outside_air > 0.0 {
            // Mixed air supply (formule 4.7)
            let f_v1 = ventilation::f_v(theta_i, theta_e, theta_t, delta_v);
            let theta_a = room.internal_air_temperature.unwrap_or(theta_i);
            let f_v2 =
                ventilation::f_v_adjacent(theta_i, theta_e, theta_a, delta_v);
            let h = ventilation::h_ventilation_mixed(
                room.ventilation_rate,
                room.fraction_outside_air,
                f_v1,
                f_v2,
            );
            let fv_eff = if room.ventilation_rate > 0.0 {
                h / (1.2 * room.ventilation_rate)
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
            let h = ventilation::h_ventilation(room.ventilation_rate, fv);
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
            let h = ventilation::h_ventilation(room.ventilation_rate, fv);
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

    // ISSO_51_2023_FORMULE3_3_ERRATUM: Φ_vent = Φ_v - Φ_i
    // Φ_vent is used in the quadratic sum (non-simultaneous)
    let phi_vent = (phi_v - phi_i).max(0.0);

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

    // --- System losses ---
    // For now, assume no embedded heating (will be expanded later)
    let phi_system = 0.0;

    // --- Basis heat loss ---
    // Φ_basis = Φ_T,ie + Φ_T,io + Φ_T,ig + Φ_i + Φ_system
    let phi_t_exterior = h_t_ie * (theta_i - theta_e);
    let phi_t_unheated = h_t_io * (theta_i - theta_e);
    let phi_t_ground = h_t_ig * (theta_i - theta_e);
    let phi_basis = phi_t_exterior + phi_t_unheated + phi_t_ground + phi_i + phi_system;

    // --- Non-simultaneous losses (quadratic sum) ---
    let phi_t_adj_building = h_t_ib * (theta_i - theta_e);
    let phi_extra = quadratic_sum::quadratic_sum(phi_vent, phi_t_adj_building, phi_hu);

    // --- Total ---
    let total = quadratic_sum::total_heat_loss(phi_basis, phi_extra);
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
            q_v: room.ventilation_rate,
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
            phi_floor_loss: 0.0,
            phi_wall_loss: 0.0,
            phi_ceiling_loss: 0.0,
            phi_system_total: phi_system,
            norm_refs: vec![],
        },
        total_heat_loss: total,
        basis_heat_loss: phi_basis,
        extra_heat_loss: phi_extra,
    })
}
