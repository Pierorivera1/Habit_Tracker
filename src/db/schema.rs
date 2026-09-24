use rusqlite::{Connection, Result};

pub const SCHEMA_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS habits (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    archived    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS habit_completions (
    habit_id    INTEGER NOT NULL,
    date        TEXT NOT NULL,
    completed   INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (habit_id, date),
    FOREIGN KEY (habit_id) REFERENCES habits(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS todos (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT NOT NULL,
    notes        TEXT,
    completed    INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    due_date     TEXT,
    position     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key          TEXT PRIMARY KEY,
    value        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_completions_date ON habit_completions(date);
CREATE INDEX IF NOT EXISTS idx_todos_completed ON todos(completed);
CREATE INDEX IF NOT EXISTS idx_todos_due ON todos(due_date);
"#;

/// Configures SQLite PRAGMAs for high performance and integrity:
/// - WAL mode (Write-Ahead Logging) for concurrent reads and crash safety
/// - Foreign keys enabled (enforces relational integrity and CASCADE)
/// - Synchronous = NORMAL (safe and fast in WAL mode)
/// - Busy timeout = 5000 ms (handles transient lock contention gracefully)
pub fn apply_pragmas(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    Ok(())
}

/// Initializes the database schema and indexes if they do not already exist.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_DDL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_init_tables_exist() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_pragmas(&conn).expect("apply pragmas");
        init_schema(&conn).expect("init schema");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name ASC;")
            .expect("prepare table query");
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .expect("query map")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect tables");

        assert!(tables.contains(&"habits".to_string()));
        assert!(tables.contains(&"habit_completions".to_string()));
        assert!(tables.contains(&"todos".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_schema_indexes_exist() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_pragmas(&conn).expect("apply pragmas");
        init_schema(&conn).expect("init schema");

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name ASC;")
            .expect("prepare index query");
        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .expect("query map")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect indexes");

        assert!(indexes.contains(&"idx_completions_date".to_string()));
        assert!(indexes.contains(&"idx_todos_completed".to_string()));
        assert!(indexes.contains(&"idx_todos_due".to_string()));
    }

    #[test]
    fn test_pragmas_applied() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_pragmas(&conn).expect("apply pragmas");

        let fk: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("query foreign_keys");
        assert_eq!(fk, 1, "Foreign keys must be enabled");

        let busy_timeout: i32 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .expect("query busy_timeout");
        assert_eq!(busy_timeout, 5000, "Busy timeout should be 5000ms");
    }

    #[test]
    fn test_foreign_key_cascade_deletion() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_pragmas(&conn).expect("apply pragmas");
        init_schema(&conn).expect("init schema");

        conn.execute("INSERT INTO habits (name) VALUES ('Test Habit');", [])
            .expect("insert habit");
        let habit_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO habit_completions (habit_id, date, completed) VALUES (?1, '2026-09-23', 1);",
            rusqlite::params![habit_id],
        )
        .expect("insert completion");

        // Verify completion exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM habit_completions WHERE habit_id = ?1;",
                rusqlite::params![habit_id],
                |row| row.get(0),
            )
            .expect("count completions");
        assert_eq!(count, 1);

        // Delete habit; should cascade to completions
        conn.execute(
            "DELETE FROM habits WHERE id = ?1;",
            rusqlite::params![habit_id],
        )
        .expect("delete habit");

        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM habit_completions WHERE habit_id = ?1;",
                rusqlite::params![habit_id],
                |row| row.get(0),
            )
            .expect("count completions after");
        assert_eq!(count_after, 0, "Completions must be cascade deleted");
    }
}
