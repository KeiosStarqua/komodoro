# docs/

## Purpose

Owns the durable product and architecture contracts for Komodoro: what the focus engine is, which stack is locked, and how the timer, UI, and database relate.

## Ownership

- Product shape, program forms, and surfaces live in [`product.md`](product.md)
- Stack, layering, timer state machine, and persistence live in [`architecture.md`](architecture.md)
- GitHub Release channels live in [`releases.md`](releases.md)
- Root [`AGENTS.md`](../AGENTS.md) holds repo-wide DOX rules, the stack lock, and architecture principles
- [`README.md`](../README.md) stays a short entry point and links here

## Local Contracts

- `product.md` is the tie-breaker for product scope. `architecture.md` is the tie-breaker for stack and runtime structure.
- Pomodoro is a preset of the focus engine. Do not document a hardcoded 25/5 loop as the product.
- Named activities (Deep Work, Reading, Coding, Review) are labels on a phase, not new timer states.
- Docs stay operational. Record a decision when it changes what to build; do not keep a diary of rejected drafts.
- Vietnamese prose is fine. Keep identifiers, state names, and stack names in English.

## Work Guidance

## Verification

## Child DOX Index

| Path | Scope |
|------|-------|
| [`product.md`](product.md) | Focus engine, program forms, surfaces, non-goals |
| [`architecture.md`](architecture.md) | Locked stack, layers, state machine, SQLite, desktop integration |
| [`releases.md`](releases.md) | GitHub Release channels: Stable, Beta, Nightly |
| [`examples/`](examples/) | YAML programs the parser accepts |
