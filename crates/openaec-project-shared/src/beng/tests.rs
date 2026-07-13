//! Integratietests voor [`compute_beng`] — synthetisch all-electric
//! rijtjeshuis (WP-bodem + WTW-D + PV) met plausibiliteits- en
//! monotonie-asserts. Geen golden-waarden: die staan (rood, `#[ignore]`) in
//! `tests/beng_golden.rs` en worden pas in F3 geactiveerd.

use super::*;

use crate::energy::{
    CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput, EnergyInput, HeatEmissionType,
    HeatGeneratorType, HeatingInput, PvInput, ValueSource, ValueSourceKind, VentilationInput,
    VentilationSystemType,
};
use crate::beng_geometry::{
    BengAdjacency, BengBoundary, BengGeometry, BengZone, OpaqueConstructionDef, RcOrU, VlakType,
};
use crate::geometry::{
    BoundaryKind, Construction, ConstructionKind, Opening, OpeningKind, SharedGeometry, Space,
};
use crate::shared::{BuildingTypeShared, ResidentialType};
use crate::ProjectV2;

/// Bouwt een exterieur-constructie met één raam.
fn wall(
    id: &str,
    orientation_deg: f64,
    area_m2: f64,
    u_value: f64,
    window_area: f64,
) -> Construction {
    Construction {
        id: id.into(),
        description: format!("gevel {id}"),
        kind: ConstructionKind::Wall,
        boundary: BoundaryKind::Exterior,
        area_m2,
        u_value,
        orientation_deg: Some(orientation_deg),
        slope_deg: Some(90.0),
        openings: if window_area > 0.0 {
            vec![Opening {
                id: format!("{id}-raam"),
                kind: OpeningKind::Window,
                area_m2: window_area,
                u_value: 1.4,
                g_value: Some(0.6),
                frame_fraction: Some(0.25),
                movable_shading: None,
                obstruction: Default::default(),
            }]
        } else {
            vec![]
        },
        layers: vec![],
        adjacent_space_id: None,
        psi_thermal_bridge: None,
        ground_perimeter_m: None,
    }
}

/// Plat dak of vloer (opaak, geen ramen).
fn opaque(id: &str, kind: ConstructionKind, boundary: BoundaryKind, area_m2: f64, u: f64) -> Construction {
    Construction {
        id: id.into(),
        description: id.into(),
        kind,
        boundary,
        area_m2,
        u_value: u,
        orientation_deg: if matches!(kind, ConstructionKind::Wall) { Some(0.0) } else { None },
        slope_deg: Some(if matches!(kind, ConstructionKind::Roof) { 0.0 } else { 90.0 }),
        openings: vec![],
        layers: vec![],
        adjacent_space_id: None,
        psi_thermal_bridge: None,
        ground_perimeter_m: None,
    }
}

/// Uitwendige constructie met vrij instelbare kind/oriëntatie/helling (voor de
/// §5.7.2-bucket-classificatietests), optioneel met één raam.
fn face(
    id: &str,
    kind: ConstructionKind,
    orientation_deg: Option<f64>,
    slope_deg: Option<f64>,
    area_m2: f64,
    u_value: f64,
    window_area: f64,
) -> Construction {
    Construction {
        id: id.into(),
        description: id.into(),
        kind,
        boundary: BoundaryKind::Exterior,
        area_m2,
        u_value,
        orientation_deg,
        slope_deg,
        openings: if window_area > 0.0 {
            vec![Opening {
                id: format!("{id}-raam"),
                kind: OpeningKind::Window,
                area_m2: window_area,
                u_value: 1.4,
                g_value: Some(0.6),
                frame_fraction: Some(0.25),
                movable_shading: None,
                obstruction: Default::default(),
            }]
        } else {
            vec![]
        },
        layers: vec![],
        adjacent_space_id: None,
        psi_thermal_bridge: None,
        ground_perimeter_m: None,
    }
}

