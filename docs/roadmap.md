# 路线图

## 阶段 0 - CLI POC ✅（当前）

- [x] ps-core：Merge、Debounce、SQLite、inject
- [x] ps-macos：AX + Keystroke + permissions
- [x] CLI：run、list、recover、doctor、status、smoke、inject
- [x] 测试 + `scripts/e2e_macos.sh`
- [x] 文档 + AGENTS.md

## 阶段 1 - GUI App 骨架（macOS 优先）

- [x] 新建 `crates/ps-gui`（Tauri 2）
- [x] 托盘常驻 + 主窗口框架
- [x] 内嵌 daemon（同进程）
- [x] **Current Prompt 页**：实时展示 merge 结果
- [x] **权限页**：状态检测 + 打开系统设置 + Copy Path
- [x] **历史页**：读 SQLite，复制恢复
- [x] **设置页**：映射 config.json
- [x] IPC：`current-prompt-update` 事件

## 阶段 2 - macOS 打包与体验

- [x] `.app` + `.dmg` 构建脚本（`scripts/build_macos_app.sh`；签名分发待 Apple 证书）
- [x] LaunchAgent 开机自启
- [x] 崩溃 Watchdog + Toast 通知
- [x] 全局热键打开主窗口
- [x] 日志查看器
- [x] 界面语言切换（英文 / 简体中文）

## 阶段 3 - Windows + Linux 对等

- [x] `ps-windows`：UIA + `ps-input` keystroke
- [x] `ps-linux`：AT-SPI + `ps-input` keystroke
- [x] Windows / Linux GUI 权限页
- [x] 跨平台 autostart（Registry / XDG desktop）
- [x] `scripts/e2e.sh` + 各平台 optional live 脚本
- [x] GitHub Actions CI（macOS / Windows / Ubuntu）
- [ ] `.msi` / Linux 安装包签名与分发
- [ ] Windows / Linux 实机 live capture 回归（CI 仅 smoke + inject）

## 阶段 4 - 增强

- [ ] Terminal 适配器（iTerm2 API、WezTerm）
- [ ] 可选 VS Code/Cursor extension 作为高置信源
- [ ] 进程识别表自动更新（Codex / Claude Code bundle id）
- [ ] 可选本地加密
- [ ] 自动更新

## 阶段 5 -  polish

- [ ] IME 预编辑态完善
- [ ] 多显示器 / 多 Space 窗口标题
- [ ] 排除 App UI（拖拽 bundle 到黑名单）
- [ ] 导出 / 导入全部 drafts

## 优先级原则

1. **Current Prompt 在 GUI 可见** - 用户核心诉求
2. **权限在 GUI 内可完成** - 不再查文档找 binary 路径
3. **跨平台功能对等** - macOS 不长期领先 Windows
4. CLI 保持可用 - 脚本与 CI 友好
