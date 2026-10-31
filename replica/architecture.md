# Architecture — Google Calendar desktop clone (Rust, macOS)

From `replica/recon.md` + `replica/features.csv`.

## 1. Stack (one line each why)

| layer | choice | why |
| --- | --- | --- |
| desktop shell | Tauri 2 (Rust) | native macOS window/menu/notifications, all core logic in Rust, prior art in Aleksey's stack |
| core logic | Rust workspace crate `gcal-core` | pure, testable (TDD), no GUI dependency |
| persistence | SQLite via rusqlite | local-first, single user, zero ops; bundled with Tauri |
| recurrence | `rrule` crate | RFC 5545 RRULE parse + expansion, battle-tested |
| time | chrono + chrono-tz | IANA timezones + DST correct wall-clock semantics |
| UI | React + TypeScript + Vite | Google-Calendar-like grid UI is far faster in a webview; state via zustand; no server |
| packaging | Tauri bundler (dmg/app) | macOS app bundle |

No network, no auth, no server. Single local DB, single user.

## 2. Schema (SQLite DDL — `gcal-core/migrations/001_init.sql`)

```sql
CREATE TABLE calendars (
  id        TEXT PRIMARY KEY,          -- uuid
  name      TEXT NOT NULL,
  color     TEXT NOT NULL,             -- hex, e.g. '#039be5'
  is_primary INTEGER NOT NULL DEFAULT 0,
  visible   INTEGER NOT NULL DEFAULT 1,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE events (
  id             TEXT PRIMARY KEY,
  calendar_id    TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
  title          TEXT NOT NULL,
  all_day        INTEGER NOT NULL DEFAULT 0,
  start_date     TEXT,                 -- all-day: 'YYYY-MM-DD' (floating)
  end_date       TEXT,                 -- all-day: exclusive end date 'YYYY-MM-DD'
  start_utc      TEXT,                 -- timed: RFC3339 with offset, UTC-normalized
  end_utc        TEXT,
  tz_id          TEXT,                 -- IANA e.g. 'Europe/Moscow'
  rrule          TEXT,                 -- iCal RRULE or NULL
  description    TEXT NOT NULL DEFAULT '',
  location       TEXT NOT NULL DEFAULT '',
  reminder_minutes INTEGER,            -- NULL = default
  status         TEXT NOT NULL DEFAULT 'confirmed', -- confirmed|tentative|cancelled
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);
CREATE INDEX idx_events_calendar ON events(calendar_id);
CREATE INDEX idx_events_start  ON events(start_utc);
CREATE INDEX idx_events_start_date ON events(start_date);

CREATE TABLE tasks (
  id         TEXT PRIMARY KEY,
  title      TEXT NOT NULL,
  notes      TEXT NOT NULL DEFAULT '',
  due_date   TEXT,                     -- 'YYYY-MM-DD' or NULL
  status     TEXT NOT NULL DEFAULT 'needs_action', -- needs_action|completed
  position   INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  completed_at TEXT
);
```

Times: store timed events as UTC RFC3339 + IANA `tz_id` for wall-clock DST-preserving
recurrence. All-day events are floating dates (`start_date`/`end_date`, exclusive end,
mirroring the Calendar data model). `updated_at` bump on every mutation (for future sync).

## 3. API — Tauri commands, one group per flow (invoked from frontend via invoke)

| command | does what | input | output |
| --- | --- | --- | --- |
| get_calendars | list calendars | — | Vec<Calendar> |
| add_calendar | create calendar | name, color | Calendar |
| set_calendar_visible | toggle visibility | id, visible | () |
| list_events_range | events in [start,end] incl. recurrence expansion | start_utc, end_utc | Vec<EventInstance> |
| get_event | single event + series info | id | EventDetail |
| create_event | create event | EventInput | Event |
| update_event | update event (mode: this/this_and_following/all) | id, EventInput, mode | Event |
| delete_event | delete event (mode: this/this_and_following/all) | id, mode | () |
| list_tasks | tasks sorted | filter(completed\?) | Vec<Task> |
| create_task | add task | title | Task |
| update_task | edit/complete | id, patch | Task |
| delete_task | remove | id | () |
| get_settings | tz/startOfWeek prefs | — | Settings |

Flow mapping: F01-F03 -> create_event; F05 -> update_event; F06 -> delete_event(3 modes);
F07 -> tasks commands; S05/S09 -> get_calendars.

## 4. Parts that bite (design decisions)

1. **Timezone/DST**: recurring timed events keep `tz_id` + UTC anchor; expansion computes
   wall-clock in `tz_id` then converts to UTC (Karnaugh rule: "5pm stays 5pm"). All-day is
   floating-date (no tz shift), exclusive end date.
2. **Recurrence + series edits**: events have a `series_id` when recurring; occurrences are
   derived by expansion, stored as instances (not rows). "This" -> create an exception row
   (overrides) ; "this and following" -> split series with UNTIL at occurrence; "all" -> edit
   master. Overrides table for exceptions.
3. **Event overlap layout**: sort by start, assign columns for overlap-up-to-3, left-align,
   width = 1/n. All-day chips in a strip above the grid.
4. **TDD**: all logic in `gcal-core` (no tauri dep) so `cargo test -p gcal-core` runs headless
   and fast. Overrides table by design (not afterthought).

## 5. Build order (from features.csv)

1. **Vertical slice (M1)**: gcal-core: calendar seed + create/list events in range + SQLite +
   tests -> Tauri `list_events_range`/`create_event` wired -> frontend month view shows an
   event you created. Proves stack.
2. **Must-haves**: full event CRUD + edit modal + all-day + week/day/agenda views + tasks panel.
3. **Should-haves**: recurrence UI+expansion+series modes, mini-calendar, calendar visibility.
4. **Could-haves** (later): drag-reschedule, search, reminders/local notifications, .ics import.

Riskiest: RRULE expansion + series-edit correctness; DST wall-clock; event overlap layout.

Next: /replica-design (visual), and openspec tasks drive the TDD build.
