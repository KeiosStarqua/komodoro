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

/// Aggregated focus history. `focus_ms` counts completed focus sessions only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SessionSummary {
    pub focus_ms: u64,
    pub focus_completed: u32,
    pub focus_skipped: u32,
}

impl SessionSummary {
    /// Summarize focus sessions started at or after `since_ms`.
    /// Interrupted sessions without an end time do not count as focused time.
    pub fn since(sessions: &[Session], since_ms: i64) -> Self {
        let mut summary = Self::default();
        for session in sessions {
            if session.phase != PhaseKind::Focus || session.started_at_ms < since_ms {
                continue;
            }
            match (session.completed, session.ended_at_ms) {
                (true, Some(end)) => {
                    summary.focus_completed += 1;
                    summary.focus_ms += end.saturating_sub(session.started_at_ms).max(0) as u64;
                }
                (false, Some(_)) => summary.focus_skipped += 1,
                _ => {}
            }
        }
        summary
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_only_completed_focus_inside_the_window() {
        let session = |phase, started, ended, completed| Session {
            id: Uuid::nil(),
            program_id: Uuid::nil(),
            phase,
            label: None,
            started_at_ms: started,
            ended_at_ms: ended,
            completed,
        };
        let sessions = vec![
            session(PhaseKind::Focus, 1_000, Some(2_000), true),
            session(PhaseKind::Focus, 3_000, Some(3_500), true),
            session(PhaseKind::ShortBreak, 4_000, Some(4_100), true),
            session(PhaseKind::Focus, 5_000, Some(5_250), false),
            session(PhaseKind::Focus, 6_000, None, false),
            session(PhaseKind::Focus, 100, Some(900), true),
        ];
        let summary = SessionSummary::since(&sessions, 1_000);
        assert_eq!(summary.focus_completed, 2);
        assert_eq!(summary.focus_ms, 1_500);
        assert_eq!(summary.focus_skipped, 1);
    }
}