/// Synthetisch all-electric rijtjeshuis (Bouwbesluit+ isolatie, WP-bodem,
/// balansventilatie D met WTW, vrije bodemkoeling, PV zuid). `pv_kwp` schaalt
/// het PV-veld zodat de monotonie-tests kunnen variëren.
fn synthetic_rijtjeshuis(pv_kwp: f64) -> ProjectV2 {
    let mut p = ProjectV2::new("Synthetisch rijtjeshuis");
    p.shared.building_type = BuildingTypeShared::Woning {
        subtype: ResidentialType::Terraced,
    };
    p.shared.gross_floor_area_m2 = Some(87.0);
    p.shared.num_storeys = Some(2);
    p.shared.construction_year = Some(2022);

    p.geometry = SharedGeometry {
        spaces: vec![Space {
            id: "s1".into(),
            name: "Woning".into(),
            function: None,
            floor_area_m2: 87.0,
            height_m: 2.7,
            theta_i_winter_c: Some(20.0),
            theta_i_summer_c: Some(24.0),
            constructions: vec![
                // Voor- en achtergevel (ZW / NO), rest zijn bouwmuren (geen verlies).
                wall("gevel-zw", 225.0, 34.0, 0.21, 12.0),
                wall("gevel-no", 45.0, 34.0, 0.21, 6.0),
                opaque("dak", ConstructionKind::Roof, BoundaryKind::Exterior, 44.0, 0.16),
                opaque("vloer", ConstructionKind::Floor, BoundaryKind::Ground, 44.0, 0.26),
            ],
        }],
        ..Default::default()
    };

    p.energy = Some(EnergyInput {
        heating: Some(HeatingInput {
            generator: HeatGeneratorType::HeatPumpGround,
            cop: Some(4.5),
            hr_class: None,
            district_factor: None,
            emission: Some(HeatEmissionType::FloorHeating),
            distribution_efficiency: None,
            control_factor: None,
            coverage_fraction: 1.0,
            source: None,
        }),
        dhw: Some(DhwInput {
            generator: DhwGeneratorType::HeatPump,
            efficiency: Some(2.8),
            dwtw: None,
            has_solar_boiler: false,
            solar_boiler_fraction: None,
            source: None,
        }),
        ventilation: Some(VentilationInput {
            system: VentilationSystemType::D,
            wtw_efficiency: Some(0.85),
            sfp_w_per_m3h: None,
            bypass_enabled: true,
            mechanical_supply_m3_per_h: Some(150.0),
            mechanical_exhaust_m3_per_h: Some(150.0),
            infiltration_m3_per_h: None,
            q_v10_spec_dm3_s_m2: None,
            source: None,
        }),
        cooling: Some(CoolingInput {
            generator: CoolingGeneratorType::FreeCooling,
            seer: None,
            cop: None,
            free_cooling_fraction: Some(0.4),
            source: None,
        }),
        pv: if pv_kwp > 0.0 {
            vec![PvInput {
                id: Some("dak-zuid".into()),
                name: None,
                peak_power_kwp: pv_kwp,
                azimuth_degrees: 180.0,
                tilt_degrees: 15.0,
                system_efficiency: None,
                inverter_efficiency: None,
                shadow_factor: None,
                source: None,
            }]
        } else {
            vec![]
        },
        automation: None,
    });

    p
}

/// Zet op elk raam-`Opening` een beweegbare zonwering + externe belemmering.
/// Gebruikt door de F3d-2-smoke om de RVO-typische instelling na te bootsen.
fn apply_shading(p: &mut ProjectV2, shading: Option<nta8800_model::MovableSunShading>, obstruction: nta8800_model::Obstruction) {
    for space in &mut p.geometry.spaces {
        for c in &mut space.constructions {
            for o in &mut c.openings {
                if matches!(o.kind, OpeningKind::Window) {
                    o.movable_shading = shading;
                    o.obstruction = obstruction;
                }
            }
        }
    }
}

/// F3d-2 smoke — WP-bodem-tussenwoning, PV = 0 (matcht het F2b-anker).
/// Rapporteert B1/B2/B3/koeling voor drie stappen en toetst tegen het
/// RVO-anker (54,8 / 29,3 / 59 %). `cargo test -p openaec-project-shared
/// f3d2_smoke -- --ignored --nocapture`.
#[test]
#[ignore = "smoke/diagnostiek — draai handmatig met --nocapture"]
fn f3d2_smoke_wp_tussenwoning() {
    use nta8800_model::{MovableSunShading, Obstruction, ShadingControl};
    let screens = MovableSunShading { f_c: 0.20, control: ShadingControl::ManualResidential };

    let report = |label: &str, p: &ProjectV2| {
        let r = compute_beng(p).expect("compute_beng ok");
        println!(
            "{label:<28} B1={:6.1}  B2={:6.1}  B3={:5.1}%  koeling={:6.2} kWh/m²  TOjuli={:.2}K",
            r.beng1.value,
            r.beng2.value,
            r.beng3.value,
            r.service_breakdown_kwh_m2.cooling,
            r.tojuli.max_tojuli_k,
        );
        r
    };

    println!("\n=== F3d-2 smoke — WP-tussenwoning, PV=0, RVO-anker B1 54,8 / B2 29,3 / B3 59% ===");
    let base = synthetic_rijtjeshuis(0.0);
    report("baseline (geen zonwering)", &base);

    let mut screens_only = synthetic_rijtjeshuis(0.0);
    apply_shading(&mut screens_only, Some(screens), Obstruction::None);
    report("screens (split, geen belemm.)", &screens_only);

    let mut full = synthetic_rijtjeshuis(0.0);
    apply_shading(&mut full, Some(screens), Obstruction::Minimal);
    let r_full = report("screens + belemmering (F3d-2)", &full);

    // Variant zonder actieve koeling → per-oriëntatie-TOjuli (§5.7.2).
    let mut no_cool = synthetic_rijtjeshuis(0.0);
    apply_shading(&mut no_cool, Some(screens), Obstruction::Minimal);
    if let Some(e) = no_cool.energy.as_mut() {
        e.cooling = None;
    }
    let r_nc = compute_beng(&no_cool).expect("compute_beng ok");
    println!(
        "zonder koeling (F3d-2)       TOjuli={:.2}K (limiet {:.2}K)  method={:?}",
        r_nc.tojuli.max_tojuli_k, r_nc.tojuli.limit_k, r_nc.tojuli.method
    );

    // Zwakke sanity (geen anker-assert; kalibratie vergt realistische invoer):
    // de belemmering + balans-splitsing tillen B1 t.o.v. het screens-only-geval.
    assert!(r_full.beng1.value.is_finite());
}

