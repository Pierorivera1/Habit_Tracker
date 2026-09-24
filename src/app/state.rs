//! Central application state, screen management, and Elm update lifecycle.

use crate::app::message::Message;
use crate::db::Database;
use crate::domain::{DailyStats, Habit, Todo};
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// Identifies the active screen / view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Main dashboard displaying daily habits, to-dos, and metrics sidebar.
    #[default]
    Main,
    /// Settings screen for managing habits (reordering, archiving, adding) and backups.
    Settings,
}

/// Identifies the current keyboard input mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode where single-key shortcuts are active.
    #[default]
    Normal,
    /// Actively typing a new to-do task title in the inline input.
    AddingTodo,
    /// Actively editing an existing to-do task title.
    EditingTodo(i64),
    /// Actively editing attached notes for a to-do task.
    EditingNotes(i64),
    /// Actively typing a new habit name in Settings.
    AddingHabit,
    /// Actively renaming an existing habit in Settings.
    RenamingHabit(i64),
    /// Waiting for confirmation `[y/n]` to delete a to-do task.
    ConfirmDelete(i64),
}

/// Central application state holding UI state, working caches, and the database connection.
pub struct AppState {
    /// Currently displayed screen.
    pub screen: Screen,
    /// Whether the right metrics sidebar is open.
    pub sidebar_open: bool,
    /// Currently inspected calendar date (defaults to today).
    pub selected_date: NaiveDate,
    /// Unified cursor selection index for traversing lists via j/k.
    pub selected_index: usize,
    /// Active habits for `selected_date` paired with their completion status for that date.
    pub habits: Vec<(Habit, bool)>,
    /// All active to-dos loaded from SQLite.
    pub todos: Vec<Todo>,
    /// 30-day historical completion statistics ending on `selected_date`.
    pub daily_stats_30_days: Vec<DailyStats>,
    /// Per-habit 30-day consistency rates ending on `selected_date` (habit_id -> percentage).
    pub habit_success_rates: Vec<(i64, f32)>,
    /// Monthly heatmap habit completion count map for year and month of `selected_date`.
    pub monthly_heatmap: HashMap<NaiveDate, usize>,
    /// Active keyboard input mode (isolates text input from single-key shortcuts).
    pub input_mode: InputMode,
    /// Working buffer for inline text inputs (task titles, habit names).
    pub input_buffer: String,
    /// Working buffer for task notes.
    pub notes_buffer: String,
    /// Optional status / toast message shown in the status bar or notification banner.
    pub status_message: Option<String>,
    /// Whether the keyboard shortcut help modal is currently visible.
    pub show_help: bool,
    /// Database connection handle.
    pub db: Database,
}

impl AppState {
    /// Creates a new AppState instance, initializing state and loading all records for today.
    /// Returns `Err(String)` if database initial queries fail.
    pub fn new(db: Database) -> Result<Self, String> {
        let today = chrono::Local::now().date_naive();
        let habits = db
            .get_habits_for_date(today)
            .map_err(|e| format!("Failed to load habits for today: {}", e))?;
        let todos = db
            .get_all_todos()
            .map_err(|e| format!("Failed to load todos: {}", e))?;
        let daily_stats_30_days = db
            .get_30_day_completion_stats(today)
            .map_err(|e| format!("Failed to load 30-day stats: {}", e))?;

        let mut habit_success_rates = Vec::with_capacity(habits.len());
        for (habit, _) in &habits {
            let rate = db
                .get_habit_success_rate_30_days(habit.id, today)
                .map_err(|e| format!("Failed to load habit success rate: {}", e))?;
            habit_success_rates.push((habit.id, rate));
        }

        let monthly_heatmap = db
            .get_monthly_heatmap(today.year(), today.month())
            .map_err(|e| format!("Failed to load monthly heatmap: {}", e))?;

        Ok(Self {
            screen: Screen::Main,
            sidebar_open: true,
            selected_date: today,
            selected_index: 0,
            habits,
            todos,
            daily_stats_30_days,
            habit_success_rates,
            monthly_heatmap,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            notes_buffer: String::new(),
            status_message: None,
            show_help: false,
            db,
        })
    }

    /// Returns the total count of selectable items for the currently active screen.
    pub fn total_items_for_screen(&self) -> usize {
        match self.screen {
            Screen::Main => self.habits.len() + self.todos.len(),
            Screen::Settings => self.habits.len(),
        }
    }

    /// Clamps `selected_index` so it remains within valid bounds `0..total_items`.
    pub fn clamp_selected_index(&mut self) {
        let total = self.total_items_for_screen();
        if total == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= total {
            self.selected_index = total - 1;
        }
    }

