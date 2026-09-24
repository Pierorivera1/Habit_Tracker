//! Contextual bottom shortcut footer and modal dialog components.

use crate::app::message::Message;
use crate::app::state::{AppState, InputMode, Screen};
use crate::ui::theme;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

/// Renders the persistent bottom shortcut bar reflecting active contextual keybindings.
pub fn bottom_shortcut_bar<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut shortcut_items: Vec<(&'static str, &'static str)> = Vec::new();

    if state.show_help {
        shortcut_items.push(("Esc / ?", "Close Help"));
    } else {
        match &state.input_mode {
            InputMode::ConfirmDelete(_) => {
                shortcut_items.push(("y / Enter", "Confirm Delete"));
                shortcut_items.push(("n / Esc", "Cancel"));
            }
            InputMode::AddingTodo => {
                shortcut_items.push(("Enter", "Save Task"));
                shortcut_items.push(("Esc", "Cancel"));
            }
            InputMode::EditingTodo(_) => {
                shortcut_items.push(("Enter", "Save Title"));
                shortcut_items.push(("Esc", "Cancel"));
            }
            InputMode::EditingNotes(_) => {
                shortcut_items.push(("Enter", "Save Notes"));
                shortcut_items.push(("Esc", "Cancel"));
            }
            InputMode::AddingHabit => {
                shortcut_items.push(("Enter", "Save Habit"));
                shortcut_items.push(("Esc", "Cancel"));
            }
            InputMode::RenamingHabit(_) => {
                shortcut_items.push(("Enter", "Save Name"));
                shortcut_items.push(("Esc", "Cancel"));
            }
            InputMode::Normal => match state.screen {
                Screen::Main => {
                    shortcut_items.push(("j/k", "Navigate"));
                    shortcut_items.push(("Space", "Toggle"));
                    shortcut_items.push(("h/l", "+/-Day"));
                    shortcut_items.push(("t", "Today"));
                    shortcut_items.push(("i", "Add To-Do"));
                    shortcut_items.push(("Enter", "Edit"));
                    shortcut_items.push(("n", "Notes"));
                    shortcut_items.push(("d", "Delete"));
                    shortcut_items.push(("s/Tab", "Sidebar"));
                    shortcut_items.push(("o", "Settings"));
                    shortcut_items.push(("?", "Help"));
                    shortcut_items.push(("q", "Quit"));
                }
                Screen::Settings => {
                    shortcut_items.push(("j/k", "Navigate"));
                    shortcut_items.push(("i", "Add Habit"));
                    shortcut_items.push(("r", "Rename"));
                    shortcut_items.push(("J/K", "Reorder"));
                    shortcut_items.push(("d", "Archive"));
                    shortcut_items.push(("e", "Export JSON"));
                    shortcut_items.push(("Esc", "Back"));
                }
            },
        }
    }

    let mut row_elem = row![].spacing(14).align_y(Alignment::Center);

    for (key, label) in shortcut_items {
        row_elem = row_elem.push(render_shortcut_pill(key, label));
    }

    let scrollable_bar = scrollable(row_elem).direction(scrollable::Direction::Horizontal(
        scrollable::Scrollbar::default().scroller_width(4.0),
    ));

    container(scrollable_bar)
        .width(Length::Fill)
        .padding([6, 16])
        .style(theme::bottom_bar_container)
        .into()
}

/// Helper function to render a `[Key] Label` badge pair.
pub fn render_shortcut_pill<'a>(key: &'static str, label: &'static str) -> Element<'a, Message> {
    let key_box = container(text(key).size(11).color(theme::TEXT_PRIMARY))
        .style(theme::pill_container)
        .padding([2, 5]);

    let label_text = text(label).size(12).color(theme::TEXT_SECONDARY);

    row![key_box, Space::with_width(5), label_text]
        .align_y(Alignment::Center)
        .into()
}

/// Renders the confirmation modal when deleting a to-do item.
pub fn confirm_delete_modal<'a>(todo_id: i64, todo_title: Option<&'a str>) -> Element<'a, Message> {
    let title_text = text("Confirm Deletion")
        .size(18)
        .color(theme::ACCENT_DANGER);

    let message_text = if let Some(title) = todo_title {
        format!("Are you sure you want to permanently delete \"{}\"?", title)
    } else {
        "Are you sure you want to permanently delete this task?".to_string()
    };

    let confirm_btn = button(text("Delete [y / Enter]").size(13))
        .on_press(Message::ConfirmDelete(todo_id))
        .style(theme::danger_button)
        .padding([6, 14]);

    let cancel_btn = button(text("Cancel [n / Esc]").size(13))
        .on_press(Message::CancelModal)
        .style(theme::secondary_button)
        .padding([6, 14]);

    let action_row =
        row![cancel_btn, Space::with_width(12), confirm_btn].align_y(Alignment::Center);

    let card = container(
        column![
            title_text,
            Space::with_height(10),
            text(message_text).size(14).color(theme::TEXT_PRIMARY),
            Space::with_height(16),
            action_row,
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fixed(400.0)),
    )
    .style(theme::modal_card_container)
    .padding(20);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::modal_backdrop)
        .into()
}