#[test]
fn compute_beng_errors_without_energy_block() {
    let mut p = synthetic_rijtjeshuis(4.0);
    p.energy = None;
    assert!(matches!(compute_beng(&p), Err(BengError::MissingEnergyInput)));
}

// ---------------------------------------------------------------------------
// MZ-V2a — multi-rekenzone (gepoold, indicatief)
// ---------------------------------------------------------------------------

/// Synthetisch multi-rekenzone `beng_geometry` (geen klantdata): twee
/// rekenzones binnen één woning, elk met een vloer-op-grond + een zuidgevel.
/// Zwaar/licht/open → herkende bouwwijze-codes (C_m afleidbaar).
fn multizone_beng_geometry(a_g_zone1: f64, a_g_zone2: f64) -> BengGeometry {
    let opaque = |id: &str, kind: VlakType, rc: f64| OpaqueConstructionDef {
        id: id.into(),
        omschrijving: id.into(),
        kind,
        thermal: RcOrU::Rc(rc),
    };
    let floor = |id: &str, area: f64| BengBoundary {
        id: id.into(),
        omschrijving: "vloer".into(),
        vlak_type: VlakType::Vloer,
        grenst_aan: BengAdjacency::VloerOpMaaiveldBovenGrond,
        bruto_buiten_opp_m2: area,
        helling_deg: None,
        omtrek_p_m: Some(4.0 * area.sqrt()),
        constructie_ref: "vloer-def".into(),
        ramen: vec![],
    };
    let wall = |id: &str| BengBoundary {
        id: id.into(),
        omschrijving: "gevel".into(),
        vlak_type: VlakType::Gevel,
        grenst_aan: BengAdjacency::Buitenlucht {
            orientatie: crate::Orientation::Zuid,
        },
        bruto_buiten_opp_m2: 30.0,
        helling_deg: Some(90.0),
        omtrek_p_m: None,
        constructie_ref: "gevel-def".into(),
        ramen: vec![],
    };
    let zone = |id: &str, a_g: f64| BengZone {
        id: id.into(),
        naam: id.into(),
        a_g_m2: a_g,
        bouwwijze_vloer: Some("CONSTRM_FL_26".into()),
        bouwwijze_wand: Some("CONSTRM_W_11".into()),
        woningtype: Some("TWON_VRIJ_K".into()),
        gevels: vec![floor(&format!("{id}-vloer"), a_g), wall(&format!("{id}-gevel-z"))],
    };
    BengGeometry {
        opaque_defs: vec![
            opaque("vloer-def", VlakType::Vloer, 3.7),
            opaque("gevel-def", VlakType::Gevel, 4.7),
        ],
        window_defs: vec![],
        zones: vec![zone("zone-groot", a_g_zone1), zone("zone-klein", a_g_zone2)],
    }
}

/// Φ_int-regressie (MZ-V2a): twee rekenzones (100 + 50 m²) → interne warmtewinst
/// op A_g;tot = 150 (§6.6.2, formule 7.21), **niet** op de eerste zone. Pint de
/// note-flux op de som-berekening en het gepoolde A_g op het BENG-resultaat.
#[test]
fn multizone_phi_int_scales_on_total_a_g() {
    let mut p = synthetic_rijtjeshuis(0.0);
    p.beng_geometry = Some(multizone_beng_geometry(100.0, 50.0));
    let r = compute_beng(&p).expect("compute_beng ok");

    // De view poolt beide zones → A_g;tot = 150.
    assert!((r.a_g_m2 - 150.0).abs() < 1e-6, "pooled A_g = {}", r.a_g_m2);

    let phi_note = r
        .notes
        .iter()
        .find(|n| n.contains("Interne warmtewinst (C3b)"))
        .expect("Φ_int-note aanwezig");
    assert!(phi_note.contains("A_g;tot = 150.00 m²"), "note: {phi_note}");

    // Flux op de som (150), niet op de eerste zone (100).
    let flux_sum = dynamics::derive_internal_gains_woningbouw(150.0, 1.0).heat_flux_per_m2
        [nta8800_model::time::Month::Juli];
    assert!(phi_note.contains(&format!("{flux_sum:.2} W/m²")), "note: {phi_note}");
    let flux_first = dynamics::derive_internal_gains_woningbouw(100.0, 1.0).heat_flux_per_m2
        [nta8800_model::time::Month::Juli];
    assert!(
        !phi_note.contains(&format!("= {flux_first:.2} W/m²")),
        "note gebruikt (fout) de eerste-zone-flux: {phi_note}"
    );
}

