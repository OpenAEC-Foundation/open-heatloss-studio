//! BENG golden-vangrail — fase F0 (rood/`#[ignore]`).
//!
//! Deze tests leggen de officiële BENG-eindwaarden vast VÓÓR er een end-to-end
//! `compute_beng(ProjectV2)`-functie bestaat (die komt in fase F2). Ze volgen het
//! isso53 §6.1/§6.2-precedent (`crates/isso53-core/tests/golden.rs`): de referentie
//! staat rood totdat de engine hem kan halen, en anti-fudge is absoluut — een
//! afwijking wordt gedocumenteerd/geanalyseerd, nooit weggepoetst in de expected.
//!
//! Twee bronlagen:
//!
//! 1. **RVO BENG-voorbeeldconcepten woningbouw** (DGMR B.2017.1387.02.R001 v003,
//!    26-03-2021; `tests/references/rvo-beng-voorbeeldconcepten-woningbouw-2021.pdf`).
//!    Officiële eindwaarden per woningtype × concept. Gerekend met NTA 8800:2020
//!    (validatietool v1.49) — versie-delta t.o.v. onze 2025+C1-implementatie
//!    verantwoordt de ±10%-starttolerantie. Fixtures onder
//!    `tests/verification/beng_rvo_voorbeeldconcepten/`.
//!
//! 2. **Certified Uniec 3.3.x replay** (open-energy-studio, John Heikens, LGPL-3.0).
//!    `meta.uniecReference` uit twee `.oes.json`-projecten. Deterministische invoer +
//!    sub-totalen per dienst → diagnostisch sterker dan alleen eindwaarden. Fixtures
//!    onder `tests/verification/beng_uniec_crosscheck/`.
//!
//! De `#[ignore]`-tests parsen hun fixtures (structuur-validatie) en eindigen in
//! `unimplemented!("compute_beng ontbreekt nog — F2")`. Eén niet-genegeerde test
//! (`all_expected_fixtures_have_provenance`) draait wél mee in `cargo test` en
//! bewaakt dat elke expected-waarde herleidbare provenance heeft (paginanr/JSON-pad)
//! — het vangnet tegen kale, ongeverifieerde expected-waarden.

use std::collections::BTreeMap;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// RVO-fixture schema (tests/verification/beng_rvo_voorbeeldconcepten/*/expected.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RvoExpected {
    case: String,
    #[allow(dead_code)]
    woningtype: String,
    #[allow(dead_code)]
    gebouwcode: String,
    ag_m2: f64,
    als_ag_ratio: f64,
    #[allow(dead_code)]
    bouwwijze: String,
    eisen: RvoEisen,
    source: RvoSource,
    tolerance: RvoTolerance,
    concepts: Vec<RvoConcept>,
}

#[derive(Debug, Deserialize)]
struct RvoEisen {
    beng1_max_kwh_m2_jr: f64,
    beng2_max_kwh_m2_jr: f64,
    beng3_min_pct: f64,
    tojuli_max: f64,
}

#[derive(Debug, Deserialize)]
struct RvoSource {
    document: String,
    eisen_page: u32,
    resultaten_page: u32,
}

#[derive(Debug, Deserialize)]
struct RvoTolerance {
    beng1_pct: f64,
    beng2_pct: f64,
    beng3_abs_pp: f64,
    motivatie: String,
}

#[derive(Debug, Deserialize)]
struct RvoConcept {
    id: String,
    expected: RvoConceptExpected,
    provenance: RvoProvenance,
}

#[derive(Debug, Deserialize)]
struct RvoConceptExpected {
    beng1_kwh_m2_jr: f64,
    beng2_kwh_m2_jr: f64,
    beng3_pct: f64,
    tojuli: f64,
    #[allow(dead_code)]
    wp_pv: f64,
}

#[derive(Debug, Deserialize)]
struct RvoProvenance {
    page: u32,
    row: String,
}

// ---------------------------------------------------------------------------
// Uniec-fixture schema (tests/verification/beng_uniec_crosscheck/*/expected.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct UniecExpected {
    case: String,
    #[allow(dead_code)]
    project_name: String,
    source: UniecSource,
    tolerance: UniecTolerance,
    expected: UniecValues,
    provenance: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct UniecSource {
    repo: String,
    license: String,
    certified_tool: String,
    provenance_jsonpath_root: String,
}

