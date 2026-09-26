use komodoro_core::{AppSettings, AppSnapshot, ProgramRecord, Session, Task, UiCommand};
use serde::de::DeserializeOwned;
use serde_json::Value;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn snapshot() -> Result<AppSnapshot, String> {
    call("snapshot", &serde_json::json!({})).await
}

pub async fn dispatch(command: UiCommand) -> Result<AppSnapshot, String> {
    call("dispatch", &serde_json::json!({ "command": command })).await
}

pub async fn programs() -> Result<Vec<ProgramRecord>, String> {
    call("list_programs", &serde_json::json!({})).await
}

pub async fn save_program(name: &str, source: &str) -> Result<ProgramRecord, String> {
    call(
        "save_program",
        &serde_json::json!({ "name": name, "source": source }),
    )
    .await
}

pub async fn activate_program(id: Uuid) -> Result<AppSnapshot, String> {
    call("activate_program", &serde_json::json!({ "id": id })).await
}

pub async fn tasks() -> Result<Vec<Task>, String> {
    call("list_tasks", &serde_json::json!({})).await
}

pub async fn add_task(title: &str) -> Result<Task, String> {
    call("add_task", &serde_json::json!({ "title": title })).await
}

pub async fn set_task_done(id: Uuid, done: bool) -> Result<(), String> {
    call(
        "set_task_done",
        &serde_json::json!({ "id": id, "done": done }),
    )
    .await
}

pub async fn sessions() -> Result<Vec<Session>, String> {
    call("list_sessions", &serde_json::json!({})).await
}

pub async fn settings() -> Result<AppSettings, String> {
    call("app_settings", &serde_json::json!({})).await
}

pub async fn set_notifications(enabled: bool) -> Result<(), String> {
    call(
        "set_notifications",
        &serde_json::json!({ "enabled": enabled }),
    )
    .await
}

/// Bridge through JSON text: `invoke` receives a plain JS object, and the reply
/// is parsed by `serde_json`, so 64-bit fields round-trip like they do in Rust.
async fn call<T: DeserializeOwned>(cmd: &str, args: &Value) -> Result<T, String> {
    let args = js_sys::JSON::parse(&args.to_string()).map_err(|err| format!("{err:?}"))?;
    let reply = invoke(cmd, args).await.map_err(js_error)?;
    let text = js_sys::JSON::stringify(&reply)
        .map_err(|err| format!("{err:?}"))?
        .as_string()
        .ok_or_else(|| format!("{cmd} returned a value JSON cannot express"))?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn js_error(value: JsValue) -> String {
    if let Some(text) = value.as_string() {
        return text;
    }
    let debug = format!("{value:?}");
    if debug.contains("undefined") || debug.contains("__TAURI__") {
        "Open Komodoro as the desktop app. The timer does not run in a plain browser.".into()
    } else {
        debug
    }
}
