//! Lookup tables for ISSO 53 calculations.
//!
//! TODO batch 2 — submodules with literal table data from ISSO publication 53:
//! - `temperature` (tabel 2.2 ontwerpbinnentemperatuur)
//! - `thermal_bridge` (tabel 3.1 ΔU_TB)
//! - `thermal_mass` (tabel 2.4 c_eff per zwaarte)
//! - `adjacent_unheated` (tabel 4.2 f_k onverwarmde ruimten)
//! - `ground_params` (tabel 4.3 a/b/c/n/d voor U_equiv)
//! - `infiltration` (tabel 4.5 q_is, tabel 4.9 q_i,spec,reken)
//! - `building_type` (tabel 4.6 f_type, tabel 4.8 f_typ)
//! - `ventilation_system` (tabel 4.7 f_inf)
//! - `ventilation_requirements` (tabel 4.10 Bouwbesluit eisen)
//! - `occupancy` (tabel 4.11 defaults pers/m²)
//! - `source_fraction` (tabel 5.1 z gebouwniveau)
