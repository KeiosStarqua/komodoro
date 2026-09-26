use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app::shell;
use crate::ipc;
use komodoro_core::ProgramRecord;

const PLACEHOLDER: &str =
    "cycle:\n  - type: focus\n    duration: 45m\n  - type: short_break\n    duration: 10m\n";

#[component]
pub fn SettingsView() -> impl IntoView {
    let shell = shell();
    let programs = RwSignal::new(Vec::<ProgramRecord>::new());
    let shortcut = RwSignal::new(String::new());
    let notifications = RwSignal::new(true);
    let name = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());

    let reload = move || {
        spawn_local(async move {
            match ipc::programs().await {
                Ok(list) => programs.set(list),
                Err(err) => shell.notice.set(err),
            }
            if let Ok(settings) = ipc::settings().await {
                shortcut.set(settings.shortcut);
                notifications.set(settings.notifications);
            }
        });
    };
    reload();

    view! {
        <header class="top">
            <h1>"Settings"</h1>
        </header>
        <div class="row">
            <span>"Shortcut"</span>
            <span class="meta">{move || shortcut.get()}</span>
        </div>
        <label class="row">
            <span>"Notifications"</span>
            <input
                type="checkbox"
                prop:checked=move || notifications.get()
                on:change=move |event| {
                    let enabled = event_target_checked(&event);
                    notifications.set(enabled);
                    spawn_local(async move {
                        if let Err(err) = ipc::set_notifications(enabled).await {
                            shell.notice.set(err);
                        }
                    });
                }
            />
        </label>
        <h2>"Programs"</h2>
        <ul class="rows">
            {move || {
                let active = shell.snapshot.get().map(|snapshot| snapshot.program_id);
                programs
                    .get()
                    .into_iter()
                    .map(|program| {
                        let id = program.id;
                        let is_active = active == Some(id);
                        let label = program.name.clone();
                        view! {
                            <li class="row">
                                <span>{label}</span>
                                {is_active.then(|| view! { <span class="meta">"Active"</span> })}
                                {(!is_active).then(|| view! {
                                    <button
                                        type="button"
                                        class="ghost"
                                        on:click=move |_| {
                                            spawn_local(async move {
                                                match ipc::activate_program(id).await {
                                                    Ok(snapshot) => {
                                                        shell.snapshot.set(Some(snapshot));
                                                        shell.notice.set(String::new());
                                                    }
                                                    Err(err) => shell.notice.set(err),
                                                }
                                            });
                                        }
                                    >
                                        "Use"
                                    </button>
                                })}
                            </li>
                        }
                    })
                    .collect_view()
            }}
        </ul>
        <form
            class="editor"
            on:submit=move |event| {
                event.prevent_default();
                let program_name = name.get_untracked();
                let body = source.get_untracked();
                spawn_local(async move {
                    match ipc::save_program(program_name.trim(), &body).await {
                        Ok(_) => {
                            name.set(String::new());
                            source.set(String::new());
                            shell.notice.set(String::new());
                            reload();
                        }
                        Err(err) => shell.notice.set(err),
                    }
                });
            }
        >
            <label>
                "Name"
                <input
                    type="text"
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                />
            </label>
            <label>
                "Program"
                <textarea
                    placeholder=PLACEHOLDER
                    prop:value=move || source.get()
                    on:input=move |event| source.set(event_target_value(&event))
                ></textarea>
            </label>
            <button type="submit" class="primary">"Save program"</button>
        </form>
    }
}
