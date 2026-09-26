use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app::shell;
use crate::ipc;
use komodoro_core::{format_remaining, PhaseKind, TimerState, UiCommand};

#[component]
pub fn TimerView() -> impl IntoView {
    let shell = shell();

    let send = move |command: UiCommand| {
        spawn_local(async move {
            match ipc::dispatch(command).await {
                Ok(snapshot) => {
                    shell.snapshot.set(Some(snapshot));
                    shell.notice.set(String::new());
                }
                Err(err) => shell.notice.set(err),
            }
        });
    };

    let phase_class = move || {
        let kind = shell
            .snapshot
            .get()
            .and_then(|snapshot| displayed_kind(&snapshot));
        match kind {
            Some(PhaseKind::ShortBreak) => "phase break",
            Some(PhaseKind::LongBreak) => "phase long",
            _ => "phase focus",
        }
    };
    let phase_title = move || {
        shell
            .snapshot
            .get()
            .and_then(|snapshot| displayed_kind(&snapshot))
            .map(PhaseKind::title)
            .unwrap_or("Ready")
            .to_string()
    };
    let activity = move || {
        shell
            .snapshot
            .get()
            .and_then(|snapshot| displayed_label(&snapshot))
            .unwrap_or_default()
    };
    let program_name = move || {
        shell
            .snapshot
            .get()
            .map(|snapshot| snapshot.program_name)
            .unwrap_or_default()
    };
    let clock = move || {
        shell
            .snapshot
            .get()
            .map(|snapshot| format_remaining(displayed_ms(&snapshot)))
            .unwrap_or_else(|| "—".into())
    };
    let action = move || match shell.snapshot.get().map(|snapshot| snapshot.timer) {
        Some(TimerState::Paused(_)) => "Resume",
        Some(TimerState::Focus(_))
        | Some(TimerState::ShortBreak(_))
        | Some(TimerState::LongBreak(_)) => "Pause",
        _ => "Start",
    };
    let show_skip = move || {
        matches!(
            shell.snapshot.get().map(|snapshot| snapshot.timer),
            Some(TimerState::Focus(_))
                | Some(TimerState::ShortBreak(_))
                | Some(TimerState::LongBreak(_))
        )
    };

    view! {
        <header class="top">
            <p class=phase_class>{phase_title}</p>
            <p class="program">{program_name}</p>
        </header>
        <p class="activity">{activity}</p>
        <p class="clock">{clock}</p>
        <div class="actions">
            <button
                type="button"
                class="primary"
                on:click=move |_| {
                    let command = match shell.snapshot.get_untracked().map(|snapshot| snapshot.timer) {
                        Some(TimerState::Paused(_)) => UiCommand::Resume,
                        Some(TimerState::Focus(_))
                        | Some(TimerState::ShortBreak(_))
                        | Some(TimerState::LongBreak(_)) => UiCommand::Pause,
                        _ => UiCommand::Start,
                    };
                    send(command);
                }
            >
                {action}
            </button>
            <button
                type="button"
                class="ghost"
                class:hidden=move || !show_skip()
                on:click=move |_| send(UiCommand::Skip)
            >
                "Skip"
            </button>
        </div>
    }
}

fn displayed_kind(snapshot: &komodoro_core::AppSnapshot) -> Option<PhaseKind> {
    snapshot.timer.phase_kind().or(snapshot.ready_kind)
}

fn displayed_label(snapshot: &komodoro_core::AppSnapshot) -> Option<String> {
    snapshot
        .timer
        .label()
        .map(str::to_string)
        .or_else(|| snapshot.ready_label.clone())
}

fn displayed_ms(snapshot: &komodoro_core::AppSnapshot) -> u64 {
    snapshot
        .timer
        .remaining_ms()
        .or(snapshot.ready_ms)
        .unwrap_or(0)
}
