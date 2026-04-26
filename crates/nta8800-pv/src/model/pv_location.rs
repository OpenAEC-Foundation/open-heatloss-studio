//! Geografische locatie voor PV-berekeningen.

use crate::errors::PvError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Geografische locatie voor PV-opbrengst berekeningen.
///
/// Definieert de positie waar de PV-installatie zich bevindt, voor
/// eventuele toekomstige locatie-specifieke correcties (V2).
/// V1 gebruikt alleen klimaatdata De Bilt, dus locatie is documentair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PvLocation {
    /// Latitude (breedtegraad) in decimale graden.
    ///
    /// Positief = noordelijk halfrond, negatief = zuidelijk halfrond.
    /// Nederland: ~51.0° - 53.5°. Moet tussen -90° en +90° liggen.
    pub latitude: f64,

    /// Longitude (lengtegraad) in decimale graden.
    ///
    /// Positief = oostelijk van Greenwich, negatief = westelijk.
    /// Nederland: ~3.3° - 7.2°. Moet tussen -180° en +180° liggen.
    pub longitude: f64,

    /// Optionele naam/beschrijving van de locatie.
    ///
    /// Voor documentatie en rapportage. Bijv. "Utrecht", "Amsterdam Noord".
    #[serde(default)]
    pub name: Option<String>,
}

impl PvLocation {
    /// Creëert een nieuwe PV-locatie met validatie.
    ///
    /// # Argumenten
    ///
    /// * `latitude` - Breedtegraad in decimale graden (-90° - +90°)
    /// * `longitude` - Lengtegraad in decimale graden (-180° - +180°)
    ///
    /// # Errors
    ///
    /// Retourneert [`PvError`] als latitude of longitude buiten het
    /// geldige bereik ligt.
    ///
    /// # Example
    ///
    /// ```
    /// use nta8800_pv::PvLocation;
    ///
    /// // Utrecht centrum
    /// let location = PvLocation::new(52.0907, 5.1214)?;
    /// assert_eq!(location.latitude, 52.0907);
    /// assert_eq!(location.longitude, 5.1214);
    /// # Ok::<(), nta8800_pv::PvError>(())
    /// ```
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, PvError> {
        Self::validate_latitude(latitude)?;
        Self::validate_longitude(longitude)?;

        Ok(Self {
            latitude,
            longitude,
            name: None,
        })
    }

    /// Creëert een PV-locatie met naam.
    ///
    /// Zoals [`Self::new`], maar met optionele naam/beschrijving.
    ///
    /// # Example
    ///
    /// ```
    /// use nta8800_pv::PvLocation;
    ///
    /// let location = PvLocation::with_name(52.0907, 5.1214, "Utrecht")?;
    /// assert_eq!(location.name, Some("Utrecht".to_string()));
    /// # Ok::<(), nta8800_pv::PvError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Retourneert [`PvError`] als latitude of longitude buiten het geldige bereik ligt.
    pub fn with_name(
        latitude: f64,
        longitude: f64,
        name: impl Into<String>,
    ) -> Result<Self, PvError> {
        let mut location = Self::new(latitude, longitude)?;
        location.name = Some(name.into());
        Ok(location)
    }

    /// Berekent de afstand tot een andere locatie in kilometers (haversine).
    ///
    /// Gebruikt de haversine-formule voor berekening van de orthodrome afstand
    /// over het aardoppervlak. Accuraat voor afstanden binnen Nederland.
    ///
    /// # Example
    ///
    /// ```
    /// use nta8800_pv::PvLocation;
    ///
    /// let utrecht = PvLocation::new(52.0907, 5.1214)?;
    /// let amsterdam = PvLocation::new(52.3676, 4.9041)?;
    /// let distance_km = utrecht.distance_to(&amsterdam);
    /// assert!((distance_km - 34.0).abs() < 1.0); // ~34 km hemelsbreed
    /// # Ok::<(), nta8800_pv::PvError>(())
    /// ```
    #[must_use]
    pub fn distance_to(&self, other: &PvLocation) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = self.latitude.to_radians();
        let lat2_rad = other.latitude.to_radians();
        let delta_lat_rad = (other.latitude - self.latitude).to_radians();
        let delta_lon_rad = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat_rad / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon_rad / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    fn validate_latitude(latitude: f64) -> Result<(), PvError> {
        if (-90.0..=90.0).contains(&latitude) {
            Ok(())
        } else {
            Err(PvError::InvalidLatitude(latitude))
        }
    }

