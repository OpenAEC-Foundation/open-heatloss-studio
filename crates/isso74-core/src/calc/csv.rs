//! CSV parser for the simulation-independent hourly input.
//!
//! # Column contract
//!
//! * **Header row required.**
//! * **Column 1** = `hour` (hour-of-year, 1..=8760) **or** an ISO-8601
//!   datetime (`YYYY-MM-DDTHH:MM` / `YYYY-MM-DD HH:MM`). The header name of
//!   column 1 is free (`hour`, `time`, `datetime`, …).
//! * A column named (case-insensitive) `T_buiten` / `t_buiten` / `T_outdoor`
//!   = outdoor air temperature [°C]. Required.
//! * Every remaining column = operative temperature θ_o [°C] for one room;
//!   the header is the room name.
//!
//! # Robustness
//! * Separator: `;` or `,` (auto-detected from the header row — the separator
//!   that yields the most columns wins; ties favour `;` for NL Excel).
//! * Decimal: `.` or `,`. When the separator is `,` the decimal is `.`; when
//!   the separator is `;` a `,` inside a value is treated as decimal comma.
//! * Blank lines are skipped. Leading/trailing whitespace is trimmed.

use crate::error::{Isso74Error, Result};

/// One parsed hourly record.
#[derive(Debug, Clone)]
pub struct HourRecord {
    /// Hour-of-year, 1-based (1..=8760, or more for leap-aware sets).
    pub hour_of_year: u32,
    /// ISO weekday (1 = Mon … 7 = Sun).
    pub iso_weekday: u8,
    /// Calendar day index (1-based) within the dataset.
    pub day_index: u32,
    /// Hour-of-day (0..=23).
    pub hour_of_day: u8,
    /// Outdoor air temperature [°C].
    pub t_outdoor: f64,
    /// Operative temperature θ_o per room, aligned to [`ParsedCsv::room_names`].
    pub theta_o: Vec<f64>,
}

/// Result of parsing the CSV.
#[derive(Debug, Clone)]
pub struct ParsedCsv {
    /// Room names in column order (matches `theta_o` indexing).
    pub room_names: Vec<String>,
    /// Hourly records in file order.
    pub records: Vec<HourRecord>,
}

const OUTDOOR_HEADERS: &[&str] = &["t_buiten", "t_outdoor", "buiten", "outdoor", "te"];

/// Detect the separator by counting columns for each candidate on the header.
fn detect_separator(header: &str) -> char {
    let semi = header.matches(';').count();
    let comma = header.matches(',').count();
    // `;` favoured on tie (NL Excel default).
    if semi >= comma && semi > 0 {
        ';'
    } else if comma > 0 {
        ','
    } else {
        ';'
    }
}

/// Split a line on `sep`, trimming each cell.
fn split_line(line: &str, sep: char) -> Vec<String> {
    line.split(sep).map(|c| c.trim().to_string()).collect()
}

/// Parse a numeric cell tolerating a decimal comma (only meaningful when the
/// field separator is `;`).
fn parse_num(cell: &str, sep: char, field: &str) -> Result<f64> {
    let normalized = if sep == ';' {
        cell.replace(',', ".")
    } else {
        cell.to_string()
    };
    normalized
        .parse::<f64>()
        .map_err(|_| Isso74Error::CsvParse(format!("could not parse '{cell}' as number in column '{field}'")))
}

