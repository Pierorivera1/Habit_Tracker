//! Settings view for managing habits, reordering, archiving, and JSON exports.

use crate::app::message::Message;
use crate::app::state::{AppState, InputMode};
use crate::domain::Habit;
use crate::ui::theme;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the Settings view.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut content = column![].spacing(24).padding(24).width(Length::Fill);

    // -------------------------------------------------------------------------
    // Top Header: Title & Back Button
    // -------------------------------------------------------------------------
    let back_button = button(text("◀ [Esc] Back to Dashboard").size(13))
        .on_press(Message::CloseSettings)
        .style(theme::secondary_button)
        .padding([6, 12]);

    let title_bar = row![
        text("Settings & Configuration")
            .size(20)
            .color(theme::TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        back_button,
    ]
    .align_y(Alignment::Center);

    content = content.push(title_bar);

    // -------------------------------------------------------------------------
    // SECTION 1: Habit Management
    // -------------------------------------------------------------------------
    let add_button = button(text("[i] + Add Habit").size(13))
        .on_press(Message::SetInputMode(InputMode::AddingHabit))
        .style(theme::primary_button)
        .padding([4, 10]);

    let habit_section_header = row![
        column![
            text("Habit Management").size(17).color(theme::TEXT_PRIMARY),
            text("Reorder [J/K], Rename [r], or Archive [d] daily habits")
                .size(12)
                .color(theme::TEXT_SECONDARY),
        ],
        Space::with_width(Length::Fill),
        add_button,
    ]
    .align_y(Alignment::Center);

    content = content.push(habit_section_header);

    // Inline Add Habit text input
    if state.input_mode == InputMode::AddingHabit {
        let input_field = text_input(
            "New habit name... (Enter to save, Esc to cancel)",
            &state.input_buffer,
        )
        .id(text_input::Id::new("habit_input"))
        .on_input(Message::InputChanged)
        .on_submit(Message::SubmitInput)
        .style(theme::custom_text_input)
        .padding(10)
        .size(14);

        let add_box = container(
            row![
                text(">").size(15).color(theme::ACCENT_CYAN),
                Space::with_width(8),
                input_field,
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::selected_card_container)
        .padding([8, 12])
        .width(Length::Fill);

        content = content.push(add_box);
    }

    if state.habits.is_empty() && state.input_mode != InputMode::AddingHabit {
        let empty_box = container(
            text("No active habits defined. Press 'i' to add your first habit.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        )
        .style(theme::card_container)
        .padding(16)
        .width(Length::Fill);

        content = content.push(empty_box);
    } else {
        let mut habit_list = column![].spacing(8).width(Length::Fill);

        let total = state.habits.len();
        for (idx, (habit, _)) in state.habits.iter().enumerate() {
            let is_selected = state.selected_index == idx;
            let row_elem = render_settings_habit_row(habit, idx, total, is_selected, state);
            habit_list = habit_list.push(row_elem);
        }

        content = content.push(habit_list);
    }

    // -------------------------------------------------------------------------
    // SECTION 2: Data Portability & Backup
    // -------------------------------------------------------------------------
    let export_button = button(text("[e] Export Data (JSON)").size(13))
        .on_press(Message::ExportData)
        .style(theme::secondary_button)
        .padding([6, 14]);

    let export_card = container(
        column![
            text("Data Portability & Backup")
                .size(16)
                .color(theme::TEXT_PRIMARY),
            Space::with_height(4),
            text(
                "Export all habits, completion history, and to-do items into standard JSON format."
            )
            .size(13)
            .color(theme::TEXT_SECONDARY),
            Space::with_height(10),
            export_button,
        ]
        .width(Length::Fill),
    )
    .style(theme::card_container)
    .padding(16)
    .width(Length::Fill);

    content = content.push(export_card);

    // -------------------------------------------------------------------------
    // SECTION 3: System Diagnostics & Storage
    // -------------------------------------------------------------------------
    let diagnostics_card = container(
        column![
            text("System & Storage").size(15).color(theme::TEXT_PRIMARY),
            Space::with_height(6),
            text("Application: Habit Tracker v0.1.0")
                .size(12)
                .color(theme::TEXT_MUTED),
            text("Engine: SQLite with WAL (Write-Ahead Logging)")
                .size(12)
                .color(theme::TEXT_MUTED),
            text("Default Path: ~/.local/share/habitodo/habitodo.db")
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .width(Length::Fill),
    )
    .style(theme::card_container)
    .padding(16)
    .width(Length::Fill);

    content = content.push(diagnostics_card);

    scrollable(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

/// Renders a single habit row in Settings view with action buttons.
fn render_settings_habit_row<'a>(
    habit: &'a Habit,
    index: usize,
    total: usize,
    is_selected: bool,
    state: &'a AppState,
) -> Element<'a, Message> {
    let position_badge = container(
        text(format!("#{}", index + 1))
            .size(12)
            .color(theme::TEXT_MUTED),
    )
    .style(theme::pill_container)
    .padding([2, 6]);

    let is_renaming = state.input_mode == InputMode::RenamingHabit(habit.id);

    let main_content: Element<'a, Message> = if is_renaming {
        let rename_input = text_input("Habit name...", &state.input_buffer)
            .id(text_input::Id::new("rename_habit_input"))
            .on_input(Message::InputChanged)
            .on_submit(Message::SubmitInput)
            .style(theme::custom_text_input)
            .padding(6)
            .size(14);

        row![position_badge, Space::with_width(8), rename_input]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    } else {
        let habit_name = text(&habit.name).size(15).color(theme::TEXT_PRIMARY);

        row![position_badge, Space::with_width(10), habit_name]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    };

    // Actions: Up (K), Down (J), Rename (r), Archive (d)
    let is_first = index == 0;
    let is_last = index + 1 >= total;

    let up_btn = if !is_first {
        button(text("▲ [K]").size(11))
            .on_press(Message::ReorderHabit(habit.id, true))
            .style(theme::secondary_button)
            .padding([3, 7])
    } else {
        button(text("▲").size(11))
            .style(theme::secondary_button)
            .padding([3, 7])
    };

    let down_btn = if !is_last {
        button(text("▼ [J]").size(11))
            .on_press(Message::ReorderHabit(habit.id, false))
            .style(theme::secondary_button)
            .padding([3, 7])
    } else {
        button(text("▼").size(11))
            .style(theme::secondary_button)
            .padding([3, 7])
    };

    let rename_btn = button(text("Rename [r]").size(11))
        .on_press(Message::SetInputMode(InputMode::RenamingHabit(habit.id)))
        .style(theme::secondary_button)
        .padding([3, 7]);

    let archive_btn = button(text("Archive [d]").size(11))
        .on_press(Message::ArchiveHabit(habit.id))
        .style(theme::secondary_button)
        .padding([3, 7]);

    let actions = row![up_btn, down_btn, rename_btn, archive_btn]
        .spacing(6)
        .align_y(Alignment::Center);

    let row_container = row![main_content, Space::with_width(Length::Fill), actions,]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let card_style = if is_selected {
        theme::selected_card_container
    } else {
        theme::card_container
    };

    container(row_container)
        .style(card_style)
        .padding([10, 14])
        .width(Length::Fill)
        .into()
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Screen;
    use crate::db::Database;
    use crate::domain::Habit;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        let mut state = AppState::new(db).unwrap();
        state.screen = Screen::Settings;
        state
    }

    #[test]
    fn test_settings_view_empty_habits() {
        let state = setup_test_state();
        let _elem = view(&state);
    }

    #[test]
    fn test_settings_view_with_habits_and_selection() {
        let mut state = setup_test_state();
        state.habits = vec![
            (
                Habit {
                    id: 1,
                    name: "Habit 1".to_string(),
                    position: 1,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            ),
            (
                Habit {
                    id: 2,
                    name: "Habit 2".to_string(),
                    position: 2,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            ),
            (
                Habit {
                    id: 3,
                    name: "Habit 3".to_string(),
                    position: 3,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            ),
        ];

        // First habit selected (is_first = true, is_last = false)
        {
            state.selected_index = 0;
            let _elem = view(&state);
        }

        // Middle habit selected (is_first = false, is_last = false)
        {
            state.selected_index = 1;
            let _elem = view(&state);
        }

        // Last habit selected (is_first = false, is_last = true)
        {
            state.selected_index = 2;
            let _elem = view(&state);
        }
    }

    #[test]
    fn test_settings_view_input_modes() {
        let mut state = setup_test_state();

        // Adding habit
        {
            state.input_mode = InputMode::AddingHabit;
            state.input_buffer = "Read Books".to_string();
            let _elem = view(&state);
        }

        // Renaming habit
        {
            state.habits = vec![(
                Habit {
                    id: 42,
                    name: "Old Name".to_string(),
                    position: 1,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            )];
            state.input_mode = InputMode::RenamingHabit(42);
            state.input_buffer = "New Name".to_string();
            let _elem = view(&state);
        }
    }
}
