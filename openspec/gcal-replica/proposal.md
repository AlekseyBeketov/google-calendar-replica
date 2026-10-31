# Proposal: gcal-replica

## Why

Aleksey wants a local-first Google Calendar / Tasks clone as a native macOS desktop app
written in Rust. Reason: personal productivity tool — a desktop calendar with recurring
events, day/week/month/agenda views, and a tasks panel, with no cloud dependency. This is
built spec-first with TDD so the core logic (recurrence, timezones, storage) is correct
before any UI glue.

## What Changes

- New Rust **Tauri 2** macOS desktop app in this repo.
- New **`gcal-core`** Rust workspace crate: SQLite storage, event model, recurrence (RRULE)
  expansion, timezone/DST handling, tasks — fully unit-tested (TDD), no GUI dependency.
- New **Tauri shell** (`src-tauri`) exposing typed IPC commands over the core.
- New **React + TypeScript + Vite** frontend with Google-Calendar-style month / week / day /
  agenda views, an event create/edit modal (timed, all-day, recurring), and a tasks panel.
- Multi-calendar support (name + color + visibility toggle).
- No auth, no cloud sync, no server (explicitly out of scope).

## Capabilities

**New capabilities** (each becomes `specs/<name>/spec.md`):

- `calendar-events` — event CRUD, timed events with IANA timezone + DST-safe wall-clock,
  all-day events, list events in a date range through the IPC layer.
- `recurrence` — RFC 5545 RRULE parse/expand and series-aware update/delete
  (this / this-and-following / all), plus per-occurrence exceptions.
- `tasks` — task creation, completion (toggle, strikethrough), deletion, ordering; due dates
  with overdue highlighting.
- `calendar-views` — month (7x6 grid), week and day (time grid with overlap/all-day layout),
  4-day agenda, mini-calendar, and navigation (today / prev / next / view switch).
- `calendars` — multiple calendars with name, color and visibility toggle that filters
  events across every view.

**Modified capabilities**: none (greenfield repo).

## Impact

- New `gcal-core/` crate (SQLite via rusqlite, chrono/chrono-tz, `rrule`).
- New `src-tauri/` Tauri 2 app crate + capabilities.
- New `frontend/` Vite + React + TypeScript webview app.
- New `*` schema migrations: `calendars`, `events` (+ `overrides`), `tasks`.
- Dev workflow: TDD (`cargo test -p gcal-core`), spec-driven via this change; git push on
  feature close; graphify + tgrep indexes refreshed after pushes.
