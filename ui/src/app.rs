use gloo_timers::callback::Interval;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ipc;
use crate::settings_view::SettingsView;
use crate::stats_view::StatsView;
use crate::tasks_view::TasksView;
use crate::timer_view::TimerView;
use komodoro_core::AppSnapshot;

#[derive(Clone, Copy)]
pub struct Shell {
    pub snapshot: RwSignal<Option<AppSnapshot>>,
    pub notice: RwSignal<String>,
}

pub fn shell() -> Shell {
    expect_context()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Timer,
    Tasks,
    Stats,
    Settings,
}

#[component]
pub fn App() -> impl IntoView {
    let snapshot = RwSignal::new(None::<AppSnapshot>);
    let notice = RwSignal::new(String::new());
    let connected = RwSignal::new(true);
    let tab = RwSignal::new(Tab::Timer);
    provide_context(Shell { snapshot, notice });

    let poll = move || {
        if !connected.get_untracked() {
            return;
        }
        spawn_local(async move {
            match ipc::snapshot().await {
                Ok(value) => {
                    snapshot.set(Some(value));
                    notice.set(String::new());
                }
                Err(err) => {
                    connected.set(false);
                    if snapshot.get_untracked().is_none() {
                        notice.set(err);
                    }
                }
            }
        });
    };
    poll();
    std::mem::forget(Interval::new(1_000, poll));

    view! {
        <div class="app">
            <nav class="rail">
                <p class="brand">"Komodoro"</p>
                <TabButton tab=Tab::Timer current=tab label="Timer"/>
                <TabButton tab=Tab::Tasks current=tab label="Tasks"/>
                <TabButton tab=Tab::Stats current=tab label="Stats"/>
                <TabButton tab=Tab::Settings current=tab label="Settings"/>
            </nav>
            <main class="stage">
                {move || {
                    let text = notice.get();
                    (!text.is_empty()).then(|| view! { <p class="notice">{text}</p> })
                }}
                {move || match tab.get() {
                    Tab::Timer => view! { <TimerView/> }.into_any(),
                    Tab::Tasks => view! { <TasksView/> }.into_any(),
                    Tab::Stats => view! { <StatsView/> }.into_any(),
                    Tab::Settings => view! { <SettingsView/> }.into_any(),
                }}
            </main>
        </div>
    }
}

#[component]
fn TabButton(tab: Tab, current: RwSignal<Tab>, label: &'static str) -> impl IntoView {
    view! {
        <button
            type="button"
            attr:aria-current=move || (current.get() == tab).then_some("page")
            on:click=move |_| current.set(tab)
        >
            {label}
        </button>
    }
}
