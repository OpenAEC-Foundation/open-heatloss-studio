//! Integratie-tests voor `bijlage_aa` module — formules AA.1 t/m AA.13.
//!
//! Deze tests valideren:
//! 1. Sanity: alle-nullen invoer → output nul
//! 2. End-to-end woning met 1 ruimte
//! 3. Tabel AA.3 lookups + lineaire β-interpolatie
//! 4. f_iso bouwjaar mapping (tabel AA.2)
//! 5. Golden-master cross-validatie placeholder (#[ignore])

use approx::assert_abs_diff_eq;

use nta8800_cooling::bijlage_aa::{
    calculate_bijlage_aa, i_sol, theta_e, BijlageAaInput, BouwjaarKlasseAa, Orientatie, RaamAa,
    RuimteAa, ZonweringType, FIXED_DEDUCTION_W_PER_M2,
};

// ---------------------------------------------------------------------------
// Sanity tests
// ---------------------------------------------------------------------------

#[test]
fn sanity_alle_nullen_geeft_nul_capaciteit() {
    // 1 woning, 1 bewoner (minimum positief), 1 ruimte 1 m² met 1 raam van 0 m².
    // Geen ventilatie, bouwjaar 2025 (laagste f_iso). Verwacht: q_C < 35 → 0 kW.
    let input = BijlageAaInput {
        aantal_woonfuncties: 1,
        bewoners_per_woonfunctie: 1.0,
        bouwjaar: 2025,
        infiltratie_m3_per_h: 0.0,
        natuurlijke_ventilatie_m3_per_h: 0.0,
        mechanische_ventilatie_m3_per_h: 0.0,
        ruimten: vec![RuimteAa {
            naam: "leeg".to_string(),
            is_woonvertrek: false,
            oppervlakte_m2: 100.0,
            opaque_oppervlakte_m2: 0.0,
            ramen: vec![],
        }],
    };
    let r = calculate_bijlage_aa(&input).unwrap();
    assert_abs_diff_eq!(r.p_sol_zone_w, 0.0, epsilon = 1e-9);
    assert_abs_diff_eq!(r.p_tr_ntr_zone_w, 0.0, epsilon = 1e-9);
    // P_gl ~ 0 (geen ramen) en P_V ~ 0 (geen flow), maar P_int > 0
    // (1 bewoner × 180 W = 180 W). Over 100 m² → 1.8 W/m² << 35 → 0 kW capaciteit.
    assert_abs_diff_eq!(r.b_c_req_zone_kw, 0.0, epsilon = 1e-9);
}

#[test]
fn sanity_geen_ruimten_is_error() {
    let input = BijlageAaInput {
        aantal_woonfuncties: 1,
        bewoners_per_woonfunctie: 2.5,
        bouwjaar: 2020,
        infiltratie_m3_per_h: 100.0,
        natuurlijke_ventilatie_m3_per_h: 0.0,
        mechanische_ventilatie_m3_per_h: 100.0,
        ruimten: vec![],
    };
    assert!(calculate_bijlage_aa(&input).is_err());
}

// ---------------------------------------------------------------------------
// End-to-end 1-room woning
// ---------------------------------------------------------------------------