/// Renders a modal dialog for adding a new to-do task.
pub fn add_todo_modal<'a>(input_buffer: &'a str) -> Element<'a, Message> {
    let title = text("Add New To-Do").size(18).color(theme::TEXT_PRIMARY);

    let input_field = text_input(
        "Enter task title... (Enter to save, Esc to cancel)",
        input_buffer,
    )
    .on_input(Message::InputChanged)
    .on_submit(Message::SubmitInput)
    .style(theme::custom_text_input)
    .padding(10)
    .size(14);

    let save_btn = button(text("Save [Enter]").size(13))
        .on_press(Message::SubmitInput)
        .style(theme::primary_button)
        .padding([6, 14]);

    let cancel_btn = button(text("Cancel [Esc]").size(13))
        .on_press(Message::CancelModal)
        .style(theme::secondary_button)
        .padding([6, 14]);

    let action_row = row![cancel_btn, Space::with_width(12), save_btn].align_y(Alignment::Center);

    let card = container(
        column![
            title,
            Space::with_height(12),
            input_field,
            Space::with_height(16),
            action_row,
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fixed(440.0)),
    )
    .style(theme::modal_card_container)
    .padding(20);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::modal_backdrop)
        .into()
}

/// Renders a modal dialog for adding a new habit.
pub fn add_habit_modal<'a>(input_buffer: &'a str) -> Element<'a, Message> {
    let title = text("Add New Habit").size(18).color(theme::TEXT_PRIMARY);

    let input_field = text_input(
        "Enter habit name... (Enter to save, Esc to cancel)",
        input_buffer,
    )
    .on_input(Message::InputChanged)
    .on_submit(Message::SubmitInput)
    .style(theme::custom_text_input)
    .padding(10)
    .size(14);

    let save_btn = button(text("Save [Enter]").size(13))
        .on_press(Message::SubmitInput)
        .style(theme::primary_button)
        .padding([6, 14]);

    let cancel_btn = button(text("Cancel [Esc]").size(13))
        .on_press(Message::CancelModal)
        .style(theme::secondary_button)
        .padding([6, 14]);

    let action_row = row![cancel_btn, Space::with_width(12), save_btn].align_y(Alignment::Center);

    let card = container(
        column![
            title,
            Space::with_height(12),
            input_field,
            Space::with_height(16),
            action_row,
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fixed(440.0)),
    )
    .style(theme::modal_card_container)
    .padding(20);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::modal_backdrop)
        .into()
}

/// Renders a modal dialog for renaming a habit.
pub fn rename_habit_modal<'a>(_habit_id: i64, input_buffer: &'a str) -> Element<'a, Message> {
    let title = text("Rename Habit").size(18).color(theme::TEXT_PRIMARY);

    let input_field = text_input("Enter new habit name...", input_buffer)
        .on_input(Message::InputChanged)
        .on_submit(Message::SubmitInput)
        .style(theme::custom_text_input)
        .padding(10)
        .size(14);

    let save_btn = button(text("Save [Enter]").size(13))
        .on_press(Message::SubmitInput)
        .style(theme::primary_button)
        .padding([6, 14]);

    let cancel_btn = button(text("Cancel [Esc]").size(13))
        .on_press(Message::CancelModal)
        .style(theme::secondary_button)
        .padding([6, 14]);

    let action_row = row![cancel_btn, Space::with_width(12), save_btn].align_y(Alignment::Center);

    let card = container(
        column![
            title,
            Space::with_height(12),
            input_field,
            Space::with_height(16),
            action_row,
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(Length::Fixed(440.0)),
    )
    .style(theme::modal_card_container)
    .padding(20);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::modal_backdrop)
        .into()
}

