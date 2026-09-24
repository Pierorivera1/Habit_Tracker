use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

use crate::domain::{DailyStats, ExportData, Habit, HabitCompletion, Todo};

/// Retrieves all active (non-archived) habits for the given date, with their completion status.
/// Defaults unrecorded habits to completed = false via LEFT JOIN and COALESCE.
pub fn get_habits_for_date(conn: &Connection, date: NaiveDate) -> Result<Vec<(Habit, bool)>> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let mut stmt = conn.prepare(
        "
        SELECT 
            h.id, 
            h.name, 
            h.position, 
            h.created_at, 
            h.archived, 
            COALESCE(c.completed, 0) AS completed
        FROM habits h
        LEFT JOIN habit_completions c 
            ON c.habit_id = h.id AND c.date = ?1
        WHERE h.archived = 0
        ORDER BY h.position ASC, h.id ASC;
        ",
    )?;

    let rows = stmt.query_map(params![date_str], |row| {
        let habit = Habit {
            id: row.get(0)?,
            name: row.get(1)?,
            position: row.get(2)?,
            created_at: row.get(3)?,
            archived: row.get::<_, i64>(4)? != 0,
        };
        let completed = row.get::<_, i64>(5)? != 0;
        Ok((habit, completed))
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// Atomically toggles a habit completion status for a given date.
/// If no record exists, inserts completed = 1 (true).
/// If a record exists, flips 1 -> 0 or 0 -> 1.
/// Returns the new completion status (true if completed, false if uncompleted).
pub fn toggle_habit_completion(conn: &Connection, habit_id: i64, date: NaiveDate) -> Result<bool> {
    let date_str = date.format("%Y-%m-%d").to_string();

    // Atomic UPSERT toggling the completed flag
    conn.execute(
        "
        INSERT INTO habit_completions (habit_id, date, completed)
        VALUES (?1, ?2, 1)
        ON CONFLICT(habit_id, date)
        DO UPDATE SET completed = 1 - habit_completions.completed;
        ",
        params![habit_id, date_str],
    )?;

    // Fetch the updated status
    let new_status: i64 = conn.query_row(
        "SELECT completed FROM habit_completions WHERE habit_id = ?1 AND date = ?2;",
        params![habit_id, date_str],
        |row| row.get(0),
    )?;

    Ok(new_status != 0)
}

/// Returns aggregated daily completion stats for the past 30 days ending at `end_date`.
pub fn get_30_day_completion_stats(
    conn: &Connection,
    end_date: NaiveDate,
) -> Result<Vec<DailyStats>> {
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let mut stmt = conn.prepare(
        "
        SELECT 
            date,
            CAST(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) AS INTEGER) AS completed_count,
            CAST(COUNT(*) AS INTEGER) AS total_count
        FROM habit_completions
        WHERE date >= date(?1, '-30 days') AND date <= ?1
        GROUP BY date
        ORDER BY date ASC;
        ",
    )?;

    let rows = stmt.query_map(params![end_str], |row| {
        let date_str: String = row.get(0)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or(end_date);
        let completed: i64 = row.get(1)?;
        let total: i64 = row.get(2)?;
        Ok(DailyStats {
            date,
            total_habits: total as usize,
            completed_habits: completed as usize,
        })
    })?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }
    Ok(stats)
}

/// Returns the 30-day success rate (0.0 to 100.0%) for a specific habit up to `end_date`.
/// Calculates completed days out of 30.
pub fn get_habit_success_rate_30_days(
    conn: &Connection,
    habit_id: i64,
    end_date: NaiveDate,
) -> Result<f32> {
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let completed_days: i64 = conn.query_row(
        "
        SELECT COUNT(*)
        FROM habit_completions
        WHERE habit_id = ?1
          AND completed = 1
          AND date >= date(?2, '-29 days')
          AND date <= ?2;
        ",
        params![habit_id, end_str],
        |row| row.get(0),
    )?;

    let rate = (completed_days as f32 / 30.0) * 100.0;
    Ok(rate.min(100.0))
}

