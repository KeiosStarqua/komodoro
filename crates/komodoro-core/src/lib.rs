//! Focus engine core. No Tauri, Leptos, or SQLite imports.
//!
//! The timer accepts a [`TimerEvent`] and returns the next [`TimerState`].
//! Phase order comes from a [`Program`]. A clock at the edge sends [`TimerEvent::Tick`].

mod model;
mod ports;
mod program;
mod timer;

pub use model::{
    AppSettings, Effect, FocusEvent, FocusEventKind, ProgramRecord, Session, StoreError, Task,
    SETTING_ACTIVE_PROGRAM, SETTING_NOTIFICATIONS,
};
pub use ports::{ProgramRepository, SessionRepository, SettingsRepository, TaskRepository};
pub use program::{PhaseKind, PhaseSpec, Program, ProgramCursor, ProgramError};
pub use timer::{
    format_remaining, AppSnapshot, Engine, Paused, PhaseRuntime, TimerError, TimerEvent,
    TimerState, UiCommand,
};
