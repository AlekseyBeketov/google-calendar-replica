//! Data shapes shared by gcal-core and the Tauri shell (serde).
//!
//! Conventions:
//! - An [`Event`] is either timed (start_utc/end_utc + tz_id) or all-day
//!   (start_date/end_date, exclusive end), selected by `all_day`.
//! - Timed UTC values are RFC3339; all-day dates are floating 'YYYY-MM-DD'.
//! - [`Scope`] picks series-aware update/delete behaviour (This / ThisAndFollowing /
//!   All). Non-recurring events ignore it.

use serde::{Deserialize, Serialize};

/// Edit/delete scope on a recurring series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    This,
    ThisAndFollowing,
    All,
}

/// Event status (RFC 5545 subset used by Google Calendar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

/// A persisted event (single or recurring).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub calendar_id: String,
    pub series_id: Option<String>,
    pub title: String,
    pub all_day: bool,
    // timed fields (present when all_day == false)
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub tz_id: Option<String>,
    // all-day fields (present when all_day == true), end exclusive
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub rrule: Option<String>,
    pub description: String,
    pub location: String,
    pub reminder_minutes: Option<i64>,
    pub status: EventStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Input to create an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    pub calendar_id: String,
    pub title: String,
    #[serde(default)]
    pub all_day: bool,
    /// Required when !all_day (RFC3339).
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub tz_id: Option<String>,
    /// Required when all_day. Exclusive end date.
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: String,
    pub reminder_minutes: Option<i64>,
    #[serde(default = "default_status")]
    pub status: EventStatus,
    /// Optional when all_day == false for a single occurrence.
    pub rrule: Option<String>,
}

fn default_status() -> EventStatus {
    EventStatus::Confirmed
}

/// Patch for `update_event`. Only present field overrides apply.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventPatch {
    pub calendar_id: Option<String>,
    pub title: Option<String>,
    pub all_day: Option<bool>,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub tz_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub rrule: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    pub reminder_minutes: Option<i64>,
    #[serde(default)]
    pub status: Option<EventStatus>,
}

/// One concrete occurrence of a series (expanded), or a single event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInstance {
    pub event_id: String,
    pub series_id: Option<String>,
    /// Occurrence day in local date 'YYYY-MM-DD' (or floating date for all-day).
    pub occurrence_date: String,
    /// UTC of the occurrence start (timed). For all-day, both are dates.
    pub start: String,
    pub end: String,
    pub all_day: bool,
    pub title: String,
    pub calendar_id: String,
    pub color: String,
    pub is_exception: bool,
    pub cancelled: bool,
}

/// A calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_primary: bool,
    pub visible: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// New calendar input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarInput {
    pub name: String,
    pub color: String,
}

/// A task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub due_date: Option<String>,
    pub status: TaskStatus,
    pub position: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
    /// Computed: true when active and due_date < today (floating date compare).
    pub overdue: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NeedsAction,
    Completed,
}

/// Input to create a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
}

/// Patch for update_task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    // Presence-based: None = keep current value, Some(v) = set ("" unsets shown
    // by the caller clearing a field via Some("")).
}

/// Completion patch passed to update_task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteInput {
    pub completed: bool,
}
