//! EDR-golden-vangrail — fase F3d-5 (rood/`#[ignore]`).
//!
//! Legt de officiële **EDR-attesteringstestset** (ISSO 54 v2.0, "Testen EP-woningen
//! BRL 9501 NTA 8800", CCvD InstallQ 12-05-2022) vast als golden-laag. Volgt het
//! F0-precedent (`beng_golden.rs`): fixtures parsen + provenance bewaken staat
//! groen; de reken-asserts staan `#[ignore]` tot de engine ze kan halen.
//!
//! **Spiegelbeeld van de RVO-laag.** Bij RVO zijn de *uitkomsten* publiek maar de
//! *geometrie-invoer* zit in een niet-publieke Bijlage 4-Excel. Bij EDR is het
//! omgekeerd: de *invoer* staat volledig + normatief in de PDF-tekst, maar de
//! *resultaatgetallen* zitten in een **apart Excel-document ("Bijlage 2", p67) dat
//! niet in ons bezit is**. Er staat geen enkel resultaatgetal in de PDF zelf.
//!
//! Twee gevolgen voor de goldens:
//!
//! 1. **Geblokkeerd op Bijlage 2-Excel:** alle energie-eindwaarden (EP1/EP2/EP3/
//!    Q_H;nd/TOjuli + deelposten uit h5, p53). In `expected.json` staan die met
//!    `value: null` + `blocked_on`. Het provenance-vangnet hieronder bewaakt dat
//!    niemand daar stilzwijgend een berekend getal in zet (anti-fudge).
//! 2. **Nu al assertbaar (niet Excel-geblokkeerd):** `Ag` (96 m²) en `Als`
//!    (247,2 m²) staan expliciet in de EPW001-tekst (p5). Dat maakt een
//!    geometrie-golden mogelijk die losstaat van de nog-kapotte PV/energie-keten —
//!    de eerste fase-2-activatie (zie `geometry_golden_activation_path`).
//!
//! Officiële afkeurtolerantie: **±1,0%** (p67) — attesteringsniveau, veel strakker
//! dan de ±10% RVO-starttolerantie.
//!
//! Volledige analyse: `docs/2026-07-12-f3d5-edr-testset-analyse.md`.

use std::collections::BTreeMap;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// EDR-fixture schema (tests/verification/beng_edr_epw/*/expected.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EdrExpected {
    case: String,
    test: String,
    #[allow(dead_code)]
    omschrijving: String,
    /// `null` voor de referentietest EPW001, gevuld voor de varianten.
    delta_vs_epw001: Option<String>,
    source: EdrSource,
    tolerance: EdrTolerance,
    geometry_expected: EdrGeometry,
    /// h5-uitvoergrootheden. Waarde per grootheid is een los object (met een
    /// `_comment`-sleutel ertussen), daarom `serde_json::Value` + handmatige parse.
    grootheden: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct EdrSource {
    document: String,
    invoer_pages: Vec<u32>,
    results_source: String,
}

#[derive(Debug, Deserialize)]
struct EdrTolerance {
    afkeur_pct: f64,
    motivatie: String,
}

#[derive(Debug, Deserialize)]
struct EdrGeometry {
    ag_m2: GeoVal,
    als_m2: GeoVal,
    als_ag_ratio: GeoVal,
    // EPW001 draagt daarnaast volume/perimeter; varianten niet. Optioneel.
    #[allow(dead_code)]
    volume_m3: Option<GeoVal>,
    #[allow(dead_code)]
    perimeter_bg_vloer_m: Option<GeoVal>,
}

#[derive(Debug, Deserialize)]
struct GeoVal {
    value: f64,
    provenance: String,
}

/// Eén h5-uitvoergrootheid. In fase 1 is `value` altijd `None` en `blocked_on`
/// altijd `Some` (Bijlage 2-Excel ontbreekt).
#[derive(Debug, Deserialize)]
struct Grootheid {
    #[allow(dead_code)]
    symbool: String,
    #[allow(dead_code)]
    eenheid: String,
    value: Option<f64>,
    blocked_on: Option<String>,
    provenance: String,
}

// ---------------------------------------------------------------------------
// Fixture-bestanden (compile-time ingesloten)
// ---------------------------------------------------------------------------

const EPW001_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw001/expected.json");
const EPW001_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw001/input.json");
const EPW002C_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw002c/expected.json");
const EPW002C_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw002c/input.json");
const EPW004D_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw004d/expected.json");
const EPW004D_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw004d/input.json");
const EPW101P_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw101p/expected.json");
const EPW101P_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw101p/input.json");
const EPW203F_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw203f/expected.json");
const EPW203F_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw203f/input.json");
const EPW301A_EXPECTED: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw301a/expected.json");
const EPW301A_INPUT: &str =
    include_str!("../../../tests/verification/beng_edr_epw/epw301a/input.json");

