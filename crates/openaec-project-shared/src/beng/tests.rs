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

    // BENG 2 (primair fossiel) positief maar bescheiden voor all-electric + PV.
    assert!(r.beng2.value > 0.0, "BENG 2 = {}", r.beng2.value);
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

    // De EP-crate rekent primaire-energiefactor(PV) = 0, dus de PV-dienst
    // levert 0 primair energiegebruik (F3-gat, zie de note in `compute_beng`).
    assert!(r.service_breakdown_kwh_m2.pv <= 0.0);
    assert!(r.service_breakdown_kwh_m2.heating > 0.0);
    assert!(!r.energy_label.is_empty());
    // De PV-netting-gap wordt expliciet gerapporteerd.
    assert!(r.notes.iter().any(|n| n.contains("PV verlaagt BENG 2")));
}

#[test]
fn more_pv_raises_beng3_and_leaves_beng2_unchanged_f3_gap() {
    let low = compute_beng(&synthetic_rijtjeshuis(2.0)).expect("low pv ok");
    let high = compute_beng(&synthetic_rijtjeshuis(8.0)).expect("high pv ok");

    // BENG 3 (hernieuwbaar aandeel) stijgt monotoon met het PV-vermogen.
    assert!(
        high.beng3.value > low.beng3.value,
        "meer PV zou BENG 3 moeten verhogen: {} vs {}",
        high.beng3.value,
        low.beng3.value
    );

    // TRIPWIRE — F3-gat: de EP-crate rekent f_prim(PV) = 0, dus PV verlaagt
    // BENG 2 nu NIET. Deze assert faalt zodra iemand de §5.5-zelfconsumptie-
    // aftrek implementeert; werk dan de assert (en de note) bij.
    assert!(
        (high.beng2.value - low.beng2.value).abs() < 1e-9,
        "BENG 2 wijzigt met PV — PV-netting kennelijk geïmplementeerd? {} vs {}",
        high.beng2.value,
        low.beng2.value
    );
}

#[test]
fn no_active_cooling_uses_whole_zone_screening() {
    let mut p = synthetic_rijtjeshuis(4.0);
    // Verwijder het koelsysteem → geen actieve koeling → screening-methode.
    if let Some(e) = p.energy.as_mut() {
        e.cooling = None;
    }
    let r = compute_beng(&p).expect("compute_beng ok");
    assert!(!r.tojuli.actively_cooled);
    assert_eq!(r.tojuli.method, TojuliMethod::WholeZoneScreening);
    assert!(r.tojuli.pass.is_none());
    assert!(r.tojuli.max_tojuli_k >= 0.0);
}

#[test]
fn result_serializes_to_json() {
    let r = compute_beng(&synthetic_rijtjeshuis(4.0)).expect("compute_beng ok");
    let json = serde_json::to_string(&r).unwrap();
    let back: BengResult = serde_json::from_str(&json).unwrap();
    assert_eq!(r, back);
}

