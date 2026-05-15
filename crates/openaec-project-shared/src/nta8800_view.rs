//! View-mapper: `ProjectV2` (shared + geometry) → NTA 8800 building model.
//!
//! Produceert de `Rekenzone[]` + `EnergiefunctieRuimte[]` + `Construction[]` +
//! `Window[]` structuren die `nta8800-demand` en `nta8800-cooling` als input
//! verwachten. Hiermee draait dezelfde geometrie zowel ISSO 51 als NTA 8800,
//! per ADR-002.
//!
//! ## V1 scope
//!
//! - **Eén Rekenzone per project** (= gehele gebouw). Multi-zone werk is V2.
//! - **Eén EFR per Space.** Aggregatie van meerdere Spaces in één EFR komt
//!   later wanneer multi-rekenzone projecten ondersteund worden.
//! - **`UsageFunction`** afgeleid uit `SharedProject.building_type`. Voor
//!   utiliteit-subtypes is de mapping 1-op-1; voor woningen krijgen alle
//!   subtypes [`UsageFunction::Woonfunctie`].
//! - **Constructies:** layered constructions worden 1-op-1 omgezet; pure
//!   U-waarde-only constructies (zonder layers) krijgen een synthetische
//!   single-layer fallback met R_total = 1/U − R_si − R_se zodat de
//!   layered-rekenkant blijft kloppen.
//! - **Openings (ramen):** alleen `OpeningKind::Window` wordt geconverteerd
//!   naar [`nta8800_model::Window`]. Deuren worden als opaak onderdeel van
//!   de parent Construction beschouwd (zelfde U-waarde).
//!
//! ## V2 backlog
//!
//! - Multi-rekenzone splitsing op gebouw-segmenten / verdiepingen / EFR-groepen
//! - Per-EFR `UsageFunction` override (gemengd gebruik)
//! - Schaduw / overstek-modeling (komt uit `nta8800-demand::solar_gains` V2)
//! - Thermal bridges aggregatie naar [`nta8800_model::ThermalBridgeCategory`]

use nta8800_model::geometry::{Construction as N8Construction, ConstructionLayer as N8Layer};
use nta8800_model::geometry::window::Window as N8Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::units::{Area, ThermalTransmittance};
use nta8800_model::zoning::{EnergiefunctieRuimte, Rekenzone, UsageFunction};
use nta8800_model::ModelResult;

use crate::geometry::{
    BoundaryKind, Construction as SharedConstruction, ConstructionKind, Opening,
    OpeningKind, SharedGeometry, Space,
};
use crate::project::ProjectV2;
use crate::shared::{BuildingTypeShared, SharedProject, UtilityType};

/// Het resultaat van een geometry-conversie. Alle vectors zijn parallel
/// in de zin dat `Construction.id`s gerefereerd worden door
/// `Window.construction_id`, en `Rekenzone.construction_ids` /
/// `Rekenzone.efr_ids` cross-verwijzen naar respectievelijk
/// `Construction.id` en `EnergiefunctieRuimte.id`.
#[derive(Debug, Clone, Default)]
pub struct Nta8800View {
    /// Eén rekenzone (V1 — gehele gebouw als één zone).
    pub rekenzones: Vec<Rekenzone>,
    /// EFR's, één per `SharedGeometry::Space`.
    pub efrs: Vec<EnergiefunctieRuimte>,
    /// Opaque constructies (wanden/vloeren/daken) en deuren.
    pub constructions: Vec<N8Construction>,
    /// Ramen (transparante openingen).
    pub windows: Vec<N8Window>,
}

/// Produceer een [`Nta8800View`] uit het hele `ProjectV2`.
///
/// # Errors
///
/// Geeft [`nta8800_model::ModelError`] terug bij bereikfouten (bv. helling
/// buiten 0..=180°). Geometrie-velden met fysieke onmogelijkheden (negatieve
/// oppervlakte, U=0) worden in V1 niet geweigerd — kalker validatie volgt
/// in F7 wanneer de UI de invoer bewaakt.
pub fn project_to_nta8800(project: &ProjectV2) -> ModelResult<Nta8800View> {
    geometry_to_nta8800(&project.shared, &project.geometry)
}

