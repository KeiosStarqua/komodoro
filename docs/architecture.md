# Kiến trúc

## Stack đã chốt

| Layer | Công nghệ | Vai trò |
|-------|-----------|---------|
| Desktop runtime | **Tauri 2** | Process native, IPC, tray, global shortcut, notification, đóng gói |
| UI | **Leptos** (WASM) | UI reactive, cùng ngôn ngữ với core |
| Core | **Rust + Tokio** | State machine, session, concurrency |
| Database | **SQLite** | Local-first, không server |
| Serialization | **Serde** | Program, IPC payload, settings |

```text
Rust
├── Tauri 2
├── Leptos
├── Tokio
├── SQLite
└── Serde
```

Chọn Leptos thay vì Svelte để UI cũng là Rust. Svelte 5 + Tauri vẫn là một stack desktop hợp lý, nhưng không phải stack của repo này. Elixir + Phoenix LiveView hợp để học actor / OTP, không hợp làm app Pomodoro desktop cá nhân.

Driver SQLite đã chốt: **rusqlite** với feature `bundled`. Core không được phụ thuộc driver.

## Luồng

```text
┌──────────────────────┐
│    Leptos WASM UI    │
│                      │
│  Timer / Tasks /     │
│  Stats / Settings    │
└──────────┬───────────┘
           │
     Tauri IPC
           │
┌──────────▼───────────┐
│    src-tauri shell   │
│                      │
│  Clock / Tray /      │
│  Shortcut / Notify   │
└──────────┬───────────┘
           │
┌──────────▼───────────┐
│   komodoro-core      │
│                      │
│  Timer State Machine │
│  Program interpreter │
└──────────┬───────────┘
           │
┌──────────▼───────────┐
│  komodoro-storage    │
│                      │
│  SQLite (rusqlite)   │
│                      │
│  programs            │
│  sessions            │
│  tasks               │
│  focus_events        │
│  settings            │
└──────────────────────┘
```

UI gửi `TimerEvent` và vẽ state trả về. UI không đếm ngược.

Notification, global shortcut (ví dụ `Ctrl+Shift+P`), và tray thuộc adapter Tauri để session chạy nền. Không làm các thứ này trong webview.

Đóng gói bằng Tauri 2: `.deb`, `.AppImage`, `.dmg`, `.msi`.

## Core thuần

Timer state machine và program interpreter là Rust thuần:

- Không import Tauri, Leptos, SQLite.
- Nhận state + event + program, trả state kế tiếp.
- Thời gian đi vào bằng event `Tick` từ một clock ở rìa. Không dùng `setInterval` trong UI làm nguồn sự thật.

`programs` là bảng bắt buộc vì engine đọc program từ DB, không đọc hằng số.

## State machine

```text
              ┌──────────┐
              │   IDLE   │
              └────┬─────┘
                   │ start
                   ▼
              ┌──────────┐
        ┌────▶│  FOCUS   │
        │     └────┬─────┘
        │          │ timeout
        │          ▼
        │     ┌──────────┐
        │     │  BREAK   │
        │     └────┬─────┘
        │          │ timeout
        │          ▼
        │     ┌──────────┐
        └─────│  FOCUS   │
              └──────────┘
```

Sơ đồ trên là hình dạng. Nhánh tiếp theo do **program** quyết định, không hard-code Focus → ShortBreak → Focus.

```rust
enum TimerState {
    Idle,
    Focus,
    ShortBreak,
    LongBreak,
    Paused,
}

enum TimerEvent {
    Start,
    Pause,
    Resume,
    Skip,
    Tick,
    Complete,
}
```

`Paused` nhớ phase đang dở. `Skip` và `Complete` hỏi program phase kế tiếp. `Tick` chỉ trừ thời gian còn lại của phase hiện tại; hết giờ thì thành `Complete`. Nếu một `Tick` lớn hơn phần còn lại, phase hiện tại kết thúc một lần — thời gian dư không bị cộng sang phase sau.

## Crate

```text
crates/komodoro-core      FSM, program interpreter, port
crates/komodoro-storage   rusqlite
src-tauri                 IPC, clock Tokio, tray, shortcut, notification
ui                        Leptos CSR, bốn surface
```

Dependency đi vào trong: `ui` và `src-tauri` dùng core. `komodoro-storage` dùng core. Core không import Tauri, Leptos, hay rusqlite.

## SQLite

Driver đã chốt: **rusqlite** với feature `bundled`. File DB nằm trong app data dir (`komodoro.db`). Bảng: `programs`, `sessions`, `tasks`, `focus_events`, `settings`.

Program lưu dạng JSON của enum `Program`. YAML trong `docs/examples/` là dạng người viết; `Program::parse_source` đọc YAML đó.

## Đọc program

- `rules` thiếu `long_break.duration` thì long break dài 15 phút.
- `day_plan` thiếu thời lượng thì focus 45 phút, break 10 phút. Activity `Break` / `short break` là `ShortBreak`. Activity `long break` là `LongBreak`. Activity khác là `Focus` mang đúng nhãn đó. Tên block (Morning, Afternoon) không phải state.
- Cycle và rules lặp. Day plan chạy một lượt rồi về `Idle`.
- Preset Pomodoro là `Program::pomodoro()`: focus 25 phút, short break 5 phút, long break 15 phút, sau 4 focus session.
