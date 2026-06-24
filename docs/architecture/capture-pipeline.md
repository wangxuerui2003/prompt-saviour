# 捕获管线

## 双轨并行

Prompt Saviour 不依赖单一 OS API。
GUI 与 Terminal 场景同时运行两轨，再 Merge。

| 轨道 | 机制 | 擅长 | 弱项 |
|------|------|------|------|
| **A: AX/UIA** | 轮询 focused element 的 value | Electron App 完整 textarea | Terminal 当前行 |
| **B: Keystroke** | pass-through 全局按键缓冲 | Terminal、未知 App | 修饰键编辑需配合 A |

## Track A: Accessibility（macOS 已实现）

- 间隔：`ax_poll_ms`（默认 400ms）
- API：`AXUIElementCreateSystemWide` → focused → `kAXValueAttribute`
- 权限：Accessibility（辅助功能）
- 修饰键：检测到 Cmd/Option/Ctrl 时 `request_immediate_ax_poll()`，下一轮 50ms 延迟

### 适用 App

Codex App、Claude Code App、Cursor、VS Code、TextEdit 等 Electron/native textarea。

## Track B: Keystroke Buffer（macOS 已实现）

- 库：`rdev` listen（pass-through，不拦截按键）
- 权限：Input Monitoring（输入监控）；macOS 上与 AX 常一同授予
- 按 `SessionContext.slot_key()` 维护 `HashMap<String, String>`

### 按键语义

| 输入 | 行为 |
|------|------|
| 可打印键 | append（US 布局映射，大小写靠 AX 校正） |
| Backspace | pop |
| Option+Backspace | delete_word_backward + AX poll |
| Enter | `\n` |
| Cmd/Option/Ctrl + 键 | AX 即时 poll；Cmd+V 读剪贴板 append |
| Shift 等 | 修饰键状态追踪 |

### Terminal 识别

`MergeEngine` 用 bundle id **或** app_name 判断 Terminal：
iTerm2、Terminal、WezTerm、Warp、Alacritty、kitty 等。

## Merge 规则

```
if terminal_session:
    content = longest(keystroke, ax, clipboard)
else:  # GUI
    content = ax if len(ax) >= 8 else longest(all)
```

输出 `CaptureSource::Merged` 快照。

最小长度：**8 字符**（`chars().count()`）。

## Debounce 落盘

`DebounceEngine`：

1. `observe(snapshot)` - 仅当 slot+content fingerprint 变化时重置计时
2. `poll_ready()` - idle ≥ `debounce_ms` 且非重复 fingerprint → 写入

避免旧 bug：每 100ms merge 重置计时导致永不落盘。

## Storage 去重

`upsert_snapshot` 写入前查询同 slot 最新 content。
相同则跳过 INSERT（仍保留历史版本当 content 变化时新增行）。

## inject 通道（调试）

`~/.prompt-saviour/inject.json`：

```json
{
  "text": "...",
  "app_name": "TextEdit",
  "bundle_id": "com.apple.TextEdit",
  "window_title": "TextEdit"
}
```

Daemon main loop 每轮 `take_inject_snapshot()`，消费后删除文件。
CLI：`prompt-saviour inject "text" --app X --bundle Y`

## Windows 规划

| 轨道 | API |
|------|-----|
| A | UI Automation `ValuePattern` |
| B | `SetWindowsHookEx(WH_KEYBOARD_LL)` pass-through |
| Merge / Debounce / Storage | 复用 ps-core |

## 已知限制（v0.1）

- Keystroke 层非 US 键盘 / IME 预编辑态 imperfect；GUI 靠 AX 兜底
- 无光标位置：Delete 键、鼠标选中删除依赖 AX
- `run` 需前台 terminal；无 GUI tray
- Watchdog 未实现