#[derive(Debug, Deserialize)]
struct UniecTolerance {
    beng1_pct: f64,
    beng2_pct: f64,
    beng3_abs_pp: f64,
    motivatie: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // data-mirror van meta.uniecReference; sub-totalen worden pas in F3 geasserteerd
struct UniecValues {
    beng1_kwh_m2_jr: f64,
    beng1_limit_kwh_m2_jr: f64,
    beng2_kwh_m2_jr: f64,
    beng2_limit_kwh_m2_jr: f64,
    beng3_pct: f64,
    beng3_limit_pct: f64,
    energy_label: String,
    heating_primary_kwh: f64,
    hot_water_primary_kwh: f64,
    cooling_primary_kwh: f64,
    fans_primary_kwh: f64,
    pv_production_kwh: f64,
    cooling_demand_kwh: f64,
}

// ---------------------------------------------------------------------------
// Fixture-bestanden (compile-time ingesloten)
// ---------------------------------------------------------------------------

const RVO_TUSSENWONING_EXPECTED: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/tussenwoning-m-g13/expected.json"
);
const RVO_TUSSENWONING_INPUT: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/tussenwoning-m-g13/input.json"
);
const RVO_HOEKWONING_EXPECTED: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/hoekwoning-m-g11/expected.json"
);
const RVO_HOEKWONING_INPUT: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/hoekwoning-m-g11/input.json"
);
const RVO_VRIJSTAAND_EXPECTED: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/vrijstaande-l-g12/expected.json"
);
const RVO_VRIJSTAAND_INPUT: &str = include_str!(
    "../../../tests/verification/beng_rvo_voorbeeldconcepten/vrijstaande-l-g12/input.json"
);

const UNIEC_GOUDA_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_uniec_crosscheck/gouda-2467/expected.json");
const UNIEC_GOUDA_INPUT: &str =
    include_str!("../../../tests/verification/beng_uniec_crosscheck/gouda-2467/input.oes.json");
const UNIEC_AALTEN_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_uniec_crosscheck/aalten-2522/expected.json");
const UNIEC_AALTEN_INPUT: &str =
    include_str!("../../../tests/verification/beng_uniec_crosscheck/aalten-2522/input.oes.json");

const RVO_FIXTURES: &[(&str, &str)] = &[
    ("tussenwoning-m-g13", RVO_TUSSENWONING_EXPECTED),
    ("hoekwoning-m-g11", RVO_HOEKWONING_EXPECTED),
    ("vrijstaande-l-g12", RVO_VRIJSTAAND_EXPECTED),
];

const UNIEC_FIXTURES: &[(&str, &str, &str)] = &[
    ("gouda-2467", UNIEC_GOUDA_EXPECTED, UNIEC_GOUDA_INPUT),
    ("aalten-2522", UNIEC_AALTEN_EXPECTED, UNIEC_AALTEN_INPUT),
];

// ---------------------------------------------------------------------------
// `.oes.json` (open-energy-studio project{}-schema) → `ProjectV2`
// ---------------------------------------------------------------------------
//
// De Uniec-fixtures dragen John Heikens' `project{}`-blok (engine-compleet:
// 1 rekenzone met per-oriëntatie-gevelvlakken + ramen, constructies met U-waarde,
// installaties per dienst). Dit is een ánder schema dan `ProjectV2`; deze
// converter is de deterministische brug. Elke aanname is herleidbaar tot een
// oes-veld; de gedocumenteerde gaten (koudebruggen, gemeten qv10) staan in de
// fixture-README's onder "Bekende engine-gaps".

use openaec_project_shared::beng::{compute_beng, BengResult};
use openaec_project_shared::energy::{
    CoolingGeneratorType, CoolingInput, DhwGeneratorType, DhwInput, EnergyInput, HeatEmissionType,
    HeatGeneratorType, HeatingInput, PvInput, VentilationInput, VentilationSystemType,
};
use openaec_project_shared::geometry::{
    BoundaryKind, Construction, ConstructionKind, Opening, OpeningKind, SharedGeometry, Space,
};
use openaec_project_shared::shared::{BuildingTypeShared, ResidentialType};
use openaec_project_shared::ProjectV2;

