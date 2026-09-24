//! Pure keybinding translation engine for Habitodo.
//!
//! Maps raw iced keyboard events and modifiers into domain `Message` actions
//! based on active `Screen` and `InputMode`, with strict text input focus isolation.

use crate::app::message::Message;
use crate::app::state::{AppState, InputMode, Screen};
use iced::keyboard::{key::Named, Key, Modifiers};

/// Checks if an active input mode is capturing textual keyboard input.
pub fn is_text_input_mode(input_mode: &InputMode) -> bool {
    matches!(
        input_mode,
        InputMode::AddingTodo
            | InputMode::EditingTodo(_)
            | InputMode::EditingNotes(_)
            | InputMode::AddingHabit
            | InputMode::RenamingHabit(_)
    )
}

/// Helper checking if a key matches a specific character string.
fn is_char(key: &Key, expected: &str) -> bool {
    matches!(key, Key::Character(c) if c.as_str() == expected)
}

/// Helper checking if a key matches an uppercase character or a lowercase character with Shift held.
fn is_upper_or_shifted(key: &Key, upper: &str, lower: &str, modifiers: Modifiers) -> bool {
    is_char(key, upper) || (is_char(key, lower) && modifiers.shift())
}

/// Helper checking if a key matches a lowercase character without Shift held.
fn is_lower_unshifted(key: &Key, lower: &str, modifiers: Modifiers) -> bool {
    is_char(key, lower) && !modifiers.shift()
}