/// Variant van [`project_to_nta8800`] die direct met `SharedProject` +
/// `SharedGeometry` werkt. Handig voor unit tests en wanneer alleen de
/// geometrie nodig is.
///
/// # Errors
/// Zie [`project_to_nta8800`].
pub fn geometry_to_nta8800(
    shared: &SharedProject,
    geometry: &SharedGeometry,
) -> ModelResult<Nta8800View> {
    let gebouw_id = "gebouw_1".to_string();
    let zone_id = "zone_1".to_string();
    let usage_function = map_usage_function(&shared.building_type);

    let mut all_constructions: Vec<N8Construction> = Vec::new();
    let mut all_windows: Vec<N8Window> = Vec::new();
    let mut efrs: Vec<EnergiefunctieRuimte> = Vec::new();

    let mut total_floor_area: Area = 0.0;
    let mut total_volume: f64 = 0.0;

    for space in &geometry.spaces {
        let floor_area: Area = space.floor_area_m2;
        let height_m = space.height_m;
        total_floor_area += floor_area;
        total_volume += floor_area * height_m;

        efrs.push(EnergiefunctieRuimte {
            id: space.id.clone(),
            name: space.name.clone(),
            rekenzone_id: zone_id.clone(),
            floor_area,
            usage_function,
        });

        for construction in space.constructions.iter() {
            let (n8_constr, opening_windows) =
                map_construction(&space.id, construction)?;
            all_constructions.push(n8_constr);
            all_windows.extend(opening_windows);
        }
    }

    // Volume fallback: gebruik gross_floor_area_m2 × 2.7 als geen Spaces.
    if total_volume <= 0.0 {
        let fallback_area = shared.gross_floor_area_m2.unwrap_or(0.0);
        total_floor_area = total_floor_area.max(fallback_area);
        total_volume = fallback_area * 2.7;
    }

    let rekenzone = Rekenzone {
        id: zone_id.clone(),
        name: shared.name.clone(),
        gebouw_id,
        floor_area: total_floor_area,
        volume: total_volume,
        efr_ids: efrs.iter().map(|e| e.id.clone()).collect(),
        constructions: all_constructions.iter().map(|c| c.id.clone()).collect(),
        windows: all_windows.iter().map(|w| w.id.clone()).collect(),
        openings: Vec::new(),
        thermal_bridges_linear: Vec::new(),
        thermal_bridges_point: Vec::new(),
    };

    Ok(Nta8800View {
        rekenzones: vec![rekenzone],
        efrs,
        constructions: all_constructions,
        windows: all_windows,
    })
}

/// Map het V2-gebouwtype naar de juiste NTA 8800 [`UsageFunction`].
///
/// Voor woning-subtypes is de mapping uniform [`UsageFunction::Woonfunctie`] —
/// het isolatie/koel-gedrag hangt niet van portiek/galerij/etc. af binnen
/// NTA 8800 hoofdstuk 7/10. Voor utiliteit is de mapping per subtype.
#[must_use]
pub fn map_usage_function(bt: &BuildingTypeShared) -> UsageFunction {
    match bt {
        BuildingTypeShared::Woning { .. } => UsageFunction::Woonfunctie,
        BuildingTypeShared::Utiliteit { subtype } => match subtype {
            UtilityType::Office => UsageFunction::Kantoorfunctie,
            UtilityType::Education => UsageFunction::Onderwijsfunctie,
            UtilityType::Assembly => UsageFunction::Bijeenkomstfunctie,
            UtilityType::Healthcare => UsageFunction::Gezondheidszorgfunctie,
            UtilityType::Lodging => UsageFunction::Logiesfunctie,
            UtilityType::Sport => UsageFunction::Sportfunctie,
            UtilityType::Retail => UsageFunction::Winkelfunctie,
            UtilityType::Industrial => UsageFunction::Industriefunctie,
            UtilityType::Other => UsageFunction::OverigeGebruiksfunctie,
        },
    }
}