/// Returns habit completion count per day for the specified month.
pub fn get_monthly_heatmap(
    conn: &Connection,
    year: i32,
    month: u32,
) -> Result<HashMap<NaiveDate, usize>> {
    if !(1..=12).contains(&month) {
        return Ok(HashMap::new());
    }

    let start_date = match NaiveDate::from_ymd_opt(year, month, 1) {
        Some(d) => d,
        None => return Ok(HashMap::new()),
    };

    let (next_year, next_month) = if month == 12 {
        (year.checked_add(1), 1)
    } else {
        (Some(year), month + 1)
    };

    let next_month_start = next_year.and_then(|y| NaiveDate::from_ymd_opt(y, next_month, 1));
    let end_date = match next_month_start {
        Some(nms) => nms.pred_opt().unwrap_or(start_date),
        None => NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(start_date),
    };

    let start_date_str = start_date.format("%Y-%m-%d").to_string();
    let end_date_str = end_date.format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "
        SELECT 
            date,
            CAST(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) AS INTEGER) AS completed_count
        FROM habit_completions
        WHERE date >= ?1 AND date <= ?2
        GROUP BY date;
        ",
    )?;

    let rows = stmt.query_map(params![start_date_str, end_date_str], |row| {
        let d_str: String = row.get(0)?;
        let date = NaiveDate::parse_from_str(&d_str, "%Y-%m-%d").ok();
        let count: i64 = row.get(1)?;
        Ok((date, count as usize))
    })?;

    let mut heatmap = HashMap::new();
    for row in rows {
        let (maybe_date, count) = row?;
        if let Some(date) = maybe_date {
            heatmap.insert(date, count);
        }
    }

    Ok(heatmap)
}

/// Returns all to-dos ordered with uncompleted first, then by position and id.
pub fn get_all_todos(conn: &Connection) -> Result<Vec<Todo>> {
    let mut stmt = conn.prepare(
        "
        SELECT 
            id, 
            title, 
            notes, 
            completed, 
            created_at, 
            completed_at, 
            due_date, 
            position
        FROM todos
        ORDER BY completed ASC, position ASC, id ASC;
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let notes: Option<String> = row.get(2)?;
        let completed: bool = row.get::<_, i64>(3)? != 0;
        let created_at: String = row.get(4)?;
        let completed_at: Option<String> = row.get(5)?;
        let due_date_str: Option<String> = row.get(6)?;
        let due_date = due_date_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        let position: i64 = row.get(7)?;

        Ok(Todo {
            id,
            title,
            notes,
            completed,
            created_at,
            completed_at,
            due_date,
            position,
        })
    })?;

    let mut todos = Vec::new();
    for row in rows {
        todos.push(row?);
    }
    Ok(todos)
}

/// Inserts a new to-do item and returns its newly created ID.
pub fn add_todo(
    conn: &Connection,
    title: &str,
    notes: Option<&str>,
    due_date: Option<NaiveDate>,
) -> Result<i64> {
    let due_date_str = due_date.map(|d| d.format("%Y-%m-%d").to_string());
    conn.execute(
        "
        INSERT INTO todos (title, notes, completed, created_at, due_date, position)
        VALUES (
            ?1,
            ?2,
            0,
            datetime('now'),
            ?3,
            COALESCE((SELECT MAX(position) + 1 FROM todos), 0)
        );
        ",
        params![title, notes, due_date_str],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Toggles a to-do item's completion status.
/// Sets completed_at to the current timestamp when completed, or NULL when uncompleted.
/// Returns the new completed state.
pub fn toggle_todo(conn: &Connection, todo_id: i64) -> Result<bool> {
    let current_completed: i64 = conn.query_row(
        "SELECT completed FROM todos WHERE id = ?1;",
        params![todo_id],
        |row| row.get(0),
    )?;

    let new_completed = if current_completed == 0 { 1 } else { 0 };

    conn.execute(
        "
        UPDATE todos
        SET completed = ?2,
            completed_at = CASE WHEN ?2 = 1 THEN datetime('now') ELSE NULL END
        WHERE id = ?1;
        ",
        params![todo_id, new_completed],
    )?;

    Ok(new_completed != 0)
}

/// Deletes a to-do item by ID.
pub fn delete_todo(conn: &Connection, todo_id: i64) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1;", params![todo_id])?;
    Ok(())
}

/// Updates the notes of a to-do item.
pub fn update_todo_notes(conn: &Connection, todo_id: i64, notes: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE todos SET notes = ?2 WHERE id = ?1;",
        params![todo_id, notes],
    )?;
    Ok(())
}

