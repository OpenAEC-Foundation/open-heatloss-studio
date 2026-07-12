//! F6 fase 2 — brug van de gevel-georiënteerde [`BengGeometry`] naar de
//! ruimte-georiënteerde [`SharedGeometry`] die de gevalideerde demand-keten voedt.
//!
//! ## Waarom een brug en géén tweede rekenpad
//!
//! [`crate::compute_beng`] draait de demand-tak (transmissie §8.1, ventilatie
//! §11, maandbalans §7) via [`crate::compute_tojuli_full`], die uitsluitend een
//! [`SharedGeometry`] (spaces → constructions → openings) leest. Om de
//! gevel-georiënteerde BENG-invoer door diezelfde — reeds tegen de goldens
//! gepinde — keten te halen, vertaalt deze module `BengGeometry` naar een
//! **equivalente `SharedGeometry`**. Er komt dus géén parallel rekenpad bij; de
//! brug zet alleen de invoer om.
//!
//! ## De kern-heroriëntatie: buiten-oppervlak per gevel
//!
//! De ISSO 51-`SharedGeometry` wordt in de studio met **binnen**-oppervlakten
//! per kamer gevuld; NTA 8800/Uniec rekent met **buiten**-oppervlakten per gevel
//! op rekenzone-niveau. Die mismatch is de hoofdverdachte van de bewezen
//! Q_H;nd-onderschatting (BENG 1 −26 % op Aalten). De brug zet daarom
//! [`BengBoundary::bruto_buiten_opp_m2`] 1-op-1 op [`Construction::area_m2`], zodat
//! de transmissie-tak (`Σ A_buiten · U`) het grotere buitenschil-oppervlak ziet.
//!
//! ## Conventie-keuze — spiegelt exact de bestaande keten
//!
//! De brug volgt **exact** de conventie die de bestaande `.oes.json`-converter
//! (`oes_to_projectv2`) hanteert, zodat de gevalideerde keten byte-voor-byte
//! dezelfde structuur verwerkt en alleen de oppervlakte-bron verandert:
//!
//! - **Eén [`Construction`] per gevel** met `area_m2 = bruto_buiten_opp_m2`
//!   (bruto, **inclusief** ramen/deuren) en de opake U-waarde. De transmissie-tak
//!   ([`build_transmission_elements`](crate::tojuli)) telt `A_bruto · U_opaak`;
//!   ramen tellen daar **niet** apart bij (openings voeden alleen de zonwinst).
//!   Dat is dezelfde vereenvoudiging als de oes-keten (ramen transmitteren op
//!   opake U in de demand-tak); zo isoleert de meting het effect van de
//!   buiten-oppervlakte-bron. De certified raam-U leeft wél op de opening
//!   ([`Opening::u_value`]) en voedt de TOjuli-noemer (`opening_h`, §5.7.2).
//! - **Ramen → [`Opening`]** per kozijnmerk-plaatsing: een `Raam` wordt
//!   [`OpeningKind::Window`] met g-waarde (zonwinst); een `Deur`/`PaneelInKozijn`
//!   wordt [`OpeningKind::Door`] zonder g-waarde (opaak, geen zontoetreding).
//!
//! ## Rc → U (NTA 8800 tabel C.2, p. 778)
//!
//! [`RcOrU::Rc`] wordt omgezet met `U = 1 / (R_si + R_c + R_se)`. De
//! oppervlakteweerstanden komen uit [`crate::nta8800_view::surface_resistances`]
//! — de projectbrede implementatie van NTA 8800 tabel C.2 (overgenomen uit
//! NEN-EN-ISO 6946:2017 §6.8): `R_si` = 0,10 (warmtestroom omhoog/dak) / 0,13
//! (horizontaal/gevel) / 0,17 (omlaag/vloer); `R_se` = 0,04 (0 voor grond/water,
//! daar verzorgt het grondmodel de buitenzijde). Voor de Aalten-schil reproduceert
//! dit exact de certified U's: wand 4,70 → 0,205, dak 6,30 → 0,155, vloer 3,70 →
//! 0,258.
//!
//! ## Gedocumenteerde ketenbeperkingen (buiten deze fase)
//!
//! Twee posten uit de BENG-invoer worden door de huidige (gedeelde) keten nog
//! niet benut; ze worden meegenomen/omgezet maar het rekenpad negeert ze, dus de
//! brug verandert daar niets aan (geen tweede rekenpad):
//!
//! - **Vloer-op-grond P/A-methode** — [`BengBoundary::omtrek_p_m`] is bekend, maar
//!   [`crate::compute_tojuli_full`] hanteert een forfaitaire grond-conductantie
//!   `h_g;an = 10 W/K` (§8.3.1-fallback), onafhankelijk van A/U of omtrek P. De
//!   omtrek reist mee in de BENG-invoer maar stuurt de berekening (nog) niet.
//! - **Raam-U in de demand-transmissie** — zie de conventie-keuze hierboven: de
//!   raam-U voedt de TOjuli-noemer maar niet de demand-`H_D` (ramen op opake U).

