use serde::{Deserialize, Serialize};

pub const MINUTE_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    Focus,
    ShortBreak,
    LongBreak,
}

impl PhaseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::ShortBreak => "short_break",
            Self::LongBreak => "long_break",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Focus => "Focus",
            Self::ShortBreak => "Short break",
            Self::LongBreak => "Long break",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "focus" => Some(Self::Focus),
            "short_break" => Some(Self::ShortBreak),
            "long_break" => Some(Self::LongBreak),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseSpec {
    pub kind: PhaseKind,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Where the interpreter is inside a program.
///
/// `phase_index` addresses a cycle or a flattened day plan.
/// Rules use `focus_sessions_completed` and `rules_on_break` instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProgramCursor {
    pub phase_index: usize,
    pub focus_sessions_completed: u32,
    pub rules_on_break: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Program {
    Cycle {
        phases: Vec<PhaseSpec>,
    },
    Rules {
        focus_ms: u64,
        short_break_ms: u64,
        long_break_ms: u64,
        sessions_before_long_break: u32,
    },
    DayPlan {
        phases: Vec<PhaseSpec>,
    },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProgramError {
    #[error("could not parse program: {0}")]
    Parse(String),
    #[error("duration must look like 45m, 10m, 30s, or 1h")]
    Duration,
    #[error("phase duration must be greater than zero")]
    ZeroDuration,
    #[error("rules need at least one focus session before a long break")]
    Sessions,
    #[error("unsupported rules action {0}")]
    Action(String),
    #[error("program has no phases")]
    Empty,
    #[error("day plan activity is empty")]
    Activity,
}

impl Program {
    pub fn pomodoro() -> Self {
        Self::Rules {
            focus_ms: 25 * MINUTE_MS,
            short_break_ms: 5 * MINUTE_MS,
            long_break_ms: 15 * MINUTE_MS,
            sessions_before_long_break: 4,
        }
    }

    pub fn deep_cycle() -> Self {
        Self::Cycle {
            phases: vec![
                phase(PhaseKind::Focus, 45, None),
                phase(PhaseKind::ShortBreak, 10, None),
                phase(PhaseKind::Focus, 45, None),
                phase(PhaseKind::LongBreak, 30, None),
            ],
        }
    }

    pub fn sample_day() -> Self {
        Self::DayPlan {
            phases: vec![
                labeled(PhaseKind::Focus, 45, "Deep Work"),
                labeled(PhaseKind::Focus, 45, "Deep Work"),
                phase(PhaseKind::ShortBreak, 10, None),
                labeled(PhaseKind::Focus, 45, "Reading"),
                labeled(PhaseKind::Focus, 45, "Coding"),
                labeled(PhaseKind::Focus, 45, "Coding"),
                labeled(PhaseKind::Focus, 45, "Coding"),
                labeled(PhaseKind::Focus, 45, "Review"),
            ],
        }
    }

    pub fn parse_source(source: &str) -> Result<Self, ProgramError> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return Err(ProgramError::Parse("empty source".into()));
        }
        if trimmed.starts_with('{') {
            return serde_json::from_str(trimmed)
                .map_err(|err| ProgramError::Parse(err.to_string()));
        }
        let file: ProgramFile =
            serde_yaml::from_str(trimmed).map_err(|err| ProgramError::Parse(err.to_string()))?;
        Self::from_file(file)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Cycle { phases } | Self::DayPlan { phases } => phases.is_empty(),
            Self::Rules { .. } => false,
        }
    }

    pub fn current_phase(&self, cursor: &ProgramCursor) -> Option<PhaseSpec> {
        match self {
            Self::Cycle { phases } => {
                if phases.is_empty() {
                    None
                } else {
                    Some(phases[cursor.phase_index % phases.len()].clone())
                }
            }
            Self::DayPlan { phases } => phases.get(cursor.phase_index).cloned(),
            Self::Rules {
                focus_ms,
                short_break_ms,
                long_break_ms,
                sessions_before_long_break,
            } => {
                if cursor.rules_on_break {
                    let long = *sessions_before_long_break > 0
                        && cursor
                            .focus_sessions_completed
                            .is_multiple_of(*sessions_before_long_break);
                    Some(if long {
                        phase(PhaseKind::LongBreak, 0, None).with_ms(*long_break_ms)
                    } else {
                        phase(PhaseKind::ShortBreak, 0, None).with_ms(*short_break_ms)
                    })
                } else {
                    Some(phase(PhaseKind::Focus, 0, None).with_ms(*focus_ms))
                }
            }
        }
    }

    /// Next cursor after `finished` ends. `None` means the day plan is over.
    /// Cycles and rules keep going.
    pub fn advance(&self, cursor: &ProgramCursor, finished: PhaseKind) -> Option<ProgramCursor> {
        match self {
            Self::Cycle { phases } => {
                if phases.is_empty() {
                    return None;
                }
                let mut next = cursor.clone();
                next.phase_index = cursor.phase_index.saturating_add(1);
                Some(next)
            }
            Self::DayPlan { phases } => {
                let next_index = cursor.phase_index.saturating_add(1);
                if next_index >= phases.len() {
                    None
                } else {
                    let mut next = cursor.clone();
                    next.phase_index = next_index;
                    Some(next)
                }
            }
            Self::Rules { .. } => {
                let mut next = cursor.clone();
                match finished {
                    PhaseKind::Focus => {
                        next.focus_sessions_completed =
                            cursor.focus_sessions_completed.saturating_add(1);
                        next.rules_on_break = true;
                    }
                    PhaseKind::ShortBreak | PhaseKind::LongBreak => {
                        next.rules_on_break = false;
                    }
                }
                Some(next)
            }
        }
    }

    fn from_file(file: ProgramFile) -> Result<Self, ProgramError> {
        match (file.cycle, file.rules, file.day_plan) {
            (Some(items), None, None) => {
                let phases = items
                    .into_iter()
                    .map(CyclePhaseFile::into_spec)
                    .collect::<Result<Vec<_>, _>>()?;
                if phases.is_empty() {
                    return Err(ProgramError::Empty);
                }
                Ok(Self::Cycle { phases })
            }
            (None, Some(rules), None) => rules.into_program(),
            (None, None, Some(plan)) => plan.into_program(),
            _ => Err(ProgramError::Parse(
                "program needs exactly one of cycle, rules, or day_plan".into(),
            )),
        }
    }
}

