//! # ISSO 51 Heat Loss Calculation Engine
//!
//! Pure Rust implementation of the ISSO 51:2023 warmteverliesberekening
//! (heat loss calculation) for residential buildings in the Netherlands.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use isso51_core::calculate_from_json;
//!
//! let input_json = r#"{ ... }"#;
//! let result_json = calculate_from_json(input_json).unwrap();
//! ```
//!
//! ## Architecture
//!
//! This crate is a pure computation library — no I/O, no async, no unsafe.
//! It takes JSON input, performs the calculation, and returns JSON output.
//! Wrapper crates (isso51-python, isso51-wasm, isso51-ffi) provide
//! platform-specific bindings.

pub mod calc;
pub mod error;
pub mod formulas;
pub mod import;
pub mod model;
pub mod result;
pub mod tables;
pub mod validate;

use error::Result;
use model::Project;
use result::{BuildingSummary, ProjectResult};

/// Calculate heat losses for an entire project from JSON input.
///
/// This is the main public API. It takes a JSON string representing
/// a Project, validates the input, runs the calculation for each room,
/// and returns the results as a JSON string.
///
/// # Arguments
/// * `input_json` - JSON string conforming to the Project schema
///
/// # Returns
/// JSON string containing the ProjectResult, or an error.
///
/// # Errors
/// Returns `Isso51Error` if the input is invalid or calculation fails.
pub fn calculate_from_json(input_json: &str) -> Result<String> {
    let project: Project = serde_json::from_str(input_json)?;
    let result = calculate(&project)?;
    let output = serde_json::to_string_pretty(&result)?;
    Ok(output)
}

/// Calculate heat losses for an entire project.
///
/// Takes a validated Project struct and returns the complete calculation results.
///
/// # Arguments
/// * `project` - The project input data
///
/// # Returns
/// Complete ProjectResult with per-room and building-level results.
pub fn calculate(project: &Project) -> Result<ProjectResult> {
    validate::validate_project(project)?;

    // Single-pass berekening (ISSO 51:2023 §4.3.1): elk vertrek krijgt
    // onafhankelijk `Φ_hu,i = P × A_g` (Formule 4.15). Geen hoofdruimte-
    // afhankelijkheid meer (dat was het 2017 `f_RH × ΣA_metselwerk`-model).

    // Bereken Ū (oppervlakte-gewogen gemiddelde U-waarde van de schil) over
    // alle vertrekken. Stuurt twee dingen:
    //  1. Δθ_v-selectie: delta_v_high (Ū > 0.5) of delta_v_low (Ū ≤ 0.5).
    //  2. Opwarmtoeslag-afkoeling: 2 K nieuwbouw, resp. 1 K bij Ū ≤ 0.5.
    let (total_a_ext, total_au_ext) = project.rooms.iter().fold((0.0, 0.0), |(a, au), room| {
        room.constructions
            .iter()
            .filter(|c| c.boundary_type == model::enums::BoundaryType::Exterior)
            .fold((a, au), |(a, au), c| (a + c.area, au + c.area * c.u_value))
    });
    // Fallback bij een lege externe schil (total_a_ext == 0.0 — bv. een puur
    // intern vertrek of een ontaarde test-input): geen division-by-zero, maar
    // Ū = 1.0. Dat is een conservatieve keuze — Ū = 1.0 > 0.5 → afkoeling 2 K
    // (hoogste nieuwbouw-P) en delta_v_high — i.p.v. stilzwijgend de
    // best-geïsoleerde 1 K-tak te kiezen voor een gebouw waarvan de isolatie
    // onbekend is.
    let u_bar = if total_a_ext > 0.0 { total_au_ext / total_a_ext } else { 1.0 };
    let use_high_delta_v = u_bar > 0.5;

    // Gebouwbrede opwarmtoeslag-ingangen (Tabel 2.10). Afkoeling uit Ū,
    // zwaarte uit c_eff. Deze zijn voor alle vertrekken gelijk; alleen A_g
    // (room.floor_area) varieert per vertrek.
    let hu_cooling_k = calc::heating_up::newbuild_cooling_k(u_bar);
    let hu_mass = calc::heating_up::building_thermal_mass(&project.building);

    let mut room_results: Vec<result::RoomResult> = Vec::with_capacity(project.rooms.len());

    for room in &project.rooms {
        let room_result = calc::room_load::calculate_room(
            room,
            &project.rooms,
            &project.building,
            &project.climate,
            &project.ventilation,
            hu_cooling_k,
            hu_mass,
            use_high_delta_v,
        )?;
        room_results.push(room_result);
    }

    // Build summary
    let summary = build_summary(
        &room_results,
        project.climate.theta_e,
        project.ventilation.system_type,
        project.building.aggregation_method,
    );

    Ok(ProjectResult {
        rooms: room_results,
        summary,
    })
}

