mod app;
mod ipc;
mod settings_view;
mod stats_view;
mod tasks_view;
mod timer_view;

use app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> });
}
