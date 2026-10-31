# Tasks — gcal-replica

## 1. Workspace skeleton (repo changes)

- [ ] 1.1 Root Cargo workspace: `gcal-core` (lib) + `src-tauri` (bin) members, shared Cargo.toml
- [ ] 1.2 `gcal-core` deps: rusqlite(bundled), chrono, chrono-tz, rrule, uuid, serde, thiserror
- [ ] 1.3 Migration 001_init.sql embedded: calendars/events/event_overrides/tasks DDL + indexes
- [ ] 1.4 DB open + run_migrations + seed primary calendar, unit-tested
- [ ] 1.5 `frontend/` Vite + React + TS scaffold with `npm run build` green

## 2. gcal-core: events (calendar-events spec)

- [ ] 2.1 Event model (timed/all-day variants, serde) + validation (end>start) unit-tested
- [ ] 2.2 create_event + get_event + update_event + delete_event (single, non-recurring) + tests
- [ ] 2.3 list_events_in_range with calendar-visibility filter + tests
- [ ] 2.4 DST wall-clock preservation test (Europe/Moscow DST transition weekly recurrence)
- [ ] 2.5 all-day floating-date semantics + overlap-range test

## 3. gcal-core: recurrence (recurrence spec)

- [ ] 3.1 RRULE parse/validate wrapper around rrule crate + malformed-rule test
- [ ] 3.2 Series events: occurrence expansion in range (daily/weekly/monthly + COUNT/INTERVAL) + tests
- [ ] 3.3 Exceptions: delete single occurrence (scope this) via event_overrides + test
- [ ] 3.4 this-and-following: split series (UNTIL + new master) + test
- [ ] 3.5 whole-series edit propagates to occurrences + test

## 4. gcal-core: tasks + calendars (tasks/calendars specs)

- [ ] 4.1 Task CRUD + completion toggle (completed_at, ordering) + tests
- [ ] 4.2 Overdue flag computed from due_date vs today + test
- [ ] 4.3 Calendar CRUD + visibility toggle filtering range listings + tests

## 5. Tauri shell (IPC)

- [ ] 5.1 src-tauri: tauri.conf.json + capabilities (core+event) + main/lib.rs stub builds
- [ ] 5.2 Manage<AppStore> state (DB) + commands: get_calendars, add/set_visible, create/update/delete_event, list_events_range, task commands
- [ ] 5.3 serde request/response DTO mapping; `cargo check` in src-tauri passes

## 6. Frontend (calendar-views spec + tasks UI)

- [ ] 6.1 App shell: topbar (today / ‹ › / date-range title / view switch), sidebar (mini-cal + calendars), tasks panel drawer
- [ ] 6.2 Month view grid with event chips + all-day + overflow "+N more"
- [ ] 6.3 Week + day time grids with all-day strip, correctly positioned/blended timed events
- [ ] 6.4 Agenda 4-day grouped list
- [ ] 6.5 Create/edit modal: title, timed/all-day, start/end pickers, repeat select, calendar select — wired to IPC
- [ ] 6.6 Tasks panel: add/complete/delete via IPC; overdue styling
- [ ] 6.7 Calendar visibility toggles + color rendering everywhere

## 7. Integration + verification

- [ ] 7.1 `cargo test` (workspace) all green
- [ ] 7.2 `npm run build` green; tauri dev assets resolve
- [ ] 7.3 `cargo build` (src-tauri, debug) green
- [ ] 7.4 README: build/run/test instructions; feature checklist from features.csv
- [ ] 7.5 Commit per feature group + git push; refresh graphify + tgrep indexes after push
