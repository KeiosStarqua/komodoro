# crates/komodoro-core

## Purpose

Pure focus engine: program interpreter, timer state machine, session summaries, and the persistence ports the shell implements.

## Ownership

This crate owns phase order, durations, timer transitions, and history aggregation. It does not own the clock, the database, or the pixels.

## Local Contracts

- Do not import Tauri, Leptos, rusqlite, or any other framework.
- Time enters only as `TimerEvent::Tick { elapsed_ms }`. The UI must not count down.
- Pomodoro is `Program::pomodoro()`, a rules program. Do not special-case 25/5 inside the state machine.
- `Skip` and `Complete` both ask the program for the next phase. A tick that reaches zero is `Complete`.
- Leftover milliseconds on an oversized tick are dropped. They are not applied to the following phase.
- Cycles and rules loop. A day plan runs once, then the timer returns to `Idle`.
- Repository traits in `ports` are the only persistence API. Errors are `StoreError`, not a driver error.
- `SessionSummary::since` aggregates a session list behind a caller-supplied boundary. Focused time counts completed focus sessions only; skipped and interrupted sessions add none.

## Work Guidance

- Add a program form by extending `Program` and `Program::parse_source`, then cover it with a table test.
- YAML samples live in `docs/examples/` and are loaded with `include_str!` from the parser tests.
- Activity names are `PhaseSpec.label`. They are not new `TimerState` variants.

## Verification

`cargo test -p komodoro-core`

## Child DOX Index

| Path | Scope |
|------|-------|
| (none) | |
