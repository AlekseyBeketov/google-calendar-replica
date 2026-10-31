# Spec: calendar-views

## ADDED Requirements

### Requirement: Month view
The system SHALL render a month grid (6x7) starting on the configured-first-weekday,
highlighting today, showing each event as a colored chip in its day cell, with an overflow
"+N more" affordance when a day holds more chips than fit.

#### Scenario: Render month
- **WHEN** the current month is 2026-11
- **THEN** the grid shows 30 day cells starting on the month's first weekday
- **AND** a timed event chip appears in its day and an "all-day" chip in its day cell

### Requirement: Week and day views
The system SHALL render a time grid: an all-day strip plus 24 hourly rows; timed events are
positioned by UTC-clock in the event's timezone, overlapping events are laid out side by
side sharing the width, all-day events occupy the strip.

#### Scenario: Position timed event
- **WHEN** an event runs 09:00-09:30 Europe/Moscow
- **THEN** it is drawn at hour 9 and is 30 minutes tall

#### Scenario: Overlap layout
- **WHEN** two events overlap in time
- **THEN** they are placed in separate columns sharing the same time slot

### Requirement: Agenda view
The system SHALL render a 4-day agenda grouping events by date with time and all-day
markers, ordered by start time.

#### Scenario: Grouped agenda
- **WHEN** listing 2026-11-02..2026-11-06
- **THEN** events appear under their local date headers sorted by start

### Requirement: Navigation and mini-calendar
The system SHALL provide today / previous / next navigation and a view switch
(month/week/day/agenda); a mini-calendar SHALL show busy-day dots and jump the main view to
a clicked date.

#### Scenario: Nav resets to today
- **WHEN** the user presses Today from any range
- **THEN** the view recenters on today

#### Scenario: Mini-calendar jump
- **WHEN** a date is clicked in the mini-calendar
- **THEN** the main view centers on that date
