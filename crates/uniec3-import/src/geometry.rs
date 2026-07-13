//! Fase 4c — entity-graaf → [`BengGeometry`].
//!
//! Loopt het geometrie-pad `UNIT → UNIT-RZ → BEGR → CONSTRD/CONSTRT`, resolve't
//! de `LIBCONSTRD`/`LIBCONSTRT`-bibliotheken (via de `*_LIB`-GUID-properties) en
//! mapt de Uniec-enumcodes op de DTO-enums. Zie formaat-analyse §4/§5a.
//!
//! **Id-strategie:** de gegenereerde DTO-id's zijn de Uniec-instance-GUID's
//! (gegarandeerd uniek). `constructie_ref`/`kozijn_ref` gebruiken de `*_LIB`-
//! GUID, die exact de `LIBCONSTRD`/`LIBCONSTRT`-`data_id` is → referentie-
//! integriteit zonder her-mapping. De round-trip-test vergelijkt daarom op
//! wáárde (opp/U/Rc/ggl/oriëntatie), niet op id-string.

use std::collections::{HashMap, HashSet};

use openaec_project_shared::beng_geometry::{
    BengAdjacency, BengBoundary, BengGeometry, BengWindowPlacement, BengZone, KozijnType,
    OpaqueConstructionDef, RcOrU, VlakType, WindowDef,
};
use openaec_project_shared::{MovableSunShading, Obstruction, Orientation};

use crate::error::{Result, Uniec3ImportError};
use crate::parse::{Entity, EntityIndex};

/// Metadata van een `LIBCONSTRT`-kozijnmerk, los van het invoermodel.
///
/// Uniec kent twee modi: **oppervlakte-per-merk** (`LIBCONSTRT_AC` gevuld → één
/// gedeelde [`WindowDef`]) en **oppervlakte-per-raam** (`AC` leeg → het
/// oppervlak staat op de plaatsing `CONSTRT_OPP`, dus per plaatsing een eigen
/// def). `area` onderscheidt de twee.
struct LibWindow {
    kind: KozijnType,
    u: f64,
    ggl: Option<f64>,
    omschrijving: String,
    area: Option<f64>,
}

/// Werk-context die door de gevel-/raam-traversal gedeeld wordt: de resolvebare
/// def-id's en de accumulerende kozijn-bibliotheek (voor de per-raam-modus).
struct GeoCtx {
    /// `LIBCONSTRT`-GUID → merk-metadata (beide invoermodi).
    lib_windows: HashMap<String, LibWindow>,
    /// Id's van reeds opgenomen [`WindowDef`]s (per-merk up-front + per-raam
    /// gaandeweg) — voorkomt duplicaten.
    window_ids: HashSet<String>,
    /// Geldige opake-constructie-id's (voor referentie-integriteit).
    opaque_ids: HashSet<String>,
    /// De opbouwende kozijn-bibliotheek.
    window_defs: Vec<WindowDef>,
}

/// Map de geometrie van één single-unit woning naar [`BengGeometry`].
///
/// Verzamelt onbekende enumcodes/afwijkingen in `warnings` (tolerant parsen);
/// faalt alleen als er geen bruikbare rekenzone-geometrie is.
pub fn map_geometry(idx: &EntityIndex, warnings: &mut Vec<String>) -> Result<BengGeometry> {
    let opaque_defs = map_opaque_defs(idx, warnings);
    let lib_windows = index_lib_windows(idx, warnings);

    // Per-merk-defs (AC gevuld) staan vooraf in de bibliotheek — ook ongebruikte,
    // zodat de bibliotheek de Uniec-kozijnmerkenlijst spiegelt. Per-raam-defs
    // (AC leeg) komen gaandeweg de traversal binnen.
    let mut window_defs = Vec::new();
    let mut window_ids = HashSet::new();
    for (id, lib) in &lib_windows {
        if let Some(area) = lib.area {
            window_defs.push(WindowDef {
                id: id.clone(),
                omschrijving: lib.omschrijving.clone(),
                kind: lib.kind,
                u_w_per_m2k: lib.u,
                ggl: lib.ggl,
                area_m2: area,
            });
            window_ids.insert(id.clone());
        }
    }

    let mut ctx = GeoCtx {
        lib_windows,
        window_ids,
        opaque_ids: opaque_defs.iter().map(|d| d.id.clone()).collect(),
        window_defs,
    };

    let zones = map_zones(idx, &mut ctx, warnings)?;

    // Deterministische volgorde (de HashMap-iteratie hierboven is willekeurig).
    ctx.window_defs.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(BengGeometry {
        opaque_defs,
        window_defs: ctx.window_defs,
        zones,
    })
}