#[test]
fn e2e_1_room_woning_zuid_raam() {
    // 1-room woning: 20 m² woonkamer, 1 zuid-raam 3 m², bouwjaar 2020,
    // lichte bouwwijze (impliciet — SWM speelt geen rol in bijlage AA),
    // geen overstek/zonwering, U = 1.0, g = 0.5.
    let input = BijlageAaInput {
        aantal_woonfuncties: 1,
        bewoners_per_woonfunctie: 2.0,
        bouwjaar: 2020,
        infiltratie_m3_per_h: 25.0,
        natuurlijke_ventilatie_m3_per_h: 0.0,
        mechanische_ventilatie_m3_per_h: 50.0,
        ruimten: vec![RuimteAa {
            naam: "woonkamer".to_string(),
            is_woonvertrek: true,
            oppervlakte_m2: 20.0,
            opaque_oppervlakte_m2: 30.0, // 30 m² ondoorzichtige schil
            ramen: vec![RaamAa {
                oppervlakte_m2: 3.0,
                g_waarde: 0.5,
                u_waarde_w_per_m2k: 1.0,
                f_sh: 1.0,
                zonwering: ZonweringType::Geen,
                helling_beta_deg: 90.0, // verticale gevel
                orientatie: Orientatie::Zuid,
            }],
        }],
    };
    let r = calculate_bijlage_aa(&input).unwrap();

    // Verifieer plausibele bereiken
    assert!(
        r.q_c_zone_w_per_m2 > 5.0,
        "q_C te laag: {} W/m²",
        r.q_c_zone_w_per_m2
    );
    assert!(
        r.q_c_zone_w_per_m2 < 200.0,
        "q_C onrealistisch hoog: {} W/m²",
        r.q_c_zone_w_per_m2
    );
    assert!(r.b_c_req_zone_kw >= 0.0);
    assert!(r.p_sol_zone_w > 0.0, "Zuid-raam moet zoninstraling geven");

    // Maatgevend tijdstip voor verticale zuidgevel ligt rond 12-13 h
    assert!(
        (12..=14).contains(&r.maatgevend_tijdstip_uur),
        "verwacht maatgevend tijdstip 12-14h voor zuidgevel, kreeg {}h",
        r.maatgevend_tijdstip_uur
    );

    // Eén ruimte = zelfde resultaat als zone-niveau (op rounding na)
    assert_eq!(r.ruimten.len(), 1);
    assert_abs_diff_eq!(
        r.ruimten[0].q_c_w_per_m2,
        r.q_c_zone_w_per_m2,
        epsilon = 1e-6
    );
}

#[test]
fn e2e_typische_eengezinswoning_kwh_bereik() {
    // 120 m² eengezinswoning: 80 m² woon (1 grote west-raam), 40 m² overig
    // (2 slaapkamers met noord-raam). Bouwjaar 2020. Geen zonwering.
    let input = BijlageAaInput {
        aantal_woonfuncties: 1,
        bewoners_per_woonfunctie: 3.0,
        bouwjaar: 2020,
        infiltratie_m3_per_h: 50.0,
        natuurlijke_ventilatie_m3_per_h: 0.0,
        mechanische_ventilatie_m3_per_h: 200.0,
        ruimten: vec![
            RuimteAa {
                naam: "woonkamer".to_string(),
                is_woonvertrek: true,
                oppervlakte_m2: 80.0,
                opaque_oppervlakte_m2: 60.0,
                ramen: vec![RaamAa {
                    oppervlakte_m2: 10.0,
                    g_waarde: 0.6,
                    u_waarde_w_per_m2k: 1.2,
                    f_sh: 1.0,
                    zonwering: ZonweringType::Geen,
                    helling_beta_deg: 90.0,
                    orientatie: Orientatie::West,
                }],
            },
            RuimteAa {
                naam: "slaapkamer".to_string(),
                is_woonvertrek: false,
                oppervlakte_m2: 40.0,
                opaque_oppervlakte_m2: 40.0,
                ramen: vec![RaamAa {
                    oppervlakte_m2: 2.5,
                    g_waarde: 0.6,
                    u_waarde_w_per_m2k: 1.2,
                    f_sh: 1.0,
                    zonwering: ZonweringType::Geen,
                    helling_beta_deg: 90.0,
                    orientatie: Orientatie::Noord,
                }],
            },
        ],
    };
    let r = calculate_bijlage_aa(&input).unwrap();

    // Plausibele bereiken voor q_C en B_C
    assert!(
        (5.0..=200.0).contains(&r.q_c_zone_w_per_m2),
        "q_C buiten plausibel bereik: {} W/m²",
        r.q_c_zone_w_per_m2
    );

    // West-raam → maatgevend tijdstip moet in middag/avond zitten (14-17h)
    assert!(
        (13..=17).contains(&r.maatgevend_tijdstip_uur),
        "verwacht middag/avond voor West-raam: {}h",
        r.maatgevend_tijdstip_uur
    );

    assert_eq!(r.ruimten.len(), 2);
    // Woonkamer moet 2× hogere interne warmtelast per m² hebben dan slaapkamer
    let q_int_woon = r.ruimten[0].p_int_w / 80.0;
    let q_int_slaap = r.ruimten[1].p_int_w / 40.0;
    assert_abs_diff_eq!(q_int_woon / q_int_slaap, 2.0, epsilon = 1e-9);
}

