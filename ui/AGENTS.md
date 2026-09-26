# ui

## Purpose

Leptos CSR interface for the four surfaces: Timer, Tasks, Stats, Settings.

## Ownership

Layout, copy, and view state live here. The countdown, program rules, and database do not.

## Local Contracts

- Build with Trunk (`scripts/ui.sh`), target `wasm32-unknown-unknown`. This crate is not a `cargo run` binary. The script sets `NO_COLOR=true` when that variable is present, because Trunk rejects `NO_COLOR=1`.
- Talk to the shell only through `window.__TAURI__.core.invoke` in `ipc.rs`.
- `ipc.rs` bridges command args and replies as JSON text. `invoke` needs a plain JS object, never a serialized map.
- Render `AppSnapshot`. Do not subtract remaining time in the view.
- Timer, Tasks, Stats, and Settings are the only surfaces.

## Work Guidance

- Keep the timer stage scannable: phase on the left, program name on the right, clock on the left edge, actions on the next row.
- Rows share one left edge and one right edge. Do not wrap each row in a card.
- Stats answers "how much did I focus today" first: the total sits on the left edge above the history rows.
- A history row with no end time reads `interrupted`, except the open session which reads `in progress`.

## Verification

`cargo check -p komodoro-ui --target wasm32-unknown-unknown`

The running app is checked with `cargo tauri dev`.

## Child DOX Index

| Path | Scope |
|------|-------|
| (none) | |
