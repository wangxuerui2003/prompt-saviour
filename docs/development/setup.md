# 开发环境

## 要求

- Rust **1.75+**（当前 toolchain 1.94）
- macOS：Xcode CLT（AX、CoreGraphics）
- Windows（规划）：VS Build Tools

## 克隆与构建

```bash
git clone <repo> prompt-saviour
cd prompt-saviour
cargo build --release
```

产物：`target/release/prompt-saviour`

Debug：

```bash
cargo build
./target/debug/prompt-saviour smoke
```

## 工作区结构

| Crate | 职责 |
|-------|------|
| `ps-core` | 平台无关逻辑 |
| `ps-macos` | macOS 捕获 |
| `ps-daemon` | CLI 入口 |

规划新增：

| Crate | 职责 |
|-------|------|
| `ps-windows` | Windows 捕获 |
| `ps-gui` | Tauri 前端 + commands |

## 本地运行 daemon

```bash
# 可选：隔离数据
export PROMPT_SAVIOUR_HOME=/tmp/ps-dev
export RUST_LOG=prompt_saviour=debug

cargo run -- run
```

另开 terminal：

```bash
export PROMPT_SAVIOUR_HOME=/tmp/ps-dev
cargo run -- list
```

## 代码规范

- Edition 2021
- 错误：`anyhow`（binary）/ `thiserror`（库边界可选）
- 日志：`tracing` + `tracing-subscriber`
- 并发：`parking_lot`、`Arc`、专用 capture 线程
- 最小 diff；不 over-engineer

## GUI 开发（规划）

```bash
# 预期结构
cd crates/ps-gui
npm install
cargo tauri dev
```

Tauri backend 直接依赖 `ps-core` + `ps-macos` / `ps-windows`。
IPC 事件：`current-prompt-update`、`permission-status`、`draft-saved`。

## 相关文档

- [测试](testing.md)
- [架构](../architecture/overview.md)
- [AGENTS.md](../../AGENTS.md)
