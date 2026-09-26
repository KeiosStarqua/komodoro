use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use komodoro_core::{
    FocusEvent, PhaseKind, Program, ProgramRecord, ProgramRepository, Session, SessionRepository,
    SettingsRepository, StoreError, Task, TaskRepository, SETTING_ACTIVE_PROGRAM,
};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

const MIGRATION_1: &str = "
CREATE TABLE programs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    body TEXT NOT NULL,
    preset INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    program_id TEXT NOT NULL REFERENCES programs(id),
    phase TEXT NOT NULL,
    label TEXT,
    started_at_ms INTEGER NOT NULL,
    ended_at_ms INTEGER,
    completed INTEGER NOT NULL
);
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    done INTEGER NOT NULL,
    session_id TEXT REFERENCES sessions(id),
    created_at_ms INTEGER NOT NULL
);
CREATE TABLE focus_events (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id),
    program_id TEXT NOT NULL REFERENCES programs(id),
    phase TEXT NOT NULL,
    label TEXT,
    at_ms INTEGER NOT NULL,
    kind TEXT NOT NULL
);
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE INDEX sessions_started ON sessions (started_at_ms);
";

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let conn = db(Connection::open(path))?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = db(Connection::open_in_memory())?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn seed_if_empty(&self) -> Result<(), StoreError> {
        if !self.list_programs()?.is_empty() {
            return Ok(());
        }
        let now = now_ms();
        let pomodoro = record("Pomodoro", Program::pomodoro(), true, now);
        let cycle = record("Deep cycle", Program::deep_cycle(), false, now);
        let day = record("Studio day", Program::sample_day(), false, now);
        let active = pomodoro.id;
        self.insert_program(&pomodoro)?;
        self.insert_program(&cycle)?;
        self.insert_program(&day)?;
        self.set_active_program(active)?;
        Ok(())
    }

    fn migrate(&self) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute_batch("PRAGMA foreign_keys = ON;"))?;
        let version: i64 = db(conn.pragma_query_value(None, "user_version", |row| row.get(0)))?;
        if version < 1 {
            db(conn.execute_batch(MIGRATION_1))?;
            db(conn.pragma_update(None, "user_version", 1))?;
        }
        Ok(())
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn
            .lock()
            .map_err(|err| StoreError::Message(err.to_string()))
    }
}

impl ProgramRepository for SqliteStore {
    fn list_programs(&self) -> Result<Vec<ProgramRecord>, StoreError> {
        let conn = self.conn()?;
        let mut statement = db(conn.prepare(
            "SELECT id, name, body, preset, created_at_ms, updated_at_ms
             FROM programs ORDER BY created_at_ms ASC",
        ))?;
        let rows = db(statement.query_map([], map_program))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(store_err)
    }

    fn get_program(&self, id: Uuid) -> Result<Option<ProgramRecord>, StoreError> {
        let conn = self.conn()?;
        let mut statement = db(conn.prepare(
            "SELECT id, name, body, preset, created_at_ms, updated_at_ms
             FROM programs WHERE id = ?1",
        ))?;
        let row = db(statement
            .query_row(params![id.to_string()], map_program)
            .optional())?;
        Ok(row)
    }

    fn insert_program(&self, record: &ProgramRecord) -> Result<(), StoreError> {
        let body = serde_json::to_string(&record.program)
            .map_err(|err| StoreError::Message(err.to_string()))?;
        let conn = self.conn()?;
        db(conn.execute(
            "INSERT INTO programs (id, name, body, preset, created_at_ms, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.id.to_string(),
                record.name,
                body,
                record.preset as i64,
                record.created_at_ms,
                record.updated_at_ms,
            ],
        ))?;
        Ok(())
    }

    fn active_program_id(&self) -> Result<Option<Uuid>, StoreError> {
        let Some(value) = self.get_setting(SETTING_ACTIVE_PROGRAM)? else {
            return Ok(None);
        };
        Uuid::parse_str(&value)
            .map(Some)
            .map_err(|err| StoreError::Message(err.to_string()))
    }

    fn set_active_program(&self, id: Uuid) -> Result<(), StoreError> {
        self.set_setting(SETTING_ACTIVE_PROGRAM, &id.to_string())
    }
}

impl SessionRepository for SqliteStore {
    fn insert_session(&self, session: &Session) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "INSERT INTO sessions
             (id, program_id, phase, label, started_at_ms, ended_at_ms, completed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.id.to_string(),
                session.program_id.to_string(),
                session.phase.as_str(),
                session.label,
                session.started_at_ms,
                session.ended_at_ms,
                session.completed as i64,
            ],
        ))?;
        Ok(())
    }

    fn finish_session(
        &self,
        id: Uuid,
        ended_at_ms: i64,
        completed: bool,
    ) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "UPDATE sessions SET ended_at_ms = ?1, completed = ?2 WHERE id = ?3",
            params![ended_at_ms, completed as i64, id.to_string()],
        ))?;
        Ok(())
    }

    fn list_sessions(&self, limit: u32) -> Result<Vec<Session>, StoreError> {
        let conn = self.conn()?;
        let mut statement = db(conn.prepare(
            "SELECT id, program_id, phase, label, started_at_ms, ended_at_ms, completed
             FROM sessions ORDER BY started_at_ms DESC LIMIT ?1",
        ))?;
        let rows = db(statement.query_map(params![limit], map_session))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(store_err)
    }

    fn insert_focus_event(&self, event: &FocusEvent) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "INSERT INTO focus_events
             (id, session_id, program_id, phase, label, at_ms, kind)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id.to_string(),
                event.session_id.map(|id| id.to_string()),
                event.program_id.to_string(),
                event.phase.as_str(),
                event.label,
                event.at_ms,
                event.kind.as_str(),
            ],
        ))?;
        Ok(())
    }
}

