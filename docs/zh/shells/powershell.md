# PowerShell 指南

[English](../../en/shells/powershell.md) | [中文](powershell.md)

PowerShell 提供通过 `Ctrl+E` 的手动补全、失败命令的错误诊断，以及通过 PSReadLine predictor 的实验性自动模式（PowerShell 7.2+）。

## 模式

| 模式 | 触发方式 | 描述 |
|---|---|---|
| `ps-inline` | `Ctrl+E` | 手动单候选项补全 |

## 快速开始

```powershell
# Set up integration
nudge setup powershell --force
nudge restart

# Open a new PowerShell window, then:
# Type a partial command, press Ctrl+E
git st<Ctrl+E>
# → git status
```

## 设置

```powershell
nudge setup powershell --force
```

这会在 PowerShell 的 `$PROFILE` 中添加一行 `source` 命令，使每次会话都加载 `integration.ps1`。

验证：

```powershell
nudge status
# Should show: Running (pid: ...)
```

加载时，你应该看到：`Nudge loaded (manual mode).` 或 `Nudge loaded (manual mode + error diagnosis).`

## PSReadLine Predictor（自动模式）

PowerShell 7.2+ 配合 PSReadLine 2.2.0+ 支持通过 `NudgePredictor` 模块的实验性自动模式。

### 要求

- PowerShell 7.2 或更高版本
- PSReadLine 2.2.0 或更高版本

### 工作原理

1. `NudgePredictor` 注册为 PSReadLine 预测源
2. 输入时，PSReadLine 从所有已注册的预测源请求预测
3. Nudge 返回 LLM 驱动的建议，与内置历史记录预测一同显示
4. 建议以内联灰色文字形式出现（PSReadLine 的 `InlineView`）

### 启用

```yaml
# ~/.nudge/config/config.yaml
trigger:
  mode: auto
```

```powershell
nudge restart
# Reopen PowerShell
```

如果不满足自动模式的要求，集成脚本会自动回退到手动模式并显示警告。

## 错误诊断

当 `diagnosis.enabled: true` 时，PowerShell 集成会捕获失败的命令并提供修复建议。

### 工作原理

1. 每个命令执行后会运行一个提示符钩子
2. 它检查 `$LASTEXITCODE`（外部命令）和 `$Error`（PowerShell 异常）
3. 检测到失败时，调用 `nudge diagnose` 并传递错误详情
4. 诊断以黄色文字显示，附带建议的修复方案
5. 按 `Tab` 接受建议的修复并插入命令行

### 配置

```yaml
diagnosis:
  enabled: true
```

### 捕获的内容

| 错误来源 | 检测方式 | 发送内容 |
|---|---|---|
| 外部命令失败 | `$LASTEXITCODE ≠ 0` | 来自历史记录的命令 + 退出码 |
| PowerShell 异常 | `$Error` 计数增加 | 异常消息 + 脚本堆栈跟踪 + 类别 |

### 交互式命令排除

与 Zsh 相同 -- 交互式命令（vim、ssh 等）会被排除在诊断之外。检查和扩展：

```powershell
nudge info --field interactive_commands
```

### Tab 接受

当诊断建议可用时，`Tab` 会插入该建议。否则，`Tab` 回退到默认的 PowerShell Tab 补全。

## 快捷键

| 按键 | 操作 |
|---|---|
| `Ctrl+E` | 手动补全 |
| `Tab` | 接受诊断建议（如果可用），否则执行默认 Tab 补全 |

## Profile 故障排查

### Profile 未加载

检查 profile 是否存在且包含 Nudge 的 source 行：

```powershell
Test-Path $PROFILE
Get-Content $PROFILE | Select-String "nudge"
```

如果缺失，重新运行设置：

```powershell
nudge setup powershell --force
```

### 多个 Profile

PowerShell 有多个 profile 路径（CurrentUserCurrentHost、CurrentUserAllHosts 等）。Nudge 写入 `$PROFILE`，即 `CurrentUserCurrentHost`。如果你使用不同的 profile，请手动复制 source 行。

### 执行策略

如果脚本被阻止，你可能需要：

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Windows 环境说明

- IPC 使用命名管道（`\\.\pipe\nudge_{username}`）而非 Unix 套接字
- 守护进程通过 `Start-Process` 以 `-WindowStyle Hidden` 启动
- 配置路径：`%USERPROFILE%\.nudge\config\`
- 日志路径：`%USERPROFILE%\.nudge\logs\`

## 故障排查

| 症状 | 可能原因 | 修复方法 |
|---|---|---|
| 缺少 "Nudge loaded" 消息 | Profile 未加载 | `nudge setup powershell --force` |
| `Ctrl+E` 无响应 | 按键处理器未注册 | 检查 `Get-PSReadLineKeyHandler \| Where-Object Key -eq 'Ctrl+e'` |
| 诊断未显示 | `diagnosis.enabled` 为 false | 设置为 `true` 并重启 |
| 加载时出现自动模式警告 | PS 版本 < 7.2 或 PSReadLine < 2.2.0 | 升级或使用手动模式 |
| 守护进程无法启动 | 权限问题 | 检查 `nudge status` 和 Windows 事件日志 |

## 边界

- 完整的自动幽灵文字模式需要 PowerShell 7.2+ 和 PSReadLine 2.2.0+
- 没有弹出选择器（仅 Bash 支持此功能）
- 行为取决于 PowerShell 宿主能力（ISE vs Console vs Windows Terminal）
- PSReadLine predictor 不支持 Zsh 中可用的 overlay 显示模式
