use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use komodoro_core::{
    AppSettings, AppSnapshot, Effect, Engine, FocusEvent, FocusEventKind, Program, ProgramRecord,
    ProgramRepository, Session, SessionRepository, SettingsRepository, Task, TaskRepository,
    TimerEvent, TimerState, UiCommand, SETTING_NOTIFICATIONS,
};
use komodoro_storage::SqliteStore;
use tauri::{App, AppHandle, Manager};
use uuid::Uuid;

pub const SHORTCUT_LABEL: &str = "Ctrl+Shift+P";

pub struct Desktop {
    pub store: SqliteStore,
    pub engine: Mutex<Engine>,
    pub open_session: Mutex<Option<Uuid>>,
}

pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let store = SqliteStore::open(dir.join("komodoro.db"))?;
    store.seed_if_empty()?;
    let active = load_active(&store)?;
    let engine = Engine::new(active.id, active.name, active.program);
    app.manage(Desktop {
        store,
        engine: Mutex::new(engine),
        open_session: Mutex::new(None),
    });

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        let mut last = std::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last).as_millis() as u64;
            last = now;
            if elapsed > 0 {
                let _ = dispatch_timer(
                    &handle,
                    TimerEvent::Tick {
                        elapsed_ms: elapsed,
                    },
                );
            }
        }
    });
    Ok(())
}

pub fn snapshot(app: &AppHandle) -> Result<AppSnapshot, String> {
    current_snapshot(&app.state::<Desktop>())
}

pub fn dispatch(app: &AppHandle, command: UiCommand) -> Result<AppSnapshot, String> {
    dispatch_timer(app, command.into())
}

pub fn toggle(app: &AppHandle) -> Result<AppSnapshot, String> {
    let desktop = app.state::<Desktop>();
    let event = {
        let engine = lock(&desktop.engine)?;
        match engine.state() {
            TimerState::Idle => TimerEvent::Start,
            TimerState::Paused(_) => TimerEvent::Resume,
            TimerState::Focus(_) | TimerState::ShortBreak(_) | TimerState::LongBreak(_) => {
                TimerEvent::Pause
            }
        }
    };
    dispatch_timer(app, event)
}

pub fn list_programs(app: &AppHandle) -> Result<Vec<ProgramRecord>, String> {
    app.state::<Desktop>()
        .store
        .list_programs()
        .map_err(|err| err.to_string())
}

pub fn save_program(
    app: &AppHandle,
    name: String,
    source: String,
) -> Result<ProgramRecord, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name is required".into());
    }
    let program = Program::parse_source(&source).map_err(|err| err.to_string())?;
    let now = now_ms();
    let record = ProgramRecord {
        id: Uuid::new_v4(),
        name: name.to_string(),
        program,
        preset: false,
        created_at_ms: now,
        updated_at_ms: now,
    };
    app.state::<Desktop>()
        .store
        .insert_program(&record)
        .map_err(|err| err.to_string())?;
    Ok(record)
}

pub fn activate_program(app: &AppHandle, id: Uuid) -> Result<AppSnapshot, String> {
    let desktop = app.state::<Desktop>();
    let record = desktop
        .store
        .get_program(id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "program not found".to_string())?;
    {
        let mut engine = lock(&desktop.engine)?;
        engine
            .set_program(record.id, record.name, record.program)
            .map_err(|err| err.to_string())?;
    }
    desktop
        .store
        .set_active_program(id)
        .map_err(|err| err.to_string())?;
    current_snapshot(&desktop)
}

pub fn list_tasks(app: &AppHandle) -> Result<Vec<Task>, String> {
    app.state::<Desktop>()
        .store
        .list_tasks()
        .map_err(|err| err.to_string())
}

pub fn add_task(app: &AppHandle, title: String) -> Result<Task, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("title is required".into());
    }
    let desktop = app.state::<Desktop>();
    let session_id = *lock(&desktop.open_session)?;
    let task = Task {
        id: Uuid::new_v4(),
        title: title.to_string(),
        done: false,
        session_id,
        created_at_ms: now_ms(),
    };
    desktop
        .store
        .insert_task(&task)
        .map_err(|err| err.to_string())?;
    Ok(task)
}

pub fn set_task_done(app: &AppHandle, id: Uuid, done: bool) -> Result<(), String> {
    app.state::<Desktop>()
        .store
        .set_task_done(id, done)
        .map_err(|err| err.to_string())
}

