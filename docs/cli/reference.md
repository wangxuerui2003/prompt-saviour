# CLI 参考

二进制名：`prompt-saviour`
构建：`cargo build --release` → `target/release/prompt-saviour`

当前为 **CLI POC**。
GUI App 上线后，下列命令仍保留，与 GUI 共用数据目录。

## 全局

| 环境变量 | 说明 |
|----------|------|
| `PROMPT_SAVIOUR_HOME` | 数据根目录（默认 `~/.prompt-saviour`） |
| `RUST_LOG` | 日志，如 `RUST_LOG=prompt_saviour=debug` |

---

## `run`

启动捕获 daemon（**macOS / Windows / Linux**）。

```bash
prompt-saviour run
```

行为：

- 启动平台 GUI 文本轮询（macOS AX / Windows UIA / Linux AT-SPI）
- 启动 Keystroke 监听（`ps-input` / rdev，需平台输入权限）
- 主循环：merge → debounce → SQLite
- 监听 `inject.json`
- **Ctrl+C** 优雅退出

前台占用 terminal。
规划：GUI 版改为后台 daemon + tray。

---

## `list`

列出最近 draft。

```bash
prompt-saviour list
prompt-saviour list --limit 50
```

输出列：id、source、字符数、时间、App 名、预览。

---

## `recover`

恢复 draft 到**系统剪贴板**。

```bash
prompt-saviour recover          # 最新一条
prompt-saviour recover 42       # 指定 id
```

不会自动粘贴到前台窗口（避免惊吓用户）。
GUI 规划可选「一键粘贴」。

---

## `doctor`

检查 / 请求平台捕获权限。

```bash
prompt-saviour doctor
```

| 平台 | 显示项 |
|------|--------|
| macOS | Accessibility / Input Monitoring |
| Windows | UI Automation / Input monitoring |
| Linux | AT-SPI / Input monitoring |

macOS 未授权时会尝试弹出系统授权对话框。

---

## `status`

显示路径与配置。

```bash
prompt-saviour status
```

输出：数据库路径、config 路径、inject 文件路径、`ax_poll_ms`、`debounce_ms`。

---

## `smoke`

**无 OS 权限**的 pipeline 自检。

```bash
prompt-saviour smoke
prompt-saviour smoke --text "my custom test prompt text"
```

内存中 merge + debounce + 写入 SQLite，打印 `Smoke OK: draft #N`。

用于 CI 与首次验证安装。

---

## `inject`

向**运行中**的 daemon 队列测试 payload。

```bash
prompt-saviour inject "prompt text here" --app TextEdit --bundle com.apple.TextEdit
```

写入 `inject.json`，daemon 约 100ms 内消费。
用于 E2E 与 debug。

注意：`TEXT` 为**位置参数**，不是 `--text`。

---

## 退出码

| 码 | 含义 |
|----|------|
| 0 | 成功 |
| 非 0 | 错误（权限、IO、无 draft 等） |

---

## 与 GUI 映射

见 [GUI 规格 - 与 CLI 的关系](../product/gui-app.md#与-cli-的关系)。
