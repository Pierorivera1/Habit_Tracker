//! Presentation layer for Habitodo.
//!
//! Provides the dark mode design system, reusable widgets, navigation headers,
//! metrics sidebar with canvas charts, settings management view, and pure view function.

pub mod components;
pub mod main_view;
pub mod settings_view;
pub mod sidebar;
pub mod theme;
pub mod top_bar;

use crate::app::message::Message;
use crate::app::state::{AppState, InputMode, Screen};
use iced::widget::{column, container, row, stack};
use iced::{Element, Length};

/// Pure top-level view function for Habitodo following Elm Architecture.
///
/// Assembles:
/// - Top navigation bar with active date and quick actions
/// - Main area (MainView + collapsible Metrics Sidebar) or SettingsView
/// - Bottom contextual shortcut guide bar
/// - Modal overlays (Confirm Delete, Keyboard Shortcuts Help)
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    // 1. Top Navigation Bar
    let top_bar_elem = top_bar::view(state);

    // 2. Main Content Area
    let main_area_elem: Element<'a, Message> = match state.screen {
        Screen::Main => {
            let mut main_row = row![main_view::view(state)]
                .width(Length::Fill)
                .height(Length::Fill);

            if state.sidebar_open {
                main_row = main_row.push(sidebar::view(state));
            }

            main_row.into()
        }
        Screen::Settings => settings_view::view(state),
    };

    // 3. Bottom Shortcut Bar
    let bottom_bar_elem = components::bottom_shortcut_bar(state);

    // Assemble root layout column
    let base_layout = column![
        top_bar_elem,
        container(main_area_elem)
            .width(Length::Fill)
            .height(Length::Fill),
        bottom_bar_elem,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let base_container = container(base_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::root_container);

    // 4. Modal Overlays
    if state.show_help {
        stack![base_container, components::help_modal()].into()
    } else if let InputMode::ConfirmDelete(todo_id) = state.input_mode {
        let todo_title = state
            .todos
            .iter()
            .find(|t| t.id == todo_id)
            .map(|t| t.title.as_str());

        stack![
            base_container,
            components::confirm_delete_modal(todo_id, todo_title)
        ]
        .into()
    } else {
        base_container.into()
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::{DailyStats, Habit, Todo};
    use chrono::NaiveDate;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        AppState::new(db).unwrap()
    }

    #[test]
    fn test_top_level_view_main_screen_sidebar_open_and_closed() {
        let mut state = setup_test_state();

        // Main screen, sidebar open
        {
            state.screen = Screen::Main;
            state.sidebar_open = true;
            let _elem = view(&state);
        }

        // Main screen, sidebar closed
        {
            state.sidebar_open = false;
            let _elem = view(&state);
        }
    }

    #[test]
    fn test_top_level_view_settings_screen() {
        let mut state = setup_test_state();
        state.screen = Screen::Settings;
        let _elem = view(&state);
    }

    #[test]
    fn test_top_level_view_overlays_help_and_delete() {
        let mut state = setup_test_state();

        // Help modal overlay
        {
            state.show_help = true;
            let _elem = view(&state);
        }

        // Confirm delete modal overlay
        {
            state.show_help = false;
            state.input_mode = InputMode::ConfirmDelete(1);
            let _elem = view(&state);
        }
    }

    #[test]
    fn test_top_level_view_scaled_stress() {
        let mut state = setup_test_state();

        // Populate with 50 habits and 50 todos
        state.habits = (1..=50)
            .map(|i| {
                (
                    Habit {
                        id: i,
                        name: format!("Habit number {}", i),
                        position: i,
                        created_at: "2026-09-01".to_string(),
                        archived: false,
                    },
                    i % 2 == 0,
                )
            })
            .collect();

        state.todos = (1..=50)
            .map(|i| Todo {
                id: i,
                title: format!("Todo task {}", i),
                notes: if i % 3 == 0 {
                    Some("Notes attached".to_string())
                } else {
                    None
                },
                completed: i % 4 == 0,
                created_at: "2026-09-01".to_string(),
                completed_at: None,
                due_date: Some(NaiveDate::from_ymd_opt(2026, 9, 25).unwrap()),
                position: i,
            })
            .collect();

        state.daily_stats_30_days = (1..=30)
            .map(|i| DailyStats {
                date: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                total_habits: 50,
                completed_habits: i,
            })
            .collect();

        state.habit_success_rates = (1..=50).map(|i| (i, (i as f32 * 2.0).min(100.0))).collect();

        // View in Main screen
        {
            state.screen = Screen::Main;
            state.selected_index = 75; // selected inside todos
            let _elem = view(&state);
        }

        // View in Settings screen
        {
            state.screen = Screen::Settings;
            state.selected_index = 25; // selected inside habits
            let _elem = view(&state);
        }
    }
}