/// MZ-V2b: bij meerdere rekenzones draagt het resultaat de norm-exact-note (geen
/// INDICATIEF-markering meer) plus een per-zone-C_m-note per rekenzone.
#[test]
fn multizone_emits_v2b_note_and_per_zone_cm() {
    let mut p = synthetic_rijtjeshuis(0.0);
    p.beng_geometry = Some(multizone_beng_geometry(120.0, 40.0));
    let r = compute_beng(&p).expect("compute_beng ok");

    assert!(
        !r.notes.iter().any(|n| n.contains("INDICATIEF (MZ-V2a)")),
        "V2b mag geen INDICATIEF (MZ-V2a)-note meer dragen: {:?}",
        r.notes
    );
    assert!(
        r.notes.iter().any(|n| n.contains("MZ-V2b (norm-exact)")),
        "V2b-norm-exact-note ontbreekt: {:?}",
        r.notes
    );
    // Per-zone C_m-note voor beide rekenzones (§7.7).
    assert!(
        r.notes.iter().any(|n| n.contains("Rekenzone 'zone-groot'")),
        "per-zone-note zone-groot ontbreekt: {:?}",
        r.notes
    );
    assert!(
        r.notes.iter().any(|n| n.contains("Rekenzone 'zone-klein'")),
        "per-zone-note zone-klein ontbreekt: {:?}",
        r.notes
    );
}

/// Één rekenzone via `beng_geometry` → géén indicatief-note (byte-identiek gedrag
/// t.o.v. vóór MZ-V2a; de dominante zone valt samen met die ene zone).
#[test]
fn singlezone_beng_geometry_has_no_indicative_note() {
    let mut p = synthetic_rijtjeshuis(0.0);
    let mut geo = multizone_beng_geometry(90.0, 10.0);
    geo.zones.truncate(1);
    p.beng_geometry = Some(geo);
    let r = compute_beng(&p).expect("compute_beng ok");

    assert!(
        !r.notes.iter().any(|n| n.contains("INDICATIEF (MZ-V2a)")),
        "single-zone mag geen indicatief-note hebben: {:?}",
        r.notes
    );
}

/// QC-guard: een `beng_geometry` met **lege** zones + Woonfunctie mag niet paniken.
/// De match-guard `!bg.zones.is_empty()` laat de bridging-arm niet toe → val terug
/// op de ruimte-geometrie (pre-V2b-gedrag: geen bridging, geen C3b-Φ_int-deling).
/// De defensieve `a_g_total > 0`-guard dekt bovendien een hypothetische A_g;tot = 0
/// (die `beng_geometry.rs`-validatie al uitsluit). Reproduceert: `compute_beng` Ok
/// zonder MZ-note.
#[test]
fn empty_zones_beng_geometry_does_not_panic() {
    let mut p = synthetic_rijtjeshuis(0.0);
    let mut geo = multizone_beng_geometry(90.0, 10.0);
    geo.zones.clear();
    p.beng_geometry = Some(geo);

    // Mag niet paniken; de bridging-arm draait niet (lege zones) → ruimte-geometrie.
    let r = compute_beng(&p).expect("lege zones: compute_beng valt terug op de ruimte-geometrie");
    assert!(
        !r.notes
            .iter()
            .any(|n| n.contains("MZ-V2b") || n.contains("INDICATIEF (MZ-V2a)")),
        "lege zones mogen geen multi-zone-note geven (arm draaide niet): {:?}",
        r.notes
    );
    // A_g volgt de ruimte-geometrie (87 m²), niet de lege BENG-zones.
    assert!((r.a_g_m2 - 87.0).abs() < 1e-6, "A_g = {}", r.a_g_m2);
}

#[test]
fn synthetic_house_produces_plausible_beng() {
    let p = synthetic_rijtjeshuis(4.0);
    let r = compute_beng(&p).expect("compute_beng ok");

    // BENG 1 (energiebehoefte) in een ruime plausibele band voor een goed
    // geïsoleerde nieuwbouwwoning.
    assert!(
        (30.0..=120.0).contains(&r.beng1.value),
        "BENG 1 buiten band: {}",
        r.beng1.value
    );
    // Woonfunctie ⇒ grenswaarden aanwezig.
    assert!(r.beng1.limit.is_some());
    assert!(r.beng2.limit.is_some());
    assert!(r.beng3.limit.is_some());

    // BENG 2 (primair fossiel) in een norm-plausibele band en voldoet aan de eis.
    // Na de F3b-koelfix + rencold salderen 4 kWp PV de all-electric-woning tot
    // een negatieve BENG 2 — norm-valide (§5.5.2 opm. 11, geen clamp), vandaar de
    // negatieve ondergrens i.p.v. een positiviteits-assert.
    assert!(
        (-100.0..150.0).contains(&r.beng2.value),
        "BENG 2 buiten plausibele band: {}",
        r.beng2.value
    );
    assert_eq!(r.beng2.pass, Some(true), "BENG 2 zou moeten voldoen: {}", r.beng2.value);
    // BENG 3 (hernieuwbaar aandeel) > 0 dankzij PV.
    assert!(r.beng3.value > 0.0, "BENG 3 = {}", r.beng3.value);
    assert!(r.beng3.value <= 100.0);

    // Vrije bodemkoeling = actief koelsysteem ⇒ TOjuli 0, geacht te voldoen.
    assert!(r.tojuli.actively_cooled);
    assert_eq!(r.tojuli.pass, Some(true));
    assert!(r.tojuli.max_tojuli_k.abs() < 1e-12);

    // Geometrie-kentallen.
    assert!((r.a_g_m2 - 87.0).abs() < 1e-6);
    assert!(r.a_ls_m2 > 0.0);
    assert!(r.als_ag_ratio > 0.0);

    // PV vermijdt primair-fossiele energie (fP;exp;el = 1,45, §5.5), dus de
    // PV-dienst levert een negatief primair energiegebruik.
    assert!(r.service_breakdown_kwh_m2.pv < 0.0);
    assert!(r.service_breakdown_kwh_m2.heating > 0.0);
    assert!(!r.energy_label.is_empty());
    // F3b — vrije-koeling-opwekkingsstap (EER_fc + backup, tabel 10.34/10.29):
    // koeling zit na de fix in een plausibele band, niet meer op de ~56
    // kWh/(m²·jr) van de COP=1,0-modellering. (Het residu boven de RVO-referentie
    // is F_sh=1,0-overschatting van Q_C;nd — F3d, buiten F3b-scope.)
    assert!(
        (0.0..30.0).contains(&r.service_breakdown_kwh_m2.cooling),
        "koeling buiten band na F3b-fix: {}",
        r.service_breakdown_kwh_m2.cooling
    );
    // De F3b-gap-note (rencold telt niet mee) is verdwenen.
    assert!(!r.notes.iter().any(|n| n.contains("levert QC;gen;out pas in F3b")));
}

