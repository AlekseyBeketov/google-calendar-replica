# Spec: recurrence

## ADDED Requirements

### Requirement: Parse and validate RRULE
The system SHALL parse RFC5545 RRULE strings (FREQ, INTERVAL, BYDAY, BYMONTHDAY, BYMONTH,
COUNT, UNTIL, WKST) and reject malformed rules.

#### Scenario: Valid RRULE
- **WHEN** the RRULE "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE" is parsed
- **THEN** it is accepted and expands on Mondays and Wednesdays every 2 weeks

#### Scenario: Malformed RRULE
- **WHEN** the RRULE "FREQ=FORNIGHTLY" is parsed
- **THEN** parsing fails with a descriptive error

### Requirement: Expand occurrences in a range
The system SHALL generate the concrete occurrences of a series that fall inside a query
range, with each occurrence's start and end computed in the event's timezone, DST-correct:
the local wall-clock time of an occurrence MUST stay fixed even when the UTC offset shifts
(5pm stays 5pm local). Expansion MUST support DAILY, WEEKLY, MONTHLY, YEARLY frequencies
with INTERVAL.

#### Scenario: Weekly expansion
- **WHEN** a series FREQ=WEEKLY;BYDAY=MO starts 2026-11-02 and the range is the whole month
- **THEN** occurrences are generated for every Monday in the month

#### Scenario: DST-correct weekly series
- **WHEN** a weekly series starts Fri 2026-03-27 17:00 Europe/Berlin (before spring-forward
  on 2026-03-29) with range covering both sides of the transition
- **THEN** occurrence on 2026-03-27 is 17:00+01:00 (16:00Z) and occurrence on 2026-04-03 is
  17:00+02:00 (15:00Z), i.e. local 17:00 preserved in both

#### Scenario: COUNT-bounded series
- **WHEN** a daily series has COUNT=3
- **THEN** exactly three occurrences are returned regardless of range length

### Requirement: Series-aware update and delete
The system SHALL support three edit/delete scopes on a recurring event: single occurrence
(this), this-and-following, and the whole series. Per-occurrence changes (this) MUST be
stored as exceptions without mutating the master series.

#### Scenario: Delete a single occurrence
- **WHEN** the third occurrence of a series is deleted with scope "this"
- **THEN** the first two (and fourth onward) still appear and the third is omitted

#### Scenario: This-and-following update
- **WHEN** an occurrence is moved with scope "this_and_following"
- **THEN** the original series ends just before that occurrence and a new series starts at it

#### Scenario: Edit whole series
- **WHEN** the master's title is changed with scope "all"
- **THEN** every occurrence (including exceptions) reflects the new title
