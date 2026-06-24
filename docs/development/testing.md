# 测试

## 单元测试

```bash
cargo test
```

覆盖：

| 模块 | 测试 |
|------|------|
| `ps-core::merge` | GUI 优先 AX、Terminal 优先 keystroke、app_name 识别 |
| `ps-core::config` | DebounceEngine 时序与去重 |
| `ps-core::storage` | 同 slot 去重、内容更新 |
| `ps-core::inject` | inject.json 读写 |
| `ps-macos::keystroke` | delete_word_backward |
| `ps-macos::context` | frontmost_session（有 GUI 时） |

## 集成测试

`crates/ps-daemon/tests/pipeline_e2e.rs`：

- pipeline_persists_gui_prompt_after_debounce
- pipeline_terminal_prefers_keystroke_track
- config_respects_prompt_saviour_home
- recover_roundtrip_content

使用 `PROMPT_SAVIOUR_HOME` + 全局 `ENV_LOCK` 避免并行冲突。

```bash
cargo test -p ps-daemon --test pipeline_e2e
```

## Smoke（无权限）

```bash
cargo run -- smoke --text "hello smoke test prompt"
cargo run -- list
```

## 跨平台 E2E

```bash
bash scripts/e2e.sh
```

阶段：

1. **workspace `cargo test`** - 含 `gui_e2e`、pipeline、平台 stub
2. **CLI smoke** - 无权限 pipeline
3. **GUI launch smoke** - 进程存活 2s
4. **daemon inject** - inject.json 通道
5. **Optional live capture** - 按 OS 调用 `e2e_macos.sh` / `e2e_windows.sh` / `e2e_linux.sh`

```bash
cargo test -p prompt-saviour-gui --test gui_e2e
```

## macOS 专用（可选 live TextEdit）

```bash
bash scripts/e2e_macos.sh
```

## GUI E2E（后续）

- Tauri WebDriver 或 Playwright UI 自动化
- 权限 mock 层（CI 无 AX）
- 截图回归：权限页、Current Prompt 页、空状态

## 修复 bug 流程（项目要求）

1. 先 E2E 或 smoke 复现
2. 写失败测试（若可）
3. 修复
4. `cargo test` + `bash scripts/e2e.sh`