#[test]
fn free_cooling_yields_renewable_cold_raising_beng3() {
    // §5.6.2.2 (5.34): de vrij geleverde koude (EER ≥ 8) telt als hernieuwbaar,
    // dus vrije bodemkoeling verhoogt BENG 3 t.o.v. compressiekoeling (EER 3 < 8)
    // bij verder identieke invoer.
    let free = compute_beng(&synthetic_rijtjeshuis(0.0)).expect("free cooling ok");

    let mut p = synthetic_rijtjeshuis(0.0);
    if let Some(e) = p.energy.as_mut() {
        e.cooling = Some(CoolingInput {
            generator: CoolingGeneratorType::Compression,
            seer: None,
            cop: None,
            free_cooling_fraction: None,
            source: None,
        });
    }
    let compression = compute_beng(&p).expect("compression ok");

    assert!(
        free.beng3.value > compression.beng3.value,
        "vrije koeling (rencold) zou BENG 3 moeten verhogen: {} vs {}",
        free.beng3.value,
        compression.beng3.value
    );
    // Compressie levert geen rencold én kost meer eindenergie (EER 3 vs deels vrij)
    // → hoger primair-fossiel koelverbruik.
    assert!(
        compression.service_breakdown_kwh_m2.cooling > free.service_breakdown_kwh_m2.cooling,
        "compressiekoeling zou meer primair koelverbruik moeten geven: {} vs {}",
        compression.service_breakdown_kwh_m2.cooling,
        free.service_breakdown_kwh_m2.cooling
    );
}

#[test]
fn more_pv_raises_beng3_and_lowers_beng2() {
    let low = compute_beng(&synthetic_rijtjeshuis(2.0)).expect("low pv ok");
    let high = compute_beng(&synthetic_rijtjeshuis(8.0)).expect("high pv ok");

    // BENG 3 (hernieuwbaar aandeel) stijgt monotoon met het PV-vermogen.
    assert!(
        high.beng3.value > low.beng3.value,
        "meer PV zou BENG 3 moeten verhogen: {} vs {}",
        high.beng3.value,
        low.beng3.value
    );

    // PV-saldering (§5.5, formule 5.10 + tabel 5.2, fP;exp;el = 1,45): meer PV
    // verlaagt het karakteristieke primair-fossiele energiegebruik (BENG 2).
    // Grootte-orde: ΔPV = 6 kWp levert honderden kWh/jr extra × 1,45 primair over
    // ~87 m² → duidelijk meetbaar verschil, ruim boven numerieke ruis.
    assert!(
        high.beng2.value < low.beng2.value - 1e-6,
        "meer PV zou BENG 2 moeten verlagen: {} vs {}",
        high.beng2.value,
        low.beng2.value
    );
}

#[test]
fn no_active_cooling_uses_per_orientation() {
    let mut p = synthetic_rijtjeshuis(4.0);
    // Verwijder het koelsysteem → geen actieve koeling → per-oriëntatie-toets.
    if let Some(e) = p.energy.as_mut() {
        e.cooling = None;
    }
    let r = compute_beng(&p).expect("compute_beng ok");
    assert!(!r.tojuli.actively_cooled);
    assert_eq!(r.tojuli.method, TojuliMethod::PerOrientation);
    // Zonder actieve koeling levert de per-oriëntatie-toets nu een pass/fail.
    assert!(r.tojuli.pass.is_some());
    assert!(r.tojuli.max_tojuli_k >= 0.0);
    assert!((r.tojuli.limit_k - 1.20).abs() < 1e-9);
}

// ---------------------------------------------------------------------------
// Per-oriëntatie TOjuli-opbouw (§5.7.2) — build_tojuli_orientation_inputs
// ---------------------------------------------------------------------------

use nta8800_model::location::Orientation;
use nta8800_model::time::MonthlyProfile;