impl TaskRepository for SqliteStore {
    fn list_tasks(&self) -> Result<Vec<Task>, StoreError> {
        let conn = self.conn()?;
        let mut statement = db(conn.prepare(
            "SELECT id, title, done, session_id, created_at_ms
             FROM tasks ORDER BY created_at_ms ASC",
        ))?;
        let rows = db(statement.query_map([], map_task))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(store_err)
    }

    fn insert_task(&self, task: &Task) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "INSERT INTO tasks (id, title, done, session_id, created_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                task.id.to_string(),
                task.title,
                task.done as i64,
                task.session_id.map(|id| id.to_string()),
                task.created_at_ms,
            ],
        ))?;
        Ok(())
    }

    fn set_task_done(&self, id: Uuid, done: bool) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "UPDATE tasks SET done = ?1 WHERE id = ?2",
            params![done as i64, id.to_string()],
        ))?;
        Ok(())
    }
}

impl SettingsRepository for SqliteStore {
    fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        let conn = self.conn()?;
        let mut statement = db(conn.prepare("SELECT value FROM settings WHERE key = ?1"))?;
        let value = db(statement
            .query_row(params![key], |row| row.get(0))
            .optional())?;
        Ok(value)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let conn = self.conn()?;
        db(conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        ))?;
        Ok(())
    }
}

fn record(name: &str, program: Program, preset: bool, now: i64) -> ProgramRecord {
    ProgramRecord {
        id: Uuid::new_v4(),
        name: name.to_string(),
        program,
        preset,
        created_at_ms: now,
        updated_at_ms: now,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn db<T>(result: Result<T, rusqlite::Error>) -> Result<T, StoreError> {
    result.map_err(store_err)
}

fn store_err(err: rusqlite::Error) -> StoreError {
    StoreError::Message(err.to_string())
}

fn map_program(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProgramRecord> {
    let id: String = row.get(0)?;
    let body: String = row.get(2)?;
    let program = serde_json::from_str(&body).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(ProgramRecord {
        id: parse_id(&id)?,
        name: row.get(1)?,
        program,
        preset: row.get::<_, i64>(3)? != 0,
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
    })
}

fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    let phase: String = row.get(2)?;
    Ok(Session {
        id: parse_id(&row.get::<_, String>(0)?)?,
        program_id: parse_id(&row.get::<_, String>(1)?)?,
        phase: phase_kind(&phase)?,
        label: row.get(3)?,
        started_at_ms: row.get(4)?,
        ended_at_ms: row.get(5)?,
        completed: row.get::<_, i64>(6)? != 0,
    })
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let session_id: Option<String> = row.get(3)?;
    Ok(Task {
        id: parse_id(&row.get::<_, String>(0)?)?,
        title: row.get(1)?,
        done: row.get::<_, i64>(2)? != 0,
        session_id: session_id.as_deref().map(parse_id).transpose()?,
        created_at_ms: row.get(4)?,
    })
}

fn parse_id(value: &str) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

fn phase_kind(value: &str) -> rusqlite::Result<PhaseKind> {
    PhaseKind::parse(value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(format!("unknown phase {value}"))),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use komodoro_core::{FocusEventKind, SETTING_NOTIFICATIONS};

    #[test]
    fn seed_loads_three_programs_and_activates_pomodoro() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.seed_if_empty().unwrap();
        store.seed_if_empty().unwrap();
        let programs = store.list_programs().unwrap();
        assert_eq!(programs.len(), 3);
        assert!(programs[0].preset);
        assert_eq!(programs[0].name, "Pomodoro");
        assert_eq!(store.active_program_id().unwrap(), Some(programs[0].id));
        assert_eq!(
            store.get_program(programs[1].id).unwrap().unwrap().name,
            "Deep cycle"
        );
    }

    #[test]
    fn sessions_tasks_and_settings_round_trip() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.seed_if_empty().unwrap();
        let program_id = store.active_program_id().unwrap().unwrap();
        let session = Session {
            id: Uuid::new_v4(),
            program_id,
            phase: PhaseKind::Focus,
            label: Some("Deep Work".into()),
            started_at_ms: 10,
            ended_at_ms: None,
            completed: false,
        };
        store.insert_session(&session).unwrap();
        store
            .insert_focus_event(&FocusEvent {
                id: Uuid::new_v4(),
                session_id: Some(session.id),
                program_id,
                phase: PhaseKind::Focus,
                label: session.label.clone(),
                at_ms: 10,
                kind: FocusEventKind::Started,
            })
            .unwrap();
        store.finish_session(session.id, 20, true).unwrap();
        let listed = store.list_sessions(10).unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].completed);
        assert_eq!(listed[0].ended_at_ms, Some(20));

        let task = Task {
            id: Uuid::new_v4(),
            title: "Write the plan".into(),
            done: false,
            session_id: Some(session.id),
            created_at_ms: 11,
        };
        store.insert_task(&task).unwrap();
        store.set_task_done(task.id, true).unwrap();
        assert!(store.list_tasks().unwrap()[0].done);

        store.set_setting(SETTING_NOTIFICATIONS, "0").unwrap();
        assert_eq!(
            store.get_setting(SETTING_NOTIFICATIONS).unwrap().as_deref(),
            Some("0")
        );
    }
}