// ---------------------------------------------------------------------------
// Bibliotheken
// ---------------------------------------------------------------------------

fn map_opaque_defs(idx: &EntityIndex, warnings: &mut Vec<String>) -> Vec<OpaqueConstructionDef> {
    let mut defs = Vec::new();
    for e in idx.of_type("LIBCONSTRD") {
        let kind = match e.prop("LIBCONSTRD_TYPE") {
            Some("LIBVLAK_VLOER") => VlakType::Vloer,
            Some("LIBVLAK_GEVEL") => VlakType::Gevel,
            Some("LIBVLAK_DAK") => VlakType::Dak,
            Some("LIBVLAK_KELDERWAND") => VlakType::Kelderwand,
            Some("LIBVLAK_BODEM") => VlakType::Bodem,
            other => {
                warnings.push(format!(
                    "LIBCONSTRD {}: onbekend type {:?} → Gevel aangenomen",
                    &e.data_id[..8.min(e.data_id.len())],
                    other
                ));
                VlakType::Gevel
            }
        };
        // Uniec voert opake constructies in als Rc (vrije invoer). Ontbreekt de
        // Rc, dan slaan we deze definitie over met een waarschuwing.
        let Some(rc) = e.num("LIBCONSTRD_RC") else {
            warnings.push(format!(
                "LIBCONSTRD {} ({}) mist LIBCONSTRD_RC → overgeslagen",
                &e.data_id[..8.min(e.data_id.len())],
                e.prop("LIBCONSTRD_OMSCHR").unwrap_or("")
            ));
            continue;
        };
        defs.push(OpaqueConstructionDef {
            id: e.data_id.clone(),
            omschrijving: e.prop("LIBCONSTRD_OMSCHR").unwrap_or("").to_string(),
            kind,
            thermal: RcOrU::Rc(rc),
        });
    }
    defs
}

/// Indexeer de `LIBCONSTRT`-kozijnmerken op GUID met hun metadata (kind/U/ggl +
/// optioneel per-merk-oppervlak). Merken zonder U worden overgeslagen (kan geen
/// raam zijn); het oppervlak mag ontbreken (per-raam-modus, zie [`LibWindow`]).
fn index_lib_windows(idx: &EntityIndex, warnings: &mut Vec<String>) -> HashMap<String, LibWindow> {
    let mut libs = HashMap::new();
    for e in idx.of_type("LIBCONSTRT") {
        let kind = match e.prop("LIBCONSTRT_TYPE") {
            Some("TRANSTYPE_RAAM") => KozijnType::Raam,
            Some("TRANSTYPE_DEUR") => KozijnType::Deur,
            Some("TRANSTYPE_PANEEL") => KozijnType::PaneelInKozijn,
            other => {
                warnings.push(format!(
                    "LIBCONSTRT {}: onbekend type {:?} → Raam aangenomen",
                    &e.data_id[..8.min(e.data_id.len())],
                    other
                ));
                KozijnType::Raam
            }
        };
        let Some(u) = e.num("LIBCONSTRT_U") else {
            warnings.push(format!(
                "LIBCONSTRT {} ({}) mist LIBCONSTRT_U → overgeslagen",
                &e.data_id[..8.min(e.data_id.len())],
                e.prop("LIBCONSTRT_OMSCHR").unwrap_or("")
            ));
            continue;
        };
        libs.insert(
            e.data_id.clone(),
            LibWindow {
                kind,
                u,
                ggl: e.num("LIBCONSTRT_G"),
                omschrijving: e.prop("LIBCONSTRT_OMSCHR").unwrap_or("").to_string(),
                area: e.num("LIBCONSTRT_AC"),
            },
        );
    }
    libs
}

// ---------------------------------------------------------------------------
// Rekenzones
// ---------------------------------------------------------------------------

