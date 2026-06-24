# 产品概览

## 要解决的问题

用户在 coding agent 的输入框里（GUI chat 或 Terminal CLI）写了很长一段 prompt，尚未提交时 harness 崩溃，输入随进程一起消失。
Undo、session state、readline history 通常救不回来。

典型场景：

- Cursor / VS Code Copilot Chat 里写了长 prompt，Electron 进程崩溃
- Codex App、Claude Code App 独立客户端崩溃
- Terminal 里 `claude` / `codex` CLI 的 TUI 或 readline 输入丢失

## 产品定位

- **Local-only**：数据只在 `~/.prompt-saviour/`（或 `PROMPT_SAVIOUR_HOME`），零网络
- **无感捕获**：不替代输入法，不劫持输入链路（pass-through 监听）
- **全覆盖优先**：GUI App + Terminal + 未知 App，双轨捕获 + Merge
- **跨平台**：macOS 与 Windows 同等重要（GUI App 统一体验）

## 非目标（当前阶段）

- 不做云端同步
- 不做 prompt 版本协作
- 不替代 agent 本身的输入框
- 不做企业 MDM / 合规审计级隐私策略（个人工具，local-only）

## 目标用户

- 日常使用 Cursor、Codex、Claude Code 等 agent 的开发者
- 在 Terminal 里跑 CLI agent 的用户
- 曾因崩溃丢失长 prompt 而 frustrated 的人

## 成功标准

1. 用户正常打字，零额外操作
2. 崩溃后 30 秒内能找回最近 draft
3. GUI 里一眼看到：权限状态、当前捕获的 prompt、历史列表
4. macOS + Windows 功能对等（捕获能力因 OS API 略有差异）