/// Pure function translating raw iced keyboard events into domain `Message` actions.
///
/// # Strict Text Input Focus Isolation
/// When `input_mode` is actively capturing text (e.g. `AddingTodo`, `EditingTodo`, `AddingHabit`),
/// single-character keys (`j`, `k`, `q`, `a`, `d`, `Space`) are NOT intercepted, ensuring
/// that keystrokes flow directly into the text input widget. Only `Enter` (commit) and
/// `Esc` (cancel) are translated into messages.
pub fn handle_key(
    key: &Key,
    modifiers: Modifiers,
    screen: Screen,
    input_mode: &InputMode,
) -> Option<Message> {
    // -----------------------------------------------------------------
    // 1. Modal / Text Input Focus Protection
    // -----------------------------------------------------------------
    if let InputMode::ConfirmDelete(id) = input_mode {
        return match key {
            Key::Character(c) if c.as_str().eq_ignore_ascii_case("y") => {
                Some(Message::ConfirmDelete(*id))
            }
            Key::Named(Named::Enter) => Some(Message::ConfirmDelete(*id)),
            Key::Character(c) if c.as_str().eq_ignore_ascii_case("n") => Some(Message::CancelModal),
            Key::Named(Named::Escape) => Some(Message::CancelModal),
            _ => None,
        };
    }

    if is_text_input_mode(input_mode) {
        return match key {
            Key::Named(Named::Enter) => Some(Message::SubmitInput),
            Key::Named(Named::Escape) => Some(Message::CancelModal),
            // Strictly isolate text input: all single-char keys return None
            _ => None,
        };
    }

    // -----------------------------------------------------------------
    // 2. Global Shortcuts (Active across all screens in Normal mode)
    // -----------------------------------------------------------------
    if is_char(key, "q") && !modifiers.control() && !modifiers.alt() {
        return Some(Message::Quit);
    }
    if is_char(key, "s") || matches!(key, Key::Named(Named::Tab)) {
        return Some(Message::ToggleSidebar);
    }
    if is_char(key, "?") {
        return Some(Message::ToggleHelp);
    }

    // -----------------------------------------------------------------
    // 3. Screen-Specific Shortcuts
    // -----------------------------------------------------------------
    match screen {
        Screen::Main => match key {
            // Navigation
            _ if is_lower_unshifted(key, "j", modifiers)
                || matches!(key, Key::Named(Named::ArrowDown)) =>
            {
                Some(Message::NavigateDown)
            }
            _ if is_lower_unshifted(key, "k", modifiers)
                || matches!(key, Key::Named(Named::ArrowUp)) =>
            {
                Some(Message::NavigateUp)
            }
            _ if is_lower_unshifted(key, "h", modifiers)
                || matches!(key, Key::Named(Named::ArrowLeft)) =>
            {
                Some(Message::PreviousDay)
            }
            _ if is_lower_unshifted(key, "l", modifiers)
                || matches!(key, Key::Named(Named::ArrowRight)) =>
            {
                Some(Message::NextDay)
            }
            _ if is_lower_unshifted(key, "t", modifiers) => Some(Message::GoToday),
            _ if is_lower_unshifted(key, "g", modifiers) => Some(Message::GoTop),
            _ if is_upper_or_shifted(key, "G", "g", modifiers) => Some(Message::GoBottom),

            // Actions
            Key::Named(Named::Space) => Some(Message::ToggleSelected),
            _ if is_char(key, "x") => Some(Message::CompleteSelected),
            _ if is_char(key, "i") => Some(Message::SetInputMode(InputMode::AddingTodo)),
            Key::Named(Named::Enter) => Some(Message::StartEdit),
            _ if is_char(key, "d") => Some(Message::DeleteSelected),
            _ if is_char(key, "n") => Some(Message::StartEditNotes),
            _ if is_char(key, "o") => Some(Message::SwitchScreen(Screen::Settings)),
            Key::Named(Named::Escape) => Some(Message::CancelModal),
            _ => None,
        },

        Screen::Settings => match key {
            // Navigation
            _ if is_lower_unshifted(key, "j", modifiers)
                || matches!(key, Key::Named(Named::ArrowDown)) =>
            {
                Some(Message::NavigateDown)
            }
            _ if is_lower_unshifted(key, "k", modifiers)
                || matches!(key, Key::Named(Named::ArrowUp)) =>
            {
                Some(Message::NavigateUp)
            }
            _ if is_lower_unshifted(key, "g", modifiers) => Some(Message::GoTop),
            _ if is_upper_or_shifted(key, "G", "g", modifiers) => Some(Message::GoBottom),

            // Reordering via J (down) and K (up)
            _ if is_upper_or_shifted(key, "K", "k", modifiers) => {
                Some(Message::ReorderSelected(true))
            }
            _ if is_upper_or_shifted(key, "J", "j", modifiers) => {
                Some(Message::ReorderSelected(false))
            }

            // Actions
            _ if is_char(key, "i") => Some(Message::SetInputMode(InputMode::AddingHabit)),
            _ if is_char(key, "d") => Some(Message::ArchiveSelected),
            _ if is_char(key, "r") => Some(Message::StartRename),
            _ if is_char(key, "e") => Some(Message::ExportData),
            Key::Named(Named::Escape) => Some(Message::SwitchScreen(Screen::Main)),
            _ if is_char(key, "o") => Some(Message::SwitchScreen(Screen::Main)),
            _ => None,
        },
    }
}