use nta8800_model::ModelError;

use crate::beng_geometry::{
    BengAdjacency, BengBoundary, BengGeometry, BengWindowPlacement, KozijnType,
    OpaqueConstructionDef, RcOrU, VlakType,
};
use crate::geometry::{
    BoundaryKind, Construction, ConstructionKind, Opening, OpeningKind, SharedGeometry, Space,
};
use crate::nta8800_view::surface_resistances;

/// Forfaitaire verdiepingshoogte [m] wanneer geen bestaande geometrie een
/// zone-volume levert. Gelijk aan de `nta8800_view`-standaard voor woningbouw;
/// het zone-volume (`A_g · h`) voedt de ventilatie-`H_ve` en de tijdconstante τ.
const DEFAULT_ZONE_HEIGHT_M: f64 = 2.7;

/// Vertaal een [`BengGeometry`] naar een [`SharedGeometry`] voor de demand-keten.
///
/// Per BENG-rekenzone ontstaat één [`Space`] (`floor_area_m2 = a_g_m2`); elke
/// [`BengBoundary`] wordt één [`Construction`] met de buiten-oppervlakte en de
/// opake U-waarde, met de kozijn-plaatsingen als [`Opening`]s. De lineaire
/// koudebruggen van `existing` worden ongewijzigd overgenomen (BENG-invoer codeert
/// ze niet; ze horen op rekenzone-niveau en moeten door de brug bewaard blijven).
///
/// De verdiepingshoogte wordt uit `existing` afgeleid (`Σ volume / Σ vloeropp`)
/// zodat het zone-volume — en daarmee de ventilatie-`H_ve` en τ — bij een
/// meting behouden blijft; ontbreekt dat, dan geldt [`DEFAULT_ZONE_HEIGHT_M`].
///
/// # Errors
///
/// - Propageert [`BengGeometry::validate`]-fouten (referentie-integriteit,
///   plausibiliteit).
/// - [`ModelError::InvalidInput`] als een `Raam`-kozijnmerk geen g-waarde draagt
///   (geen stilzwijgend forfait — transparantie-huisregel).
pub fn beng_geometry_to_shared(
    beng: &BengGeometry,
    existing: &SharedGeometry,
) -> Result<SharedGeometry, ModelError> {
    beng.validate()?;

    let height_m = effective_zone_height_m(existing);

    let mut spaces = Vec::with_capacity(beng.zones.len());
    for zone in &beng.zones {
        let mut constructions = Vec::with_capacity(zone.gevels.len());
        for gevel in &zone.gevels {
            constructions.push(map_boundary(beng, gevel)?);
        }
        spaces.push(Space {
            id: zone.id.clone(),
            name: if zone.naam.is_empty() {
                zone.id.clone()
            } else {
                zone.naam.clone()
            },
            function: None,
            floor_area_m2: zone.a_g_m2,
            height_m,
            constructions,
            // Zone-brede setpoints; identiek aan de oes-converter zodat de
            // demand-keten dezelfde binnentemperatuur ziet.
            theta_i_winter_c: Some(20.0),
            theta_i_summer_c: Some(24.0),
        });
    }

    Ok(SharedGeometry {
        spaces,
        // Koudebruggen (§8.2.3, Σψ·L) reizen mee uit de bestaande geometrie —
        // BENG-invoer codeert ze niet, maar ze horen op rekenzone-niveau.
        thermal_bridges: existing.thermal_bridges.clone(),
    })
}