/// Minimale [`TojuliResult`] met alleen de door de per-oriëntatie-opbouw gelezen
/// velden gevuld (julikoudebehoefte + H_V); de rest is neutraal.
fn fake_tj(july_q_c_mj: f64, h_v: f64) -> crate::tojuli::TojuliResult {
    crate::tojuli::TojuliResult {
        monthly_q_c_nd_mj: MonthlyProfile::from_constant(july_q_c_mj),
        monthly_q_c_use_mj: MonthlyProfile::from_constant(0.0),
        annual_q_c_use_mj: 0.0,
        annual_q_c_use_kwh: 0.0,
        annual_rencold_mj: 0.0,
        monthly_q_h_nd_mj: MonthlyProfile::from_constant(0.0),
        transmission_h_t_w_per_k: 0.0,
        ventilation_h_v_w_per_k: h_v,
        monthly_theta_e_c: MonthlyProfile::from_constant(18.0),
        tau_hours: 100.0,
    }
}

fn geom(constructions: Vec<Construction>) -> SharedGeometry {
    SharedGeometry {
        spaces: vec![Space {
            id: "s1".into(),
            name: "zone".into(),
            function: None,
            floor_area_m2: 80.0,
            height_m: 2.7,
            theta_i_winter_c: Some(20.0),
            theta_i_summer_c: Some(24.0),
            constructions,
        }],
        ..Default::default()
    }
}

fn find(inputs: &[TojuliOrientationInput], or: Orientation) -> Option<&TojuliOrientationInput> {
    inputs.iter().find(|i| i.orientation == or)
}

