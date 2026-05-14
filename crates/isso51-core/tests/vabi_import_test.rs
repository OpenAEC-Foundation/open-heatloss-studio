//! Integration tests for Vabi import functionality.
//!
//! These tests validate the Vabi `.vp` import against known reference files.
//! Tests are marked with `#[ignore]` so they skip gracefully if reference
//! files are missing (they're in .gitignore for privacy).

#[cfg(feature = "vabi-import")]
mod vabi_tests {
    use isso51_core::import::import_vabi_project;
    use std::path::{Path, PathBuf};

    /// Resolve a path relative to the workspace `tests/references/` directory.
    /// `CARGO_MANIFEST_DIR` is the crate dir (`crates/isso51-core`), so we go up two levels.
    fn reference_path(filename: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/references")
            .join(filename)
    }

    /// Test import of Voorweg 210a project.
    ///
    /// This is the main validation test against the known reference project.
    /// Expected values are from the 13-05 discovery session.
    #[test]
    fn test_import_voorweg_210a() {
        let vp_path = reference_path("Voorweg 210a - nieuw.vp");
        let vp_path = vp_path.as_path();

        if !vp_path.exists() {
            println!("⏭️  Skipping test: {} not found (gitignored reference file)", vp_path.display());
            return;
        }

        let result = import_vabi_project(vp_path);

        match result {
            Ok(project) => {
                println!("\n✅ Successfully imported: {}", project.info.name);

                // Basic validation
                assert!(!project.info.name.is_empty(), "Project name should not be empty");

                // Expected: 21 rooms (from session doc)
                assert_eq!(
                    project.rooms.len(), 21,
                    "Expected 21 rooms, found {}", project.rooms.len()
                );

                // Expected: qv10 = 0.4, infiltration_method = Specific
                assert!(
                    (project.building.qv10 - 0.4).abs() < 0.1,
                    "Expected qv10 ≈ 0.4, found {}", project.building.qv10
                );
                assert_eq!(
                    project.building.infiltration_method,
                    isso51_core::model::enums::InfiltrationMethod::VabiCompat,
                    "Expected VabiCompat infiltration method (maps from Vabi Specific)"
                );

                // Expected: theta_e = -10.0°C
                assert!(
                    (project.climate.theta_e - (-10.0)).abs() < 0.5,
                    "Expected theta_e ≈ -10.0°C, found {}", project.climate.theta_e
                );

                // Print summary for manual verification
                println!("\n📊 Import Summary:");
                println!("  Project: {}", project.info.name);
                println!("  Rooms: {}", project.rooms.len());
                println!("  qv10: {:.3} ({})", project.building.qv10,
                    match project.building.infiltration_method {
                        isso51_core::model::enums::InfiltrationMethod::VabiCompat => "VabiCompat",
                        isso51_core::model::enums::InfiltrationMethod::MeasuredQv10 => "MeasuredQv10",
                        _ => "Other"
                    }
                );
                println!("  theta_e: {:.1}°C", project.climate.theta_e);
                println!("  Building type: {:?}", project.building.building_type);
                println!("  Ventilation: {:?}", project.ventilation.system_type);

                // Room details (first 5)
                println!("\n🏠 First 5 rooms:");
                for (i, room) in project.rooms.iter().take(5).enumerate() {
                    let theta_i = room.internal_air_temperature.unwrap_or(20.0);
                    println!("  {}. {} ({}) - {:.1}°C",
                        i+1, room.name, room.id, theta_i);
                }
                if project.rooms.len() > 5 {
                    println!("  ... and {} more", project.rooms.len() - 5);
                }
            }
            Err(e) => {
                panic!("Import failed: {}", e);
            }
        }
    }

    /// Test import of the second reference project (Groningen OPDC, ~106 rooms).
    #[test]
    fn test_import_24221_project() {
        let vp_path = reference_path("24221-20250618.vp");
        let vp_path = vp_path.as_path();

        if !vp_path.exists() {
            println!("⏭️  Skipping test: {} not found (gitignored reference file)", vp_path.display());
            return;
        }

        let result = import_vabi_project(vp_path);

        match result {
            Ok(project) => {
                println!("\n✅ Successfully imported: {}", project.info.name);

                // Basic validation
                assert!(!project.info.name.is_empty(), "Project name should not be empty");
                assert!(!project.rooms.is_empty(), "Should have at least one room");

                // Print summary
                println!("\n📊 Import Summary:");
                println!("  Project: {}", project.info.name);
                println!("  Rooms: {}", project.rooms.len());
                println!("  qv10: {:.3} ({:?})", project.building.qv10, project.building.infiltration_method);
                println!("  theta_e: {:.1}°C", project.climate.theta_e);
                println!("  Building type: {:?}", project.building.building_type);
                println!("  Ventilation: {:?}", project.ventilation.system_type);
            }
            Err(e) => {
                panic!("Import failed: {}", e);
            }
        }
    }

    /// Test that import fails gracefully on invalid files.
    #[test]
    fn test_import_invalid_file() {
        let result = import_vabi_project(Path::new("nonexistent.vp"));
        assert!(result.is_err(), "Should fail on nonexistent file");

        // Test with regular file (not a .vp)
        let result = import_vabi_project(Path::new("Cargo.toml"));
        assert!(result.is_err(), "Should fail on non-ZIP file");
    }

    /// Test specific error types.
    #[test]
    fn test_error_types() {
        let result = import_vabi_project(Path::new("nonexistent.vp"));

        if let Err(e) = result {
            // Should be a VabiZipError for file not found
            let error_msg = e.to_string();
            assert!(error_msg.contains("Vabi ZIP error") || error_msg.contains("Cannot open"),
                "Expected ZIP error, got: {}", error_msg);
        } else {
            panic!("Expected error for nonexistent file");
        }
    }
}