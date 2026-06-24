# Windows 授权

Windows 版 Prompt Saviour 使用 **UI Automation**（GUI 文本）与 **rdev 全局按键监听**（Terminal / 降级路径）。

## 需要的 capability

| 能力 | API | 用途 | GUI 展示 |
|------|-----|------|----------|
| UI Automation | `IUIAutomation` | 读 Electron / Win32 文本框 | UI Automation |
| 全局按键 hook | `rdev` | Terminal、Ctrl+V 粘贴 | Input monitoring |
| 剪贴板 | `arboard` | Ctrl+V 路径 | 通常无需单独权限 |
| 开机自启 | HKCU `Run` | 托盘常驻 | Settings 开关 |

## CLI

```bash
prompt-saviour doctor
prompt-saviour smoke --text "no permissions needed"
```

## GUI 权限页

- 状态卡片：UI Automation、Input monitoring
- **Open Settings** → `ms-settings:privacy`
- Copy Path：将实际 `.exe` 路径加入杀软/辅助功能白名单（若需要）

## 与 macOS 差异

| 项目 | macOS | Windows |
|------|-------|---------|
| GUI 读文本 | AX | UIA |
| 全局按键 | Input Monitoring + rdev | rdev hook |
| 安装形态 | `.app` / `.dmg` | `.msi`（`npm run tauri build`） |

## 杀软 / 企业环境

全局按键 hook 可能触发 Windows Defender 或 EDR。
若 `doctor` 显示 input monitoring 缺失，尝试将二进制加入信任列表。

## 测试

```bash
bash scripts/e2e.sh
bash scripts/e2e_windows.sh   # optional live Notepad
```