#[test]
fn per_orientation_south_heavy_glazing_governs() {
    // Zuid: groot raam; Noord: klein raam; gelijke wandoppervlakte. De
    // zonwinst-gewogen teller maakt Zuid maatgevend.
    let inputs = build_tojuli_orientation_inputs(
        &geom(vec![
            wall("z", 180.0, 20.0, 0.3, 15.0),
            wall("n", 0.0, 20.0, 0.3, 2.0),
        ]),
        &fake_tj(500.0, 30.0),
    );
    let z = find(&inputs, Orientation::Zuid).expect("zuid aanwezig");
    let n = find(&inputs, Orientation::Noord).expect("noord aanwezig");
    assert!(
        z.q_c_nd_juli_kwh > n.q_c_nd_juli_kwh,
        "zuid-teller {} zou groter moeten zijn dan noord {}",
        z.q_c_nd_juli_kwh,
        n.q_c_nd_juli_kwh
    );

    // De maatgevende oriëntatie (hoogste TOjuli) is Zuid.
    let zone = nta8800_ep::tojuli_zone(&inputs, super::T_JULI_H, false);
    let per_max = zone
        .per_orientation
        .iter()
        .filter_map(|r| r.tojuli_k.map(|k| (r.orientation, k)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .expect("een beoordeelde oriëntatie");
    assert_eq!(per_max.0, Orientation::Zuid, "Zuid zou maatgevend moeten zijn");
}

#[test]
fn per_orientation_windowless_zone_is_zero() {
    // Geen ramen én geen julikoudebehoefte → alle TOjuli 0, zone voldoet.
    let inputs = build_tojuli_orientation_inputs(
        &geom(vec![
            wall("z", 180.0, 20.0, 0.3, 0.0),
            wall("o", 90.0, 20.0, 0.3, 0.0),
        ]),
        &fake_tj(0.0, 30.0),
    );
    for i in &inputs {
        assert!(i.q_c_nd_juli_kwh.abs() < 1e-12, "teller niet 0: {i:?}");
    }
    let zone = nta8800_ep::tojuli_zone(&inputs, super::T_JULI_H, false);
    assert!(zone.max_tojuli_k.abs() < 1e-12);
    assert!(zone.pass);
}

#[test]
fn per_orientation_symmetric_facades_equal_denominator() {
    // Identieke gevel+raam op Oost en West: de noemer-termen (A_T, H_C;D, H_gr,
    // H_ve) zijn per oriëntatie gelijk; de teller verschilt uitsluitend door de
    // julizoninstraling (norm-correct: Oost 104,9 vs West 112,7 W/m²).
    let inputs = build_tojuli_orientation_inputs(
        &geom(vec![
            wall("o", 90.0, 20.0, 0.3, 10.0),
            wall("w", 270.0, 20.0, 0.3, 10.0),
        ]),
        &fake_tj(400.0, 24.0),
    );
    let o = find(&inputs, Orientation::Oost).expect("oost");
    let w = find(&inputs, Orientation::West).expect("west");
    assert!((o.a_t_m2 - w.a_t_m2).abs() < 1e-9);
    assert!((o.h_c_d_juli_w_per_k - w.h_c_d_juli_w_per_k).abs() < 1e-9);
    assert!((o.h_gr_an_juli_w_per_k - w.h_gr_an_juli_w_per_k).abs() < 1e-9);
    assert!((o.h_c_ve_juli_w_per_k - w.h_c_ve_juli_w_per_k).abs() < 1e-9);
    // Teller-verhouding volgt de julizoninstraling-verhouding Oost/West.
    let ratio = o.q_c_nd_juli_kwh / w.q_c_nd_juli_kwh;
    assert!((ratio - 104.9 / 112.7).abs() < 1e-6, "verhouding {ratio}");
}

#[test]
fn per_orientation_pitched_roof_is_orientation_bound() {
    // §5.7.2 Stap A/2: een hellend dakvlak (Roof, helling 45°) mét azimuth is
    // ORIËNTATIEGEBONDEN — A·U, raamoppervlak en zonwinst landen in de bucket
    // van zijn oriëntatie (Zuid), niet in de pro-rata-pool.
    let roof = face("dak-z", ConstructionKind::Roof, Some(180.0), Some(45.0), 30.0, 0.16, 4.0);
    let wall_n = face("gevel-n", ConstructionKind::Wall, Some(0.0), Some(90.0), 20.0, 0.3, 0.0);
    let inputs = build_tojuli_orientation_inputs(&geom(vec![roof, wall_n]), &fake_tj(300.0, 20.0));

    let z = find(&inputs, Orientation::Zuid).expect("zuid aanwezig door het dakvlak");
    // A_T;Zuid = dak 30 + raam 4 = 34 m²; geen horizontaal element → H_C;D;Zuid
    // = 30·0,16 + 4·1,4 = 10,4 W/K (geen pro-rata-bijdrage).
    assert!((z.a_t_m2 - 34.0).abs() < 1e-9, "A_T Zuid = {}", z.a_t_m2);
    assert!((z.h_c_d_juli_w_per_k - 10.4).abs() < 1e-9, "H_C;D Zuid = {}", z.h_c_d_juli_w_per_k);
    // Zuid krijgt zonwinst (raam op het dakvlak) → teller > 0.
    assert!(z.q_c_nd_juli_kwh > 0.0);
}

#[test]
fn per_orientation_flat_roof_with_azimuth_is_prorata() {
    // Een (bijna-)plat dak (helling 0° ≤ 5°, §7.6.6.4) met een — mogelijk
    // abusievelijk — ingevulde azimuth telt NIET als oriëntatie-element: het gaat
    // naar de pro-rata-pool (§5.7.2 Stap 3/4), niet in A_T;or.
    let flat = face("dak-plat", ConstructionKind::Roof, Some(180.0), Some(0.0), 40.0, 0.16, 0.0);
    let wall_z = face("gevel-z", ConstructionKind::Wall, Some(180.0), Some(90.0), 20.0, 0.3, 10.0);
    let inputs = build_tojuli_orientation_inputs(&geom(vec![flat, wall_z]), &fake_tj(300.0, 24.0));

    // Alleen de Zuid-gevel levert een oriëntatie-bucket; het platte dak niet.
    assert_eq!(inputs.len(), 1, "alleen Zuid verwacht, kreeg {inputs:?}");
    let z = find(&inputs, Orientation::Zuid).expect("zuid");
    // A_T;Zuid = gevel 20 + raam 10 = 30 (platte dak zit NIET in A_T).
    assert!((z.a_t_m2 - 30.0).abs() < 1e-9, "A_T Zuid = {}", z.a_t_m2);
    // H_C;D;Zuid = gevel 20·0,3 + raam 10·1,4 = 20 W/K + pro-rata van het platte
    // dak (frac=1 → 40·0,16 = 6,4) = 26,4 W/K.
    assert!((z.h_c_d_juli_w_per_k - 26.4).abs() < 1e-9, "H_C;D Zuid = {}", z.h_c_d_juli_w_per_k);
}

#[test]
fn result_serializes_to_json() {
    let r = compute_beng(&synthetic_rijtjeshuis(4.0)).expect("compute_beng ok");
    let json = serde_json::to_string(&r).unwrap();
    let back: BengResult = serde_json::from_str(&json).unwrap();
    assert_eq!(r, back);
}

// ---------------------------------------------------------------------------
// Bronregistratie (F4c) — doorvoer naar notes + value_sources, calc-invariant
// ---------------------------------------------------------------------------

/// Basis met een DWTW-*unit* (echte reken-invoer) zodat de bron-metadata op een
/// bestaand veld gehangen kan worden zonder de berekening te veranderen.
fn base_with_dwtw() -> ProjectV2 {
    let mut p = synthetic_rijtjeshuis(4.0);
    if let Some(d) = p.energy.as_mut().and_then(|e| e.dhw.as_mut()) {
        d.dwtw = Some(crate::energy::DwtwInput {
            efficiency: 0.45,
            douche_aandeel: None,
            source: None,
        });
    }
    p
}

/// Hang niet-forfaitaire bron-metadata op verwarming, de bestaande DWTW-unit en
/// het eerste PV-veld — puur metadata, verandert géén reken-invoer.
fn attach_sources(p: &mut ProjectV2) {
    let e = p.energy.as_mut().expect("energy");
    if let Some(h) = e.heating.as_mut() {
        h.source = Some(ValueSource {
            kind: ValueSourceKind::Kwaliteitsverklaring,
            reference: Some("BCRG-20231234".into()),
        });
    }
    if let Some(w) = e.dhw.as_mut().and_then(|d| d.dwtw.as_mut()) {
        w.source = Some(ValueSource {
            kind: ValueSourceKind::Gelijkwaardigheidsverklaring,
            reference: None,
        });
    }
    if let Some(pv) = e.pv.first_mut() {
        pv.source = Some(ValueSource {
            kind: ValueSourceKind::Meting,
            reference: Some("  MEET-2024-07  ".into()),
        });
    }
}

#[test]
fn value_sources_flow_to_report_and_notes() {
    let mut p = base_with_dwtw();
    attach_sources(&mut p);
    let r = compute_beng(&p).expect("compute_beng ok");

    // Gestructureerd rapport-veld: drie niet-forfaitaire bronnen.
    assert_eq!(r.value_sources.len(), 3, "value_sources: {:?}", r.value_sources);
    let heating = r
        .value_sources
        .iter()
        .find(|s| s.system == BengSubsystem::Heating)
        .expect("heating-bron");
    assert_eq!(heating.kind, ValueSourceKind::Kwaliteitsverklaring);
    assert_eq!(heating.reference.as_deref(), Some("BCRG-20231234"));
    assert!(r.value_sources.iter().any(|s| s.system == BengSubsystem::Dwtw));
    let pv = r
        .value_sources
        .iter()
        .find(|s| s.system == BengSubsystem::Pv)
        .expect("pv-bron");
    assert_eq!(pv.label.as_deref(), Some("dak-zuid")); // uit PvInput.id
    // De referentie is getrimd opgenomen in het rapport-veld ("  MEET-...  " → "MEET-...").
    assert_eq!(pv.reference.as_deref(), Some("MEET-2024-07"));

    // Menselijk-leesbare doorvoer in notes (transparantie).
    assert!(
        r.notes.iter().any(|n| n.contains("BCRG-20231234")),
        "notes zou de kwaliteitsverklaring moeten noemen: {:?}",
        r.notes
    );
    // Referentie wordt getrimd in de note.
    assert!(r.notes.iter().any(|n| n.contains("ref. MEET-2024-07")));
}

#[test]
fn forfait_source_is_not_reported() {
    // Een expliciet forfait = norm-default → geen dossierstuk, geen report-entry.
    let mut p = synthetic_rijtjeshuis(0.0);
    if let Some(h) = p.energy.as_mut().and_then(|e| e.heating.as_mut()) {
        h.source = Some(ValueSource {
            kind: ValueSourceKind::Forfait,
            reference: None,
        });
    }
    let r = compute_beng(&p).expect("compute_beng ok");
    assert!(r.value_sources.is_empty(), "forfait mag niet gerapporteerd: {:?}", r.value_sources);
}

#[test]
fn reference_is_trimmed_and_capped_at_200_chars() {
    // Vrije-tekst-referentie mag notes/rapport/PDF niet opblazen: getrimd +
    // afgekapt op 200 tekens bij het opnemen (de ruwe DTO-invoer blijft heel).
    let mut p = synthetic_rijtjeshuis(0.0);
    let long = format!("  {}  ", "x".repeat(500));
    if let Some(h) = p.energy.as_mut().and_then(|e| e.heating.as_mut()) {
        h.source = Some(ValueSource {
            kind: ValueSourceKind::Overig,
            reference: Some(long.clone()),
        });
    }
    let r = compute_beng(&p).expect("compute_beng ok");
    let heating = r
        .value_sources
        .iter()
        .find(|s| s.system == BengSubsystem::Heating)
        .expect("heating-bron");
    let reference = heating.reference.as_deref().expect("reference aanwezig");
    assert_eq!(reference.chars().count(), 200, "referentie moet op 200 tekens gekapt zijn");
    assert!(reference.chars().all(|c| c == 'x'), "witruimte moet getrimd zijn");
    // De note draagt dezelfde gekapte referentie (geen 500-teken-lap): prefix
    // (~60) + de 200-teken-cap, ruim onder de 500-teken-invoer.
    let note = r.notes.iter().find(|n| n.contains("ref. ")).expect("bron-note");
    assert!(note.chars().count() < 300, "note mag niet opgeblazen zijn: {} tekens", note.chars().count());
}

#[test]
fn source_metadata_does_not_change_the_calculation() {
    // Bronregistratie is puur metadata: elke berekende indicator is identiek met
    // of zonder bron; alleen notes + value_sources verschillen. Beide projecten
    // hebben dezelfde reken-invoer (incl. DWTW-unit) — enige verschil = de bron.
    let base = compute_beng(&base_with_dwtw()).expect("base");
    let mut sourced_project = base_with_dwtw();
    attach_sources(&mut sourced_project);
    let sourced = compute_beng(&sourced_project).expect("sourced");

    assert_eq!(base.beng1, sourced.beng1);
    assert_eq!(base.beng2, sourced.beng2);
    assert_eq!(base.beng3, sourced.beng3);
    assert_eq!(base.tojuli, sourced.tojuli);
    assert_eq!(base.energy_label, sourced.energy_label);
    assert_eq!(base.service_breakdown_kwh_m2, sourced.service_breakdown_kwh_m2);
    assert_eq!(base.renewable_share, sourced.renewable_share);

    // Metadata verschilt wél.
    assert!(base.value_sources.is_empty());
    assert!(!sourced.value_sources.is_empty());
}