pub fn list_sessions(app: &AppHandle) -> Result<Vec<Session>, String> {
    app.state::<Desktop>()
        .store
        .list_sessions(100)
        .map_err(|err| err.to_string())
}

pub fn app_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let desktop = app.state::<Desktop>();
    Ok(AppSettings {
        notifications: notifications_on(&desktop)?,
        shortcut: SHORTCUT_LABEL.to_string(),
    })
}

pub fn set_notifications(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let value = if enabled { "1" } else { "0" };
    app.state::<Desktop>()
        .store
        .set_setting(SETTING_NOTIFICATIONS, value)
        .map_err(|err| err.to_string())
}

fn dispatch_timer(app: &AppHandle, event: TimerEvent) -> Result<AppSnapshot, String> {
    let desktop = app.state::<Desktop>();
    let (program_id, effects) = {
        let mut engine = lock(&desktop.engine)?;
        let effects = engine.handle(event).map_err(|err| err.to_string())?;
        (engine.program_id(), effects)
    };
    apply_effects(app, &desktop, program_id, effects)?;
    current_snapshot(&desktop)
}

fn apply_effects(
    app: &AppHandle,
    desktop: &Desktop,
    program_id: Uuid,
    effects: Vec<Effect>,
) -> Result<(), String> {
    let now = now_ms();
    let notify = notifications_on(desktop)?;
    for effect in effects {
        match effect {
            Effect::OpenSession { phase, label } => {
                let id = Uuid::new_v4();
                desktop
                    .store
                    .insert_session(&Session {
                        id,
                        program_id,
                        phase,
                        label: label.clone(),
                        started_at_ms: now,
                        ended_at_ms: None,
                        completed: false,
                    })
                    .map_err(|err| err.to_string())?;
                desktop
                    .store
                    .insert_focus_event(&FocusEvent {
                        id: Uuid::new_v4(),
                        session_id: Some(id),
                        program_id,
                        phase,
                        label,
                        at_ms: now,
                        kind: FocusEventKind::Started,
                    })
                    .map_err(|err| err.to_string())?;
                *lock(&desktop.open_session)? = Some(id);
            }
            Effect::CloseSession {
                phase,
                label,
                completed,
            } => {
                let mut open = lock(&desktop.open_session)?;
                if let Some(id) = open.take() {
                    desktop
                        .store
                        .finish_session(id, now, completed)
                        .map_err(|err| err.to_string())?;
                    desktop
                        .store
                        .insert_focus_event(&FocusEvent {
                            id: Uuid::new_v4(),
                            session_id: Some(id),
                            program_id,
                            phase,
                            label,
                            at_ms: now,
                            kind: if completed {
                                FocusEventKind::Completed
                            } else {
                                FocusEventKind::Skipped
                            },
                        })
                        .map_err(|err| err.to_string())?;
                }
            }
            Effect::Notify { title, body } => {
                if notify {
                    notify_user(app, &title, &body);
                }
            }
        }
    }
    Ok(())
}

fn current_snapshot(desktop: &Desktop) -> Result<AppSnapshot, String> {
    let engine = lock(&desktop.engine)?;
    let open_session_id = *lock(&desktop.open_session)?;
    let ready = engine.ready_phase();
    Ok(AppSnapshot {
        timer: engine.state().clone(),
        program_id: engine.program_id(),
        program_name: engine.program_name().to_string(),
        open_session_id,
        ready_kind: ready.as_ref().map(|phase| phase.kind),
        ready_label: ready.as_ref().and_then(|phase| phase.label.clone()),
        ready_ms: ready.as_ref().map(|phase| phase.duration_ms),
    })
}

fn load_active(store: &SqliteStore) -> Result<ProgramRecord, StoreErrorString> {
    if let Some(id) = store.active_program_id()? {
        if let Some(record) = store.get_program(id)? {
            return Ok(record);
        }
    }
    store
        .list_programs()?
        .into_iter()
        .next()
        .ok_or_else(|| komodoro_core::StoreError::Message("no program in the database".into()))
}

fn notifications_on(desktop: &Desktop) -> Result<bool, String> {
    match desktop
        .store
        .get_setting(SETTING_NOTIFICATIONS)
        .map_err(|err| err.to_string())?
    {
        Some(value) => Ok(value != "0"),
        None => Ok(true),
    }
}

fn notify_user(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    mutex.lock().map_err(|err| err.to_string())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

type StoreErrorString = komodoro_core::StoreError;