/// Updates the title of a to-do item.
pub fn update_todo_title(conn: &Connection, todo_id: i64, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE todos SET title = ?2 WHERE id = ?1;",
        params![todo_id, title],
    )?;
    Ok(())
}

/// Updates the due date of a to-do item.
pub fn update_todo_due_date(
    conn: &Connection,
    todo_id: i64,
    due_date: Option<NaiveDate>,
) -> Result<()> {
    let due_date_str = due_date.map(|d| d.format("%Y-%m-%d").to_string());
    conn.execute(
        "UPDATE todos SET due_date = ?2 WHERE id = ?1;",
        params![todo_id, due_date_str],
    )?;
    Ok(())
}

/// Inserts a new habit and returns its newly created ID.
pub fn add_habit(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "
        INSERT INTO habits (name, position, archived)
        VALUES (?1, COALESCE((SELECT MAX(position) + 1 FROM habits), 0), 0);
        ",
        params![name],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Soft-deletes a habit by setting archived = 1.
pub fn archive_habit(conn: &Connection, habit_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE habits SET archived = 1 WHERE id = ?1;",
        params![habit_id],
    )?;
    Ok(())
}

/// Renames a habit.
pub fn rename_habit(conn: &Connection, habit_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE habits SET name = ?2 WHERE id = ?1;",
        params![habit_id, new_name],
    )?;
    Ok(())
}

/// Reorders a habit up or down among active habits.
/// Normalizes positions on reorder to guarantee consistent sequential ordering.
pub fn reorder_habit(conn: &Connection, habit_id: i64, move_up: bool) -> Result<()> {
    // Fetch all active habits in their current display order
    let mut stmt =
        conn.prepare("SELECT id FROM habits WHERE archived = 0 ORDER BY position ASC, id ASC;")?;
    let mut ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    let index = match ids.iter().position(|&id| id == habit_id) {
        Some(idx) => idx,
        None => return Ok(()), // Habit not found or archived; no-op
    };

    if move_up {
        if index > 0 {
            ids.swap(index, index - 1);
        } else {
            return Ok(());
        }
    } else if index + 1 < ids.len() {
        ids.swap(index, index + 1);
    } else {
        return Ok(());
    }

    // Persist new normalized positions inside an explicit transaction
    let res = (|| -> Result<()> {
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        for (pos, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE habits SET position = ?1 WHERE id = ?2;",
                params![pos as i64, id],
            )?;
        }
        conn.execute_batch("COMMIT;")?;
        Ok(())
    })();

    if let Err(e) = res {
        let _ = conn.execute_batch("ROLLBACK;");
        return Err(e);
    }

    Ok(())
}