/// (map-naam, expected.json, input.json) per fixture.
const EDR_FIXTURES: &[(&str, &str, &str)] = &[
    ("epw001", EPW001_EXPECTED, EPW001_INPUT),
    ("epw002c", EPW002C_EXPECTED, EPW002C_INPUT),
    ("epw004d", EPW004D_EXPECTED, EPW004D_INPUT),
    ("epw101p", EPW101P_EXPECTED, EPW101P_INPUT),
    ("epw203f", EPW203F_EXPECTED, EPW203F_INPUT),
    ("epw301a", EPW301A_EXPECTED, EPW301A_INPUT),
];

// ---------------------------------------------------------------------------
// Vangnet-test (draait mee in `cargo test`) — provenance + anti-fudge
// ---------------------------------------------------------------------------

/// Bewaakt de EDR-fixtures: elke fixture heeft bron (document + paginanrs +
/// results-source), een positieve tolerantie met motivatie, geometrie-expected
/// (Ag/Als > 0 met provenance) én — de kern van de anti-fudge — dat *elke*
/// h5-grootheid in fase 1 `value: null` + een `blocked_on`-marker heeft. Zo kan
/// niemand stilzwijgend een berekende (of verzonnen) uitkomst als "expected"
/// binnensmokkelen zolang het Bijlage 2-Excel ontbreekt.
#[test]
fn all_edr_fixtures_have_provenance_and_no_fudge() {
    for (name, exp_raw, input_raw) in EDR_FIXTURES {
        let fx: EdrExpected = serde_json::from_str(exp_raw)
            .unwrap_or_else(|e| panic!("{name}: expected.json parse-fout: {e}"));

        assert_eq!(&fx.case, name, "{name}: case-veld matcht de map niet");
        assert!(
            fx.test.starts_with("EP-W"),
            "{name}: test-code '{}' is geen EP-W…",
            fx.test
        );

        // Referentie heeft geen delta; varianten wél.
        if *name == "epw001" {
            assert!(fx.delta_vs_epw001.is_none(), "epw001 mag geen delta hebben");
        } else {
            assert!(
                fx.delta_vs_epw001.as_deref().map_or(false, |d| !d.trim().is_empty()),
                "{name}: variant zonder delta_vs_epw001"
            );
        }

        // Bron-provenance.
        assert!(
            fx.source.document.contains("ISSO 54"),
            "{name}: source.document verwijst niet naar ISSO 54"
        );
        assert!(
            !fx.source.invoer_pages.is_empty() && fx.source.invoer_pages.iter().all(|p| *p > 0),
            "{name}: invoer_pages ontbreekt/bevat 0"
        );
        assert!(
            fx.source.results_source.to_lowercase().contains("bijlage 2"),
            "{name}: results_source noemt Bijlage 2 niet (de Excel-blokkade)"
        );

        // Tolerantie = officiële ±1,0%.
        assert!(
            (fx.tolerance.afkeur_pct - 1.0).abs() < 1e-9,
            "{name}: afkeur_pct is {} i.p.v. de officiële 1,0%",
            fx.tolerance.afkeur_pct
        );
        assert!(
            !fx.tolerance.motivatie.trim().is_empty(),
            "{name}: tolerantie zonder motivatie"
        );

        // Geometrie-expected (nu assertbaar; niet Excel-geblokkeerd).
        for (label, gv) in [
            ("ag_m2", &fx.geometry_expected.ag_m2),
            ("als_m2", &fx.geometry_expected.als_m2),
            ("als_ag_ratio", &fx.geometry_expected.als_ag_ratio),
        ] {
            assert!(gv.value > 0.0, "{name}: geometry_expected.{label} <= 0");
            assert!(
                !gv.provenance.trim().is_empty(),
                "{name}: geometry_expected.{label} zonder provenance"
            );
        }
        // Interne consistentie: als_ag_ratio == als/ag (±0,5%).
        let ratio = fx.geometry_expected.als_m2.value / fx.geometry_expected.ag_m2.value;
        let stated = fx.geometry_expected.als_ag_ratio.value;
        assert!(
            (ratio - stated).abs() / stated < 0.005,
            "{name}: als_ag_ratio {stated} wijkt af van als/ag {ratio:.4}"
        );

        // h5-grootheden: allemaal geblokkeerd, met provenance (anti-fudge).
        assert!(!fx.grootheden.is_empty(), "{name}: lege grootheden-map");
        let mut real_keys = 0usize;
        for (key, raw) in &fx.grootheden {
            if key.starts_with('_') {
                continue; // _comment e.d.
            }
            real_keys += 1;
            let g: Grootheid = serde_json::from_value(raw.clone())
                .unwrap_or_else(|e| panic!("{name}: grootheid '{key}' parse-fout: {e}"));
            assert!(
                g.value.is_none(),
                "{name}: grootheid '{key}' heeft in fase 1 een value ({:?}) — Bijlage 2 ontbreekt, dit ruikt naar fudge",
                g.value
            );
            assert!(
                g.blocked_on.as_deref().map_or(false, |b| b.to_lowercase().contains("bijlage 2")),
                "{name}: grootheid '{key}' mist een 'blocked_on: Bijlage 2'-marker"
            );
            assert!(
                !g.provenance.trim().is_empty(),
                "{name}: grootheid '{key}' zonder provenance"
            );
        }
        assert!(real_keys > 0, "{name}: geen enkele echte h5-grootheid");

        // input.json is best-effort documentatie; valideer alleen geldige JSON.
        let _input: serde_json::Value = serde_json::from_str(input_raw)
            .unwrap_or_else(|e| panic!("{name}: input.json parse-fout: {e}"));
    }
}