/// Convenience wrapper delegating to `handle_key` using current state properties.
pub fn handle_key_event(key: &Key, modifiers: Modifiers, app_state: &AppState) -> Option<Message> {
    handle_key(key, modifiers, app_state.screen, &app_state.input_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_key(c: &str) -> Key {
        Key::Character(c.into())
    }

    #[test]
    fn test_normal_mode_main_navigation_shortcuts() {
        let empty_mod = Modifiers::empty();
        let shift_mod = Modifiers::SHIFT;

        // j / ArrowDown -> NavigateDown
        assert_eq!(
            handle_key(&char_key("j"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::NavigateDown)
        );
        assert_eq!(
            handle_key(
                &Key::Named(Named::ArrowDown),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::NavigateDown)
        );

        // k / ArrowUp -> NavigateUp
        assert_eq!(
            handle_key(&char_key("k"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::NavigateUp)
        );
        assert_eq!(
            handle_key(
                &Key::Named(Named::ArrowUp),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::NavigateUp)
        );

        // h / ArrowLeft -> PreviousDay
        assert_eq!(
            handle_key(&char_key("h"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::PreviousDay)
        );
        assert_eq!(
            handle_key(
                &Key::Named(Named::ArrowLeft),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::PreviousDay)
        );

        // l / ArrowRight -> NextDay
        assert_eq!(
            handle_key(&char_key("l"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::NextDay)
        );
        assert_eq!(
            handle_key(
                &Key::Named(Named::ArrowRight),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::NextDay)
        );

        // t -> GoToday
        assert_eq!(
            handle_key(&char_key("t"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::GoToday)
        );

        // g -> GoTop
        assert_eq!(
            handle_key(&char_key("g"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::GoTop)
        );

        // G (uppercase or shifted) -> GoBottom
        assert_eq!(
            handle_key(&char_key("G"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::GoBottom)
        );
        assert_eq!(
            handle_key(&char_key("g"), shift_mod, Screen::Main, &InputMode::Normal),
            Some(Message::GoBottom)
        );
    }

    #[test]
    fn test_normal_mode_main_action_shortcuts() {
        let empty_mod = Modifiers::empty();

        // Space -> ToggleSelected
        assert_eq!(
            handle_key(
                &Key::Named(Named::Space),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::ToggleSelected)
        );

        // x -> CompleteSelected
        assert_eq!(
            handle_key(&char_key("x"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::CompleteSelected)
        );

        // i -> SetInputMode(AddingTodo)
        assert_eq!(
            handle_key(&char_key("i"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::SetInputMode(InputMode::AddingTodo))
        );

        // a -> Unassigned
        assert_eq!(
            handle_key(&char_key("a"), empty_mod, Screen::Main, &InputMode::Normal),
            None
        );

        // Enter -> StartEdit
        assert_eq!(
            handle_key(
                &Key::Named(Named::Enter),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::StartEdit)
        );

        // d -> DeleteSelected
        assert_eq!(
            handle_key(&char_key("d"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::DeleteSelected)
        );

        // n -> StartEditNotes
        assert_eq!(
            handle_key(&char_key("n"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::StartEditNotes)
        );

        // o -> SwitchScreen(Settings)
        assert_eq!(
            handle_key(&char_key("o"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::SwitchScreen(Screen::Settings))
        );

        // q -> Quit
        assert_eq!(
            handle_key(&char_key("q"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::Quit)
        );

        // s / Tab -> ToggleSidebar
        assert_eq!(
            handle_key(&char_key("s"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::ToggleSidebar)
        );
        assert_eq!(
            handle_key(
                &Key::Named(Named::Tab),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::ToggleSidebar)
        );

        // ? -> ToggleHelp
        assert_eq!(
            handle_key(&char_key("?"), empty_mod, Screen::Main, &InputMode::Normal),
            Some(Message::ToggleHelp)
        );

        // Esc -> CancelModal
        assert_eq!(
            handle_key(
                &Key::Named(Named::Escape),
                empty_mod,
                Screen::Main,
                &InputMode::Normal
            ),
            Some(Message::CancelModal)
        );
    }

    #[test]
    fn test_normal_mode_settings_shortcuts() {
        let empty_mod = Modifiers::empty();
        let shift_mod = Modifiers::SHIFT;

        // j / k navigation in settings
        assert_eq!(
            handle_key(
                &char_key("j"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::NavigateDown)
        );
        assert_eq!(
            handle_key(
                &char_key("k"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::NavigateUp)
        );

        // i -> AddingHabit
        assert_eq!(
            handle_key(
                &char_key("i"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::SetInputMode(InputMode::AddingHabit))
        );

        // a -> Unassigned
        assert_eq!(
            handle_key(
                &char_key("a"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            None
        );

        // d -> ArchiveSelected
        assert_eq!(
            handle_key(
                &char_key("d"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ArchiveSelected)
        );

        // r -> StartRename
        assert_eq!(
            handle_key(
                &char_key("r"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::StartRename)
        );

        // Reordering: K (up) and J (down)
        assert_eq!(
            handle_key(
                &char_key("K"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ReorderSelected(true))
        );
        assert_eq!(
            handle_key(
                &char_key("k"),
                shift_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ReorderSelected(true))
        );

        assert_eq!(
            handle_key(
                &char_key("J"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ReorderSelected(false))
        );
        assert_eq!(
            handle_key(
                &char_key("j"),
                shift_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ReorderSelected(false))
        );

        // e -> ExportData
        assert_eq!(
            handle_key(
                &char_key("e"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::ExportData)
        );

        // Esc / o -> SwitchScreen(Main)
        assert_eq!(
            handle_key(
                &Key::Named(Named::Escape),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::SwitchScreen(Screen::Main))
        );
        assert_eq!(
            handle_key(
                &char_key("o"),
                empty_mod,
                Screen::Settings,
                &InputMode::Normal
            ),
            Some(Message::SwitchScreen(Screen::Main))
        );
    }

    #[test]
    fn test_strict_text_input_focus_isolation() {
        let empty_mod = Modifiers::empty();

        let input_modes = vec![
            InputMode::AddingTodo,
            InputMode::EditingTodo(1),
            InputMode::EditingNotes(1),
            InputMode::AddingHabit,
            InputMode::RenamingHabit(1),
        ];

        let shortcut_keys = vec![
            char_key("j"),
            char_key("k"),
            char_key("h"),
            char_key("l"),
            char_key("q"),
            char_key("a"),
            char_key("d"),
            char_key("x"),
            char_key("s"),
            char_key("t"),
            char_key("o"),
            char_key("?"),
            Key::Named(Named::Space),
            Key::Named(Named::Tab),
        ];

        for mode in &input_modes {
            // All shortcuts MUST be None during text input
            for key in &shortcut_keys {
                assert_eq!(
                    handle_key(key, empty_mod, Screen::Main, mode),
                    None,
                    "Key {:?} must NOT trigger shortcut in mode {:?}",
                    key,
                    mode
                );
            }

            // Only Enter (commit) and Escape (cancel) are translated
            assert_eq!(
                handle_key(&Key::Named(Named::Enter), empty_mod, Screen::Main, mode),
                Some(Message::SubmitInput)
            );
            assert_eq!(
                handle_key(&Key::Named(Named::Escape), empty_mod, Screen::Main, mode),
                Some(Message::CancelModal)
            );
        }
    }

    #[test]
    fn test_confirm_delete_modal_keys() {
        let empty_mod = Modifiers::empty();
        let mode = InputMode::ConfirmDelete(99);

        // y / Y / Enter -> ConfirmDelete(99)
        assert_eq!(
            handle_key(&char_key("y"), empty_mod, Screen::Main, &mode),
            Some(Message::ConfirmDelete(99))
        );
        assert_eq!(
            handle_key(&char_key("Y"), empty_mod, Screen::Main, &mode),
            Some(Message::ConfirmDelete(99))
        );
        assert_eq!(
            handle_key(&Key::Named(Named::Enter), empty_mod, Screen::Main, &mode),
            Some(Message::ConfirmDelete(99))
        );

        // n / N / Esc -> CancelModal
        assert_eq!(
            handle_key(&char_key("n"), empty_mod, Screen::Main, &mode),
            Some(Message::CancelModal)
        );
        assert_eq!(
            handle_key(&char_key("N"), empty_mod, Screen::Main, &mode),
            Some(Message::CancelModal)
        );
        assert_eq!(
            handle_key(&Key::Named(Named::Escape), empty_mod, Screen::Main, &mode),
            Some(Message::CancelModal)
        );

        // Other keys are ignored
        assert_eq!(
            handle_key(&char_key("j"), empty_mod, Screen::Main, &mode),
            None
        );
        assert_eq!(
            handle_key(&char_key("q"), empty_mod, Screen::Main, &mode),
            None
        );
        assert_eq!(
            handle_key(&Key::Named(Named::Space), empty_mod, Screen::Main, &mode),
            None
        );
    }
}