/// Exports the full database contents into `ExportData`.
pub fn export_data(conn: &Connection) -> Result<ExportData> {
    // 1. Habits
    let mut h_stmt = conn.prepare(
        "SELECT id, name, position, created_at, archived FROM habits ORDER BY position ASC, id ASC;",
    )?;
    let habits = h_stmt
        .query_map([], |row| {
            Ok(Habit {
                id: row.get(0)?,
                name: row.get(1)?,
                position: row.get(2)?,
                created_at: row.get(3)?,
                archived: row.get::<_, i64>(4)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // 2. Completions
    let mut c_stmt = conn.prepare(
        "SELECT habit_id, date, completed FROM habit_completions ORDER BY date ASC, habit_id ASC;",
    )?;
    let habit_completions = c_stmt
        .query_map([], |row| {
            let date_str: String = row.get(1)?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_default();
            Ok(HabitCompletion {
                habit_id: row.get(0)?,
                date,
                completed: row.get::<_, i64>(2)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // 3. Todos
    let todos = get_all_todos(conn)?;

    // 4. Settings
    let mut s_stmt = conn.prepare("SELECT key, value FROM settings;")?;
    let settings_rows = s_stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let val: String = row.get(1)?;
        Ok((key, val))
    })?;

    let mut settings = HashMap::new();
    for row in settings_rows {
        let (k, v) = row?;
        settings.insert(k, v);
    }

    Ok(ExportData {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        habits,
        habit_completions,
        todos,
        settings,
    })
}

/// Imports data into the database from an `ExportData` container within an atomic transaction.
pub fn import_data(conn: &Connection, data: &ExportData) -> Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;

    let import_result = (|| -> Result<()> {
        // Insert or replace habits
        for h in &data.habits {
            conn.execute(
                "
                INSERT OR REPLACE INTO habits (id, name, position, created_at, archived)
                VALUES (?1, ?2, ?3, ?4, ?5);
                ",
                params![
                    h.id,
                    h.name,
                    h.position,
                    h.created_at,
                    if h.archived { 1 } else { 0 }
                ],
            )?;
        }

        // Insert or replace completions
        for c in &data.habit_completions {
            let date_str = c.date.format("%Y-%m-%d").to_string();
            conn.execute(
                "
                INSERT OR REPLACE INTO habit_completions (habit_id, date, completed)
                VALUES (?1, ?2, ?3);
                ",
                params![c.habit_id, date_str, if c.completed { 1 } else { 0 }],
            )?;
        }

        // Insert or replace todos
        for t in &data.todos {
            let due_date_str = t.due_date.map(|d| d.format("%Y-%m-%d").to_string());
            conn.execute(
                "
                INSERT OR REPLACE INTO todos (id, title, notes, completed, created_at, completed_at, due_date, position)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
                ",
                params![
                    t.id,
                    t.title,
                    t.notes,
                    if t.completed { 1 } else { 0 },
                    t.created_at,
                    t.completed_at,
                    due_date_str,
                    t.position,
                ],
            )?;
        }

        // Insert or replace settings
        for (k, v) in &data.settings {
            conn.execute(
                "
                INSERT OR REPLACE INTO settings (key, value)
                VALUES (?1, ?2);
                ",
                params![k, v],
            )?;
        }

        Ok(())
    })();

    match import_result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        }
    }
}

/// Retrieves an application setting value by key.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1;")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

/// Stores or updates an application setting value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "
        INSERT INTO settings (key, value)
        VALUES (?1, ?2)
        ON CONFLICT(key)
        DO UPDATE SET value = excluded.value;
        ",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{apply_pragmas, init_schema};
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open memory db");
        apply_pragmas(&conn).expect("pragmas");
        init_schema(&conn).expect("schema");
        conn
    }

    #[test]
    fn test_get_habits_for_date_left_join() {
        let conn = setup_test_db();
        let id1 = add_habit(&conn, "Habit 1").expect("add 1");
        let id2 = add_habit(&conn, "Habit 2").expect("add 2");
        let id3 = add_habit(&conn, "Habit 3").expect("add 3");

        let today = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();
        // Toggle only Habit 2
        let status = toggle_habit_completion(&conn, id2, today).expect("toggle");
        assert!(status);

        let list = get_habits_for_date(&conn, today).expect("get habits");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].0.id, id1);
        assert!(!list[0].1); // unrecorded defaults to false

        assert_eq!(list[1].0.id, id2);
        assert!(list[1].1); // toggled is true

        assert_eq!(list[2].0.id, id3);
        assert!(!list[2].1); // unrecorded defaults to false
    }

    #[test]
    fn test_toggle_habit_atomic_upsert() {
        let conn = setup_test_db();
        let hid = add_habit(&conn, "Daily Walk").expect("add habit");
        let date = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();

        // 1st toggle: false -> true
        let s1 = toggle_habit_completion(&conn, hid, date).expect("toggle 1");
        assert!(s1);

        // 2nd toggle: true -> false
        let s2 = toggle_habit_completion(&conn, hid, date).expect("toggle 2");
        assert!(!s2);

        // 3rd toggle: false -> true
        let s3 = toggle_habit_completion(&conn, hid, date).expect("toggle 3");
        assert!(s3);

        // Verify exactly 1 row exists in habit_completions
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM habit_completions WHERE habit_id = ?1 AND date = ?2;",
                params![hid, "2026-09-23"],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_30_day_stats_and_success_rate() {
        let conn = setup_test_db();
        let hid1 = add_habit(&conn, "Read").expect("add 1");
        let hid2 = add_habit(&conn, "Exercise").expect("add 2");

        let today = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();

        // Mark 15 days for hid1 and 10 days for hid2
        for i in 0..15 {
            let d = today - chrono::Duration::days(i);
            toggle_habit_completion(&conn, hid1, d).expect("toggle hid1");
        }
        for i in 0..10 {
            let d = today - chrono::Duration::days(i);
            toggle_habit_completion(&conn, hid2, d).expect("toggle hid2");
        }

        let stats = get_30_day_completion_stats(&conn, today).expect("stats");
        // Should have 15 days of records
        assert_eq!(stats.len(), 15);

        // For the latest 10 days, total = 2, completed = 2 (100%)
        let latest = stats.last().unwrap();
        assert_eq!(latest.date, today);
        assert_eq!(latest.total_habits, 2);
        assert_eq!(latest.completed_habits, 2);
        assert_eq!(latest.percentage(), 100.0);

        // Success rate for hid1: 15 / 30 = 50.0%
        let rate1 = get_habit_success_rate_30_days(&conn, hid1, today).expect("rate1");
        assert_eq!(rate1, 50.0);

        // Success rate for hid2: 10 / 30 = 33.333332%
        let rate2 = get_habit_success_rate_30_days(&conn, hid2, today).expect("rate2");
        assert!((rate2 - 33.333332).abs() < 0.01);
    }

    #[test]
    fn test_monthly_heatmap() {
        let conn = setup_test_db();
        let hid = add_habit(&conn, "Code").expect("add habit");

        let d1 = NaiveDate::from_ymd_opt(2026, 9, 5).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 9, 12).unwrap();
        let d_oct = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();

        toggle_habit_completion(&conn, hid, d1).expect("toggle 1");
        toggle_habit_completion(&conn, hid, d2).expect("toggle 2");
        toggle_habit_completion(&conn, hid, d_oct).expect("toggle oct");

        let heatmap = get_monthly_heatmap(&conn, 2026, 9).expect("heatmap");
        assert_eq!(heatmap.get(&d1), Some(&1));
        assert_eq!(heatmap.get(&d2), Some(&1));
        assert_eq!(heatmap.get(&d_oct), None); // Oct should not appear in Sep heatmap
    }

    #[test]
    fn test_todo_crud_and_timestamps() {
        let conn = setup_test_db();
        let due = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let tid = add_todo(&conn, "Write Report", Some("Urgent"), Some(due)).expect("add todo");

        let todos = get_all_todos(&conn).expect("get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, tid);
        assert_eq!(todos[0].title, "Write Report");
        assert_eq!(todos[0].notes.as_deref(), Some("Urgent"));
        assert!(!todos[0].completed);
        assert_eq!(todos[0].completed_at, None);
        assert_eq!(todos[0].due_date, Some(due));

        // Toggle to completed
        let new_state = toggle_todo(&conn, tid).expect("toggle todo");
        assert!(new_state);

        let todos_after = get_all_todos(&conn).expect("get todos 2");
        assert!(todos_after[0].completed);
        assert!(todos_after[0].completed_at.is_some());

        // Update notes
        update_todo_notes(&conn, tid, Some("Updated notes")).expect("update notes");
        let todos_after_notes = get_all_todos(&conn).expect("get todos 3");
        assert_eq!(todos_after_notes[0].notes.as_deref(), Some("Updated notes"));

        // Delete todo
        delete_todo(&conn, tid).expect("delete todo");
        let todos_empty = get_all_todos(&conn).expect("get todos 4");
        assert_eq!(todos_empty.len(), 0);
    }

    #[test]
    fn test_habit_reordering_and_archiving() {
        let conn = setup_test_db();
        let h1 = add_habit(&conn, "Habit 1").expect("add 1");
        let h2 = add_habit(&conn, "Habit 2").expect("add 2");
        let h3 = add_habit(&conn, "Habit 3").expect("add 3");

        let today = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();
        let habits = get_habits_for_date(&conn, today).expect("habits");
        assert_eq!(habits[0].0.id, h1);
        assert_eq!(habits[1].0.id, h2);
        assert_eq!(habits[2].0.id, h3);

        // Move Habit 2 up -> Habit 2 should now be first
        reorder_habit(&conn, h2, true).expect("reorder up");
        let habits_reordered = get_habits_for_date(&conn, today).expect("habits");
        assert_eq!(habits_reordered[0].0.id, h2);
        assert_eq!(habits_reordered[1].0.id, h1);
        assert_eq!(habits_reordered[2].0.id, h3);

        // Archive Habit 1 -> should no longer appear in daily habits
        archive_habit(&conn, h1).expect("archive");
        let habits_after_archive = get_habits_for_date(&conn, today).expect("habits");
        assert_eq!(habits_after_archive.len(), 2);
        assert_eq!(habits_after_archive[0].0.id, h2);
        assert_eq!(habits_after_archive[1].0.id, h3);
    }

    #[test]
    fn test_export_and_import_roundtrip() {
        let conn = setup_test_db();
        let h1 = add_habit(&conn, "Hydrate").expect("add habit");
        let date = NaiveDate::from_ymd_opt(2026, 9, 23).unwrap();
        toggle_habit_completion(&conn, h1, date).expect("toggle");

        let _tid = add_todo(&conn, "Buy Groceries", None, None).expect("add todo");
        set_setting(&conn, "theme", "nord").expect("set setting");

        let export = export_data(&conn).expect("export data");
        assert_eq!(export.habits.len(), 1);
        assert_eq!(export.habit_completions.len(), 1);
        assert_eq!(export.todos.len(), 1);
        assert_eq!(
            export.settings.get("theme").map(|s| s.as_str()),
            Some("nord")
        );

        // Now import into a fresh database
        let conn2 = setup_test_db();
        import_data(&conn2, &export).expect("import data");

        let habits2 = get_habits_for_date(&conn2, date).expect("habits2");
        assert_eq!(habits2.len(), 1);
        assert_eq!(habits2[0].0.name, "Hydrate");
        assert!(habits2[0].1); // completed

        let todos2 = get_all_todos(&conn2).expect("todos2");
        assert_eq!(todos2.len(), 1);
        assert_eq!(todos2[0].title, "Buy Groceries");

        let setting2 = get_setting(&conn2, "theme").expect("get setting");
        assert_eq!(setting2.as_deref(), Some("nord"));
    }
}
