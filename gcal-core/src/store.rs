//! AppStore: a crate-pure façade over the SQLite connection exposing the API
//! surface in `replica/architecture.md`. The Tauri shell holds one of these and
//! maps its methods onto invoke commands; no Tauri types appear here.

use crate::calendars;
use crate::events;
use crate::models::{
    Calendar, CalendarInput, Event, EventInput, EventInstance, EventPatch, Scope, Task,
    TaskInput, TaskPatch,
};
use crate::tasks;
use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

/// Owns the shared SQLite connection and exposes domain operations.
pub struct AppStore {
    conn: Connection,
}

impl AppStore {
    /// Open (or create/migrate/seed) a database file.
    pub fn open(path: &Path) -> Result<Self> {
        Ok(AppStore {
            conn: crate::db::open_db(path)?,
        })
    }

    /// Open an in-memory database (used by tests and the Tauri shell in dev).
    pub fn open_in_memory() -> Result<Self> {
        Ok(AppStore {
            conn: crate::db::open_in_memory()?,
        })
    }

    /// Borrow the raw connection (for queries not covered by the façade).
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // ---- calendars ----
    pub fn get_calendars(&self) -> Result<Vec<Calendar>> {
        calendars::list(&self.conn)
    }

    pub fn add_calendar(&self, input: CalendarInput) -> Result<Calendar> {
        calendars::create(&self.conn, input)
    }

    pub fn set_calendar_visible(&self, id: &str, visible: bool) -> Result<()> {
        calendars::set_visible(&self.conn, id, visible)
    }

    // ---- events ----
    pub fn list_events_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<EventInstance>> {
        events::list_events_in_range(&self.conn, start, end)
    }

    pub fn get_event(&self, id: &str) -> Result<events::EventDetail> {
        events::get_event_detail(&self.conn, id)
    }

    pub fn create_event(&self, input: EventInput) -> Result<Event> {
        events::create_event(&self.conn, input)
    }

    pub fn update_event(
        &self,
        id: &str,
        patch: EventPatch,
        scope: Scope,
        occurrence_date: Option<chrono::NaiveDate>,
    ) -> Result<Event> {
        events::update_event(&self.conn, id, patch, scope, occurrence_date)
    }

    pub fn delete_event(
        &self,
        id: &str,
        scope: Scope,
        occurrence_date: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        events::delete_event(&self.conn, id, scope, occurrence_date)
    }

    // ---- tasks ----
    pub fn list_tasks(&self, only_active: bool) -> Result<Vec<Task>> {
        tasks::list(&self.conn, only_active)
    }

    pub fn create_task(&self, input: TaskInput) -> Result<Task> {
        tasks::create(&self.conn, input)
    }

    pub fn update_task(&self, id: &str, patch: TaskPatch) -> Result<Task> {
        tasks::update(&self.conn, id, patch)
    }

    pub fn set_task_completed(&self, id: &str, completed: bool) -> Result<Task> {
        tasks::set_completed(&self.conn, id, completed)
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        tasks::delete(&self.conn, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EventInput, EventStatus};

    #[test]
    fn facade_end_to_end() {
        let store = AppStore::open_in_memory().unwrap();
        // calendars
        let cals = store.get_calendars().unwrap();
        assert_eq!(cals.len(), 1);
        assert!(cals[0].is_primary);
        let work = store
            .add_calendar(CalendarInput { name: "Work".into(), color: "#f00".into() })
            .unwrap();
        store.set_calendar_visible(&work.id, true).unwrap();

        // events
        let ev = store
            .create_event(EventInput {
                calendar_id: work.id.clone(),
                title: "Standup".into(),
                all_day: false,
                start_utc: Some("2026-11-03T09:00:00+03:00".into()),
                end_utc: Some("2026-11-03T09:30:00+03:00".into()),
                tz_id: Some("Europe/Moscow".into()),
                start_date: None,
                end_date: None,
                description: String::new(),
                location: String::new(),
                reminder_minutes: None,
                status: EventStatus::Confirmed,
                rrule: None,
            })
            .unwrap();
        let detail = store.get_event(&ev.id).unwrap();
        assert_eq!(detail.event.title, "Standup");
        assert!(!detail.is_series);

        let start = chrono::DateTime::parse_from_rfc3339("2026-11-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let end = chrono::DateTime::parse_from_rfc3339("2026-11-09T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let insts = store.list_events_range(start, end).unwrap();
        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].color, "#f00", "event renders in calendar color");

        // tasks
        let t = store
            .create_task(TaskInput { title: "Ship".into(), notes: None, due_date: Some("2026-11-10".into()) })
            .unwrap();
        store.set_task_completed(&t.id, true).unwrap();
        let done = store.list_tasks(true).unwrap();
        assert!(done.iter().all(|x| x.status == crate::models::TaskStatus::NeedsAction), "completed tasks excluded from active list");
    }

    #[test]
    fn facade_delete_event_scopes() {
        let store = AppStore::open_in_memory().unwrap();
        let cal = store.get_calendars().unwrap().remove(0);
        let ev = store
            .create_event(EventInput {
                calendar_id: cal.id.clone(),
                title: "Weekly".into(),
                all_day: false,
                start_utc: Some("2026-11-02T09:00:00+03:00".into()),
                end_utc: Some("2026-11-02T09:30:00+03:00".into()),
                tz_id: Some("Europe/Moscow".into()),
                start_date: None,
                end_date: None,
                description: String::new(),
                location: String::new(),
                reminder_minutes: None,
                status: EventStatus::Confirmed,
                rrule: Some("FREQ=WEEKLY;BYDAY=MO".into()),
            })
            .unwrap();
        let start = "2026-11-01T00:00:00Z";
        let end = "2026-12-01T00:00:00Z";
        let lo = chrono::DateTime::parse_from_rfc3339(start).unwrap().with_timezone(&Utc);
        let hi = chrono::DateTime::parse_from_rfc3339(end).unwrap().with_timezone(&Utc);
        assert_eq!(store.list_events_range(lo, hi).unwrap().len(), 5);
        let occ = chrono::NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        store
            .delete_event(&ev.id, Scope::This, Some(occ))
            .unwrap();
        assert_eq!(store.list_events_range(lo, hi).unwrap().len(), 4);
    }
}
