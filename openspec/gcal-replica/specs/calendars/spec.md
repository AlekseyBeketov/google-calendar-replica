# Spec: calendars

## ADDED Requirements

### Requirement: Multiple calendars with color
The system SHALL support multiple calendars, each with a name and hex color. On startup a
primary calendar SHALL be seeded.

#### Scenario: Primary seeded
- **WHEN** the database is first initialized
- **THEN** a primary calendar named "My Calendar" exists and is flagged primary

#### Scenario: Add calendar
- **WHEN** a calendar named "Work" with color "#039be5" is created
- **THEN** events on it render with that color

### Requirement: Visibility toggle
The system SHALL allow hiding/showing a calendar; hidden calendars' events MUST be excluded
from every view's range query.

#### Scenario: Hide a calendar
- **WHEN** a calendar is hidden
- **THEN** its events are not returned by range listings until it is shown again