/// oes-oriëntatiecode → azimut in graden (DTO-conventie 0 = noord, kloksgewijs).
/// `horizontal`/onbekend → `None` (geen oriëntatiegebonden vlak).
fn oes_orientation_deg(o: &str) -> Option<f64> {
    match o {
        "N" => Some(0.0),
        "NE" => Some(45.0),
        "E" => Some(90.0),
        "SE" => Some(135.0),
        "S" => Some(180.0),
        "SW" => Some(225.0),
        "W" => Some(270.0),
        "NW" => Some(315.0),
        _ => None,
    }
}

/// Bouw een [`ProjectV2`] uit een `.oes.json`-`project{}`-blok.
///
/// **Mapping-keuzes (provenance = oes-veld):**
/// - Geometrie: elk `surface` → één [`Construction`]; `type` wall/roof/floor →
///   [`ConstructionKind`] + [`BoundaryKind`] (wall/roof = `Exterior`, floor =
///   `Ground`). Slope: wall 90°, hellend dak 45°, vloer geen. U-waarde uit het
///   `constructions[]`-blok via `constructionId`. Ramen → [`Opening`] (incl. de
///   als raam-met-`gValue=0` gemodelleerde deuren — behoudt hun U·A, 0 zonwinst).
/// - `Space.floor_area_m2` = `zone.floorArea` (= A_g), níet het vloer-surface
///   (grondcontact). `height_m` = `zone.height` (levert `floorArea·height` =
///   `zone.volume`).
/// - Installaties: heat_pump_air + vloerverwarming; tapwater-WP (`efficiency` =
///   SCOP_W); ventilatie D met `sfp/3,6` (oes W/(dm³/s) → W/(m³/h)) en
///   `wtw_efficiency = Some(η)` (η = 0 in de bron: geen effectieve WTW, maar de
///   `Some`-tak activeert de fan-SFP-doorgifte); compressiekoeling (`eer` = SEER);
///   PV-velden 1-op-1.
///
/// **Niet-injecteerbaar (→ norm-forfait, gedocumenteerde gap):** de gemeten
/// `airTightness.qv10` (geen ProjectV2-veld → infiltratie valt op het
/// tabel-11.13-forfait per [`BuildingLeakageType`]) en de expliciete
/// `thermalBridges` (de nta8800-view propageert `thermal_bridges_linear` niet →
/// H_T zonder koudebrugtoeslag). `subtype` stuurt uitsluitend dat
/// leakage-forfait; het is per case op de werkelijke typologie gezet.
fn oes_to_projectv2(input: &serde_json::Value, subtype: ResidentialType) -> ProjectV2 {
    let project = &input["project"];

    let mut con_u: BTreeMap<String, f64> = BTreeMap::new();
    for c in project["constructions"].as_array().expect("constructions[]") {
        con_u.insert(
            c["id"].as_str().expect("construction.id").to_string(),
            c["uValue"].as_f64().expect("construction.uValue"),
        );
    }

    let zone = &project["zones"][0];
    let floor_area = zone["floorArea"].as_f64().expect("zone.floorArea");
    let height = zone["height"].as_f64().expect("zone.height");

    let mut constructions: Vec<Construction> = Vec::new();
    for s in zone["surfaces"].as_array().expect("surfaces[]") {
        let stype = s["type"].as_str().expect("surface.type");
        let (kind, boundary, slope) = match stype {
            "wall" => (ConstructionKind::Wall, BoundaryKind::Exterior, Some(90.0)),
            "roof" => (ConstructionKind::Roof, BoundaryKind::Exterior, Some(45.0)),
            "floor" => (ConstructionKind::Floor, BoundaryKind::Ground, None),
            other => panic!("onbekend oes surface.type: {other}"),
        };
        let cid = s["constructionId"].as_str().expect("surface.constructionId");
        let u = *con_u.get(cid).unwrap_or_else(|| panic!("geen constructie {cid}"));

        let mut openings: Vec<Opening> = Vec::new();
        if let Some(wins) = s["windows"].as_array() {
            for w in wins {
                openings.push(Opening {
                    id: w["id"].as_str().unwrap_or("win").to_string(),
                    kind: OpeningKind::Window,
                    area_m2: w["area"].as_f64().expect("window.area"),
                    u_value: w["uValue"].as_f64().expect("window.uValue"),
                    g_value: Some(w["gValue"].as_f64().unwrap_or(0.0)),
                    frame_fraction: None,
                    movable_shading: None,
                    obstruction: Default::default(),
                });
            }
        }

        constructions.push(Construction {
            id: s["id"].as_str().unwrap_or("surf").to_string(),
            description: s["name"].as_str().unwrap_or("").to_string(),
            kind,
            boundary,
            area_m2: s["area"].as_f64().expect("surface.area"),
            u_value: u,
            orientation_deg: oes_orientation_deg(s["orientation"].as_str().unwrap_or("horizontal")),
            slope_deg: slope,
            openings,
            layers: vec![],
            adjacent_space_id: None,
            psi_thermal_bridge: None,
        });
    }

    let mut p = ProjectV2::new(project["name"].as_str().unwrap_or("oes-project"));
    p.shared.building_type = BuildingTypeShared::Woning { subtype };
    p.shared.gross_floor_area_m2 = Some(floor_area);
    p.shared.construction_year = Some(2020);
    p.geometry = SharedGeometry {
        spaces: vec![Space {
            id: zone["id"].as_str().unwrap_or("zone").to_string(),
            name: zone["name"].as_str().unwrap_or("zone").to_string(),
            function: None,
            floor_area_m2: floor_area,
            height_m: height,
            theta_i_winter_c: Some(20.0),
            theta_i_summer_c: Some(24.0),
            constructions,
        }],
    };

    let heat = &project["heatingSystems"][0];
    let hw = &project["hotWaterSystems"][0];
    let vent = &project["ventilationSystems"][0];
    let cool = &project["coolingSystems"][0];

    let pv: Vec<PvInput> = project["solarPV"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|v| PvInput {
                    id: v["id"].as_str().map(String::from),
                    name: v["name"].as_str().map(String::from),
                    peak_power_kwp: v["peakPower"].as_f64().expect("pv.peakPower"),
                    azimuth_degrees: oes_orientation_deg(v["orientation"].as_str().unwrap_or("S"))
                        .unwrap_or(180.0),
                    tilt_degrees: v["tilt"].as_f64().unwrap_or(30.0),
                    system_efficiency: None,
                    inverter_efficiency: None,
                    shadow_factor: None,
                })
                .collect()
        })
        .unwrap_or_default();

    p.energy = Some(EnergyInput {
        heating: Some(HeatingInput {
            generator: HeatGeneratorType::HeatPumpAir,
            cop: heat["cop"].as_f64(),
            hr_class: None,
            district_factor: None,
            emission: Some(HeatEmissionType::FloorHeating),
            distribution_efficiency: None,
            control_factor: None,
            coverage_fraction: heat["coverageFraction"].as_f64().unwrap_or(1.0),
        }),
        dhw: Some(DhwInput {
            generator: DhwGeneratorType::HeatPump,
            efficiency: hw["efficiency"].as_f64(),
            dwtw: None,
            has_solar_boiler: false,
            solar_boiler_fraction: None,
        }),
        ventilation: Some(VentilationInput {
            system: VentilationSystemType::D,
            wtw_efficiency: Some(vent["heatRecoveryEfficiency"].as_f64().unwrap_or(0.0)),
            sfp_w_per_m3h: vent["sfp"].as_f64().map(|s| s / 3.6),
            bypass_enabled: false,
            mechanical_supply_m3_per_h: None,
            mechanical_exhaust_m3_per_h: None,
            infiltration_m3_per_h: None,
        }),
        cooling: Some(CoolingInput {
            generator: CoolingGeneratorType::Compression,
            seer: cool["eer"].as_f64(),
            cop: None,
            free_cooling_fraction: None,
        }),
        pv,
        automation: None,
    });

    p
}

