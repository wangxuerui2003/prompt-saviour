# 架构概览

## Crate 结构

```
prompt-saviour/
├── crates/ps-core/       # 平台无关：draft、merge、storage、debounce、inject、config
├── crates/ps-macos/      # macOS：AX、keystroke、context、permissions
├── crates/ps-windows/    # 📋 规划
├── crates/ps-daemon/     # CLI 二进制 prompt-saviour
└── crates/ps-gui/        # 📋 Tauri App（规划）
```

## 进程模型

### 当前（CLI POC）

```
┌─────────────────────────────────────┐
│  prompt-saviour run  (foreground)    │
│  ├── AX poll thread                 │
│  ├── Keystroke thread (rdev)        │
│  ├── Main loop: merge → debounce    │
│  └── SQLite WAL writes              │
└─────────────────────────────────────┘
```

用户需手动在 terminal 启动 `run`。
另开 terminal 执行 `list` / `recover`。

### 目标（GUI App）

```
┌──────────────────┐     IPC/events      ┌─────────────────┐
│  Tauri GUI       │◄──────────────────►│  ps-core daemon │
│  tray + windows  │                    │  capture threads│
└──────────────────┘                    └────────┬────────┘
                                                   │
                                            ~/.prompt-saviour/
```

GUI 负责展示；daemon 负责捕获（可同进程 Tauri backend 或 sidecar）。

## 数据流（简化）

```
User typing in Agent App
        │
        ├─► Track A: AX/UIA poll ──► ax_snapshot
        │
        └─► Track B: Keystroke buffer ──► keystroke_snapshot
                    │
                    ▼
              MergeEngine
                    │
                    ▼
              DebounceEngine (500ms idle)
                    │
                    ▼
              DraftStore (SQLite)
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
   CLI list/recover        GUI Current Prompt + History
```

## 关键类型

| 类型 | Crate | 作用 |
|------|-------|------|
| `SessionContext` | ps-core | bundle_id、app_name、pid、window_title |
| `CaptureSnapshot` | ps-core | 单次捕获 + slot key |
| `DraftRecord` | ps-core | 持久化行 |
| `MergeEngine` | ps-core | 多源合并 |
| `DebounceEngine` | ps-core | 防抖 + 去重 |
| `CaptureHub` | ps-macos | 线程化捕获入口 |

## Slot Key

```
SHA256(bundle_id + ":" + pid + ":" + window_title)
```

同一 App 多窗口 → 不同 slot。
Merge 与存储均按 slot 区分。

## 环境变量

| 变量 | 作用 |
|------|------|
| `PROMPT_SAVIOUR_HOME` | 覆盖数据目录（测试/E2E） |
| `RUST_LOG` | 日志级别，如 `prompt_saviour=debug` |

## 跨平台抽象（规划）

```rust
// ps-core/src/platform.rs (规划)
trait CapturePlatform {
    fn check_permissions(&self) -> PermissionStatus;
    fn start_capture(&self, hub: &CaptureHub) -> Result<()>;
    fn read_focused_text(&self) -> Result<Option<String>>;
}
```

- `ps-macos`: `MacCapturePlatform`
- `ps-windows`: `WindowsCapturePlatform`

GUI 与 CLI 只依赖 `ps-core` + platform crate。
