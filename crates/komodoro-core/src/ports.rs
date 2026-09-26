use uuid::Uuid;

use crate::model::{FocusEvent, ProgramRecord, Session, StoreError, Task};

pub trait ProgramRepository {
    fn list_programs(&self) -> Result<Vec<ProgramRecord>, StoreError>;
    fn get_program(&self, id: Uuid) -> Result<Option<ProgramRecord>, StoreError>;
    fn insert_program(&self, record: &ProgramRecord) -> Result<(), StoreError>;
    fn active_program_id(&self) -> Result<Option<Uuid>, StoreError>;
    fn set_active_program(&self, id: Uuid) -> Result<(), StoreError>;
}

pub trait SessionRepository {
    fn insert_session(&self, session: &Session) -> Result<(), StoreError>;
    fn finish_session(&self, id: Uuid, ended_at_ms: i64, completed: bool)
        -> Result<(), StoreError>;
    fn list_sessions(&self, limit: u32) -> Result<Vec<Session>, StoreError>;
    fn insert_focus_event(&self, event: &FocusEvent) -> Result<(), StoreError>;
}

pub trait TaskRepository {
    fn list_tasks(&self) -> Result<Vec<Task>, StoreError>;
    fn insert_task(&self, task: &Task) -> Result<(), StoreError>;
    fn set_task_done(&self, id: Uuid, done: bool) -> Result<(), StoreError>;
}

pub trait SettingsRepository {
    fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError>;
}
