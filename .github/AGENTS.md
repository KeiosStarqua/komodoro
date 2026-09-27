# .github

## Purpose

Owns CI and GitHub Release pipelines: unit tests, and publishing desktop bundles on three channels.

## Ownership

Workflows and the shared Tauri publish action live here. Timer rules stay in `komodoro-core`. In-app updater behavior stays out until that feature lands.

## Local Contracts

- `workflows/ci.yml` runs `cargo test`. It does not bundle the desktop app.
- Release jobs use `.github/actions/tauri-publish`. Frontend is Trunk / Leptos (`scripts/ui.sh build`). Do not add npm to the publish path.
- `tauriScript` is `cargo tauri`. cargo-binstall installs `cargo-tauri`, not a `tauri` binary. On Apple Silicon, pass `--pkg-fmt zip` so binstall does not fall back to the x86_64 CLI.
- Every publish runner needs `wasm32-unknown-unknown` because `beforeBuildCommand` compiles the UI to WASM.
- Linux images need WebKitGTK 4.1, `libayatana-appindicator3-dev`, and `libxdo` (global shortcut). Do not also install `libappindicator3-dev`; it conflicts with the ayatana package.
- **Stable**: git tag `vX.Y.Z` → GitHub Release, latest. **Beta**: tag `vX.Y.Z-beta.N` → prerelease. **Nightly**: rolling tag `nightly`, always prerelease, never latest.
- `uploadUpdaterJson` stays false until updater signing exists. Do not commit `TAURI_SIGNING_PRIVATE_KEY`.

## Work Guidance

- Cut channels as documented in [`docs/releases.md`](../docs/releases.md).
- `workflow_dispatch` on `release.yml` and `nightly.yml` is the way to publish a test build without a tag.
- macOS CI uses ad-hoc signing (`signingIdentity: "-"` in `src-tauri/tauri.conf.json`) until Apple certificates exist.
- In-app auto-update is a product feature, not a new workflow.

## Verification

- `cargo test` on `ci.yml`.
- Actions → **nightly** or **release** → confirm assets on the GitHub Release (`.deb` / `.AppImage` / `.dmg` / `.msi` or NSIS).

## Child DOX Index

| Path | Scope |
|------|-------|
| [`actions/tauri-publish/`](actions/tauri-publish/) | Shared Trunk + Tauri 2 build and upload |
| [`workflows/ci.yml`](workflows/ci.yml) | Unit tests |
| [`workflows/release.yml`](workflows/release.yml) | Stable and Beta |
| [`workflows/nightly.yml`](workflows/nightly.yml) | Rolling nightly |
