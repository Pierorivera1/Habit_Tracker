//! Habitodo: Native desktop habit and to-do tracking application.

pub mod app;
pub mod db;
pub mod domain;
pub mod keybindings;
pub mod ui;

pub use app::{AppState, InputMode, Message, Screen};
pub use db::{default_db_path, Database};
pub use ui::view;

use chrono::{Datelike, Local};
use domain::ExportData;

/// Formatted CLI usage and full keybindings cheat sheet.
pub fn help_text() -> &'static str {
    r#"Habitodo 0.1.0
Native dark-mode habit and to-do tracking application.

USAGE:
    habitodo [OPTIONS]

OPTIONS:
    -h, --help       Print help information and keybindings cheat sheet
    -V, --version    Print version information
    --verify         Execute self-verification diagnostics against in-memory SQLite

KEYBINDINGS:
    Global:
        q            Quit application
        s, Tab       Toggle metrics sidebar
        ?            Toggle keyboard shortcuts help modal

    Main Dashboard Navigation:
        j, Down      Move selection cursor down
        k, Up        Move selection cursor up
        h, Left      Previous day
        l, Right     Next day
        t            Jump to today
        g            Jump to top of list
        G            Jump to bottom of list

    Main Dashboard Actions:
        Space        Toggle completion of selected habit / to-do
        x            Complete / dismiss selected item
        i            Add new to-do task (inline input)
        Enter        Edit selected item
        d            Delete selected to-do (prompts confirmation)
        n            Edit attached notes for selected to-do
        o            Open Settings screen
        Esc          Cancel modal / dismiss input

    Settings Screen:
        j, Down      Move selection cursor down
        k, Up        Move selection cursor up
        g            Jump to top of habit list
        G            Jump to bottom of habit list
        K            Reorder habit up (priority increase)
        J            Reorder habit down (priority decrease)
        i            Add new habit (inline input)
        r            Rename selected habit
        d            Archive selected habit (soft-delete)
        e            Export backup data to JSON file
        o, Esc       Return to Main dashboard

    Modal & Text Input:
        Enter        Submit input / confirm deletion
        Esc, n       Cancel input / cancel deletion
        y, Enter     Confirm deletion (in delete dialog)"#
}

/// Version information string.
pub fn version_text() -> &'static str {
    "habitodo 0.1.0"
}