/// Renders the keyboard shortcuts help guide modal overlay.
pub fn help_modal<'a>() -> Element<'a, Message> {
    let title = text("Keyboard Shortcuts Guide")
        .size(20)
        .color(theme::ACCENT_CYAN);

    let close_btn = button(text("[Esc / ?] Close").size(13))
        .on_press(Message::ToggleHelp)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let header = row![title, Space::with_width(Length::Fill), close_btn].align_y(Alignment::Center);

    // Section 1: Global Navigation
    let global_sec = column![
        text("Global Shortcuts").size(15).color(theme::ACCENT_GREEN),
        Space::with_height(6),
        render_shortcut_pill("q", "Quit application"),
        render_shortcut_pill("s / Tab", "Toggle metrics sidebar"),
        render_shortcut_pill("o", "Open Settings / Dashboard"),
        render_shortcut_pill("Esc", "Cancel / Return to Main"),
        render_shortcut_pill("?", "Toggle this help dialog"),
    ]
    .spacing(4);

    // Section 2: Main Navigation
    let nav_sec = column![
        text("Main Navigation").size(15).color(theme::ACCENT_BLUE),
        Space::with_height(6),
        render_shortcut_pill("j / ↓", "Move cursor down"),
        render_shortcut_pill("k / ↑", "Move cursor up"),
        render_shortcut_pill("h", "Previous calendar day"),
        render_shortcut_pill("l", "Next calendar day"),
        render_shortcut_pill("t", "Jump to today"),
        render_shortcut_pill("g / G", "Jump to top / bottom"),
    ]
    .spacing(4);

    // Section 3: Task Actions
    let action_sec = column![
        text("Task Actions").size(15).color(theme::ACCENT_CYAN),
        Space::with_height(6),
        render_shortcut_pill("Space", "Toggle completed status"),
        render_shortcut_pill("x", "Complete / dismiss item"),
        render_shortcut_pill("i", "Add new to-do item"),
        render_shortcut_pill("Enter", "Edit task title"),
        render_shortcut_pill("n", "Edit task notes"),
        render_shortcut_pill("d", "Delete to-do (with confirm)"),
    ]
    .spacing(4);

    // Section 4: Settings View
    let settings_sec = column![
        text("Settings View").size(15).color(theme::TEXT_PRIMARY),
        Space::with_height(6),
        render_shortcut_pill("j / k", "Select habit"),
        render_shortcut_pill("i", "Add new daily habit"),
        render_shortcut_pill("r", "Rename selected habit"),
        render_shortcut_pill("J / K", "Reorder habit down / up"),
        render_shortcut_pill("d", "Archive / soft-delete habit"),
        render_shortcut_pill("e", "Export full database backup to JSON"),
    ]
    .spacing(4);

    let guide_grid = row![
        column![global_sec, Space::with_height(12), action_sec].spacing(10),
        Space::with_width(32),
        column![nav_sec, Space::with_height(12), settings_sec].spacing(10),
    ];

    let card =
        container(column![header, Space::with_height(16), guide_grid,].width(Length::Fixed(560.0)))
            .style(theme::modal_card_container)
            .padding(24);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(theme::modal_backdrop)
        .into()
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        AppState::new(db).unwrap()
    }

    #[test]
    fn test_bottom_shortcut_bar_all_modes() {
        let mut state = setup_test_state();

        // Main Screen Normal
        {
            state.screen = Screen::Main;
            state.input_mode = InputMode::Normal;
            let _bar = bottom_shortcut_bar(&state);
        }

        // Settings Screen Normal
        {
            state.screen = Screen::Settings;
            state.input_mode = InputMode::Normal;
            let _bar = bottom_shortcut_bar(&state);
        }

        // Adding To-Do
        {
            state.input_mode = InputMode::AddingTodo;
            let _bar = bottom_shortcut_bar(&state);
        }

        // Editing To-Do
        {
            state.input_mode = InputMode::EditingTodo(1);
            let _bar = bottom_shortcut_bar(&state);
        }

        // Editing Notes
        {
            state.input_mode = InputMode::EditingNotes(1);
            let _bar = bottom_shortcut_bar(&state);
        }

        // Adding Habit
        {
            state.input_mode = InputMode::AddingHabit;
            let _bar = bottom_shortcut_bar(&state);
        }

        // Renaming Habit
        {
            state.input_mode = InputMode::RenamingHabit(1);
            let _bar = bottom_shortcut_bar(&state);
        }

        // Confirm Delete
        {
            state.input_mode = InputMode::ConfirmDelete(1);
            let _bar = bottom_shortcut_bar(&state);
        }

        // Help Modal Active
        {
            state.show_help = true;
            let _bar = bottom_shortcut_bar(&state);
        }
    }

    #[test]
    fn test_confirm_delete_modal_variants() {
        let _modal_with_title = confirm_delete_modal(1, Some("Finish report"));
        let _modal_without_title = confirm_delete_modal(2, None);
    }

    #[test]
    fn test_help_modal() {
        let _modal = help_modal();
    }

    #[test]
    fn test_input_modals() {
        let _todo_modal = add_todo_modal("New Task");
        let _habit_modal = add_habit_modal("Morning Run");
        let _rename_modal = rename_habit_modal(1, "Updated Run");
    }

    #[test]
    fn test_render_shortcut_pill() {
        let _pill = render_shortcut_pill("Space", "Toggle");
    }
}
