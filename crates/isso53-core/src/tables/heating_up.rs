//! Specifieke toeslag voor bedrijfsbeperking φ_hu,i [W/m²].
//!
//! Bron: ISSO 53 (2016) §4.8, tabel 4.13 (vrije afkoeling) + tabel 4.14
//! (beperkte afkoeling), PDF p.53.
//!
//! De toeslag hangt af van:
//! - **Afkoel-regime:** vrije afkoeling (tabel 4.13) of beperkte afkoeling
//!   (slechts enkele graden, tabel 4.14);
//! - **Opwarmtijd** [h] — rij-as, lineair interpoleerbaar tussen rijwaarden;
//! - **Zwaarte gebouw** (l/z) — afgeleid uit c_eff: `c_eff ≤ 70 → l (licht)`,
//!   anders `z (zwaar)` (§4.8.1, PDF p.53);
//! - **Aantal luchtwisselingen** tijdens afkoelperiode (0,1 of 0,5);
//! - **Mate van verlaging** — voor 4.13 het aantal úren verlaging
//!   {8, 14, 62=weekend}; voor 4.14 het aantal gráden verlaging {1..5}.
//!
//! `-` (niet gedefinieerd) is in de constanten gerepresenteerd als `None`.

/// Aantal luchtwisselingen tijdens de afkoelperiode (kolom-subkeuze).
/// §4.8 voetnoot 1: bij gesloten ramen/deuren + uitgeschakelde installatie → 0,1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirChanges {
    /// 0,1 luchtwisselingen (gesloten ramen/deuren, installatie uit).
    Low,
    /// 0,5 luchtwisselingen.
    High,
}

/// Zwaarte van het gebouw, afgeleid uit c_eff (§4.8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingWeight {
    /// Licht/middelzwaar (l): c_eff ≤ 70 Wh/(m³·K).
    Light,
    /// Zwaar (z): c_eff > 70 Wh/(m³·K).
    Heavy,
}

impl BuildingWeight {
    /// Bepaal de zwaarteklasse uit c_eff [Wh/(m³·K)] volgens §4.8.1:
    /// `c_eff ≤ 70 → licht (l)`, anders `zwaar (z)`.
    pub fn from_c_eff(c_eff: f64) -> Self {
        if c_eff <= 70.0 {
            BuildingWeight::Light
        } else {
            BuildingWeight::Heavy
        }
    }
}

/// Opwarmtijd-rijwaarden [h] voor tabel 4.13 (vrije afkoeling).
/// ISSO 53 tabel 4.13, PDF p.53.
pub const WARMUP_HOURS_FREE: [f64; 7] = [0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 12.0];

/// Opwarmtijd-rijwaarden [h] voor tabel 4.14 (beperkte afkoeling).
/// ISSO 53 tabel 4.14, PDF p.53.
pub const WARMUP_HOURS_LIMITED: [f64; 5] = [0.5, 1.0, 2.0, 3.0, 4.0];

/// Geldige uren-verlaging keuzes voor vrije afkoeling (tabel 4.13).
/// 8 = tweeploegendienst (voetnoot 2), 14 = standaard dagverlaging,
/// 62 = weekendverlaging (voetnoot 3).
pub const SETBACK_HOURS_FREE: [u32; 3] = [8, 14, 62];