    fn validate_longitude(longitude: f64) -> Result<(), PvError> {
        if (-180.0..=180.0).contains(&longitude) {
            Ok(())
        } else {
            Err(PvError::InvalidLongitude(longitude))
        }
    }
}

/// Voorgedefinieerde locaties voor Nederland.
impl PvLocation {
    /// De Bilt (KNMI hoofdstation).
    #[must_use]
    pub fn de_bilt() -> Self {
        Self {
            latitude: 52.1015,
            longitude: 5.1807,
            name: Some("De Bilt".to_string()),
        }
    }

    /// Amsterdam centrum.
    #[must_use]
    pub fn amsterdam() -> Self {
        Self {
            latitude: 52.3676,
            longitude: 4.9041,
            name: Some("Amsterdam".to_string()),
        }
    }

    /// Rotterdam centrum.
    #[must_use]
    pub fn rotterdam() -> Self {
        Self {
            latitude: 51.9244,
            longitude: 4.4777,
            name: Some("Rotterdam".to_string()),
        }
    }

    /// Utrecht centrum.
    #[must_use]
    pub fn utrecht() -> Self {
        Self {
            latitude: 52.0907,
            longitude: 5.1214,
            name: Some("Utrecht".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn new_with_valid_coordinates_succeeds() {
        let location = PvLocation::new(52.1, 5.2).unwrap();
        assert_eq!(location.latitude, 52.1);
        assert_eq!(location.longitude, 5.2);
        assert_eq!(location.name, None);
    }

    #[test]
    fn with_name_sets_name() {
        let location = PvLocation::with_name(52.1, 5.2, "Test Location").unwrap();
        assert_eq!(location.name, Some("Test Location".to_string()));
    }

    #[test]
    fn invalid_latitude_returns_error() {
        let result = PvLocation::new(-91.0, 5.2);
        assert_eq!(result.unwrap_err(), PvError::InvalidLatitude(-91.0));

        let result = PvLocation::new(91.0, 5.2);
        assert_eq!(result.unwrap_err(), PvError::InvalidLatitude(91.0));
    }

    #[test]
    fn invalid_longitude_returns_error() {
        let result = PvLocation::new(52.1, -181.0);
        assert_eq!(result.unwrap_err(), PvError::InvalidLongitude(-181.0));

        let result = PvLocation::new(52.1, 181.0);
        assert_eq!(result.unwrap_err(), PvError::InvalidLongitude(181.0));
    }

    #[test]
    fn edge_case_coordinates() {
        // Exacte grenzen moeten geldig zijn
        assert!(PvLocation::new(-90.0, -180.0).is_ok());
        assert!(PvLocation::new(90.0, 180.0).is_ok());
        assert!(PvLocation::new(0.0, 0.0).is_ok());
    }

    #[test]
    fn predefined_locations_are_valid() {
        let de_bilt = PvLocation::de_bilt();
        assert_eq!(de_bilt.name, Some("De Bilt".to_string()));
        assert!((de_bilt.latitude - 52.1015).abs() < 0.001);

        let amsterdam = PvLocation::amsterdam();
        assert_eq!(amsterdam.name, Some("Amsterdam".to_string()));

        let rotterdam = PvLocation::rotterdam();
        assert_eq!(rotterdam.name, Some("Rotterdam".to_string()));

        let utrecht = PvLocation::utrecht();
        assert_eq!(utrecht.name, Some("Utrecht".to_string()));
    }

    #[test]
    fn distance_calculation_utrecht_amsterdam() {
        let utrecht = PvLocation::utrecht();
        let amsterdam = PvLocation::amsterdam();
        let distance = utrecht.distance_to(&amsterdam);

        // Utrecht-Amsterdam haversine ≈ 34 km (hemelsbreed; 46 km is wegafstand)
        assert_abs_diff_eq!(distance, 34.0, epsilon = 1.0);
    }

    #[test]
    fn distance_to_same_location_is_zero() {
        let location = PvLocation::new(52.1, 5.2).unwrap();
        let distance = location.distance_to(&location);
        assert_abs_diff_eq!(distance, 0.0, epsilon = 0.001);
    }

    #[test]
    fn distance_is_symmetric() {
        let loc1 = PvLocation::new(52.1, 5.2).unwrap();
        let loc2 = PvLocation::new(51.9, 4.9).unwrap();

        let dist1_to_2 = loc1.distance_to(&loc2);
        let dist2_to_1 = loc2.distance_to(&loc1);

        assert_abs_diff_eq!(dist1_to_2, dist2_to_1, epsilon = 0.001);
    }
}