fn map_zones(
    idx: &EntityIndex,
    ctx: &mut GeoCtx,
    warnings: &mut Vec<String>,
) -> Result<Vec<BengZone>> {
    let units = idx.of_type("UNIT");
    if units.len() > 1 {
        return Err(Uniec3ImportError::MultiUnitUnsupported(format!(
            "{} UNIT-entiteiten (appartementen/meergezins)",
            units.len()
        )));
    }
    let unit = units
        .into_iter()
        .next()
        .ok_or_else(|| Uniec3ImportError::MissingGeometry("geen UNIT-entiteit".to_string()))?;

    // MZ-V2a: één UNIT mag N rekenzones (UNIT-RZ) dragen — elke RZ wordt een eigen
    // `BengZone`. De multi-UNIT-guard hierboven blijft (appartementen/meergezins =
    // apart pakket). De BENG-keten poolt de zones (nog) tot één rekenzone, dus het
    // resultaat is bij N > 1 indicatief; de importer markeert dat met een warning.
    let unit_rzs = idx.children_of(unit, "UNIT-RZ");
    if unit_rzs.is_empty() {
        return Err(Uniec3ImportError::MissingGeometry(
            "geen UNIT-RZ onder UNIT".to_string(),
        ));
    }
    if unit_rzs.len() > 1 {
        warnings.push(format!(
            "{} rekenzones geïmporteerd; de BENG-resultaten worden gepoold berekend \
             (indicatief) — norm-exact per-rekenzone-rekenen volgt (MZ-V2b, NTA 8800 §6.6.2)",
            unit_rzs.len()
        ));
    }

    let mut zones = Vec::with_capacity(unit_rzs.len());
    for unit_rz in unit_rzs {
        zones.push(map_zone(idx, unit, unit_rz, ctx, warnings)?);
    }
    Ok(zones)
}

/// Map één `UNIT-RZ` (rekenzone) naar een [`BengZone`]. De rekenzone-metadata
/// (bouwwijze/omschrijving) hangt op de `RZ` waar `UNIT-RZID` naar wijst; de
/// begrenzingen hangen als `BEGR`-children onder de `UNIT-RZ` zelf. `woningtype`
/// is UNIT-breed en dus voor alle zones gelijk.
fn map_zone(
    idx: &EntityIndex,
    unit: &Entity,
    unit_rz: &Entity,
    ctx: &mut GeoCtx,
    warnings: &mut Vec<String>,
) -> Result<BengZone> {
    let rz = unit_rz.prop("UNIT-RZID").and_then(|id| idx.get(id));

    let a_g = unit_rz.num("UNIT-RZAG").ok_or_else(|| {
        Uniec3ImportError::MissingGeometry("UNIT-RZ mist gebruiksoppervlak (UNIT-RZAG)".to_string())
    })?;

    // Bouwwijze-codes met pre-3.2-terugval: als de gesplitste `_VL`/`_W` ontbreken
    // (oude exports), lees het ongesplitste `RZ_BOUWW` en dupliceer (analyse §6).
    let (bouwwijze_vloer, bouwwijze_wand) = match rz {
        Some(rz) => {
            let vl = rz.prop("RZ_BOUWW_VL").map(str::to_string);
            let wa = rz.prop("RZ_BOUWW_W").map(str::to_string);
            if vl.is_none() && wa.is_none() {
                if let Some(legacy) = rz.prop("RZ_BOUWW") {
                    warnings.push(
                        "RZ_BOUWW_VL/_W ontbreken (pre-3.2 export) → RZ_BOUWW gedupliceerd"
                            .to_string(),
                    );
                    (Some(legacy.to_string()), Some(legacy.to_string()))
                } else {
                    (None, None)
                }
            } else {
                (vl, wa)
            }
        }
        None => (None, None),
    };

    Ok(BengZone {
        id: rz.map_or_else(|| unit_rz.data_id.clone(), |r| r.data_id.clone()),
        naam: rz
            .and_then(|r| r.prop("RZ_OMSCHR"))
            .unwrap_or("")
            .to_string(),
        a_g_m2: a_g,
        bouwwijze_vloer,
        bouwwijze_wand,
        woningtype: unit.prop("UNIT_TYPEWON").map(str::to_string),
        gevels: map_boundaries(idx, unit_rz, ctx, warnings),
    })
}

// ---------------------------------------------------------------------------
// Begrenzingen (gevels)
// ---------------------------------------------------------------------------

