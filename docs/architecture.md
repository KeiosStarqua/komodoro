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

Driver SQLite (`rusqlite` hoặc `sqlx`) chưa chốt. Core không được phụ thuộc driver nào.

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
│      Rust Core       │
│                      │
│  Timer State Machine │
│  Program interpreter │
│  Session Manager     │
│  Notification        │
│  Global Shortcut     │
│  Tray                │
└──────────┬───────────┘
           │
┌──────────▼───────────┐
│       SQLite         │
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

`Paused` nhớ phase đang dở. `Skip` và `Complete` hỏi program phase kế tiếp. `Tick` chỉ trừ thời gian còn lại của phase hiện tại; hết giờ thì thành `Complete`.
