# Komodoro

Focus engine lập trình được, chạy native và local-first. Pomodoro cổ điển là một preset.

**Stack:** Tauri 2 · Leptos · Rust · Tokio · SQLite · Serde

| Tài liệu | Nội dung |
|----------|----------|
| [docs/product.md](docs/product.md) | Engine, ba dạng program, bề mặt app |
| [docs/architecture.md](docs/architecture.md) | Layer, state machine, SQLite, tray / shortcut |

Giấy phép [MIT](LICENSE).

## Chạy

Cần Rust stable (kèm target `wasm32-unknown-unknown`), [Trunk](https://trunkrs.dev/), và Tauri CLI 2.

```sh
cargo test
cargo install tauri-cli --version "^2" --locked
cargo install trunk --locked
sh scripts/ui.sh serve   # chỉ UI, không có timer
cargo tauri dev          # app desktop, trong thư mục repo
```

Linux: cài gói Tauri 2 trước khi `cargo tauri dev`.

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  patchelf pkg-config libdbus-1-dev
```

`cargo test` chạy core và storage. UI build bằng Trunk, không bằng `cargo run`.

| Path | Việc |
|------|------|
| `crates/komodoro-core` | State machine và program |
| `crates/komodoro-storage` | SQLite |
| `src-tauri` | Clock, tray, `Ctrl+Shift+P`, notification |
| `ui` | Timer, Tasks, Stats, Settings |
| `docs/examples` | YAML mẫu |
