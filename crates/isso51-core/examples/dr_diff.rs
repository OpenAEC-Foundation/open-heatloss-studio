//! Diagnostic diff: per-component actual vs expected voor de
//! `dr_engineering_woningbouw` fixture.
//!
//! Toont per-kamer breakdown (phi_t_ie / phi_t_ia / phi_t_iae / phi_t_ib /
//! phi_t_ig / phi_i / phi_v / phi_vent / phi_hu / phi_hl_i) actual vs Vabi
//! expected + gebouw-aggregatie. Onmisbaar bij diagnose van afwijkingen
//! tussen onze engine en Vabi Elements voor woningbouw-fixtures.
//!
//! Gebruik: `cargo run --example dr_diff -p isso51-core`
//!
//! Historie: ontwikkeld 2026-05-13 voor Bug Y + AggregationMethod diagnose.

use isso51_core::calculate_from_json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = std::path::Path::new(manifest).parent().unwrap().parent().unwrap();
    let fix = root.join("tests/fixtures/dr_engineering_woningbouw.json");
    let exp = root.join("tests/fixtures/dr_engineering_woningbouw_result.json");

    let input = fs::read_to_string(&fix)?;
    let expected: serde_json::Value = serde_json::from_str(&fs::read_to_string(&exp)?)?;

    let project: serde_json::Value = serde_json::from_str(&input)?;
    let theta_e = project["climate"]["theta_e"].as_f64().unwrap_or(-10.0);

    let out_json = calculate_from_json(&input)?;
    let actual: serde_json::Value = serde_json::from_str(&out_json)?;

    println!("=== theta_e = {} ===\n", theta_e);

    // Per-room comparison
    println!("{:<6} {:<22} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "id", "name", "comp", "exp", "act", "Δ", "comp", "exp", "act", "Δ");
    println!("{}", "-".repeat(120));

    let expected_rooms = expected["rooms"].as_array().unwrap();
    let actual_rooms = actual["rooms"].as_array().unwrap();

    // Sums for building-level reconciliation
    let mut sum_act = std::collections::HashMap::<&str, f64>::new();
    let mut sum_exp = std::collections::HashMap::<&str, f64>::new();
    for k in ["phi_t_ie","phi_t_ia","phi_t_iae","phi_t_ib","phi_t_ig","phi_t_iw","phi_i","phi_vent","phi_v","phi_hu","phi_hl_i"] {
        sum_act.insert(k, 0.0);
        sum_exp.insert(k, 0.0);
    }

    for er in expected_rooms {
        let id = er["room_id"].as_str().unwrap();
        let name = er["room_name"].as_str().unwrap();
        let ar = actual_rooms.iter().find(|r| r["room_id"].as_str() == Some(id));
        if ar.is_none() {
            println!("{:<6} {:<22} MISSING IN ACTUAL", id, name);
            continue;
        }
        let ar = ar.unwrap();
        let theta_i = er["theta_i"].as_f64().unwrap_or(20.0);
        let dt = theta_i - theta_e;

        // Actual phi_t breakdown from h_t × Δθ
        let h_ie = ar["transmission"]["h_t_exterior"].as_f64().unwrap_or(0.0);
        let h_ia = ar["transmission"]["h_t_adjacent_rooms"].as_f64().unwrap_or(0.0);
        let h_io = ar["transmission"]["h_t_unheated"].as_f64().unwrap_or(0.0);
        let h_ib = ar["transmission"]["h_t_adjacent_buildings"].as_f64().unwrap_or(0.0);
        let h_ig = ar["transmission"]["h_t_ground"].as_f64().unwrap_or(0.0);
        let h_iw = ar["transmission"]["h_t_water"].as_f64().unwrap_or(0.0);

        let a_phi_t_ie  = h_ie * dt;
        let a_phi_t_ia  = h_ia * dt;
        let a_phi_t_iae = h_io * dt;
        let a_phi_t_ib  = h_ib * dt;
        let a_phi_t_ig  = h_ig * dt;
        let a_phi_t_iw  = h_iw * dt;

        let a_phi_i    = ar["infiltration"]["phi_i"].as_f64().unwrap_or(0.0);
        let a_phi_v    = ar["ventilation"]["phi_v"].as_f64().unwrap_or(0.0);
        let a_phi_vent = ar["ventilation"]["phi_vent"].as_f64().unwrap_or(0.0);
        let a_phi_hu   = ar["heating_up"]["phi_hu"].as_f64().unwrap_or(0.0);
        let a_phi_hl   = ar["total_heat_loss"].as_f64().unwrap_or(0.0);

        let e_phi_t_ie  = er["phi_t_ie"].as_f64().unwrap_or(0.0);
        let e_phi_t_ia  = er["phi_t_ia"].as_f64().unwrap_or(0.0);
        let e_phi_t_iae = er["phi_t_iae"].as_f64().unwrap_or(0.0);
        let e_phi_t_ib  = er["phi_t_iaBE"].as_f64().unwrap_or(0.0);
        let e_phi_t_ig  = er["phi_t_ig"].as_f64().unwrap_or(0.0);
        let e_phi_i     = er["phi_i"].as_f64().unwrap_or(0.0);
        let e_phi_vent  = er["phi_vent"].as_f64().unwrap_or(0.0);
        let e_phi_hu    = er["phi_hu"].as_f64().unwrap_or(0.0);
        let e_phi_hl    = er["phi_hl_i"].as_f64().unwrap_or(0.0);

        // Update sums
        *sum_act.get_mut("phi_t_ie").unwrap()  += a_phi_t_ie;
        *sum_act.get_mut("phi_t_ia").unwrap()  += a_phi_t_ia;
        *sum_act.get_mut("phi_t_iae").unwrap() += a_phi_t_iae;
        *sum_act.get_mut("phi_t_ib").unwrap()  += a_phi_t_ib;
        *sum_act.get_mut("phi_t_ig").unwrap()  += a_phi_t_ig;
        *sum_act.get_mut("phi_t_iw").unwrap()  += a_phi_t_iw;
        *sum_act.get_mut("phi_i").unwrap()     += a_phi_i;
        *sum_act.get_mut("phi_v").unwrap()     += a_phi_v;
        *sum_act.get_mut("phi_vent").unwrap()  += a_phi_vent;
        *sum_act.get_mut("phi_hu").unwrap()    += a_phi_hu;
        *sum_act.get_mut("phi_hl_i").unwrap()  += a_phi_hl;

        *sum_exp.get_mut("phi_t_ie").unwrap()  += e_phi_t_ie;
        *sum_exp.get_mut("phi_t_ia").unwrap()  += e_phi_t_ia;
        *sum_exp.get_mut("phi_t_iae").unwrap() += e_phi_t_iae;
        *sum_exp.get_mut("phi_t_ib").unwrap()  += e_phi_t_ib;
        *sum_exp.get_mut("phi_t_ig").unwrap()  += e_phi_t_ig;
        *sum_exp.get_mut("phi_i").unwrap()     += e_phi_i;
        *sum_exp.get_mut("phi_vent").unwrap()  += e_phi_vent;
        *sum_exp.get_mut("phi_hu").unwrap()    += e_phi_hu;
        *sum_exp.get_mut("phi_hl_i").unwrap()  += e_phi_hl;

        println!("\n{} {} (θ_i={:.0}°C, Δθ={:.0})", id, name, theta_i, dt);
        print_row("phi_t_ie",  e_phi_t_ie,  a_phi_t_ie);
        print_row("phi_t_ia",  e_phi_t_ia,  a_phi_t_ia);
        print_row("phi_t_iae", e_phi_t_iae, a_phi_t_iae);
        print_row("phi_t_ib",  e_phi_t_ib,  a_phi_t_ib);
        print_row("phi_t_ig",  e_phi_t_ig,  a_phi_t_ig);
        if a_phi_t_iw.abs() > 0.1 { print_row("phi_t_iw", 0.0, a_phi_t_iw); }
        print_row("phi_i",     e_phi_i,     a_phi_i);
        print_row("phi_v",     0.0,         a_phi_v);
        print_row("phi_vent",  e_phi_vent,  a_phi_vent);
        print_row("phi_hu",    e_phi_hu,    a_phi_hu);
        print_row("phi_hl_i",  e_phi_hl,    a_phi_hl);
    }

    // Building-level summary diff
    let bs = &actual["summary"];
    let eb = &expected["building"];

    println!("\n=== BUILDING TOTAL ===\n");
    print_bldg("phi_t_ie",  eb["phi_t_ie"].as_f64().unwrap_or(0.0),  *sum_act.get("phi_t_ie").unwrap());
    print_bldg("phi_t_ia",  eb["phi_t_ia"].as_f64().unwrap_or(0.0),  *sum_act.get("phi_t_ia").unwrap());
    print_bldg("phi_t_iae", eb["phi_t_iae"].as_f64().unwrap_or(0.0), *sum_act.get("phi_t_iae").unwrap());
    print_bldg("phi_t_iaBE",eb["phi_t_iaBE"].as_f64().unwrap_or(0.0),*sum_act.get("phi_t_ib").unwrap());
    print_bldg("phi_t_ig",  eb["phi_t_ig"].as_f64().unwrap_or(0.0),  *sum_act.get("phi_t_ig").unwrap());
    print_bldg("phi_i",     eb["phi_i"].as_f64().unwrap_or(0.0),     *sum_act.get("phi_i").unwrap());
    print_bldg("phi_basis", eb["phi_basis"].as_f64().unwrap_or(0.0), bs["phi_basis_total"].as_f64().unwrap_or(0.0));
    print_bldg("phi_vent",  eb["phi_vent"].as_f64().unwrap_or(0.0),  bs["phi_vent_building"].as_f64().unwrap_or(0.0));
    print_bldg("phi_hu",    eb["phi_hu"].as_f64().unwrap_or(0.0),    bs["phi_hu_building"].as_f64().unwrap_or(0.0));
    print_bldg("phi_extra", eb["phi_extra"].as_f64().unwrap_or(0.0), bs["phi_extra_quadratic"].as_f64().unwrap_or(0.0));
    print_bldg("phi_hl_build", eb["phi_hl_build"].as_f64().unwrap_or(0.0), bs["connection_capacity"].as_f64().unwrap_or(0.0));

    println!("\n=== Σ phi_hl_i (room totals) actual vs Vabi ===");
    print_bldg("Σphi_hl_i", *sum_exp.get("phi_hl_i").unwrap(), *sum_act.get("phi_hl_i").unwrap());

    Ok(())
}

fn print_row(label: &str, expected: f64, actual: f64) {
    let d = actual - expected;
    let mark = if d.abs() < 2.0 { " " } else if d > 0.0 { "+" } else { "-" };
    println!("  {:<10} exp={:>8.1}  act={:>8.1}  Δ={:>+7.1} {}", label, expected, actual, d, mark);
}

fn print_bldg(label: &str, expected: f64, actual: f64) {
    let d = actual - expected;
    let pct = if expected.abs() > 1e-3 { 100.0 * d / expected } else { 0.0 };
    let mark = if d.abs() < 2.0 { " " } else if d > 0.0 { "+" } else { "-" };
    println!("  {:<14} exp={:>9.1}  act={:>9.1}  Δ={:>+8.1} W ({:>+6.1} %) {}", label, expected, actual, d, pct, mark);
}
