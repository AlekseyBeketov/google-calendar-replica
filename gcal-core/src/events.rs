//! Event CRUD, range listing (with recurrence expansion + overrides) and
//! series-aware update/delete (this / this-and-following / all).

use crate::error::{Error, Result};
use crate::models::{
    Calendar, Event, EventInput, EventInstance, EventPatch, EventStatus, Scope,
};
use crate::recurrence::{self};
use crate::time::now_rfc3339;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;

const EVENT_COLS: &str =
    "id, calendar_id, series_id, title, all_day, start_date, end_date, start_utc, end_utc, \
     tz_id, rrule, description, location, reminder_minutes, status, created_at, updated_at";

fn row_to_event(r: &rusqlite::Row<'_>) -> rusqlite::Result<Event> {
    let status: String = r.get("status")?;
    let status = match status.as_str() {
        "tentative" => EventStatus::Tentative,
        "cancelled" => EventStatus::Cancelled,
        _ => EventStatus::Confirmed,
    };
    Ok(Event {
        id: r.get("id")?,
        calendar_id: r.get("calendar_id")?,
        series_id: r.get("series_id")?,
        title: r.get("title")?,
        all_day: r.get::<_, i64>("all_day")? != 0,
        start_date: r.get("start_date")?,
        end_date: r.get("end_date")?,
        start_utc: r.get("start_utc")?,
        end_utc: r.get("end_utc")?,
        tz_id: r.get("tz_id")?,
        rrule: r.get("rrule")?,
        description: r.get("description")?,
        location: r.get("location")?,
        reminder_minutes: r.get("reminder_minutes")?,
        status,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

/// Validate a timed input: require start/end/utc and end > start.
fn validate_timed(input: &EventInput) -> Result<()> {
    let (s, e) = (input.start_utc.as_ref(), input.end_utc.as_ref());
    match (s, e) {
        (Some(s), Some(e)) => {
            if recurrence::parse_utc(e)? <= recurrence::parse_utc(s)? {
                return Err(Error::Validation(
                    "event end must be after start".into(),
                ));
            }
            Ok(())
        }
        _ => Err(Error::Validation(
            "timed events require start_utc and end_utc".into(),
        )),
    }
}

/// Validate an all-day input: require dates and end > start.
fn validate_allday(input: &EventInput) -> Result<()> {
    let (s, e) = (input.start_date.as_ref(), input.end_date.as_ref());
    match (s, e) {
        (Some(s), Some(e)) => {
            let sd = parse_date(s)?;
            let ed = parse_date(e)?;
            if ed <= sd {
                return Err(Error::Validation(
                    "all-day end date must be after start date".into(),
                ));
            }
            Ok(())
        }
        _ => Err(Error::Validation(
            "all-day events require start_date and end_date".into(),
        )),
    }
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| Error::Validation(format!("invalid date '{s}': {e}")))
}

/// Validate an input (timed vs. all-day) and canonicalize for storage.
fn validate_input(input: &EventInput) -> Result<()> {
    if input.all_day {
        validate_allday(input)?;
    } else {
        validate_timed(input)?;
    }
    // Validate any RRULE in the event's own timezone.
    if let Some(rrule) = &input.rrule {
        if rrule.trim().is_empty() {
            return Err(Error::Validation("empty rrule".into()));
        }
        let start = event_start_utc(input)?;
        let tz = input.tz_id.as_deref().unwrap_or("UTC");
        recurrence::parse_validate(rrule, tz, start)?;
    }
    Ok(())
}

/// The event's start as a UTC instant (all-day uses 00:00 of start_date).
fn event_start_utc(input: &EventInput) -> Result<DateTime<Utc>> {
    if input.all_day {
        let d = parse_date(input.start_date.as_deref().unwrap())?;
        Ok(d.and_hms_opt(0, 0, 0).unwrap().and_utc())
    } else {
        recurrence::parse_utc(input.start_utc.as_deref().unwrap())
    }
}

/// Create a single event (recurring or not).
pub fn create_event(conn: &Connection, input: EventInput) -> Result<Event> {
    // calendar must exist
    crate::calendars::get(conn, &input.calendar_id)?;
    validate_input(&input)?;
    let now = now_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let series_id = if input.rrule.is_some() {
        Some(id.clone())
    } else {
        None
    };
    conn.execute(
        "INSERT INTO events
           (id, calendar_id, series_id, title, all_day, start_date, end_date,
            start_utc, end_utc, tz_id, rrule, description, location,
            reminder_minutes, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
        rusqlite::params![
            id,
            input.calendar_id,
            series_id,
            input.title,
            input.all_day as i64,
            input.start_date,
            input.end_date,
            input.start_utc,
            input.end_utc,
            input.tz_id,
            input.rrule,
            input.description,
            input.location,
            input.reminder_minutes,
            status_to_str(input.status),
            now,
        ],
    )?;
    require_event(conn, &id)
}

fn status_to_str(s: EventStatus) -> &'static str {
    match s {
        EventStatus::Confirmed => "confirmed",
        EventStatus::Tentative => "tentative",
        EventStatus::Cancelled => "cancelled",
    }
}

/// Fetch a single event by id.
pub fn get_event(conn: &Connection, id: &str) -> Option<Event> {
    conn.query_row(
        &format!("SELECT {EVENT_COLS} FROM events WHERE id = ?1"),
        [id],
        row_to_event,
    )
    .ok()
}

/// Like get_event but errors on missing.
pub fn require_event(conn: &Connection, id: &str) -> Result<Event> {
    get_event(conn, id).ok_or_else(|| Error::NotFound(format!("event {id}")))
}

/// Duration of a timed event in seconds (end - start).
pub fn event_duration_secs(event: &Event) -> Result<i64> {
    let s = recurrence::parse_utc(event.start_utc.as_ref().unwrap())?;
    let e = recurrence::parse_utc(event.end_utc.as_ref().unwrap())?;
    Ok(e.signed_duration_since(s).num_seconds())
}

/// Apply one half of an EventPatch to an existing Event (used for overrides and
/// whole-series edits). Missing Option fields keep current values.
fn apply_patch(event: &mut Event, patch: &EventPatch) {
    if let Some(v) = &patch.calendar_id {
        event.calendar_id = v.clone();
    }
    if let Some(v) = &patch.title {
        event.title = v.clone();
    }
    if let Some(v) = &patch.all_day {
        event.all_day = *v;
    }
    if let Some(v) = &patch.start_utc {
        event.start_utc = Some(v.clone());
    }
    if let Some(v) = &patch.end_utc {
        event.end_utc = Some(v.clone());
    }
    if let Some(v) = &patch.tz_id {
        event.tz_id = Some(v.clone());
    }
    if let Some(v) = &patch.start_date {
        event.start_date = Some(v.clone());
    }
    if let Some(v) = &patch.end_date {
        event.end_date = Some(v.clone());
    }
    if let Some(v) = &patch.rrule {
        event.rrule = Some(v.clone());
    }
    if let Some(v) = &patch.description {
        event.description = v.clone();
    }
    if let Some(v) = &patch.location {
        event.location = v.clone();
    }
    if let Some(v) = patch.reminder_minutes {
        event.reminder_minutes = Some(v);
    }
    if let Some(v) = &patch.status {
        event.status = *v;
    }
}

/// Persist an Event's mutable fields back to the row (bumps updated_at).
fn persist_event(conn: &Connection, event: &Event) -> Result<()> {
    conn.execute(
        "UPDATE events SET calendar_id=?1, title=?2, all_day=?3, start_date=?4, end_date=?5,
         start_utc=?6, end_utc=?7, tz_id=?8, rrule=?9, description=?10, location=?11,
         reminder_minutes=?12, status=?13, updated_at=?14 WHERE id=?15",
        rusqlite::params![
            event.calendar_id,
            event.title,
            event.all_day as i64,
            event.start_date,
            event.end_date,
            event.start_utc,
            event.end_utc,
            event.tz_id,
            event.rrule,
            event.description,
            event.location,
            event.reminder_minutes,
            status_to_str(event.status),
            now_rfc3339(),
            event.id,
        ],
    )?;
    Ok(())
}

/// Fetch overrides for an event, keyed by occurrence date.
type OverrideRow = (String, String, Option<String>); // occurrence_day, action, patch_json

fn fetch_overrides(conn: &Connection, event_id: &str) -> Result<Vec<OverrideRow>> {
    let mut stmt = conn.prepare(
        "SELECT occurrence_day, action, patch_json FROM event_overrides WHERE event_id = ?1",
    )?;
    let rows = stmt.query_map([event_id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// After a whole-series title change, drop the `title` key from any stored
/// per-occurrence override patches so the new master title shows everywhere.
fn strip_title_from_override_patches(conn: &Connection, event_id: &str) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, patch_json FROM event_overrides WHERE event_id = ?1 AND action = 'update' AND patch_json IS NOT NULL",
    )?;
    let rows = stmt.query_map([event_id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    let mut targets: Vec<(String, String)> = Vec::new();
    for r in rows {
        targets.push(r?);
    }
    for (oid, pj) in targets {
        if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&pj) {
            if let Some(obj) = v.as_object_mut() {
                if obj.remove("title").is_some() {
                    if let Ok(s) = serde_json::to_string(&v) {
                        conn.execute(
                            "UPDATE event_overrides SET patch_json = ?2 WHERE id = ?1",
                            rusqlite::params![oid, s],
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Look up the occurrence (UTC start) of a series whose local date is
/// `occurrence_date` in `tz`. Returns the master event and occurrence UTC.
pub fn find_occurrence(
    _conn: &Connection,
    event: &Event,
    occurrence_date: NaiveDate,
) -> Result<DateTime<Utc>> {
    let tz = recurrence::tz_from_id(event.tz_id.as_deref().ok_or_else(|| {
        Error::Validation("cannot target an occurrence of an all-day/series without tz".into())
    })?)?;
    let start_utc = recurrence::parse_utc(event.start_utc.as_ref().unwrap())?;
    // Wide range around the target date so no occurrence is missed.
    let lo = tz
        .from_local_datetime(&occurrence_date.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .unwrap()
        .with_timezone(&Utc)
        - chrono::Duration::days(2);
    let hi = lo + chrono::Duration::days(20);
    let occs = recurrence::expand_range(
        event.rrule.as_deref().unwrap(),
        event.tz_id.as_deref().unwrap(),
        start_utc,
        lo,
        hi,
    )?;
    occs.into_iter()
        .find(|utc| recurrence::local_date_of(*utc, tz) == occurrence_date)
        .ok_or_else(|| {
            Error::NotFound(format!(
                "occurrence {occurrence_date} of event {}",
                event.id
            ))
        })
}

/// Delete an event (or occurrence/series). `occurrence_date` is required for
/// scope = This / ThisAndFollowing on recurring events.
pub fn delete_event(
    conn: &Connection,
    id: &str,
    scope: Scope,
    occurrence_date: Option<NaiveDate>,
) -> Result<()> {
    let event = require_event(conn, id)?;
    match scope {
        Scope::All => {
            conn.execute("DELETE FROM events WHERE id = ?1", [id])?;
            Ok(())
        }
        Scope::This => {
            let occ_date = occurrence_date.ok_or_else(|| {
                Error::Validation("occurrence_date required for 'this' delete".into())
            })?;
            if event.rrule.is_none() {
                // A non-recurring event: deleting "this" is just deleting it.
                conn.execute("DELETE FROM events WHERE id = ?1", [id])?;
                return Ok(());
            }
            // Verify the occurrence exists, then record a delete override.
            find_occurrence(conn, &event, occ_date)?;
            conn.execute(
                "INSERT INTO event_overrides (id, event_id, occurrence_day, action, patch_json)
                 VALUES (?1, ?2, ?3, 'delete', NULL)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    id,
                    occ_date.format("%Y-%m-%d").to_string(),
                ],
            )?;
            Ok(())
        }
        Scope::ThisAndFollowing => {
            let occ_date = occurrence_date.ok_or_else(|| {
                Error::Validation("occurrence_date required for 'this_and_following' delete".into())
            })?;
            if event.rrule.is_none() {
                conn.execute("DELETE FROM events WHERE id = ?1", [id])?;
                return Ok(());
            }
            // End the master series just before the occurrence (UNTIL is inclusive):
            // occurrence - 1s. Then create no new series (whole tail is gone).
            let occ = find_occurrence(conn, &event, occ_date)?;
            let cut = occ - chrono::Duration::seconds(1);
            let until = cut.format("%Y%m%dT%H%M%SZ").to_string();
            let new_rrule = with_until(event.rrule.as_deref().unwrap_or(""), &until, true)?;
            conn.execute(
                "UPDATE events SET rrule = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id, new_rrule, now_rfc3339()],
            )?;
            Ok(())
        }
    }
}

/// Append `UNTIL=<utc YYYYMMDDTHHMMSSZ>` to an RRULE string, or replace an
/// existing UNTIL. Returns the new RRULE string.
fn with_until(rrule: &str, until_utc: &str, _fresh: bool) -> Result<String> {
    // Strip any existing UNTIL segment, then append ours.
    let cleaned: Vec<String> = rrule
        .split(';')
        .filter(|part| !part.starts_with("UNTIL"))
        .map(|s| s.to_string())
        .collect();
    if cleaned.is_empty() {
        return Err(Error::Rrule("cannot set UNTIL on empty RRULE".into()));
    }
    let mut parts = cleaned.join(";");
    parts.push_str(&format!(";UNTIL={until_utc}"));
    Ok(parts)
}

/// Update an event (or occurrence/series). Mode semantics per scope on
/// recurring events; `occurrence_date` required for This / ThisAndFollowing.
pub fn update_event(
    conn: &Connection,
    id: &str,
    patch: EventPatch,
    scope: Scope,
    occurrence_date: Option<NaiveDate>,
) -> Result<Event> {
    let mut event = require_event(conn, id)?;
    match scope {
        Scope::All => {
            // Title changes on the master must propagate to per-occurrence
            // exceptions so every occurrence reflects the new series title.
            let title_changed = patch.title.is_some();
            apply_patch(&mut event, &patch);
            if !event.all_day {
                validate_field_set(&event, "start_utc")?;
                validate_field_set(&event, "end_utc")?;
                if recurrence::parse_utc(event.end_utc.as_deref().unwrap())?
                    <= recurrence::parse_utc(event.start_utc.as_deref().unwrap())?
                {
                    return Err(Error::Validation("event end must be after start".into()));
                }
            }
            persist_event(conn, &event)?;
            if title_changed {
                strip_title_from_override_patches(conn, id)?;
            }
            Ok(event)
        }
        Scope::This => {
            let occ_date = occurrence_date.ok_or_else(|| {
                Error::Validation("occurrence_date required for 'this' update".into())
            })?;
            if event.rrule.is_none() {
                // Not recurring: behave as 'all'.
                apply_patch(&mut event, &patch);
                persist_event(conn, &event)?;
                return Ok(event);
            }
            find_occurrence(conn, &event, occ_date)?;
            let patch_json = serde_json::to_string(&patch)
                .map_err(|e| Error::Internal(format!("serialize patch: {e}")))?;
            // Upsert the override for that occurrence day.
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM event_overrides WHERE event_id = ?1 AND occurrence_day = ?2",
                    rusqlite::params![id, occ_date.format("%Y-%m-%d").to_string()],
                    |r| r.get(0),
                )
                .ok();
            match existing {
                Some(oid) => {
                    conn.execute(
                        "UPDATE event_overrides SET action='update', patch_json=?3 WHERE id=?1",
                        rusqlite::params![oid, occ_date.format("%Y-%m-%d").to_string(), patch_json],
                    )?;
                }
                None => {
                    conn.execute(
                        "INSERT INTO event_overrides (id, event_id, occurrence_day, action, patch_json)
                         VALUES (?1, ?2, ?3, 'update', ?4)",
                        rusqlite::params![
                            uuid::Uuid::new_v4().to_string(),
                            id,
                            occ_date.format("%Y-%m-%d").to_string(),
                            patch_json,
                        ],
                    )?;
                }
            }
            Ok(event)
        }
        Scope::ThisAndFollowing => {
            let occ_date = occurrence_date.ok_or_else(|| {
                Error::Validation("occurrence_date required for 'this_and_following' update".into())
            })?;
            if event.rrule.is_none() {
                apply_patch(&mut event, &patch);
                persist_event(conn, &event)?;
                return Ok(event);
            }
            // Base duration of the series.
            let dur_secs = event_duration_secs(&event)?;
            let occ = find_occurrence(conn, &event, occ_date)?;
            // 1. Cut original series just before the occurrence (UNTIL inclusive).
            let cut = occ - chrono::Duration::seconds(1);
            let until = cut.format("%Y%m%dT%H%M%SZ").to_string();
            let new_rrule = with_until(event.rrule.as_deref().unwrap_or(""), &until, true)?;
            conn.execute(
                "UPDATE events SET rrule = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id, new_rrule, now_rfc3339()],
            )?;
            // 2. Determine the new occurrence start/end (patched or original).
            let occ_start = patch
                .start_utc
                .as_deref()
                .map(recurrence::parse_utc)
                .transpose()?
                .unwrap_or(occ);
            let occ_end = if let Some(e) = patch.end_utc.as_deref() {
                recurrence::parse_utc(e)?
            } else {
                occ_start + chrono::Duration::seconds(dur_secs)
            };
            // 3. Create a new master starting at the (movable) occurrence with the
            //    same series identity and RRULE.
            let master = EventInput {
                calendar_id: event.calendar_id.clone(),
                title: patch.title.clone().unwrap_or_else(|| event.title.clone()),
                all_day: event.all_day,
                start_utc: Some(occ_start.to_rfc3339()),
                end_utc: Some(occ_end.to_rfc3339()),
                tz_id: patch.tz_id.clone().or_else(|| event.tz_id.clone()),
                start_date: if event.all_day {
                    Some(occ_date.format("%Y-%m-%d").to_string())
                } else {
                    None
                },
                end_date: if event.all_day {
                    Some(
                        (occ_date + chrono::Days::new(1))
                            .format("%Y-%m-%d")
                            .to_string(),
                    )
                } else {
                    None
                },
                description: event.description.clone(),
                location: event.location.clone(),
                reminder_minutes: event.reminder_minutes,
                status: event.status,
                rrule: event.rrule.clone(),
            };
            // New master carries over the original's series_id so the series is
            // split across two master rows but shares series identity.
            let new_event = create_event(conn, master)?;
            // Rewire series_id to the shared series id.
            let series_id = event.series_id.as_deref().unwrap_or(&event.id).to_string();
            conn.execute(
                "UPDATE events SET series_id = ?2 WHERE id = ?1",
                rusqlite::params![new_event.id, series_id],
            )?;
            Ok(require_event(conn, &new_event.id)?)
        }
    }
}

fn validate_field_set(e: &Event, field: &str) -> Result<()> {
    match field {
        "start_utc" if e.start_utc.is_none() => {
            Err(Error::Validation("timed event requires start_utc".into()))
        }
        "end_utc" if e.end_utc.is_none() => {
            Err(Error::Validation("timed event requires end_utc".into()))
        }
        _ => Ok(()),
    }
}

/// List event instances whose [start,end) overlaps `[range_start, range_end)`.
/// Recurring series are expanded; overrides applied; hidden calendars excluded.
pub fn list_events_in_range(
    conn: &Connection,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
) -> Result<Vec<EventInstance>> {
    // Visible calendars only.
    let visible: Vec<Calendar> = crate::calendars::list(conn)?
        .into_iter()
        .filter(|c| c.visible)
        .collect();

    let mut out = Vec::new();
    for cal in &visible {
        // Non-recurring timed events overlapping [start,end).
        let mut stmt = conn.prepare(
            "SELECT * FROM events WHERE calendar_id = ?1 AND rrule IS NULL AND all_day = 0 AND \
             start_utc < ?3 AND end_utc > ?2",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![cal.id, range_start.to_rfc3339(), range_end.to_rfc3339()],
            row_to_event,
        )?;
        for r in rows {
            let e = r?;
            if let Some(inst) = timed_instance(&e, &cal.color, false, false) {
                out.push(inst);
            }
        }

        // Non-recurring all-day events overlapping [start_date, end_date).
        let mut stmt = conn.prepare(
            "SELECT * FROM events WHERE calendar_id = ?1 AND rrule IS NULL AND all_day = 1 AND \
             start_date < ?3 AND end_date > ?2",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![
                cal.id,
                range_start.date_naive().format("%Y-%m-%d").to_string(),
                range_end.date_naive().format("%Y-%m-%d").to_string()
            ],
            row_to_event,
        )?;
        for r in rows {
            let e = r?;
            if let Some(inst) = allday_instance(&e, &cal.color, false) {
                out.push(inst);
            }
        }

        // Recurring series: expand each event's occurrences, apply overrides.
        let mut stmt = conn.prepare(
            "SELECT * FROM events WHERE calendar_id = ?1 AND rrule IS NOT NULL",
        )?;
        let rows = stmt.query_map([&cal.id], row_to_event)?;
        for r in rows {
            let master = r?;
            expand_series_instances(conn, &master, &cal.color, range_start, range_end, &mut out)?;
        }
    }

    out.sort_by(|a, b| a.start.cmp(&b.start));
    Ok(out)
}

/// Expanded instances for a recurring master in the given range with overrides.
fn expand_series_instances(
    conn: &Connection,
    master: &Event,
    color: &str,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    out: &mut Vec<EventInstance>,
) -> Result<()> {
    if master.all_day {
        return expand_series_allday(conn, master, color, range_start, range_end, out);
    }
    let tz = recurrence::tz_from_id(master.tz_id.as_deref().unwrap())?;
    let start_utc = recurrence::parse_utc(master.start_utc.as_deref().unwrap())?;
    let dur = Duration::seconds(event_duration_secs(master)?);
    // Precise overlap: occ_start < range_end && occ_start + dur > range_start.
    let candidates = recurrence::expand_range(
        master.rrule.as_deref().unwrap(),
        master.tz_id.as_deref().unwrap(),
        start_utc,
        range_start - dur,
        range_end,
    )?;
    let overrides = fetch_overrides(conn, &master.id)?;

    // Apply 'update' patches once per occurrence date.
    for occ in candidates {
        let occ_start = occ;
        let occ_end = occ_start + dur;
        if !(occ_start < range_end && occ_end > range_start) {
            continue;
        }
        let local_date = recurrence::local_date_of(occ_start, tz);
        let day = local_date.format("%Y-%m-%d").to_string();

        // Build the base instance.
        let mut title = master.title.clone();
        let mut start = occ_start;
        let mut end = occ_end;
        let mut cancelled = false;
        let mut is_exception = false;

        for (oid, action, patch_json) in &overrides {
            if *oid != day {
                continue;
            }
            match action.as_str() {
                "delete" => {
                    cancelled = true;
                }
                "update" => {
                    if let Some(pj) = patch_json {
                        if let Ok(patch) = serde_json::from_str::<EventPatch>(pj) {
                            // Apply patch: title + start/end (absolute edits).
                            if let Some(t) = &patch.title {
                                title = t.clone();
                            }
                            if let (Some(s), Some(e)) = (&patch.start_utc, &patch.end_utc) {
                                if let (Ok(ns), Ok(ne)) =
                                    (recurrence::parse_utc(s), recurrence::parse_utc(e))
                                {
                                    start = ns;
                                    end = ne;
                                }
                            }
                            is_exception = true;
                        }
                    }
                }
                _ => {}
            }
        }

        if cancelled {
            continue;
        }

        out.push(EventInstance {
            event_id: master.id.clone(),
            series_id: master.series_id.clone(),
            occurrence_date: day,
            start: start.to_rfc3339(),
            end: end.to_rfc3339(),
            all_day: false,
            title,
            calendar_id: master.calendar_id.clone(),
            color: color.to_string(),
            is_exception,
            cancelled: false,
        });
    }
    Ok(())
}

/// All-day recurring series overlap by [start_date,end_date) dates.
fn expand_series_allday(
    conn: &Connection,
    master: &Event,
    color: &str,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    out: &mut Vec<EventInstance>,
) -> Result<()> {
    // All-day recurrence anchors at the floating start_date at 00:00.
    let master_date = master
        .start_date
        .as_deref()
        .map(parse_date)
        .transpose()?
        .unwrap();
    let anchor_utc = Utc
        .with_ymd_and_hms(master_date.year(), master_date.month(), master_date.day(), 0, 0, 0)
        .single()
        .unwrap();
    let tz: Tz = "UTC".parse().unwrap();
    let start_naive = range_start.date_naive();
    let end_naive = range_end.date_naive();
    let lo = Utc
        .with_ymd_and_hms(start_naive.year(), start_naive.month(), start_naive.day(), 0, 0, 0)
        .single()
        .unwrap()
        - chrono::Duration::days(400);
    let hi = Utc
        .with_ymd_and_hms(end_naive.year(), end_naive.month(), end_naive.day(), 0, 0, 0)
        .single()
        .unwrap()
        + chrono::Duration::days(400);
    let candidates = recurrence::expand_range(master.rrule.as_deref().unwrap(), "UTC", anchor_utc, lo, hi)?;
    let overrides = fetch_overrides(conn, &master.id)?;
    for occ in candidates {
        let day = recurrence::local_date_of(occ, tz);
        if day >= end_naive || day < start_naive {
            continue;
        }
        let day_s = day.format("%Y-%m-%d").to_string();
        let mut title = master.title.clone();
        let mut start = day;
        let mut end = day + chrono::Days::new(1);
        let mut cancelled = false;
        for (oid, action, patch_json) in &overrides {
            if *oid != day_s {
                continue;
            }
            match action.as_str() {
                "delete" => cancelled = true,
                "update" => {
                    if let Some(pj) = patch_json {
                        if let Ok(patch) = serde_json::from_str::<EventPatch>(pj) {
                            if let Some(t) = &patch.title {
                                title = t.clone();
                            }
                            if let (Some(s), Some(e)) = (&patch.start_date, &patch.end_date) {
                                if let (Ok(sd), Ok(ed)) = (parse_date(s), parse_date(e)) {
                                    start = sd;
                                    end = ed;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if cancelled {
            continue;
        }
        out.push(EventInstance {
            event_id: master.id.clone(),
            series_id: master.series_id.clone(),
            occurrence_date: day_s,
            start: start.format("%Y-%m-%d").to_string(),
            end: end.format("%Y-%m-%d").to_string(),
            all_day: true,
            title,
            calendar_id: master.calendar_id.clone(),
            color: color.to_string(),
            is_exception: false,
            cancelled: false,
        });
    }
    Ok(())
}

fn timed_instance(
    e: &Event,
    color: &str,
    is_exception: bool,
    cancelled: bool,
) -> Option<EventInstance> {
    if e.status == EventStatus::Cancelled {
        return None;
    }
    let tz = e
        .tz_id
        .as_deref()
        .and_then(|t| t.parse::<Tz>().ok())
        .unwrap_or_else(|| "UTC".parse().unwrap());
    let start = recurrence::parse_utc(e.start_utc.as_deref()?).ok()?;
    let end = recurrence::parse_utc(e.end_utc.as_deref()?).ok()?;
    let day = recurrence::local_date_of(start, tz).format("%Y-%m-%d").to_string();
    Some(EventInstance {
        event_id: e.id.clone(),
        series_id: e.series_id.clone(),
        occurrence_date: day,
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        all_day: false,
        title: e.title.clone(),
        calendar_id: e.calendar_id.clone(),
        color: color.to_string(),
        is_exception,
        cancelled,
    })
}

fn allday_instance(e: &Event, color: &str, is_exception: bool) -> Option<EventInstance> {
    let start = e.start_date.as_deref()?.to_string();
    let end = e.end_date.as_deref()?.to_string();
    Some(EventInstance {
        event_id: e.id.clone(),
        series_id: e.series_id.clone(),
        occurrence_date: start.clone(),
        start,
        end,
        all_day: true,
        title: e.title.clone(),
        calendar_id: e.calendar_id.clone(),
        color: color.to_string(),
        is_exception,
        cancelled: false,
    })
}

/// Get a single event plus its series' expanded metadata (API `get_event`).
pub fn get_event_detail(conn: &Connection, id: &str) -> Result<EventDetail> {
    let e = require_event(conn, id)?;
    Ok(EventDetail {
        event: e.clone(),
        series_id: e.series_id.clone(),
        is_series: e.rrule.is_some(),
    })
}

/// Lightweight extended detail for get_event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventDetail {
    pub event: Event,
    pub series_id: Option<String>,
    pub is_series: bool,
}

use chrono::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendars;
    use crate::db;
    use chrono::NaiveDate;

    fn conn() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn primary_id(c: &Connection) -> String {
        calendars::list(c).unwrap().remove(0).id
    }

    fn utc(s: &str) -> DateTime<Utc> {
        recurrence::parse_utc(s).unwrap()
    }

    fn timed(
        cal: &str,
        title: &str,
        start: &str,
        end: &str,
        rrule: Option<&str>,
    ) -> EventInput {
        EventInput {
            calendar_id: cal.into(),
            title: title.into(),
            all_day: false,
            start_utc: Some(start.into()),
            end_utc: Some(end.into()),
            tz_id: Some("Europe/Moscow".into()),
            start_date: None,
            end_date: None,
            description: String::new(),
            location: String::new(),
            reminder_minutes: None,
            status: EventStatus::Confirmed,
            rrule: rrule.map(|s| s.to_string()),
        }
    }

    fn allday(cal: &str, title: &str, start: &str, end: &str) -> EventInput {
        EventInput {
            calendar_id: cal.into(),
            title: title.into(),
            all_day: true,
            start_utc: None,
            end_utc: None,
            tz_id: None,
            start_date: Some(start.into()),
            end_date: Some(end.into()),
            description: String::new(),
            location: String::new(),
            reminder_minutes: None,
            status: EventStatus::Confirmed,
            rrule: None,
        }
    }

    fn day(s: &str) -> NaiveDate {
        parse_date(s).unwrap()
    }

    #[test]
    fn event_end_before_start() {
        let c = conn();
        let cal = primary_id(&c);
        let bad = timed(&cal, "Bad", "2026-11-03T10:00:00+03:00", "2026-11-03T09:00:00+03:00", None);
        assert!(create_event(&c, bad).is_err());
        let n: i64 = c.query_row("SELECT count(*) FROM events", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "nothing stored for rejected mutation");
    }

    #[test]
    fn create_timed_event_moscow_utc() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(&c, timed(&cal, "Standup", "2026-11-03T09:00:00+03:00", "2026-11-03T09:30:00+03:00", None)).unwrap();
        assert_eq!(e.title, "Standup");
        assert_eq!(e.tz_id.as_deref(), Some("Europe/Moscow"));
        // UTC normalization: 09:00+03:00 = 06:00Z.
        let start = recurrence::parse_utc(e.start_utc.as_deref().unwrap()).unwrap();
        assert_eq!(start, utc("2026-11-03T06:00:00Z"));
        let end = recurrence::parse_utc(e.end_utc.as_deref().unwrap()).unwrap();
        assert_eq!(end, utc("2026-11-03T06:30:00Z"));
    }

    #[test]
    fn create_all_day_exclusive_end() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(&c, allday(&cal, "Holiday", "2026-11-03", "2026-11-04")).unwrap();
        assert_eq!(e.start_date.as_deref(), Some("2026-11-03"));
        assert_eq!(e.end_date.as_deref(), Some("2026-11-04"));
        // Returned for the day it spans (query the full day cell).
        let insts = list_events_in_range(&c, utc("2026-11-03T00:00:00Z"), utc("2026-11-04T00:00:00Z")).unwrap();
        assert_eq!(insts.len(), 1);
        assert!(insts[0].all_day);
        assert_eq!(insts[0].occurrence_date, "2026-11-03");
    }

    #[test]
    fn multi_day_all_day_stays_floating() {
        let c = conn();
        let cal = primary_id(&c);
        create_event(&c, allday(&cal, "Trip", "2026-11-03", "2026-11-06")).unwrap();
        // Any UTC range inside the dates returns it unchanged.
        let insts = list_events_in_range(&c, utc("2026-11-04T02:00:00Z"), utc("2026-11-05T22:00:00Z")).unwrap();
        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].start, "2026-11-03");
        assert_eq!(insts[0].end, "2026-11-06");
    }

    #[test]
    fn edit_event_title_and_delete() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(&c, timed(&cal, "Old", "2026-11-03T10:00:00+03:00", "2026-11-03T11:00:00+03:00", None)).unwrap();
        let patched = update_event(
            &c,
            &e.id,
            EventPatch { title: Some("New".into()), ..Default::default() },
            Scope::All,
            None,
        )
        .unwrap();
        assert_eq!(patched.title, "New");
        let range = (utc("2026-11-02T00:00:00Z"), utc("2026-11-09T00:00:00Z"));
        let insts = list_events_in_range(&c, range.0, range.1).unwrap();
        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].title, "New");
        // Delete, then it disappears.
        delete_event(&c, &e.id, Scope::All, None).unwrap();
        let insts = list_events_in_range(&c, range.0, range.1).unwrap();
        assert!(insts.is_empty());
    }

    #[test]
    fn range_query_overlap() {
        let c = conn();
        let cal = primary_id(&c);
        create_event(&c, timed(&cal, "Standup", "2026-11-03T09:00:00+03:00", "2026-11-03T09:30:00+03:00", None)).unwrap();
        create_event(&c, allday(&cal, "Holiday", "2026-11-04", "2026-11-05")).unwrap();
        // Query the week 2026-11-02..2026-11-09.
        let insts = list_events_in_range(&c, utc("2026-11-02T00:00:00Z"), utc("2026-11-09T00:00:00Z")).unwrap();
        assert_eq!(insts.len(), 2, "both overlapping timed and all-day returned");
    }

    #[test]
    fn weekly_series_expands_in_month() {
        let c = conn();
        let cal = primary_id(&c);
        // Weekly Mondays 09:00 Moscow starting 2026-11-02 (06:00Z).
        let e = create_event(
            &c,
            timed(&cal, "Standup", "2026-11-02T09:00:00+03:00", "2026-11-02T09:30:00+03:00", Some("FREQ=WEEKLY;BYDAY=MO")),
        )
        .unwrap();
        let insts = list_events_in_range(&c, utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z")).unwrap();
        let mondays: Vec<String> = insts.iter().map(|i| i.occurrence_date.clone()).collect();
        assert_eq!(
            mondays,
            vec![
                "2026-11-02".to_string(),
                "2026-11-09".to_string(),
                "2026-11-16".to_string(),
                "2026-11-23".to_string(),
                "2026-11-30".to_string(),
            ]
        );
        assert!(insts.iter().all(|i| i.event_id == e.id));
    }

    #[test]
    fn delete_single_occurrence_via_override() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(
            &c,
            timed(&cal, "Standup", "2026-11-02T09:00:00+03:00", "2026-11-02T09:30:00+03:00", Some("FREQ=WEEKLY;BYDAY=MO")),
        )
        .unwrap();
        // Delete the third occurrence (2026-11-16) with scope this.
        delete_event(&c, &e.id, Scope::This, Some(day("2026-11-16"))).unwrap();
        let insts = list_events_in_range(&c, utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z")).unwrap();
        let mondays: Vec<String> = insts.iter().map(|i| i.occurrence_date.clone()).collect();
        assert_eq!(
            mondays,
            vec![
                "2026-11-02".to_string(),
                "2026-11-09".to_string(),
                "2026-11-23".to_string(),
                "2026-11-30".to_string(),
            ],
            "third occurrence omitted, first two and fourth onwards kept"
        );
    }

    #[test]
    fn this_and_following_splits_series() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(
            &c,
            timed(&cal, "Standup", "2026-11-02T09:00:00+03:00", "2026-11-02T09:30:00+03:00", Some("FREQ=WEEKLY;BYDAY=MO")),
        )
        .unwrap();
        // Move the 2026-11-16 occurrence with scope this_and_following to 12:00.
        let r = update_event(
            &c,
            &e.id,
            EventPatch {
                start_utc: Some("2026-11-16T12:00:00+03:00".into()),
                end_utc: Some("2026-11-16T12:30:00+03:00".into()),
                ..Default::default()
            },
            Scope::ThisAndFollowing,
            Some(day("2026-11-16")),
        )
        .unwrap();
        // Original series now ends before 2026-11-16 (first two occurrences).
        let insts = list_events_in_range(&c, utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z")).unwrap();
        let days: Vec<String> = insts.iter().map(|i| i.occurrence_date.clone()).collect();
        assert!(days.contains(&"2026-11-02".to_string()), "old series keeps earlier occurrences");
        assert!(days.contains(&"2026-11-09".to_string()), "old series lasts through the occurrence before the cut");
        // New series begins at the moved occurrence (11-16, moved to 12:00 local = 09:00Z).
        let moved = insts.iter().find(|i| i.occurrence_date == "2026-11-16").expect("new series starts at 11-16");
        assert_eq!(utc(&moved.start), utc("2026-11-16T09:00:00Z"), "moved occurrence start applied on the new master");
        assert!(days.contains(&"2026-11-23".to_string()) && days.contains(&"2026-11-30".to_string()), "new series continues");
        assert!(insts.iter().all(|i| i.title == "Standup"));
        let _ = r;
    }

    #[test]
    fn whole_series_edit_propagates() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(
            &c,
            timed(&cal, "Standup", "2026-11-02T09:00:00+03:00", "2026-11-02T09:30:00+03:00", Some("FREQ=WEEKLY;BYDAY=MO")),
        )
        .unwrap();
        // Rename the whole series.
        update_event(&c, &e.id, EventPatch { title: Some("Daily Sync".into()), ..Default::default() }, Scope::All, None).unwrap();
        let insts = list_events_in_range(&c, utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z")).unwrap();
        assert!(insts.len() >= 4);
        assert!(insts.iter().all(|i| i.title == "Daily Sync"), "every occurrence reflects the new title");
    }

    #[test]
    fn cancelled_event_hidden() {
        let c = conn();
        let cal = primary_id(&c);
        let e = create_event(&c, timed(&cal, "Maybe", "2026-11-03T10:00:00+03:00", "2026-11-03T11:00:00+03:00", None)).unwrap();
        update_event(&c, &e.id, EventPatch { status: Some(EventStatus::Cancelled), ..Default::default() }, Scope::All, None).unwrap();
        let insts = list_events_in_range(&c, utc("2026-11-02T00:00:00Z"), utc("2026-11-09T00:00:00Z")).unwrap();
        assert!(insts.is_empty());
    }
}