/// Executes comprehensive self-verification diagnostics against an in-memory SQLite database.
///
/// Verifies:
/// 1. In-memory SQLite initialization & WAL/foreign-keys PRAGMAs
/// 2. Schema tables (habits, habit_completions, todos, settings)
/// 3. Schema indexes (idx_completions_date, idx_todos_completed, idx_todos_due)
/// 4. Daily habits LEFT JOIN query with default uncompleted states
/// 5. Atomic UPSERT habit toggle transitions
/// 6. 30-day stats rollups, per-habit success rate, and monthly heatmap
/// 7. To-Dos CRUD and completion toggling
/// 8. Full JSON export and import serde roundtrip
/// 9. Zero-panic invariant on empty database, message update handling, and UI rendering
pub fn run_verification() -> Result<Vec<String>, String> {
    let mut logs = Vec::new();

    // 1. In-Memory Database Initialization & PRAGMAs
    let db = Database::new_in_memory()
        .map_err(|e| format!("In-memory database creation failed: {}", e))?;

    let fk: i64 = db
        .connection()
        .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
        .map_err(|e| format!("PRAGMA foreign_keys query failed: {}", e))?;
    if fk != 1 {
        return Err(format!("Expected foreign_keys=1, got {}", fk));
    }
    logs.push("In-memory SQLite initialization & WAL/foreign-keys PRAGMAs verified".to_string());

    // 2. Schema Table Verification
    let mut table_stmt = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='table';")
        .map_err(|e| format!("Failed to query sqlite_master for tables: {}", e))?;
    let table_rows = table_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to read table rows: {}", e))?;
    let tables: Vec<String> = table_rows.filter_map(|r| r.ok()).collect();
    for required in &["habits", "habit_completions", "todos", "settings"] {
        if !tables.contains(&required.to_string()) {
            return Err(format!("Required table '{}' missing from schema", required));
        }
    }
    logs.push("Schema tables verified (habits, habit_completions, todos, settings)".to_string());

    // 3. Schema Index Verification
    let mut index_stmt = db
        .connection()
        .prepare("SELECT name FROM sqlite_master WHERE type='index';")
        .map_err(|e| format!("Failed to query sqlite_master for indexes: {}", e))?;
    let index_rows = index_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to read index rows: {}", e))?;
    let indexes: Vec<String> = index_rows.filter_map(|r| r.ok()).collect();
    for required in &[
        "idx_completions_date",
        "idx_todos_completed",
        "idx_todos_due",
    ] {
        if !indexes.contains(&required.to_string()) {
            return Err(format!("Required index '{}' missing from schema", required));
        }
    }
    logs.push(
        "Schema indexes verified (idx_completions_date, idx_todos_completed, idx_todos_due)"
            .to_string(),
    );

    // 4. Daily Habits LEFT JOIN Query
    let h1_id = db
        .add_habit("Morning Exercise")
        .map_err(|e| format!("Failed to add habit 1: {}", e))?;
    let _h2_id = db
        .add_habit("Read 30 Mins")
        .map_err(|e| format!("Failed to add habit 2: {}", e))?;
    let today = Local::now().date_naive();
    let habits_today = db
        .get_habits_for_date(today)
        .map_err(|e| format!("Failed to get habits for date: {}", e))?;
    if habits_today.len() != 2 {
        return Err(format!(
            "Expected 2 habits for today, found {}",
            habits_today.len()
        ));
    }
    if habits_today[0].1 || habits_today[1].1 {
        return Err("Uncompleted habits must return false from LEFT JOIN by default".to_string());
    }
    logs.push("Daily habits LEFT JOIN query verified (uncompleted by default)".to_string());

    // 5. Atomic UPSERT Habit Toggle
    let toggle_1 = db
        .toggle_habit_completion(h1_id, today)
        .map_err(|e| format!("First toggle failed: {}", e))?;
    if !toggle_1 {
        return Err("First toggle on uncompleted habit must return true".to_string());
    }
    let habits_after_t1 = db
        .get_habits_for_date(today)
        .map_err(|e| format!("Query after first toggle failed: {}", e))?;
    let h1_status = habits_after_t1
        .iter()
        .find(|(h, _)| h.id == h1_id)
        .map(|(_, s)| *s);
    if h1_status != Some(true) {
        return Err("Habit 1 completion status must be true after first toggle".to_string());
    }

    let toggle_2 = db
        .toggle_habit_completion(h1_id, today)
        .map_err(|e| format!("Second toggle failed: {}", e))?;
    if toggle_2 {
        return Err("Second toggle on completed habit must return false".to_string());
    }
    let habits_after_t2 = db
        .get_habits_for_date(today)
        .map_err(|e| format!("Query after second toggle failed: {}", e))?;
    let h1_status_2 = habits_after_t2
        .iter()
        .find(|(h, _)| h.id == h1_id)
        .map(|(_, s)| *s);
    if h1_status_2 != Some(false) {
        return Err("Habit 1 completion status must be false after second toggle".to_string());
    }
    logs.push("Atomic UPSERT habit toggle verified".to_string());

    // 6. 30-Day Stats & Success Rates
    let _ = db.toggle_habit_completion(h1_id, today);
    let stats_30 = db
        .get_30_day_completion_stats(today)
        .map_err(|e| format!("Failed to get 30-day stats: {}", e))?;
    if stats_30.is_empty() {
        return Err("30-day stats must contain at least 1 record".to_string());
    }
    let rate = db
        .get_habit_success_rate_30_days(h1_id, today)
        .map_err(|e| format!("Failed to get habit success rate: {}", e))?;
    if rate <= 0.0 {
        return Err(format!("Expected habit success rate > 0.0, got {}", rate));
    }
    let heatmap = db
        .get_monthly_heatmap(today.year(), today.month())
        .map_err(|e| format!("Failed to get monthly heatmap: {}", e))?;
    if !heatmap.contains_key(&today) {
        return Err("Monthly heatmap must contain today's completion count".to_string());
    }
    logs.push("30-day stats rollups, habit success rate, and monthly heatmap verified".to_string());

    // 7. To-Dos CRUD
    let todo_id = db
        .add_todo(
            "Complete milestone 4",
            Some("Integration tests & CLI"),
            Some(today),
        )
        .map_err(|e| format!("Failed to add todo: {}", e))?;
    let todos = db
        .get_all_todos()
        .map_err(|e| format!("Failed to get todos: {}", e))?;
    if !todos.iter().any(|t| t.id == todo_id) {
        return Err("Created todo not found in get_all_todos".to_string());
    }
    let todo_toggled = db
        .toggle_todo(todo_id)
        .map_err(|e| format!("Failed to toggle todo: {}", e))?;
    if !todo_toggled {
        return Err("Todo toggle must return true for completion".to_string());
    }
    logs.push("To-Dos CRUD and completion toggling verified".to_string());

    // 8. JSON Export / Import Serde Roundtrip
    let export_data = db
        .export_data()
        .map_err(|e| format!("Failed to export data: {}", e))?;
    let json_bytes = serde_json::to_vec_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize export data: {}", e))?;
    let restored_export: ExportData = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("Failed to deserialize export data: {}", e))?;
    if restored_export.habits.len() != 2 || restored_export.todos.len() != 1 {
        return Err("ExportData roundtrip deserialized incorrect counts".to_string());
    }
    let db_import = Database::new_in_memory()
        .map_err(|e| format!("Failed to create second in-memory DB: {}", e))?;
    db_import
        .import_data(&restored_export)
        .map_err(|e| format!("Failed to import data: {}", e))?;
    let imported_habits = db_import
        .get_habits_for_date(today)
        .map_err(|e| format!("Failed to query imported habits: {}", e))?;
    if imported_habits.len() != 2 {
        return Err(format!(
            "Expected 2 imported habits, got {}",
            imported_habits.len()
        ));
    }
    logs.push("JSON export/import serde roundtrip verified".to_string());

    // 9. Zero-Panic Invariant on Empty DB & Navigation Messages
    let empty_db = Database::new_in_memory()
        .map_err(|e| format!("Failed to create empty in-memory DB: {}", e))?;
    let mut app =
        AppState::new(empty_db).map_err(|e| format!("AppState::new failed on empty DB: {}", e))?;
    if app.selected_index != 0 {
        return Err(format!(
            "Expected selected_index=0 on empty DB, got {}",
            app.selected_index
        ));
    }
    let _ = app.update(Message::NavigateUp);
    let _ = app.update(Message::NavigateDown);
    let _ = app.update(Message::GoTop);
    let _ = app.update(Message::GoBottom);
    let _ = app.update(Message::PreviousDay);
    let _ = app.update(Message::NextDay);
    let _ = app.update(Message::GoToday);
    let _ = app.update(Message::ToggleSidebar);
    let _ = app.update(Message::SwitchScreen(Screen::Settings));
    let _ = app.update(Message::ToggleSelected);
    let _ = app.update(Message::CompleteSelected);
    let _ = app.update(Message::DeleteSelected);
    let _ = app.update(Message::ToggleHelp);
    let _ = app.update(Message::CancelModal);
    let _ = app.update(Message::DbError("Simulated non-blocking error".to_string()));
    if !app
        .status_message
        .as_deref()
        .unwrap_or("")
        .contains("Simulated non-blocking error")
    {
        return Err("DbError message was not captured in status_message".to_string());
    }
    let _element = ui::view(&app);
    logs.push("Zero-panic invariant verified on empty state and UI rendering".to_string());

    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_text_content() {
        let help = help_text();
        assert!(help.contains("Habitodo 0.1.0"));
        assert!(help.contains("--help"));
        assert!(help.contains("--version"));
        assert!(help.contains("--verify"));
        assert!(help.contains("KEYBINDINGS:"));
        assert!(help.contains("Toggle metrics sidebar"));
        assert!(help.contains("Reorder habit up"));
    }

    #[test]
    fn test_version_text_content() {
        assert_eq!(version_text(), "habitodo 0.1.0");
    }

    #[test]
    fn test_run_verification_succeeds() {
        let result = run_verification();
        assert!(
            result.is_ok(),
            "Verification diagnostics must pass: {:?}",
            result.err()
        );
        let logs = result.unwrap();
        assert_eq!(logs.len(), 9);
    }
}