fn map_boundaries(
    idx: &EntityIndex,
    unit_rz: &Entity,
    ctx: &mut GeoCtx,
    warnings: &mut Vec<String>,
) -> Vec<BengBoundary> {
    let mut gevels = Vec::new();
    for begr in idx.children_of(unit_rz, "BEGR") {
        let vlak_type = match begr.prop("BEGR_VLAK") {
            Some("VLAK_VLOER") => VlakType::Vloer,
            Some("VLAK_GEVEL") => VlakType::Gevel,
            Some("VLAK_DAK") => VlakType::Dak,
            Some("VLAK_VLOER_BOVBUI") => VlakType::VloerBovenBuitenlucht,
            Some("VLAK_KELDERWAND") => VlakType::Kelderwand,
            other => {
                warnings.push(format!(
                    "BEGR {} ({}): onbekend BEGR_VLAK {:?} → overgeslagen",
                    &begr.data_id[..8.min(begr.data_id.len())],
                    begr.prop("BEGR_OMSCHR").unwrap_or(""),
                    other
                ));
                continue;
            }
        };

        let Some(area) = begr.num("BEGR_A") else {
            warnings.push(format!(
                "BEGR {} mist oppervlak (BEGR_A) → overgeslagen",
                &begr.data_id[..8.min(begr.data_id.len())]
            ));
            continue;
        };

        let grenst_aan = map_adjacency(begr, vlak_type, warnings);

        // Opake constructie-referentie via CONSTRD-child → CONSTRD_LIB-GUID. Moet
        // resolven naar een opgenomen OpaqueConstructionDef (referentie-
        // integriteit); een dangling ref → gevel overslaan i.p.v. hard falen.
        let Some(constructie_ref) = idx
            .child_of(begr, "CONSTRD")
            .and_then(|c| c.prop("CONSTRD_LIB"))
            .map(str::to_string)
        else {
            warnings.push(format!(
                "BEGR {} ({}) mist CONSTRD/CONSTRD_LIB → overgeslagen",
                &begr.data_id[..8.min(begr.data_id.len())],
                begr.prop("BEGR_OMSCHR").unwrap_or("")
            ));
            continue;
        };
        if !ctx.opaque_ids.contains(&constructie_ref) {
            warnings.push(format!(
                "BEGR {} ({}): opake constructie {} niet gevonden → gevel overgeslagen",
                &begr.data_id[..8.min(begr.data_id.len())],
                begr.prop("BEGR_OMSCHR").unwrap_or(""),
                &constructie_ref[..8.min(constructie_ref.len())]
            ));
            continue;
        }

        // Omtrek P van het vloerveld uit de CONSTRKENMV-child (KENMV_OMTR_VL).
        // We lezen 'm voor élk vlak en houden alleen een positieve waarde over:
        // niet-vloervlakken dragen 0/afwezig → `None`. Zo krijgt zowel de
        // vloer-op-grond (P/A verplicht) als de vloer-op-kruipruimte (P aanwezig
        // in het bestand, optioneel voor de norm) zijn omtrek — consistent met de
        // certified capture.
        let omtrek_p_m = idx
            .child_of(begr, "CONSTRKENMV")
            .and_then(|c| c.num("KENMV_OMTR_VL"))
            .filter(|&p| p > 0.0);

        let ramen = map_windows(idx, begr, ctx, warnings);

        gevels.push(BengBoundary {
            id: begr.data_id.clone(),
            omschrijving: begr.prop("BEGR_OMSCHR").unwrap_or("").to_string(),
            vlak_type,
            grenst_aan,
            bruto_buiten_opp_m2: area,
            helling_deg: begr.num("BEGR_HEL"),
            omtrek_p_m,
            constructie_ref,
            ramen,
        });
    }
    gevels
}