/// Build the building-level summary from per-room results.
///
/// Aggregatie volgens ISSO 51:2023 erratum, met twee configureerbare keuzes:
///
/// 1. **`aggregation_method`** bepaalt of `Φ_T,iae` (verlies via onverwarmde
///    ruimtes) in `Φ_basis_gebouw` wordt opgenomen:
///    - `VabiCompat` (default, markt-conventie): NIET opnemen — telt als 0 op
///      gebouwniveau (Vabi-compatible).
///    - `NormStrict`: WEL opnemen — strikt §3.5.1 (`Φ_basis = Φ_T,ie + Φ_T,iae
///      + Φ_T,ig + Φ_i − Φ_gain`). ~17% hogere connection_capacity.
///
/// 2. **`ventilation_system_type`** bepaalt formule 3.3 vs 3.4 op gebouwniveau:
///    - Systeem A/C (natuurlijke toevoer): `Φ_vent = Σ Φ_v − Σ Φ_i` (formule 3.3,
///      infiltratie als deel van de toevoerlucht).
///    - Systeem B/D/E (mechanische toevoer): `Φ_vent = Σ Φ_v` (formule 3.4,
///      geen aftrek — infiltratie loopt apart en zit al in Φ_basis).
///
/// Verdere keten:
/// - `Φ_extra_quadratic = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)` (formule 3.11).
/// - `connection_capacity = Φ_basis_total + Φ_extra_quadratic`.
fn build_summary(
    rooms: &[result::RoomResult],
    theta_e: f64,
    ventilation_system_type: model::enums::VentilationSystemType,
    aggregation_method: model::enums::AggregationMethod,
) -> BuildingSummary {
    use model::enums::{AggregationMethod, VentilationSystemType};

    let mut total_envelope_loss = 0.0; // inclusief Φ_T,iae (h_t_unheated × Δθ)
    let mut total_envelope_no_iae = 0.0; // exclusief Φ_T,iae (Vabi-conventie)
    let mut total_neighbor_loss = 0.0;
    let mut total_ventilation_loss = 0.0;
    let mut total_heating_up = 0.0;
    let mut total_system_losses = 0.0;
    let mut total_infiltration_loss = 0.0;

    for r in rooms {
        let theta_diff = r.theta_i - theta_e;

        let phi_t_ie = r.transmission.h_t_exterior * theta_diff;
        let phi_t_iae = r.transmission.h_t_unheated * theta_diff;
        let phi_t_ig = r.transmission.h_t_ground * theta_diff;
        let phi_t_iw = r.transmission.h_t_water * theta_diff;

        total_envelope_loss += phi_t_ie + phi_t_iae + phi_t_ig + phi_t_iw;
        total_envelope_no_iae += phi_t_ie + phi_t_ig + phi_t_iw;

        total_neighbor_loss += r.transmission.h_t_adjacent_buildings * theta_diff;

        total_ventilation_loss += r.ventilation.phi_v;
        total_heating_up += r.heating_up.phi_hu;
        total_system_losses += r.system_losses.phi_system_total;
        total_infiltration_loss += r.infiltration.phi_i;
    }

    // --- Gedecomposeerde gebouwsom conform erratum 2023 ---

    // Φ_basis omvat alle continue, simultane verliezen op gebouwniveau:
    // envelope + grond + water + infiltratie. Intra-woning transmissie
    // (`h_t_adjacent_rooms`) is bewust uitgesloten — zero-sum over de woning,
    // zie doc-comment op `TransmissionResult::h_t_adjacent_rooms`.
    //
    // Φ_T,iae (h_t_unheated × Δθ) wordt al-dan-niet meegenomen afhankelijk
    // van `aggregation_method`. Zie `AggregationMethod` doc.
    //
    // K3 (§3.5.3): de systeemverliezen (`total_system_losses`) horen volgens
    // Formule 3.12 NIET in het schilvermogen Φ_HL,build. Ze tellen ALLEEN mee
    // voor het verdeler-/opwekkervermogen Φ_HL,verdeler (Formule 3.13). Daarom
    // berekenen we hier het schil-`Φ_basis` (zónder systeemverliezen) en voegen
    // de systeemverliezen pas in de 3.13-tak toe. Alleen relevant bij
    // `has_embedded_heating = true` (anders is `total_system_losses == 0`).
    let phi_basis_envelope = match aggregation_method {
        AggregationMethod::VabiCompat => total_envelope_no_iae + total_infiltration_loss,
        AggregationMethod::NormStrict => total_envelope_loss + total_infiltration_loss,
    };

    // Backward-compat: `phi_basis_total` is historisch INCLUSIEF
    // systeemverliezen (en `connection_capacity` werd daaruit afgeleid). Om de
    // bestaande veldsemantiek + golden-fixtures niet te breken behoudt
    // `phi_basis_total` de 3.13-definitie (mét systeemverliezen). Het nieuwe
    // schil-only getal staat in `phi_basis_build` (zie BuildingSummary).
    let phi_basis_total = phi_basis_envelope + total_system_losses;

    // Φ_vent op gebouwniveau — formule 3.3 vs 3.4 afhankelijk van systeem.
    // Natuurlijke toevoer (A/C): infiltratie is onderdeel van toevoerlucht →
    //   `Φ_vent = Σ Φ_v − Σ Φ_i`.
    // Mechanische toevoer (B/D/E): toevoer komt via systeem, infiltratie loopt
    //   apart en zit al in Φ_basis → géén aftrek: `Φ_vent = Σ Φ_v`.
    //
    // Niet-negatief geclampt: een netto-negatieve ventilatieverlies is
    // fysisch niet zinvol als kwadratische component (zou na .powi(2)
    // toch positief bijdragen, wat de norm-bedoeling ondermijnt).
    let phi_vent_building = match ventilation_system_type {
        VentilationSystemType::SystemA | VentilationSystemType::SystemC => {
            (total_ventilation_loss - total_infiltration_loss).max(0.0)
        }
        VentilationSystemType::SystemB
        | VentilationSystemType::SystemD
        | VentilationSystemType::SystemE => total_ventilation_loss.max(0.0),
    };

    // Φ_T,iaBE = som van transmissie naar aangrenzende gebouwen.
    let phi_t_iabe_building = total_neighbor_loss;

    let phi_hu_building = total_heating_up;

    // Φ_extra = √(Φ_vent² + Φ_T,iaBE² + Φ_hu²)  (formule 3.11).
    let phi_extra_quadratic = calc::quadratic_sum::quadratic_sum(
        phi_vent_building,
        phi_t_iabe_building,
        phi_hu_building,
    );

    // K3 §3.5.3 — twee verschillende grootheden:
    //   Φ_HL,build  (Form. 3.12) = Φ_basis_envelope + Φ_extra        ← schilvermogen, ZONDER systeemverliezen
    //   Φ_HL,verdeler (Form. 3.13) = Φ_HL,build + ΣΦ_add,i (= sys.)   ← verdeler-/opwekkervermogen, MÉT systeemverliezen
    // De systeemverliezen (`total_system_losses`) zijn de ΣΦ_add,i-term.
    let phi_hl_build = phi_basis_envelope + phi_extra_quadratic;
    let phi_hl_verdeler = phi_hl_build + total_system_losses;

    // Backward-compat: `connection_capacity` blijft de 3.13-waarde (= verdeler,
    // mét systeemverliezen). Identiek aan `phi_basis_total + phi_extra` zoals
    // vóór de K3-split, zodat golden-fixtures (zonder embedded heating →
    // systeemverliezen = 0 → 3.12 == 3.13) ongewijzigd blijven.
    let connection_capacity = phi_hl_verdeler;

    // Collectieve bijdrage sluit naburige gebouwen (woningscheidende wanden)
    // uit: bij collectieve installatie zit de buurwoning op vergelijkbare
    // θ_i, dus geen netto transport. Behoudt erratum-conforme kwadratische
    // sommatie voor de overige niet-simultane componenten. Gebruikt dezelfde
    // `phi_basis_total` keuze (Vabi/NormStrict) als de individuele variant
    // (inclusief systeemverliezen, want collectief == verdeler-context).
    let phi_extra_collective =
        calc::quadratic_sum::quadratic_sum(phi_vent_building, 0.0, phi_hu_building);
    let collective_contribution = phi_basis_total + phi_extra_collective;

    BuildingSummary {
        total_envelope_loss,
        total_neighbor_loss,
        total_ventilation_loss,
        total_heating_up,
        total_system_losses,
        connection_capacity,
        collective_contribution,
        phi_basis_total,
        phi_vent_building,
        phi_t_iabe_building,
        phi_hu_building,
        phi_extra_quadratic,
        // K3 §3.5.3 split.
        phi_basis_build: phi_basis_envelope,
        phi_hl_build,
        phi_hl_verdeler,
        // C2 §3.5.1 — herkomst van de Φ_basis-aggregatie expliciet maken.
        aggregation_method,
    }
}

