//! Thermal import: parse a Revit thermal export JSON and map it to an ISSO 51 Project.
//!
//! The thermal export contains rooms (heated, unheated, and pseudo-rooms like outside/ground/water),
//! constructions with layer build-ups, openings (windows/doors), and open connections.
//!
//! The mapping creates Room objects only for heated and unheated rooms, maps constructions to
//! ConstructionElements, and determines BoundaryType based on the adjacent room type.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::sfb::build_sfb_name;
use crate::model::{
    BoundaryType, Building, BuildingType, ConstructionElement, DesignConditions, GroundParameters,
    HeatingSystem, InfiltrationMethod, MaterialType, Project, ProjectInfo, Room, RoomFunction,
    SecurityClass, VentilationConfig, VentilationSystemType, VerticalPosition,
};

// ─── Input types (deserialized from thermal-import JSON) ───

/// Top-level container for a Revit thermal export.
#[derive(Debug, Clone, Deserialize)]
pub struct ThermalImport {
    pub version: String,
    pub source: String,
    pub exported_at: String,
    #[serde(default)]
    pub project_name: Option<String>,
    pub rooms: Vec<ThermalRoom>,
    pub constructions: Vec<ThermalConstruction>,
    #[serde(default)]
    pub openings: Vec<ThermalOpening>,
    #[serde(default)]
    pub open_connections: Vec<ThermalOpenConnection>,
}

/// A room from the thermal export.
#[derive(Debug, Clone, Deserialize)]
pub struct ThermalRoom {
    pub id: String,
    #[serde(default)]
    pub revit_id: Option<i64>,
    pub name: String,
    #[serde(rename = "type")]
    pub room_type: ThermalRoomType,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub area_m2: Option<f64>,
    #[serde(default)]
    pub height_m: Option<f64>,
    #[serde(default)]
    pub volume_m3: Option<f64>,
    #[serde(default)]
    pub boundary_polygon: Option<Vec<[f64; 2]>>,
}

/// Thermal zone classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalRoomType {
    Heated,
    Unheated,
    Outside,
    Ground,
    Water,
}

/// A construction (wall/floor/ceiling/roof) between two rooms.
#[derive(Debug, Clone, Deserialize)]
pub struct ThermalConstruction {
    pub id: String,
    pub room_a: String,
    pub room_b: String,
    pub orientation: ThermalOrientation,
    #[serde(default)]
    pub compass: Option<String>,
    pub gross_area_m2: f64,
    #[serde(default)]
    pub revit_element_id: Option<i64>,
    #[serde(default)]
    pub revit_type_name: Option<String>,
    #[serde(default)]
    pub layers: Vec<ThermalLayer>,
}

/// Orientation of a construction element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalOrientation {
    Wall,
    Floor,
    Ceiling,
    Roof,
}

/// A single material layer in a construction assembly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThermalLayer {
    pub material: String,
    pub thickness_mm: f64,
    #[serde(default)]
    pub distance_from_interior_mm: Option<f64>,
    #[serde(default = "default_layer_type")]
    #[serde(rename = "type")]
    pub layer_type: ThermalLayerType,
    #[serde(default)]
    pub lambda: Option<f64>,
}

fn default_layer_type() -> ThermalLayerType {
    ThermalLayerType::Solid
}

/// Layer material type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalLayerType {
    Solid,
    AirGap,
}

/// An opening (window/door/curtain wall) in a construction.
#[derive(Debug, Clone, Deserialize)]
pub struct ThermalOpening {
    pub id: String,
    pub construction_id: String,
    #[serde(rename = "type")]
    pub opening_type: ThermalOpeningType,
    pub width_mm: f64,
    pub height_mm: f64,
    #[serde(default)]
    pub sill_height_mm: Option<f64>,
    #[serde(default)]
    pub u_value: Option<f64>,
    #[serde(default)]
    pub revit_element_id: Option<i64>,
    #[serde(default)]
    pub revit_type_name: Option<String>,
}

/// Opening type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalOpeningType {
    Window,
    Door,
    CurtainWall,
}

/// An open connection between two rooms (room separator without wall).
#[derive(Debug, Clone, Deserialize)]
pub struct ThermalOpenConnection {
    pub room_a: String,
    pub room_b: String,
    pub area_m2: f64,
}

// ─── Output types ───

/// Result of mapping a thermal import to an ISSO 51 Project.
#[derive(Debug, Clone, Serialize)]
pub struct ThermalImportResult {
    /// The mapped ISSO 51 Project, ready for editing and calculation.
    pub project: Project,
    /// Warnings generated during the mapping process.
    pub warnings: Vec<String>,
    /// Unique constructions (shared between rooms), grouped by layer fingerprint.
    /// Each `ConstructionElement.catalog_ref` points to one of these entries.
    pub construction_catalog: Vec<CatalogEntry>,
    /// Room polygons for 3D viewer rendering.
    pub room_polygons: Vec<RoomPolygon>,
}

/// One unique construction (layer composition) in the catalog.
///
/// Constructions with identical layer fingerprints across the entire project are
/// merged into a single `CatalogEntry`. The `used_for` field records which
/// `(BoundaryType, ThermalOrientation)` combinations actually use this entry.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    /// Catalog ID, format `cat-{n}`.
    pub id: String,
    /// SfB-based description (e.g. `21_Stuc_KZS_PIR_Spouw_Klinker`).
    /// Receives a `_{thickness}mm` (or `_a` / `_b`) suffix on naming collisions.
    pub description: String,
    /// Layer composition from interior to exterior.
    pub layers: Vec<ThermalLayer>,
    /// First-encountered Revit type name (debug info, may be `None`).
    pub revit_type_name: Option<String>,
    /// Distinct `(BoundaryType, ThermalOrientation)` combinations in which
    /// this catalog entry is used. Informative for the UI.
    pub used_for: Vec<(BoundaryType, ThermalOrientation)>,
    /// Total area in m² across every surface that uses this entry.
    pub total_area_m2: f64,
    /// Number of raw surfaces in the source export that use this entry.
    pub surface_count: usize,
}

/// Room polygon for 3D viewer.
#[derive(Debug, Clone, Serialize)]
pub struct RoomPolygon {
    pub room_id: String,
    pub name: String,
    pub level: Option<String>,
    pub height_m: f64,
    pub polygon: Vec<[f64; 2]>,
}

// ─── Mapping logic ───

/// Build a stable layer fingerprint string from a layer stack.
///
/// Format per layer: `{material lowercase}|{thickness_mm:1}|{layer_type:?}`,
/// joined with `::`. The fingerprint is the catalog grouping key. Lambda is
/// deliberately excluded so the same physical construction with slightly
/// different lambda values from different Revit projects still groups.
fn layer_fingerprint(layers: &[ThermalLayer]) -> String {
    layers
        .iter()
        .map(|l| {
            format!(
                "{}|{:.1}|{:?}",
                l.material.trim().to_lowercase(),
                l.thickness_mm,
                l.layer_type
            )
        })
        .collect::<Vec<_>>()
        .join("::")
}

/// Sum of all layer thicknesses in mm. Used as the collision-suffix value.
fn total_thickness_mm(layers: &[ThermalLayer]) -> f64 {
    layers.iter().map(|l| l.thickness_mm).sum()
}

/// One raw surface fed into phase 3 of `map_thermal_import`. Tracks every
/// occurrence of a construction in the source export so the catalog can
/// aggregate areas, count surfaces and back-fill `catalog_ref` on the
/// already-grouped per-room `ConstructionElement`s.
struct RawSurface {
    fingerprint: String,
    layers: Vec<ThermalLayer>,
    boundary_type: BoundaryType,
    orientation: ThermalOrientation,
    revit_type_name: Option<String>,
    /// Net area of this single surface in m².
    area_m2: f64,
    /// Coordinates back into the per-room result so we can set
    /// `catalog_ref` after collision handling: `(room_index, element_index)`.
    room_index: usize,
    element_index: usize,
}

