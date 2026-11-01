//! RRULE recurrence: parse/validate, DST-correct expansion, and helpers.
//!
//! The `rrule` crate parses a full iCalendar `RRULESET` string (a `DTSTART`
//! with `TZID` plus the `RRULE`). It expands occurrences in that IANA timezone
//! keeping the local wall clock constant ("5pm stays 5pm"), shifting UTC per
//! the DST offset. We parse the chain `DTSTART;TZID=<tz>:<wallclock>\nRRULE:<rrule>`
//! and re-localize each occurrence through chrono-tz for an authoritative value.

use crate::error::{Error, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};

/// Resolve an IANA timezone id (chrono-tz 0.10, our crate's chrono_tz).
pub fn tz_from_id(tz_id: &str) -> Result<chrono_tz::Tz> {
    tz_id
        .parse::<chrono_tz::Tz>()
        .map_err(|_| Error::Validation(format!("unknown IANA timezone '{tz_id}'")))
}

/// Build the iCalendar chain string fed to the rrule crate.
fn build_ics(rrule_str: &str, tz_id: &str, dt_start: DateTime<Utc>) -> Result<String> {
    let tz = tz_from_id(tz_id)?;
    let naive = dt_start.with_timezone(&tz).naive_local();
    let ds = naive.format("%Y%m%dT%H%M%S").to_string();
    Ok(format!("DTSTART;TZID={tz_id}:{ds}\nRRULE:{rrule_str}"))
}

fn parse_set(rrule_str: &str, tz_id: &str, dt_start: DateTime<Utc>) -> Result<rrule::RRuleSet> {
    let ics = build_ics(rrule_str, tz_id, dt_start)?;
    ics.parse::<rrule::RRuleSet>()
        .map_err(|e: rrule::RRuleError| {
            Error::Rrule(format!(
                "invalid RRULE '{rrule_str}' (tz {tz_id}, start {dt_start}): {e}"
            ))
        })
}

/// Re-localize an occurrence produced by the RRULE iterator into UTC.
/// `occ` is a `DateTime<rrule::Tz>`; we take its naive wall clock and resolve
/// the real offset in `tz` at that instant (handles skipped/ambiguous DST).
type Occ = chrono::DateTime<rrule::Tz>;

fn wall_clock_to_utc(occ: Occ, tz: chrono_tz::Tz) -> DateTime<Utc> {
    let naive = occ.naive_local();
    match tz.from_local_datetime(&naive).earliest() {
        Some(dt) => dt.with_timezone(&Utc),
        None => occ.with_timezone(&Utc),
    }
}

/// Parse + validate an RRULE string for an event with the given tz/start.
/// Malformed rules error.
pub fn parse_validate(rrule_str: &str, tz_id: &str, dt_start: DateTime<Utc>) -> Result<()> {
    parse_set(rrule_str, tz_id, dt_start).map(|_| ())
}

/// Expand all occurrences of a validated series (bounded by COUNT/UNTIL;
/// unbounded rules capped at 65535).
pub fn expand_all(
    rrule_str: &str,
    tz_id: &str,
    dt_start: DateTime<Utc>,
) -> Result<Vec<DateTime<Utc>>> {
    let set = parse_set(rrule_str, tz_id, dt_start)?;
    let tz = tz_from_id(tz_id)?;
    Ok(set
        .all(65535)
        .dates
        .into_iter()
        .map(|occ| wall_clock_to_utc(occ, tz))
        .collect())
}

/// Expand occurrences that could fall in [range_start, range_end). Bounds the
/// work with the set's after/before filters, then re-localizes; callers filter
/// precisely by overlap.
pub fn expand_range(
    rrule_str: &str,
    tz_id: &str,
    dt_start: DateTime<Utc>,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
) -> Result<Vec<DateTime<Utc>>> {
    let set = parse_set(rrule_str, tz_id, dt_start)?;
    let tz_rr = set.get_dt_start().timezone();
    let tz = tz_from_id(tz_id)?;
    let margin = Duration::days(1);
    let lo = (range_start - margin).with_timezone(&tz_rr);
    let hi = (range_end + margin).with_timezone(&tz_rr);
    let set = set.after(lo).before(hi);
    Ok(set
        .all(65535)
        .dates
        .into_iter()
        .map(|occ| wall_clock_to_utc(occ, tz))
        .collect())
}

