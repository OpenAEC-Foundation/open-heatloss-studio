//! Toe te rekenen infiltratie-fractie z op gebouwniveau.
//!
//! Bron: ISSO 53 (2016) tabel 5.1, PDF p.56-57.
//!
//! Bij de bepaling van het infiltratiewarmteverlies per vertrek wordt
//! aangenomen dat de wind op de betreffende buitengevel staat. Omdat de wind
//! niet op alle buitengevels tegelijk staat, zou simpelweg sommeren van de
//! infiltratie per vertrek tot overdimensionering leiden. Daarom wordt het
//! gesommeerde infiltratiewarmteverlies op gebouwniveau met de fractie z
//! vermenigvuldigd (formule 5.2).

/// Configuratie van de warmteopwekkers op gebouwniveau.
/// ISSO 53 tabel 5.1 (PDF p.56-57).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceZoneConfig {
    /// Systemen met volledig gescheiden warmteopwekkers per zone.
    SeparatePerZone,
    /// Overige gevallen.
    Other,
}

/// Toe te rekenen infiltratie-fractie z op gebouwniveau.
/// ISSO 53 tabel 5.1 (PDF p.56-57), dimensieloos.
///
/// - `SeparatePerZone` → 1,0 (elke zone heeft een eigen opwekker, dus de
///   maatgevende windrichting per zone telt volledig mee);
/// - `Other` → 0,5 (de wind staat niet op alle gevels tegelijk).
pub fn source_fraction_z(config: SourceZoneConfig) -> f64 {
    match config {
        SourceZoneConfig::SeparatePerZone => 1.0,
        SourceZoneConfig::Other => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_fraction_values() {
        assert_eq!(source_fraction_z(SourceZoneConfig::SeparatePerZone), 1.0);
        assert_eq!(source_fraction_z(SourceZoneConfig::Other), 0.5);
    }
}
