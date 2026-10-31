# Spec: calendar-events

## ADDED Requirements

### Requirement: Create a timed event
The system SHALL allow creating an event with a title, start and end time (with an IANA
timezone), persisted to SQLite. The end MUST be after the start.

#### Scenario: Create timed event
- **WHEN** a user creates an event "Standup" with start 09:00 and end 09:30 Europe/Moscow
- **THEN** the event is stored with utc start 06:00, utc end 06:30, tz_id "Europe/Moscow"
- **AND** listing events in the containing range returns it

#### Scenario: Reject end before start
- **WHEN** a user creates an event with end before start
- **THEN** the system rejects the mutation with an error and stores nothing

### Requirement: Create an all-day event
The system SHALL support all-day events stored as floating dates `start_date` and exclusive
`end_date` (e.g. 2026-11-03..2026-11-04 for a single day), never shifted by timezone.

#### Scenario: Create all-day event
- **WHEN** a user creates an all-day event on 2026-11-03
- **THEN** it is stored with start_date "2026-11-03" and end_date "2026-11-04"
- **AND** it is returned for every day cell it spans

#### Scenario: Multi-day all-day stays floating
- **WHEN** an all-day event spans 2026-11-03 through 2026-11-06
- **THEN** listing with any UTC range inside those dates returns it unchanged

### Requirement: Update and delete events
The system SHALL support editing title/time/calendar of an event and deleting it. Deleted
events MUST disappear from all range listings.

#### Scenario: Edit event title
- **WHEN** an event's title is edited from "Old" to "New"
- **THEN** listing events returns "New"

#### Scenario: Delete event
- **WHEN** an event is deleted
- **THEN** it is no longer returned by any range listing

### Requirement: List events in a range
The system SHALL return timed events whose [start,end) overlaps the query range, and all-day
events whose [start_date,end_date) overlaps, with recurrence expanded (see recurrence spec).

#### Scenario: Range query
- **WHEN** querying the week 2026-11-02..2026-11-09
- **THEN** overlapping timed and all-day events are returned with their UTc/date fields

### Requirement: DST-safe timed events
Timed events computed by recurrence MUST preserve wall-clock time across daylight-saving
transitions (5pm stays 5pm local), converting to UTC per the effective offset.

#### Scenario: Wall-clock preserved across DST
- **WHEN** a weekly recurrence starts at 17:00 Europe/Berlin on a date before the spring-forward
- **THEN** each occurrence has local 17:00 (its utc value differing by the changed offset)

#### Scenario: Fixed-offset fallback
- **WHEN** an event's tz_id is a UTC-fixed zone (e.g. "Etc/GMT-3")
- **THEN** expansions keep the wall clock and constant offset