/// Map de begrenzing-aangrenzing uit de `BEGR_VLOER`/`BEGR_GEVEL`/`BEGR_DAK`-
/// codes (analyse §5a). Onbekende codes → tolerante terugval + waarschuwing.
fn map_adjacency(
    begr: &crate::parse::Entity,
    vlak_type: VlakType,
    warnings: &mut Vec<String>,
) -> BengAdjacency {
    match vlak_type {
        VlakType::Vloer | VlakType::VloerBovenBuitenlucht | VlakType::Bodem => {
            match begr.prop("BEGR_VLOER") {
                Some("VL_MV_KR") => BengAdjacency::VloerOpMaaiveldBovenKruipruimte,
                Some("VL_MV_GRSP") => BengAdjacency::VloerOpMaaiveldBovenGrond,
                Some("VL_MV_KLDR") => BengAdjacency::VloerOpMaaiveldBovenOnverwarmdeKelder,
                Some("VL_OMV_KR") => BengAdjacency::VloerOnderMaaiveldBovenKruipruimte,
                Some("VL_OMV_GRSP") => BengAdjacency::VloerOnderMaaiveldBovenGrond,
                Some("VL_OMV_KLDR") => BengAdjacency::VloerOnderMaaiveldBovenOnverwarmdeKelder,
                // Drijvende woning: vloer grenst aan open water (geen grond-P/A,
                // dus géén omtrek-eis) — Uniec `VL_WATER`.
                Some("VL_WATER") => BengAdjacency::Water,
                other => {
                    warnings.push(format!(
                        "BEGR {}: onbekende BEGR_VLOER {:?} → vloer-op-maaiveld-boven-grond aangenomen",
                        &begr.data_id[..8.min(begr.data_id.len())],
                        other
                    ));
                    BengAdjacency::VloerOpMaaiveldBovenGrond
                }
            }
        }
        VlakType::Gevel | VlakType::Kelderwand => match begr.prop("BEGR_GEVEL") {
            // Onderwaterlijn-gevel van een drijvende woning grenst aan open water
            // — Uniec `GVL_WATER` (draagt geen kompas-oriëntatie).
            Some("GVL_WATER") => BengAdjacency::Water,
            code => match orientation_from_code(code, "GVL_BTNL_") {
                Some(o) => BengAdjacency::Buitenlucht { orientatie: o },
                None => adjacency_fallback_or_buitenlucht(begr, "gevel", warnings),
            },
        },
        VlakType::Dak => match orientation_from_code(begr.prop("BEGR_DAK"), "DAK_BTNL_") {
            Some(o) => BengAdjacency::Buitenlucht { orientatie: o },
            None => adjacency_fallback_or_buitenlucht(begr, "dak", warnings),
        },
    }
}

fn adjacency_fallback_or_buitenlucht(
    begr: &crate::parse::Entity,
    what: &str,
    warnings: &mut Vec<String>,
) -> BengAdjacency {
    warnings.push(format!(
        "BEGR {} ({}): geen buitenlucht-oriëntatiecode → noord aangenomen",
        &begr.data_id[..8.min(begr.data_id.len())],
        what
    ));
    BengAdjacency::Buitenlucht {
        orientatie: Orientation::Noord,
    }
}

