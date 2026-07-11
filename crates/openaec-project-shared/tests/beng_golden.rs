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

    // Zodra F2 klaar is:
    //   1. Reconstrueer een ProjectV2 per concept (wacht op Bijlage 4-geometrie).
    //   2. let result = compute_beng(&project);
    //   3. Vergelijk result.beng1/2/3 + tojuli met c.expected binnen fx.tolerance.
    unimplemented!("compute_beng ontbreekt nog — F2 (case {name})");
}

#[test]
#[ignore = "wacht op compute_beng (F2)"]
fn rvo_tussenwoning_m_g13() {
    rvo_golden_body(
        "tussenwoning-m-g13",
        RVO_TUSSENWONING_EXPECTED,
        RVO_TUSSENWONING_INPUT,
    );
}

#[test]
#[ignore = "wacht op compute_beng (F2)"]
fn rvo_hoekwoning_m_g11() {
    rvo_golden_body(
        "hoekwoning-m-g11",
        RVO_HOEKWONING_EXPECTED,
        RVO_HOEKWONING_INPUT,
    );
}

#[test]
#[ignore = "wacht op compute_beng (F2)"]
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

/// Structuur-check + rode golden voor één certified Uniec-project.
fn uniec_golden_body(name: &str, expected_raw: &str, input_raw: &str) {
    let fx: UniecExpected =
        serde_json::from_str(expected_raw).unwrap_or_else(|e| panic!("{name}: expected: {e}"));
    let input: serde_json::Value =
        serde_json::from_str(input_raw).unwrap_or_else(|e| panic!("{name}: input: {e}"));
    assert!(input["project"].is_object(), "{name}: project{{}}-blok ontbreekt");
    assert!(fx.expected.beng1_kwh_m2_jr > 0.0);

    // Zodra F2 klaar is:
    //   1. let project = ProjectV2::from(&input["project"]);  // .oes.json → ProjectV2
    //   2. let result = compute_beng(&project);
    //   3. Vergelijk result.beng1/2/3 + sub-totalen met fx.expected binnen fx.tolerance.
    unimplemented!("compute_beng ontbreekt nog — F2 (case {name})");
}

#[test]
#[ignore = "wacht op compute_beng (F2)"]
fn uniec_gouda_2467() {
    uniec_golden_body("gouda-2467", UNIEC_GOUDA_EXPECTED, UNIEC_GOUDA_INPUT);
}

#[test]
#[ignore = "wacht op compute_beng (F2)"]
fn uniec_aalten_2522() {
    uniec_golden_body("aalten-2522", UNIEC_AALTEN_EXPECTED, UNIEC_AALTEN_INPUT);
}