/// Derive (hour_of_year, weekday, day_index, hour_of_day) from the first cell.
///
/// Supports a bare hour-of-year integer (1-based) OR an ISO datetime. For the
/// hour-of-year form we assume a non-leap reference year starting on a Monday
/// for weekday derivation — documented as a toets-laag convenience. For ISO
/// datetimes we compute the weekday from the date.
fn parse_time(cell: &str, row_index: usize) -> Result<(u32, u8, u32, u8)> {
    // Try ISO datetime first (contains '-' and ':' or 'T').
    if cell.contains('-') && (cell.contains(':') || cell.contains('T')) {
        return parse_iso_datetime(cell);
    }
    // Otherwise treat as hour-of-year (1-based).
    let hoy: u32 = cell
        .parse()
        .map_err(|_| Isso74Error::CsvParse(format!("row {}: '{cell}' is neither hour-of-year nor ISO datetime", row_index + 1)))?;
    if hoy == 0 {
        return Err(Isso74Error::CsvParse("hour-of-year is 1-based; got 0".to_string()));
    }
    let zero = hoy - 1;
    let hour_of_day = (zero % 24) as u8;
    let day_index = zero / 24 + 1;
    // Reference year assumed to start on Monday (ISO weekday 1).
    let iso_weekday = ((day_index - 1) % 7 + 1) as u8;
    Ok((hoy, iso_weekday, day_index, hour_of_day))
}

