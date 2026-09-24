use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_completed_true() -> bool {
    true
}

fn default_export_version() -> u32 {
    1
}

/// Represents a non-negotiable daily habit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Habit {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub position: i64,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub archived: bool,
}

/// Represents a habit completion record for a specific calendar date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HabitCompletion {
    pub habit_id: i64,
    pub date: NaiveDate,
    #[serde(default = "default_completed_true")]
    pub completed: bool,
}

/// Represents a free-form to-do item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub position: i64,
}

/// Aggregated habit completion statistics for a single calendar date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: NaiveDate,
    pub total_habits: usize,
    pub completed_habits: usize,
}

impl DailyStats {
    pub fn new(date: NaiveDate, total_habits: usize, completed_habits: usize) -> Self {
        Self {
            date,
            total_habits,
            completed_habits,
        }
    }

    /// Computes completion percentage from 0.0 to 100.0%.
    /// Safely handles the case where total_habits == 0 by returning 0.0 without division by zero.
    pub fn percentage(&self) -> f32 {
        if self.total_habits == 0 {
            0.0
        } else {
            (self.completed_habits as f32 / self.total_habits as f32) * 100.0
        }
    }

    /// Convenience alias returning completed habit count.
    pub fn completed_count(&self) -> usize {
        self.completed_habits
    }
}

/// Container for full database backup and export/import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportData {
    #[serde(default = "default_export_version")]
    pub version: u32,
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub habits: Vec<Habit>,
    #[serde(default, alias = "completions")]
    pub habit_completions: Vec<HabitCompletion>,
    #[serde(default)]
    pub todos: Vec<Todo>,
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl Default for ExportData {
    fn default() -> Self {
        Self {
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            habits: Vec::new(),
            habit_completions: Vec::new(),
            todos: Vec::new(),
            settings: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_stats_percentage_zero_habits() {
        let stats = DailyStats {
            date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
            total_habits: 0,
            completed_habits: 0,
        };
        assert_eq!(stats.percentage(), 0.0);
    }

    #[test]
    fn test_daily_stats_percentage_calculations() {
        let stats = DailyStats {
            date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
            total_habits: 4,
            completed_habits: 3,
        };
        assert_eq!(stats.percentage(), 75.0);

        let stats_full = DailyStats {
            date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
            total_habits: 5,
            completed_habits: 5,
        };
        assert_eq!(stats_full.percentage(), 100.0);

        let stats_none = DailyStats {
            date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
            total_habits: 10,
            completed_habits: 0,
        };
        assert_eq!(stats_none.percentage(), 0.0);
    }

    #[test]
    fn test_habit_serde_roundtrip() {
        let habit = Habit {
            id: 1,
            name: "Morning Meditation".to_string(),
            position: 0,
            created_at: "2026-09-23 08:00:00".to_string(),
            archived: false,
        };
        let json = serde_json::to_string(&habit).expect("serialize habit");
        let deserialized: Habit = serde_json::from_str(&json).expect("deserialize habit");
        assert_eq!(habit, deserialized);
    }

    #[test]
    fn test_todo_serde_with_optional_fields() {
        let todo = Todo {
            id: 42,
            title: "Review Rust PR".to_string(),
            notes: Some("Check memory safety".to_string()),
            completed: true,
            created_at: "2026-09-23T12:00:00Z".to_string(),
            completed_at: Some("2026-09-23T14:30:00Z".to_string()),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 9, 25).unwrap()),
            position: 2,
        };
        let json = serde_json::to_string(&todo).expect("serialize todo");
        let deserialized: Todo = serde_json::from_str(&json).expect("deserialize todo");
        assert_eq!(todo, deserialized);

        // Deserializing minimal JSON with missing optional fields
        let minimal_json = r#"{"id": 43, "title": "Quick task"}"#;
        let min_todo: Todo = serde_json::from_str(minimal_json).expect("deserialize minimal todo");
        assert_eq!(min_todo.id, 43);
        assert_eq!(min_todo.title, "Quick task");
        assert_eq!(min_todo.notes, None);
        assert!(!min_todo.completed);
        assert_eq!(min_todo.completed_at, None);
        assert_eq!(min_todo.due_date, None);
        assert_eq!(min_todo.position, 0);
    }

    #[test]
    fn test_export_data_serde_roundtrip_and_alias() {
        let mut settings = HashMap::new();
        settings.insert("theme".to_string(), "dark".to_string());

        let export = ExportData {
            version: 1,
            exported_at: "2026-09-23T22:00:00Z".to_string(),
            habits: vec![Habit {
                id: 1,
                name: "Exercise".to_string(),
                position: 0,
                created_at: "2026-09-01T00:00:00Z".to_string(),
                archived: false,
            }],
            habit_completions: vec![HabitCompletion {
                habit_id: 1,
                date: NaiveDate::from_ymd_opt(2026, 9, 23).unwrap(),
                completed: true,
            }],
            todos: vec![],
            settings,
        };

        let json = serde_json::to_string_pretty(&export).expect("serialize export");
        let deserialized: ExportData = serde_json::from_str(&json).expect("deserialize export");
        assert_eq!(export, deserialized);

        // Test alias "completions" instead of "habit_completions"
        let legacy_json = r#"{
            "exported_at": "2026-09-23T22:00:00Z",
            "habits": [],
            "completions": [
                {"habit_id": 1, "date": "2026-09-23", "completed": true}
            ],
            "todos": []
        }"#;
        let parsed: ExportData =
            serde_json::from_str(legacy_json).expect("deserialize legacy json");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.habit_completions.len(), 1);
        assert_eq!(parsed.habit_completions[0].habit_id, 1);
        assert!(parsed.habit_completions[0].completed);
    }
}
