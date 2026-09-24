//! Top navigation bar component for Habitodo.

use crate::app::message::Message;
use crate::app::state::{AppState, Screen};
use crate::ui::theme;
use chrono::Local;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

/// Renders the top navigation bar with application title, calendar date navigator,
/// quick action buttons, and status/toast messages.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    // Left: Brand Title
    let title = text("Habit Tracker").size(20).color(theme::TEXT_PRIMARY);

    // Center: Date Navigation
    let is_today = state.selected_date == Local::now().date_naive();
    let formatted_date = state.selected_date.format("%A, %b %-d, %Y").to_string();

    let prev_button = button(text("◀ [h]").size(13))
        .on_press(Message::PreviousDay)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let next_button = button(text("[l] ▶").size(13))
        .on_press(Message::NextDay)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let today_button = button(text("[t] Today").size(13))
        .on_press(Message::GoToday)
        .style(if is_today {
            theme::primary_button
        } else {
            theme::secondary_button
        })
        .padding([4, 10]);

    let date_label = text(formatted_date).size(15).color(if is_today {
        theme::ACCENT_CYAN
    } else {
        theme::TEXT_PRIMARY
    });

    let mut date_nav = row![prev_button, date_label]
        .spacing(12)
        .align_y(Alignment::Center);

    if is_today {
        date_nav = date_nav.push(
            container(text("TODAY").size(11).color(theme::ACCENT_GREEN))
                .style(theme::pill_container)
                .padding([2, 6]),
        );
    }

    date_nav = date_nav.push(today_button).push(next_button);

    // Right: Quick Action Controls
    let settings_msg = match state.screen {
        Screen::Main => Message::OpenSettings,
        Screen::Settings => Message::CloseSettings,
    };
    let settings_label = match state.screen {
        Screen::Main => "[o] Settings",
        Screen::Settings => "[Esc] Dashboard",
    };

    let settings_button = button(text(settings_label).size(13))
        .on_press(settings_msg)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let sidebar_label = if state.sidebar_open {
        "[s] Hide Metrics"
    } else {
        "[s] Show Metrics"
    };

    let sidebar_button = button(text(sidebar_label).size(13))
        .on_press(Message::ToggleSidebar)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let help_button = button(text("[?] Help").size(13))
        .on_press(Message::ToggleHelp)
        .style(theme::secondary_button)
        .padding([4, 10]);

    let actions = row![sidebar_button, settings_button, help_button]
        .spacing(8)
        .align_y(Alignment::Center);

    let main_row = row![
        title,
        Space::with_width(Length::Fill),
        date_nav,
        Space::with_width(Length::Fill),
        actions
    ]
    .align_y(Alignment::Center)
    .padding([10, 20]);

    // Optional status message / toast banner
    if let Some(status) = &state.status_message {
        let is_error =
            status.contains("error") || status.contains("Error") || status.contains("failed");
        let status_color = if is_error {
            theme::ACCENT_DANGER
        } else {
            theme::ACCENT_GREEN
        };

        let status_text = text(status).size(13).color(status_color);

        let status_badge = container(status_text)
            .style(if is_error {
                theme::danger_badge_container
            } else {
                theme::pill_container
            })
            .padding([4, 12]);

        let col = iced::widget::column![
            main_row,
            container(status_badge)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 6.0,
                    left: 0.0,
                })
        ];

        container(col)
            .width(Length::Fill)
            .style(theme::top_bar_container)
            .into()
    } else {
        container(main_row)
            .width(Length::Fill)
            .style(theme::top_bar_container)
            .into()
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::NaiveDate;

    fn setup_test_state() -> AppState {
        let db = Database::new_in_memory().unwrap();
        AppState::new(db).unwrap()
    }

    #[test]
    fn test_top_bar_view_default() {
        let state = setup_test_state();
        let _element = view(&state);
        // Verify element creates cleanly without panic
    }

    #[test]
    fn test_top_bar_view_with_status_messages() {
        let mut state = setup_test_state();

        // Normal success status
        {
            state.status_message = Some("Task saved successfully".to_string());
            let _elem_success = view(&state);
        }

        // Error status
        {
            state.status_message = Some("Database error: disk full".to_string());
            let _elem_error = view(&state);
        }
    }

    #[test]
    fn test_top_bar_view_date_variants() {
        let mut state = setup_test_state();

        // Historical date
        {
            state.selected_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
            let _elem_past = view(&state);
        }

        // Future date
        {
            state.selected_date = NaiveDate::from_ymd_opt(2035, 12, 31).unwrap();
            let _elem_future = view(&state);
        }
    }

    #[test]
    fn test_top_bar_view_screens() {
        let mut state = setup_test_state();

        {
            state.screen = Screen::Main;
            let _elem_main = view(&state);
        }

        {
            state.screen = Screen::Settings;
            let _elem_settings = view(&state);
        }
    }
}