/// Subtype per Uniec-case (stuurt uitsluitend het infiltratie-leakage-forfait).
/// Gouda: vrijstaand met kap. Aalten: vrijstaande woning met zadeldak.
fn uniec_subtype(case: &str) -> ResidentialType {
    match case {
        "gouda-2467" | "aalten-2522" => ResidentialType::Detached,
        other => panic!("onbekende uniec-case: {other}"),
    }
}

/// Diagnostische meting — print berekend/expected/delta voor beide Uniec-cases.
/// `cargo test -p openaec-project-shared --test beng_golden uniec_measure -- --ignored --nocapture`.
#[test]
#[ignore = "diagnostiek — draai handmatig met --nocapture"]
fn uniec_measure() {
    for (name, exp_raw, input_raw) in UNIEC_FIXTURES {
        let fx: UniecExpected = serde_json::from_str(exp_raw).unwrap();
        let input: serde_json::Value = serde_json::from_str(input_raw).unwrap();
        let project = oes_to_projectv2(&input, uniec_subtype(name));
        let r: BengResult = compute_beng(&project).expect("compute_beng ok");
        let e = &fx.expected;
        let d = |calc: f64, exp: f64| (calc - exp) / exp * 100.0;
        println!("\n=== {name} (A_g={:.1} A_ls={:.1} vf={:.2}) ===", r.a_g_m2, r.a_ls_m2, r.als_ag_ratio);
        println!("  BENG1  calc={:7.2}  exp={:7.2}  d={:+6.1}%  (lim {:.1})", r.beng1.value, e.beng1_kwh_m2_jr, d(r.beng1.value, e.beng1_kwh_m2_jr), e.beng1_limit_kwh_m2_jr);
        println!("  BENG2  calc={:7.2}  exp={:7.2}  d={:+6.1}%  (lim {:.1})", r.beng2.value, e.beng2_kwh_m2_jr, d(r.beng2.value, e.beng2_kwh_m2_jr), e.beng2_limit_kwh_m2_jr);
        println!("  BENG3  calc={:7.2}  exp={:7.2}  d={:+6.1}pp (lim {:.0})", r.beng3.value, e.beng3_pct, r.beng3.value - e.beng3_pct, e.beng3_limit_pct);
        println!("  label  calc={:>6}  exp={:>6}", r.energy_label, e.energy_label);
        let sb = &r.service_breakdown_kwh_m2;
        println!("  sub/m² heating={:6.2} dhw={:6.2} cooling={:6.2} fans={:6.2} pv={:6.2}", sb.heating, sb.dhw, sb.cooling, sb.ventilation_aux, sb.pv);
        println!("  sub-totaal (primair kWh, ·A_g): heating={:6.0}(exp {}) dhw={:6.0}(exp {}) cooling={:6.0}(exp {}) fans={:6.0}(exp {}) pv={:7.0}(exp {})",
            sb.heating * r.a_g_m2, e.heating_primary_kwh,
            sb.dhw * r.a_g_m2, e.hot_water_primary_kwh,
            sb.cooling * r.a_g_m2, e.cooling_primary_kwh,
            sb.ventilation_aux * r.a_g_m2, e.fans_primary_kwh,
            sb.pv * r.a_g_m2, e.pv_production_kwh);
    }
}