/// Tabel 4.13 — specifieke toeslag φ_hu,i [W/m²] bij **vrije** afkoeling.
/// ISSO 53 tabel 4.13, PDF p.53.
///
/// Indexering: `FREE_COOLING[row][col]` met
/// - `row` = index in [`WARMUP_HOURS_FREE`] (opwarmtijd 0,5..12 h);
/// - `col` = 12 kolommen in de volgorde
///   `[uren × luchtwisselingen × zwaarte]`:
///   `8/0,1/l, 8/0,1/z, 8/0,5/l, 8/0,5/z, 14/0,1/l, 14/0,1/z, 14/0,5/l,
///    14/0,5/z, 62/0,1/l, 62/0,1/z, 62/0,5/l, 62/0,5/z`.
///
/// `None` = `-` (niet gedefinieerd in de norm).
#[rustfmt::skip]
pub const FREE_COOLING: [[Option<f64>; 12]; 7] = [
    // opwarmtijd 0,5 h
    [Some(63.0), Some(16.0), Some(74.0), Some(26.0), Some(88.0), Some(38.0), Some(91.0), Some(56.0), Some(92.0), None,         Some(92.0), None        ],
    // opwarmtijd 1 h
    [Some(34.0), Some(10.0), Some(43.0), Some(16.0), Some(50.0), Some(29.0), Some(50.0), Some(43.0), Some(55.0), Some(100.0),  Some(55.0), None        ],
    // opwarmtijd 2 h
    [Some(14.0), Some(3.0),  Some(21.0), Some(8.0),  Some(28.0), Some(18.0), Some(28.0), Some(29.0), Some(32.0), Some(86.0),   Some(32.0), None        ],
    // opwarmtijd 3 h
    [Some(5.0),  Some(0.0),  Some(10.0), Some(2.0),  Some(17.0), Some(12.0), Some(18.0), Some(21.0), Some(23.0), Some(73.0),   Some(23.0), Some(94.0)  ],
    // opwarmtijd 4 h
    [Some(0.0),  Some(0.0),  Some(3.0),  Some(0.0),  Some(11.0), Some(7.0),  Some(12.0), Some(15.0), Some(17.0), Some(64.0),   Some(17.0), Some(84.0)  ],
    // opwarmtijd 6 h
    [Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(3.0),  Some(1.0),  Some(5.0),  Some(5.0),  Some(10.0), Some(52.0),   Some(10.0), Some(70.0)  ],
    // opwarmtijd 12 h
    [Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(0.0),  Some(2.0),  Some(31.0),   Some(2.0),  Some(45.0)  ],
];

/// Tabel 4.14 — specifieke toeslag φ_hu,i [W/m²] bij **beperkte** afkoeling.
/// ISSO 53 tabel 4.14, PDF p.53.
///
/// Indexering: `LIMITED_COOLING[row][col]` met
/// - `row` = index in [`WARMUP_HOURS_LIMITED`] (opwarmtijd 0,5..4 h);
/// - `col` = 20 kolommen in de volgorde `[graden × luchtwisselingen × zwaarte]`:
///   `1/0,1/l, 1/0,1/z, 1/0,5/l, 1/0,5/z, 2/0,1/l, 2/0,1/z, 2/0,5/l, 2/0,5/z,
///    3/0,1/l, 3/0,1/z, 3/0,5/l, 3/0,5/z, 4/0,1/l, 4/0,1/z, 4/0,5/l, 4/0,5/z,
///    5/0,1/l, 5/0,1/z, 5/0,5/l, 5/0,5/z`.
///
/// `None` = `-` (niet gedefinieerd; voor verlaging 5 °C zijn rijen 0,5 en 1 h
/// niet in de norm gegeven).
#[rustfmt::skip]
pub const LIMITED_COOLING: [[Option<f64>; 20]; 5] = [
    // opwarmtijd 0,5 h
    [Some(12.0), Some(12.0), Some(14.0), Some(18.0), Some(27.0), Some(28.0), Some(29.0), Some(35.0), Some(39.0), Some(44.0), Some(44.0), Some(53.0), Some(50.0), Some(60.0), Some(58.0), Some(69.0), None,        None,        None,        None       ],
    // opwarmtijd 1 h
    [Some(8.0),  Some(8.0),  Some(10.0), Some(14.0), Some(19.0), Some(21.0), Some(21.0), Some(28.0), Some(26.0), Some(34.0), Some(32.0), Some(43.0), Some(33.0), Some(48.0), Some(41.0), Some(56.0), None,        None,        None,        None       ],
    // opwarmtijd 2 h
    [Some(5.0),  Some(5.0),  Some(7.0),  Some(11.0), Some(10.0), Some(15.0), Some(13.0), Some(22.0), Some(15.0), Some(25.0), Some(21.0), Some(33.0), Some(20.0), Some(35.0), Some(28.0), Some(43.0), Some(43.0), Some(85.0), Some(47.0), Some(94.0) ],
    // opwarmtijd 3 h
    [Some(3.0),  Some(3.0),  Some(5.0),  Some(10.0), Some(7.0),  Some(12.0), Some(10.0), Some(19.0), Some(9.0),  Some(20.0), Some(15.0), Some(27.0), Some(14.0), Some(29.0), Some(21.0), Some(37.0), Some(33.0), Some(75.0), Some(37.0), Some(84.0) ],
    // opwarmtijd 4 h
    [Some(2.0),  Some(2.0),  Some(4.0),  Some(9.0),  Some(5.0),  Some(10.0), Some(8.0),  Some(17.0), Some(7.0),  Some(18.0), Some(13.0), Some(25.0), Some(10.0), Some(26.0), Some(17.0), Some(34.0), Some(28.0), Some(72.0), Some(31.0), Some(76.0) ],
];

