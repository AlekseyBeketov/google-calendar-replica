//! gcal-core — Google Calendar replica core.
//!
//! GUI-free Rust crate: SQLite persistence, event model, RRULE recurrence,
//! timezone/DST-safe timed events, all-day events, tasks and calendars.
//! No Tauri dependency: `cargo test -p gcal-core` runs headless and fast.
//!
//! Conventions (replica/architecture.md):
//! - timed events store UTC RFC3339 + IANA `tz_id` (DST-safe wall clock);
//! - all-day events are floating dates (`start_date`/`end_date`, exclusive end);
//! - recurrence is an iCal RRULE expanded in Rust; overrides = per-occurrence
//!   exceptions (this / this-and-following / all).

pub mod db;
pub mod error;
pub mod time;

pub use error::{Error, Result};