/// Map a `ThermalImport` into a `ThermalImportResult`.
///
/// Creates Room objects only for heated and unheated rooms. Pseudo-rooms (outside,
/// ground, water) are used to determine BoundaryType but are not included as rooms.
///
/// U-values are set to 0.0 (placeholder) — the user calculates them via the
/// Rc-calculator in the frontend.
pub fn map_thermal_import(input: ThermalImport) -> ThermalImportResult {
    let mut warnings: Vec<String> = Vec::new();

    // Build lookup: room_id → ThermalRoom
    let room_map: HashMap<&str, &ThermalRoom> =
        input.rooms.iter().map(|r| (r.id.as_str(), r)).collect();

    // Build lookup: construction_id → list of openings
    let mut openings_by_construction: HashMap<&str, Vec<&ThermalOpening>> = HashMap::new();
    for opening in &input.openings {
        openings_by_construction
            .entry(opening.construction_id.as_str())
            .or_default()
            .push(opening);
    }

    // Collect rooms that should become ISSO 51 Room objects (heated + unheated only).
    let real_rooms: Vec<&ThermalRoom> = input
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type, ThermalRoomType::Heated | ThermalRoomType::Unheated))
        .collect();

    // Group constructions by room_a.
    let mut constructions_by_room: HashMap<&str, Vec<&ThermalConstruction>> = HashMap::new();
    for c in &input.constructions {
        constructions_by_room
            .entry(c.room_a.as_str())
            .or_default()
            .push(c);
    }

    // Collect raw surfaces from every room for the global catalog (phase 3).
    let mut raw_surfaces: Vec<RawSurface> = Vec::new();

    // Collect room polygons for 3D viewer.
    let mut room_polygons: Vec<RoomPolygon> = Vec::new();

    // Map each real room.
    let mut isso_rooms: Vec<Room> = Vec::new();
    for thermal_room in &real_rooms {
        let floor_area = thermal_room.area_m2.unwrap_or(0.0);
        let height = thermal_room.height_m.unwrap_or(2.6);

        // Build polygon info if available.
        if let Some(ref polygon) = thermal_room.boundary_polygon {
            room_polygons.push(RoomPolygon {
                room_id: thermal_room.id.clone(),
                name: thermal_room.name.clone(),
                level: thermal_room.level.clone(),
                height_m: height,
                polygon: polygon.clone(),
            });
        }

        // Map constructions for this room.
        // Phase 1: Collect individual construction surfaces and openings.
        let mut raw_elements: Vec<ConstructionElement> = Vec::new();
        let mut opening_elements: Vec<ConstructionElement> = Vec::new();
        let mut elem_counter: u32 = 0;

        // Track grouping info per element: (revit_type_name, boundary_type, orientation, layers)
        // Used for grouping in phase 2.
        struct GroupingInfo {
            revit_type_name: String,
            boundary_type: BoundaryType,
            orientation: ThermalOrientation,
            layers: Vec<ThermalLayer>,
            adjacent_room_id: Option<String>,
        }
        let mut grouping_infos: Vec<GroupingInfo> = Vec::new();

        if let Some(constructions) = constructions_by_room.get(thermal_room.id.as_str()) {
            for construction in constructions {
                // Look up room_b to determine boundary type.
                let room_b = room_map.get(construction.room_b.as_str());
                if room_b.is_none() {
                    warnings.push(format!(
                        "Constructie '{}': room_b '{}' niet gevonden in rooms lijst",
                        construction.id, construction.room_b
                    ));
                }

                // Water → Ground: ISSO 51 behandelt watercontact vergelijkbaar met
                // grondcontact (constante temperatuur, correctiefactor). De gebruiker
                // past de temperatuurcorrectie handmatig aan in de warmteverlies tool.
                let boundary_type = room_b
                    .map(|rb| match rb.room_type {
                        ThermalRoomType::Outside => BoundaryType::Exterior,
                        ThermalRoomType::Ground | ThermalRoomType::Water => BoundaryType::Ground,
                        ThermalRoomType::Unheated => BoundaryType::UnheatedSpace,
                        ThermalRoomType::Heated => BoundaryType::AdjacentRoom,
                    })
                    .unwrap_or(BoundaryType::Exterior);

                let vertical_position = match construction.orientation {
                    ThermalOrientation::Floor => VerticalPosition::Floor,
                    ThermalOrientation::Ceiling | ThermalOrientation::Roof => {
                        VerticalPosition::Ceiling
                    }
                    ThermalOrientation::Wall => VerticalPosition::Wall,
                };

                // Warn if construction has no layers.
                if construction.layers.is_empty() {
                    warnings.push(format!(
                        "Constructie '{}' ({}) heeft geen lagen — U-waarde kan niet berekend worden",
                        construction.id,
                        construction.revit_type_name.as_deref().unwrap_or("onbekend"),
                    ));
                }

                // Calculate net area (gross minus openings).
                let openings_in_construction =
                    openings_by_construction.get(construction.id.as_str());
                let total_opening_area: f64 = openings_in_construction
                    .map(|ops| {
                        ops.iter()
                            .map(|o| (o.width_mm * o.height_mm) / 1_000_000.0)
                            .sum()
                    })
                    .unwrap_or(0.0);
                let net_area = (construction.gross_area_m2 - total_opening_area).max(0.0);

                // Filter: skip 0 m² construction surfaces.
                if net_area <= 0.0 {
                    warnings.push(format!(
                        "Constructie '{}' ({}) overgeslagen: netto oppervlak is 0 m²",
                        construction.id,
                        construction.revit_type_name.as_deref().unwrap_or("onbekend"),
                    ));
                } else {
                    // Adjacent room info.
                    let adjacent_room_id = if boundary_type == BoundaryType::AdjacentRoom
                        || boundary_type == BoundaryType::UnheatedSpace
                    {
                        Some(construction.room_b.clone())
                    } else {
                        None
                    };

                    // Ground parameters for ground elements.
                    let ground_params = if boundary_type == BoundaryType::Ground {
                        Some(GroundParameters {
                            u_equivalent: 0.0,
                            ground_water_factor: 1.0,
                            fg2: 1.0,
                        })
                    } else {
                        None
                    };

                    elem_counter += 1;
                    raw_elements.push(ConstructionElement {
                        id: format!("{}-c{}", thermal_room.id, elem_counter),
                        description: String::new(), // will be set during grouping
                        area: net_area,
                        u_value: 0.0, // placeholder — user calculates via Rc-calculator
                        boundary_type,
                        material_type: MaterialType::Masonry, // default; user adjusts
                        temperature_factor: None,
                        adjacent_room_id: adjacent_room_id.clone(),
                        adjacent_temperature: None,
                        vertical_position,
                        use_forfaitaire_thermal_bridge: boundary_type == BoundaryType::Exterior,
                        custom_delta_u_tb: None,
                        ground_params,
                        has_embedded_heating: false,
                        catalog_ref: None, // filled in during phase 3
                    });

                    grouping_infos.push(GroupingInfo {
                        revit_type_name: construction
                            .revit_type_name
                            .clone()
                            .unwrap_or_else(|| "onbekend".to_string()),
                        boundary_type,
                        orientation: construction.orientation,
                        layers: construction.layers.clone(),
                        adjacent_room_id,
                    });
                }

                // Map openings as separate ConstructionElements (not grouped).
                if let Some(ops) = openings_in_construction {
                    for opening in ops {
                        let opening_area = (opening.width_mm * opening.height_mm) / 1_000_000.0;
                        let opening_desc = format!(
                            "{} — {}",
                            opening
                                .revit_type_name
                                .as_deref()
                                .unwrap_or(match opening.opening_type {
                                    ThermalOpeningType::Window => "raam",
                                    ThermalOpeningType::Door => "deur",
                                    ThermalOpeningType::CurtainWall => "vliesgevel",
                                }),
                            construction
                                .compass
                                .as_deref()
                                .unwrap_or(""),
                        );

                        elem_counter += 1;
                        opening_elements.push(ConstructionElement {
                            id: format!("{}-c{}", thermal_room.id, elem_counter),
                            description: opening_desc,
                            area: opening_area,
                            u_value: opening.u_value.unwrap_or(0.0),
                            boundary_type,
                            material_type: MaterialType::NonMasonry,
                            temperature_factor: None,
                            adjacent_room_id: if boundary_type == BoundaryType::AdjacentRoom
                                || boundary_type == BoundaryType::UnheatedSpace
                            {
                                Some(construction.room_b.clone())
                            } else {
                                None
                            },
                            adjacent_temperature: None,
                            vertical_position: VerticalPosition::Wall,
                            use_forfaitaire_thermal_bridge: boundary_type
                                == BoundaryType::Exterior,
                            custom_delta_u_tb: None,
                            ground_params: None,
                            has_embedded_heating: false,
                            catalog_ref: None, // openings are intentionally outside the catalog
                        });
                    }
                }
            }
        }

        // Phase 2: Group construction surfaces by
        // (layer_fingerprint, boundary_type, orientation, adjacent_room_id?).
        // Surfaces with the same key are merged: areas summed, SfB-based name assigned.
        //
        // The first component is the **layer fingerprint** (not `revit_type_name`).
        // Rationale: PyRevit's ThermalExport does not always populate
        // `revit_type_name` — in the 3056 woonboot fixture all 83 surfaces have
        // `None`, which previously collapsed onto a single `"onbekend"` key and
        // merged fysiek ongelijke wanden (b.v. spouwmuur + drijflichaam + binnenwand)
        // tot één ConstructionElement per ruimte. Door te groeperen op de stabiele
        // layer fingerprint worden surfaces met verschillende samenstellingen nu
        // als aparte elementen behandeld, ook wanneer hun `revit_type_name` leeg is.
        //
        // The `adjacent_room_id` component is only included for interior boundaries
        // (`AdjacentRoom` / `UnheatedSpace`) so walls that look identical but separate
        // different rooms (e.g. 3 binnenwanden van hetzelfde type naar 3 verschillende
        // kamers) worden niet meer onterecht tot één entry gemerged — anders zou de
        // gebruiker de adjacency-info (cruciaal voor temperatuur-factor checks en voor
        // verificatie door de gebruiker) kwijtraken in de geaggregeerde `adjacent_room_id`.
        // Voor `Exterior`, `Ground` en `AdjacentBuilding` is er geen adjacent room,
        // dus dan is deze component simpelweg `None` en verandert het gedrag niet.
        let mut elements: Vec<ConstructionElement> = Vec::new();

        // Group key: (layer_fingerprint, boundary_type discriminant, orientation
        // discriminant, adjacent_room_id for interior boundaries only).
        type GroupKey = (String, u8, u8, Option<String>);

        fn boundary_discriminant(bt: BoundaryType) -> u8 {
            match bt {
                BoundaryType::Exterior => 0,
                BoundaryType::Ground => 1,
                BoundaryType::UnheatedSpace => 2,
                BoundaryType::AdjacentRoom => 3,
                BoundaryType::AdjacentBuilding => 4,
            }
        }

        fn orientation_discriminant(o: ThermalOrientation) -> u8 {
            match o {
                ThermalOrientation::Wall => 0,
                ThermalOrientation::Floor => 1,
                ThermalOrientation::Ceiling => 2,
                ThermalOrientation::Roof => 3,
            }
        }

        /// Returns the adjacency discriminator used in the grouping key.
        ///
        /// Interior boundaries (`AdjacentRoom` / `UnheatedSpace`) return the
        /// per-element `adjacent_room_id` so walls that separate different
        /// rooms are kept as distinct groups. Non-interior boundary types
        /// always return `None`.
        ///
        /// This is intentionally written as an exhaustive match (no `_` arm):
        /// when a new `BoundaryType` variant is added, the compiler forces us
        /// to decide here whether it carries adjacency semantics, preventing
        /// silent mis-grouping bugs.
        fn adjacency_key(bt: BoundaryType, adjacent_room_id: &Option<String>) -> Option<String> {
            match bt {
                BoundaryType::AdjacentRoom | BoundaryType::UnheatedSpace => {
                    adjacent_room_id.clone()
                }
                BoundaryType::Exterior
                | BoundaryType::Ground
                | BoundaryType::AdjacentBuilding => None,
            }
        }

        // Build groups, preserving insertion order.
        let mut group_order: Vec<GroupKey> = Vec::new();
        let mut groups: HashMap<GroupKey, Vec<usize>> = HashMap::new();

        for (idx, info) in grouping_infos.iter().enumerate() {
            // For interior boundaries the adjacency key discriminates groups;
            // for Exterior/Ground/AdjacentBuilding it is always `None` so those
            // paths stay behaviour-compatible with the pre-fix grouping.
            let adj_key = adjacency_key(info.boundary_type, &info.adjacent_room_id);
            let key: GroupKey = (
                layer_fingerprint(&info.layers),
                boundary_discriminant(info.boundary_type),
                orientation_discriminant(info.orientation),
                adj_key,
            );
            let entry = groups.entry(key.clone()).or_default();
            if entry.is_empty() {
                group_order.push(key);
            }
            entry.push(idx);
        }

        // The phase-2 group element index in `elements` for each merged group.
        // Recorded so the raw_surfaces collected below point at the correct
        // (room_index, element_index) tuple for catalog_ref back-fill in phase 3.
        let room_index = isso_rooms.len();

        let mut group_counter: u32 = 0;
        for key in &group_order {
            let indices = &groups[key];
            let first_idx = indices[0];
            let first_info = &grouping_infos[first_idx];
            let first_elem = &raw_elements[first_idx];

            // Sum areas across all surfaces in this group.
            let total_area: f64 = indices.iter().map(|&i| raw_elements[i].area).sum();

            // Generate SfB-based description.
            //
            // Bewust geen adjacent-room suffix toegevoegd aan de SfB description voor
            // interior boundaries: de `adjacent_room_id` blijft bewaard als apart veld
            // op `ConstructionElement` (zie het `first_info.adjacent_room_id.clone()`
            // een paar regels lager), dus de frontend kan de bestemmingsruimte direct
            // uit dat veld tonen. SfB-namen vervuilen met room-IDs maakt groeperingen
            // onleesbaar en dupliceert bovendien informatie die al gestructureerd
            // beschikbaar is.
            let description = build_sfb_name(
                first_info.boundary_type,
                first_info.orientation,
                &first_info.layers,
            );

            // Log grouping info when multiple surfaces are merged.
            if indices.len() > 1 {
                warnings.push(format!(
                    "Ruimte '{}': {} grensvlakken samengevoegd tot '{}' (totaal {:.2} m²)",
                    thermal_room.name,
                    indices.len(),
                    description,
                    total_area,
                ));
            }

            let group_elem_index = elements.len();
            group_counter += 1;
            elements.push(ConstructionElement {
                id: format!("{}-g{}", thermal_room.id, group_counter),
                description,
                area: total_area,
                u_value: first_elem.u_value,
                boundary_type: first_elem.boundary_type,
                material_type: first_elem.material_type,
                temperature_factor: first_elem.temperature_factor,
                adjacent_room_id: first_info.adjacent_room_id.clone(),
                adjacent_temperature: first_elem.adjacent_temperature,
                vertical_position: first_elem.vertical_position,
                use_forfaitaire_thermal_bridge: first_elem.use_forfaitaire_thermal_bridge,
                custom_delta_u_tb: first_elem.custom_delta_u_tb,
                ground_params: first_elem.ground_params.clone(),
                has_embedded_heating: first_elem.has_embedded_heating,
                catalog_ref: None, // filled in during phase 3
            });

            // Record one RawSurface per source raw_element in this group for the
            // global catalog. All surfaces in a phase-2 group share the same
            // (room_index, group_elem_index) so phase 3 can back-fill `catalog_ref`
            // on the merged element.
            for &raw_idx in indices {
                let raw_info = &grouping_infos[raw_idx];
                raw_surfaces.push(RawSurface {
                    fingerprint: layer_fingerprint(&raw_info.layers),
                    layers: raw_info.layers.clone(),
                    boundary_type: raw_info.boundary_type,
                    orientation: raw_info.orientation,
                    revit_type_name: if raw_info.revit_type_name == "onbekend" {
                        None
                    } else {
                        Some(raw_info.revit_type_name.clone())
                    },
                    area_m2: raw_elements[raw_idx].area,
                    room_index,
                    element_index: group_elem_index,
                });
            }
        }

        // Add opening elements (not grouped).
        elements.extend(opening_elements);

        if elements.is_empty() {
            warnings.push(format!(
                "Ruimte '{}' heeft geen constructie-elementen — controleer of er wanden/vloeren zijn toegewezen",
                thermal_room.name
            ));
        }

        // Determine room function based on room type.
        let function = match thermal_room.room_type {
            ThermalRoomType::Heated => RoomFunction::LivingRoom,   // default; user adjusts
            ThermalRoomType::Unheated => RoomFunction::Storage,     // default for unheated
            _ => RoomFunction::Custom,
        };

        isso_rooms.push(Room {
            id: thermal_room.id.clone(),
            name: thermal_room.name.clone(),
            function,
            custom_temperature: None,
            floor_area,
            height,
            constructions: elements,
            heating_system: HeatingSystem::RadiatorLt, // default; user adjusts
            ventilation_rate: None,
            has_mechanical_exhaust: false,
            has_mechanical_supply: false,
            fraction_outside_air: 1.0,
            supply_air_temperature: None,
            internal_air_temperature: None,
            clamp_positive: true,
        });
    }

    // ─── Phase 3: build the global construction catalog ───
    //
    // Group every raw surface across the entire project by **layer fingerprint
    // alone** (per spec besluit 1a). The catalog entries become the single
    // source of truth for unique constructions; the per-room phase-2 grouped
    // `ConstructionElement`s receive a `catalog_ref` pointing back to the
    // catalog entry.
    let construction_catalog = build_construction_catalog(&raw_surfaces, &mut isso_rooms);

    // Calculate total floor area from heated rooms.
    let total_floor_area: f64 = isso_rooms
        .iter()
        .filter(|r| r.function != RoomFunction::Storage)
        .map(|r| r.floor_area)
        .sum();

    let project = Project {
        info: ProjectInfo {
            name: input
                .project_name
                .unwrap_or_else(|| "Thermal Import".to_string()),
            project_number: None,
            address: None,
            client: None,
            date: Some(input.exported_at.clone()),
            engineer: None,
            notes: Some(format!("Geimporteerd uit {} ({})", input.source, input.version)),
        },
        building: Building {
            building_type: BuildingType::Detached, // default; user adjusts
            qv10: 0.0,                             // must be entered by user
            total_floor_area,
            security_class: SecurityClass::B,       // default
            has_night_setback: false,
            warmup_time: 2.0,
            building_height: None,
            num_floors: 1,
            infiltration_method: InfiltrationMethod::PerExteriorArea,
        },
        climate: DesignConditions::default(),
        ventilation: VentilationConfig {
            system_type: VentilationSystemType::SystemC,  // default; user adjusts
            has_heat_recovery: false,
            heat_recovery_efficiency: None,
            frost_protection: None,
            supply_temperature: None,
            has_preheating: false,
            preheating_temperature: None,
        },
        rooms: isso_rooms,
    };

    ThermalImportResult {
        project,
        warnings,
        construction_catalog,
        room_polygons,
    }
}