// ---------------------------------------------------------------------------
// Tabel AA.3 lookup tests
// ---------------------------------------------------------------------------

#[test]
fn tabel_aa3_beta30_zuid_12h_xlsm_referentie() {
    // Xlsm "Tabel AA": β=30°, kolom Z (γ=180°), tijdstip 12 h → 1078.073647
    let v = i_sol(30.0, Orientatie::Zuid, 12).unwrap();
    assert_abs_diff_eq!(v, 1078.073647, epsilon = 1e-3);
}

#[test]
fn tabel_aa3_beta60_west_15h_xlsm_referentie() {
    // β=60°, γ=270° (W), 15 h → 1083.243155
    let v = i_sol(60.0, Orientatie::West, 15).unwrap();
    assert_abs_diff_eq!(v, 1083.243155, epsilon = 1e-3);
}

#[test]
fn tabel_aa3_beta90_noord_minimum() {
    // β=90° (gevel), γ=0° (N), 12 h → 141.665748
    let v = i_sol(90.0, Orientatie::Noord, 12).unwrap();
    assert_abs_diff_eq!(v, 141.665748, epsilon = 1e-3);
}

#[test]
fn tabel_aa3_beta0_horizontaal_uniform() {
    // β=0° (plat dak) is uniform over alle oriëntaties.
    let v1 = i_sol(0.0, Orientatie::Zuid, 12).unwrap();
    let v2 = i_sol(0.0, Orientatie::Noord, 12).unwrap();
    let v3 = i_sol(0.0, Orientatie::Horizontaal, 12).unwrap();
    assert_abs_diff_eq!(v1, v2, epsilon = 1e-9);
    assert_abs_diff_eq!(v2, v3, epsilon = 1e-9);
    // En 914.118562 volgens xlsm I_sol;mi kolom om 12 h
    assert_abs_diff_eq!(v1, 914.118562, epsilon = 1e-3);
}

#[test]
fn interpolatie_beta375_lineair() {
    // β = 37.5° is precies tussen 30° en 45°
    let lo = i_sol(30.0, Orientatie::Zuid, 13).unwrap();
    let hi = i_sol(45.0, Orientatie::Zuid, 13).unwrap();
    let mid = i_sol(37.5, Orientatie::Zuid, 13).unwrap();
    assert_abs_diff_eq!(mid, (lo + hi) / 2.0, epsilon = 1e-9);
}

#[test]
fn interpolatie_beta35_kwart_tussen_30_en_45() {
    // β=35° ligt 1/3 tussen 30° en 45° (30+1/3·15=35)
    let lo = i_sol(30.0, Orientatie::Oost, 10).unwrap();
    let hi = i_sol(45.0, Orientatie::Oost, 10).unwrap();
    let q = i_sol(35.0, Orientatie::Oost, 10).unwrap();
    let expected = lo + (1.0 / 3.0) * (hi - lo);
    assert_abs_diff_eq!(q, expected, epsilon = 1e-6);
}

// ---------------------------------------------------------------------------
// f_iso bouwjaar mapping (tabel AA.2)
// ---------------------------------------------------------------------------

#[test]
fn f_iso_bouwjaar_mapping_uit_norm() {
    assert_abs_diff_eq!(
        BouwjaarKlasseAa::from_year(1960).f_iso(),
        17.0,
        epsilon = 1e-9
    );
    assert_abs_diff_eq!(
        BouwjaarKlasseAa::from_year(1980).f_iso(),
        10.0,
        epsilon = 1e-9
    );
    assert_abs_diff_eq!(
        BouwjaarKlasseAa::from_year(2010).f_iso(),
        3.2,
        epsilon = 1e-9
    );
    assert_abs_diff_eq!(
        BouwjaarKlasseAa::from_year(2025).f_iso(),
        2.2,
        epsilon = 1e-9
    );
}