/// Leid een representatieve verdiepingshoogte af uit de bestaande geometrie
/// (`Σ volume / Σ vloeroppervlak`); terugval op [`DEFAULT_ZONE_HEIGHT_M`].
fn effective_zone_height_m(existing: &SharedGeometry) -> f64 {
    let floor_area: f64 = existing.spaces.iter().map(|s| s.floor_area_m2).sum();
    let volume: f64 = existing
        .spaces
        .iter()
        .map(|s| s.floor_area_m2 * s.height_m)
        .sum();
    if floor_area > 0.0 && volume > 0.0 {
        volume / floor_area
    } else {
        DEFAULT_ZONE_HEIGHT_M
    }
}

/// Map één [`BengBoundary`] naar een [`Construction`] (bruto buiten-opp + opake U
/// + kozijn-openings).
fn map_boundary(beng: &BengGeometry, gevel: &BengBoundary) -> Result<Construction, ModelError> {
    // `validate()` (hierboven) heeft de referentie al gecontroleerd; deze lookup
    // faalt dus niet, maar we geven een nette fout i.p.v. een unwrap.
    let def = beng.opaque_def(&gevel.constructie_ref).ok_or_else(|| {
        ModelError::ReferenceNotFound {
            kind: "OpaqueConstructionDef".into(),
            id: gevel.constructie_ref.clone(),
        }
    })?;

    let kind = construction_kind(gevel.vlak_type);
    let boundary = boundary_kind(&gevel.grenst_aan);
    let u_value = opaque_u_value(def, kind, boundary);
    let orientation_deg = gevel.grenst_aan.orientatie().and_then(|o| o.degrees());

    let mut openings = Vec::with_capacity(gevel.ramen.len());
    for (idx, raam) in gevel.ramen.iter().enumerate() {
        openings.push(map_window(beng, gevel, idx, raam)?);
    }

    Ok(Construction {
        id: gevel.id.clone(),
        description: gevel.omschrijving.clone(),
        kind,
        boundary,
        // KERN-HERORIËNTATIE: buiten-oppervlak per gevel (bruto, incl. ramen),
        // niet binnen-oppervlak per kamer.
        area_m2: gevel.bruto_buiten_opp_m2,
        u_value,
        orientation_deg,
        slope_deg: gevel.helling_deg,
        openings,
        layers: Vec::new(),
        adjacent_space_id: None,
        psi_thermal_bridge: None,
    })
}

/// Map een kozijn-plaatsing naar een [`Opening`]. `Raam` → venster met g-waarde;
/// `Deur`/`PaneelInKozijn` → opake deur zonder g-waarde.
fn map_window(
    beng: &BengGeometry,
    gevel: &BengBoundary,
    idx: usize,
    raam: &BengWindowPlacement,
) -> Result<Opening, ModelError> {
    let def = beng.window_def(&raam.kozijn_ref).ok_or_else(|| {
        ModelError::ReferenceNotFound {
            kind: "WindowDef".into(),
            id: raam.kozijn_ref.clone(),
        }
    })?;

    let area_m2 = f64::from(raam.aantal) * def.area_m2;

    let (kind, g_value) = match def.kind {
        KozijnType::Raam => {
            // Een raam zónder g-waarde is onvolledige certified-invoer; een
            // forfait zou een verborgen aanname zijn (transparantie-huisregel).
            let g = def.ggl.ok_or_else(|| ModelError::InvalidInput {
                context: format!("WindowDef[{}].ggl", def.id),
                reason: "een raam (KozijnType::Raam) vereist een g-waarde (ggl) voor de \
                         zonwinst — een forfait wordt niet stilzwijgend toegepast"
                    .into(),
            })?;
            (OpeningKind::Window, Some(g))
        }
        // Deur/paneel: opaak vlak in het kozijn, geen zontoetreding (§7.9 werkt
        // op ramen). Uniec `ggl = 0`/n.v.t. voor deze typen.
        KozijnType::Deur | KozijnType::PaneelInKozijn => (OpeningKind::Door, None),
    };

    Ok(Opening {
        id: format!("{}-kozijn{}-{}", gevel.id, idx, raam.kozijn_ref),
        kind,
        area_m2,
        u_value: def.u_w_per_m2k,
        g_value,
        // `None` → de demand-keten hanteert de forfaitaire kozijnfractie 0,25
        // (identiek aan de oes-converter).
        frame_fraction: None,
        // Beweegbare zonwering + externe belemmering zijn al `nta8800-model`-typen
        // op de plaatsing → 1-op-1 door.
        movable_shading: raam.zonwering,
        obstruction: raam.belemmering,
    })
}

