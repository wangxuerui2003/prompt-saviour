# macOS 授权指南

Prompt Saviour 当前是 **CLI 二进制**，不是 `.app`。
系统设置里不会自动出现「Prompt Saviour」名称。
你必须手动添加**正在运行的那个二进制文件**的完整路径。

## 需要的权限

| 权限 | 系统设置路径 | 用途 | 必需？ |
|------|--------------|------|--------|
| **Accessibility**（辅助功能） | 隐私与安全性 → 辅助功能 | AX 读取 GUI textarea | GUI 捕获必需 |
| **Input Monitoring**（输入监控） | 隐私与安全性 → 输入监控 | Keystroke pass-through | Terminal / 兜底必需 |

两者建议都开。
只开 AX 时 GUI App 可用，Terminal CLI agent 捕获弱。
只开 Input Monitoring 时 Terminal 较好，GUI 无 AX 快照。

## 二进制路径

编译 release：

```bash
cd /path/to/prompt-saviour
cargo build --release
```

授权目标：

```
/path/to/prompt-saviour/target/release/prompt-saviour
```

Debug 构建（`cargo run`）是**另一个文件**：

```
/path/to/prompt-saviour/target/debug/prompt-saviour
```

**release 与 debug 需分别授权。**

## 步骤

### 1. 触发授权弹窗（可选）

```bash
./target/release/prompt-saviour doctor
```

可能弹出系统对话框。
若未弹出，继续手动添加。

### 2. 手动添加 - 辅助功能

1. 打开 **系统设置 → 隐私与安全性 → 辅助功能**
2. 点击 **+**（若列表锁定，先点 🔒 解锁）
3. 按 **⌘⇧G**，粘贴二进制完整路径
4. 勾选启用
5. 若之前拒绝过，需先移除再重新添加

### 3. 手动添加 - 输入监控

1. **系统设置 → 隐私与安全性 → 输入监控**
2. 同样 **+** → **⌘⇧G** → 选择同一 `prompt-saviour` 二进制
3. 勾选启用
4. **可能需要重启 daemon** 或重新运行 `prompt-saviour run`

### 4. 验证

```bash
./target/release/prompt-saviour doctor
# 应显示 granted

./target/release/prompt-saviour run
# 在 Cursor 输入 8+ 字符，等 ~1 秒

./target/release/prompt-saviour list
```

## GUI App 阶段（规划）

安装 `.app` 后，授权对象为：

```
/Applications/Prompt Saviour.app
```

GUI **权限与系统** 页将展示：

- 实时 ✅ / ❌ 状态
- 一键打开对应系统设置 pane
- Copy Path 按钮
- 授权后 Refresh

无需用户记忆 binary 路径。

## Automation（可选）

`scripts/e2e_macos.sh` 用 AppleScript 向 TextEdit 模拟按键。
需要给 **Terminal / Cursor / osascript** 的 Automation 权限。
**日常使用 Prompt Saviour 不需要此权限。**

## 故障排除

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| `doctor` 仍 missing | 授权了错误路径（debug vs release） | 核对路径 |
| `list` 始终空 | 未运行 `run` 或权限缺失 | 先 `run`，再查权限 |
| AX 有、Terminal 无 | 未开 Input Monitoring | 开输入监控 |
| 换机器 clone 后失效 | 新 binary 路径 | 重新添加 |

## 无权限自检

以下命令**不需要**任何系统权限：

```bash
prompt-saviour smoke --text "test prompt content here"
prompt-saviour list
bash scripts/e2e_macos.sh   # smoke + inject 段
```