// ---------------------------------------------------------------------------
// Vangnet-test (draait mee in `cargo test`) — provenance-discipline
// ---------------------------------------------------------------------------

/// Bewaakt dat elke golden-expected-waarde herleidbaar is: RVO-waarden hebben een
/// paginanummer + rij-aanduiding, Uniec-waarden een JSON-pad. Plus een anti-fudge-
/// controle op de Uniec-cases: de expected-waarden moeten EXACT gelijk zijn aan
/// `meta.uniecReference` in de bijbehorende `input.oes.json` (bewijs dat niemand een
/// expected met de hand heeft bijgesteld).
#[test]
fn all_expected_fixtures_have_provenance() {
    // --- RVO ---
    for (name, raw) in RVO_FIXTURES {
        let fx: RvoExpected = serde_json::from_str(raw)
            .unwrap_or_else(|e| panic!("{name}: expected.json parse-fout: {e}"));

        assert_eq!(&fx.case, name, "{name}: case-veld matcht de map niet");
        assert!(
            fx.source.document.contains(".pdf"),
            "{name}: source.document verwijst niet naar een PDF"
        );
        assert!(
            fx.source.eisen_page > 0 && fx.source.resultaten_page > 0,
            "{name}: eisen/resultaten paginanummer ontbreekt"
        );
        assert!(
            !fx.tolerance.motivatie.trim().is_empty(),
            "{name}: tolerantie zonder motivatie"
        );
        assert!(
            fx.tolerance.beng1_pct > 0.0
                && fx.tolerance.beng2_pct > 0.0
                && fx.tolerance.beng3_abs_pp > 0.0,
            "{name}: niet-positieve tolerantie"
        );
        assert!(fx.ag_m2 > 0.0 && fx.als_ag_ratio > 0.0, "{name}: geometrie-kentallen ontbreken");
        assert!(
            fx.eisen.beng1_max_kwh_m2_jr > 0.0
                && fx.eisen.beng2_max_kwh_m2_jr > 0.0
                && fx.eisen.beng3_min_pct > 0.0
                && fx.eisen.tojuli_max > 0.0,
            "{name}: eisen incompleet"
        );
        assert_eq!(fx.concepts.len(), 3, "{name}: verwacht 3 concepten");

        for c in &fx.concepts {
            assert!(
                c.provenance.page > 0,
                "{name}/{}: provenance.page ontbreekt",
                c.id
            );
            assert!(
                !c.provenance.row.trim().is_empty(),
                "{name}/{}: provenance.row leeg",
                c.id
            );
            assert!(
                c.expected.beng1_kwh_m2_jr > 0.0
                    && c.expected.beng2_kwh_m2_jr > 0.0
                    && c.expected.beng3_pct > 0.0,
                "{name}/{}: expected BENG-waarde is nul/ontbreekt",
                c.id
            );
            assert!(
                c.expected.tojuli >= 0.0,
                "{name}/{}: negatieve TOjuli",
                c.id
            );
        }
    }

    // --- Uniec ---
    for (name, exp_raw, input_raw) in UNIEC_FIXTURES {
        let fx: UniecExpected = serde_json::from_str(exp_raw)
            .unwrap_or_else(|e| panic!("{name}: expected.json parse-fout: {e}"));

        assert_eq!(&fx.case, name, "{name}: case-veld matcht de map niet");
        assert!(
            !fx.source.repo.is_empty()
                && !fx.source.license.is_empty()
                && !fx.source.certified_tool.is_empty()
                && !fx.source.provenance_jsonpath_root.is_empty(),
            "{name}: source-provenance incompleet"
        );
        assert!(
            !fx.tolerance.motivatie.trim().is_empty(),
            "{name}: tolerantie zonder motivatie"
        );
        assert!(
            fx.tolerance.beng1_pct > 0.0
                && fx.tolerance.beng2_pct > 0.0
                && fx.tolerance.beng3_abs_pp > 0.0,
            "{name}: niet-positieve tolerantie"
        );
        assert!(!fx.provenance.is_empty(), "{name}: lege provenance-map");
        for (key, path) in &fx.provenance {
            assert!(
                !path.trim().is_empty(),
                "{name}: provenance voor '{key}' is leeg"
            );
        }
        // Elke waarde die we asserten moet een provenance-pad hebben.
        for key in [
            "beng1_kwh_m2_jr",
            "beng2_kwh_m2_jr",
            "beng3_pct",
            "energy_label",
        ] {
            assert!(
                fx.provenance.contains_key(key),
                "{name}: geen provenance-pad voor '{key}'"
            );
        }

        // Anti-fudge: expected == meta.uniecReference uit de bron.
        let input: serde_json::Value = serde_json::from_str(input_raw)
            .unwrap_or_else(|e| panic!("{name}: input.oes.json parse-fout: {e}"));
        let uref = &input["meta"]["uniecReference"];
        assert!(
            uref.is_object(),
            "{name}: meta.uniecReference ontbreekt in input.oes.json"
        );
        assert_no_fudge(name, "beng1", uref["beng1"].as_f64(), fx.expected.beng1_kwh_m2_jr);
        assert_no_fudge(name, "beng2", uref["beng2"].as_f64(), fx.expected.beng2_kwh_m2_jr);
        assert_no_fudge(name, "beng3", uref["beng3"].as_f64(), fx.expected.beng3_pct);
        assert_no_fudge(
            name,
            "beng1Limit",
            uref["beng1Limit"].as_f64(),
            fx.expected.beng1_limit_kwh_m2_jr,
        );
        assert_eq!(
            uref["energyLabel"].as_str(),
            Some(fx.expected.energy_label.as_str()),
            "{name}: energyLabel wijkt af van bron (fudge?)"
        );
    }
}

