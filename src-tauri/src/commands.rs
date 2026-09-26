use komodoro_core::{AppSettings, AppSnapshot, ProgramRecord, Session, Task, UiCommand};
use tauri::AppHandle;
use uuid::Uuid;

use crate::runtime;

#[tauri::command]
pub fn snapshot(app: AppHandle) -> Result<AppSnapshot, String> {
    runtime::snapshot(&app)
}

#[tauri::command]
pub fn dispatch(app: AppHandle, command: UiCommand) -> Result<AppSnapshot, String> {
    runtime::dispatch(&app, command)
}

#[tauri::command]
pub fn list_programs(app: AppHandle) -> Result<Vec<ProgramRecord>, String> {
    runtime::list_programs(&app)
}

#[tauri::command]
pub fn save_program(app: AppHandle, name: String, source: String) -> Result<ProgramRecord, String> {
    runtime::save_program(&app, name, source)
}

#[tauri::command]
pub fn activate_program(app: AppHandle, id: Uuid) -> Result<AppSnapshot, String> {
    runtime::activate_program(&app, id)
}

#[tauri::command]
pub fn list_tasks(app: AppHandle) -> Result<Vec<Task>, String> {
    runtime::list_tasks(&app)
}

#[tauri::command]
pub fn add_task(app: AppHandle, title: String) -> Result<Task, String> {
    runtime::add_task(&app, title)
}

#[tauri::command]
pub fn set_task_done(app: AppHandle, id: Uuid, done: bool) -> Result<(), String> {
    runtime::set_task_done(&app, id, done)
}

#[tauri::command]
pub fn list_sessions(app: AppHandle) -> Result<Vec<Session>, String> {
    runtime::list_sessions(&app)
}

#[tauri::command]
pub fn app_settings(app: AppHandle) -> Result<AppSettings, String> {
    runtime::app_settings(&app)
}

#[tauri::command]
pub fn set_notifications(app: AppHandle, enabled: bool) -> Result<(), String> {
    runtime::set_notifications(&app, enabled)
}