/// `GVL_BTNL_N`/`DAK_BTNL_HOR` → [`Orientation`]. `prefix` is `GVL_BTNL_` of
/// `DAK_BTNL_`; het suffix is de kompas-/HOR-code.
fn orientation_from_code(code: Option<&str>, prefix: &str) -> Option<Orientation> {
    let suffix = code?.strip_prefix(prefix)?;
    Some(match suffix {
        "N" => Orientation::Noord,
        "NO" => Orientation::NoordOost,
        "O" => Orientation::Oost,
        "ZO" => Orientation::ZuidOost,
        "Z" => Orientation::Zuid,
        "ZW" => Orientation::ZuidWest,
        "W" => Orientation::West,
        "NW" => Orientation::NoordWest,
        "HOR" => Orientation::Horizontaal,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Ramen/deuren
// ---------------------------------------------------------------------------

fn map_windows(
    idx: &EntityIndex,
    begr: &Entity,
    ctx: &mut GeoCtx,
    warnings: &mut Vec<String>,
) -> Vec<BengWindowPlacement> {
    let mut ramen = Vec::new();
    for constrt in idx.children_of(begr, "CONSTRT") {
        let Some(lib_id) = constrt.prop("CONSTRT_LIB") else {
            warnings.push(format!(
                "CONSTRT {} mist CONSTRT_LIB → overgeslagen",
                &constrt.data_id[..8.min(constrt.data_id.len())]
            ));
            continue;
        };
        let aantal = constrt
            .prop("CONSTRT_AANT")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);

        // Resolve het kozijnmerk. Bij de per-merk-modus (AC gevuld) verwijst de
        // plaatsing naar de gedeelde WindowDef; bij de per-raam-modus (AC leeg)
        // synthetiseren we een plaatsing-eigen WindowDef uit CONSTRT_OPP.
        let Some(kozijn_ref) = resolve_window_ref(constrt, lib_id, aantal, ctx, warnings) else {
            continue;
        };

        let belemmering = match constrt.prop("CONSTRT_BESCH") {
            None => Obstruction::None,
            Some("BELEMTYPE_MIN") => Obstruction::Minimal,
            Some(code) if code.starts_with("BELEMTYPE_ZIJ") => {
                warnings.push(format!(
                    "CONSTRT {}: zijbelemmering {} → benaderd als 'minimal' (Obstruction-V1 kent geen zij)",
                    &constrt.data_id[..8.min(constrt.data_id.len())],
                    code
                ));
                Obstruction::Minimal
            }
            Some(other) => {
                warnings.push(format!(
                    "CONSTRT {}: onbekende belemmering {} → 'minimal'",
                    &constrt.data_id[..8.min(constrt.data_id.len())],
                    other
                ));
                Obstruction::Minimal
            }
        };

        let zonwering = map_shading(constrt, warnings);

        // ZOMERNVENT_NAANW = "niet aanwezig" → false; elke andere waarde → true.
        let zomernachtventilatie = matches!(constrt.prop("CONSTRT_ZNVENT"), Some(v) if v != "ZOMERNVENT_NAANW");

        ramen.push(BengWindowPlacement {
            kozijn_ref,
            aantal,
            belemmering,
            zonwering,
            zomernachtventilatie,
        });
    }
    ramen
}

/// Resolve de kozijn-referentie van één plaatsing en retourneer de te gebruiken
/// `WindowDef`-id, of `None` als het merk onbruikbaar is (dangling / geen U).
///
/// - **Per-merk-modus** (`LibWindow::area` gevuld): de plaatsing hergebruikt de
///   up-front opgenomen gedeelde def; id = `LIBCONSTRT`-GUID.
/// - **Per-raam-modus** (`area` leeg): het oppervlak staat op `CONSTRT_OPP`; we
///   synthetiseren een plaatsing-eigen def (id = `CONSTRT`-GUID, oppervlak =
///   `CONSTRT_OPP / aantal`) en voegen die aan de bibliotheek toe.
fn resolve_window_ref(
    constrt: &Entity,
    lib_id: &str,
    aantal: u32,
    ctx: &mut GeoCtx,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let Some(lib) = ctx.lib_windows.get(lib_id) else {
        warnings.push(format!(
            "CONSTRT {}: kozijnmerk {} niet gevonden → raam overgeslagen",
            &constrt.data_id[..8.min(constrt.data_id.len())],
            &lib_id[..8.min(lib_id.len())]
        ));
        return None;
    };

    // Per-merk-modus: de gedeelde def staat al in de bibliotheek.
    if lib.area.is_some() {
        return Some(lib_id.to_string());
    }

    // Per-raam-modus: oppervlak per exemplaar uit CONSTRT_OPP.
    let Some(opp) = constrt.num("CONSTRT_OPP") else {
        warnings.push(format!(
            "CONSTRT {}: per-raam-modus zonder CONSTRT_OPP → raam overgeslagen",
            &constrt.data_id[..8.min(constrt.data_id.len())]
        ));
        return None;
    };
    let per_exemplaar = opp / f64::from(aantal.max(1));
    if per_exemplaar <= 0.0 {
        warnings.push(format!(
            "CONSTRT {}: niet-positief raamoppervlak → overgeslagen",
            &constrt.data_id[..8.min(constrt.data_id.len())]
        ));
        return None;
    }
    let synth_id = constrt.data_id.clone();
    if ctx.window_ids.insert(synth_id.clone()) {
        let (kind, u, ggl, omschrijving) =
            (lib.kind, lib.u, lib.ggl, lib.omschrijving.clone());
        ctx.window_defs.push(WindowDef {
            id: synth_id.clone(),
            omschrijving,
            kind,
            u_w_per_m2k: u,
            ggl,
            area_m2: per_exemplaar,
        });
    }
    Some(synth_id)
}

/// `CONSTRT_ZONW` → optionele [`MovableSunShading`]. `ZONW_GEEN`/leeg → geen
/// zonwering. Overige codes worden in V1 nog niet naar `F_c`/regime vertaald →
/// waarschuwing + `None`.
fn map_shading(
    constrt: &crate::parse::Entity,
    warnings: &mut Vec<String>,
) -> Option<MovableSunShading> {
    match constrt.prop("CONSTRT_ZONW") {
        None | Some("ZONW_GEEN") => None,
        Some(code) => {
            warnings.push(format!(
                "CONSTRT {}: zonwering {} nog niet gemapt (V1) → geen zonwering",
                &constrt.data_id[..8.min(constrt.data_id.len())],
                code
            ));
            None
        }
    }
}
