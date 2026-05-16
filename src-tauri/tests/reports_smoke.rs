//! End-to-end smoke test: minimal ReportData -> valid PDF bytes.
use isso51_desktop_lib::reports::{generator, schema::*};

#[test]
fn generates_valid_pdf_for_minimal_input() {
    let data = ReportData {
        template: "standaard_rapport".into(),
        format: PaperFormat::A4,
        orientation: Orientation::Portrait,
        project: "Test Project".into(),
        project_number: None,
        client: None,
        author: "tester".into(),
        date: Some("2026-05-09".into()),
        version: "1.0".into(),
        status: ReportStatus::CONCEPT,
        cover: Some(Cover {
            subtitle: Some("Warmteverliesberekening".into()),
            image: None,
        }),
        colofon: None,
        toc: None,
        sections: vec![Section {
            title: "Uitgangspunten".into(),
            level: 1,
            content: vec![
                Block::Paragraph {
                    text: "Hello".into(),
                },
                Block::Spacer { height_mm: 4.0 },
                Block::Table {
                    title: Some("Klimaat".into()),
                    headers: vec!["Param".into(), "Waarde".into()],
                    rows: vec![
                        vec!["theta_e".into(), "-10".into()],
                        vec!["theta_b".into(), "17".into()],
                    ],
                },
            ],
        }],
        backcover: Some(BackcoverConfig { enabled: true }),
    };

    let bytes = generator::generate_pdf(&data).expect("generate succeeds");

    assert!(
        bytes.len() > 1000,
        "PDF should be at least 1KB, got {}",
        bytes.len()
    );
    assert_eq!(&bytes[0..4], b"%PDF", "missing PDF magic header");

    // Try parse with lopdf to confirm structural validity
    let doc = lopdf::Document::load_mem(&bytes).expect("lopdf parses output");
    let pages = doc.get_pages();
    assert!(!pages.is_empty(), "expected at least one page");
}
