# Spec: tasks

## ADDED Requirements

### Requirement: Create and delete tasks
The system SHALL allow creating a task with a title and optional due date, and deleting it.
Tasks persist in SQLite.

#### Scenario: Add task
- **WHEN** a user adds the task "Prepare Q3 report" with due 2026-11-10
- **THEN** it is stored with status "needs_action" and appears in the task list

#### Scenario: Delete task
- **WHEN** a task is deleted
- **THEN** it no longer appears in any task list

### Requirement: Toggle completion
The system SHALL mark a task completed/back-to-active, recording a completion timestamp and
moving the task to the completed group.

#### Scenario: Complete a task
- **WHEN** a task is marked completed
- **THEN** status is "completed", `completed_at` is set, and it sorts after active tasks

#### Scenario: Reopen
- **WHEN** a completed task is marked active again
- **THEN** status returns to "needs_action" and `completed_at` is cleared

### Requirement: Ordering and overdue
Active tasks SHALL keep a manual position; tasks with a due date in the past SHALL render as
overdue in the UI (visual flag computed from due_date vs today).

#### Scenario: Overdue flag
- **WHEN** a task has due_date before today and status "needs_action"
- **THEN** the listing marks it overdue

#### Scenario: Manual order preserved
- **WHEN** two active tasks are reordered
- **THEN** the listing returns them in the new order
