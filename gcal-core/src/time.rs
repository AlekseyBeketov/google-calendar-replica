//! Shared time helpers for gcal-core.
//!
//! Conventions (see replica/architecture.md): timed events store UTC RFC3339
//! plus an IANA timezone id; all-day events are floating dates. All clock reads
//! go through here so tests can pin the "now".

use chrono::{DateTime, Utc};

/// Current UTC time as RFC3339 (used for created_at/updated_at and "today").
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}
