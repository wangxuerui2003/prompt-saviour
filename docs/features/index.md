# 功能清单

状态图例：**✅ 已实现** · **🚧 进行中 / POC** · **📋 规划**

## 捕获与保护

| 功能 | 状态 | 说明 |
|------|------|------|
| GUI App AX/UIA 轮询 | ✅ macOS | Cursor、Codex App、Claude Code App、TextEdit 等 |
| 全局 Keystroke buffer | ✅ macOS | pass-through，`rdev`；Terminal 兜底 |
| 修饰键触发即时 AX 重读 | ✅ macOS | Cmd/Option/Ctrl 组合 |
| Cmd+V 粘贴捕获 | ✅ macOS | 读剪贴板 append 到 keystroke buffer |
| Option+Backspace 删词 | ✅ macOS | keystroke 模拟 + AX 重读 |
| 剪贴板后台监听 | ❌ 已移除 | 曾污染无关 copy；改为仅 Cmd+V 路径 |
| Windows UIA 轮询 | 📋 | `ps-windows` |
| Windows 低级键盘 Hook | 📋 | WH_KEYBOARD_LL |
| Terminal 适配器（iTerm2 等） | 📋 | 补充 Keystroke 盲区 |
| Harness 插件（VS Code extension） | 📋 | 100% 置信度补充源 |

## 数据处理

| 功能 | 状态 | 说明 |
|------|------|------|
| 多源 Merge Engine | ✅ | GUI 优先 AX；Terminal 优先最长 buffer |
| Debounce 落盘 | ✅ | 默认 500ms；内容不变不重置 |
| 同 slot 去重 | ✅ | 内容相同跳过 INSERT |
| SQLite WAL 存储 | ✅ | `~/.prompt-saviour/drafts.db` |
| 保留策略 | ✅ | 30 天 / 500 条 / 每 slot 最多 5 版 |
| `PROMPT_SAVIOUR_HOME` | ✅ | 测试隔离 |
| inject.json 注入通道 | ✅ | E2E / debug |
| 本地加密 | 📋 | 当前明文；个人工具 |

## 恢复

| 功能 | 状态 | 说明 |
|------|------|------|
| CLI `list` | ✅ | |
| CLI `recover` | ✅ | 复制到系统剪贴板 |
| 按 id 恢复 | ✅ | |
| 崩溃 Toast | 📋 | Watchdog + 系统通知 |
| 全局热键 picker | 📋 | GUI 阶段 |
| 模拟粘贴到前台 | 📋 | 可选；默认仅 clipboard |

## 权限与健康

| 功能 | 状态 | 说明 |
|------|------|------|
| CLI `doctor` | ✅ macOS | 检测 + 触发 AX 授权弹窗 |
| GUI 权限面板 | 📋 | 见 [gui-app.md](../product/gui-app.md) |
| 二进制路径展示 | 📋 GUI | CLI 仅文字提示 |
| Windows 权限引导 | 📋 | |

## 可观测性

| 功能 | 状态 | 说明 |
|------|------|------|
| 结构化日志 `tracing` | ✅ | `RUST_LOG` |
| CLI `status` | ✅ | 路径与配置 |
| GUI 实时 Current Prompt | 📋 | |
| GUI 仪表盘 | 📋 | |
| 日志查看器 | 📋 GUI | |

## 交付与安装

| 功能 | 状态 | 说明 |
|------|------|------|
| Rust CLI 二进制 | ✅ | `cargo build --release` |
| 跨平台 GUI App | 📋 | Tauri 2 目标 |
| macOS `.app` / `.dmg` | 📋 | |
| Windows 安装包 | 📋 | |
| 开机自启 | 📋 | LaunchAgent / Task Scheduler |
| CLI 保留 | 📋 | 与 GUI 共用 daemon |

## 开发者

| 功能 | 状态 | 说明 |
|------|------|------|
| `smoke` | ✅ | 无权限 pipeline 自检 |
| `inject` | ✅ | 向运行中 daemon 队列 payload |
| 单元 + 集成测试 | ✅ | `cargo test` |
| `scripts/e2e_macos.sh` | ✅ | smoke + inject + 可选 live |
| CI | 📋 | |

## 支持的目标 App（设计覆盖）

| App | GUI | Terminal | 预期捕获 |
|-----|-----|----------|----------|
| Cursor | ✅ | - | AX 主 |
| VS Code + Copilot | ✅ | - | AX 主 |
| Codex App | ✅ | - | AX 主 |
| Claude Code App | ✅ | - | AX 主 |
| Claude Code CLI | - | ✅ | Keystroke 主 |
| Codex CLI | - | ✅ | Keystroke 主 |
| Windsurf / Zed | ✅ | - | AX 通用 |
| iTerm2 / Terminal.app | - | ✅ | Keystroke + 适配器 |
| Windows Terminal | - | 📋 | UIA + Hook |
| 浏览器 Web UI | 📋 | - | Hook 兜底 |
