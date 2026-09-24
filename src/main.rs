//! Habitodo: Native desktop habit and to-do tracking application.
//!
//! Entry point for Habitodo desktop application, CLI argument dispatcher,
//! and iced GUI application bootstrapping.

use habitodo::app::{AppState, InputMode, Message, Screen};
use habitodo::db::{default_db_path, Database};
use habitodo::keybindings::handle_key;
use habitodo::{help_text, run_verification, version_text};
use iced::{event, window, Event, Size, Subscription, Task, Theme};
use std::sync::RwLock;

/// Global context cache storing the active screen and input mode for pure key mapping.
static KEY_CTX: RwLock<(Screen, InputMode)> = RwLock::new((Screen::Main, InputMode::Normal));

/// Updates the cached keyboard navigation context from the latest AppState.
fn update_key_context(screen: Screen, input_mode: &InputMode) {
    if let Ok(mut ctx) = KEY_CTX.write() {
        *ctx = (screen, input_mode.clone());
    }
}

/// Translates raw iced runtime events into application domain messages.
/// Filters out captured events (e.g. active text input) and translates
/// uncaptured key presses through `keybindings::handle_key`.
fn handle_runtime_event(
    event: Event,
    status: event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    if status == event::Status::Captured {
        return None;
    }

    match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            let (screen, input_mode) = {
                let ctx = KEY_CTX.read().ok()?;
                ctx.clone()
            };
            handle_key(&key, modifiers, screen, &input_mode)
        }
        _ => None,
    }
}

/// Periodic clock tick subscription producing `Message::Tick` every 60 seconds
/// to update active dates and status message dismissals.
fn clock_ticks() -> Subscription<Message> {
    struct ClockTick;
    Subscription::run_with_id(
        std::any::TypeId::of::<ClockTick>(),
        iced::stream::channel(4, |mut sender| async move {
            use iced::futures::{SinkExt, StreamExt};
            let (mut tx, mut rx) = iced::futures::channel::mpsc::channel(4);
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                if tx.try_send(()).is_err() {
                    break;
                }
            });
            while rx.next().await.is_some() {
                if sender.send(Message::Tick).await.is_err() {
                    break;
                }
            }
        }),
    )
}

/// Central application subscription wiring:
/// - Listens for runtime keyboard events mapped through `handle_key`
/// - Produces periodic clock ticks every 60 seconds
fn subscription(state: &AppState) -> Subscription<Message> {
    update_key_context(state.screen, &state.input_mode);

    let keyboard_sub = event::listen_with(handle_runtime_event);
    let tick_sub = clock_ticks();

    Subscription::batch([keyboard_sub, tick_sub])
}

/// Builds the custom dark theme adhering to Habitodo's color palette specifications.
fn custom_theme(_state: &AppState) -> Theme {
    use habitodo::ui::theme;
    use iced::theme::Palette;

    Theme::custom(
        "HabitodoDark".to_string(),
        Palette {
            background: theme::BG_MAIN,
            text: theme::TEXT_PRIMARY,
            primary: theme::ACCENT_CYAN,
            success: theme::ACCENT_GREEN,
            danger: theme::ACCENT_DANGER,
        },
    )
}

/// Bootstraps and executes the iced graphical user interface.
fn run_gui() -> iced::Result {
    let window_settings = window::Settings {
        size: Size::new(1100.0, 700.0),
        min_size: Some(Size::new(800.0, 500.0)),
        ..Default::default()
    };

    let db_path = default_db_path();
    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(err) => {
            eprintln!(
                "Warning: Failed to open database at {}: {}. Falling back to in-memory database.",
                db_path.display(),
                err
            );
            Database::new_in_memory().expect("In-memory database initialization failed")
        }
    };

    let initial_state = match AppState::new(db) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Error initializing application state: {}", err);
            std::process::exit(1);
        }
    };

    iced::application("Habit Tracker", AppState::update, habitodo::ui::view)
        .subscription(subscription)
        .theme(custom_theme)
        .window(window_settings)
        .run_with(move || (initial_state, Task::none()))
}

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();

    if let Some(first_arg) = args.get(1) {
        match first_arg.as_str() {
            "-h" | "--help" => {
                println!("{}", help_text());
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("{}", version_text());
                std::process::exit(0);
            }
            "--verify" => match run_verification() {
                Ok(logs) => {
                    println!("=== Habitodo Self-Verification Diagnostics ===");
                    for log in &logs {
                        println!("[PASS] {}", log);
                    }
                    println!(
                        "All diagnostics passed successfully! ({} checks passed)",
                        logs.len()
                    );
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("=== Habitodo Self-Verification FAILED ===");
                    eprintln!("[FAIL] {}", err);
                    std::process::exit(1);
                }
            },
            unrecognized => {
                eprintln!("error: unrecognized argument '{}'\n", unrecognized);
                eprintln!("Usage: habitodo [OPTIONS]\n");
                eprintln!("For more information, try '--help'.");
                std::process::exit(1);
            }
        }
    }

    run_gui()
}