/// Minimal ISO-8601 (`YYYY-MM-DD[ T]HH[:MM]`) parser → (hoy, weekday, day, hod).
fn parse_iso_datetime(cell: &str) -> Result<(u32, u8, u32, u8)> {
    let norm = cell.replace('T', " ");
    let mut parts = norm.split_whitespace();
    let date = parts
        .next()
        .ok_or_else(|| Isso74Error::CsvParse(format!("invalid datetime '{cell}'")))?;
    let time = parts.next().unwrap_or("00:00");

    let mut d = date.split('-');
    let y: i64 = d.next().and_then(|s| s.parse().ok()).ok_or_else(|| Isso74Error::CsvParse(format!("invalid year in '{cell}'")))?;
    let m: u32 = d.next().and_then(|s| s.parse().ok()).ok_or_else(|| Isso74Error::CsvParse(format!("invalid month in '{cell}'")))?;
    let day: u32 = d.next().and_then(|s| s.parse().ok()).ok_or_else(|| Isso74Error::CsvParse(format!("invalid day in '{cell}'")))?;
    let hour_of_day: u8 = time.split(':').next().and_then(|s| s.parse().ok()).ok_or_else(|| Isso74Error::CsvParse(format!("invalid hour in '{cell}'")))?;

    // Day-of-year.
    let doy = day_of_year(y, m, day)?;
    let hoy = (doy - 1) * 24 + hour_of_day as u32 + 1;
    // Weekday via a proleptic Gregorian algorithm (Sakamoto), 0=Sun → map to ISO.
    let w = sakamoto_weekday(y, m, day);
    let iso_weekday = if w == 0 { 7 } else { w } as u8;
    Ok((hoy, iso_weekday, doy, hour_of_day))
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn day_of_year(y: i64, m: u32, d: u32) -> Result<u32> {
    if !(1..=12).contains(&m) || d == 0 || d > 31 {
        return Err(Isso74Error::CsvParse(format!("invalid date {y}-{m}-{d}")));
    }
    let mut days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap(y) {
        days[1] = 29;
    }
    let mut doy = d;
    for month in &days[..(m as usize - 1)] {
        doy += month;
    }
    Ok(doy)
}

/// Sakamoto's algorithm: returns 0=Sunday … 6=Saturday.
fn sakamoto_weekday(y: i64, m: u32, d: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let w = (yy + yy / 4 - yy / 100 + yy / 400 + t[(m - 1) as usize] as i64 + d as i64) % 7;
    ((w % 7 + 7) % 7) as u32
}

/// Parse the full CSV into a [`ParsedCsv`].
pub fn parse_csv(content: &str) -> Result<ParsedCsv> {
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| Isso74Error::CsvParse("empty CSV (no header row)".to_string()))?;
    let sep = detect_separator(header);
    let headers = split_line(header, sep);
    if headers.len() < 3 {
        return Err(Isso74Error::CsvParse(format!(
            "expected at least 3 columns (time, T_buiten, ≥1 room); got {} with separator '{sep}'",
            headers.len()
        )));
    }

    // Find the outdoor-temperature column index (search columns 1..n; col 0 is time).
    let outdoor_idx = headers
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, h)| {
            let lh = h.to_lowercase();
            OUTDOOR_HEADERS.iter().any(|o| lh == *o)
        })
        .map(|(i, _)| i)
        .ok_or_else(|| {
            Isso74Error::MissingParameter(format!(
                "no outdoor temperature column (expected one of {OUTDOOR_HEADERS:?})"
            ))
        })?;

    // Room columns = all columns except time (0) and outdoor.
    let mut room_cols: Vec<usize> = Vec::new();
    let mut room_names: Vec<String> = Vec::new();
    for (i, h) in headers.iter().enumerate() {
        if i == 0 || i == outdoor_idx {
            continue;
        }
        room_cols.push(i);
        room_names.push(h.clone());
    }
    if room_names.is_empty() {
        return Err(Isso74Error::MissingParameter(
            "no room (θ_o) columns found".to_string(),
        ));
    }

    let ncols = headers.len();
    let mut records = Vec::new();
    for (row_index, line) in lines.enumerate() {
        let cells = split_line(line, sep);
        if cells.len() != ncols {
            return Err(Isso74Error::CsvParse(format!(
                "row {} has {} cells, expected {ncols}",
                row_index + 2,
                cells.len()
            )));
        }
        let (hour_of_year, iso_weekday, day_index, hour_of_day) =
            parse_time(&cells[0], row_index)?;
        let t_outdoor = parse_num(&cells[outdoor_idx], sep, &headers[outdoor_idx])?;
        let mut theta_o = Vec::with_capacity(room_cols.len());
        for &c in &room_cols {
            theta_o.push(parse_num(&cells[c], sep, &headers[c])?);
        }
        records.push(HourRecord {
            hour_of_year,
            iso_weekday,
            day_index,
            hour_of_day,
            t_outdoor,
            theta_o,
        });
    }

    if records.is_empty() {
        return Err(Isso74Error::CsvParse("CSV has a header but no data rows".to_string()));
    }

    Ok(ParsedCsv {
        room_names,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_semicolon_decimal_comma() {
        let csv = "hour;T_buiten;Kantoor 1\n1;10,5;22,3\n2;11,0;23,1\n";
        let p = parse_csv(csv).unwrap();
        assert_eq!(p.room_names, vec!["Kantoor 1"]);
        assert_eq!(p.records.len(), 2);
        assert!((p.records[0].t_outdoor - 10.5).abs() < 1e-9);
        assert!((p.records[0].theta_o[0] - 22.3).abs() < 1e-9);
    }

    #[test]
    fn parses_comma_separator_dot_decimal() {
        let csv = "hour,T_buiten,Room A,Room B\n1,10.5,22.0,23.0\n";
        let p = parse_csv(csv).unwrap();
        assert_eq!(p.room_names, vec!["Room A", "Room B"]);
        assert!((p.records[0].theta_o[1] - 23.0).abs() < 1e-9);
    }

    #[test]
    fn hour_of_year_to_weekday_day_hour() {
        // hour 1 → day 1, hour-of-day 0, weekday Monday (1).
        let csv = "hour;T_buiten;R\n1;5;20\n25;5;20\n";
        let p = parse_csv(csv).unwrap();
        assert_eq!(p.records[0].hour_of_day, 0);
        assert_eq!(p.records[0].day_index, 1);
        assert_eq!(p.records[0].iso_weekday, 1);
        // hour 25 → day 2, hour-of-day 0, weekday Tuesday (2).
        assert_eq!(p.records[1].day_index, 2);
        assert_eq!(p.records[1].iso_weekday, 2);
    }

    #[test]
    fn iso_datetime_weekday() {
        // 2021-07-01 is a Thursday (ISO 4).
        let csv = "datetime;T_buiten;R\n2021-07-01T12:00;25;28\n";
        let p = parse_csv(csv).unwrap();
        assert_eq!(p.records[0].iso_weekday, 4);
        assert_eq!(p.records[0].hour_of_day, 12);
    }

    #[test]
    fn rejects_missing_outdoor_column() {
        let csv = "hour;Room A;Room B\n1;22;23\n";
        assert!(parse_csv(csv).is_err());
    }
}
