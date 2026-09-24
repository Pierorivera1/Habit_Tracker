//! Main content view displaying Daily Habits and To-Dos.

use crate::app::message::Message;
use crate::app::state::{AppState, InputMode};
use crate::domain::{Habit, Todo};
use crate::ui::theme;
use chrono::Local;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

/// Creates a strikethrough representation of the string using unicode combining strikethrough characters.
pub fn strikethrough(s: &str) -> String {
    s.chars().flat_map(|c| [c, '\u{0336}']).collect()
}

/// Renders the main content view with the Daily Habits list and To-Dos list.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut content = column![].spacing(20).padding(20).width(Length::Fill);

    // =========================================================================
    // SECTION 1: Daily Habits (Non-negotiables)
    // =========================================================================
    let completed_habits_count = state.habits.iter().filter(|(_, c)| *c).count();
    let total_habits = state.habits.len();

    let habit_badge_text = if total_habits == 0 {
        "0 habits".to_string()
    } else {
        format!("{}/{} completed", completed_habits_count, total_habits)
    };

    let habit_header = row![
        text("Daily Habits").size(17).color(theme::TEXT_PRIMARY),
        Space::with_width(12),
        container(text(habit_badge_text).size(12).color(theme::ACCENT_GREEN))
            .style(theme::pill_container)
            .padding([3, 8]),
        Space::with_width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    content = content.push(habit_header);

    if state.habits.is_empty() {
        let empty_msg = container(
            text("No daily habits yet. Press 'o' to add habits in Settings.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        )
        .padding(16)
        .style(theme::card_container)
        .width(Length::Fill);

        content = content.push(empty_msg);
    } else {
        let mut habit_list = column![].spacing(8).width(Length::Fill);

        for (idx, (habit, is_completed)) in state.habits.iter().enumerate() {
            let is_selected = state.selected_index == idx;
            let habit_row = render_habit_row(habit, *is_completed, is_selected, state);
            habit_list = habit_list.push(habit_row);
        }

        content = content.push(habit_list);
    }

    // =========================================================================
    // SECTION 2: To-Dos
    // =========================================================================
    let pending_todos_count = state.todos.iter().filter(|t| !t.completed).count();
    let total_todos = state.todos.len();

    let todo_badge_text = format!("{}/{} pending", pending_todos_count, total_todos);

    let add_button = button(text("+ New Task [i]").size(13))
        .on_press(Message::SetInputMode(InputMode::AddingTodo))
        .style(theme::primary_button)
        .padding([4, 10]);

    let todo_header = row![
        text("To-Dos").size(17).color(theme::TEXT_PRIMARY),
        Space::with_width(12),
        container(text(todo_badge_text).size(12).color(theme::ACCENT_BLUE))
            .style(theme::pill_container)
            .padding([3, 8]),
        Space::with_width(Length::Fill),
        add_button,
    ]
    .align_y(Alignment::Center);

    content = content.push(todo_header);

    // Inline Add To-Do text input when active
    if state.input_mode == InputMode::AddingTodo {
        let input_field = text_input(
            "Enter task title... (Enter to save, Esc to cancel)",
            &state.input_buffer,
        )
        .id(text_input::Id::new("todo_input"))
        .on_input(Message::InputChanged)
        .on_submit(Message::SubmitInput)
        .style(theme::custom_text_input)
        .padding(10)
        .size(14);

        let add_row = container(
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

        content = content.push(add_row);
    }

    if state.todos.is_empty() && state.input_mode != InputMode::AddingTodo {
        let empty_msg = container(
            text("No to-dos on your list. Press 'a' to add a task.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        )
        .padding(16)
        .style(theme::card_container)
        .width(Length::Fill);

        content = content.push(empty_msg);
    } else {
        let mut todo_list = column![].spacing(8).width(Length::Fill);

        let habit_count = state.habits.len();
        for (idx, todo) in state.todos.iter().enumerate() {
            let global_idx = habit_count + idx;
            let is_selected = state.selected_index == global_idx;
            let todo_row = render_todo_row(todo, is_selected, state);
            todo_list = todo_list.push(todo_row);
        }

        content = content.push(todo_list);
    }

    scrollable(content)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

/// Renders an individual Habit row card.
fn render_habit_row<'a>(
    habit: &'a Habit,
    is_completed: bool,
    is_selected: bool,
    state: &'a AppState,
) -> Element<'a, Message> {
    let check = checkbox("", is_completed)
        .on_toggle(move |_| Message::ToggleHabit(habit.id))
        .style(theme::custom_checkbox);

    let habit_name = if is_completed {
        text(strikethrough(&habit.name))
            .size(15)
            .color(Color::from_rgba(
                156.0 / 255.0,
                163.0 / 255.0,
                175.0 / 255.0,
                0.5,
            ))
    } else {
        text(&habit.name).size(15).color(theme::TEXT_PRIMARY)
    };

    let mut row_content = row![
        check,
        Space::with_width(8),
        habit_name,
        Space::with_width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    // Show 30-day consistency badge if calculated
    if let Some((_, rate)) = state
        .habit_success_rates
        .iter()
        .find(|(id, _)| *id == habit.id)
    {
        let rate_text = format!("{:.0}% 30d", rate);
        let badge_color = if *rate >= 80.0 {
            theme::ACCENT_GREEN
        } else if *rate >= 50.0 {
            theme::ACCENT_BLUE
        } else {
            theme::TEXT_MUTED
        };

        row_content = row_content.push(
            container(text(rate_text).size(12).color(badge_color))
                .style(theme::pill_container)
                .padding([2, 6]),
        );
    }

    let card_style = if is_selected {
        theme::selected_card_container
    } else {
        theme::card_container
    };

    container(row_content)
        .style(card_style)
        .padding([10, 14])
        .width(Length::Fill)
        .into()
}

/// Renders an individual To-Do row card.
fn render_todo_row<'a>(
    todo: &'a Todo,
    is_selected: bool,
    state: &'a AppState,
) -> Element<'a, Message> {
    let check = checkbox("", todo.completed)
        .on_toggle(move |_| Message::ToggleTodo(todo.id))
        .style(theme::custom_checkbox);

    let is_editing_this_todo = state.input_mode == InputMode::EditingTodo(todo.id);
    let is_editing_this_notes = state.input_mode == InputMode::EditingNotes(todo.id);

    let mut col = column![].spacing(6).width(Length::Fill);

    if is_editing_this_todo {
        let edit_input = text_input("Task title...", &state.input_buffer)
            .id(text_input::Id::new("edit_todo_input"))
            .on_input(Message::InputChanged)
            .on_submit(Message::SubmitInput)
            .style(theme::custom_text_input)
            .padding(6)
            .size(14);

        let row_content = row![
            check,
            Space::with_width(8),
            edit_input,
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center);

        col = col.push(row_content);
    } else {
        let title_elem = if todo.completed {
            text(strikethrough(&todo.title))
                .size(15)
                .color(Color::from_rgba(
                    156.0 / 255.0,
                    163.0 / 255.0,
                    175.0 / 255.0,
                    0.5,
                ))
        } else {
            text(&todo.title).size(15).color(theme::TEXT_PRIMARY)
        };

        let mut row_content = row![
            check,
            Space::with_width(8),
            title_elem,
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center);

        // Due date badge
        if let Some(due) = todo.due_date {
            let today = Local::now().date_naive();
            let is_overdue = due < today && !todo.completed;

            let badge_text = if is_overdue {
                format!("⚠ Due: {}", due)
            } else {
                format!("📅 Due: {}", due)
            };

            let badge = if is_overdue {
                container(text(badge_text).size(12).color(theme::ACCENT_DANGER))
                    .style(theme::danger_badge_container)
                    .padding([2, 6])
            } else {
                container(text(badge_text).size(12).color(theme::TEXT_SECONDARY))
                    .style(theme::pill_container)
                    .padding([2, 6])
            };

            row_content = row_content.push(badge).push(Space::with_width(6));
        }

        // Notes indicator badge
        if todo.notes.is_some() {
            let notes_badge = container(text("📝 Notes").size(12).color(theme::ACCENT_BLUE))
                .style(theme::pill_container)
                .padding([2, 6]);

            row_content = row_content.push(notes_badge);
        }

        col = col.push(row_content);
    }

    // Notes editing or inline notes preview when selected
    if is_editing_this_notes {
        let notes_input = text_input(
            "Add notes... (Enter to save, Esc to cancel)",
            &state.notes_buffer,
        )
        .id(text_input::Id::new("edit_notes_input"))
        .on_input(Message::NotesChanged)
        .on_submit(Message::SubmitInput)
        .style(theme::custom_text_input)
        .padding(6)
        .size(13);

        let notes_box = container(notes_input)
            .style(theme::pill_container)
            .padding(8)
            .width(Length::Fill);

        col = col.push(notes_box);
    } else if is_selected {
        if let Some(notes) = &todo.notes {
            if !notes.is_empty() {
                let notes_display = container(
                    row![
                        text("📝").size(13).color(theme::ACCENT_BLUE),
                        Space::with_width(6),
                        text(notes).size(13).color(theme::TEXT_SECONDARY),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(theme::pill_container)
                .padding(8)
                .width(Length::Fill);

                col = col.push(notes_display);
            }
        }
    }

    let card_style = if is_selected {
        theme::selected_card_container
    } else {
        theme::card_container
    };

    container(col)
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
    use crate::db::Database;
    use crate::domain::{Habit, Todo};
    use chrono::Days;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        AppState::new(db).unwrap()
    }

    #[test]
    fn test_strikethrough_formatting() {
        let input = "Meditate";
        let output = strikethrough(input);
        assert!(output.contains('\u{0336}'));
        let combining_count = output.chars().filter(|c| *c == '\u{0336}').count();
        assert_eq!(combining_count, input.chars().count());
    }

    #[test]
    fn test_main_view_empty_state() {
        let state = setup_test_state();
        let _elem = view(&state);
    }

    #[test]
    fn test_main_view_with_habits_and_selection() {
        let mut state = setup_test_state();
        state.habits = vec![
            (
                Habit {
                    id: 1,
                    name: "Exercise".to_string(),
                    position: 1,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                false,
            ),
            (
                Habit {
                    id: 2,
                    name: "Read 20 pages".to_string(),
                    position: 2,
                    created_at: "2026-09-01".to_string(),
                    archived: false,
                },
                true,
            ),
        ];

        // Highlight first habit
        {
            state.selected_index = 0;
            let _elem0 = view(&state);
        }

        // Highlight completed habit
        {
            state.selected_index = 1;
            let _elem1 = view(&state);
        }
    }

    #[test]
    fn test_main_view_with_todos_and_badges() {
        let mut state = setup_test_state();
        let today = Local::now().date_naive();
        let overdue = today.checked_sub_days(Days::new(2)).unwrap();
        let future = today.checked_add_days(Days::new(5)).unwrap();

        state.todos = vec![
            Todo {
                id: 1,
                title: "Buy groceries".to_string(),
                notes: Some("Milk, eggs, coffee".to_string()),
                completed: false,
                created_at: "2026-09-01".to_string(),
                completed_at: None,
                due_date: Some(overdue),
                position: 1,
            },
            Todo {
                id: 2,
                title: "Review PR".to_string(),
                notes: None,
                completed: true,
                created_at: "2026-09-01".to_string(),
                completed_at: Some("2026-09-02".to_string()),
                due_date: Some(future),
                position: 2,
            },
        ];

        // Highlight todo
        {
            state.selected_index = 0;
            let _elem = view(&state);
        }

        // Test with InputMode::AddingTodo
        {
            state.input_mode = InputMode::AddingTodo;
            state.input_buffer = "New Task".to_string();
            let _elem = view(&state);
        }

        // Test with InputMode::EditingTodo
        {
            state.input_mode = InputMode::EditingTodo(1);
            state.input_buffer = "Updated title".to_string();
            let _elem = view(&state);
        }

        // Test with InputMode::EditingNotes
        {
            state.input_mode = InputMode::EditingNotes(1);
            state.notes_buffer = "Updated notes".to_string();
            let _elem = view(&state);
        }
    }
}
