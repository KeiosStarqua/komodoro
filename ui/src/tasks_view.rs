use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::app::shell;
use crate::ipc;
use komodoro_core::Task;

#[component]
pub fn TasksView() -> impl IntoView {
    let shell = shell();
    let tasks = RwSignal::new(Vec::<Task>::new());
    let draft = RwSignal::new(String::new());

    let reload = move || {
        spawn_local(async move {
            match ipc::tasks().await {
                Ok(list) => tasks.set(list),
                Err(err) => shell.notice.set(err),
            }
        });
    };
    reload();

    view! {
        <header class="top">
            <h1>"Tasks"</h1>
        </header>
        <form
            class="composer"
            on:submit=move |event| {
                event.prevent_default();
                let title = draft.get_untracked();
                if title.trim().is_empty() {
                    return;
                }
                spawn_local(async move {
                    match ipc::add_task(title.trim()).await {
                        Ok(_) => {
                            draft.set(String::new());
                            reload();
                        }
                        Err(err) => shell.notice.set(err),
                    }
                });
            }
        >
            <input
                type="text"
                placeholder="What are you focusing on?"
                prop:value=move || draft.get()
                on:input=move |event| draft.set(event_target_value(&event))
            />
            <button type="submit">"Add"</button>
        </form>
        <ul class="rows">
            {move || {
                tasks
                    .get()
                    .into_iter()
                    .map(|task| {
                        let id = task.id;
                        let done = task.done;
                        let title = task.title.clone();
                        view! {
                            <li class="row" class:done=done>
                                <input
                                    type="checkbox"
                                    prop:checked=done
                                    on:change=move |event| {
                                        let checked = event_target_checked(&event);
                                        spawn_local(async move {
                                            if let Err(err) = ipc::set_task_done(id, checked).await {
                                                shell.notice.set(err);
                                            }
                                            reload();
                                        });
                                    }
                                />
                                <span>{title}</span>
                            </li>
                        }
                    })
                    .collect_view()
            }}
        </ul>
    }
}
