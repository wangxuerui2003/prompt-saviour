# Linux 权限

Prompt Saviour 在 Linux 上使用 **AT-SPI**（GUI 文本）与 **rdev 全局按键监听**（Terminal / 降级路径）。

## 需要的能力

| 能力 | 用途 | 检测 |
|------|------|------|
| AT-SPI / a11y bus | 读取焦点控件文本 | `prompt-saviour doctor` |
| 全局按键 hook | Terminal、Ctrl+V 粘贴 | `doctor` 通过 `rdev` 探测 |

## 桌面环境差异

- **X11 + GNOME/KDE**：AT-SPI 通常默认可用；按键监听可能需要将用户加入 `input` 组（发行版而异）。
- **Wayland**：`rdev` 全局监听常受限；GUI 捕获仍可能通过 AT-SPI 工作，Terminal 捕获可能降级为仅 inject。
- **无图形会话**（SSH）：仅 `smoke` / `inject` / 存储命令可用。

## GUI

Permissions 页显示 AT-SPI 与 Input monitoring 状态，并提供：

- 打开隐私/辅助功能设置（GNOME `gnome-control-center` 等，因 DE 而异）
- Copy Path：将 **实际运行的二进制路径** 加入辅助功能白名单（若 DE 支持按路径授权）

## CLI

```bash
prompt-saviour doctor
prompt-saviour smoke --text "no permissions needed"
```

## 开机自启

Settings → Launch at login 写入 `~/.config/autostart/prompt-saviour.desktop`。

## 限制

- 不同 DE 的设置深链不统一；Permissions 页会尝试常见入口。
- IME 预编辑与复杂布局下按键缓冲可能不完整（见路线图 polish 阶段）。