/// Map één `SharedGeometry::Construction` naar de NTA 8800 equivalent
/// plus losse `Window` entries voor de transparante openingen.
fn map_construction(
    space_id: &str,
    c: &SharedConstruction,
) -> ModelResult<(N8Construction, Vec<N8Window>)> {
    let (r_si, r_se) = surface_resistances(c.kind, c.boundary);

    let layers: Vec<N8Layer> = if c.layers.is_empty() {
        // Geen layered input — synthetische single-layer met R_total = 1/U.
        // Wordt door consumers bij voorkeur via Construction.r_total() gelezen,
        // maar laat de individuele materiaal-info ongewenst leeg.
        let r_total: f64 = if c.u_value > 0.0 {
            1.0 / c.u_value
        } else {
            0.0
        };
        let r_layers = (r_total - r_si - r_se).max(0.001);
        // Synthetische "Equivalent layer" — 1 m dikte met λ zo gekozen dat
        // R == r_layers. Geen materiaal-naam suggereert "user-input U-waarde".
        let lambda = 1.0 / r_layers;
        vec![N8Layer {
            material_name: format!("synthetic_u_{:.3}", c.u_value),
            thickness: 1.0,
            lambda,
        }]
    } else {
        c.layers
            .iter()
            .map(|l| N8Layer {
                material_name: l.material.clone(),
                thickness: l.thickness_mm / 1000.0,
                lambda: if l.lambda_w_per_mk > 0.0 {
                    l.lambda_w_per_mk
                } else {
                    // Pre-computed R-waarde: thickness=1 m, λ=1/R zodat d/λ=R.
                    let r = l.r_m2k_per_w.unwrap_or(0.001).max(0.001);
                    1.0 / r
                },
            })
            .collect()
    };

    let n8_construction = N8Construction {
        id: c.id.clone(),
        name: c.description.clone(),
        layers,
        r_si,
        r_se,
    };

    let mut windows: Vec<N8Window> = Vec::new();
    for opening in c.openings.iter() {
        if let OpeningKind::Window = opening.kind {
            let window = map_window(space_id, c, opening)?;
            windows.push(window);
        }
        // Deuren: opaak deel, doen niet mee in window-lijst; hun U-waarde
        // wordt impliciet via de parent Construction-mass-flow gehandled.
        // V2: aparte Door-struct in NTA 8800-model overwegen.
    }

    Ok((n8_construction, windows))
}

/// Map een `Window`-opening naar [`nta8800_model::Window`].
fn map_window(
    space_id: &str,
    parent: &SharedConstruction,
    opening: &Opening,
) -> ModelResult<N8Window> {
    let orientation = parent
        .orientation_deg
        .map(orientation_from_degrees)
        .unwrap_or(Orientation::Horizontaal);
    let tilt = Tilt::new(parent.slope_deg.unwrap_or(90.0))?;

    let u_value: ThermalTransmittance = opening.u_value;

    let _ = space_id; // Window heeft geen direct space-veld; mapping via EFR loopt
                       // via Rekenzone.window_ids + cross-ref Construction.id.
    Ok(N8Window {
        id: opening.id.clone(),
        construction_id: parent.id.clone(),
        area: opening.area_m2,
        orientation,
        tilt,
        u_value,
        g_value: opening.g_value.unwrap_or(0.6),
        frame_fraction: opening.frame_fraction.unwrap_or(0.25),
    })
}

/// Mapping ISO 6946 R_si / R_se per oriëntatie + boundary-type.
///
/// - Verticale wand: R_si=0.13, R_se=0.04
/// - Vloer (richting omlaag): R_si=0.17, R_se=0.04
/// - Plafond/dak (richting omhoog): R_si=0.10, R_se=0.04
/// - Grondvlak: R_se=0 (grondmodel verzorgt buitenzijde)
/// - Open water: idem grondvlak
#[must_use]
pub fn surface_resistances(kind: ConstructionKind, boundary: BoundaryKind) -> (f64, f64) {
    let r_si = match kind {
        ConstructionKind::Wall => 0.13,
        ConstructionKind::Floor => 0.17,
        ConstructionKind::Ceiling | ConstructionKind::Roof => 0.10,
    };
    let r_se = match boundary {
        BoundaryKind::Ground | BoundaryKind::OpenWater => 0.0,
        _ => 0.04,
    };
    (r_si, r_se)
}

