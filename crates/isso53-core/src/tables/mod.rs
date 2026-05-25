//! Lookup tables for ISSO 53 calculations.
//!
//! Pure data — geen rekenlogica. Elke submodule bevat de letterlijke
//! tabeldata uit ISSO-publicatie 53 (2016) met een doc comment dat het
//! tabelnummer en de PDF-pagina vermeldt.
//!
//! Submodules:
//! - [`temperature`] — tabel 2.2 ontwerpbinnentemperatuur θ_i (PDF p.20);
//! - [`thermal_bridge`] — tabel 3.1 thermische-bruggen-toeslag ΔU_TB (PDF p.28);
//! - [`thermal_mass`] — tabel 2.4 specifieke opslagcapaciteit c_eff (PDF p.24);
//! - [`adjacent_unheated`] — tabel 4.2 correctiefactor f_k (PDF p.41-42);
//! - [`ground_params`] — tabel 4.3 parameters voor U_equiv (PDF p.44);
//! - [`infiltration`] — tabel 4.5 q_is + tabel 4.9 q_i,spec,reken (PDF p.45-47);
//! - [`building_type`] — tabel 4.6 f_type + tabel 4.8 f_typ (PDF p.46-47);
//! - [`ventilation_system`] — tabel 4.7 f_inf (PDF p.46-47);
//! - [`ventilation_requirements`] — tabel 4.10 ventilatie-eisen (PDF p.48-50);
//! - [`occupancy`] — tabel 4.11 default bezettingsdichtheid (PDF p.51);
//! - [`source_fraction`] — tabel 5.1 infiltratie-fractie z (PDF p.56-57).

pub mod adjacent_unheated;
pub mod building_type;
pub mod ground_params;
pub mod infiltration;
pub mod nen8088;
pub mod occupancy;
pub mod source_fraction;
pub mod temperature;
pub mod temperature_stratification;
pub mod thermal_bridge;
pub mod thermal_mass;
pub mod ventilation_requirements;
pub mod ventilation_system;

// Re-exports — convenience access to the lookup functions and data types.
pub use adjacent_unheated::f_k;
pub use building_type::{f_type, f_typ};
pub use ground_params::{
    ground_params, GroundParams, GroundSurfaceKind, B_PRIME_MAX, B_PRIME_MIN, U_EQUIV_MIN,
    Z_DEPTH_MAX,
};
pub use infiltration::{
    q_i_spec_reken, q_is_known, q_is_known_from_values, BuildingHeightClass, Qv10Class,
    QIS_TABLE_4_5,
};
pub use occupancy::{default_occupancy, default_occupancy_simple, OccupancyContext};
pub use source_fraction::{source_fraction_z, SourceZoneConfig};
pub use temperature::{
    design_indoor_temperature, design_indoor_temperature_shell, TEMPERATURE_IS_EXTERIOR,
};
pub use temperature_stratification::delta_theta_2;
pub use thermal_bridge::{delta_u_tb, ThermalBridgeSituation, DELTA_U_TB_DEFAULT};
pub use thermal_mass::c_eff;
pub use ventilation_requirements::{
    requirement, requirement_by_description, ventilation_rate_per_person, VentilationRequirement,
    KITCHEN_VENTILATION_DM3_S_M2, SHOWER_EXTRACT_MIN_DM3_S, TOILET_EXTRACT_MIN_DM3_S,
    VENTILATION_REQUIREMENTS_4_10,
};
pub use ventilation_system::f_inf;
pub use nen8088::{f_type_nen8088, f_inf_nen8088, f_jaar_nta8800};