/// Faalt als de expected-waarde niet exact overeenkomt met de bronwaarde uit
/// `meta.uniecReference`. Exacte gelijkheid mag hier: beide zijn dezelfde
/// JSON-literal, letterlijk getranscribeerd.
fn assert_no_fudge(case: &str, field: &str, source: Option<f64>, expected: f64) {
    let source = source.unwrap_or_else(|| panic!("{case}: bronveld '{field}' ontbreekt/geen getal"));
    assert!(
        (source - expected).abs() < 1e-9,
        "{case}: expected.{field}={expected} wijkt af van bron {source} (anti-fudge)"
    );
}

// ---------------------------------------------------------------------------
// RVO-goldens (rood — wachten op compute_beng, F2)
// ---------------------------------------------------------------------------

/// Structuur-check + rode golden voor één RVO-woningtype: parse expected + input,
/// verifieer 3 concepten, dan `unimplemented!` tot `compute_beng` bestaat.
fn rvo_golden_body(name: &str, expected_raw: &str, input_raw: &str) {
    let fx: RvoExpected =
        serde_json::from_str(expected_raw).unwrap_or_else(|e| panic!("{name}: expected: {e}"));
    // input.json is best-effort documentatie; valideer alleen dat het geldige JSON is.
    let _input: serde_json::Value =
        serde_json::from_str(input_raw).unwrap_or_else(|e| panic!("{name}: input: {e}"));
    assert_eq!(fx.concepts.len(), 3);

    // compute_beng (F2) bestaat; de blokkade is de INVOER: de per-gevel-geometrie
    // (gevelvlakken m², ramen per oriëntatie, Als) staat niet in de publieke RVO-
    // PDF's maar in de niet-gepubliceerde Bijlage 4 (Excel). Zodra die er is:
    //   1. Reconstrueer een ProjectV2 per concept (analoog aan oes_to_projectv2).
    //   2. let result = compute_beng(&project);
    //   3. Vergelijk result.beng1/2/3 + tojuli met c.expected binnen fx.tolerance.
    unimplemented!("RVO Bijlage 4-geometrie ontbreekt — invoer-blokkade (case {name})");
}

