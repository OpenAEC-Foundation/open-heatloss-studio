//! Input validation for ISSO 53 calculations.

use crate::error::{Isso53Error, Result};
use crate::model::Project;

/// Validate project input according to ISSO 53 requirements.
///
/// # ISSO 53 specific validations
/// - Room height must be ≤ 4.0m (refer to ISSO 57 for taller buildings)
/// - All required fields must be present
/// - Values must be within reasonable ranges
pub fn validate_project(project: &Project) -> Result<()> {
    // Validate each room
    for room in &project.rooms {
        validate_room_height(room.height)?;

        if room.floor_area <= 0.0 {
            return Err(Isso53Error::InvalidInput(format!(
                "Room '{}' has non-positive floor area: {}",
                room.name, room.floor_area
            )));
        }

        if room.constructions.is_empty() {
            return Err(Isso53Error::InvalidInput(format!(
                "Room '{}' has no construction elements",
                room.name
            )));
        }
    }

    // Validate building
    if project.building.construction_year < 1900 || project.building.construction_year > 2030 {
        return Err(Isso53Error::InvalidInput(format!(
            "Building construction year {} is out of reasonable range (1900-2030)",
            project.building.construction_year
        )));
    }

    Ok(())
}

/// Validate that room height is within ISSO 53 scope (≤ 4.0m).
fn validate_room_height(height: f64) -> Result<()> {
    if height > 4.0 {
        return Err(Isso53Error::HeightExceedsLimit { height });
    }

    if height <= 0.0 {
        return Err(Isso53Error::InvalidInput(format!(
            "Room height must be positive, got: {}",
            height
        )));
    }

    Ok(())
}