    /// Reloads all in-memory data for `selected_date` from SQLite.
    pub fn reload_data(&mut self) -> Result<(), rusqlite::Error> {
        self.habits = self.db.get_habits_for_date(self.selected_date)?;
        self.todos = self.db.get_all_todos()?;
        self.daily_stats_30_days = self.db.get_30_day_completion_stats(self.selected_date)?;

        let mut rates = Vec::with_capacity(self.habits.len());
        for (habit, _) in &self.habits {
            let rate = self
                .db
                .get_habit_success_rate_30_days(habit.id, self.selected_date)?;
            rates.push((habit.id, rate));
        }
        self.habit_success_rates = rates;

        self.monthly_heatmap = self
            .db
            .get_monthly_heatmap(self.selected_date.year(), self.selected_date.month())?;

        self.clamp_selected_index();
        Ok(())
    }

    /// Returns reference to selected habit if cursor points to a habit.
    pub fn selected_habit(&self) -> Option<&(Habit, bool)> {
        match self.screen {
            Screen::Main | Screen::Settings => self.habits.get(self.selected_index),
        }
    }

    /// Returns reference to selected todo if cursor in Main view points to a todo.
    pub fn selected_todo(&self) -> Option<&Todo> {
        match self.screen {
            Screen::Main => {
                if self.selected_index >= self.habits.len() {
                    let todo_idx = self.selected_index - self.habits.len();
                    self.todos.get(todo_idx)
                } else {
                    None
                }
            }
            Screen::Settings => None,
        }
    }

