//! Task CRUD: add/complete/reopen/edit/delete, manual ordering, overdue flag.

use crate::error::{Error, Result};
use crate::models::{Task, TaskInput, TaskPatch, TaskStatus};
use crate::time::now_rfc3339;
use chrono::NaiveDate;
use rusqlite::Connection;

const SELECT: &str =
    "SELECT id, title, notes, due_date, status, position, created_at, completed_at FROM tasks";

fn status_str(s: TaskStatus) -> &'static str {
    match s {
        TaskStatus::NeedsAction => "needs_action",
        TaskStatus::Completed => "completed",
    }
}

/// Parse status text -> enum.
fn parse_status(s: &str) -> TaskStatus {
    match s {
        "completed" => TaskStatus::Completed,
        _ => TaskStatus::NeedsAction,
    }
}

/// Build a Task from a row and the current "now".
fn task_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let status = parse_status(&r.get::<_, String>("status")?);
    let due_date: Option<String> = r.get("due_date")?;
    let completed_at: Option<String> = r.get("completed_at")?;
    let overdue = compute_overdue(status, due_date.as_deref(), &crate::time::now_rfc3339());
    Ok(Task {
        id: r.get("id")?,
        title: r.get("title")?,
        notes: r.get("notes")?,
        due_date,
        status,
        position: r.get("position")?,
        created_at: r.get("created_at")?,
        completed_at,
        overdue,
    })
}

/// overdue = needs_action AND due_date < today (date-only compare).
fn compute_overdue(status: TaskStatus, due: Option<&str>, now: &str) -> bool {
    if status != TaskStatus::NeedsAction {
        return false;
    }
    let due = match due.and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) {
        Some(d) => d,
        None => return false,
    };
    let today = chrono::DateTime::parse_from_rfc3339(now)
        .map(|dt| dt.date_naive())
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());
    due < today
}

fn fetch(conn: &Connection, id: &str) -> Result<Task> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], task_from_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!("task {id}")),
            other => Error::Sql(other),
        })
}