#[test]
fn f_iso_grens_2015_inclusief() {
    // > 2015 in tabel AA.2 (norm-tekst is "> 2015") → bouwjaar 2015 valt
    // strikt genomen niet in "> 2015", maar onze impl is `>= 2015` om
    // praktische redenen (rekentool-rondegedrag).
    assert_eq!(BouwjaarKlasseAa::from_year(2015), BouwjaarKlasseAa::Van2015);
    assert_eq!(
        BouwjaarKlasseAa::from_year(2014),
        BouwjaarKlasseAa::Van1992Tot2015
    );
}

// ---------------------------------------------------------------------------
// Tabel AA.1 — θ_e
// ---------------------------------------------------------------------------

#[test]
fn tabel_aa1_piek_temperatuur() {
    assert_eq!(theta_e(17), Some(30.6));
    assert_eq!(theta_e(9), Some(24.7));
    assert_eq!(theta_e(21), Some(23.4));
}

// ---------------------------------------------------------------------------
// Fixed deduction constant
// ---------------------------------------------------------------------------

#[test]
fn fixed_deduction_is_35() {
    assert_abs_diff_eq!(FIXED_DEDUCTION_W_PER_M2, 35.0, epsilon = 1e-9);
}

// ---------------------------------------------------------------------------
// Zonwering F_c lookup
// ---------------------------------------------------------------------------

#[test]
fn zonwering_screen_donker_f_c_0_12() {
    assert_abs_diff_eq!(ZonweringType::ScreenDonker.f_c(), 0.12, epsilon = 1e-9);
}

#[test]
fn zonwering_geen_f_c_1_0() {
    assert_abs_diff_eq!(ZonweringType::Geen.f_c(), 1.0, epsilon = 1e-9);
}

#[test]
fn zonwering_effect_op_p_sol() {
    // Met zonwering moet P_sol significant lager zijn dan zonder
    let basis = RaamAa {
        oppervlakte_m2: 10.0,
        g_waarde: 0.6,
        u_waarde_w_per_m2k: 1.2,
        f_sh: 1.0,
        zonwering: ZonweringType::Geen,
        helling_beta_deg: 90.0,
        orientatie: Orientatie::Zuid,
    };
    let met_zonwering = RaamAa {
        zonwering: ZonweringType::ScreenDonker,
        ..basis
    };

    let mk_input = |raam: RaamAa| BijlageAaInput {
        aantal_woonfuncties: 1,
        bewoners_per_woonfunctie: 2.0,
        bouwjaar: 2020,
        infiltratie_m3_per_h: 50.0,
        natuurlijke_ventilatie_m3_per_h: 0.0,
        mechanische_ventilatie_m3_per_h: 100.0,
        ruimten: vec![RuimteAa {
            naam: "test".to_string(),
            is_woonvertrek: true,
            oppervlakte_m2: 50.0,
            opaque_oppervlakte_m2: 40.0,
            ramen: vec![raam],
        }],
    };
    let zonder = calculate_bijlage_aa(&mk_input(basis)).unwrap();
    let met = calculate_bijlage_aa(&mk_input(met_zonwering)).unwrap();

    assert!(
        met.p_sol_zone_w < zonder.p_sol_zone_w * 0.5,
        "screen-donker (F_c=0.12) moet P_sol minstens halveren: {} → {}",
        zonder.p_sol_zone_w,
        met.p_sol_zone_w
    );
}

// ---------------------------------------------------------------------------
// Golden-master cross-validatie placeholder
// ---------------------------------------------------------------------------

/// Placeholder voor cross-validatie tegen RVO-rekentool xlsm.
///
/// Wordt geïgnoreerd tot een specifieke testcase met geverifieerde xlsm-output
/// is opgesteld. Vereist:
/// 1. Een ingevulde xlsm met bekende invoer per ruimte
/// 2. De resultaten uit "Projectgegevens en Resultaten" (q_C, B_C;req per zone)
/// 3. Tolerantie ~1% (rekening houdend met xlsm-rounding van I_sol)
///
/// Activeer met `cargo test -p nta8800-cooling -- --ignored`.
#[test]
#[ignore = "vereist ingevulde RVO-rekentool xlsm met bekende output"]
fn golden_master_xlsm_cross_validatie() {
    // TODO(volgende sessie): vul concrete waarden in uit rekentool
    panic!("placeholder — zie doc-comment");
}
