use js_sys::Date;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app::shell;
use crate::ipc;
use komodoro_core::{format_remaining, Session, SessionSummary};
use uuid::Uuid;

#[component]
pub fn StatsView() -> impl IntoView {
    let shell = shell();
    let sessions = RwSignal::new(Vec::<Session>::new());
    let open_session = RwSignal::new(None::<Uuid>);

    spawn_local(async move {
        match ipc::sessions().await {
            Ok(list) => {
                let open = shell
                    .snapshot
                    .get_untracked()
                    .and_then(|snapshot| snapshot.open_session_id);
                open_session.set(open);
                sessions.set(list);
            }
            Err(err) => shell.notice.set(err),
        }
    });

    let today = move || SessionSummary::since(&sessions.get(), start_of_local_day_ms());

    view! {
        <header class="top">
            <h1>"Stats"</h1>
        </header>
        <p class="summary">
            <span class="figure">{move || format_remaining(today().focus_ms)}</span>
            {move || {
                let summary = today();
                format!(
                    " focused today · {} sessions · {} skipped",
                    summary.focus_completed, summary.focus_skipped
                )
            }}
        </p>
        <ul class="rows">
            {move || {
                let list = sessions.get();
                if list.is_empty() {
                    return view! { <li class="empty">"No focus history yet."</li> }.into_any();
                }
                let open = open_session.get();
                list.into_iter()
                    .map(|session| {
                        let title = session_title(&session);
                        let meta = session_meta(&session, open);
                        view! {
                            <li class="row">
                                <span>{title}</span>
                                <span class="meta">{meta}</span>
                            </li>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </ul>
    }
}

fn session_title(session: &Session) -> String {
    match &session.label {
        Some(label) => format!("{} · {label}", session.phase.title()),
        None => session.phase.title().to_string(),
    }
}

fn session_meta(session: &Session, open_session: Option<Uuid>) -> String {
    match session.ended_at_ms {
        Some(end) if session.completed => {
            let elapsed = end.saturating_sub(session.started_at_ms).max(0) as u64;
            format_remaining(elapsed)
        }
        Some(_) => "skipped".into(),
        None if open_session == Some(session.id) => "in progress".into(),
        None => "interrupted".into(),
    }
}

/// Midnight of the current local day, in Unix milliseconds.
fn start_of_local_day_ms() -> i64 {
    const DAY_MS: f64 = 86_400_000.0;
    let now = Date::new_0();
    let offset_ms = now.get_timezone_offset() as f64 * 60_000.0;
    let local_ms = now.get_time() - offset_ms;
    ((local_ms / DAY_MS).floor() * DAY_MS + offset_ms) as i64
}