    /// Central Elm update function. Handles user interactions, updates in-memory state,
    /// persists mutations immediately to SQLite, and returns an iced Task.
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            // -------------------------------------------------------------
            // Navigation
            // -------------------------------------------------------------
            Message::NavigateUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                iced::Task::none()
            }
            Message::NavigateDown => {
                let total = self.total_items_for_screen();
                if total > 0 && self.selected_index + 1 < total {
                    self.selected_index += 1;
                }
                iced::Task::none()
            }
            Message::PreviousDay => {
                if let Some(prev) = self.selected_date.pred_opt() {
                    self.selected_date = prev;
                    if let Err(e) = self.reload_data() {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::NextDay => {
                if let Some(next) = self.selected_date.succ_opt() {
                    self.selected_date = next;
                    if let Err(e) = self.reload_data() {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::GoToday => {
                let today = chrono::Local::now().date_naive();
                if self.selected_date != today {
                    self.selected_date = today;
                    if let Err(e) = self.reload_data() {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::GoTop => {
                self.selected_index = 0;
                iced::Task::none()
            }
            Message::GoBottom => {
                let total = self.total_items_for_screen();
                if total > 0 {
                    self.selected_index = total - 1;
                } else {
                    self.selected_index = 0;
                }
                iced::Task::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                iced::Task::none()
            }
            Message::SwitchScreen(screen) => {
                self.screen = screen;
                self.selected_index = 0;
                self.clamp_selected_index();
                iced::Task::none()
            }
            Message::OpenSettings => {
                self.screen = Screen::Settings;
                self.selected_index = 0;
                self.clamp_selected_index();
                iced::Task::none()
            }
            Message::CloseSettings => {
                self.screen = Screen::Main;
                self.selected_index = 0;
                self.clamp_selected_index();
                iced::Task::none()
            }
            Message::ToggleHelp => {
                self.show_help = !self.show_help;
                iced::Task::none()
            }
            Message::Quit => iced::exit(),

            // -------------------------------------------------------------
            // Habit Actions (Write-Through)
            // -------------------------------------------------------------
            Message::ToggleHabit(habit_id) => {
                match self
                    .db
                    .toggle_habit_completion(habit_id, self.selected_date)
                {
                    Ok(_) => {
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::AddHabit(name) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    self.input_mode = InputMode::Normal;
                    return iced::Task::none();
                }
                match self.db.add_habit(trimmed) {
                    Ok(_) => {
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::ArchiveHabit(habit_id) => {
                match self.db.archive_habit(habit_id) {
                    Ok(()) => {
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::RenameHabit(habit_id, new_name) => {
                let trimmed = new_name.trim();
                if trimmed.is_empty() {
                    self.input_mode = InputMode::Normal;
                    return iced::Task::none();
                }
                match self.db.rename_habit(habit_id, trimmed) {
                    Ok(()) => {
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::ReorderHabit(habit_id, move_up) => {
                match self.db.reorder_habit(habit_id, move_up) {
                    Ok(()) => {
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                        if move_up {
                            if self.selected_index > 0 {
                                self.selected_index -= 1;
                            }
                        } else if self.selected_index + 1 < self.habits.len() {
                            self.selected_index += 1;
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }

            // -------------------------------------------------------------
            // To-Do Actions (Write-Through)
            // -------------------------------------------------------------
            Message::ToggleTodo(todo_id) => {
                match self.db.toggle_todo(todo_id) {
                    Ok(_) => {
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::AddTodo {
                title,
                notes,
                due_date,
            } => {
                let trimmed = title.trim();
                if trimmed.is_empty() {
                    self.input_mode = InputMode::Normal;
                    return iced::Task::none();
                }
                match self.db.add_todo(trimmed, notes.as_deref(), due_date) {
                    Ok(_) => {
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                        self.notes_buffer.clear();
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::DeleteTodo(todo_id) => {
                match self.db.delete_todo(todo_id) {
                    Ok(()) => {
                        self.input_mode = InputMode::Normal;
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::UpdateTodoNotes(todo_id, notes) => {
                let trimmed = notes.trim();
                let notes_opt = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                };
                match self.db.update_todo_notes(todo_id, notes_opt) {
                    Ok(()) => {
                        self.input_mode = InputMode::Normal;
                        self.notes_buffer.clear();
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }

            // -------------------------------------------------------------
            // Contextual Selection Actions
            // -------------------------------------------------------------
            Message::ToggleSelected => match self.screen {
                Screen::Main => {
                    if self.selected_index < self.habits.len() {
                        let habit_id = self.habits[self.selected_index].0.id;
                        self.update(Message::ToggleHabit(habit_id))
                    } else {
                        let todo_idx = self.selected_index.saturating_sub(self.habits.len());
                        if let Some(todo) = self.todos.get(todo_idx) {
                            let todo_id = todo.id;
                            self.update(Message::ToggleTodo(todo_id))
                        } else {
                            iced::Task::none()
                        }
                    }
                }
                Screen::Settings => iced::Task::none(),
            },
            Message::CompleteSelected => match self.screen {
                Screen::Main => {
                    if self.selected_index < self.habits.len() {
                        let (habit, completed) = &self.habits[self.selected_index];
                        if !completed {
                            let habit_id = habit.id;
                            self.update(Message::ToggleHabit(habit_id))
                        } else {
                            iced::Task::none()
                        }
                    } else {
                        let todo_idx = self.selected_index.saturating_sub(self.habits.len());
                        if let Some(todo) = self.todos.get(todo_idx) {
                            if !todo.completed {
                                let todo_id = todo.id;
                                self.update(Message::ToggleTodo(todo_id))
                            } else {
                                iced::Task::none()
                            }
                        } else {
                            iced::Task::none()
                        }
                    }
                }
                Screen::Settings => iced::Task::none(),
            },
            Message::DeleteSelected => match self.screen {
                Screen::Main => {
                    if self.selected_index < self.habits.len() {
                        self.status_message = Some(
                            "Habits cannot be deleted from main view. Use Settings (o) to archive."
                                .into(),
                        );
                        iced::Task::none()
                    } else {
                        let todo_idx = self.selected_index.saturating_sub(self.habits.len());
                        if let Some(todo) = self.todos.get(todo_idx) {
                            self.input_mode = InputMode::ConfirmDelete(todo.id);
                        }
                        iced::Task::none()
                    }
                }
                Screen::Settings => iced::Task::none(),
            },
            Message::ArchiveSelected => {
                if self.screen == Screen::Settings && self.selected_index < self.habits.len() {
                    let habit_id = self.habits[self.selected_index].0.id;
                    self.update(Message::ArchiveHabit(habit_id))
                } else {
                    iced::Task::none()
                }
            }
            Message::ReorderSelected(move_up) => {
                if self.screen == Screen::Settings && self.selected_index < self.habits.len() {
                    let habit_id = self.habits[self.selected_index].0.id;
                    self.update(Message::ReorderHabit(habit_id, move_up))
                } else {
                    iced::Task::none()
                }
            }
            Message::StartEdit => {
                match self.screen {
                    Screen::Main => {
                        if self.selected_index >= self.habits.len() {
                            let todo_idx = self.selected_index.saturating_sub(self.habits.len());
                            if let Some(todo) = self.todos.get(todo_idx) {
                                self.input_mode = InputMode::EditingTodo(todo.id);
                                self.input_buffer = todo.title.clone();
                                self.notes_buffer = todo.notes.clone().unwrap_or_default();
                                return iced::widget::text_input::focus(
                                    iced::widget::text_input::Id::new("edit_todo_input"),
                                );
                            }
                        }
                    }
                    Screen::Settings => {
                        if self.selected_index < self.habits.len() {
                            let (habit, _) = &self.habits[self.selected_index];
                            self.input_mode = InputMode::RenamingHabit(habit.id);
                            self.input_buffer = habit.name.clone();
                            return iced::widget::text_input::focus(
                                iced::widget::text_input::Id::new("rename_habit_input"),
                            );
                        }
                    }
                }
                iced::Task::none()
            }
            Message::StartEditNotes => {
                if self.screen == Screen::Main && self.selected_index >= self.habits.len() {
                    let todo_idx = self.selected_index.saturating_sub(self.habits.len());
                    if let Some(todo) = self.todos.get(todo_idx) {
                        self.input_mode = InputMode::EditingNotes(todo.id);
                        self.notes_buffer = todo.notes.clone().unwrap_or_default();
                        self.input_buffer.clear();
                        return iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "edit_notes_input",
                        ));
                    }
                }
                iced::Task::none()
            }
            Message::StartRename => {
                if self.screen == Screen::Settings && self.selected_index < self.habits.len() {
                    let (habit, _) = &self.habits[self.selected_index];
                    self.input_mode = InputMode::RenamingHabit(habit.id);
                    self.input_buffer = habit.name.clone();
                    return iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                        "rename_habit_input",
                    ));
                }
                iced::Task::none()
            }

            // -------------------------------------------------------------
            // Settings & Export
            // -------------------------------------------------------------
            Message::ExportData => {
                match self.db.export_data() {
                    Ok(data) => match serde_json::to_string_pretty(&data) {
                        Ok(json_str) => {
                            let date_str =
                                chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string();
                            let file_name = format!("habitodo_export_{}.json", date_str);
                            let target_dir = dirs::download_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let target_path = target_dir.join(&file_name);

                            match std::fs::write(&target_path, json_str.as_bytes()) {
                                Ok(()) => {
                                    self.status_message =
                                        Some(format!("Export saved to {}", target_path.display()));
                                }
                                Err(e) => {
                                    let fallback = std::path::PathBuf::from(&file_name);
                                    match std::fs::write(&fallback, json_str.as_bytes()) {
                                        Ok(()) => {
                                            self.status_message = Some(format!(
                                                "Export saved to {}",
                                                fallback.display()
                                            ));
                                        }
                                        Err(e2) => {
                                            self.status_message = Some(format!(
                                                "Failed to export data: {} (fallback: {})",
                                                e, e2
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.status_message =
                                Some(format!("Export serialization error: {}", e));
                        }
                    },
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::ImportData(boxed_data) => {
                match self.db.import_data(&boxed_data) {
                    Ok(()) => {
                        if let Err(e) = self.reload_data() {
                            self.status_message = Some(format!("Database error: {}", e));
                        } else {
                            self.status_message = Some("Data imported successfully".into());
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                }
                iced::Task::none()
            }
            Message::DataExported(result) => {
                match result {
                    Ok(path) => {
                        self.status_message = Some(format!("Export saved to {}", path));
                    }
                    Err(err) => {
                        self.status_message = Some(format!("Export failed: {}", err));
                    }
                }
                iced::Task::none()
            }

            // -------------------------------------------------------------
            // Modals & Input Mode
            // -------------------------------------------------------------
            Message::SetInputMode(mode) => {
                let focus_task = match &mode {
                    InputMode::AddingTodo => {
                        self.input_buffer.clear();
                        self.notes_buffer.clear();
                        iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "todo_input",
                        ))
                    }
                    InputMode::AddingHabit => {
                        self.input_buffer.clear();
                        self.notes_buffer.clear();
                        iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "habit_input",
                        ))
                    }
                    InputMode::EditingTodo(id) => {
                        if let Some(todo) = self.todos.iter().find(|t| t.id == *id) {
                            self.input_buffer = todo.title.clone();
                            self.notes_buffer = todo.notes.clone().unwrap_or_default();
                        }
                        iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "edit_todo_input",
                        ))
                    }
                    InputMode::EditingNotes(id) => {
                        if let Some(todo) = self.todos.iter().find(|t| t.id == *id) {
                            self.notes_buffer = todo.notes.clone().unwrap_or_default();
                            self.input_buffer.clear();
                        }
                        iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "edit_notes_input",
                        ))
                    }
                    InputMode::RenamingHabit(id) => {
                        if let Some((habit, _)) = self.habits.iter().find(|(h, _)| h.id == *id) {
                            self.input_buffer = habit.name.clone();
                        }
                        iced::widget::text_input::focus(iced::widget::text_input::Id::new(
                            "rename_habit_input",
                        ))
                    }
                    InputMode::Normal | InputMode::ConfirmDelete(_) => iced::Task::none(),
                };
                self.input_mode = mode;
                focus_task
            }
            Message::InputChanged(val) => {
                self.input_buffer = val;
                iced::Task::none()
            }
            Message::NotesChanged(val) => {
                self.notes_buffer = val;
                iced::Task::none()
            }
            Message::DueDateChanged(_due) => iced::Task::none(),
            Message::ConfirmDelete(id) => self.update(Message::DeleteTodo(id)),
            Message::CancelModal => {
                if self.input_mode != InputMode::Normal {
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    self.notes_buffer.clear();
                } else if self.show_help {
                    self.show_help = false;
                } else if self.screen == Screen::Settings {
                    self.screen = Screen::Main;
                    self.selected_index = 0;
                    self.clamp_selected_index();
                }
                iced::Task::none()
            }
            Message::SubmitInput => match self.input_mode {
                InputMode::AddingTodo => {
                    let title = self.input_buffer.trim().to_string();
                    let notes = if self.notes_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.notes_buffer.trim().to_string())
                    };
                    if !title.is_empty() {
                        self.update(Message::AddTodo {
                            title,
                            notes,
                            due_date: None,
                        })
                    } else {
                        self.input_mode = InputMode::Normal;
                        iced::Task::none()
                    }
                }
                InputMode::EditingTodo(id) => {
                    let title = self.input_buffer.trim().to_string();
                    if !title.is_empty() {
                        if let Err(e) = self.db.update_todo_title(id, &title) {
                            self.status_message = Some(format!("Database error: {}", e));
                        }
                    }
                    let notes = if self.notes_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.notes_buffer.trim())
                    };
                    if let Err(e) = self.db.update_todo_notes(id, notes) {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    self.notes_buffer.clear();
                    if let Err(e) = self.reload_data() {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                    iced::Task::none()
                }
                InputMode::EditingNotes(id) => {
                    let notes = if self.notes_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.notes_buffer.trim())
                    };
                    if let Err(e) = self.db.update_todo_notes(id, notes) {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                    self.input_mode = InputMode::Normal;
                    self.notes_buffer.clear();
                    if let Err(e) = self.reload_data() {
                        self.status_message = Some(format!("Database error: {}", e));
                    }
                    iced::Task::none()
                }
                InputMode::AddingHabit => {
                    let name = self.input_buffer.trim().to_string();
                    if !name.is_empty() {
                        self.update(Message::AddHabit(name))
                    } else {
                        self.input_mode = InputMode::Normal;
                        iced::Task::none()
                    }
                }
                InputMode::RenamingHabit(id) => {
                    let name = self.input_buffer.trim().to_string();
                    if !name.is_empty() {
                        self.update(Message::RenameHabit(id, name))
                    } else {
                        self.input_mode = InputMode::Normal;
                        iced::Task::none()
                    }
                }
                InputMode::ConfirmDelete(id) => self.update(Message::DeleteTodo(id)),
                InputMode::Normal => iced::Task::none(),
            },

            // -------------------------------------------------------------
            // Error & Status
            // -------------------------------------------------------------
            Message::DbError(err) => {
                self.status_message = Some(format!("Database error: {}", err));
                iced::Task::none()
            }
            Message::ClearStatusMessage => {
                self.status_message = None;
                iced::Task::none()
            }

            // -------------------------------------------------------------
            // Lifecycle
            // -------------------------------------------------------------
            Message::Init => {
                if let Err(e) = self.reload_data() {
                    self.status_message = Some(format!("Database error: {}", e));
                }
                iced::Task::none()
            }
            Message::Tick => iced::Task::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().expect("In-memory DB creation failed");
        AppState::new(db).expect("AppState initialization failed")
    }

    #[test]
    fn test_app_state_initialization_on_empty_db() {
        let state = setup_test_state();
        assert_eq!(state.screen, Screen::Main);
        assert!(state.sidebar_open);
        assert_eq!(state.selected_index, 0);
        assert!(state.habits.is_empty());
        assert!(state.todos.is_empty());
        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(state.input_buffer.is_empty());
        assert!(state.notes_buffer.is_empty());
        assert!(state.status_message.is_none());
        assert!(!state.show_help);
    }

    #[test]
    fn test_navigation_boundary_checks_empty_and_non_empty() {
        let mut state = setup_test_state();

        // 1. Empty DB boundary checks
        let _ = state.update(Message::NavigateDown);
        assert_eq!(state.selected_index, 0);
        let _ = state.update(Message::NavigateUp);
        assert_eq!(state.selected_index, 0);
        let _ = state.update(Message::GoTop);
        assert_eq!(state.selected_index, 0);
        let _ = state.update(Message::GoBottom);
        assert_eq!(state.selected_index, 0);

        // 2. Add 2 habits and 2 todos (total items = 4, indices 0..=3)
        let _ = state.update(Message::AddHabit("Habit 1".into()));
        let _ = state.update(Message::AddHabit("Habit 2".into()));
        let _ = state.update(Message::AddTodo {
            title: "Todo 1".into(),
            notes: None,
            due_date: None,
        });
        let _ = state.update(Message::AddTodo {
            title: "Todo 2".into(),
            notes: None,
            due_date: None,
        });

        assert_eq!(state.habits.len(), 2);
        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.total_items_for_screen(), 4);

        // Navigate down step by step
        state.selected_index = 0;
        let _ = state.update(Message::NavigateDown);
        assert_eq!(state.selected_index, 1);
        let _ = state.update(Message::NavigateDown);
        assert_eq!(state.selected_index, 2);
        let _ = state.update(Message::NavigateDown);
        assert_eq!(state.selected_index, 3);
        // Clamped at bottom index
        let _ = state.update(Message::NavigateDown);
        assert_eq!(state.selected_index, 3);

        // Navigate up step by step
        let _ = state.update(Message::NavigateUp);
        assert_eq!(state.selected_index, 2);
        let _ = state.update(Message::NavigateUp);
        assert_eq!(state.selected_index, 1);
        let _ = state.update(Message::NavigateUp);
        assert_eq!(state.selected_index, 0);
        // Clamped at top index
        let _ = state.update(Message::NavigateUp);
        assert_eq!(state.selected_index, 0);

        // Jump to bottom and top
        let _ = state.update(Message::GoBottom);
        assert_eq!(state.selected_index, 3);
        let _ = state.update(Message::GoTop);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_date_navigation_transitions() {
        let mut state = setup_test_state();
        let initial_date = state.selected_date;

        let _ = state.update(Message::NextDay);
        assert_eq!(state.selected_date, initial_date.succ_opt().unwrap());

        let _ = state.update(Message::PreviousDay);
        assert_eq!(state.selected_date, initial_date);

        let _ = state.update(Message::PreviousDay);
        assert_eq!(state.selected_date, initial_date.pred_opt().unwrap());

        let _ = state.update(Message::GoToday);
        assert_eq!(state.selected_date, chrono::Local::now().date_naive());
    }

    #[test]
    fn test_habit_actions_write_through() {
        let mut state = setup_test_state();

        // 1. Add Habit
        let _ = state.update(Message::AddHabit("Morning Run".into()));
        assert_eq!(state.habits.len(), 1);
        let habit_id = state.habits[0].0.id;
        assert_eq!(state.habits[0].0.name, "Morning Run");
        assert!(!state.habits[0].1); // Initially uncompleted

        // Verify in SQLite directly
        let db_habits = state.db.get_habits_for_date(state.selected_date).unwrap();
        assert_eq!(db_habits.len(), 1);
        assert_eq!(db_habits[0].0.name, "Morning Run");

        // 2. Toggle Habit
        let _ = state.update(Message::ToggleHabit(habit_id));
        assert!(state.habits[0].1); // Now completed
        let db_habits_after_toggle = state.db.get_habits_for_date(state.selected_date).unwrap();
        assert!(db_habits_after_toggle[0].1);

        // Toggle back to false
        let _ = state.update(Message::ToggleHabit(habit_id));
        assert!(!state.habits[0].1);

        // 3. Rename Habit
        let _ = state.update(Message::RenameHabit(habit_id, "Evening Run".into()));
        assert_eq!(state.habits[0].0.name, "Evening Run");
        let db_habits_renamed = state.db.get_habits_for_date(state.selected_date).unwrap();
        assert_eq!(db_habits_renamed[0].0.name, "Evening Run");

        // 4. Archive Habit
        let _ = state.update(Message::ArchiveHabit(habit_id));
        assert!(state.habits.is_empty());
        let db_habits_archived = state.db.get_habits_for_date(state.selected_date).unwrap();
        assert!(db_habits_archived.is_empty());
    }

    #[test]
    fn test_todo_actions_write_through() {
        let mut state = setup_test_state();

        // 1. Add Todo
        let due = NaiveDate::from_ymd_opt(2026, 10, 15);
        let _ = state.update(Message::AddTodo {
            title: "Write documentation".into(),
            notes: Some("Detailed guide".into()),
            due_date: due,
        });

        assert_eq!(state.todos.len(), 1);
        let todo_id = state.todos[0].id;
        assert_eq!(state.todos[0].title, "Write documentation");
        assert_eq!(state.todos[0].notes.as_deref(), Some("Detailed guide"));
        assert_eq!(state.todos[0].due_date, due);
        assert!(!state.todos[0].completed);

        // Verify in SQLite directly
        let db_todos = state.db.get_all_todos().unwrap();
        assert_eq!(db_todos.len(), 1);
        assert_eq!(db_todos[0].title, "Write documentation");

        // 2. Toggle Todo
        let _ = state.update(Message::ToggleTodo(todo_id));
        assert!(state.todos[0].completed);
        assert!(state.todos[0].completed_at.is_some());

        let db_todos_toggled = state.db.get_all_todos().unwrap();
        assert!(db_todos_toggled[0].completed);

        // 3. Update Todo Notes
        let _ = state.update(Message::UpdateTodoNotes(todo_id, "Updated notes".into()));
        assert_eq!(state.todos[0].notes.as_deref(), Some("Updated notes"));

        // 4. Delete Todo
        let _ = state.update(Message::DeleteTodo(todo_id));
        assert!(state.todos.is_empty());
        let db_todos_deleted = state.db.get_all_todos().unwrap();
        assert!(db_todos_deleted.is_empty());
    }

    #[test]
    fn test_contextual_selection_actions() {
        let mut state = setup_test_state();
        let _ = state.update(Message::AddHabit("Daily Meditation".into()));
        let _ = state.update(Message::AddTodo {
            title: "Call dentist".into(),
            notes: None,
            due_date: None,
        });

        assert_eq!(state.habits.len(), 1);
        assert_eq!(state.todos.len(), 1);

        // 1. Cursor on habit (index 0): ToggleSelected toggles habit
        state.selected_index = 0;
        let _ = state.update(Message::ToggleSelected);
        assert!(state.habits[0].1); // Habit completed

        // 2. Cursor on todo (index 1): ToggleSelected toggles todo
        state.selected_index = 1;
        let _ = state.update(Message::ToggleSelected);
        assert!(state.todos[0].completed);

        // 3. Cursor on habit (index 0): DeleteSelected emits notice that habits can't be deleted in Main
        state.selected_index = 0;
        let _ = state.update(Message::DeleteSelected);
        assert!(state
            .status_message
            .as_ref()
            .unwrap()
            .contains("Habits cannot be deleted from main view"));
        assert_eq!(state.input_mode, InputMode::Normal);

        // 4. Cursor on todo (index 1): DeleteSelected enters ConfirmDelete mode
        state.selected_index = 1;
        let todo_id = state.todos[0].id;
        let _ = state.update(Message::DeleteSelected);
        assert_eq!(state.input_mode, InputMode::ConfirmDelete(todo_id));

        // Confirm deletion
        let _ = state.update(Message::ConfirmDelete(todo_id));
        assert!(state.todos.is_empty());
        assert_eq!(state.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_settings_reorder_write_through() {
        let mut state = setup_test_state();
        let _ = state.update(Message::SwitchScreen(Screen::Settings));
        assert_eq!(state.screen, Screen::Settings);

        let _ = state.update(Message::AddHabit("Habit Alpha".into()));
        let _ = state.update(Message::AddHabit("Habit Beta".into()));
        let _ = state.update(Message::AddHabit("Habit Gamma".into()));

        assert_eq!(state.habits.len(), 3);
        assert_eq!(state.habits[0].0.name, "Habit Alpha");
        assert_eq!(state.habits[1].0.name, "Habit Beta");
        assert_eq!(state.habits[2].0.name, "Habit Gamma");

        // Reorder Beta up (index 1 -> index 0)
        let beta_id = state.habits[1].0.id;
        let _ = state.update(Message::ReorderHabit(beta_id, true));

        assert_eq!(state.habits[0].0.name, "Habit Beta");
        assert_eq!(state.habits[1].0.name, "Habit Alpha");
        assert_eq!(state.habits[2].0.name, "Habit Gamma");

        // Verify in SQLite directly
        let db_habits = state.db.get_habits_for_date(state.selected_date).unwrap();
        assert_eq!(db_habits[0].0.name, "Habit Beta");
        assert_eq!(db_habits[1].0.name, "Habit Alpha");

        // ReorderSelected down on index 0
        state.selected_index = 0;
        let _ = state.update(Message::ReorderSelected(false));
        assert_eq!(state.habits[0].0.name, "Habit Alpha");
        assert_eq!(state.habits[1].0.name, "Habit Beta");
    }

    #[test]
    fn test_non_blocking_error_handling() {
        let mut state = setup_test_state();

        // 1. Explicit DbError sets status_message without panicking
        let _ = state.update(Message::DbError("Connection reset".into()));
        assert_eq!(
            state.status_message.as_deref(),
            Some("Database error: Connection reset")
        );

        let _ = state.update(Message::ClearStatusMessage);
        assert!(state.status_message.is_none());

        // 2. Unique constraint violation on duplicate habit name
        let _ = state.update(Message::AddHabit("Reading".into()));
        assert!(state.status_message.is_none());

        // Add duplicate name
        let _ = state.update(Message::AddHabit("Reading".into()));
        assert!(state.status_message.is_some());
        assert!(state
            .status_message
            .as_ref()
            .unwrap()
            .contains("Database error"));
    }

    #[test]
    fn test_modal_and_input_mode_transitions() {
        let mut state = setup_test_state();

        // 1. Add habit mode
        let _ = state.update(Message::SetInputMode(InputMode::AddingHabit));
        assert_eq!(state.input_mode, InputMode::AddingHabit);
        let _ = state.update(Message::InputChanged("Coding".into()));
        assert_eq!(state.input_buffer, "Coding");
        let _ = state.update(Message::SubmitInput);
        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.habits.len(), 1);
        assert_eq!(state.habits[0].0.name, "Coding");

        // 2. Add todo mode with notes
        let _ = state.update(Message::SetInputMode(InputMode::AddingTodo));
        assert_eq!(state.input_mode, InputMode::AddingTodo);
        let _ = state.update(Message::InputChanged("Review PR".into()));
        let _ = state.update(Message::NotesChanged("Check tests".into()));
        let _ = state.update(Message::SubmitInput);
        assert_eq!(state.input_mode, InputMode::Normal);
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Review PR");
        assert_eq!(state.todos[0].notes.as_deref(), Some("Check tests"));

        // 3. Hierarchical CancelModal
        // Level 1: Input mode active
        let _ = state.update(Message::SetInputMode(InputMode::AddingTodo));
        let _ = state.update(Message::CancelModal);
        assert_eq!(state.input_mode, InputMode::Normal);

        // Level 2: Help modal active
        let _ = state.update(Message::ToggleHelp);
        assert!(state.show_help);
        let _ = state.update(Message::CancelModal);
        assert!(!state.show_help);

        // Level 3: Settings screen active
        let _ = state.update(Message::SwitchScreen(Screen::Settings));
        assert_eq!(state.screen, Screen::Settings);
        let _ = state.update(Message::CancelModal);
        assert_eq!(state.screen, Screen::Main);
    }

    #[test]
    fn test_export_and_import_transitions() {
        let mut state = setup_test_state();
        let _ = state.update(Message::AddHabit("Stretch".into()));
        let _ = state.update(Message::AddTodo {
            title: "Water plants".into(),
            notes: None,
            due_date: None,
        });

        // Trigger export
        let _ = state.update(Message::ExportData);
        assert!(state.status_message.is_some());
        assert!(state
            .status_message
            .as_ref()
            .unwrap()
            .contains("Export saved to"));

        // Test ImportData with in-memory payload
        let export_data = state.db.export_data().unwrap();
        let mut new_state = setup_test_state();
        assert!(new_state.habits.is_empty());
        assert!(new_state.todos.is_empty());

        let _ = new_state.update(Message::ImportData(Box::new(export_data)));
        assert_eq!(new_state.habits.len(), 1);
        assert_eq!(new_state.habits[0].0.name, "Stretch");
        assert_eq!(new_state.todos.len(), 1);
        assert_eq!(new_state.todos[0].title, "Water plants");
    }

    #[test]
    fn test_selected_item_helpers_and_screen_switching() {
        let mut state = setup_test_state();
        let _ = state.update(Message::AddHabit("Meditate".into()));
        let _ = state.update(Message::AddTodo {
            title: "Clean desk".into(),
            notes: Some("Organize papers".into()),
            due_date: None,
        });

        // Index 0: points to habit
        state.selected_index = 0;
        assert_eq!(state.selected_habit().unwrap().0.name, "Meditate");
        assert!(state.selected_todo().is_none());

        // Index 1: points to todo
        state.selected_index = 1;
        assert!(state.selected_habit().is_none());
        assert_eq!(state.selected_todo().unwrap().title, "Clean desk");

        // Switch to Settings: todo helper must return None
        let _ = state.update(Message::SwitchScreen(Screen::Settings));
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.selected_habit().unwrap().0.name, "Meditate");
        assert!(state.selected_todo().is_none());
    }

    #[test]
    fn test_whitespace_and_empty_inputs_safety() {
        let mut state = setup_test_state();

        // Empty / whitespace habit name ignored
        let _ = state.update(Message::AddHabit("   ".into()));
        assert!(state.habits.is_empty());
        assert_eq!(state.input_mode, InputMode::Normal);

        // Empty / whitespace todo title ignored
        let _ = state.update(Message::AddTodo {
            title: "   ".into(),
            notes: None,
            due_date: None,
        });
        assert!(state.todos.is_empty());
        assert_eq!(state.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_notes_editing_flow() {
        let mut state = setup_test_state();
        let _ = state.update(Message::AddTodo {
            title: "Write report".into(),
            notes: None,
            due_date: None,
        });

        // Enter notes editing
        state.selected_index = 0;
        let _ = state.update(Message::StartEditNotes);
        let todo_id = state.todos[0].id;
        assert_eq!(state.input_mode, InputMode::EditingNotes(todo_id));

        // Type notes and submit
        let _ = state.update(Message::NotesChanged("Preliminary draft".into()));
        let _ = state.update(Message::SubmitInput);

        assert_eq!(state.input_mode, InputMode::Normal);
        assert_eq!(state.todos[0].notes.as_deref(), Some("Preliminary draft"));
    }

    #[test]
    fn test_extreme_date_navigation_invariants() {
        let mut state = setup_test_state();

        // Test near date bounds
        state.selected_date = NaiveDate::MIN;
        let _ = state.update(Message::PreviousDay);
        assert_eq!(state.selected_date, NaiveDate::MIN); // Clamped, no panic

        state.selected_date = NaiveDate::MAX;
        let _ = state.update(Message::NextDay);
        assert_eq!(state.selected_date, NaiveDate::MAX); // Clamped, no panic
    }
}
