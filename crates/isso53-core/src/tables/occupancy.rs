//! Default bezettingsdichtheid (personen per m²).
//!
//! Bron: ISSO 53 (2016) tabel 4.11, PDF p.51.
//!
//! Tabel 4.11 geeft het minimaal aan te houden aantal personen per m²
//! verblijfsgebied. Gebruik deze richtwaarden wanneer het aantal personen
//! niet door de opdrachtgever is opgegeven. De bezetting bepaalt samen met
//! tabel 4.10 (dm³/s·pp) de ventilatie-eis per ruimte.

use crate::model::enums::GebruiksFunctie;

/// Subvariant binnen een gebruiksfunctie waarvoor tabel 4.11 een afwijkende
/// bezettingsdichtheid geeft. ISSO 53 tabel 4.11 (PDF p.51).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OccupancyContext {
    /// Standaardvariant van de gebruiksfunctie ("andere gebruiksfunctie").
    #[default]
    Default,
    /// Bijeenkomstfunctie voor het aanschouwen van sport.
    BijeenkomstSport,
    /// Celfunctie voor bezoekers.
    CelBezoekers,
    /// Gezondheidszorgfunctie met bedgebied.
    GezondheidszorgBedgebied,
}

/// Default bezettingsdichtheid in personen per m² verblijfsgebied.
/// ISSO 53 tabel 4.11 (PDF p.51).
///
/// Retourneert `None` voor gebruiksfuncties waarvoor tabel 4.11 "n.v.t."
/// aangeeft (industrie, sport, winkel) — daar is geen persoonsgebonden
/// bezettingsdefault en moet het aantal personen expliciet worden opgegeven.
pub fn default_occupancy(functie: GebruiksFunctie, context: OccupancyContext) -> Option<f64> {
    match (functie, context) {
        (GebruiksFunctie::Bijeenkomst, OccupancyContext::BijeenkomstSport) => Some(0.3),
        (GebruiksFunctie::Bijeenkomst, _) => Some(0.125),
        (GebruiksFunctie::Cel, OccupancyContext::CelBezoekers) => Some(0.125),
        (GebruiksFunctie::Cel, _) => Some(0.05),
        (GebruiksFunctie::Gezondheidszorg, OccupancyContext::GezondheidszorgBedgebied) => {
            Some(0.125)
        }
        (GebruiksFunctie::Gezondheidszorg, _) => Some(0.05),
        (GebruiksFunctie::Kantoor, _) => Some(0.05),
        (GebruiksFunctie::Logies, _) => Some(0.05),
        (GebruiksFunctie::Onderwijs, _) => Some(0.125),
        // n.v.t. volgens tabel 4.11.
        (GebruiksFunctie::Industrie, _)
        | (GebruiksFunctie::Sport, _)
        | (GebruiksFunctie::Winkel, _) => None,
    }
}

/// Default bezettingsdichtheid voor de standaardvariant van een
/// gebruiksfunctie. Convenience-wrapper rond [`default_occupancy`].
/// ISSO 53 tabel 4.11 (PDF p.51).
pub fn default_occupancy_simple(functie: GebruiksFunctie) -> Option<f64> {
    default_occupancy(functie, OccupancyContext::Default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kantoor_default() {
        assert_eq!(
            default_occupancy_simple(GebruiksFunctie::Kantoor),
            Some(0.05)
        );
    }

    #[test]
    fn test_onderwijs_default() {
        assert_eq!(
            default_occupancy_simple(GebruiksFunctie::Onderwijs),
            Some(0.125)
        );
    }

    #[test]
    fn test_bijeenkomst_sport_vs_default() {
        assert_eq!(
            default_occupancy(GebruiksFunctie::Bijeenkomst, OccupancyContext::Default),
            Some(0.125)
        );
        assert_eq!(
            default_occupancy(
                GebruiksFunctie::Bijeenkomst,
                OccupancyContext::BijeenkomstSport
            ),
            Some(0.3)
        );
    }

    #[test]
    fn test_cel_bezoekers_vs_default() {
        assert_eq!(
            default_occupancy(GebruiksFunctie::Cel, OccupancyContext::Default),
            Some(0.05)
        );
        assert_eq!(
            default_occupancy(GebruiksFunctie::Cel, OccupancyContext::CelBezoekers),
            Some(0.125)
        );
    }

    #[test]
    fn test_gezondheidszorg_bedgebied_vs_default() {
        assert_eq!(
            default_occupancy(GebruiksFunctie::Gezondheidszorg, OccupancyContext::Default),
            Some(0.05)
        );
        assert_eq!(
            default_occupancy(
                GebruiksFunctie::Gezondheidszorg,
                OccupancyContext::GezondheidszorgBedgebied
            ),
            Some(0.125)
        );
    }

    #[test]
    fn test_nvt_functions() {
        assert_eq!(default_occupancy_simple(GebruiksFunctie::Industrie), None);
        assert_eq!(default_occupancy_simple(GebruiksFunctie::Sport), None);
        assert_eq!(default_occupancy_simple(GebruiksFunctie::Winkel), None);
    }
}
