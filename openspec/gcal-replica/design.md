# Design — gcal-replica

## Context

Greenfield Tauri 2 (Rust) macOS desktop app cloning Google Calendar's local, single-user
surface. Specs in `specs/*`; recon + architecture in `replica/` at repo root. All logic
must live in a GUI-free Rust crate so TDD runs headless.

## Goals

- Correct, tested core: event CRUD, RRULE recurrence, DST-safe times, tasks, calendar
  visibility (the specs).
- Google-Calendar-like UI in a webview: month/week/day/agenda + modal + tasks panel.
- Local-only SQLite persistence; zero network.

## Non-Goals

- Cloud sync, sharing/RSVP, Meet, third-party calendar subscriptions, search, drag
  rescheduling, reminders (v2).

## Decisions

1. **Three-crate layout**: `gcal-core` (pure Rust, rusqlite+chrono+rrule) ← `src-tauri`
   (Tauri 2 app: state, commands, migrations) → `frontend` (React+TS+Vite, zustand,
   `@tauri-apps/api`). Why: `cargo test -p gcal-core` is fast, headless TDD. Alternative
   rejected: logic in Tauri crate couples tests to the runtime.
2. **Recurrence via `rrule` crate** (RFC5545 parse+expand) + own thin series layer:
   `series_id`, per-occurrence `overrides` table, and an `exceptions` table for deleted
   occurrences (scopes this / this-and-following / all). Alternative rejected: hand-rolled
   recurrence — the hard part, off the shelf is correct.
3. **Time model**: timed events store `start_utc`/`end_utc` RFC3339 + `tz_id`; expansion
   computes local wall-clock in `tz_id` per occurrence then converts to UTC (DST-safe,
   "5pm stays 5pm"). All-day: floating `start_date`/`end_date` (exclusive end), never tz
   shifted. Rejected: single offset column (breaks DST), naive local datetimes (unportable).
4. **Schema**: `calendars`, `events`, `event_overrides` (exceptions by occurrence start),
   `tasks`. Every mutation bumps `updated_at`. Range queries filter by UTC overlap +
   calendar visibility join.
5. **Frontend state**: zustand store mirrors viewport (date, view mode); data fetched per
   range via `invoke`; CSS replicates Google Calendar's density (Material-ish red/blue
   palette). Events grouped client-side for month cells; time grid layering for overlap.

## Risks / Trade-offs

- [RRULE edge cases (BYMONTHDAY endpoints, exceptions)] → rely on `rrule` crate tests; our
  tests pin the five spec scenarios.
- [Series split (this-and-following) correctness] → implement as master UNTIL + new master;
  test the exact spec scenario.
- [All-day vs timed semantics drift] → single `EventKind` enum in core, serde as tagged
  union to the frontend.
- [First Tauri `cargo build` is heavy (~5-10 min)] → iterate on `cargo test` for the core;
  shell built once.
- [Webview layout complexity] → keep overlap logic to max-3 columns; simple + correct beats
  pixel-perfect for v1.

## Migration Plan

Bluefield. Migration `001_init.sql` runs on first launch (embedded in gcal-core). Seed
primary calendar. Rollback: not applicable (local, no shippable DB yet).

## Open Questions

- Start-of-week default: Europe/Moscow (Monday). Expose in settings later — fixed Monday.
- Default all-day vs timed in quick-create modal: all-day default off (matches GCal "off").