/// Kolom-index in [`FREE_COOLING`] voor de gegeven combinatie.
/// Layout: 3 uren-blokken × {l,z per luchtw} → `(uren_idx*4) + air*2 + weight`.
fn free_col(setback_hours: u32, air: AirChanges, weight: BuildingWeight) -> Option<usize> {
    let hours_idx = SETBACK_HOURS_FREE.iter().position(|&h| h == setback_hours)?;
    let air_idx = match air {
        AirChanges::Low => 0,
        AirChanges::High => 1,
    };
    let weight_idx = match weight {
        BuildingWeight::Light => 0,
        BuildingWeight::Heavy => 1,
    };
    Some(hours_idx * 4 + air_idx * 2 + weight_idx)
}

/// Kolom-index in [`LIMITED_COOLING`] voor de gegeven combinatie.
/// Layout: 5 graden-blokken (1..5) × {l,z per luchtw} →
/// `((graden-1)*4) + air*2 + weight`.
fn limited_col(degrees: u32, air: AirChanges, weight: BuildingWeight) -> Option<usize> {
    if !(1..=5).contains(&degrees) {
        return None;
    }
    let degrees_idx = (degrees - 1) as usize;
    let air_idx = match air {
        AirChanges::Low => 0,
        AirChanges::High => 1,
    };
    let weight_idx = match weight {
        BuildingWeight::Light => 0,
        BuildingWeight::Heavy => 1,
    };
    Some(degrees_idx * 4 + air_idx * 2 + weight_idx)
}