#[test]
#[ignore = "geometrie-reconstructie geblokkeerd op RVO Bijlage 4 (per-gevel m²/ramen/Als \
            ontbreken in de publieke PDF); input.json is documentatie-only. compute_beng (F2) \
            bestaat — dit is een invoer-provenance-blokkade, geen engine-blokkade. Zie README."]
fn rvo_tussenwoning_m_g13() {
    rvo_golden_body(
        "tussenwoning-m-g13",
        RVO_TUSSENWONING_EXPECTED,
        RVO_TUSSENWONING_INPUT,
    );
}

#[test]
#[ignore = "geometrie-reconstructie geblokkeerd op RVO Bijlage 4 (per-gevel m²/ramen/Als \
            ontbreken in de publieke PDF); input.json is documentatie-only. compute_beng (F2) \
            bestaat — invoer-provenance-blokkade, geen engine-blokkade. Zie README."]
fn rvo_hoekwoning_m_g11() {
    rvo_golden_body(
        "hoekwoning-m-g11",
        RVO_HOEKWONING_EXPECTED,
        RVO_HOEKWONING_INPUT,
    );
}

#[test]
#[ignore = "geometrie-reconstructie geblokkeerd op RVO Bijlage 4 (per-gevel m²/ramen/Als \
            ontbreken in de publieke PDF); input.json is documentatie-only. compute_beng (F2) \
            bestaat — invoer-provenance-blokkade, geen engine-blokkade. Zie README."]