/// Parse an RFC3339 UTC string into a `DateTime<Utc>`.
pub fn parse_utc(s: &str) -> Result<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::Validation(format!("invalid RFC3339 '{s}': {e}")))
}

/// Format a `DateTime<Utc>` as RFC3339.
pub fn fmt_utc(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// The floating local date (yyyy-mm-dd) on which a UTC instant falls, in `tz`.
pub fn local_date_of(utc: DateTime<Utc>, tz: chrono_tz::Tz) -> chrono::NaiveDate {
    utc.with_timezone(&tz).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn berlin() -> chrono_tz::Tz {
        "Europe/Berlin".parse().unwrap()
    }

    #[test]
    fn malformed_rrule_rejected() {
        let start = parse_utc("2026-03-27T16:00:00Z").unwrap();
        assert!(parse_validate("FREQ=FORNIGHTLY", "Europe/Berlin", start).is_err());
    }

    #[test]
    fn valid_rrule_accepted() {
        let start = parse_utc("2026-11-02T08:00:00Z").unwrap();
        parse_validate("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE", "Europe/Berlin", start).unwrap();
    }

    #[test]
    fn dst_wall_clock_preserved() {
        // Weekly Fri 17:00 Berlin, starts before spring-forward 2026-03-29.
        let start = parse_utc("2026-03-27T16:00:00Z").unwrap();
        let occs = expand_all("FREQ=WEEKLY", "Europe/Berlin", start).unwrap();
        assert_eq!(
            occs[0],
            parse_utc("2026-03-27T16:00:00Z").unwrap(),
            "17:00+01:00 Berlin = 16:00Z"
        );
        assert_eq!(
            occs[1],
            parse_utc("2026-04-03T15:00:00Z").unwrap(),
            "local 17:00 must be preserved after spring-forward (17:00+02:00 = 15:00Z)"
        );
    }

    #[test]
    fn weekly_expansion_mondays() {
        let start = parse_utc("2026-11-02T08:00:00Z").unwrap();
        let occs = expand_all("FREQ=WEEKLY;BYDAY=MO", "Europe/Berlin", start).unwrap();
        let dates: Vec<String> = occs
            .iter()
            .take(5)
            .map(|d| local_date_of(*d, berlin()).to_string())
            .collect();
        assert_eq!(
            dates,
            vec![
                "2026-11-02".to_string(),
                "2026-11-09".to_string(),
                "2026-11-16".to_string(),
                "2026-11-23".to_string(),
                "2026-11-30".to_string(),
            ]
        );
    }

    #[test]
    fn count_bounded_series() {
        let start = parse_utc("2026-11-02T08:00:00Z").unwrap();
        let occs = expand_all("FREQ=DAILY;COUNT=3", "Europe/Berlin", start).unwrap();
        assert_eq!(occs.len(), 3);
    }

    #[test]
    fn fixed_offset_fallback_constant_offset() {
        // "Etc/GMT-3" is a fixed UTC+3 zone; wall clock constant and offset fixed.
        let tz = tz_from_id("Etc/GMT-3").unwrap();
        // 17:00 local in Etc/GMT-3 == 14:00Z.
        let start = parse_utc("2026-03-27T14:00:00Z").unwrap();
        let occs = expand_all("FREQ=WEEKLY", "Etc/GMT-3", start).unwrap();
        assert_eq!(occs[0], parse_utc("2026-03-27T14:00:00Z").unwrap());
        assert_eq!(occs[1], parse_utc("2026-04-03T14:00:00Z").unwrap());
        // Wall clock in the zone stays 17:00 week to week.
        assert_eq!(
            occs[0].with_timezone(&tz).format("%H:%M").to_string(),
            "17:00"
        );
        assert_eq!(
            occs[1].with_timezone(&tz).format("%H:%M").to_string(),
            "17:00"
        );
    }
}
