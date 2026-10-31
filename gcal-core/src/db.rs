//! Database open/migrate/seed. All access via rusqlite Connection.
//! gcal-core never opens the app DB path itself — callers pass the path or a
//! shared connection in (tests use tempdirs).

use crate::error::{Error, Result};
use rusqlite::Connection;
use std::path::Path;

pub static MIGRATIONS: &[(&str, &str)] = &[(
    "001_init",
    include_str!("../migrations/001_init.sql"),
)];

/// Open (or create) a database at `path`, run pending migrations and seed a
/// primary calendar if the calendar table is empty.
pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    seed(&conn)?;
    Ok(conn)
}

/// For in-memory tests: open a temp database, migrate and seed.
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    seed(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    let installed: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='events'",
        [],
        |r| r.get(0),
    )?;
    if installed == 0 {
        for (_, sql) in MIGRATIONS {
            conn.execute_batch(sql)?;
        }
    }
    Ok(())
}

fn seed(conn: &Connection) -> Result<()> {
    let now = crate::time::now_rfc3339();
    let n: i64 = conn.query_row("SELECT count(*) FROM calendars", [], |r| r.get(0))?;
    if n == 0 {
        conn.execute(
            "INSERT INTO calendars (id, name, color, is_primary, visible, sort_order, created_at, updated_at)
             VALUES (?1, 'My Calendar', '#039be5', 1, 1, 0, ?2, ?2)",
            rusqlite::params![uuid::Uuid::new_v4().to_string(), now],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_db_migrates_and_seeds_primary() {
        let conn = open_in_memory().unwrap();
        let n: i64 = conn
            .query_row("SELECT count(*) FROM calendars", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        let primary: i64 = conn
            .query_row(
                "SELECT count(*) FROM calendars WHERE is_primary = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(primary, 1);
    }

    #[test]
    fn file_db_creates_schema() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("test.db");
        let conn = open_db(&p).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        // calendars, events, event_overrides, tasks + sqlite internals
        assert!(n >= 4, "expected >= 4 tables, got {n}");
    }
}