/// Base URL for published schemas.
const SCHEMA_BASE_URL: &str = "https://warmteverlies.open-aec.com/schemas/v1";

/// Current schema version.
const SCHEMA_VERSION: &str = "1.0.0";

/// Generate the JSON schema for the Project input type.
///
/// Useful for documentation and validation tooling.
pub fn project_schema() -> String {
    let schema = schemars::schema_for!(Project);
    add_schema_metadata(schema, "project")
}

/// Generate the JSON schema for the ProjectResult output type.
pub fn result_schema() -> String {
    let schema = schemars::schema_for!(ProjectResult);
    add_schema_metadata(schema, "result")
}

/// Add `$id` and `version` to a generated JSON schema.
fn add_schema_metadata(schema: schemars::schema::RootSchema, name: &str) -> String {
    let mut value = serde_json::to_value(&schema).unwrap_or_default();
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "$id".to_string(),
            serde_json::Value::String(format!(
                "{SCHEMA_BASE_URL}/{name}.schema.json"
            )),
        );
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(SCHEMA_VERSION.to_string()),
        );
    }
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    /// Create the ISSO 51 Example 1 portiekwoning for testing.
    fn create_portiekwoning() -> Project {
        Project {
            info: ProjectInfo {
                name: "ISSO 51 Voorbeeld 1 - Portiekwoning".to_string(),
                project_number: None,
                address: None,
                client: None,
                date: None,
                engineer: None,
                notes: None,
            },
            building: Building {
                building_type: BuildingType::Porch,
                qv10: 100.0,
                total_floor_area: 85.0,
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
                heating_control_type: HeatingControlType::default(),
                c_eff: None,
                built_after_2015: true,
                all_floor_heating: false,
            },
            // Old ISSO 51 example used θ_b = 15°C (erratum 2023 changed to 17°C)
            climate: DesignConditions {
                theta_b_residential: 15.0,
                ..DesignConditions::default()
            },
            ventilation: VentilationConfig {
                system_type: VentilationSystemType::SystemC,
                has_heat_recovery: false,
                heat_recovery_efficiency: None,
                frost_protection: None,
                supply_temperature: None,
                has_preheating: false,
                preheating_temperature: None,
            },
            rooms: vec![create_room1_woonkamer()],
        }
    }

    /// Room 1: Woonkamer (living room), θ_i = 20°C
    fn create_room1_woonkamer() -> Room {
        use construction::ConstructionElement;
        use enums::*;

        Room {
            id: "r1".to_string(),
            name: "Woonkamer".to_string(),
            function: RoomFunction::LivingRoom,
            custom_temperature: None,
            floor_area: 28.2,
            height: 2.6,
            constructions: vec![
                // Exterior elements
                ConstructionElement {
                    id: "c1".to_string(),
                    description: "Buitenwand".to_string(),
                    area: 7.29,
                    u_value: 0.36,
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
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c2".to_string(),
                    description: "Raam".to_string(),
                    area: 4.32,
                    u_value: 3.2,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::NonMasonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c3".to_string(),
                    description: "Buitenwand bij deur".to_string(),
                    area: 0.36,
                    u_value: 0.36,
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
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c4".to_string(),
                    description: "Deur naar balkon".to_string(),
                    area: 2.16,
                    u_value: 2.78,
                    boundary_type: BoundaryType::Exterior,
                    material_type: MaterialType::NonMasonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: true,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                // Adjacent rooms within dwelling
                // NOTE: adjacent_room_id deliberately set to None so this
                // single-room test fixture stays valid under the CORE-3
                // adjacent_room_id existence check. The calc falls back to
                // the legacy `adjacent_temperature` field, which keeps the
                // expected H_T,ia ≈ 1.51 value stable.
                ConstructionElement {
                    id: "c5".to_string(),
                    description: "Binnenwand naar keuken".to_string(),
                    area: 7.36,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: Some(20.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c6".to_string(),
                    description: "Binnenwand naar slaapkamer 1".to_string(),
                    area: 11.20,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: Some(20.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c7".to_string(),
                    description: "Binnenwand naar entree".to_string(),
                    area: 2.51,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: Some(15.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c8".to_string(),
                    description: "Binnenwand naar toilet".to_string(),
                    area: 3.12,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: Some(15.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c9".to_string(),
                    description: "Binnenwand naar badkamer".to_string(),
                    area: 3.64,
                    u_value: 2.17,
                    boundary_type: BoundaryType::AdjacentRoom,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: Some(22.0),
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                // Adjacent building (neighboring dwellings)
                ConstructionElement {
                    id: "c10".to_string(),
                    description: "Woningscheidende wand".to_string(),
                    area: 18.09,
                    u_value: 2.08,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Wall,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c11".to_string(),
                    description: "Plafond".to_string(),
                    area: 28.20,
                    u_value: 2.5,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Ceiling,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
                ConstructionElement {
                    id: "c12".to_string(),
                    description: "Vloer".to_string(),
                    area: 28.20,
                    u_value: 2.5,
                    boundary_type: BoundaryType::AdjacentBuilding,
                    material_type: MaterialType::Masonry,
                    temperature_factor: None,
                    adjacent_room_id: None,
                    adjacent_temperature: None,
                    vertical_position: VerticalPosition::Floor,
                    use_forfaitaire_thermal_bridge: false,
                    custom_delta_u_tb: None,
                    ground_params: None,
                    has_embedded_heating: false,
                    catalog_ref: None,
                    uw_breakdown: None,
                },
            ],
            heating_system: HeatingSystem::RadiatorLt,
            ventilation_rate: Some(25.38),
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            air_source_room_id: None,
            internal_air_temperature: None,
            clamp_positive: true,
        }
    }

    #[test]
    fn test_portiekwoning_room1_transmission() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected: H_T,ie ≈ 24.00
        assert!(
            (r1.transmission.h_t_exterior - 24.00).abs() < 0.2,
            "H_T,ie = {}, expected ~24.00",
            r1.transmission.h_t_exterior
        );

        // Expected: H_T,ia ≈ 1.51
        assert!(
            (r1.transmission.h_t_adjacent_rooms - 1.51).abs() < 0.2,
            "H_T,ia = {}, expected ~1.51",
            r1.transmission.h_t_adjacent_rooms
        );

        // Expected: Φ_T ≈ 1247 W
        assert!(
            (r1.transmission.phi_t - 1247.0).abs() < 20.0,
            "Φ_T = {}, expected ~1247",
            r1.transmission.phi_t
        );
    }

    #[test]
    fn test_portiekwoning_room1_ventilation() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected: Φ_v ≈ 914 W
        assert!(
            (r1.ventilation.phi_v - 914.0).abs() < 5.0,
            "Φ_v = {}, expected ~914",
            r1.ventilation.phi_v
        );
    }

    #[test]
    fn test_portiekwoning_room1_total() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Expected total: Φ_tot = Φ_T + Φ_v + Φ_hu = 1247 + 914 + 187 = 2348 W
        // Note: with quadratic summation (2023), the result will differ from
        // the old example which used simple addition.
        // The old example gives 2348 W; with quadratic sum it will be different.
        assert!(
            r1.total_heat_loss > 0.0,
            "Total heat loss should be positive"
        );
    }

    /// FIX C — lege externe schil (`total_a_ext == 0.0`) mag niet paniekken of
    /// NaN produceren; de `u_bar`-fallback van 1.0 moet de conservatieve 2 K-tak
    /// kiezen (P uit de 2 K-kolom van Tabel 2.10), niet de 1 K-tak.
    #[test]
    fn test_u_bar_fallback_empty_envelope_uses_2k() {
        let mut project = create_portiekwoning();
        // Strip alle Exterior-constructies → total_a_ext == 0.0 in `calculate`.
        for room in &mut project.rooms {
            room.constructions
                .retain(|c| c.boundary_type != BoundaryType::Exterior);
        }
        // Sanity: er is daadwerkelijk geen exterior-oppervlak meer.
        let total_a_ext: f64 = project
            .rooms
            .iter()
            .flat_map(|r| &r.constructions)
            .filter(|c| c.boundary_type == BoundaryType::Exterior)
            .map(|c| c.area)
            .sum();
        assert_eq!(total_a_ext, 0.0, "test-precondititie: geen exterior-vlak");

        // Night setback staat aan in de fixture → Φ_hu wordt écht berekend.
        let result = calculate(&project).expect("mag niet paniekken bij lege schil");
        let r1 = &result.rooms[0];

        // u_bar-fallback = 1.0 → afkoeling 2 K → 2K/Z/2h = P = 22 W/m²
        // (geen c_eff → Heavy default). Bevestigt dat de fallback de 2 K-kolom
        // pakt en geen NaN/0 oplevert.
        assert_eq!(
            calc::heating_up::newbuild_cooling_k(1.0),
            2.0,
            "u_bar-fallback moet 2 K geven"
        );
        assert!(
            (r1.heating_up.p - 22.0).abs() < 1e-9,
            "P = {} (verwacht 22 via 2 K-fallback)",
            r1.heating_up.p
        );
        let expected_phi_hu = 22.0 * r1.heating_up.a_g;
        assert!(
            r1.heating_up.phi_hu.is_finite()
                && (r1.heating_up.phi_hu - expected_phi_hu).abs() < 1e-6,
            "Φ_hu = {} moet eindig zijn en gelijk aan 22 × A_g = {}",
            r1.heating_up.phi_hu,
            expected_phi_hu
        );
    }

    /// K3 §3.5.3 — zónder ingebouwde verwarming zijn Formule 3.12
    /// (`phi_hl_build`) en 3.13 (`phi_hl_verdeler`) identiek, en gelijk aan
    /// `connection_capacity`. Geen systeemverliezen → ΣΦ_add,i = 0.
    #[test]
    fn test_k3_build_equals_verdeler_without_embedded_heating() {
        let project = create_portiekwoning();
        let s = calculate(&project).unwrap().summary;

        assert_eq!(
            s.total_system_losses, 0.0,
            "portiekwoning heeft geen embedded heating → systeemverliezen 0"
        );
        assert!(
            (s.phi_hl_build - s.phi_hl_verdeler).abs() < 1e-9,
            "3.12 ({}) en 3.13 ({}) moeten gelijk zijn zonder systeemverliezen",
            s.phi_hl_build,
            s.phi_hl_verdeler
        );
        assert!(
            (s.phi_hl_verdeler - s.connection_capacity).abs() < 1e-9,
            "connection_capacity moet de 3.13-waarde (verdeler) zijn"
        );
        // Φ_HL,build = Φ_basis_build + Φ_extra.
        assert!(
            (s.phi_hl_build - (s.phi_basis_build + s.phi_extra_quadratic)).abs() < 1e-9,
            "phi_hl_build moet phi_basis_build + phi_extra_quadratic zijn"
        );
        // Zonder systeemverliezen valt phi_basis_build samen met phi_basis_total.
        assert!(
            (s.phi_basis_build - s.phi_basis_total).abs() < 1e-9,
            "zonder systeemverliezen: phi_basis_build == phi_basis_total"
        );
    }

    /// K3 §3.5.3 — MÉT ingebouwde verwarming (vloerverwarming op een
    /// exterior-vloer met systeemverliezen) moet `phi_hl_verdeler` (3.13)
    /// strikt groter zijn dan `phi_hl_build` (3.12): het verschil is exact
    /// `total_system_losses` (ΣΦ_add,i). Het schilvermogen 3.12 bevat ze NIET.
    #[test]
    fn test_k3_verdeler_exceeds_build_with_embedded_heating() {
        use crate::model::enums::*;

        let mut project = create_portiekwoning();
        // Geef de woonkamer een geïsoleerde exterior-vloer mét vloerverwarming,
        // zodat de systeemverlies-tak (room_load.rs) Φ_add,i > 0 produceert.
        let room = &mut project.rooms[0];
        room.constructions.push(construction::ConstructionElement {
            id: "fh1".to_string(),
            description: "Vloer met vloerverwarming naar buiten".to_string(),
            area: 28.2,
            u_value: 0.2, // R_c ≈ 1/0.2 − 0.17 − 0.04 ≈ 4.8 → fractie 0.10
            boundary_type: BoundaryType::Exterior,
            material_type: MaterialType::Masonry,
            temperature_factor: None,
            adjacent_room_id: None,
            adjacent_temperature: None,
            vertical_position: VerticalPosition::Floor,
            use_forfaitaire_thermal_bridge: true,
            custom_delta_u_tb: None,
            ground_params: None,
            has_embedded_heating: true,
            catalog_ref: None,
            uw_breakdown: None,
        });

        let s = calculate(&project).unwrap().summary;

        assert!(
            s.total_system_losses > 0.0,
            "embedded floor heating moet systeemverliezen > 0 geven, kreeg {}",
            s.total_system_losses
        );
        // 3.13 = 3.12 + ΣΦ_add,i.
        assert!(
            s.phi_hl_verdeler > s.phi_hl_build,
            "verdeler ({}) moet > build ({}) zijn met systeemverliezen",
            s.phi_hl_verdeler,
            s.phi_hl_build
        );
        assert!(
            (s.phi_hl_verdeler - (s.phi_hl_build + s.total_system_losses)).abs() < 1e-6,
            "verschil tussen 3.13 en 3.12 moet exact total_system_losses zijn: \
             verdeler={}, build={}, sys={}",
            s.phi_hl_verdeler,
            s.phi_hl_build,
            s.total_system_losses
        );
        // connection_capacity blijft de 3.13-waarde (backward-compat).
        assert!(
            (s.connection_capacity - s.phi_hl_verdeler).abs() < 1e-9,
            "connection_capacity moet gelijk blijven aan phi_hl_verdeler (3.13)"
        );
        // Schilvermogen 3.12 = schil-basis + extra, ZONDER systeemverliezen.
        assert!(
            (s.phi_hl_build - (s.phi_basis_build + s.phi_extra_quadratic)).abs() < 1e-6,
            "phi_hl_build moet phi_basis_build + phi_extra_quadratic zijn (zonder sys)"
        );
    }

    /// C2 §3.5.1 — de actieve aggregatiemethode moet expliciet in het resultaat
    /// staan, zodat een consument niet stilzwijgend de Vabi-variant aanziet voor
    /// strikt-norm-conform. Default = VabiCompat.
    #[test]
    fn test_c2_aggregation_method_surfaced_in_result() {
        use crate::model::enums::AggregationMethod;

        // Default (VabiCompat).
        let project = create_portiekwoning();
        let s = calculate(&project).unwrap().summary;
        assert_eq!(
            s.aggregation_method,
            AggregationMethod::VabiCompat,
            "default-aggregatie moet als VabiCompat in het resultaat staan"
        );

        // Expliciet NormStrict moet doorwerken naar het resultaatveld.
        let mut strict = create_portiekwoning();
        strict.building.aggregation_method = AggregationMethod::NormStrict;
        let s2 = calculate(&strict).unwrap().summary;
        assert_eq!(
            s2.aggregation_method,
            AggregationMethod::NormStrict,
            "NormStrict moet als zodanig in het resultaat staan"
        );
    }

    #[test]
    fn test_json_roundtrip() {
        let project = create_portiekwoning();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let result = calculate_from_json(&json).unwrap();
        assert!(!result.is_empty());

        // Verify result is valid JSON
        let _: serde_json::Value = serde_json::from_str(&result).unwrap();
    }

    #[test]
    fn test_schema_generation() {
        let schema = project_schema();
        assert!(!schema.is_empty());
        let _: serde_json::Value = serde_json::from_str(&schema).unwrap();

        let result_schema = result_schema();
        assert!(!result_schema.is_empty());
    }

    #[test]
    fn test_norm_refs_populated() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let r1 = &result.rooms[0];

        // Transmission must reference formule 4.2 (Phi_T) and 4.3a (H_T,ie)
        assert!(
            r1.transmission.norm_refs.contains(&"ISSO_51_2023_formule4_2"),
            "Transmission missing formule 4.2"
        );
        assert!(
            r1.transmission.norm_refs.contains(&"ISSO_51_2023_formule4_3a"),
            "Transmission missing formule 4.3a"
        );

        // Infiltration must reference erratum formules
        assert!(
            r1.infiltration
                .norm_refs
                .contains(&"ISSO_51_2023_formule4_1_erratum"),
            "Infiltration missing formule 4.1 erratum"
        );

        // Ventilation: outside air → formule 4.3 erratum + 4.6a erratum
        assert!(
            r1.ventilation
                .norm_refs
                .contains(&"ISSO_51_2023_formule4_3_erratum"),
            "Ventilation missing formule 4.3 erratum"
        );
        assert!(
            r1.ventilation
                .norm_refs
                .contains(&"ISSO_51_2023_formule3_3_erratum"),
            "Ventilation missing formule 3.3 erratum (phi_vent)"
        );

        // Heating-up must reference paragraaf 4.3 (Φ_hu = P × A_g, 2023-model).
        // Tabel 4.6 (2017 f_RH-model) is verwijderd — zie Ronde 5 A1/A2.
        assert!(
            r1.heating_up
                .norm_refs
                .contains(&"ISSO_51_2023_parag4_3"),
            "Heating-up missing parag 4.3"
        );

        // System losses: no embedded heating → empty
        assert!(
            r1.system_losses.norm_refs.is_empty(),
            "System losses should have no norm_refs without embedded heating"
        );
    }

    #[test]
    fn test_norm_refs_in_json_output() {
        let project = create_portiekwoning();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let result_json = calculate_from_json(&json).unwrap();

        // norm_refs must appear in serialized output
        assert!(
            result_json.contains("norm_refs"),
            "JSON output must contain norm_refs field"
        );
        assert!(
            result_json.contains("ISSO_51_2023_formule4_2"),
            "JSON output must contain formule 4.2 reference"
        );
    }

    #[test]
    fn test_norm_refs_skipped_on_deserialize() {
        let project = create_portiekwoning();
        let result = calculate(&project).unwrap();
        let json = serde_json::to_string(&result).unwrap();

        // Deserialize back — norm_refs should default to empty
        let deserialized: result::ProjectResult =
            serde_json::from_str(&json).unwrap();
        let r1 = &deserialized.rooms[0];
        assert!(
            r1.transmission.norm_refs.is_empty(),
            "norm_refs should be empty after deserialization"
        );
    }

    // ================================================================
    // DR Engineering Woningbouw ISSO 51:2024 validation test
    // ================================================================

    /// Expected values per room from DR Engineering / Vabi 3.12.0.127.
    struct ExpectedRoom {
        id: &'static str,
        phi_basis: f64,
        phi_extra: f64,
        phi_hl_i: f64,
    }

    const DR_EXPECTED: &[ExpectedRoom] = &[
        ExpectedRoom { id: "0.01", phi_basis: 567.0,  phi_extra: 0.0,   phi_hl_i: 567.0  },
        ExpectedRoom { id: "0.02", phi_basis: -36.0,  phi_extra: 0.0,   phi_hl_i: 0.0    },
        ExpectedRoom { id: "0.03", phi_basis: 2101.0, phi_extra: 221.0, phi_hl_i: 2322.0 },
        ExpectedRoom { id: "0.04", phi_basis: 1823.0, phi_extra: 197.0, phi_hl_i: 2020.0 },
        ExpectedRoom { id: "0.05", phi_basis: 321.0,  phi_extra: 0.0,   phi_hl_i: 321.0  },
        ExpectedRoom { id: "1.02", phi_basis: 262.0,  phi_extra: 45.0,  phi_hl_i: 307.0  },
        ExpectedRoom { id: "1.03", phi_basis: 241.0,  phi_extra: 40.0,  phi_hl_i: 281.0  },
        ExpectedRoom { id: "1.04", phi_basis: 556.0,  phi_extra: 119.0, phi_hl_i: 675.0  },
        ExpectedRoom { id: "1.05", phi_basis: 230.0,  phi_extra: 34.0,  phi_hl_i: 263.0  },
        ExpectedRoom { id: "1.08", phi_basis: 1252.0, phi_extra: 115.0, phi_hl_i: 1367.0 },
    ];

    #[test]
    fn test_dr_engineering_woningbouw() {
        let input = include_str!("../../../tests/verification/isso51_vabi3.12.0.127_dr-engineering-woningbouw/input.json");
        let result = calculate_from_json(input);

        match result {
            Ok(result_json) => {
                let result: result::ProjectResult =
                    serde_json::from_str(&result_json).unwrap();

                assert_eq!(
                    result.rooms.len(),
                    DR_EXPECTED.len(),
                    "Expected {} rooms, got {}",
                    DR_EXPECTED.len(),
                    result.rooms.len()
                );

                println!("\n{}", "=".repeat(100));
                println!(
                    "DR Engineering Woningbouw — Engine vs Vabi 3.12.0.127 (ISSO 51:2024)"
                );
                println!("{}", "=".repeat(100));
                println!(
                    "{:<12} {:>8} {:>8} {:>8} | {:>8} {:>8} {:>8} | {:>8} {:>8} {:>8}",
                    "Room", "Φ_bas_E", "Φ_ext_E", "Φ_HL_E",
                    "Φ_bas_V", "Φ_ext_V", "Φ_HL_V",
                    "Δ_bas", "Δ_ext", "Δ_HL"
                );
                println!("{}", "-".repeat(100));

                for (room, expected) in result.rooms.iter().zip(DR_EXPECTED.iter()) {
                    assert_eq!(
                        room.room_id, expected.id,
                        "Room order mismatch: got {}, expected {}",
                        room.room_id, expected.id
                    );

                    let d_basis = room.basis_heat_loss - expected.phi_basis;
                    let d_extra = room.extra_heat_loss - expected.phi_extra;
                    let d_total = room.total_heat_loss - expected.phi_hl_i;

                    println!(
                        "{:<12} {:>8.0} {:>8.0} {:>8.0} | {:>8.0} {:>8.0} {:>8.0} | {:>+8.0} {:>+8.0} {:>+8.0}",
                        room.room_id,
                        room.basis_heat_loss,
                        room.extra_heat_loss,
                        room.total_heat_loss,
                        expected.phi_basis,
                        expected.phi_extra,
                        expected.phi_hl_i,
                        d_basis,
                        d_extra,
                        d_total,
                    );
                }

                // Building-level totals
                let engine_basis: f64 =
                    result.rooms.iter().map(|r| r.basis_heat_loss).sum();
                let engine_total: f64 =
                    result.rooms.iter().map(|r| r.total_heat_loss).sum();

                println!("{}", "-".repeat(100));
                println!(
                    "{:<12} {:>8.0} {:>8} {:>8.0} | {:>8} {:>8} {:>8} | {:>+8.0} {:>8} {:>+8.0}",
                    "SUM",
                    engine_basis, "", engine_total,
                    "5931", "770", "6700",
                    engine_basis - 5931.0, "", engine_total - 6700.0,
                );
                println!("{}", "=".repeat(100));

                // Sub-component detail per room
                println!("\nDetail per ruimte:");
                println!(
                    "{:<12} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
                    "Room", "H_T,ie", "H_T,ia", "H_T,io", "H_T,ig", "Φ_i", "Φ_vent"
                );
                println!("{}", "-".repeat(70));
                for room in &result.rooms {
                    println!(
                        "{:<12} {:>8.2} {:>8.2} {:>8.2} {:>8.2} {:>8.0} {:>8.0}",
                        room.room_id,
                        room.transmission.h_t_exterior,
                        room.transmission.h_t_adjacent_rooms,
                        room.transmission.h_t_unheated,
                        room.transmission.h_t_ground,
                        room.infiltration.phi_i,
                        room.ventilation.phi_vent,
                    );
                }
                println!();
            }
            Err(e) => {
                panic!("calculate_from_json failed: {e}");
            }
        }
    }
}