fn rvo_vrijstaande_l_g12() {
    rvo_golden_body(
        "vrijstaande-l-g12",
        RVO_VRIJSTAAND_EXPECTED,
        RVO_VRIJSTAAND_INPUT,
    );
}

// ---------------------------------------------------------------------------
// Uniec-replay-goldens (rood — wachten op compute_beng, F2)
// ---------------------------------------------------------------------------

/// End-to-end golden voor één certified Uniec-project: `.oes.json` →
/// [`oes_to_projectv2`] → [`compute_beng`], daarna BENG 1/2/3 + label binnen de
/// per-case-tolerantie uit `expected.json`. Anti-fudge: de tolerantie is de
/// bron-tolerantie, niet opgerekt naar de huidige engine-afstand.
fn uniec_golden_body(name: &str, expected_raw: &str, input_raw: &str) {
    let fx: UniecExpected =
        serde_json::from_str(expected_raw).unwrap_or_else(|e| panic!("{name}: expected: {e}"));
    let input: serde_json::Value =
        serde_json::from_str(input_raw).unwrap_or_else(|e| panic!("{name}: input: {e}"));
    assert!(input["project"].is_object(), "{name}: project{{}}-blok ontbreekt");
    assert!(fx.expected.beng1_kwh_m2_jr > 0.0);

    let project = oes_to_projectv2(&input, uniec_subtype(name));
    let r = compute_beng(&project).unwrap_or_else(|e| panic!("{name}: compute_beng: {e}"));
    let e = &fx.expected;
    let t = &fx.tolerance;

    let rel_pct = |calc: f64, exp: f64| (calc - exp) / exp * 100.0;
    assert!(
        rel_pct(r.beng1.value, e.beng1_kwh_m2_jr).abs() <= t.beng1_pct,
        "{name}: BENG1 {:.2} wijkt {:+.1}% af (tol ±{:.0}%) van {:.2}",
        r.beng1.value,
        rel_pct(r.beng1.value, e.beng1_kwh_m2_jr),
        t.beng1_pct,
        e.beng1_kwh_m2_jr
    );
    assert!(
        rel_pct(r.beng2.value, e.beng2_kwh_m2_jr).abs() <= t.beng2_pct,
        "{name}: BENG2 {:.2} wijkt {:+.1}% af (tol ±{:.0}%) van {:.2}",
        r.beng2.value,
        rel_pct(r.beng2.value, e.beng2_kwh_m2_jr),
        t.beng2_pct,
        e.beng2_kwh_m2_jr
    );
    assert!(
        (r.beng3.value - e.beng3_pct).abs() <= t.beng3_abs_pp,
        "{name}: BENG3 {:.2} wijkt {:+.1}pp af (tol ±{:.1}pp) van {:.2}",
        r.beng3.value,
        r.beng3.value - e.beng3_pct,
        t.beng3_abs_pp,
        e.beng3_pct
    );
    assert_eq!(
        r.energy_label, e.energy_label,
        "{name}: energielabel {} ≠ certified {}",
        r.energy_label, e.energy_label
    );
}

#[test]
#[ignore = "F3d-3 gap: PV-west valt door de map_pv-azimuthnormalisatie (270°→−90°, cos-clamp) op ~0 \
            → BENG2 +94%, BENG3 −40pp; koeling +517% (F_sh=1,0), verwarming −58% (koudebruggen niet \
            gepropageerd). Buiten ±6/8/3pp. Zie fixture-README §engine-gaps."]
fn uniec_gouda_2467() {
    uniec_golden_body("gouda-2467", UNIEC_GOUDA_EXPECTED, UNIEC_GOUDA_INPUT);
}

#[test]
#[ignore = "F3d-3 gap: PV-noord uit de bron (orientation \"N\") levert ~0 vs certified 3811 kWh \
            (bron-inconsistentie), plus koeling +108% / verwarming −47% → BENG2 +175%, BENG3 −43pp. \
            Buiten ±6/10/3pp. Zie fixture-README §engine-gaps."]
fn uniec_aalten_2522() {
    uniec_golden_body("aalten-2522", UNIEC_AALTEN_EXPECTED, UNIEC_AALTEN_INPUT);
}
