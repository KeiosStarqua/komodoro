use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::program::{PhaseKind, Program};

pub const SETTING_ACTIVE_PROGRAM: &str = "active_program_id";
pub const SETTING_NOTIFICATIONS: &str = "notifications";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusEventKind {
    Started,
    Completed,
    Skipped,
}

impl FocusEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "started" => Ok(Self::Started),
            "completed" => Ok(Self::Completed),
            "skipped" => Ok(Self::Skipped),
            other => Err(StoreError::Message(format!("unknown focus event {other}"))),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramRecord {
    pub id: Uuid,
    pub name: String,
    pub program: Program,
    pub preset: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub program_id: Uuid,
    pub phase: PhaseKind,
    pub label: Option<String>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub done: bool,
    pub session_id: Option<Uuid>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusEvent {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub program_id: Uuid,
    pub phase: PhaseKind,
    pub label: Option<String>,
    pub at_ms: i64,
    pub kind: FocusEventKind,
}

/// Work for the shell after a pure transition. The edge assigns ids and timestamps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    OpenSession {
        phase: PhaseKind,
        label: Option<String>,
    },
    CloseSession {
        phase: PhaseKind,
        label: Option<String>,
        completed: bool,
    },
    Notify {
        title: String,
        body: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub notifications: bool,
    pub shortcut: String,
}