/// List tasks. `only_active` filters to needs_action; otherwise active tasks
/// come first (then completed), each group by position then created_at.
pub fn list(conn: &Connection, only_active: bool) -> Result<Vec<Task>> {
    let sql = if only_active {
        format!("{SELECT} WHERE status = 'needs_action' ORDER BY position ASC, created_at ASC")
    } else {
        format!(
            "{SELECT} ORDER BY CASE status WHEN 'needs_action' THEN 0 ELSE 1 END, position ASC, created_at ASC"
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], task_from_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Create a task with the next manual position among active tasks.
pub fn create(conn: &Connection, input: TaskInput) -> Result<Task> {
    if input.title.trim().is_empty() {
        return Err(Error::Validation("task title must not be empty".into()));
    }
    let now = now_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let max: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM tasks WHERE status = 'needs_action'",
        [],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO tasks (id, title, notes, due_date, status, position, created_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, 'needs_action', ?5, ?6, NULL)",
        rusqlite::params![
            id,
            input.title,
            input.notes.unwrap_or_default(),
            input.due_date,
            max,
            now,
        ],
    )?;
    fetch(conn, &id)
}

/// Update editable fields (title/notes/due_date). None = keep existing.
pub fn update(conn: &Connection, id: &str, patch: TaskPatch) -> Result<Task> {
    let existing = fetch(conn, id)?;
    let title = patch.title.unwrap_or(existing.title);
    let notes = patch.notes.unwrap_or(existing.notes);
    let due_date = patch.due_date.or(existing.due_date);
    if title.trim().is_empty() {
        return Err(Error::Validation("task title must not be empty".into()));
    }
    conn.execute(
        "UPDATE tasks SET title=?2, notes=?3, due_date=?4 WHERE id=?1",
        rusqlite::params![id, title, notes, due_date],
    )?;
    fetch(conn, id)
}

/// Mark a task completed (records completed_at) or reopen (clears it).
pub fn set_completed(conn: &Connection, id: &str, completed: bool) -> Result<Task> {
    let status = if completed {
        TaskStatus::Completed
    } else {
        TaskStatus::NeedsAction
    };
    let completed_at = if completed { Some(now_rfc3339()) } else { None };
    let n = conn.execute(
        "UPDATE tasks SET status = ?2, completed_at = ?3 WHERE id = ?1",
        rusqlite::params![id, status_str(status), completed_at],
    )?;
    if n == 0 {
        return Err(Error::NotFound(format!("task {id}")));
    }
    fetch(conn, id)
}

/// Reorder active tasks by id list (manual order). Unknown/missing ids are
/// appended afterwards in their current relative order.
pub fn reorder(conn: &Connection, ids: &[String]) -> Result<()> {
    let current = list(conn, true)?;
    let mut ordered: Vec<String> = Vec::new();
    for i in ids {
        if current.iter().any(|t| t.id == *i) && !ordered.contains(i) {
            ordered.push(i.clone());
        }
    }
    for t in &current {
        if !ordered.contains(&t.id) {
            ordered.push(t.id.clone());
        }
    }
    let found = ordered.len();
    for (pos, id) in ordered.iter().enumerate() {
        let affected = conn.execute(
            "UPDATE tasks SET position = ?2 WHERE id = ?1 AND status = 'needs_action'",
            rusqlite::params![id, pos as i64],
        )?;
        if affected == 0 {
            return Err(Error::NotFound(format!("task {id}")));
        }
    }
    // Every active task must have been assigned a position.
    if found != current.len() {
        return Err(Error::Internal(format!(
            "reorder touched {found} of {} active tasks",
            current.len()
        )));
    }
    Ok(())
}

/// Delete a task by id.
pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let removed = conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    if removed == 0 {
        Err(Error::NotFound(format!("task {id}")))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> Connection {
        db::open_in_memory().unwrap()
    }

    #[test]
    fn task_add_appears_in_list() {
        let c = conn();
        let t = create(
            &c,
            TaskInput {
                title: "Prepare Q3 report".into(),
                notes: None,
                due_date: Some("2026-11-10".into()),
            },
        )
        .unwrap();
        assert_eq!(t.status, TaskStatus::NeedsAction);
        assert!(list(&c, false).unwrap().iter().any(|x| x.id == t.id));
    }

    #[test]
    fn task_delete_removes() {
        let c = conn();
        let t = create(&c, TaskInput { title: "x".into(), notes: None, due_date: None }).unwrap();
        delete(&c, &t.id).unwrap();
        assert!(!list(&c, false).unwrap().iter().any(|x| x.id == t.id));
        assert!(delete(&c, &t.id).is_err());
    }

    #[test]
    fn task_complete_reopen() {
        let c = conn();
        let t = create(&c, TaskInput { title: "t".into(), notes: None, due_date: None }).unwrap();
        let done = set_completed(&c, &t.id, true).unwrap();
        assert_eq!(done.status, TaskStatus::Completed);
        assert!(done.completed_at.is_some());
        assert!(!list(&c, true).unwrap().iter().any(|x| x.id == t.id));
        let reopened = set_completed(&c, &t.id, false).unwrap();
        assert_eq!(reopened.status, TaskStatus::NeedsAction);
        assert!(reopened.completed_at.is_none());
    }

    #[test]
    fn overdue_flag_past_due() {
        let c = conn();
        let t = create(
            &c,
            TaskInput { title: "old task".into(), notes: None, due_date: Some("1970-01-01".into()) },
        )
        .unwrap();
        assert!(t.overdue);
        // Completed tasks are never overdue.
        let done = set_completed(&c, &t.id, true).unwrap();
        assert!(!done.overdue);
    }

    #[test]
    fn manual_order_preserved() {
        let c = conn();
        let a = create(&c, TaskInput { title: "A".into(), notes: None, due_date: None }).unwrap();
        let b = create(&c, TaskInput { title: "B".into(), notes: None, due_date: None }).unwrap();
        reorder(&c, &[b.id.clone(), a.id.clone()]).unwrap();
        let order: Vec<String> = list(&c, true).unwrap().iter().map(|t| t.id.clone()).collect();
        assert_eq!(order, vec![b.id.clone(), a.id.clone()]);
    }
}