/// [`RcOrU`] → U-waarde `W/(m²·K)`. Rc wordt omgerekend met de
/// oppervlakteweerstanden uit NTA 8800 tabel C.2 (via
/// [`surface_resistances`]): `U = 1 / (R_si + R_c + R_se)`.
fn opaque_u_value(def: &OpaqueConstructionDef, kind: ConstructionKind, boundary: BoundaryKind) -> f64 {
    match def.thermal {
        RcOrU::U(u) => u,
        RcOrU::Rc(rc) => {
            let (r_si, r_se) = surface_resistances(kind, boundary);
            1.0 / (r_si + rc + r_se)
        }
    }
}

/// [`VlakType`] → [`ConstructionKind`] (bepaalt R_si-richting bij de Rc→U-omzet).
fn construction_kind(vlak_type: VlakType) -> ConstructionKind {
    match vlak_type {
        VlakType::Vloer | VlakType::VloerBovenBuitenlucht | VlakType::Bodem => {
            ConstructionKind::Floor
        }
        VlakType::Gevel | VlakType::Kelderwand => ConstructionKind::Wall,
        VlakType::Dak => ConstructionKind::Roof,
    }
}

/// [`BengAdjacency`] → [`BoundaryKind`] (bepaalt de transmissie-tak: buiten,
/// grond, onverwarmde buffer, aangrenzend, water).
fn boundary_kind(adjacency: &BengAdjacency) -> BoundaryKind {
    match adjacency {
        // Directe buitenlucht; sterk geventileerde spouw ≈ buitenlucht (b ≈ 1).
        BengAdjacency::Buitenlucht { .. } | BengAdjacency::SterkGeventileerd => {
            BoundaryKind::Exterior
        }
        // Grondcontact (grond/spouw, z ≤ 0,3) → grondmodel (§8.3).
        BengAdjacency::VloerOpMaaiveldBovenGrond
        | BengAdjacency::VloerOnderMaaiveldBovenGrond => BoundaryKind::Ground,
        // Onverwarmde buffers (kruipruimte, onverwarmde kelder, AOS/AOR) → §8.4
        // b-factor-tak.
        BengAdjacency::VloerOpMaaiveldBovenKruipruimte
        | BengAdjacency::VloerOpMaaiveldBovenOnverwarmdeKelder
        | BengAdjacency::VloerOnderMaaiveldBovenKruipruimte
        | BengAdjacency::VloerOnderMaaiveldBovenOnverwarmdeKelder
        | BengAdjacency::AosForfaitair { .. }
        | BengAdjacency::AorForfaitair => BoundaryKind::UnheatedSpace,
        BengAdjacency::Water => BoundaryKind::OpenWater,
        // Aangrenzende verwarmde ruimte: netto-transmissie ≈ 0 (gelijk setpoint).
        BengAdjacency::AangrenzendeVerwarmdeRuimte => BoundaryKind::AdjacentRoom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beng_geometry::{BengWindowPlacement, BengZone, WindowDef};
    use nta8800_model::location::Orientation;
    use nta8800_model::Obstruction;

    /// Rc→U reproduceert de certified Aalten-U's (NTA 8800 tabel C.2).
    #[test]
    fn rc_to_u_matches_certified_aalten() {
        let wand = OpaqueConstructionDef {
            id: "w".into(),
            omschrijving: String::new(),
            kind: VlakType::Gevel,
            thermal: RcOrU::Rc(4.70),
        };
        let dak = OpaqueConstructionDef {
            id: "d".into(),
            omschrijving: String::new(),
            kind: VlakType::Dak,
            thermal: RcOrU::Rc(6.30),
        };
        let vloer = OpaqueConstructionDef {
            id: "v".into(),
            omschrijving: String::new(),
            kind: VlakType::Vloer,
            thermal: RcOrU::Rc(3.70),
        };
        // Wand: 1/(0,13+4,70+0,04) = 0,2053
        assert!((opaque_u_value(&wand, ConstructionKind::Wall, BoundaryKind::Exterior) - 0.205_338).abs() < 1e-5);
        // Dak: 1/(0,10+6,30+0,04) = 0,1553
        assert!((opaque_u_value(&dak, ConstructionKind::Roof, BoundaryKind::Exterior) - 0.155_280).abs() < 1e-5);
        // Vloer op grond: R_se=0 → 1/(0,17+3,70) = 0,2584
        assert!((opaque_u_value(&vloer, ConstructionKind::Floor, BoundaryKind::Ground) - 0.258_398).abs() < 1e-5);
    }

    /// Directe U-invoer gaat ongemoeid door.
    #[test]
    fn direct_u_passes_through() {
        let def = OpaqueConstructionDef {
            id: "x".into(),
            omschrijving: String::new(),
            kind: VlakType::Gevel,
            thermal: RcOrU::U(1.3),
        };
        assert!((opaque_u_value(&def, ConstructionKind::Wall, BoundaryKind::Exterior) - 1.3).abs() < 1e-12);
    }

    #[test]
    fn adjacency_maps_to_boundary_kind() {
        assert_eq!(
            boundary_kind(&BengAdjacency::Buitenlucht { orientatie: Orientation::Zuid }),
            BoundaryKind::Exterior
        );
        assert_eq!(
            boundary_kind(&BengAdjacency::VloerOpMaaiveldBovenGrond),
            BoundaryKind::Ground
        );
        assert_eq!(
            boundary_kind(&BengAdjacency::VloerOpMaaiveldBovenKruipruimte),
            BoundaryKind::UnheatedSpace
        );
        assert_eq!(boundary_kind(&BengAdjacency::Water), BoundaryKind::OpenWater);
        assert_eq!(
            boundary_kind(&BengAdjacency::AangrenzendeVerwarmdeRuimte),
            BoundaryKind::AdjacentRoom
        );
    }

    fn geo_with_one_gevel(ramen: Vec<BengWindowPlacement>, window_defs: Vec<WindowDef>) -> BengGeometry {
        BengGeometry {
            opaque_defs: vec![OpaqueConstructionDef {
                id: "def-wand".into(),
                omschrijving: "Wand".into(),
                kind: VlakType::Gevel,
                thermal: RcOrU::Rc(4.70),
            }],
            window_defs,
            zones: vec![BengZone {
                id: "rz".into(),
                naam: "woning".into(),
                a_g_m2: 67.0,
                bouwwijze_vloer: None,
                bouwwijze_wand: None,
                woningtype: None,
                gevels: vec![BengBoundary {
                    id: "gevel-z".into(),
                    omschrijving: "Wand".into(),
                    vlak_type: VlakType::Gevel,
                    grenst_aan: BengAdjacency::Buitenlucht { orientatie: Orientation::Zuid },
                    bruto_buiten_opp_m2: 23.81,
                    helling_deg: Some(90.0),
                    omtrek_p_m: None,
                    constructie_ref: "def-wand".into(),
                    ramen,
                }],
            }],
        }
    }

    #[test]
    fn raam_becomes_window_opening_with_g_value() {
        let geo = geo_with_one_gevel(
            vec![BengWindowPlacement {
                kozijn_ref: "merk-a".into(),
                aantal: 2,
                belemmering: Obstruction::Minimal,
                zonwering: None,
                zomernachtventilatie: false,
            }],
            vec![WindowDef {
                id: "merk-a".into(),
                omschrijving: "A".into(),
                kind: KozijnType::Raam,
                u_w_per_m2k: 1.3,
                ggl: Some(0.40),
                area_m2: 2.0,
            }],
        );
        let shared = beng_geometry_to_shared(&geo, &SharedGeometry::default()).unwrap();
        let c = &shared.spaces[0].constructions[0];
        assert_eq!(c.boundary, BoundaryKind::Exterior);
        assert!((c.area_m2 - 23.81).abs() < 1e-9, "bruto buiten-opp op de construction");
        assert_eq!(c.orientation_deg, Some(180.0));
        assert_eq!(c.slope_deg, Some(90.0));
        assert_eq!(c.openings.len(), 1);
        let o = &c.openings[0];
        assert_eq!(o.kind, OpeningKind::Window);
        assert!((o.area_m2 - 4.0).abs() < 1e-9, "aantal × area_m2");
        assert_eq!(o.g_value, Some(0.40));
        assert!((o.u_value - 1.3).abs() < 1e-12);
        assert_eq!(o.obstruction, Obstruction::Minimal);
    }

    #[test]
    fn deur_becomes_door_opening_without_g_value() {
        let geo = geo_with_one_gevel(
            vec![BengWindowPlacement {
                kozijn_ref: "merk-deur".into(),
                aantal: 1,
                belemmering: Obstruction::None,
                zonwering: None,
                zomernachtventilatie: false,
            }],
            vec![WindowDef {
                id: "merk-deur".into(),
                omschrijving: "deur".into(),
                kind: KozijnType::Deur,
                u_w_per_m2k: 2.0,
                ggl: Some(0.0),
                area_m2: 1.84,
            }],
        );
        let shared = beng_geometry_to_shared(&geo, &SharedGeometry::default()).unwrap();
        let o = &shared.spaces[0].constructions[0].openings[0];
        assert_eq!(o.kind, OpeningKind::Door);
        assert_eq!(o.g_value, None);
        assert!((o.u_value - 2.0).abs() < 1e-12);
    }

    #[test]
    fn raam_without_g_value_is_rejected() {
        let geo = geo_with_one_gevel(
            vec![BengWindowPlacement {
                kozijn_ref: "merk-x".into(),
                aantal: 1,
                belemmering: Obstruction::None,
                zonwering: None,
                zomernachtventilatie: false,
            }],
            vec![WindowDef {
                id: "merk-x".into(),
                omschrijving: "X".into(),
                kind: KozijnType::Raam,
                u_w_per_m2k: 1.3,
                ggl: None,
                area_m2: 2.0,
            }],
        );
        let err = beng_geometry_to_shared(&geo, &SharedGeometry::default()).unwrap_err();
        assert!(matches!(err, ModelError::InvalidInput { context, .. } if context.contains("ggl")));
    }

    #[test]
    fn zone_height_derived_from_existing_volume() {
        let existing = SharedGeometry {
            spaces: vec![Space {
                id: "s".into(),
                name: "s".into(),
                function: None,
                floor_area_m2: 67.0,
                height_m: 2.6,
                constructions: vec![],
                theta_i_winter_c: None,
                theta_i_summer_c: None,
            }],
            ..Default::default()
        };
        let geo = geo_with_one_gevel(vec![], vec![]);
        let shared = beng_geometry_to_shared(&geo, &existing).unwrap();
        // 174,2 / 67,0 = 2,6 → zone-volume behouden.
        assert!((shared.spaces[0].height_m - 2.6).abs() < 1e-9);
        assert!((shared.spaces[0].floor_area_m2 - 67.0).abs() < 1e-9);
    }

    #[test]
    fn thermal_bridges_are_carried_over() {
        use crate::geometry::ThermalBridge;
        let existing = SharedGeometry {
            spaces: vec![],
            thermal_bridges: vec![ThermalBridge {
                id: "tb".into(),
                description: "gevel-vloer".into(),
                psi_w_per_mk: 0.05,
                length_m: 26.0,
            }],
        };
        let geo = geo_with_one_gevel(vec![], vec![]);
        let shared = beng_geometry_to_shared(&geo, &existing).unwrap();
        assert_eq!(shared.thermal_bridges.len(), 1);
        assert!((shared.thermal_bridges[0].psi_w_per_mk - 0.05).abs() < 1e-12);
    }
}
