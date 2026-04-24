//! Tijd-gerelateerde types: maanden en maandelijkse profielen.
//!
//! NTA 8800 rekent in **maandelijkse stappen** voor de meeste
//! energiegebruiksberekeningen. Dit module levert een type-veilige [`Month`]
//! enum plus een generieke [`MonthlyProfile<T>`] wrapper voor data die
//! per maand varieert (klimaatdata, zoninstraling, interne warmteproductie,
//! enzovoort).

use std::ops::Index;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Maanden van het jaar — NTA 8800 rekent maandelijks.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Month {
    Januari,
    Februari,
    Maart,
    April,
    Mei,
    Juni,
    Juli,
    Augustus,
    September,
    Oktober,
    November,
    December,
}

impl Month {
    /// Geef alle twaalf maanden in chronologische volgorde.
    #[must_use]
    pub const fn all() -> [Month; 12] {
        [
            Month::Januari,
            Month::Februari,
            Month::Maart,
            Month::April,
            Month::Mei,
            Month::Juni,
            Month::Juli,
            Month::Augustus,
            Month::September,
            Month::Oktober,
            Month::November,
            Month::December,
        ]
    }

    /// Geef de nul-gebaseerde index van de maand (Januari = 0, December = 11).
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Month::Januari => 0,
            Month::Februari => 1,
            Month::Maart => 2,
            Month::April => 3,
            Month::Mei => 4,
            Month::Juni => 5,
            Month::Juli => 6,
            Month::Augustus => 7,
            Month::September => 8,
            Month::Oktober => 9,
            Month::November => 10,
            Month::December => 11,
        }
    }
}

/// Generieke wrapper voor data die per maand varieert.
///
/// Serialiseert naar een **platte JSON-array** van lengte 12 (Januari eerst),
/// dankzij `#[serde(transparent)]`. Dat houdt JSON-files compact en eenvoudig
/// te lezen voor tools buiten Rust.
///
/// # Voorbeeld
/// ```
/// # use nta8800_model::time::{Month, MonthlyProfile};
/// let temps = MonthlyProfile::from_constant(10.0_f64);
/// assert_eq!(*temps.get(Month::Januari), 10.0);
/// assert_eq!(temps[Month::December], 10.0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct MonthlyProfile<T> {
    values: [T; 12],
}

impl<T> MonthlyProfile<T> {
    /// Maak een profiel met expliciete waarden per maand (Januari eerst).
    #[must_use]
    pub const fn new(values: [T; 12]) -> Self {
        Self { values }
    }

    /// Haal de waarde voor een specifieke maand op.
    #[must_use]
    pub const fn get(&self, month: Month) -> &T {
        &self.values[month.index()]
    }

    /// Itereer over `(maand, waarde)` paren in chronologische volgorde.
    pub fn iter(&self) -> impl Iterator<Item = (Month, &T)> {
        Month::all().into_iter().zip(self.values.iter())
    }

    /// Transformeer elke waarde naar een nieuw type.
    pub fn map<U, F>(self, mut f: F) -> MonthlyProfile<U>
    where
        F: FnMut(T) -> U,
    {
        let values = self.values.map(&mut f);
        MonthlyProfile { values }
    }

    /// Geef de onderliggende array als slice (read-only).
    #[must_use]
    pub fn as_array(&self) -> &[T; 12] {
        &self.values
    }
}

impl<T: Clone> MonthlyProfile<T> {
    /// Maak een profiel waarbij alle 12 maanden identieke waarden hebben.
    #[must_use]
    pub fn from_constant(v: T) -> Self {
        Self {
            values: [(); 12].map(|()| v.clone()),
        }
    }
}

impl<T> Index<Month> for MonthlyProfile<T> {
    type Output = T;

    fn index(&self, month: Month) -> &T {
        self.get(month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_all_length() {
        assert_eq!(Month::all().len(), 12);
    }

    #[test]
    fn month_index_sequence() {
        assert_eq!(Month::Januari.index(), 0);
        assert_eq!(Month::December.index(), 11);
    }

    #[test]
    fn from_constant_fills_all_months() {
        let profile = MonthlyProfile::from_constant(4.2_f64);
        for (_month, value) in profile.iter() {
            assert!((value - 4.2).abs() < 1e-9);
        }
    }

    #[test]
    fn iter_yields_twelve_pairs() {
        let profile = MonthlyProfile::from_constant(0_i32);
        assert_eq!(profile.iter().count(), 12);
    }

    #[test]
    fn index_operator_works() {
        let profile = MonthlyProfile::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(profile[Month::Januari], 1);
        assert_eq!(profile[Month::December], 12);
    }

    #[test]
    fn map_transforms_values() {
        let profile = MonthlyProfile::from_constant(2_i32);
        let doubled = profile.map(|v| v * 2);
        assert_eq!(doubled[Month::Juli], 4);
    }

    #[test]
    fn serde_round_trip_as_flat_array() {
        let profile = MonthlyProfile::new([0.0_f64; 12]);
        let json = serde_json::to_string(&profile).unwrap();
        // transparent wrapper ⇒ JSON is a flat array
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));

        let back: MonthlyProfile<f64> = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, back);
    }

    #[test]
    fn serde_with_distinct_values() {
        let profile = MonthlyProfile::new([
            1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ]);
        let json = serde_json::to_string(&profile).unwrap();
        let back: MonthlyProfile<f64> = serde_json::from_str(&json).unwrap();
        assert!((profile[Month::Maart] - back[Month::Maart]).abs() < 1e-9);
        assert!((back[Month::December] - 12.0).abs() < 1e-9);
    }
}
