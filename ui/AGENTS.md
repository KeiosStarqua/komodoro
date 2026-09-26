# ui

## Purpose

Leptos CSR interface for the four surfaces: Timer, Tasks, Stats, Settings.

## Ownership

Layout, copy, and view state live here. The countdown, program rules, and database do not.

## Local Contracts

- Build with Trunk (`scripts/ui.sh`), target `wasm32-unknown-unknown`. This crate is not a `cargo run` binary.
- Talk to the shell only through `window.__TAURI__.core.invoke` in `ipc.rs`.
- Render `AppSnapshot`. Do not subtract remaining time in the view.
- Timer, Tasks, Stats, and Settings are the only surfaces.

## Work Guidance

- Keep the timer stage scannable: phase on the left, program name on the right, clock on the left edge, actions on the next row.
- Rows share one left edge and one right edge. Do not wrap each row in a card.

## Verification

`cargo check -p komodoro-ui --target wasm32-unknown-unknown`

The running app is checked with `cargo tauri dev`.

## Child DOX Index

| Path | Scope |
|------|-------|
| (none) | |
