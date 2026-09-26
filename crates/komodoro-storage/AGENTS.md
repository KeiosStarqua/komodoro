# crates/komodoro-storage

## Purpose

SQLite adapter for programs, sessions, tasks, focus events, and settings.

## Ownership

This crate owns the schema and the `rusqlite` driver. Core owns the traits this crate implements.

## Local Contracts

- Driver is `rusqlite` with the `bundled` feature. Do not switch drivers here without updating `docs/architecture.md`.
- Map driver errors to `StoreError`. Do not leak `rusqlite::Error` past this crate.
- `programs.body` is JSON of `Program`, not YAML.
- `settings.active_program_id` is the active program. Seed `Pomodoro`, `Deep cycle`, and `Studio day` only when the programs table is empty.
- Keep one `Connection` behind a `Mutex`. Do not open a pool.

## Work Guidance

- Schema changes bump `user_version` with a new migration branch in `migrate`.
- Foreign keys stay on.

## Verification

`cargo test -p komodoro-storage`

## Child DOX Index

| Path | Scope |
|------|-------|
| (none) | |
