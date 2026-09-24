pub mod queries;
pub mod schema;

use chrono::NaiveDate;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::domain::{DailyStats, ExportData, Habit, Todo};

/// Resolves the default XDG-compliant path for Habitodo's SQLite database:
/// `~/.local/share/habitodo/habitodo.db`
pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("habitodo")
        .join("habitodo.db")
}

/// Encapsulated SQLite database engine providing thread-safe, WAL-enabled persistence.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens or creates a database at `path`, creating parent directories if needed,
    /// applying performance & safety PRAGMAs, and initializing the schema.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(p)?;
        schema::apply_pragmas(&conn)?;
        schema::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Creates an isolated in-memory database instance with pragmas and schema initialized.
    /// Ideal for unit tests, E2E headless validation, and ephemeral sessions.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        schema::apply_pragmas(&conn)?;
        schema::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Convenience alias for `new_in_memory`.
    pub fn open_in_memory() -> Result<Self> {
        Self::new_in_memory()
    }

    /// Returns a reference to the underlying `rusqlite::Connection`.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Initializes the schema on the underlying connection (idempotent).
    pub fn init_schema(&self) -> Result<()> {
        schema::init_schema(&self.conn)
    }

    /// Retrieves active habits with completion status for the specified date via LEFT JOIN.
    pub fn get_habits_for_date(&self, date: NaiveDate) -> Result<Vec<(Habit, bool)>> {
        queries::get_habits_for_date(&self.conn, date)
    }

    /// Atomically toggles a habit completion status on `date` using atomic UPSERT.
    /// Returns the new completion status (true if completed, false if uncompleted).
    pub fn toggle_habit_completion(&self, habit_id: i64, date: NaiveDate) -> Result<bool> {
        queries::toggle_habit_completion(&self.conn, habit_id, date)
    }

    /// Returns daily completion percentage stats for the past 30 days ending at `end_date`.
    pub fn get_30_day_completion_stats(&self, end_date: NaiveDate) -> Result<Vec<DailyStats>> {
        queries::get_30_day_completion_stats(&self.conn, end_date)
    }

    /// Returns the 30-day success adherence percentage (0.0 to 100.0%) for a habit.
    pub fn get_habit_success_rate_30_days(
        &self,
        habit_id: i64,
        end_date: NaiveDate,
    ) -> Result<f32> {
        queries::get_habit_success_rate_30_days(&self.conn, habit_id, end_date)
    }

    /// Returns a map of calendar dates to completion counts for shading the monthly heatmap.
    pub fn get_monthly_heatmap(&self, year: i32, month: u32) -> Result<HashMap<NaiveDate, usize>> {
        queries::get_monthly_heatmap(&self.conn, year, month)
    }

    /// Returns all to-dos ordered with uncompleted first, then by position.
    pub fn get_all_todos(&self) -> Result<Vec<Todo>> {
        queries::get_all_todos(&self.conn)
    }

    /// Inserts a new to-do task and returns its generated ID.
    pub fn add_todo(
        &self,
        title: &str,
        notes: Option<&str>,
        due_date: Option<NaiveDate>,
    ) -> Result<i64> {
        queries::add_todo(&self.conn, title, notes, due_date)
    }

    /// Toggles a to-do item's completion flag and updates `completed_at` timestamp.
    /// Returns the new completed state.
    pub fn toggle_todo(&self, todo_id: i64) -> Result<bool> {
        queries::toggle_todo(&self.conn, todo_id)
    }

    /// Deletes a to-do item by ID.
    pub fn delete_todo(&self, todo_id: i64) -> Result<()> {
        queries::delete_todo(&self.conn, todo_id)
    }

    /// Updates the notes of a to-do item.
    pub fn update_todo_notes(&self, todo_id: i64, notes: Option<&str>) -> Result<()> {
        queries::update_todo_notes(&self.conn, todo_id, notes)
    }

    /// Updates the title of a to-do item.
    pub fn update_todo_title(&self, todo_id: i64, title: &str) -> Result<()> {
        queries::update_todo_title(&self.conn, todo_id, title)
    }

    /// Updates the due date of a to-do item.
    pub fn update_todo_due_date(&self, todo_id: i64, due_date: Option<NaiveDate>) -> Result<()> {
        queries::update_todo_due_date(&self.conn, todo_id, due_date)
    }

    /// Adds a new habit and returns its generated ID.
    pub fn add_habit(&self, name: &str) -> Result<i64> {
        queries::add_habit(&self.conn, name)
    }

    /// Soft-deletes a habit by setting `archived = 1`.
    pub fn archive_habit(&self, habit_id: i64) -> Result<()> {
        queries::archive_habit(&self.conn, habit_id)
    }

    /// Renames an existing habit.
    pub fn rename_habit(&self, habit_id: i64, new_name: &str) -> Result<()> {
        queries::rename_habit(&self.conn, habit_id, new_name)
    }

    /// Swaps/reorders a habit up or down in the display list.
    pub fn reorder_habit(&self, habit_id: i64, move_up: bool) -> Result<()> {
        queries::reorder_habit(&self.conn, habit_id, move_up)
    }

    /// Exports full database contents into `ExportData`.
    pub fn export_data(&self) -> Result<ExportData> {
        queries::export_data(&self.conn)
    }

    /// Restores full database state from an `ExportData` container atomically.
    pub fn import_data(&self, data: &ExportData) -> Result<()> {
        queries::import_data(&self.conn, data)
    }

    /// Retrieves an application setting by key.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        queries::get_setting(&self.conn, key)
    }

    /// Sets or updates an application setting.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        queries::set_setting(&self.conn, key, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_in_memory_lifecycle() {
        let db = Database::new_in_memory().expect("create in-memory db");
        db.init_schema().expect("init schema idempotent");

        let h_id = db.add_habit("Hydration").expect("add habit");
        assert!(h_id > 0);

        let today = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();
        let habits = db.get_habits_for_date(today).expect("get habits");
        assert_eq!(habits.len(), 1);
        assert!(!habits[0].1);

        let toggled = db.toggle_habit_completion(h_id, today).expect("toggle");
        assert!(toggled);

        let habits_after = db.get_habits_for_date(today).expect("get habits after");
        assert!(habits_after[0].1);
    }

    #[test]
    fn test_database_file_creation_with_tempfile() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("sub_folder").join("test_habitodo.db");

        // Parent directory sub_folder does not exist yet; Database::open should create it
        let db = Database::open(&db_path).expect("open database");
        assert!(db_path.exists());

        let t_id = db.add_todo("Test Todo", None, None).expect("add todo");
        let todos = db.get_all_todos().expect("get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, t_id);
    }

    #[test]
    fn test_zero_panic_invariant_on_empty_db() {
        let db = Database::new_in_memory().expect("open memory db");
        let today = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();

        // Empty habits
        let habits = db.get_habits_for_date(today).expect("empty habits");
        assert!(habits.is_empty());

        // Empty todos
        let todos = db.get_all_todos().expect("empty todos");
        assert!(todos.is_empty());

        // Empty 30-day stats
        let stats = db.get_30_day_completion_stats(today).expect("empty stats");
        assert!(stats.is_empty());

        // Success rate on non-existent habit
        let rate = db
            .get_habit_success_rate_30_days(999, today)
            .expect("rate non-existent");
        assert_eq!(rate, 0.0);

        // Empty heatmap
        let heatmap = db.get_monthly_heatmap(2026, 9).expect("empty heatmap");
        assert!(heatmap.is_empty());

        // Export empty db
        let export = db.export_data().expect("export empty");
        assert!(export.habits.is_empty());
        assert!(export.todos.is_empty());
        assert!(export.habit_completions.is_empty());
    }

    #[test]
    fn test_unique_constraint_returns_error_without_panic() {
        let db = Database::new_in_memory().expect("open memory db");
        db.add_habit("Exercise").expect("add first");
        let res = db.add_habit("Exercise");
        assert!(res.is_err(), "Duplicate habit name must return Err");
    }
}
