use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app::shell;
use crate::ipc;
use komodoro_core::{format_remaining, Session};

#[component]
pub fn StatsView() -> impl IntoView {
    let shell = shell();
    let sessions = RwSignal::new(Vec::<Session>::new());

    spawn_local(async move {
        match ipc::sessions().await {
            Ok(list) => sessions.set(list),
            Err(err) => shell.notice.set(err),
        }
    });

    view! {
        <header class="top">
            <h1>"Stats"</h1>
        </header>
        <ul class="rows">
            {move || {
                let list = sessions.get();
                if list.is_empty() {
                    return view! { <li class="empty">"No focus history yet."</li> }.into_any();
                }
                list.into_iter()
                    .map(|session| {
                        let title = session_title(&session);
                        let meta = session_meta(&session);
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

fn session_meta(session: &Session) -> String {
    match session.ended_at_ms {
        Some(end) if session.completed => {
            let elapsed = end.saturating_sub(session.started_at_ms).max(0) as u64;
            format_remaining(elapsed)
        }
        Some(_) => "skipped".into(),
        None => "in progress".into(),
    }
}
