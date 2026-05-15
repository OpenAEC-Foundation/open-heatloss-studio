//! Standalone CLI om een PDF-rapport te genereren van een Project JSON.
//!
//! Pattern gespiegeld op Open Calc Studio's `src-tauri/src/bin/gen_pdf.rs`.
//! Wordt door de MCP server (`mcp-server/`) aangeroepen voor `generate_pdf`
//! tool, en kan ook via CI op een fixture gedraaid worden voor smoke-testing.
//!
//! Usage:
//!   gen_pdf <input.project.json | input.isso51.json | input.ifcenergy> <output.pdf>
//!
//! Het bestand mag een raw Project zijn, een legacy `.isso51.json` envelope,
//! of een IFCX `.ifcenergy` document met `isso51::envelope::v1` attribute.
use isso51_desktop_lib::reports::{generator, schema};
use isso51_core::model::Project;
use isso51_core::result::ProjectResult;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: gen_pdf <input.json> <output.pdf>");
        std::process::exit(2);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let raw = std::fs::read_to_string(input_path)?;
    let project = parse_input_to_project(&raw)?;

    // Run calc to get result.
    let result = isso51_core::calculate(&project)
        .map_err(|e| format!("calc failed: {e}"))?;

    // Build minimale ReportData.
    let data = build_report_data(&project, &result);

    // Render PDF.
    let bytes = generator::generate_pdf(&data)
        .map_err(|e| format!("pdf generation failed: {e}"))?;

    std::fs::write(output_path, &bytes)?;
    println!("Wrote {} ({} bytes)", output_path, bytes.len());
    Ok(())
}

/// Detecteer envelope-formaat en haal Project eruit.
fn parse_input_to_project(raw: &str) -> Result<Project, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(raw)?;

    // 1. IFCX `.ifcenergy` document met envelope.
    if v.get("header").and_then(|h| h.get("ifcxVersion")).is_some()
        && v.get("data").map(|d| d.is_array()).unwrap_or(false)
    {
        if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
            for entry in arr {
                if let Some(env) = entry.get("attributes").and_then(|a| a.get("isso51::envelope::v1")) {
                    if let Some(p) = env.get("project") {
                        return Ok(serde_json::from_value(p.clone())?);
                    }
                }
            }
        }
        return Err("Geen isso51::envelope::v1 in IFCX document".into());
    }

    // 2. Legacy `.isso51.json` envelope.
    if v.get("schema").and_then(|s| s.as_str()) == Some("isso51-project-v1") {
        if let Some(p) = v.get("project") {
            return Ok(serde_json::from_value(p.clone())?);
        }
    }

    // 3. Raw Project JSON.
    Ok(serde_json::from_value(v)?)
}

/// 1x1 transparent PNG, base64 — used as placeholder for Diagrammen so the
/// section renders with the expected structure (caption + image flowable) for
/// page-break / backcover verification. The frontend builder substitutes real
/// SVG→PNG rasterized charts here.
const PLACEHOLDER_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkAAIAAAoAAv/lxKUAAAAASUVORK5CYII=";