/// Map een vrije oriëntatiehoek in graden naar de discrete NTA 8800
/// [`Orientation`]-categorieën. 0° = noord, 90° = oost, 180° = zuid,
/// 270° = west. Bin-breedte 45° met afgrendelen op 360°-modulo.
#[must_use]
pub fn orientation_from_degrees(degrees: f64) -> Orientation {
    let normalized = ((degrees % 360.0) + 360.0) % 360.0;
    let bin = ((normalized + 22.5) / 45.0).floor() as i32 % 8;
    match bin {
        0 => Orientation::Noord,
        1 => Orientation::NoordOost,
        2 => Orientation::Oost,
        3 => Orientation::ZuidOost,
        4 => Orientation::Zuid,
        5 => Orientation::ZuidWest,
        6 => Orientation::West,
        7 => Orientation::NoordWest,
        _ => Orientation::Noord,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Construction as SC, Opening, Space};
    use crate::shared::{BuildingTypeShared, ResidentialType, SharedProject, UtilityType};

    fn sample_space() -> Space {
        Space {
            id: "S1".into(),
            name: "Woonkamer".into(),
            function: None,
            floor_area_m2: 30.0,
            height_m: 2.7,
            theta_i_winter_c: Some(20.0),
            theta_i_summer_c: Some(25.0),
            constructions: vec![SC {
                id: "C1".into(),
                description: "Zuidgevel".into(),
                kind: ConstructionKind::Wall,
                boundary: BoundaryKind::Exterior,
                area_m2: 15.0,
                u_value: 0.25,
                orientation_deg: Some(180.0),
                slope_deg: Some(90.0),
                openings: vec![Opening {
                    id: "W1".into(),
                    kind: OpeningKind::Window,
                    area_m2: 3.0,
                    u_value: 1.4,
                    g_value: Some(0.6),
                    frame_fraction: Some(0.2),
                }],
                layers: vec![],
                adjacent_space_id: None,
                psi_thermal_bridge: None,
            }],
        }
    }

    #[test]
    fn maps_residential_to_woonfunctie() {
        assert_eq!(
            map_usage_function(&BuildingTypeShared::Woning {
                subtype: ResidentialType::Detached
            }),
            UsageFunction::Woonfunctie
        );
    }

    #[test]
    fn maps_office_to_kantoorfunctie() {
        assert_eq!(
            map_usage_function(&BuildingTypeShared::Utiliteit {
                subtype: UtilityType::Office
            }),
            UsageFunction::Kantoorfunctie
        );
    }

    #[test]
    fn orientation_zuid_at_180() {
        assert_eq!(orientation_from_degrees(180.0), Orientation::Zuid);
    }

    #[test]
    fn orientation_wraps_negative() {
        assert_eq!(orientation_from_degrees(-90.0), Orientation::West);
    }

    #[test]
    fn surface_resistances_vertical_wall_exterior() {
        assert_eq!(
            surface_resistances(ConstructionKind::Wall, BoundaryKind::Exterior),
            (0.13, 0.04)
        );
    }

    #[test]
    fn surface_resistances_ground_floor() {
        let (_, r_se) = surface_resistances(ConstructionKind::Floor, BoundaryKind::Ground);
        assert_eq!(r_se, 0.0);
    }

    #[test]
    fn geometry_to_nta8800_single_space_woning() {
        let shared = SharedProject::new("Test");
        let geometry = SharedGeometry {
            spaces: vec![sample_space()],
        };
        let view = geometry_to_nta8800(&shared, &geometry).unwrap();
        assert_eq!(view.rekenzones.len(), 1);
        assert_eq!(view.efrs.len(), 1);
        assert_eq!(view.constructions.len(), 1);
        assert_eq!(view.windows.len(), 1);
        assert_eq!(view.rekenzones[0].floor_area, 30.0);
        assert!((view.rekenzones[0].volume - 30.0 * 2.7).abs() < 1e-9);
        assert_eq!(view.efrs[0].usage_function, UsageFunction::Woonfunctie);
        assert_eq!(view.windows[0].orientation, Orientation::Zuid);
    }

    #[test]
    fn empty_geometry_with_gross_area_fallback() {
        let mut shared = SharedProject::new("Empty");
        shared.gross_floor_area_m2 = Some(120.0);
        let geometry = SharedGeometry::default();
        let view = geometry_to_nta8800(&shared, &geometry).unwrap();
        assert_eq!(view.rekenzones.len(), 1);
        assert_eq!(view.rekenzones[0].floor_area, 120.0);
        assert!((view.rekenzones[0].volume - 120.0 * 2.7).abs() < 1e-9);
        assert!(view.efrs.is_empty());
    }

    #[test]
    fn synthetic_layer_preserves_u_value_via_r_total() {
        let shared = SharedProject::new("X");
        let geometry = SharedGeometry {
            spaces: vec![Space {
                id: "S1".into(),
                name: "X".into(),
                function: None,
                floor_area_m2: 10.0,
                height_m: 2.7,
                theta_i_winter_c: None,
                theta_i_summer_c: None,
                constructions: vec![SC {
                    id: "C1".into(),
                    description: "wall".into(),
                    kind: ConstructionKind::Wall,
                    boundary: BoundaryKind::Exterior,
                    area_m2: 20.0,
                    u_value: 0.25,
                    orientation_deg: None,
                    slope_deg: Some(90.0),
                    openings: vec![],
                    layers: vec![],
                    adjacent_space_id: None,
                    psi_thermal_bridge: None,
                }],
            }],
        };
        let view = geometry_to_nta8800(&shared, &geometry).unwrap();
        let c = &view.constructions[0];
        // R_total = R_si + R_layers + R_se ≈ 1/U = 4.0
        let r_total = c.r_total();
        assert!((r_total - 4.0).abs() < 0.01, "R_total = {}", r_total);
    }
}
