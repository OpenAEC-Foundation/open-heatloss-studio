//! Per-room assessment: combine RMOT, ATG bandwidth, TO-uren and GTO.

use crate::calc::csv::ParsedCsv;
use crate::calc::pmv::{pmv_for_operative, weighting_factor};
use crate::calc::rmot::rmot_per_day;
use crate::error::Result;
use crate::model::Isso74Config;
use crate::result::{
    AssumptionNotes, AtgPlotPoint, AtgResult, GtoResult, Isso74Result, ProjectSummary, RoomResult,
    ToHoursResult,
};
use crate::tables::atg_bounds;

/// PMV thresholds for the GTO summer/winter accounting (ISSO 74 Bijlage A.2).
const PMV_WARM_THRESHOLD: f64 = 0.5;
const PMV_COLD_THRESHOLD: f64 = -0.5;
/// TO-uren operative-temperature thresholds (ISSO 74 Bijlage A.1).
const TO_25: f64 = 25.0;
const TO_28: f64 = 28.0;

/// Run the full assessment over a parsed CSV with the given configuration.
pub fn assess(csv: &ParsedCsv, config: &Isso74Config) -> Result<Isso74Result> {
    let rmot = rmot_per_day(csv);

    let mut rooms = Vec::with_capacity(csv.room_names.len());
    for (room_idx, room_name) in csv.room_names.iter().enumerate() {
        let variant = config.variant_for(room_name);

        let mut assessed = 0u32;
        let mut over_upper = 0u32;
        let mut under_lower = 0u32;
        let mut to25 = 0u32;
        let mut to28 = 0u32;
        let mut gto_summer = 0.0f64;
        let mut gto_winter = 0.0f64;
        let mut plot = Vec::new();

        for rec in &csv.records {
            // Only usage hours count (all toetsen).
            if !config
                .usage_hours
                .is_in_use(rec.iso_weekday, rec.hour_of_day)
            {
                continue;
            }
            let theta_o = rec.theta_o[room_idx];

            // TO-uren are an absolute θ_o test, independent of the θ_rm band.
            if theta_o > TO_25 {
                to25 += 1;
            }
            if theta_o > TO_28 {
                to28 += 1;
            }

            // GTO weighting via Fanger PMV.
            let pmv = pmv_for_operative(theta_o, &config.pmv);
            if pmv > PMV_WARM_THRESHOLD {
                gto_summer += weighting_factor(pmv);
            } else if pmv < PMV_COLD_THRESHOLD {
                gto_winter += weighting_factor(pmv);
            }

            // ATG bandwidth — only when θ_rm is defined and within validity band.
            if let Some(&theta_rm) = rmot.get(&rec.day_index) {
                if let Some(bounds) = atg_bounds(theta_rm, config.comfort_class, variant) {
                    assessed += 1;
                    let is_over = theta_o > bounds.upper;
                    let is_under = theta_o < bounds.lower;
                    if is_over {
                        over_upper += 1;
                    }
                    if is_under {
                        under_lower += 1;
                    }
                    plot.push(AtgPlotPoint {
                        hour_of_year: rec.hour_of_year,
                        theta_rm,
                        theta_o,
                        lower: bounds.lower,
                        upper: bounds.upper,
                        over_upper: is_over,
                        under_lower: is_under,
                    });
                }
            }
        }

        let exceedance = if assessed > 0 {
            (over_upper + under_lower) as f64 / assessed as f64
        } else {
            0.0
        };
        let atg = AtgResult {
            variant,
            assessed_hours: assessed,
            hours_over_upper: over_upper,
            hours_under_lower: under_lower,
            exceedance_fraction: exceedance,
            passes: over_upper == 0 && under_lower == 0,
        };

        let to_hours = ToHoursResult {
            hours_over_25: to25,
            hours_over_28: to28,
            limit_25: config.to25_limit_hours,
            limit_28: config.to28_limit_hours,
            passes: (to25 as f64) <= config.to25_limit_hours
                && (to28 as f64) <= config.to28_limit_hours,
        };

        let gto = GtoResult {
            weighted_hours_summer: gto_summer,
            weighted_hours_winter: gto_winter,
            limit: config.gto_limit_hours,
            passes: gto_summer <= config.gto_limit_hours
                && gto_winter <= config.gto_limit_hours,
        };

        let passes = atg.passes && to_hours.passes && gto.passes;
        rooms.push(RoomResult {
            room: room_name.clone(),
            atg,
            to_hours,
            gto,
            passes,
            plot,
        });
    }

    let rooms_total = rooms.len() as u32;
    let rooms_passing = rooms.iter().filter(|r| r.passes).count() as u32;
    let summary = ProjectSummary {
        rooms_total,
        rooms_passing,
        rooms_failing: rooms_total - rooms_passing,
    };

    let assumptions = AssumptionNotes {
        pmv_basis:
            "Toets-laag (ISSO 74 §A): t_air ≈ t_mrt ≈ θ_o; geen volledige comfortsimulatie. \
             clo seizoensafhankelijk (zomer/winter), metabolisme + RH + luchtsnelheid configureerbaar."
                .to_string(),
        relative_humidity_pct: config.pmv.relative_humidity_pct,
        air_velocity_m_s: config.pmv.air_velocity_m_s,
        clo_summer: config.pmv.clo_summer,
        clo_winter: config.pmv.clo_winter,
        metabolic_rate_met: config.pmv.metabolic_rate_met,
    };

    Ok(Isso74Result {
        rooms,
        summary,
        assumptions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::csv::parse_csv;

    /// Build a CSV with `days` days, constant outdoor temp, and a per-hour θ_o.
    fn build_csv(days: u32, t_out: f64, theta_o: f64) -> String {
        let mut s = String::from("hour;T_buiten;Kantoor\n");
        let total = days * 24;
        for h in 1..=total {
            s.push_str(&format!("{h};{t_out};{theta_o}\n"));
        }
        s
    }

    #[test]
    fn comfortable_room_passes_all() {
        // 14 days, outdoor 18°C → θ_rm ≈ 18; θ_o 23°C lies inside bounds and
        // gives small PMV → no exceedances.
        let csv = parse_csv(&build_csv(14, 18.0, 23.0)).unwrap();
        let res = assess(&csv, &Isso74Config::default()).unwrap();
        let room = &res.rooms[0];
        assert!(room.atg.assessed_hours > 0);
        assert!(room.passes, "expected pass, got {:?}", room);
        assert_eq!(res.summary.rooms_passing, 1);
    }

    #[test]
    fn hot_room_fails_to_and_atg() {
        // θ_o 32°C every hour → TO>28 explodes, ATG upper exceeded.
        let csv = parse_csv(&build_csv(14, 18.0, 32.0)).unwrap();
        let res = assess(&csv, &Isso74Config::default()).unwrap();
        let room = &res.rooms[0];
        assert!(room.to_hours.hours_over_28 > 0);
        assert!(!room.to_hours.passes);
        assert!(!room.atg.passes);
        assert!(!room.passes);
    }

    #[test]
    fn only_usage_hours_counted() {
        // Default office hours 08-18 ma-vr = 10 h/day × 5/7 days.
        let csv = parse_csv(&build_csv(7, 18.0, 23.0)).unwrap();
        let res = assess(&csv, &Isso74Config::default()).unwrap();
        // Days 1..7 with day 1 = Monday → weekdays 1..5 are usage days.
        // But RMOT only defined from day 2 onwards (day 1 has no history).
        // Usage days with RMOT: Tue..Fri of week 1 = 4 days × 10 h = 40 assessed.
        assert_eq!(res.rooms[0].atg.assessed_hours, 40);
    }
}
