//! Strongly-typed Message enum for Habitodo's Elm Architecture.

use crate::app::state::{InputMode, Screen};
use crate::domain::ExportData;
use chrono::NaiveDate;

/// Exhaustive message enum representing all possible user interactions,
/// navigation commands, data modifications, and lifecycle events.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // -------------------------------------------------------------
    // Navigation
    // -------------------------------------------------------------
    /// Navigate cursor up by one item.
    NavigateUp,
    /// Navigate cursor down by one item.
    NavigateDown,
    /// Move selected calendar date back by one day.
    PreviousDay,
    /// Move selected calendar date forward by one day.
    NextDay,
    /// Reset selected calendar date to today.
    GoToday,
    /// Jump selection cursor to top of the active list.
    GoTop,
    /// Jump selection cursor to bottom of the active list.
    GoBottom,
    /// Toggle visibility of the right metrics sidebar.
    ToggleSidebar,
    /// Switch active screen (e.g. Main or Settings).
    SwitchScreen(Screen),
    /// Convenience alias to switch to Settings view.
    OpenSettings,
    /// Convenience alias to switch to Main view.
    CloseSettings,
    /// Toggle visibility of keyboard shortcuts help overlay.
    ToggleHelp,
    /// Terminate application process cleanly.
    Quit,

    // -------------------------------------------------------------
    // Habit Actions
    // -------------------------------------------------------------
    /// Toggle daily completion of habit with the given ID on the selected date.
    ToggleHabit(i64),
    /// Add a new habit with the given name.
    AddHabit(String),
    /// Soft-delete / archive habit with the given ID.
    ArchiveHabit(i64),
    /// Rename habit with the given ID.
    RenameHabit(i64, String),
    /// Reorder habit up (true) or down (false) in priority.
    ReorderHabit(i64, bool),

    // -------------------------------------------------------------
    // To-Do Actions
    // -------------------------------------------------------------
    /// Toggle completion state of to-do item with the given ID.
    ToggleTodo(i64),
    /// Create a new to-do task with title and optional notes / due date.
    AddTodo {
        title: String,
        notes: Option<String>,
        due_date: Option<NaiveDate>,
    },
    /// Delete to-do item with the given ID.
    DeleteTodo(i64),
    /// Update attached notes for to-do item with the given ID.
    UpdateTodoNotes(i64, String),

    // -------------------------------------------------------------
    // Contextual Selection Actions (cursor-driven)
    // -------------------------------------------------------------
    /// Toggle completion of item currently highlighted by selection cursor.
    ToggleSelected,
    /// Complete or dismiss item currently highlighted by selection cursor.
    CompleteSelected,
    /// Prompt deletion or delete item currently highlighted by selection cursor.
    DeleteSelected,
    /// Archive habit currently highlighted by selection cursor in Settings.
    ArchiveSelected,
    /// Reorder habit currently highlighted by selection cursor up (true) or down (false).
    ReorderSelected(bool),
    /// Enter inline edit mode for highlighted item.
    StartEdit,
    /// Enter notes editing mode for highlighted to-do.
    StartEditNotes,
    /// Enter inline renaming mode for highlighted habit in Settings.
    StartRename,

    // -------------------------------------------------------------
    // Settings & Export
    // -------------------------------------------------------------
    /// Trigger JSON backup export to disk.
    ExportData,
    /// Import complete database backup from ExportData container.
    ImportData(Box<ExportData>),
    /// Notification when data export finishes with destination path or error.
    DataExported(Result<String, String>),

    // -------------------------------------------------------------
    // Modals & Input Mode
    // -------------------------------------------------------------
    /// Set current input mode (e.g. Normal, AddingTodo, EditingTodo).
    SetInputMode(InputMode),
    /// Notification that the main text input buffer changed.
    InputChanged(String),
    /// Notification that the notes input buffer changed.
    NotesChanged(String),
    /// Notification that the due date buffer changed.
    DueDateChanged(Option<NaiveDate>),
    /// Confirm deletion of to-do item with the given ID.
    ConfirmDelete(i64),
    /// Dismiss active modal, clear input buffers, and return to Normal mode.
    CancelModal,
    /// Commit active input buffer (e.g. when pressing Enter).
    SubmitInput,

    // -------------------------------------------------------------
    // Error & Status
    // -------------------------------------------------------------
    /// Non-blocking SQLite error surfaced for toast/status bar display.
    DbError(String),
    /// Clear currently visible status message.
    ClearStatusMessage,

    // -------------------------------------------------------------
    // Lifecycle
    // -------------------------------------------------------------
    /// Initial startup event to load initial data.
    Init,
    /// Periodic tick event for timing or status message dismissal.
    Tick,
}