/// Lineaire interpolatie + clamp van een kolom over de opwarmtijd-as.
///
/// `warmup_hours` wordt geclampt op `[rows[0], rows[last]]`. Tussen twee
/// gedefinieerde rijwaarden wordt lineair geïnterpoleerd. Als een van de
/// twee omliggende cellen `None` (`-`) is, valt de interpolatie terug op de
/// dichtstbijzijnde gedefinieerde cel (zie `nearest_defined`).
fn interpolate_column(
    rows: &[f64],
    column: impl Fn(usize) -> Option<f64>,
    warmup_hours: f64,
) -> Option<f64> {
    debug_assert!(!rows.is_empty());

    // Clamp onder/boven het tabelbereik.
    if warmup_hours <= rows[0] {
        return nearest_defined(rows, &column, 0, true);
    }
    let last = rows.len() - 1;
    if warmup_hours >= rows[last] {
        return nearest_defined(rows, &column, last, false);
    }

    // Vind het omhullende rij-paar [i, i+1].
    for i in 0..last {
        let (h0, h1) = (rows[i], rows[i + 1]);
        if warmup_hours >= h0 && warmup_hours <= h1 {
            let v0 = column(i);
            let v1 = column(i + 1);
            return match (v0, v1) {
                (Some(a), Some(b)) => {
                    let t = (warmup_hours - h0) / (h1 - h0);
                    Some(a + t * (b - a))
                }
                // Eén kant ongedefinieerd: gebruik de wel-gedefinieerde cel
                // (de norm definieert geen waarde voorbij die grens).
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
        }
    }
    None
}

/// Zoek vanaf `start` de dichtstbijzijnde gedefinieerde cel.
/// `search_forward`: bij clamp onder de tabel zoeken we naar voren (oplopende
/// opwarmtijd); bij clamp boven de tabel naar achteren.
fn nearest_defined(
    rows: &[f64],
    column: &impl Fn(usize) -> Option<f64>,
    start: usize,
    search_forward: bool,
) -> Option<f64> {
    if let Some(v) = column(start) {
        return Some(v);
    }
    if search_forward {
        ((start + 1)..rows.len()).find_map(&column)
    } else {
        (0..start).rev().find_map(&column)
    }
}

/// Lookup φ_hu,i [W/m²] bij **vrije** afkoeling (tabel 4.13).
///
/// Interpoleert lineair over de opwarmtijd-as en clampt op het tabelbereik
/// [0,5 h .. 12 h]. Retourneert `None` als de kolom-combinatie niet bestaat
/// of de gehele kolom ongedefinieerd is.
pub fn lookup_free_cooling(
    setback_hours: u32,
    air: AirChanges,
    weight: BuildingWeight,
    warmup_hours: f64,
) -> Option<f64> {
    let col = free_col(setback_hours, air, weight)?;
    interpolate_column(
        &WARMUP_HOURS_FREE,
        |row| FREE_COOLING[row][col],
        warmup_hours,
    )
}

/// Lookup φ_hu,i [W/m²] bij **beperkte** afkoeling (tabel 4.14).
///
/// Interpoleert lineair over de opwarmtijd-as en clampt op het tabelbereik
/// [0,5 h .. 4 h]. `degrees` ∈ {1..5}.
pub fn lookup_limited_cooling(
    degrees: u32,
    air: AirChanges,
    weight: BuildingWeight,
    warmup_hours: f64,
) -> Option<f64> {
    let col = limited_col(degrees, air, weight)?;
    interpolate_column(
        &WARMUP_HOURS_LIMITED,
        |row| LIMITED_COOLING[row][col],
        warmup_hours,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_from_c_eff_threshold() {
        // Enum c_eff-waarden: Licht=15, Gemiddeld=50, Zwaar=75.
        assert_eq!(BuildingWeight::from_c_eff(15.0), BuildingWeight::Light);
        assert_eq!(BuildingWeight::from_c_eff(50.0), BuildingWeight::Light);
        assert_eq!(BuildingWeight::from_c_eff(70.0), BuildingWeight::Light);
        assert_eq!(BuildingWeight::from_c_eff(75.0), BuildingWeight::Heavy);
    }

    #[test]
    fn free_cooling_exact_rows() {
        // Voorbeeld p.66: 14 uur, 0,1 luchtw, licht, 2 h opwarmen → 28 W/m².
        assert_eq!(
            lookup_free_cooling(14, AirChanges::Low, BuildingWeight::Light, 2.0),
            Some(28.0)
        );
        // Weekend (62 uur), 0,1 luchtw, licht, 4 h opwarmen → 17 W/m².
        assert_eq!(
            lookup_free_cooling(62, AirChanges::Low, BuildingWeight::Light, 4.0),
            Some(17.0)
        );
        // Hoek 0,5 h / 8 uur / 0,1 / licht → 63.
        assert_eq!(
            lookup_free_cooling(8, AirChanges::Low, BuildingWeight::Light, 0.5),
            Some(63.0)
        );
    }

    #[test]
    fn free_cooling_interpolation() {
        // Tussen 2 h (28) en 3 h (17) bij 14/0,1/l → op 2,5 h = 22,5.
        let v = lookup_free_cooling(14, AirChanges::Low, BuildingWeight::Light, 2.5).unwrap();
        assert!((v - 22.5).abs() < 1e-9, "got {v}");
    }

    #[test]
    fn free_cooling_clamp() {
        // Onder 0,5 h clampt naar rij 0,5.
        assert_eq!(
            lookup_free_cooling(14, AirChanges::Low, BuildingWeight::Light, 0.1),
            Some(88.0)
        );
        // Boven 12 h clampt naar rij 12.
        assert_eq!(
            lookup_free_cooling(14, AirChanges::Low, BuildingWeight::Light, 20.0),
            Some(0.0)
        );
    }

    #[test]
    fn limited_cooling_exact() {
        // 3 graden, 0,5 luchtw, zwaar, 2 h → kolom 3/0,5/z = 33.
        assert_eq!(
            lookup_limited_cooling(3, AirChanges::High, BuildingWeight::Heavy, 2.0),
            Some(33.0)
        );
        // 1 graad, 0,1, licht, 0,5 h → 12.
        assert_eq!(
            lookup_limited_cooling(1, AirChanges::Low, BuildingWeight::Light, 0.5),
            Some(12.0)
        );
    }

    #[test]
    fn limited_cooling_undefined_5deg_clamps() {
        // 5 graden bij opwarmtijd 0,5 h is `-` in de norm; clamp forward
        // naar de eerste gedefinieerde rij (2 h = 43 voor 5/0,1/l).
        assert_eq!(
            lookup_limited_cooling(5, AirChanges::Low, BuildingWeight::Light, 0.5),
            Some(43.0)
        );
    }

    #[test]
    fn invalid_degrees_returns_none() {
        assert_eq!(
            lookup_limited_cooling(6, AirChanges::Low, BuildingWeight::Light, 2.0),
            None
        );
        assert_eq!(
            lookup_limited_cooling(0, AirChanges::Low, BuildingWeight::Light, 2.0),
            None
        );
    }
}
