# DOX framework

- DOX is highly performant AGENTS.md hierarchy installed here
- Agent must follow DOX instructions across any edits

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it

## Read Before Editing

1. Read the root AGENTS.md
2. Identify every file or folder you expect to touch
3. Walk from the repository root to each target path
4. Read every AGENTS.md found along each route
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Update After Editing

Every meaningful change requires a DOX pass before the task is done.

Update the closest owning AGENTS.md when a change affects:

- purpose, scope, ownership, or responsibilities
- durable structure, contracts, workflows, or operating rules
- required inputs, outputs, permissions, constraints, side effects, or artifacts
- user preferences about behavior, communication, process, organization, or quality
- AGENTS.md creation, deletion, move, rename, or index contents

Update parent docs when parent-level structure, ownership, workflow, or child index changes. Update child docs when parent changes alter local rules. Remove stale or contradictory text immediately. Small edits that do not change behavior or contracts may leave docs unchanged, but the DOX pass still must happen.

## Hierarchy

- Root AGENTS.md is the DOX rail: project-wide instructions, global preferences, durable workflow rules, and the top-level Child DOX Index
- Child AGENTS.md files own domain-specific instructions and their own Child DOX Index
- Each parent explains what its direct children cover and what stays owned by the parent
- The closer a doc is to the work, the more specific and practical it must be

## Child Doc Shape

- Create a child AGENTS.md when a folder becomes a durable boundary with its own purpose, rules, responsibilities, workflow, materials, or quality standards
- Work Guidance must reflect the current standards of the project or user instructions; if there are no specific standards or instructions yet, leave it empty
- Verification must reflect an existing check; if no verification framework exists yet, leave it empty and update it when one exists

Default section order:
- Purpose
- Ownership
- Local Contracts
- Work Guidance
- Verification
- Child DOX Index

## Style

- Keep docs concise, current, and operational
- Document stable contracts, not diary entries
- Put broad rules in parent docs and concrete details in child docs
- Prefer direct bullets with explicit names
- Do not duplicate rules across many files unless each scope needs a local version
- Delete stale notes instead of explaining history
- Trim obvious statements, repeated rules, misplaced detail, and warnings for risks that no longer exist

## Closeout

1. Re-check changed paths against the DOX chain
2. Update nearest owning docs and any affected parents or children
3. Refresh every affected Child DOX Index
4. Remove stale or contradictory text
5. Run existing verification when relevant
6. Report any docs intentionally left unchanged and why

## User Preferences

When the user requests a durable behavior change, record it here or in the relevant child AGENTS.md.

### Product

Komodoro is a native, local-first **programmable focus engine**. Classic Pomodoro (25/5) is one preset, not the product. Phase order and durations come from a program: an explicit cycle, a rules block, or a day plan.

Product and architecture contracts: [`docs/product.md`](docs/product.md), [`docs/architecture.md`](docs/architecture.md).

### Stack

Locked: **Tauri 2 + Leptos + Rust + Tokio + SQLite + Serde**.

The UI is Leptos compiled to WASM inside Tauri. Do not add a JavaScript UI framework (Svelte, React, Next.js) or switch the runtime to Electron or Elixir unless the user reopens this decision.

Chosen so the project is a real desktop app and a way to learn Rust in depth. Multi-device sync (CRDT / Automerge) stays optional and out of the first cut.

### Architecture Principles

Code follows **Clean Architecture**: dependencies point inward (entities → use cases → adapters → frameworks). Business rules never depend on UI, DB, or framework details.

The timer finite-state machine and the program interpreter are pure Rust. They do not import Tauri, Leptos, or SQLite. The UI sends events and renders state; it does not own the countdown.

#### SOLID

- **S**: One reason to change per module
- **O**: Extend behavior with new types, not edits to stable code
- **L**: Subtypes must honor base contracts
- **I**: Small, role-specific interfaces
- **D**: Inner layers depend on abstractions, not concrete frameworks

#### Frontend: feature layer

Organize UI by feature (timer, tasks, stats, settings). Features may use shared modules; shared modules never depend on a feature. Export only the public surface of each feature.

#### Deep modules

Prefer modules with **simple interfaces and substantial hidden complexity**. Avoid shallow pass-through wrappers. Split only when a module hides real complexity — not to hit arbitrary file-size limits.

#### Design patterns

Use GoF patterns when they match a recurring problem — not for decoration. The timer core is a finite-state machine. Catalog: https://refactoring.guru/design-patterns/catalog

## Child DOX Index

| Path | Scope |
|------|-------|
| [`docs/AGENTS.md`](docs/AGENTS.md) | Product definition and architecture contracts |
