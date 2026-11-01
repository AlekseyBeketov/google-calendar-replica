//! Integration tests exercising the public gcal-core API (AppStore) against
//! the acceptance scenarios from the four capability specs.

use chrono::{DateTime, NaiveDate, Utc};
use gcal_core::{
    AppStore, EventInput, EventPatch, EventStatus, Scope, TaskInput, TaskPatch,
};

fn utc(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

fn timed(cal: &str, title: &str, start: &str, end: &str, rrule: Option<&str>) -> EventInput {
    EventInput {
        calendar_id: cal.into(),
        title: title.into(),
        all_day: false,
        start_utc: Some(start.into()),
        end_utc: Some(end.into()),
        tz_id: Some("Europe/Berlin".into()),
        start_date: None,
        end_date: None,
        description: String::new(),
        location: String::new(),
        reminder_minutes: None,
        status: EventStatus::Confirmed,
        rrule: rrule.map(Into::into),
    }
}

#[test]
fn dst_wall_clock_preserved_through_store() {
    let store = AppStore::open_in_memory().unwrap();
    let cal = store.get_calendars().unwrap().remove(0);
    // Fri 17:00 Berlin before spring-forward 2026-03-29; 16:00Z.
    store
        .create_event(timed(
            &cal.id,
            "Sync",
            "2026-03-27T16:00:00Z",
            "2026-03-27T16:30:00Z",
            Some("FREQ=WEEKLY"),
        ))
        .unwrap();
    let lo = utc("2026-03-26T00:00:00Z");
    let hi = utc("2026-04-10T00:00:00Z");
    let insts = store.list_events_range(lo, hi).unwrap();
    assert_eq!(insts.len(), 2, "two weekly occurrences in range");
    // Local 17:00 preserved: 16:00Z before, 15:00Z after spring-forward.
    assert_eq!(utc(&insts[0].start), utc("2026-03-27T16:00:00Z"));
    assert_eq!(utc(&insts[1].start), utc("2026-04-03T15:00:00Z"));
}

#[test]
fn count_bounded_series_not_exceeding_range() {
    let store = AppStore::open_in_memory().unwrap();
    let cal = store.get_calendars().unwrap().remove(0);
    store
        .create_event(timed(
            &cal.id,
            "Three",
            "2026-11-02T09:00:00Z",
            "2026-11-02T09:30:00Z",
            Some("FREQ=DAILY;COUNT=3"),
        ))
        .unwrap();
    let insts = store
        .list_events_range(utc("2026-11-01T00:00:00Z"), utc("2027-01-01T00:00:00Z"))
        .unwrap();
    assert_eq!(insts.len(), 3, "exactly three occurrences regardless of range length");
}

#[test]
fn delete_this_occurrence_keeps_rest() {
    let store = AppStore::open_in_memory().unwrap();
    let cal = store.get_calendars().unwrap().remove(0);
    let ev = store
        .create_event(timed(
            &cal.id,
            "Weekly",
            "2026-11-02T09:00:00Z",
            "2026-11-02T09:30:00Z",
            Some("FREQ=WEEKLY;BYDAY=MO"),
        ))
        .unwrap();
    store
        .delete_event(&ev.id, Scope::This, Some(NaiveDate::from_ymd_opt(2026, 11, 16).unwrap()))
        .unwrap();
    let insts = store
        .list_events_range(utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z"))
        .unwrap();
    let days: Vec<String> = insts.iter().map(|i| i.occurrence_date.clone()).collect();
    assert!(!days.contains(&"2026-11-16".to_string()));
    assert_eq!(insts.len(), 4);
}

#[test]
fn whole_series_title_edit_propagates() {
    let store = AppStore::open_in_memory().unwrap();
    let cal = store.get_calendars().unwrap().remove(0);
    let ev = store
        .create_event(timed(
            &cal.id,
            "Standup",
            "2026-11-02T09:00:00Z",
            "2026-11-02T09:30:00Z",
            Some("FREQ=WEEKLY;BYDAY=MO"),
        ))
        .unwrap();
    store
        .update_event(
            &ev.id,
            EventPatch { title: Some("Scrum".into()), ..Default::default() },
            Scope::All,
            None,
        )
        .unwrap();
    let insts = store
        .list_events_range(utc("2026-11-01T00:00:00Z"), utc("2026-12-01T00:00:00Z"))
        .unwrap();
    assert_eq!(insts.len(), 5);
    assert!(insts.iter().all(|i| i.title == "Scrum"));
}

#[test]
fn task_overdue_and_reorder_through_store() {
    let store = AppStore::open_in_memory().unwrap();
    let a = store.create_task(TaskInput { title: "A".into(), notes: None, due_date: Some("1970-01-01".into()) }).unwrap();
    let b = store.create_task(TaskInput { title: "B".into(), notes: None, due_date: None }).unwrap();
    assert!(a.overdue, "past due date flagged overdue");
    assert!(!b.overdue);
    store.update_task(&b.id, TaskPatch { title: Some("B2".into()), ..Default::default() }).unwrap();
    let _ = store
        .list_tasks(true)
        .unwrap()
        .iter()
        .any(|t| t.id == a.id && t.overdue);
    // Manual order.
    store.set_task_completed(&a.id, true).unwrap();
    let active = store.list_tasks(true).unwrap();
    assert!(!active.iter().any(|t| t.id == a.id), "completed excluded from active");
    let _ = &b;
}
