# crates/

## Purpose

Workspace library crates: the pure focus engine and the SQLite adapter that implements its ports.

## Ownership

- [`komodoro-core`](komodoro-core/AGENTS.md) owns programs, the timer state machine, and repository traits
- [`komodoro-storage`](komodoro-storage/AGENTS.md) owns the schema and the `rusqlite` driver
- The desktop shell and the Leptos UI stay in [`src-tauri`](../src-tauri/AGENTS.md) and [`ui`](../ui/AGENTS.md)
- Root [`AGENTS.md`](../AGENTS.md) holds the stack lock and architecture principles

## Local Contracts

- Dependencies point inward. `komodoro-storage` depends on `komodoro-core`. Core does not depend on storage, Tauri, or Leptos.
- Workspace members are declared in the root `Cargo.toml`.

## Work Guidance

## Verification

`cargo test` from the repo root runs these crates. `default-members` is core and storage.

## Child DOX Index

| Path | Scope |
|------|-------|
| [`komodoro-core/AGENTS.md`](komodoro-core/AGENTS.md) | Pure timer, program interpreter, persistence ports |
| [`komodoro-storage/AGENTS.md`](komodoro-storage/AGENTS.md) | SQLite adapter |
