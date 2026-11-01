//! Calendar CRUD + visibility toggle. Seeding is done in `db::seed`.

use crate::error::{Error, Result};
use crate::models::{Calendar, CalendarInput};
use crate::time::now_rfc3339;
use rusqlite::Connection;

fn row_to_calendar(r: &rusqlite::Row<'_>) -> rusqlite::Result<Calendar> {
    Ok(Calendar {
        id: r.get("id")?,
        name: r.get("name")?,
        color: r.get("color")?,
        is_primary: r.get::<_, i64>("is_primary")? != 0,
        visible: r.get::<_, i64>("visible")? != 0,
        sort_order: r.get("sort_order")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

const SELECT: &str = "SELECT id, name, color, is_primary, visible, sort_order, created_at, updated_at FROM calendars";

/// List all calendars, ordered by sort_order then created_at.
pub fn list(conn: &Connection) -> Result<Vec<Calendar>> {
    let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY sort_order ASC, created_at ASC"))?;
    let rows = stmt.query_map([], row_to_calendar)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Fetch a single calendar by id.
pub fn get(conn: &Connection, id: &str) -> Result<Calendar> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], row_to_calendar)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("calendar {id}")),
            other => Error::Sql(other),
        })
}

/// Create a calendar with a stable sort_order after the current max.
pub fn create(conn: &Connection, input: CalendarInput) -> Result<Calendar> {
    let now = now_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let max: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM calendars",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO calendars (id, name, color, is_primary, visible, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, 1, ?4, ?5, ?5)",
        rusqlite::params![id, input.name, input.color, max, now],
    )?;
    get(conn, &id)
}

/// Set a calendar's visibility flag; returns () on success.
pub fn set_visible(conn: &Connection, id: &str, visible: bool) -> Result<()> {
    let affected = conn.execute(
        "UPDATE calendars SET visible = ?2, updated_at = ?3 WHERE id = ?1",
        rusqlite::params![id, visible as i64, now_rfc3339()],
    )?;
    if affected == 0 {
        return Err(Error::NotFound(format!("calendar {id}")));
    }
    Ok(())
}

/// True if a calendar exists and is visible.
pub fn is_visible(conn: &Connection, id: &str) -> Result<bool> {
    conn.query_row("SELECT visible FROM calendars WHERE id = ?1", [id], |r| {
        r.get::<_, i64>(0)
    })
    .map(|v| v != 0)
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("calendar {id}")),
        other => Error::Sql(other),
    })
}

// Update name/color if provided.
pub fn update(
    conn: &Connection,
    id: &str,
    name: Option<String>,
    color: Option<String>,
) -> Result<Calendar> {
    let c = get(conn, id)?;
    let name = name.unwrap_or(c.name);
    let color = color.unwrap_or(c.color);
    conn.execute(
        "UPDATE calendars SET name = ?2, color = ?3, updated_at = ?4 WHERE id = ?1",
        rusqlite::params![id, name, color, now_rfc3339()],
    )?;
    get(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> Connection {
        db::open_in_memory().unwrap()
    }

    #[test]
    fn primary_seeded() {
        let c = conn();
        let all = list(&c).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "My Calendar");
        assert!(all[0].is_primary);
    }

    #[test]
    fn add_calendar() {
        let c = conn();
        let cal = create(&c, CalendarInput { name: "Work".into(), color: "#039be5".into() }).unwrap();
        assert_eq!(cal.name, "Work");
        assert_eq!(cal.color, "#039be5");
        assert!(!cal.is_primary);
        assert!(cal.visible);
        assert_eq!(list(&c).unwrap().len(), 2);
    }

    #[test]
    fn set_visible_toggles() {
        let c = conn();
        let cal = create(&c, CalendarInput { name: "Work".into(), color: "#f00".into() }).unwrap();
        set_visible(&c, &cal.id, false).unwrap();
        assert!(!get(&c, &cal.id).unwrap().visible);
        set_visible(&c, &cal.id, true).unwrap();
        assert!(get(&c, &cal.id).unwrap().visible);
    }

    #[test]
    fn calendar_visibility_filters_events() {
        use crate::events;
        use crate::models::{EventInput, EventStatus};
        use chrono::{TimeZone, Utc};

        let c = conn();
        let primary = list(&c).unwrap().remove(0);
        let work = create(&c, CalendarInput { name: "Work".into(), color: "#f00".into() }).unwrap();

        let mk = |cal_id: &str, title: &str| EventInput {
            calendar_id: cal_id.into(),
            title: title.into(),
            all_day: false,
            start_utc: Some("2026-11-03T09:00:00+03:00".into()),
            end_utc: Some("2026-11-03T10:00:00+03:00".into()),
            tz_id: Some("Europe/Moscow".into()),
            start_date: None,
            end_date: None,
            description: String::new(),
            location: String::new(),
            reminder_minutes: None,
            status: EventStatus::Confirmed,
            rrule: None,
        };
        events::create_event(&c, mk(&primary.id, "On Primary")).unwrap();
        events::create_event(&c, mk(&work.id, "On Work")).unwrap();

        let start = Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).single().unwrap();
        let end = Utc.with_ymd_and_hms(2026, 11, 9, 0, 0, 0).single().unwrap();

        // Both visible -> both returned.
        let both = events::list_events_in_range(&c, start, end).unwrap();
        assert_eq!(both.len(), 2);
        // Hide Work -> only its calendar's events are excluded.
        set_visible(&c, &work.id, false).unwrap();
        let only_primary = events::list_events_in_range(&c, start, end).unwrap();
        assert_eq!(only_primary.len(), 1);
        assert_eq!(only_primary[0].title, "On Primary");
        // Shown again -> events come back.
        set_visible(&c, &work.id, true).unwrap();
        assert_eq!(events::list_events_in_range(&c, start, end).unwrap().len(), 2);
    }

    #[test]
    fn add_calendar_multiple_with_colors() {
        let c = conn();
        let a = create(&c, CalendarInput { name: "Work".into(), color: "#039be5".into() }).unwrap();
        let b = create(&c, CalendarInput { name: "Family".into(), color: "#ff8f00".into() }).unwrap();
        let all = list(&c).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(a.color, "#039be5");
        assert_eq!(b.color, "#ff8f00");
        // sort_order strictly increasing.
        assert!(all[0].sort_order < all[1].sort_order && all[1].sort_order < all[2].sort_order);
    }
}
