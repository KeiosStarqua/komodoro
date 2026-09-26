use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::Effect;
use crate::program::{PhaseKind, PhaseSpec, Program, ProgramCursor};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRuntime {
    pub label: Option<String>,
    pub duration_ms: u64,
    pub remaining_ms: u64,
    pub cursor: ProgramCursor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paused {
    pub kind: PhaseKind,
    pub runtime: PhaseRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TimerState {
    Idle,
    Focus(PhaseRuntime),
    ShortBreak(PhaseRuntime),
    LongBreak(PhaseRuntime),
    Paused(Paused),
}

impl TimerState {
    pub fn status_name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Focus(_) => "focus",
            Self::ShortBreak(_) => "short_break",
            Self::LongBreak(_) => "long_break",
            Self::Paused(_) => "paused",
        }
    }

    pub fn remaining_ms(&self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Focus(runtime) | Self::ShortBreak(runtime) | Self::LongBreak(runtime) => {
                Some(runtime.remaining_ms)
            }
            Self::Paused(paused) => Some(paused.runtime.remaining_ms),
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Idle => None,
            Self::Focus(runtime) | Self::ShortBreak(runtime) | Self::LongBreak(runtime) => {
                runtime.label.as_deref()
            }
            Self::Paused(paused) => paused.runtime.label.as_deref(),
        }
    }

    pub fn phase_kind(&self) -> Option<PhaseKind> {
        match self {
            Self::Idle => None,
            Self::Focus(_) => Some(PhaseKind::Focus),
            Self::ShortBreak(_) => Some(PhaseKind::ShortBreak),
            Self::LongBreak(_) => Some(PhaseKind::LongBreak),
            Self::Paused(paused) => Some(paused.kind),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimerEvent {
    Start,
    Pause,
    Resume,
    Skip,
    Tick { elapsed_ms: u64 },
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCommand {
    Start,
    Pause,
    Resume,
    Skip,
}

impl From<UiCommand> for TimerEvent {
    fn from(command: UiCommand) -> Self {
        match command {
            UiCommand::Start => Self::Start,
            UiCommand::Pause => Self::Pause,
            UiCommand::Resume => Self::Resume,
            UiCommand::Skip => Self::Skip,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TimerError {
    #[error("{event} is not valid while the timer is {state}")]
    Illegal {
        state: &'static str,
        event: &'static str,
    },
    #[error("the program has no phases")]
    EmptyProgram,
    #[error("stop the timer before changing the program")]
    Busy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSnapshot {
    pub timer: TimerState,
    pub program_id: Uuid,
    pub program_name: String,
    pub open_session_id: Option<Uuid>,
    /// Phase the next Start will enter. Present while idle.
    pub ready_kind: Option<PhaseKind>,
    pub ready_label: Option<String>,
    pub ready_ms: Option<u64>,
}

#[derive(Debug)]
pub struct Engine {
    state: TimerState,
    program: Program,
    program_id: Uuid,
    program_name: String,
}

impl Engine {
    pub fn new(program_id: Uuid, program_name: impl Into<String>, program: Program) -> Self {
        Self {
            state: TimerState::Idle,
            program,
            program_id,
            program_name: program_name.into(),
        }
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }

    pub fn program_id(&self) -> Uuid {
        self.program_id
    }

    pub fn program_name(&self) -> &str {
        &self.program_name
    }

    pub fn ready_phase(&self) -> Option<PhaseSpec> {
        if matches!(self.state, TimerState::Idle) {
            self.program.current_phase(&ProgramCursor::default())
        } else {
            None
        }
    }

    pub fn handle(&mut self, event: TimerEvent) -> Result<Vec<Effect>, TimerError> {
        let (next, effects) = apply(&self.state, &self.program, event)?;
        self.state = next;
        Ok(effects)
    }

    pub fn set_program(
        &mut self,
        id: Uuid,
        name: impl Into<String>,
        program: Program,
    ) -> Result<(), TimerError> {
        if !matches!(self.state, TimerState::Idle) {
            return Err(TimerError::Busy);
        }
        if program.is_empty() {
            return Err(TimerError::EmptyProgram);
        }
        self.program_id = id;
        self.program_name = name.into();
        self.program = program;
        Ok(())
    }
}

pub fn format_remaining(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn apply(
    state: &TimerState,
    program: &Program,
    event: TimerEvent,
) -> Result<(TimerState, Vec<Effect>), TimerError> {
    match event {
        TimerEvent::Start => start(state, program),
        TimerEvent::Pause => pause(state),
        TimerEvent::Resume => resume(state),
        TimerEvent::Skip => finish(state, program, false),
        TimerEvent::Complete => finish(state, program, true),
        TimerEvent::Tick { elapsed_ms } => tick(state, program, elapsed_ms),
    }
}

fn start(state: &TimerState, program: &Program) -> Result<(TimerState, Vec<Effect>), TimerError> {
    if !matches!(state, TimerState::Idle) {
        return Err(illegal(state, "start"));
    }
    let cursor = ProgramCursor::default();
    let Some(phase) = program.current_phase(&cursor) else {
        return Err(TimerError::EmptyProgram);
    };
    Ok(enter(phase, cursor))
}

fn pause(state: &TimerState) -> Result<(TimerState, Vec<Effect>), TimerError> {
    let Some((kind, runtime)) = active(state) else {
        return Err(illegal(state, "pause"));
    };
    Ok((TimerState::Paused(Paused { kind, runtime }), Vec::new()))
}

fn resume(state: &TimerState) -> Result<(TimerState, Vec<Effect>), TimerError> {
    match state {
        TimerState::Paused(paused) => {
            Ok((running(paused.kind, paused.runtime.clone()), Vec::new()))
        }
        _ => Err(illegal(state, "resume")),
    }
}

fn tick(
    state: &TimerState,
    program: &Program,
    elapsed_ms: u64,
) -> Result<(TimerState, Vec<Effect>), TimerError> {
    let Some((kind, mut runtime)) = active(state) else {
        return Ok((state.clone(), Vec::new()));
    };
    if elapsed_ms == 0 {
        return Ok((state.clone(), Vec::new()));
    }
    if elapsed_ms >= runtime.remaining_ms {
        return finish(state, program, true);
    }
    runtime.remaining_ms -= elapsed_ms;
    Ok((running(kind, runtime), Vec::new()))
}

fn finish(
    state: &TimerState,
    program: &Program,
    completed: bool,
) -> Result<(TimerState, Vec<Effect>), TimerError> {
    let Some((kind, runtime)) = active(state) else {
        return Err(illegal(state, if completed { "complete" } else { "skip" }));
    };
    let mut effects = vec![Effect::CloseSession {
        phase: kind,
        label: runtime.label.clone(),
        completed,
    }];
    match program.advance(&runtime.cursor, kind) {
        Some(cursor) => {
            let Some(phase) = program.current_phase(&cursor) else {
                effects.push(done_notice());
                return Ok((TimerState::Idle, effects));
            };
            let (next, mut opened) = enter(phase, cursor);
            effects.append(&mut opened);
            Ok((next, effects))
        }
        None => {
            effects.push(done_notice());
            Ok((TimerState::Idle, effects))
        }
    }
}

fn enter(phase: PhaseSpec, cursor: ProgramCursor) -> (TimerState, Vec<Effect>) {
    let runtime = PhaseRuntime {
        label: phase.label.clone(),
        duration_ms: phase.duration_ms,
        remaining_ms: phase.duration_ms,
        cursor,
    };
    let effects = vec![
        Effect::OpenSession {
            phase: phase.kind,
            label: phase.label.clone(),
        },
        notify_for(&phase),
    ];
    (running(phase.kind, runtime), effects)
}

fn notify_for(phase: &PhaseSpec) -> Effect {
    let title = match &phase.label {
        Some(label) => format!("{} · {label}", phase.kind.title()),
        None => phase.kind.title().to_string(),
    };
    Effect::Notify {
        title,
        body: format_remaining(phase.duration_ms),
    }
}

fn done_notice() -> Effect {
    Effect::Notify {
        title: "Done".into(),
        body: "Program finished".into(),
    }
}

fn active(state: &TimerState) -> Option<(PhaseKind, PhaseRuntime)> {
    match state {
        TimerState::Focus(runtime) => Some((PhaseKind::Focus, runtime.clone())),
        TimerState::ShortBreak(runtime) => Some((PhaseKind::ShortBreak, runtime.clone())),
        TimerState::LongBreak(runtime) => Some((PhaseKind::LongBreak, runtime.clone())),
        TimerState::Idle | TimerState::Paused(_) => None,
    }
}

fn running(kind: PhaseKind, runtime: PhaseRuntime) -> TimerState {
    match kind {
        PhaseKind::Focus => TimerState::Focus(runtime),
        PhaseKind::ShortBreak => TimerState::ShortBreak(runtime),
        PhaseKind::LongBreak => TimerState::LongBreak(runtime),
    }
}

fn illegal(state: &TimerState, event: &'static str) -> TimerError {
    TimerError::Illegal {
        state: state.status_name(),
        event,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::MINUTE_MS;

    fn engine(program: Program) -> Engine {
        Engine::new(Uuid::nil(), "test", program)
    }

    #[test]
    fn tick_only_shrinks_the_current_phase() {
        let mut engine = engine(Program::pomodoro());
        engine.handle(TimerEvent::Start).unwrap();
        engine
            .handle(TimerEvent::Tick { elapsed_ms: 1_000 })
            .unwrap();
        let TimerState::Focus(runtime) = engine.state() else {
            panic!("focus");
        };
        assert_eq!(runtime.remaining_ms, 25 * MINUTE_MS - 1_000);
    }

    #[test]
    fn tick_that_expires_the_phase_completes_once() {
        let mut engine = engine(Program::deep_cycle());
        engine.handle(TimerEvent::Start).unwrap();
        let effects = engine
            .handle(TimerEvent::Tick {
                elapsed_ms: 45 * MINUTE_MS + 5_000,
            })
            .unwrap();
        assert!(matches!(engine.state(), TimerState::ShortBreak(_)));
        let TimerState::ShortBreak(runtime) = engine.state() else {
            panic!("break");
        };
        assert_eq!(runtime.remaining_ms, 10 * MINUTE_MS);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            Effect::CloseSession {
                completed: true,
                ..
            }
        )));
    }

    #[test]
    fn pause_remembers_the_phase_and_remaining() {
        let mut engine = engine(Program::pomodoro());
        engine.handle(TimerEvent::Start).unwrap();
        engine
            .handle(TimerEvent::Tick { elapsed_ms: 5_000 })
            .unwrap();
        engine.handle(TimerEvent::Pause).unwrap();
        let TimerState::Paused(paused) = engine.state() else {
            panic!("paused");
        };
        assert_eq!(paused.kind, PhaseKind::Focus);
        assert_eq!(paused.runtime.remaining_ms, 25 * MINUTE_MS - 5_000);
        engine
            .handle(TimerEvent::Tick { elapsed_ms: 9_000 })
            .unwrap();
        assert!(matches!(engine.state(), TimerState::Paused(_)));
        engine.handle(TimerEvent::Resume).unwrap();
        let TimerState::Focus(runtime) = engine.state() else {
            panic!("resumed");
        };
        assert_eq!(runtime.remaining_ms, 25 * MINUTE_MS - 5_000);
    }

    #[test]
    fn skip_asks_the_program_for_the_next_phase() {
        let mut engine = engine(Program::deep_cycle());
        engine.handle(TimerEvent::Start).unwrap();
        let effects = engine.handle(TimerEvent::Skip).unwrap();
        assert!(matches!(engine.state(), TimerState::ShortBreak(_)));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            Effect::CloseSession {
                completed: false,
                ..
            }
        )));
    }

    #[test]
    fn cycle_loops_and_day_plan_returns_to_idle() {
        let mut cycle = engine(Program::deep_cycle());
        cycle.handle(TimerEvent::Start).unwrap();
        for _ in 0..4 {
            cycle.handle(TimerEvent::Complete).unwrap();
        }
        assert!(matches!(cycle.state(), TimerState::Focus(_)));

        let mut day = engine(Program::sample_day());
        day.handle(TimerEvent::Start).unwrap();
        let Program::DayPlan { phases } = Program::sample_day() else {
            panic!("day");
        };
        for _ in 0..phases.len() {
            day.handle(TimerEvent::Complete).unwrap();
        }
        assert!(matches!(day.state(), TimerState::Idle));
    }

    #[test]
    fn changing_program_while_running_is_rejected() {
        let mut engine = engine(Program::pomodoro());
        engine.handle(TimerEvent::Start).unwrap();
        let err = engine
            .set_program(Uuid::nil(), "other", Program::deep_cycle())
            .unwrap_err();
        assert_eq!(err, TimerError::Busy);
    }

    #[test]
    fn idle_ignores_ticks_and_rejects_pause() {
        let mut engine = engine(Program::pomodoro());
        engine
            .handle(TimerEvent::Tick { elapsed_ms: 1_000 })
            .unwrap();
        assert!(matches!(engine.state(), TimerState::Idle));
        assert!(engine.handle(TimerEvent::Pause).is_err());
    }

    #[test]
    fn format_remaining_uses_a_clock() {
        assert_eq!(format_remaining(0), "00:00");
        assert_eq!(format_remaining(1_000), "00:01");
        assert_eq!(format_remaining(45 * MINUTE_MS), "45:00");
        assert_eq!(format_remaining(3_600_000), "1:00:00");
    }
}
