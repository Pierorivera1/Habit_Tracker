//! Application state machine and message dispatching following Elm Architecture.

pub mod message;
pub mod state;

pub use message::Message;
pub use state::{AppState, InputMode, Screen};