/// Build the global construction catalog from all raw surfaces in the project.
///
/// Steps:
/// 1. Group surfaces by layer fingerprint (insertion order preserved).
/// 2. Generate an initial SfB-based description per group.
/// 3. Resolve description collisions by appending the total layer thickness
///    in mm; on a tie, fall back to alphabetic letter suffixes (`_a`, `_b`, …).
///    The earlier entry that claimed the colliding name is **also** rewritten,
///    not just the new one, so users see consistent suffixes everywhere.
/// 4. Back-fill `catalog_ref` on every per-room `ConstructionElement` whose
///    raw surfaces map to a catalog entry.
///
/// Openings (which never enter `raw_surfaces`) keep `catalog_ref = None`.
fn build_construction_catalog(
    raw_surfaces: &[RawSurface],
    rooms: &mut [Room],
) -> Vec<CatalogEntry> {
    // ─ Group surfaces by fingerprint, preserving insertion order ─
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, surface) in raw_surfaces.iter().enumerate() {
        let entry = groups.entry(surface.fingerprint.clone()).or_default();
        if entry.is_empty() {
            order.push(surface.fingerprint.clone());
        }
        entry.push(idx);
    }

    // ─ Build initial entries with the un-suffixed SfB description ─
    struct Pending {
        fingerprint: String,
        layers: Vec<ThermalLayer>,
        revit_type_name: Option<String>,
        used_for: Vec<(BoundaryType, ThermalOrientation)>,
        total_area_m2: f64,
        surface_count: usize,
        initial_description: String,
        total_thickness_mm: f64,
    }

    let mut pending: Vec<Pending> = Vec::with_capacity(order.len());
    for fingerprint in &order {
        let indices = &groups[fingerprint];
        let first = &raw_surfaces[indices[0]];

        // Sum total area + count + collect distinct (boundary, orientation).
        let mut total_area = 0.0;
        let mut used_for: Vec<(BoundaryType, ThermalOrientation)> = Vec::new();
        for &i in indices {
            let s = &raw_surfaces[i];
            total_area += s.area_m2;
            let combo = (s.boundary_type, s.orientation);
            if !used_for.contains(&combo) {
                used_for.push(combo);
            }
        }

        let initial_description =
            build_sfb_name(first.boundary_type, first.orientation, &first.layers);
        let thickness = total_thickness_mm(&first.layers);

        pending.push(Pending {
            fingerprint: fingerprint.clone(),
            layers: first.layers.clone(),
            revit_type_name: first.revit_type_name.clone(),
            used_for,
            total_area_m2: total_area,
            surface_count: indices.len(),
            initial_description,
            total_thickness_mm: thickness,
        });
    }

    // ─ Collision detection on initial descriptions ─
    let mut name_buckets: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, p) in pending.iter().enumerate() {
        name_buckets
            .entry(p.initial_description.clone())
            .or_default()
            .push(i);
    }

    // Final resolved description per pending index.
    let mut final_descriptions: Vec<String> = vec![String::new(); pending.len()];

    for bucket in name_buckets.values() {
        if bucket.len() == 1 {
            // No collision: keep the original SfB name.
            let idx = bucket[0];
            final_descriptions[idx] = pending[idx].initial_description.clone();
            continue;
        }

        // Collision: append the total thickness as `_{rounded_mm}mm`.
        // Re-bucket by the thickness suffix to detect tie-on-thickness cases
        // where we additionally need a letter suffix `_a`/`_b`/...
        let mut by_thickness: HashMap<u64, Vec<usize>> = HashMap::new();
        for &i in bucket {
            // Round to the nearest mm — total layer thicknesses already
            // come from Revit as integer mm values in practice, but rounding
            // here keeps the suffix predictable for fractional inputs too.
            let key = pending[i].total_thickness_mm.round() as i64 as u64;
            by_thickness.entry(key).or_default().push(i);
        }

        for (_thickness_key, mut tie_group) in by_thickness {
            if tie_group.len() == 1 {
                let idx = tie_group[0];
                let p = &pending[idx];
                final_descriptions[idx] = format!(
                    "{}_{}mm",
                    p.initial_description,
                    p.total_thickness_mm.round() as i64
                );
            } else {
                // Tie on thickness too — append `_a`, `_b`, ... in stable
                // insertion order so output is deterministic.
                tie_group.sort_unstable();
                for (letter_idx, idx) in tie_group.iter().enumerate() {
                    let p = &pending[*idx];
                    let letter = (b'a' + letter_idx as u8) as char;
                    final_descriptions[*idx] = format!(
                        "{}_{}mm_{}",
                        p.initial_description,
                        p.total_thickness_mm.round() as i64,
                        letter
                    );
                }
            }
        }
    }

    // ─ Build final CatalogEntry list and back-fill catalog_ref on rooms ─
    let mut catalog: Vec<CatalogEntry> = Vec::with_capacity(pending.len());
    for (i, p) in pending.into_iter().enumerate() {
        let id = format!("cat-{}", i + 1);

        // Set catalog_ref on every ConstructionElement that uses this fingerprint.
        for surface in raw_surfaces.iter().filter(|s| s.fingerprint == p.fingerprint) {
            let elem = &mut rooms[surface.room_index].constructions[surface.element_index];
            // The element's description in the catalog refactor follows the
            // catalog entry exactly, so wijzigingen stromen door naar de
            // vertrekken-view zonder dat we daar nog iets aan moeten doen.
            elem.description = final_descriptions[i].clone();
            elem.catalog_ref = Some(id.clone());
        }

        catalog.push(CatalogEntry {
            id,
            description: final_descriptions[i].clone(),
            layers: p.layers,
            revit_type_name: p.revit_type_name,
            used_for: p.used_for,
            total_area_m2: p.total_area_m2,
            surface_count: p.surface_count,
        });
    }

    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load the test fixture.
    fn load_fixture() -> ThermalImport {
        let json = include_str!("../../../../tests/fixtures/thermal-import-sample.json");
        serde_json::from_str(json).expect("Failed to parse thermal-import-sample.json")
    }

    #[test]
    fn test_parse_thermal_import() {
        let import = load_fixture();

        assert_eq!(import.version, "1.0");
        assert_eq!(import.source, "revit-eam");
        assert_eq!(import.rooms.len(), 5);
        assert_eq!(import.constructions.len(), 5);
        assert_eq!(import.openings.len(), 4);
        assert_eq!(import.open_connections.len(), 1);

        // Verify room types.
        assert_eq!(import.rooms[0].room_type, ThermalRoomType::Heated);
        assert_eq!(import.rooms[1].room_type, ThermalRoomType::Unheated);
        assert_eq!(import.rooms[3].room_type, ThermalRoomType::Outside);
        assert_eq!(import.rooms[4].room_type, ThermalRoomType::Ground);

        // Verify construction layers.
        assert_eq!(import.constructions[0].layers.len(), 4);
        assert_eq!(import.constructions[0].layers[0].material, "Gipsplaat");
        assert_eq!(import.constructions[0].layers[1].lambda, Some(0.035));
    }

    #[test]
    fn test_map_creates_rooms_for_heated_and_unheated_only() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        // Only heated (room-0, room-2) and unheated (room-1) should become Rooms.
        // outside (room-outside) and ground (room-ground) should NOT.
        assert_eq!(result.project.rooms.len(), 3);

        let room_ids: Vec<&str> = result.project.rooms.iter().map(|r| r.id.as_str()).collect();
        assert!(room_ids.contains(&"room-0"));
        assert!(room_ids.contains(&"room-1"));
        assert!(room_ids.contains(&"room-2"));
        assert!(!room_ids.contains(&"room-outside"));
        assert!(!room_ids.contains(&"room-ground"));
    }

    #[test]
    fn test_map_boundary_types() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        // room-0's constructions:
        // constr-0: room_b=room-outside → Exterior
        // constr-1: room_b=room-1 (unheated) → UnheatedSpace
        // constr-2: room_b=room-ground → Ground
        let room_0 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-0")
            .expect("room-0 not found");

        // Find the wall to outside (constr-0 net area element).
        let exterior_elements: Vec<&ConstructionElement> = room_0
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::Exterior)
            .collect();
        assert!(
            !exterior_elements.is_empty(),
            "Should have exterior boundary elements"
        );

        // Find the wall to unheated room-1 (constr-1).
        let unheated_elements: Vec<&ConstructionElement> = room_0
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::UnheatedSpace)
            .collect();
        assert!(
            !unheated_elements.is_empty(),
            "Should have unheated space boundary elements"
        );

        // Find the floor to ground (constr-2).
        let ground_elements: Vec<&ConstructionElement> = room_0
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::Ground)
            .collect();
        assert!(
            !ground_elements.is_empty(),
            "Should have ground boundary elements"
        );
        // Ground element must have ground_params.
        assert!(
            ground_elements[0].ground_params.is_some(),
            "Ground element must have ground_params"
        );

        // room-2's constructions: constr-3 and constr-4, both to room-outside → Exterior
        let room_2 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-2")
            .expect("room-2 not found");
        let room_2_exterior: Vec<&ConstructionElement> = room_2
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::Exterior)
            .collect();
        assert!(
            room_2_exterior.len() >= 2,
            "room-2 should have at least 2 exterior elements (wall + roof + opening)"
        );
    }

    #[test]
    fn test_map_construction_catalog_returned() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        // The fixture has 5 constructions but `constr-4` (Hellend dak) has
        // empty layers and `constr-3` shares its description with constr-0.
        // The catalog must contain at least the four distinct layer
        // fingerprints (constr-0, constr-1, constr-2, constr-4-empty).
        assert!(
            result.construction_catalog.len() >= 3,
            "Catalog should have at least 3 entries, got {}",
            result.construction_catalog.len()
        );

        // The catalog entry derived from constr-0 must have its 4 layers.
        let constr_0_entry = result
            .construction_catalog
            .iter()
            .find(|e| {
                e.layers.len() == 4
                    && e.layers
                        .first()
                        .map(|l| l.material == "Gipsplaat")
                        .unwrap_or(false)
            })
            .expect("constr-0 catalog entry not found");
        assert_eq!(constr_0_entry.revit_type_name.as_deref(), Some("Spouwmuur 300mm"));

        // Every non-opening element in room-0 must point at a real catalog entry.
        let catalog_ids: std::collections::HashSet<&str> = result
            .construction_catalog
            .iter()
            .map(|e| e.id.as_str())
            .collect();
        let room_0 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-0")
            .expect("room-0 not found");
        for elem in &room_0.constructions {
            if elem.material_type == MaterialType::Masonry {
                let cref = elem
                    .catalog_ref
                    .as_deref()
                    .unwrap_or_else(|| panic!("Masonry element {} missing catalog_ref", elem.id));
                assert!(catalog_ids.contains(cref));
            }
        }
    }

    #[test]
    fn test_map_warnings_for_missing_layers() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        // constr-4 (Hellend dak) has empty layers → should generate a warning.
        let missing_layer_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("constr-4") && w.contains("geen lagen"))
            .collect();
        assert!(
            !missing_layer_warnings.is_empty(),
            "Should warn about constr-4 having no layers. Warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_map_openings_as_construction_elements() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        let room_0 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-0")
            .expect("room-0 not found");

        // room-0 has 3 constructions (constr-0, constr-1, constr-2) with openings:
        // constr-0: 2 windows (opening-0: 1200x1500, opening-1: 1800x2100)
        // constr-1: 1 door (opening-2: 830x2115)
        // constr-2: no openings
        // So we expect: 3 wall/floor elements + 3 opening elements = 6 total.
        // But openings from constr-0 are exterior (NonMasonry),
        // opening from constr-1 is to unheated space (NonMasonry).
        let non_masonry_elements: Vec<&ConstructionElement> = room_0
            .constructions
            .iter()
            .filter(|c| c.material_type == MaterialType::NonMasonry)
            .collect();
        assert_eq!(
            non_masonry_elements.len(),
            3,
            "room-0 should have 3 opening elements (2 windows + 1 door)"
        );

        // Check window area: opening-0 = 1200 * 1500 / 1e6 = 1.8 m²
        let window_1 = non_masonry_elements
            .iter()
            .find(|c| (c.area - 1.8).abs() < 0.01)
            .expect("Should have a 1.8 m² window element");
        assert_eq!(window_1.boundary_type, BoundaryType::Exterior);
        assert_eq!(window_1.material_type, MaterialType::NonMasonry);

        // Check door area: opening-2 = 830 * 2115 / 1e6 = 1.75545 m²
        let door = non_masonry_elements
            .iter()
            .find(|c| (c.area - 1.75545).abs() < 0.01)
            .expect("Should have a ~1.755 m² door element");
        assert_eq!(door.boundary_type, BoundaryType::UnheatedSpace);

        // room-2 should have 1 opening element (opening-3 from constr-3).
        let room_2 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-2")
            .expect("room-2 not found");
        let room_2_non_masonry: Vec<&ConstructionElement> = room_2
            .constructions
            .iter()
            .filter(|c| c.material_type == MaterialType::NonMasonry)
            .collect();
        assert_eq!(
            room_2_non_masonry.len(),
            1,
            "room-2 should have 1 opening element"
        );
        // opening-3: 1000 * 1200 / 1e6 = 1.2 m²
        assert!(
            (room_2_non_masonry[0].area - 1.2).abs() < 0.01,
            "Opening area should be 1.2 m², got {}",
            room_2_non_masonry[0].area
        );
    }

    #[test]
    fn test_map_project_metadata() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        assert_eq!(result.project.info.name, "Woonhuis Gouda");
        assert!(result.project.info.notes.as_ref().unwrap().contains("revit-eam"));
        assert_eq!(result.project.climate.theta_e, -10.0);
    }

    #[test]
    fn test_map_net_area_deducts_openings() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        let room_0 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-0")
            .expect("room-0 not found");

        // constr-0: gross 12.35 m², minus opening-0 (1.8) and opening-1 (3.78) = 6.77 m²
        let wall_exterior = room_0
            .constructions
            .iter()
            .find(|c| {
                c.boundary_type == BoundaryType::Exterior
                    && c.material_type == MaterialType::Masonry
                    && c.vertical_position == VerticalPosition::Wall
            })
            .expect("Should have exterior masonry wall element");
        let expected_net = 12.35 - (1.2 * 1.5) - (1.8 * 2.1);
        assert!(
            (wall_exterior.area - expected_net).abs() < 0.01,
            "Net area should be {:.2}, got {:.2}",
            expected_net,
            wall_exterior.area
        );
    }

    #[test]
    fn test_map_room_polygons() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        // room-0, room-1, room-2 have polygons; pseudo-rooms don't.
        assert_eq!(result.room_polygons.len(), 3);
        let poly_0 = result
            .room_polygons
            .iter()
            .find(|p| p.room_id == "room-0")
            .expect("room-0 polygon not found");
        assert_eq!(poly_0.polygon.len(), 4);
        assert_eq!(poly_0.height_m, 2.6);
    }

    // ─── New tests: grouping, 0 m² filtering, SfB naming ───

    #[test]
    fn test_zero_area_filtered_with_warning() {
        // Create a construction where openings consume all area → net_area = 0.
        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-08".to_string(),
            project_name: Some("Zero Area Test".to_string()),
            rooms: vec![
                ThermalRoom {
                    id: "r1".to_string(),
                    revit_id: None,
                    name: "Kamer".to_string(),
                    room_type: ThermalRoomType::Heated,
                    level: None,
                    area_m2: Some(10.0),
                    height_m: Some(2.6),
                    volume_m3: None,
                    boundary_polygon: None,
                },
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                // This construction has gross_area exactly equal to the opening area.
                ThermalConstruction {
                    id: "c-zero".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("N".to_string()),
                    gross_area_m2: 2.0,
                    revit_element_id: None,
                    revit_type_name: Some("Wand met pui".to_string()),
                    layers: vec![],
                },
                // Normal construction with area.
                ThermalConstruction {
                    id: "c-normal".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("S".to_string()),
                    gross_area_m2: 8.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur".to_string()),
                    layers: vec![],
                },
            ],
            openings: vec![
                // Opening consumes all area of c-zero.
                ThermalOpening {
                    id: "o1".to_string(),
                    construction_id: "c-zero".to_string(),
                    opening_type: ThermalOpeningType::Window,
                    width_mm: 2000.0,
                    height_mm: 1000.0,
                    sill_height_mm: None,
                    u_value: Some(1.6),
                    revit_element_id: None,
                    revit_type_name: Some("Pui".to_string()),
                },
            ],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = &result.project.rooms[0];

        // The zero-area construction should be filtered out.
        // Only the normal construction (masonry) + the opening (non-masonry) should remain.
        let masonry: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.material_type == MaterialType::Masonry)
            .collect();
        assert_eq!(masonry.len(), 1, "Should have 1 masonry element (the normal wall)");
        assert!((masonry[0].area - 8.0).abs() < 0.01, "Normal wall should have 8.0 m²");

        // Should have a warning about the zero-area construction.
        let zero_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("c-zero") && w.contains("0 m²"))
            .collect();
        assert!(
            !zero_warnings.is_empty(),
            "Should warn about zero area construction. Warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_grouping_same_type_constructions() {
        // 3 constructions with same revit_type_name, boundary_type, and orientation
        // should be merged into 1 with summed area.
        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-08".to_string(),
            project_name: Some("Grouping Test".to_string()),
            rooms: vec![
                ThermalRoom {
                    id: "r1".to_string(),
                    revit_id: None,
                    name: "Woonkamer".to_string(),
                    room_type: ThermalRoomType::Heated,
                    level: None,
                    area_m2: Some(30.0),
                    height_m: Some(2.6),
                    volume_m3: None,
                    boundary_polygon: None,
                },
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                ThermalConstruction {
                    id: "c1".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("N".to_string()),
                    gross_area_m2: 5.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur 300mm".to_string()),
                    layers: vec![
                        ThermalLayer {
                            material: "Kalkzandsteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(0.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(1.0),
                        },
                        ThermalLayer {
                            material: "PIR isolatie".to_string(),
                            thickness_mm: 120.0,
                            distance_from_interior_mm: Some(100.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.023),
                        },
                        ThermalLayer {
                            material: "Luchtspouw".to_string(),
                            thickness_mm: 40.0,
                            distance_from_interior_mm: Some(220.0),
                            layer_type: ThermalLayerType::AirGap,
                            lambda: None,
                        },
                        ThermalLayer {
                            material: "Baksteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(260.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.9),
                        },
                    ],
                },
                ThermalConstruction {
                    id: "c2".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("E".to_string()),
                    gross_area_m2: 8.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur 300mm".to_string()),
                    layers: vec![
                        ThermalLayer {
                            material: "Kalkzandsteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(0.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(1.0),
                        },
                        ThermalLayer {
                            material: "PIR isolatie".to_string(),
                            thickness_mm: 120.0,
                            distance_from_interior_mm: Some(100.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.023),
                        },
                        ThermalLayer {
                            material: "Luchtspouw".to_string(),
                            thickness_mm: 40.0,
                            distance_from_interior_mm: Some(220.0),
                            layer_type: ThermalLayerType::AirGap,
                            lambda: None,
                        },
                        ThermalLayer {
                            material: "Baksteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(260.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.9),
                        },
                    ],
                },
                ThermalConstruction {
                    id: "c3".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("W".to_string()),
                    gross_area_m2: 7.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur 300mm".to_string()),
                    layers: vec![
                        ThermalLayer {
                            material: "Kalkzandsteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(0.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(1.0),
                        },
                        ThermalLayer {
                            material: "PIR isolatie".to_string(),
                            thickness_mm: 120.0,
                            distance_from_interior_mm: Some(100.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.023),
                        },
                        ThermalLayer {
                            material: "Luchtspouw".to_string(),
                            thickness_mm: 40.0,
                            distance_from_interior_mm: Some(220.0),
                            layer_type: ThermalLayerType::AirGap,
                            lambda: None,
                        },
                        ThermalLayer {
                            material: "Baksteen".to_string(),
                            thickness_mm: 100.0,
                            distance_from_interior_mm: Some(260.0),
                            layer_type: ThermalLayerType::Solid,
                            lambda: Some(0.9),
                        },
                    ],
                },
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = &result.project.rooms[0];

        // 3 constructions with same type should be grouped into 1.
        let masonry: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.material_type == MaterialType::Masonry)
            .collect();
        assert_eq!(
            masonry.len(),
            1,
            "3 same-type constructions should be grouped into 1. Got: {:?}",
            masonry.iter().map(|c| &c.description).collect::<Vec<_>>()
        );

        // Total area should be 5 + 8 + 7 = 20.
        assert!(
            (masonry[0].area - 20.0).abs() < 0.01,
            "Grouped area should be 20.0, got {}",
            masonry[0].area
        );

        // Should have a grouping info warning.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert!(
            !group_warnings.is_empty(),
            "Should have a grouping info message. Warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_sfb_based_naming() {
        let import = load_fixture();
        let result = map_thermal_import(import);

        let room_0 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-0")
            .expect("room-0 not found");

        // constr-0: Exterior Wall with layers [Gipsplaat, Minerale wol, Luchtspouw, Baksteen]
        // → SfB 21, layers: Gips, MW, Spouw, Klinker → "21_Gips_MW_Spouw_Klinker"
        let ext_wall = room_0
            .constructions
            .iter()
            .find(|c| {
                c.boundary_type == BoundaryType::Exterior
                    && c.material_type == MaterialType::Masonry
                    && c.vertical_position == VerticalPosition::Wall
            })
            .expect("Should have exterior masonry wall");
        assert_eq!(
            ext_wall.description, "21_Gips_MW_Spouw_Klinker",
            "Exterior wall should have SfB-based name"
        );

        // constr-1: UnheatedSpace Wall with layers [Gipsplaat, Kalkzandsteen, Gipsplaat]
        // → SfB 22, layers: Gips, KZS, Gips → "22_Gips_KZS_Gips"
        let unheated_wall = room_0
            .constructions
            .iter()
            .find(|c| {
                c.boundary_type == BoundaryType::UnheatedSpace
                    && c.material_type == MaterialType::Masonry
            })
            .expect("Should have unheated space masonry wall");
        assert_eq!(
            unheated_wall.description, "22_Gips_KZS_Gips",
            "Unheated wall should have SfB-based name"
        );

        // constr-2: Ground Floor with layers [Tegels, Dekvloer, EPS isolatie, Beton]
        // → SfB 23, layers: Tegels, Dekvloer, EPS, Beton → "23_Tegels_Dekvloer_EPS_Beton"
        let ground_floor = room_0
            .constructions
            .iter()
            .find(|c| {
                c.boundary_type == BoundaryType::Ground
                    && c.material_type == MaterialType::Masonry
            })
            .expect("Should have ground floor element");
        assert_eq!(
            ground_floor.description, "23_Tegels_Dekvloer_EPS_Beton",
            "Ground floor should have SfB-based name"
        );

        // room-2: constr-4 is Exterior Roof with no layers → just "27"
        let room_2 = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "room-2")
            .expect("room-2 not found");
        let roof = room_2
            .constructions
            .iter()
            .find(|c| {
                c.vertical_position == VerticalPosition::Ceiling
                    && c.material_type == MaterialType::Masonry
            })
            .expect("Should have roof element");
        assert_eq!(
            roof.description, "27",
            "Roof without layers should just be SfB code"
        );
    }

    #[test]
    fn test_different_types_not_grouped() {
        // Constructions with different revit_type_name or different boundary_type
        // should NOT be grouped.
        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-08".to_string(),
            project_name: Some("No-Group Test".to_string()),
            rooms: vec![
                ThermalRoom {
                    id: "r1".to_string(),
                    revit_id: None,
                    name: "Kamer".to_string(),
                    room_type: ThermalRoomType::Heated,
                    level: None,
                    area_m2: Some(20.0),
                    height_m: Some(2.6),
                    volume_m3: None,
                    boundary_polygon: None,
                },
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
                ThermalRoom {
                    id: "r-unheated".to_string(),
                    revit_id: None,
                    name: "Berging".to_string(),
                    room_type: ThermalRoomType::Unheated,
                    level: None,
                    area_m2: Some(5.0),
                    height_m: Some(2.6),
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                // Same layer fingerprint but different boundary_type → should NOT group.
                ThermalConstruction {
                    id: "c1".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("N".to_string()),
                    gross_area_m2: 5.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur".to_string()),
                    layers: vec![ThermalLayer {
                        material: "Kalkzandsteen".to_string(),
                        thickness_mm: 100.0,
                        distance_from_interior_mm: Some(0.0),
                        layer_type: ThermalLayerType::Solid,
                        lambda: Some(1.0),
                    }],
                },
                ThermalConstruction {
                    id: "c2".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-unheated".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("S".to_string()),
                    gross_area_m2: 6.0,
                    revit_element_id: None,
                    revit_type_name: Some("Spouwmuur".to_string()),
                    layers: vec![ThermalLayer {
                        material: "Kalkzandsteen".to_string(),
                        thickness_mm: 100.0,
                        distance_from_interior_mm: Some(0.0),
                        layer_type: ThermalLayerType::Solid,
                        lambda: Some(1.0),
                    }],
                },
                // Different layer fingerprint (different material), same boundary_type → should NOT group.
                ThermalConstruction {
                    id: "c3".to_string(),
                    room_a: "r1".to_string(),
                    room_b: "r-out".to_string(),
                    orientation: ThermalOrientation::Wall,
                    compass: Some("E".to_string()),
                    gross_area_m2: 4.0,
                    revit_element_id: None,
                    revit_type_name: Some("Binnenwand 100mm".to_string()),
                    layers: vec![ThermalLayer {
                        material: "Gipsplaat".to_string(),
                        thickness_mm: 12.5,
                        distance_from_interior_mm: Some(0.0),
                        layer_type: ThermalLayerType::Solid,
                        lambda: Some(0.25),
                    }],
                },
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = &result.project.rooms[0];

        // Should have 3 separate masonry elements (nothing grouped).
        let masonry: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.material_type == MaterialType::Masonry)
            .collect();
        assert_eq!(
            masonry.len(),
            3,
            "3 different constructions should remain separate. Got: {:?}",
            masonry.iter().map(|c| (&c.description, c.area)).collect::<Vec<_>>()
        );

        // No grouping warnings (nothing was merged).
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert!(
            group_warnings.is_empty(),
            "Should have no grouping messages. Warnings: {:?}",
            result.warnings
        );
    }

    // ─── Bug B regression tests: adjacent_room_id in grouping key ───

    /// Helper: build a minimal ThermalImport with a heated source room plus the
    /// supplied peer rooms and constructions. Keeps the bug-B tests compact.
    fn make_import_for_adjacent_test(
        peer_rooms: Vec<ThermalRoom>,
        constructions: Vec<ThermalConstruction>,
    ) -> ThermalImport {
        let mut rooms = vec![ThermalRoom {
            id: "r1".to_string(),
            revit_id: None,
            name: "Woonkamer".to_string(),
            room_type: ThermalRoomType::Heated,
            level: None,
            area_m2: Some(30.0),
            height_m: Some(2.6),
            volume_m3: None,
            boundary_polygon: None,
        }];
        rooms.extend(peer_rooms);
        ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Bug B regression".to_string()),
            rooms,
            constructions,
            openings: vec![],
            open_connections: vec![],
        }
    }

    /// Helper: quickly create a peer room with a specific room type.
    fn peer_room(id: &str, name: &str, room_type: ThermalRoomType) -> ThermalRoom {
        ThermalRoom {
            id: id.to_string(),
            revit_id: None,
            name: name.to_string(),
            room_type,
            level: None,
            area_m2: Some(10.0),
            height_m: Some(2.6),
            volume_m3: None,
            boundary_polygon: None,
        }
    }

    /// Helper: build a bare Wall construction between `r1` and `room_b` with a given type.
    fn wall_to(
        id: &str,
        room_b: &str,
        revit_type: &str,
        gross_area_m2: f64,
    ) -> ThermalConstruction {
        ThermalConstruction {
            id: id.to_string(),
            room_a: "r1".to_string(),
            room_b: room_b.to_string(),
            orientation: ThermalOrientation::Wall,
            compass: None,
            gross_area_m2,
            revit_element_id: None,
            revit_type_name: Some(revit_type.to_string()),
            layers: vec![],
        }
    }

    #[test]
    fn test_interior_walls_to_different_rooms_not_merged() {
        // 3 walls, same revit_type_name/boundary_type/orientation, but each adjacent
        // to a different heated room → must remain 3 separate ConstructionElements
        // so the per-wall `adjacent_room_id` survives.
        let input = make_import_for_adjacent_test(
            vec![
                peer_room("r-b", "Keuken", ThermalRoomType::Heated),
                peer_room("r-c", "Slaapkamer", ThermalRoomType::Heated),
                peer_room("r-d", "Badkamer", ThermalRoomType::Heated),
            ],
            vec![
                wall_to("c1", "r-b", "Binnenwand 100mm", 5.0),
                wall_to("c2", "r-c", "Binnenwand 100mm", 6.0),
                wall_to("c3", "r-d", "Binnenwand 100mm", 7.0),
            ],
        );

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let interior: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::AdjacentRoom)
            .collect();
        assert_eq!(
            interior.len(),
            3,
            "3 walls to different rooms must NOT be merged. Got: {:?}",
            interior
                .iter()
                .map(|c| (&c.description, c.adjacent_room_id.clone(), c.area))
                .collect::<Vec<_>>()
        );

        // Each adjacent_room_id must appear exactly once.
        let mut adjacents: Vec<String> = interior
            .iter()
            .map(|c| c.adjacent_room_id.clone().expect("adjacent_room_id missing"))
            .collect();
        adjacents.sort();
        assert_eq!(adjacents, vec!["r-b".to_string(), "r-c".to_string(), "r-d".to_string()]);

        // Areas must match the inputs (nothing summed).
        let areas_by_adj: HashMap<String, f64> = interior
            .iter()
            .map(|c| (c.adjacent_room_id.clone().unwrap(), c.area))
            .collect();
        assert!((areas_by_adj["r-b"] - 5.0).abs() < 0.01);
        assert!((areas_by_adj["r-c"] - 6.0).abs() < 0.01);
        assert!((areas_by_adj["r-d"] - 7.0).abs() < 0.01);

        // No grouping-merge warnings: every group has exactly 1 entry.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert!(
            group_warnings.is_empty(),
            "No grouping should have happened. Warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_interior_walls_to_same_room_still_merged() {
        // 2 walls of the same type between r1 and the same adjacent heated room
        // should still be merged into 1 ConstructionElement (areas summed).
        let input = make_import_for_adjacent_test(
            vec![peer_room("r-b", "Keuken", ThermalRoomType::Heated)],
            vec![
                wall_to("c1", "r-b", "Binnenwand 100mm", 4.0),
                wall_to("c2", "r-b", "Binnenwand 100mm", 3.5),
            ],
        );

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let interior: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::AdjacentRoom)
            .collect();
        assert_eq!(
            interior.len(),
            1,
            "2 walls to same adjacent room must be merged. Got: {:?}",
            interior.iter().map(|c| &c.description).collect::<Vec<_>>()
        );
        assert!(
            (interior[0].area - 7.5).abs() < 0.01,
            "Merged area should be 7.5 m², got {}",
            interior[0].area
        );
        assert_eq!(interior[0].adjacent_room_id.as_deref(), Some("r-b"));

        // A grouping warning should be produced because 2 surfaces were merged.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert_eq!(
            group_warnings.len(),
            1,
            "Expected exactly 1 merge warning. Warnings: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_exterior_walls_still_merged() {
        // 2 exterior walls of the same type → still 1 merged entry.
        // adjacent_room_id is irrelevant for Exterior, so the new key component
        // (None) does not split them.
        let input = make_import_for_adjacent_test(
            vec![peer_room("r-out", "Buiten", ThermalRoomType::Outside)],
            vec![
                wall_to("c1", "r-out", "Spouwmuur 300mm", 10.0),
                wall_to("c2", "r-out", "Spouwmuur 300mm", 12.0),
            ],
        );

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let exterior: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::Exterior)
            .collect();
        assert_eq!(
            exterior.len(),
            1,
            "2 exterior walls of same type must still be merged. Got: {:?}",
            exterior.iter().map(|c| &c.description).collect::<Vec<_>>()
        );
        assert!(
            (exterior[0].area - 22.0).abs() < 0.01,
            "Merged exterior area should be 22.0 m², got {}",
            exterior[0].area
        );
        // Exterior boundaries must never carry an adjacent_room_id.
        assert!(exterior[0].adjacent_room_id.is_none());
    }

    #[test]
    fn test_unheated_space_walls_split_by_adjacent_room() {
        // Walls of the same type adjacent to two different unheated spaces
        // must not be merged — the adjacent_room_id discriminator applies to
        // UnheatedSpace exactly like it does to AdjacentRoom.
        let input = make_import_for_adjacent_test(
            vec![
                peer_room("r-berg", "Berging", ThermalRoomType::Unheated),
                peer_room("r-zolder", "Zolder", ThermalRoomType::Unheated),
            ],
            vec![
                wall_to("c1", "r-berg", "Binnenwand 100mm", 4.0),
                wall_to("c2", "r-zolder", "Binnenwand 100mm", 5.0),
            ],
        );

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let unheated: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| c.boundary_type == BoundaryType::UnheatedSpace)
            .collect();
        assert_eq!(
            unheated.len(),
            2,
            "2 walls to different unheated spaces must NOT be merged. Got: {:?}",
            unheated
                .iter()
                .map(|c| (&c.description, c.adjacent_room_id.clone(), c.area))
                .collect::<Vec<_>>()
        );
        let mut adjacents: Vec<String> = unheated
            .iter()
            .map(|c| c.adjacent_room_id.clone().expect("adjacent_room_id missing"))
            .collect();
        adjacents.sort();
        assert_eq!(adjacents, vec!["r-berg".to_string(), "r-zolder".to_string()]);

        // No merge warnings: each group contains exactly 1 wall.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert!(
            group_warnings.is_empty(),
            "No grouping should have happened. Warnings: {:?}",
            result.warnings
        );
    }

    // ─── Catalog refactor tests (bug A) ───────────────────────────────

    /// Helper: build a single solid layer.
    fn solid(material: &str, thickness_mm: f64, lambda: f64) -> ThermalLayer {
        ThermalLayer {
            material: material.to_string(),
            thickness_mm,
            distance_from_interior_mm: None,
            layer_type: ThermalLayerType::Solid,
            lambda: Some(lambda),
        }
    }

    /// Helper: build a heated room with the given id, name and area.
    fn heated_room(id: &str, name: &str) -> ThermalRoom {
        ThermalRoom {
            id: id.to_string(),
            revit_id: None,
            name: name.to_string(),
            room_type: ThermalRoomType::Heated,
            level: None,
            area_m2: Some(20.0),
            height_m: Some(2.6),
            volume_m3: None,
            boundary_polygon: None,
        }
    }

    /// Helper: build a wall construction between `room_a` and `room_b`.
    fn wall_with_layers(
        id: &str,
        room_a: &str,
        room_b: &str,
        gross_area_m2: f64,
        layers: Vec<ThermalLayer>,
    ) -> ThermalConstruction {
        ThermalConstruction {
            id: id.to_string(),
            room_a: room_a.to_string(),
            room_b: room_b.to_string(),
            orientation: ThermalOrientation::Wall,
            compass: None,
            gross_area_m2,
            revit_element_id: None,
            revit_type_name: None,
            layers,
        }
    }

    #[test]
    fn test_phase2_splits_by_layer_fingerprint_when_revit_type_name_empty() {
        // Regression: PyRevit ThermalExport often emits surfaces with
        // `revit_type_name = None`. Previously all such surfaces in one room
        // collapsed onto the key `("onbekend", Exterior, Wall, None)` and got
        // merged into one ConstructionElement, regardless of their physical
        // composition. With layer-fingerprint based grouping they must stay
        // distinct when their layer stacks differ.
        let layers_spouwmuur = vec![
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 120.0, 0.023),
            solid("Baksteen", 100.0, 0.9),
        ];
        let layers_houtskelet = vec![
            solid("Gipsplaat", 12.5, 0.25),
            solid("Minerale wol", 140.0, 0.035),
            solid("Multiplex", 18.0, 0.14),
        ];
        let layers_drijflichaam = vec![
            solid("Beton", 200.0, 2.0),
            solid("PUR isolatie", 80.0, 0.025),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Fingerprint split test".to_string()),
            rooms: vec![
                heated_room("r1", "Woonkamer"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                // All 3 walls: revit_type_name = None, Exterior, Wall — but 3
                // different layer fingerprints.
                wall_with_layers("c1", "r1", "r-out", 6.0, layers_spouwmuur),
                wall_with_layers("c2", "r1", "r-out", 4.0, layers_houtskelet),
                wall_with_layers("c3", "r1", "r-out", 5.0, layers_drijflichaam),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let exterior_walls: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| {
                c.boundary_type == BoundaryType::Exterior
                    && c.vertical_position == VerticalPosition::Wall
            })
            .collect();
        assert_eq!(
            exterior_walls.len(),
            3,
            "3 walls with distinct layer fingerprints must stay separate, got: {:?}",
            exterior_walls
                .iter()
                .map(|c| (&c.description, c.area))
                .collect::<Vec<_>>(),
        );

        // Areas must match the inputs (nothing summed).
        let areas: std::collections::HashSet<u64> = exterior_walls
            .iter()
            .map(|c| (c.area * 100.0).round() as u64)
            .collect();
        assert_eq!(
            areas,
            std::collections::HashSet::from([600u64, 400u64, 500u64]),
            "Each wall must keep its original area",
        );

        // Every wall must have its own catalog_ref, and all 3 must differ.
        let catalog_refs: Vec<&str> = exterior_walls
            .iter()
            .map(|c| {
                c.catalog_ref
                    .as_deref()
                    .expect("wall must carry a catalog_ref")
            })
            .collect();
        let unique_refs: std::collections::HashSet<&str> =
            catalog_refs.iter().copied().collect();
        assert_eq!(
            unique_refs.len(),
            3,
            "3 distinct fingerprints must map to 3 distinct catalog entries, got refs {:?}",
            catalog_refs,
        );

        // No phase-2 merge warnings: every group has exactly 1 entry.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert!(
            group_warnings.is_empty(),
            "No grouping should have happened, got warnings: {:?}",
            group_warnings,
        );
    }

    #[test]
    fn test_phase2_merges_walls_with_same_layer_fingerprint() {
        // Two exterior walls, both with revit_type_name = None, but sharing the
        // same layer stack → must merge into a single ConstructionElement with
        // summed area.
        let layers = vec![
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 120.0, 0.023),
            solid("Baksteen", 100.0, 0.9),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Fingerprint merge test".to_string()),
            rooms: vec![
                heated_room("r1", "Woonkamer"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                wall_with_layers("c1", "r1", "r-out", 6.0, layers.clone()),
                wall_with_layers("c2", "r1", "r-out", 4.5, layers.clone()),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = result
            .project
            .rooms
            .iter()
            .find(|r| r.id == "r1")
            .expect("r1 not found");

        let exterior_walls: Vec<&ConstructionElement> = room
            .constructions
            .iter()
            .filter(|c| {
                c.boundary_type == BoundaryType::Exterior
                    && c.vertical_position == VerticalPosition::Wall
            })
            .collect();
        assert_eq!(
            exterior_walls.len(),
            1,
            "2 walls with identical layer fingerprints must merge into 1, got: {:?}",
            exterior_walls
                .iter()
                .map(|c| (&c.description, c.area))
                .collect::<Vec<_>>(),
        );
        assert!(
            (exterior_walls[0].area - 10.5).abs() < 0.01,
            "Merged area must be 10.5 m², got {}",
            exterior_walls[0].area,
        );

        // Exactly one merge warning expected.
        let group_warnings: Vec<&String> = result
            .warnings
            .iter()
            .filter(|w| w.contains("samengevoegd"))
            .collect();
        assert_eq!(
            group_warnings.len(),
            1,
            "Expected exactly 1 merge warning. Warnings: {:?}",
            result.warnings,
        );
    }

    #[test]
    fn test_thermal_import_woonboot_fixture() {
        let json = include_str!("../../../../tests/fixtures/thermal_import_woonboot.json");
        let input: ThermalImport =
            serde_json::from_str(json).expect("woonboot fixture failed to parse");
        let result = map_thermal_import(input);

        // Expected from the real 3056 export: ~83 raw constructions collapse
        // to roughly 36 unique layer fingerprints. The exact number can drift
        // a little if the export is regenerated, so accept a 30..=45 window.
        let n = result.construction_catalog.len();
        assert!(
            (30..=45).contains(&n),
            "catalog moet tussen 30 en 45 entries hebben, is {}",
            n
        );

        // Every non-opening element must reference a real catalog entry.
        let catalog_ids: std::collections::HashSet<&str> = result
            .construction_catalog
            .iter()
            .map(|e| e.id.as_str())
            .collect();
        for room in &result.project.rooms {
            for elem in &room.constructions {
                let desc_lower = elem.description.to_lowercase();
                let is_opening = desc_lower.contains("raam")
                    || desc_lower.contains("deur")
                    || desc_lower.contains("vliesgevel")
                    || elem.material_type == MaterialType::NonMasonry;
                if is_opening {
                    continue;
                }
                let cref = elem.catalog_ref.as_deref().unwrap_or_else(|| {
                    panic!(
                        "Non-opening element '{}' (room {}) ontbreekt catalog_ref",
                        elem.description, room.id
                    )
                });
                assert!(
                    catalog_ids.contains(cref),
                    "catalog_ref {} bestaat niet in de catalog",
                    cref
                );
            }
        }
    }

    #[test]
    fn test_catalog_resolves_description_collision_with_thickness() {
        // Two rooms, each with a wall whose layers map to the same SfB
        // material abbreviations but with different total thicknesses.
        // Both catalog entries must end up with a `_{thickness}mm` suffix.
        let layers_thin = vec![
            solid("Stucwerk", 10.0, 0.5),
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 100.0, 0.023),
        ];
        let layers_thick = vec![
            solid("Stucwerk", 10.0, 0.5),
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 120.0, 0.023),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Collision thickness test".to_string()),
            rooms: vec![
                heated_room("r1", "Kamer 1"),
                heated_room("r2", "Kamer 2"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                wall_with_layers("c-thin", "r1", "r-out", 10.0, layers_thin),
                wall_with_layers("c-thick", "r2", "r-out", 12.0, layers_thick),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);

        // Two distinct fingerprints → two catalog entries.
        assert_eq!(result.construction_catalog.len(), 2);

        let descs: Vec<&str> = result
            .construction_catalog
            .iter()
            .map(|e| e.description.as_str())
            .collect();
        // Both entries must carry a thickness suffix (the *earlier* entry is
        // also rewritten — that's the spec contract).
        assert!(
            descs.iter().all(|d| d.contains("mm")),
            "Both colliding entries should carry a `_<mm>mm` suffix, got {:?}",
            descs
        );
        // The two suffixes must be 210mm and 230mm respectively.
        assert!(descs.iter().any(|d| d.contains("210mm")), "missing _210mm in {:?}", descs);
        assert!(descs.iter().any(|d| d.contains("230mm")), "missing _230mm in {:?}", descs);
    }

    #[test]
    fn test_catalog_collision_tie_on_thickness_uses_letter_fallback() {
        // Two fingerprints that produce the same SfB name AND the same total
        // thickness. The fingerprint differs only because the second wall uses
        // a different lambda value (lambda is excluded from the fingerprint
        // per spec, so we must force a difference via material naming).
        // We use two different material names that map to the same SfB
        // abbreviation: "Gipsplaat" and "Gipskarton" both → "Gips".
        let layers_a = vec![
            solid("Gipsplaat", 12.5, 0.21),
            solid("Kalkzandsteen", 100.0, 1.0),
        ];
        let layers_b = vec![
            solid("Gipskarton", 12.5, 0.21),
            solid("Kalkzandsteen", 100.0, 1.0),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Tie test".to_string()),
            rooms: vec![
                heated_room("r1", "Kamer 1"),
                heated_room("r2", "Kamer 2"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                wall_with_layers("c-a", "r1", "r-out", 10.0, layers_a),
                wall_with_layers("c-b", "r2", "r-out", 12.0, layers_b),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);

        // Two distinct fingerprints (different material name strings) → two entries.
        assert_eq!(
            result.construction_catalog.len(),
            2,
            "Expected 2 entries, got {:?}",
            result
                .construction_catalog
                .iter()
                .map(|e| (&e.description, &e.layers))
                .collect::<Vec<_>>()
        );

        let descs: Vec<&str> = result
            .construction_catalog
            .iter()
            .map(|e| e.description.as_str())
            .collect();
        // Both entries collide on SfB name AND on total thickness, so both
        // must end with `_a` / `_b`.
        let has_a = descs.iter().any(|d| d.ends_with("_a"));
        let has_b = descs.iter().any(|d| d.ends_with("_b"));
        assert!(
            has_a && has_b,
            "Expected `_a` and `_b` letter fallbacks, got {:?}",
            descs
        );
    }

    #[test]
    fn test_catalog_no_collision_no_suffix() {
        // Two rooms with the same wall fingerprint → exactly 1 catalog entry,
        // no thickness or letter suffix.
        let layers = vec![
            solid("Stucwerk", 10.0, 0.5),
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 100.0, 0.023),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("No collision".to_string()),
            rooms: vec![
                heated_room("r1", "Kamer 1"),
                heated_room("r2", "Kamer 2"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                wall_with_layers("c1", "r1", "r-out", 10.0, layers.clone()),
                wall_with_layers("c2", "r2", "r-out", 12.0, layers),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);

        assert_eq!(result.construction_catalog.len(), 1);
        let entry = &result.construction_catalog[0];
        assert!(
            !entry.description.contains("mm"),
            "No collision so no `_<mm>mm` suffix expected, got {}",
            entry.description
        );
        assert!(
            !entry.description.ends_with("_a") && !entry.description.ends_with("_b"),
            "No collision so no letter fallback expected, got {}",
            entry.description
        );
    }

    #[test]
    fn test_catalog_deduplicates_across_rooms() {
        // Two rooms, each with a wall sharing the same fingerprint. The
        // catalog must contain a single entry whose `surface_count == 2` and
        // whose `total_area_m2` sums both rooms' areas.
        let layers = vec![
            solid("Stucwerk", 10.0, 0.5),
            solid("Kalkzandsteen", 100.0, 1.0),
            solid("PIR isolatie", 100.0, 0.023),
        ];

        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Cross-room dedup".to_string()),
            rooms: vec![
                heated_room("r1", "Kamer 1"),
                heated_room("r2", "Kamer 2"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![
                wall_with_layers("c1", "r1", "r-out", 10.0, layers.clone()),
                wall_with_layers("c2", "r2", "r-out", 15.0, layers),
            ],
            openings: vec![],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);

        assert_eq!(result.construction_catalog.len(), 1);
        let entry = &result.construction_catalog[0];
        assert_eq!(entry.surface_count, 2, "should aggregate 2 source surfaces");
        assert!(
            (entry.total_area_m2 - 25.0).abs() < 0.001,
            "total_area_m2 should be 25.0, got {}",
            entry.total_area_m2
        );

        // Both rooms' construction elements must reference the same catalog id.
        let r1 = result.project.rooms.iter().find(|r| r.id == "r1").unwrap();
        let r2 = result.project.rooms.iter().find(|r| r.id == "r2").unwrap();
        let r1_ref = r1.constructions[0].catalog_ref.clone();
        let r2_ref = r2.constructions[0].catalog_ref.clone();
        assert_eq!(r1_ref, Some(entry.id.clone()));
        assert_eq!(r2_ref, Some(entry.id.clone()));
    }

    #[test]
    fn test_openings_have_none_catalog_ref() {
        // A room with one wall + one window. The window element must have
        // `catalog_ref == None` because openings are intentionally outside
        // the catalog (besluit 2a in the spec).
        let layers = vec![solid("Kalkzandsteen", 100.0, 1.0)];
        let input = ThermalImport {
            version: "1.0".to_string(),
            source: "test".to_string(),
            exported_at: "2026-04-09".to_string(),
            project_name: Some("Openings catalog_ref test".to_string()),
            rooms: vec![
                heated_room("r1", "Woonkamer"),
                ThermalRoom {
                    id: "r-out".to_string(),
                    revit_id: None,
                    name: "Buiten".to_string(),
                    room_type: ThermalRoomType::Outside,
                    level: None,
                    area_m2: None,
                    height_m: None,
                    volume_m3: None,
                    boundary_polygon: None,
                },
            ],
            constructions: vec![wall_with_layers("c1", "r1", "r-out", 10.0, layers)],
            openings: vec![ThermalOpening {
                id: "o1".to_string(),
                construction_id: "c1".to_string(),
                opening_type: ThermalOpeningType::Window,
                width_mm: 1200.0,
                height_mm: 1500.0,
                sill_height_mm: None,
                u_value: Some(1.4),
                revit_element_id: None,
                revit_type_name: Some("Raam standaard".to_string()),
            }],
            open_connections: vec![],
        };

        let result = map_thermal_import(input);
        let room = &result.project.rooms[0];

        let opening = room
            .constructions
            .iter()
            .find(|c| c.material_type == MaterialType::NonMasonry)
            .expect("opening element missing");
        assert!(
            opening.catalog_ref.is_none(),
            "opening must NOT have a catalog_ref, got {:?}",
            opening.catalog_ref
        );

        // The wall element does have a catalog_ref.
        let wall = room
            .constructions
            .iter()
            .find(|c| c.material_type == MaterialType::Masonry)
            .expect("wall element missing");
        assert!(wall.catalog_ref.is_some(), "wall must have a catalog_ref");
    }
}
