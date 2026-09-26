mod commands;
mod runtime;
mod shell;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if shell::is_pressed(event.state) {
                        let _ = runtime::toggle(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            runtime::init(app)?;
            shell::install(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::dispatch,
            commands::list_programs,
            commands::save_program,
            commands::activate_program,
            commands::list_tasks,
            commands::add_task,
            commands::set_task_done,
            commands::list_sessions,
            commands::app_settings,
            commands::set_notifications,
        ])
        .run(tauri::generate_context!())
        .expect("komodoro failed to start");
}
