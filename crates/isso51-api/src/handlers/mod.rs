//! Request handlers for the ISSO 51 API.

mod beng;
mod calculation;
mod cloud;
mod cooling;
mod ifc_import;
mod ifcx;
mod projects;
mod report;
mod thermal_import;
mod user;

pub use beng::beng_calculate;
pub use calculation::{calculate, calculate_v2, get_schema, health, list_schemas};
pub use cooling::{simplified_cooling, tojuli_calculate};
pub use cloud::{
    cloud_list_calculations, cloud_list_models, cloud_list_projects, cloud_save_calculation,
    cloud_status,
};
pub use ifc_import::import_ifc;
pub use ifcx::calculate_ifcx_handler;
pub use projects::{
    calculate_and_save, create_project, delete_project, get_project, list_projects,
    update_project,
};
pub use report::generate_report;
pub use thermal_import::thermal_import_handler;
pub use user::get_profile;
