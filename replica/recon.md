# Recon map — Google Calendar (desktop web) → Rust macOS desktop clone

Source: https://calendar.google.com/calendar (public product behavior + Google help center).
Scope agreed: a local-first desktop calendar + tasks app for a single user, macOS.
Target stack: Rust desktop (Tauri 2) + SQLite. Not a server product. No Google sync/auth.

## 1. Screen inventory

| ID | screen | how you get there | purpose | key components | states seen |
|----|--------|-------------------|---------|----------------|-------------|
| S01 | Month view | view switch | see a month of days, events as colored chips | 7-col grid, day number, all-day chips, timed chips, "+" in day cell | empty, filled, today highlight, overflow "+N more" |
| S02 | Week view | view switch | timed grid 7 days x 24h | time gutter (all-day row + timed grid), hour lines, event blocks, drag create | empty, dense, overlapping events |
| S03 | Day view | view switch | single day x 24h | same time grid, 1 column | empty, dense |
| S04 | Agenda (Schedule) | view switch (4-day) | upcoming events list | grouped-by-date list, time range, all-day marker | empty, past events greyed |
| S05 | Mini-calendar | left sidebar | jump dates, see dots | small month grid, dots per busy day | none |
| S06 | Event details panel | click an event | view/edit event | modal: title, date/time, calendar, guests, description, reminders, recurrence | read-only guest, "delete this/this+following/series" |
| S07 | Create event modal | click "+", click empty slot, quick add "Create" | create event | title, start/end (date+time picker), all-day toggle, repeat select, calendar select, description, location | new/duplicate/quick |
| S08 | Tasks panel (right side) | tasks icon / click Tasks | Google Tasks: add, check off, due date, list items | list, add row, checkbox, due-date chip, list selector | empty, completed struck-through |
| S09 | Right sidebar / calendar list | "My calendars" | toggle visibility, edit color | checkbox list, color dot, settings | many calendars, hidden ones |
| S10 | Search popup | search icon / Ctrl+K | fuzzy event search | text input, results, date group | empty, no-results, keyboard nav |
| S11 | Today button + prev/next + view switch | top bar | navigate time | buttons, date-range title | any |

## 2. User flows

F01 Create a timed event
    S11 top bar -> S02/S03 click an empty time slot -> S07 fill title + times -> save -> S02 chip appears
    edge: end before start, click "more options", event across midnight, overlap

F02 Create an all-day event
    S01 click day number / S02 all-day row -> S07 toggle All day -> save -> chip on all-day row or month cell

F03 Create a recurring event
    S07 "Does not repeat" -> pick Daily/Weekly/Monthly/Custom(RRULE) -> save -> series with repeat icon
    edge: custom frequency, interval, until count

F04 Navigate time and views
    S11 Today resets to today; prev/next moves per view granularity; S01/S02/S03/S04 switch layout; today's cell outlined blue

F05 Edit / reschedule
    S02/S03 drag event block to new time -> S06 panel updates; click event -> S06 edit fields -> save

F06 Delete
    S06 trash icon -> S07 confirmation: delete this event / this and following / all events

F07 Tasks
    S08 add "...", checkbox completes (strikethrough), due-date chip, move between lists, add list
    edge: overdue highlighted, completed hidden toggle

F08 Search
    S10 type -> fuzzy match on title/desc -> click result -> S06 open

F09 Calendar visibility
    S09 toggle checkbox hides calendar's events everywhere; color dot picks color

## 3. Reused components

- DateNavigator (today / < > / range title)
- MiniCalendar
- CalendarGrid (month: 6x7 cells) / TimeGrid (week/day with hour rows)
- EventChip (timed/all-day/multi-day variants with color)
- CreateEditModal (date+time pickers, all-day toggle, repeat picker, calendar picker)
- RepeatPicker (never / daily / weekly / monthly / yearly / custom)
- TasksPanel (list, add-row, checkbox, due chip)
- Sidebar (mini-cal + calendars list)
- SearchPopup
- Toast (delete feedback, save)

## 4. Inferred data model (desktop-local, single user)

Calendar
  id, name, color (hex), is_primary (bool), visible (bool)
  evidence: S09 list/colors; confidence: high

Event
  id, calendar_id, title, all_day?, start (date or datetime+tz), end, rrule?, description, location,
  reminders_minutes, status (confirmed/tentative), created_at, updated_at
  evidence: S07 form fields, S06 panel, F03; confidence: high

Task
  id, task_list_id?, title, notes, due (date), status (needs_action/completed), position, created_at
  evidence: S08; confidence: high. Desktop clone: single task list is enough for v1.

Recurrence (RRULE) represented as an iCal RRULE string stored on the event; expansion computed in Rust.
evidence: F03 custom picker maps to FREQ/INTERVAL/BYDAY/COUNT/UNTIL; confidence: high

## 5. Feature matrix → replica/features.csv

## 6. What cannot be cloned (skip)

- Google account / cloud sync / Google Workspace collaboration
- Google Meet / video conferencing integration
- Sharing, ACLs, guest invitations with RSVP
- Subscribing external .ics/.webcal feeds (could: later)
- Reminder push via Google servers (we use local notifications: could)

## 7. Sizing & hard parts

Size: M (a few weeks of human effort; this build does a working vertical slice of must-haves).

Hard parts:
1. Recurrence (RRULE) parse + expansion + series-aware edit/delete (this / this-and-following / all).
2. Timezone/DST handling of timed events + floating all-day dates (the "5pm repeats at 5pm" rule).
3. Time-grid layout: overlapping events, all-day + multi-day chips, overflow.
4. Local-first persistence (SQLite) with clean TDD-able Rust core.

Next: /replica-architect.