/// Build full ReportData voor PDF-generatie — mirrors `reportBuilder.ts`.
///
/// Sections: Uitgangspunten · Vertrekken overzicht · per-vertrek details (level 2)
/// · Diagrammen (placeholder PNGs) · Gebouwresultaten · Backcover.
/// Diagrammen-charts zijn placeholders — de frontend rasterizeert echte SVG's,
/// maar voor backend-tests volstaat een 1x1 PNG om de page-break/backcover
/// logica te valideren.
fn build_report_data(project: &Project, result: &ProjectResult) -> schema::ReportData {
    use schema::*;

    let project_name = if project.info.name.is_empty() {
        "Naamloos project".to_string()
    } else {
        project.info.name.clone()
    };
    let today = chrono_today();

    let mut sections: Vec<Section> = Vec::new();

    // ───── 1. Uitgangspunten ───────────────────────────────────────────
    sections.push(Section {
        title: "Uitgangspunten".into(),
        level: 1,
        content: vec![
            Block::Table {
                title: Some("Gebouwgegevens".into()),
                headers: vec!["Parameter".into(), "Waarde".into()],
                rows: vec![
                    vec![
                        "Gebouwtype".into(),
                        format!("{:?}", project.building.building_type),
                    ],
                    vec![
                        "Beveiligingsklasse".into(),
                        format!("{:?}", project.building.security_class),
                    ],
                    vec![
                        "q_v10-waarde".into(),
                        format!("{} dm³/s", project.building.qv10),
                    ],
                    vec![
                        "Totaal vloeroppervlak".into(),
                        format!("{} m²", project.building.total_floor_area),
                    ],
                    vec![
                        "Aantal bouwlagen".into(),
                        format!("{}", project.building.num_floors),
                    ],
                    vec![
                        "Nacht-setback".into(),
                        if project.building.has_night_setback { "Ja".into() } else { "Nee".into() },
                    ],
                    vec![
                        "Opwarmtijd".into(),
                        format!("{} uur", project.building.warmup_time),
                    ],
                ],
            },
            Block::Spacer { height_mm: 6.0 },
            Block::Table {
                title: Some("Klimaatgegevens".into()),
                headers: vec!["Parameter".into(), "Waarde".into()],
                rows: vec![
                    vec!["Buitentemperatuur (θ_e)".into(), format!("{} °C", project.climate.theta_e)],
                    vec!["Grondtemperatuur (θ_b)".into(), format!("{} °C", project.climate.theta_b_residential)],
                ],
            },
            Block::Spacer { height_mm: 6.0 },
            Block::Table {
                title: Some("Ventilatiesysteem".into()),
                headers: vec!["Parameter".into(), "Waarde".into()],
                rows: vec![
                    vec![
                        "Systeemtype".into(),
                        format!("{:?}", project.ventilation.system_type),
                    ],
                    vec![
                        "Warmteterugwinning".into(),
                        if project.ventilation.has_heat_recovery { "Ja".into() } else { "Nee".into() },
                    ],
                    vec![
                        "WTW rendement".into(),
                        match project.ventilation.heat_recovery_efficiency {
                            Some(e) => format!("{:.0}%", e * 100.0),
                            None => "-".into(),
                        },
                    ],
                ],
            },
        ],
    });

    // ───── 2. Vertrekken overzicht ──────────────────────────────────────
    let mut overview_rows: Vec<Vec<String>> = Vec::new();
    for room_result in &result.rooms {
        let phi_t = room_result.transmission.phi_t;
        let phi_v = room_result.ventilation.phi_v + room_result.infiltration.phi_i;
        let phi_hu = room_result.heating_up.phi_hu;
        let phi_sys = room_result.system_losses.phi_system_total;
        let phi_total = room_result.total_heat_loss;

        overview_rows.push(vec![
            room_result.room_name.clone(),
            format!("{:.2}", room_result.theta_i),
            format!("{:.0}", phi_t),
            format!("{:.0}", phi_v),
            format!("{:.0}", phi_hu),
            format!("{:.0}", phi_sys),
            format!("{:.0}", phi_total),
        ]);
    }
    sections.push(Section {
        title: "Vertrekken overzicht".into(),
        level: 1,
        content: vec![Block::Table {
            title: Some("Samenvatting per vertrek".into()),
            headers: vec![
                "Vertrek".into(),
                "θ_i (°C)".into(),
                "Φ_T (W)".into(),
                "Φ_v (W)".into(),
                "Φ_hu (W)".into(),
                "Φ_sys (W)".into(),
                "Φ_totaal (W)".into(),
            ],
            rows: overview_rows,
        }],
    });

    // ───── 3. Per-vertrek details (level 2) ────────────────────────────
    // Mirrors reportBuilder.ts buildRoomDetailSection — each room renders
    // four small tables (transmissie, ventilatie/infiltratie, opwarm/sys,
    // totaal). Level 2 means NO page-break before each room — the layout
    // engine flows them and breaks naturally per page capacity.
    for room in &result.rooms {
        sections.push(Section {
            title: room.room_name.clone(),
            level: 2,
            content: vec![
                Block::Paragraph {
                    text: format!("<b>Vertrek:</b> {}", room.room_name),
                },
                Block::Spacer { height_mm: 2.0 },
                Block::Table {
                    title: Some("Transmissieverliezen".into()),
                    headers: vec!["Component".into(), "Waarde".into()],
                    rows: vec![
                        vec!["H_T,ie (schil)".into(), format!("{:.2} W/K", room.transmission.h_t_exterior)],
                        vec!["H_T,ia (intern)".into(), format!("{:.2} W/K", room.transmission.h_t_adjacent_rooms)],
                        vec!["H_T,io (onverwarmd)".into(), format!("{:.2} W/K", room.transmission.h_t_unheated)],
                        vec!["H_T,ib (buurwoning)".into(), format!("{:.2} W/K", room.transmission.h_t_adjacent_buildings)],
                        vec!["H_T,ig (grond)".into(), format!("{:.2} W/K", room.transmission.h_t_ground)],
                        vec!["Φ_T totaal".into(), format!("{:.0} W", room.transmission.phi_t)],
                    ],
                },
                Block::Spacer { height_mm: 2.0 },
                Block::Table {
                    title: Some("Ventilatie & infiltratie".into()),
                    headers: vec!["Component".into(), "Waarde".into()],
                    rows: vec![
                        vec!["q_v (ventilatie)".into(), format!("{:.2} dm³/s", room.ventilation.q_v)],
                        vec!["H_v".into(), format!("{:.2} W/K", room.ventilation.h_v)],
                        vec!["Φ_v (ventilatie)".into(), format!("{:.0} W", room.ventilation.phi_v)],
                        vec!["H_i (infiltratie)".into(), format!("{:.2} W/K", room.infiltration.h_i)],
                        vec!["Φ_i (infiltratie)".into(), format!("{:.0} W", room.infiltration.phi_i)],
                    ],
                },
                Block::Spacer { height_mm: 2.0 },
                Block::Table {
                    title: Some("Opwarmtoeslag & systeemverliezen".into()),
                    headers: vec!["Component".into(), "Waarde".into()],
                    rows: vec![
                        vec!["f_RH".into(), format!("{:.2}", room.heating_up.f_rh)],
                        vec!["A_acc".into(), format!("{:.2} m²", room.heating_up.accumulating_area)],
                        vec!["Φ_hu".into(), format!("{:.0} W", room.heating_up.phi_hu)],
                        vec!["Φ_sys (totaal)".into(), format!("{:.0} W", room.system_losses.phi_system_total)],
                    ],
                },
                Block::Spacer { height_mm: 2.0 },
                Block::Table {
                    title: Some("Totaal".into()),
                    headers: vec!["Component".into(), "Waarde".into()],
                    rows: vec![
                        vec!["Φ_basis".into(), format!("{:.0} W", room.basis_heat_loss)],
                        vec!["Φ_extra".into(), format!("{:.0} W", room.extra_heat_loss)],
                        vec!["Φ_totaal".into(), format!("<b>{:.0} W</b>", room.total_heat_loss)],
                    ],
                },
            ],
        });
    }

    // ───── 4. Diagrammen (placeholder charts) ──────────────────────────
    // Real charts come from frontend SVG → PNG rasterization. For backend
    // tests we use 1x1 PNGs to verify the section + page-break behaviour.
    sections.push(Section {
        title: "Diagrammen".into(),
        level: 1,
        content: vec![
            Block::Image {
                src: ImageRef {
                    data: PLACEHOLDER_PNG_B64.into(),
                    media_type: "image/png".into(),
                    filename: Some("verliezen-per-vertrek.png".into()),
                },
                caption: Some("Warmteverliezen per vertrek".into()),
                width_mm: 170.0,
                alignment: ImageAlignment::Center,
            },
            Block::Spacer { height_mm: 4.0 },
            Block::Image {
                src: ImageRef {
                    data: PLACEHOLDER_PNG_B64.into(),
                    media_type: "image/png".into(),
                    filename: Some("gebouwtotaal.png".into()),
                },
                caption: Some("Gebouwtotaal warmteverliezen per type".into()),
                width_mm: 170.0,
                alignment: ImageAlignment::Center,
            },
            Block::Spacer { height_mm: 4.0 },
            Block::Image {
                src: ImageRef {
                    data: PLACEHOLDER_PNG_B64.into(),
                    media_type: "image/png".into(),
                    filename: Some("verlies-per-constructietype.png".into()),
                },
                caption: Some("Verlies per constructietype".into()),
                width_mm: 150.0,
                alignment: ImageAlignment::Center,
            },
        ],
    });

    // ───── 5. Gebouwresultaten ──────────────────────────────────────────
    let s = &result.summary;
    sections.push(Section {
        title: "Gebouwresultaten".into(),
        level: 1,
        content: vec![
            Block::Table {
                title: Some("Totalen".into()),
                headers: vec!["Component".into(), "Waarde".into()],
                rows: vec![
                    vec!["Transmissie (schil)".into(), format!("{:.0} W", s.total_envelope_loss)],
                    vec!["Buurwoningverlies".into(), format!("{:.0} W", s.total_neighbor_loss)],
                    vec!["Ventilatie".into(), format!("{:.0} W", s.total_ventilation_loss)],
                    vec!["Opwarmtoeslag".into(), format!("{:.0} W", s.total_heating_up)],
                    vec!["Systeemverliezen".into(), format!("{:.0} W", s.total_system_losses)],
                    vec!["Collectieve bijdrage".into(), format!("{:.0} W", s.collective_contribution)],
                ],
            },
            Block::Spacer { height_mm: 8.0 },
            Block::Paragraph {
                text: format!(
                    "Aansluitvermogen: {:.0} W (referentie: ISSO 51:2023)",
                    s.connection_capacity
                ),
            },
        ],
    });

    ReportData {
        template: "standaard_rapport".into(),
        format: PaperFormat::A4,
        orientation: Orientation::Portrait,
        project: project_name,
        project_number: project.info.project_number.clone(),
        client: project.info.client.clone(),
        author: project
            .info
            .engineer
            .clone()
            .unwrap_or_else(|| "3BM Bouwkunde".into()),
        date: project.info.date.clone().or(Some(today)),
        version: "1.0".into(),
        status: ReportStatus::CONCEPT,
        cover: Some(Cover {
            subtitle: Some("Warmteverliesberekening conform ISSO 51:2023".into()),
            image: None,
        }),
        colofon: None,
        toc: None,
        sections,
        backcover: Some(BackcoverConfig { enabled: true }),
        footer: None,
        header: None,
        style: None,
    }
}

/// Today as ISO 8601 string. Avoid `chrono` dep — use `std::time` and
/// manual date math (CLI is not running long enough for DST/leap concerns).
fn chrono_today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = now / 86_400;
    // Compute Y-M-D from Unix epoch days using Howard Hinnant's algorithm.
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let yr = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", yr, m, d)
}
