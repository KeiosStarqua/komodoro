# Sản phẩm

Komodoro là **personal execution engine**: một focus engine lập trình được, chạy native và local-first. Pomodoro cổ điển chỉ là một preset.

Mục tiêu của project: làm một app desktop dùng được, học Rust sâu, và giữ cửa để nó thành sản phẩm sau này.

## Không hard-code chu kỳ

Chu kỳ mặc định 25 phút làm / 5 phút nghỉ không được viết chết trong timer. Thứ tự phase và thời lượng đến từ một **program**.

Ba dạng program đều là first-class:

### Cycle

Danh sách phase tuần tự, mỗi phase có loại và thời lượng.

```yaml
cycle:
  - type: focus
    duration: 45m
  - type: short_break
    duration: 10m
  - type: focus
    duration: 45m
  - type: long_break
    duration: 30m
```

### Rules

Thời lượng focus / break, cộng quy tắc sau N session thì chuyển sang long break.

```yaml
rules:
  focus:
    duration: 50m
  break:
    duration: 10m
  after:
    sessions: 3
    action: long_break
```

### Day plan

Khối theo buổi. Mỗi mục là một activity lặp lại.

```text
Morning
 ├── Deep Work × 2
 ├── Break
 └── Reading × 1
Afternoon
 ├── Coding × 3
 └── Review × 1
```

Activity có tên (Deep Work, Reading, Coding, Review) là **nhãn của một phase focus**, không phải state mới của timer. Break vẫn là break.

Preset Pomodoro (25 phút focus, 5 phút short break, long break sau một số session) được biểu diễn bằng một program `rules` hoặc `cycle`, không bằng hằng số trong core.

## Bề mặt

| Surface | Việc |
|---------|------|
| Timer | Phase hiện tại, thời gian còn lại, start / pause / resume / skip |
| Tasks | Việc gắn với session |
| Stats | Lịch sử focus |
| Settings | Program, shortcut, notification |

## Ngoài phạm vi lúc này

- Đồng bộ đa thiết bị. CRDT / Automerge chỉ là hướng sau, không nằm trong bản đầu.
- Server, tài khoản, web app.
- Elixir, Phoenix LiveView, Electron, UI JavaScript.