impl PhaseSpec {
    fn with_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

fn phase(kind: PhaseKind, minutes: u64, label: Option<&str>) -> PhaseSpec {
    PhaseSpec {
        kind,
        duration_ms: minutes.saturating_mul(MINUTE_MS),
        label: label.map(str::to_string),
    }
}

fn labeled(kind: PhaseKind, minutes: u64, label: &str) -> PhaseSpec {
    phase(kind, minutes, Some(label))
}

pub fn parse_duration_ms(input: &str) -> Result<u64, ProgramError> {
    let text = input.trim();
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .ok_or(ProgramError::Duration)?;
    if split == 0 {
        return Err(ProgramError::Duration);
    }
    let (amount, unit) = text.split_at(split);
    let amount: u64 = amount.parse().map_err(|_| ProgramError::Duration)?;
    let ms = match unit.trim() {
        "ms" => amount,
        "s" => amount.saturating_mul(1_000),
        "m" => amount.saturating_mul(MINUTE_MS),
        "h" => amount.saturating_mul(3_600_000),
        _ => return Err(ProgramError::Duration),
    };
    if ms == 0 {
        Err(ProgramError::ZeroDuration)
    } else {
        Ok(ms)
    }
}

fn de_ms<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    parse_duration_ms(&text).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
struct ProgramFile {
    cycle: Option<Vec<CyclePhaseFile>>,
    rules: Option<RulesFile>,
    day_plan: Option<DayPlanFile>,
}

#[derive(Debug, Deserialize)]
struct CyclePhaseFile {
    #[serde(rename = "type")]
    kind: PhaseKind,
    #[serde(deserialize_with = "de_ms")]
    duration: u64,
    #[serde(default)]
    label: Option<String>,
}

impl CyclePhaseFile {
    fn into_spec(self) -> Result<PhaseSpec, ProgramError> {
        Ok(PhaseSpec {
            kind: self.kind,
            duration_ms: nonzero(self.duration)?,
            label: blank_to_none(self.label),
        })
    }
}

#[derive(Debug, Deserialize)]
struct RulesFile {
    focus: SpanFile,
    #[serde(rename = "break")]
    short_break: SpanFile,
    #[serde(default)]
    long_break: Option<SpanFile>,
    after: AfterFile,
}

#[derive(Debug, Deserialize)]
struct SpanFile {
    #[serde(deserialize_with = "de_ms")]
    duration: u64,
}

#[derive(Debug, Deserialize)]
struct AfterFile {
    sessions: u32,
    action: String,
}

impl RulesFile {
    fn into_program(self) -> Result<Program, ProgramError> {
        if self.after.action != "long_break" {
            return Err(ProgramError::Action(self.after.action));
        }
        if self.after.sessions == 0 {
            return Err(ProgramError::Sessions);
        }
        let long_break_ms = match self.long_break {
            Some(span) => span.duration,
            None => 15 * MINUTE_MS,
        };
        Ok(Program::Rules {
            focus_ms: nonzero(self.focus.duration)?,
            short_break_ms: nonzero(self.short_break.duration)?,
            long_break_ms: nonzero(long_break_ms)?,
            sessions_before_long_break: self.after.sessions,
        })
    }
}

#[derive(Debug, Deserialize)]
struct DayPlanFile {
    #[serde(default)]
    focus_duration: Option<String>,
    #[serde(default)]
    break_duration: Option<String>,
    blocks: Vec<BlockFile>,
}

#[derive(Debug, Deserialize)]
struct BlockFile {
    #[allow(dead_code)]
    name: String,
    items: Vec<ItemFile>,
}

#[derive(Debug, Deserialize)]
struct ItemFile {
    activity: String,
    #[serde(default = "one")]
    repeats: u32,
    #[serde(default)]
    duration: Option<String>,
}

fn one() -> u32 {
    1
}

impl DayPlanFile {
    fn into_program(self) -> Result<Program, ProgramError> {
        let focus_ms = match self.focus_duration {
            Some(value) => parse_duration_ms(&value)?,
            None => 45 * MINUTE_MS,
        };
        let break_ms = match self.break_duration {
            Some(value) => parse_duration_ms(&value)?,
            None => 10 * MINUTE_MS,
        };
        let mut phases = Vec::new();
        for block in self.blocks {
            for item in block.items {
                let activity = item.activity.trim();
                if activity.is_empty() {
                    return Err(ProgramError::Activity);
                }
                let kind = activity_kind(activity);
                let duration_ms = match item.duration {
                    Some(value) => parse_duration_ms(&value)?,
                    None => match kind {
                        PhaseKind::Focus => focus_ms,
                        PhaseKind::ShortBreak | PhaseKind::LongBreak => break_ms,
                    },
                };
                let label = match kind {
                    PhaseKind::Focus => Some(activity.to_string()),
                    PhaseKind::ShortBreak | PhaseKind::LongBreak => None,
                };
                for _ in 0..item.repeats {
                    phases.push(PhaseSpec {
                        kind,
                        duration_ms,
                        label: label.clone(),
                    });
                }
            }
        }
        if phases.is_empty() {
            return Err(ProgramError::Empty);
        }
        Ok(Program::DayPlan { phases })
    }
}

fn activity_kind(activity: &str) -> PhaseKind {
    match activity.to_ascii_lowercase().as_str() {
        "break" | "short break" | "short_break" => PhaseKind::ShortBreak,
        "long break" | "long_break" => PhaseKind::LongBreak,
        _ => PhaseKind::Focus,
    }
}

fn nonzero(duration_ms: u64) -> Result<u64, ProgramError> {
    if duration_ms == 0 {
        Err(ProgramError::ZeroDuration)
    } else {
        Ok(duration_ms)
    }
}

fn blank_to_none(label: Option<String>) -> Option<String> {
    label.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_product_cycle() {
        let program = Program::parse_source(include_str!("../../../docs/examples/cycle.yaml"))
            .expect("cycle");
        assert_eq!(program, Program::deep_cycle());
    }

    #[test]
    fn parses_product_rules_with_default_long_break() {
        let program = Program::parse_source(include_str!("../../../docs/examples/rules.yaml"))
            .expect("rules");
        assert_eq!(
            program,
            Program::Rules {
                focus_ms: 50 * MINUTE_MS,
                short_break_ms: 10 * MINUTE_MS,
                long_break_ms: 15 * MINUTE_MS,
                sessions_before_long_break: 3,
            }
        );
    }

    #[test]
    fn parses_day_plan_labels_as_focus() {
        let program =
            Program::parse_source(include_str!("../../../docs/examples/day.yaml")).expect("day");
        assert_eq!(program, Program::sample_day());
    }

    #[test]
    fn pomodoro_is_a_rules_program() {
        assert_eq!(
            Program::pomodoro(),
            Program::Rules {
                focus_ms: 25 * MINUTE_MS,
                short_break_ms: 5 * MINUTE_MS,
                long_break_ms: 15 * MINUTE_MS,
                sessions_before_long_break: 4,
            }
        );
    }

    #[test]
    fn rules_long_break_lands_after_n_focus_sessions() {
        let program = Program::parse_source(include_str!("../../../docs/examples/rules.yaml"))
            .expect("rules");
        let mut cursor = ProgramCursor::default();
        let mut kinds = Vec::new();
        for _ in 0..8 {
            let phase = program.current_phase(&cursor).expect("phase");
            kinds.push(phase.kind);
            cursor = program
                .advance(&cursor, phase.kind)
                .expect("rules continue");
        }
        assert_eq!(
            kinds,
            vec![
                PhaseKind::Focus,
                PhaseKind::ShortBreak,
                PhaseKind::Focus,
                PhaseKind::ShortBreak,
                PhaseKind::Focus,
                PhaseKind::LongBreak,
                PhaseKind::Focus,
                PhaseKind::ShortBreak,
            ]
        );
    }

    #[test]
    fn day_plan_stops_at_the_end() {
        let program = Program::sample_day();
        let Program::DayPlan { phases } = &program else {
            panic!("day plan");
        };
        let mut cursor = ProgramCursor::default();
        for phase in phases.iter().take(phases.len() - 1) {
            assert_eq!(program.current_phase(&cursor).unwrap().kind, phase.kind);
            cursor = program.advance(&cursor, phase.kind).expect("next");
        }
        assert!(program
            .advance(&cursor, phases[phases.len() - 1].kind)
            .is_none());
    }
}
