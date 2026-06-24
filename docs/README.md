# Prompt Saviour 文档

Prompt Saviour 是跨平台、本地-only 的 coding agent prompt 防丢失工具。
目标是在用户无感的情况下备份未提交的输入，并在 harness 崩溃后快速恢复。

## 文档索引

| 文档 | 说明 |
|------|------|
| [产品概览](product/overview.md) | 问题、目标用户、产品定位 |
| [GUI / App 规格](product/gui-app.md) | 跨平台 GUI 信息架构、页面、权限引导 |
| [功能清单](features/index.md) | 全部功能：已实现 / 规划中 |
| [架构概览](architecture/overview.md) | Crate 划分、进程模型、数据流 |
| [捕获管线](architecture/capture-pipeline.md) | AX、Keystroke、Merge、Debounce |
| [macOS 授权](permissions/macos.md) | 辅助功能、输入监控、Automation |
| [Windows 授权（规划）](permissions/windows.md) | UIA、低级键盘 Hook |
| [CLI 参考](cli/reference.md) | 当前 CLI 命令与行为 |
| [数据与存储](data/storage.md) | SQLite schema、路径、保留策略 |
| [开发环境](development/setup.md) | 构建、运行、环境变量 |
| [测试](development/testing.md) | 单元测试、E2E、smoke |
| [路线图](roadmap.md) | 跨平台 GUI App 与后续里程碑 |

## 当前形态 vs 目标形态

| 维度 | 当前（v0.1 POC） | 目标 |
|------|------------------|------|
| 交付形态 | Rust CLI 二进制 | 跨平台 GUI App + 可选 CLI |
| macOS 捕获 | ✅ AX + Keystroke | ✅ 同上 + Watchdog Toast |
| Windows 捕获 | ❌ | UIA + WH_KEYBOARD_LL |
| 安装 | 手动 `cargo build` | 安装包 / 自动更新 |
| 权限引导 | `doctor` 命令行 | GUI 内嵌引导与状态面板 |
| 当前 prompt 预览 | 无 | GUI 主界面实时展示 |

## 快速链接

- 根目录 [README](../README.md) - 最短上手
- [AGENTS.md](../AGENTS.md) - AI 协作者项目说明
