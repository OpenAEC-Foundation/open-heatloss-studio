//! Integratietests voor [`compute_beng`] — synthetisch all-electric
//! rijtjeshuis (WP-bodem + WTW-D + PV) met plausibiliteits- en
//! monotonie-asserts. Geen golden-waarden: die staan (rood, `#[ignore]`) in
//! `tests/beng_golden.rs` en worden pas in F3 geactiveerd.

use super::*;

use crate::energy::{
    CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput, EnergyInput, HeatEmissionType,
    HeatGeneratorType, HeatingInput, PvInput, VentilationInput, VentilationSystemType,
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
            }]
        } else {
            vec![]
        },
        layers: vec![],
        adjacent_space_id: None,
        psi_thermal_bridge: None,
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
            }]
        } else {
            vec![]
        },
        layers: vec![],
        adjacent_space_id: None,
        psi_thermal_bridge: None,
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
        }),
        dhw: Some(DhwInput {
            generator: DhwGeneratorType::HeatPump,
            efficiency: Some(2.8),
            dwtw: None,
            has_solar_boiler: false,
            solar_boiler_fraction: None,
        }),
        ventilation: Some(VentilationInput {
            system: VentilationSystemType::D,
            wtw_efficiency: Some(0.85),
            sfp_w_per_m3h: None,
            bypass_enabled: true,
            mechanical_supply_m3_per_h: Some(150.0),
            mechanical_exhaust_m3_per_h: Some(150.0),
            infiltration_m3_per_h: None,
        }),
        cooling: Some(CoolingInput {
            generator: CoolingGeneratorType::FreeCooling,
            seer: None,
            cop: None,
            free_cooling_fraction: Some(0.4),
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
            }]
        } else {
            vec![]
        },
        automation: None,
    });

    p
}

#[test]
fn compute_beng_errors_without_energy_block() {
    let mut p = synthetic_rijtjeshuis(4.0);
    p.energy = None;
    assert!(matches!(compute_beng(&p), Err(BengError::MissingEnergyInput)));
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
