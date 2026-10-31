-- 001_init.sql: schema for the Google Calendar clone.
-- All times stored as UTC RFC3339 text or floating dates; see architecture.md.

CREATE TABLE calendars (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  color      TEXT NOT NULL,
  is_primary INTEGER NOT NULL DEFAULT 0,
  visible    INTEGER NOT NULL DEFAULT 1,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE events (
  id               TEXT PRIMARY KEY,
  calendar_id      TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
  series_id        TEXT,                -- set when event is part of a recurring series
  title            TEXT NOT NULL,
  all_day          INTEGER NOT NULL DEFAULT 0,
  start_date       TEXT,                -- all-day floating start date 'YYYY-MM-DD'
  end_date         TEXT,                -- all-day exclusive end date 'YYYY-MM-DD'
  start_utc        TEXT,                -- timed: RFC3339 UTC
  end_utc          TEXT,                -- timed: RFC3339 UTC
  tz_id            TEXT,                -- IANA timezone for timed events, e.g. 'Europe/Moscow'
  rrule            TEXT,                -- iCal RRULE or NULL
  description      TEXT NOT NULL DEFAULT '',
  location         TEXT NOT NULL DEFAULT '',
  reminder_minutes INTEGER,
  status           TEXT NOT NULL DEFAULT 'confirmed', -- confirmed|tentative|cancelled
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL
);
CREATE INDEX idx_events_calendar ON events(calendar_id);
CREATE INDEX idx_events_start   ON events(start_utc);
CREATE INDEX idx_events_start_date ON events(start_date);
CREATE INDEX idx_events_series ON events(series_id);

-- Exceptions (overrides) of a recurring series: replaces/deletes a single occurrence.
CREATE TABLE event_overrides (
  id             TEXT PRIMARY KEY,
  event_id       TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  occurrence_day TEXT NOT NULL,         -- local date 'YYYY-MM-DD' of the replaced occurrence
  action         TEXT NOT NULL,         -- 'update' | 'delete'
  patch_json     TEXT                   -- for 'update': fields to override (serde EventPatch)
);

CREATE TABLE tasks (
  id           TEXT PRIMARY KEY,
  title        TEXT NOT NULL,
  notes        TEXT NOT NULL DEFAULT '',
  due_date     TEXT,
  status       TEXT NOT NULL DEFAULT 'needs_action', -- needs_action|completed
  position     INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT NOT NULL,
  completed_at TEXT
);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_due ON tasks(due_date);