// ---------------------------------------------------------------------------
// Rode goldens (`#[ignore]`) — wachten op Bijlage 2-Excel + F3d-4
// ---------------------------------------------------------------------------

/// Structuur-check + rode golden voor één EDR-fixture. Parse expected + input,
/// dan `unimplemented!` met de dubbele blokkade. Zodra Bijlage 2 er is én de
/// F3d-4-keten gemerged is:
///   1. Bouw een `ProjectV2` uit `input.json` (analoog aan `oes_to_projectv2`;
///      voor de varianten = EPW001-basis + de `delta`-override).
///   2. `let r = compute_beng(&project)?;`
///   3. Vergelijk r.beng1/2/3 + TOjuli + deelposten met `grootheden` (dan gevuld)
///      binnen ±1,0% (`tolerance.afkeur_pct`).
fn edr_golden_body(name: &str, expected_raw: &str, input_raw: &str) {
    let fx: EdrExpected =
        serde_json::from_str(expected_raw).unwrap_or_else(|e| panic!("{name}: expected: {e}"));
    let _input: serde_json::Value =
        serde_json::from_str(input_raw).unwrap_or_else(|e| panic!("{name}: input: {e}"));
    assert!(!fx.grootheden.is_empty());
    unimplemented!(
        "EDR-eindwaarden ontbreken (Bijlage 2-Excel niet in bezit) én F3d-4 PV/energie-keten \
         nog niet gemerged — case {name}. Zie fixture-README + docs/2026-07-12-f3d5-edr-testset-analyse.md"
    );
}

macro_rules! edr_golden {
    ($fn:ident, $name:literal, $exp:ident, $inp:ident) => {
        #[test]
        #[ignore = "F3d-5 fase 2: activatie na (a) verwerven Bijlage 2-Excel met de \
                    EDR-eindwaarden én (b) PV-fix (F3d-4). Fixture-invoer + provenance zijn \
                    wél actief in all_edr_fixtures_have_provenance_and_no_fudge. Zie README."]
        fn $fn() {
            edr_golden_body($name, $exp, $inp);
        }
    };
}

edr_golden!(edr_epw001, "epw001", EPW001_EXPECTED, EPW001_INPUT);
edr_golden!(edr_epw002c, "epw002c", EPW002C_EXPECTED, EPW002C_INPUT);
edr_golden!(edr_epw004d, "epw004d", EPW004D_EXPECTED, EPW004D_INPUT);
edr_golden!(edr_epw101p, "epw101p", EPW101P_EXPECTED, EPW101P_INPUT);
edr_golden!(edr_epw203f, "epw203f", EPW203F_EXPECTED, EPW203F_INPUT);
edr_golden!(edr_epw301a, "epw301a", EPW301A_EXPECTED, EPW301A_INPUT);

/// Geometrie-golden (Ag/Als) — de énige EDR-assert die *niet* Excel-geblokkeerd is
/// (waarden staan in de EPW001-tekst, p5). Eerste fase-2-activatie: zodra een
/// `edr_to_projectv2`-builder bestaat, assert:
///   `compute_beng(&project).a_g_m2 ≈ 96` en `.a_ls_m2 ≈ 247,2` binnen ±1,0%.
/// Valideert de geometrie-pijplijn los van de energie/PV-keten. Staat nu `#[ignore]`
/// omdat de builder nog niet bestaat — niet omdat de referentiewaarde ontbreekt.
#[test]
#[ignore = "F3d-5 fase 2 (eerste activatie): vereist een edr_to_projectv2-builder. \
            NIET Excel-geblokkeerd — Ag/Als staan in de EPW001-tekst (p5)."]
fn geometry_golden_activation_path() {
    let fx: EdrExpected = serde_json::from_str(EPW001_EXPECTED).unwrap();
    // Referentiewaarden die de geometrie-pijplijn moet reproduceren (±1%):
    assert!((fx.geometry_expected.ag_m2.value - 96.0).abs() < 1e-9);
    assert!((fx.geometry_expected.als_m2.value - 247.2).abs() < 1e-9);
    unimplemented!(
        "edr_to_projectv2(EPW001) ontbreekt nog. Bouw hem (analoog aan oes_to_projectv2 in \
         beng_golden.rs) en assert compute_beng(&p).a_g_m2/a_ls_m2 tegen bovenstaande."
    );
}
