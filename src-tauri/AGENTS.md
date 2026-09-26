# src-tauri

## Purpose

Desktop shell. Turns core effects into SQLite writes, notifications, a tray menu, and the global shortcut. Owns the Tokio clock that emits `Tick`.

## Ownership

IPC commands, app data path, tray, shortcut, and notification delivery live here. Timer rules stay in `komodoro-core`. SQL stays in `komodoro-storage`.

## Local Contracts

- Commands are thin wrappers over `runtime`. Do not reimplement transitions in a command.
- The clock sends real elapsed milliseconds. It does not assume every wake is exactly one second.
- `Ctrl+Shift+P` toggles start / pause / resume. Register failure must not stop the app.
- Notifications, tray, and the shortcut stay in this process. Do not implement them in `ui`.
- Changing the active program is rejected while the timer is not `Idle`.

## Work Guidance

- A new surface needs a command here and a view in `ui`. Share the DTO from core.
- Linux dev needs the Tauri 2 WebKitGTK packages before `cargo tauri dev`.

## Verification

`cargo test` covers core and storage. `cargo tauri dev` from the repo root is the shell check.

## Child DOX Index

| Path | Scope |
|------|-------|
| (none) | |
