//! Running mean outdoor temperature θ_rm — ISSO 74 Kader 3.2, formule 3.1.
//!
//! ```text
//! 7-day truncation (used here):
//!   θ_rm = 0,253 · {θ_d-1 + 0,8·θ_d-2 + 0,8²·θ_d-3 + … + 0,8⁶·θ_d-7}
//! ```
//!
//! * `θ_d-n` = etmaalgemiddelde buitentemperatuur n days back = ½·(dagmax +
//!   dagmin) of that calendar day (Kader 3.2 Opmerking, PDF p.55).
//! * RMOT is computed **per calendar day** and then applied to every hour of
//!   that day.
//!
//! For the first days of the dataset there are fewer than 7 preceding days; we
//! use the available history and re-normalise the weights so the coefficient
//! sum still equals 1 (documented toets-laag choice — avoids a cold-start bias
//! and is conservative for short simulation windows).

use crate::calc::csv::ParsedCsv;
use std::collections::BTreeMap;

/// Weight coefficient 0,8.
const ALPHA: f64 = 0.8;
/// Canonical normalisation factor for the 7-day truncation (ISSO 74 formule
/// 3.1): `1 / Σ_{k=0..6} 0,8^k = 1 / 3,95224 ≈ 0,253`. Kept for documentation
/// and as a cross-check; the implementation re-normalises by the actual weight
/// sum (which equals this for a full 7-day history).
#[allow(dead_code)]
const NORM_7DAY: f64 = 0.253;

/// Per-day mean outdoor temperature θ_d = ½(dagmax + dagmin).
fn daily_means(csv: &ParsedCsv) -> BTreeMap<u32, f64> {
    let mut min: BTreeMap<u32, f64> = BTreeMap::new();
    let mut max: BTreeMap<u32, f64> = BTreeMap::new();
    for r in &csv.records {
        min.entry(r.day_index)
            .and_modify(|v| *v = v.min(r.t_outdoor))
            .or_insert(r.t_outdoor);
        max.entry(r.day_index)
            .and_modify(|v| *v = v.max(r.t_outdoor))
            .or_insert(r.t_outdoor);
    }
    min.into_iter()
        .map(|(day, mn)| {
            let mx = max[&day];
            (day, 0.5 * (mn + mx))
        })
        .collect()
}

/// Compute θ_rm per calendar day index.
///
/// Returns a map `day_index → θ_rm`. The RMOT of a day uses the **preceding**
/// days (d-1 … d-7); a day with no preceding history has no defined θ_rm and is
/// omitted from the map (its hours are excluded from the assessment).
pub fn rmot_per_day(csv: &ParsedCsv) -> BTreeMap<u32, f64> {
    let means = daily_means(csv);
    let days: Vec<u32> = means.keys().copied().collect();
    let mut out = BTreeMap::new();

    for &day in &days {
        let mut weighted = 0.0;
        let mut weight_sum = 0.0;
        for k in 0..7u32 {
            let prev = match day.checked_sub(k + 1) {
                Some(d) if d >= 1 => d,
                _ => break,
            };
            if let Some(&mean) = means.get(&prev) {
                let w = ALPHA.powi(k as i32);
                weighted += w * mean;
                weight_sum += w;
            } else {
                // Gap in the day series — stop walking further back.
                break;
            }
        }
        if weight_sum > 0.0 {
            // Re-normalise by the actual partial weight sum. With a full 7-day
            // history weight_sum = Σ_{k=0..6} 0,8^k = 3,95224, so this is exactly
            // the canonical θ_rm = 0,253·{…} (see [`NORM_7DAY`]); with fewer
            // preceding days the weights are renormalised to sum to 1.
            out.insert(day, weighted / weight_sum);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::csv::{HourRecord, ParsedCsv};

    fn day_records(day: u32, weekday: u8, tmin: f64, tmax: f64) -> Vec<HourRecord> {
        // Two records per day carrying the min and max.
        vec![
            HourRecord {
                hour_of_year: (day - 1) * 24 + 1,
                iso_weekday: weekday,
                day_index: day,
                hour_of_day: 6,
                t_outdoor: tmin,
                theta_o: vec![22.0],
            },
            HourRecord {
                hour_of_year: (day - 1) * 24 + 15,
                iso_weekday: weekday,
                day_index: day,
                hour_of_day: 14,
                t_outdoor: tmax,
                theta_o: vec![22.0],
            },
        ]
    }

    #[test]
    fn constant_temperature_gives_same_rmot() {
        // Every day θ_d = 20 → θ_rm should also converge to 20.
        let mut records = Vec::new();
        for day in 1..=10u32 {
            records.extend(day_records(day, ((day - 1) % 7 + 1) as u8, 15.0, 25.0));
        }
        let csv = ParsedCsv {
            room_names: vec!["R".to_string()],
            records,
        };
        let rmot = rmot_per_day(&csv);
        // Day 8 has full 7-day history of θ_d = 20.
        let v = rmot[&8];
        assert!((v - 20.0).abs() < 1e-6, "expected ~20, got {v}");
    }

    #[test]
    fn known_seven_day_series() {
        // θ_d-1..7 = [20,19,18,17,16,15,14]; θ_rm = 0,253·Σ 0,8^k·θ.
        let temps = [14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0]; // day1..day7
        let mut records = Vec::new();
        for (i, &t) in temps.iter().enumerate() {
            let day = (i + 1) as u32;
            records.extend(day_records(day, 1, t, t)); // min=max=t
        }
        // Day 8 reads back day7..day1 as d-1..d-7 = [20,19,18,17,16,15,14].
        records.extend(day_records(8, 1, 30.0, 30.0));
        let csv = ParsedCsv {
            room_names: vec!["R".to_string()],
            records,
        };
        let rmot = rmot_per_day(&csv);
        // Exact normalisation: Σ 0,8^k·θ / Σ 0,8^k (= canonical 0,253·{…}).
        let weighted = 20.0
            + 0.8 * 19.0
            + 0.8f64.powi(2) * 18.0
            + 0.8f64.powi(3) * 17.0
            + 0.8f64.powi(4) * 16.0
            + 0.8f64.powi(5) * 15.0
            + 0.8f64.powi(6) * 14.0;
        let weight_sum: f64 = (0..7).map(|k| 0.8f64.powi(k)).sum();
        let expected = weighted / weight_sum;
        assert!((rmot[&8] - expected).abs() < 1e-6, "got {}, expected {expected}", rmot[&8]);
    }

    #[test]
    fn first_day_has_no_rmot() {
        let csv = ParsedCsv {
            room_names: vec!["R".to_string()],
            records: day_records(1, 1, 10.0, 20.0),
        };
        let rmot = rmot_per_day(&csv);
        assert!(!rmot.contains_key(&1));
    }
}
