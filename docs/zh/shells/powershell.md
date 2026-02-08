# PowerShell 指南

[English](../../en/shells/powershell.md) | [中文](powershell.md)

PowerShell 支持手动补全（`Ctrl+E`）、多候选 popup 选择器、失败命令诊断，以及基于 PSReadLine predictor 的实验性自动模式（PowerShell 7.2+）。

## 模式

| 模式 | 触发方式 | 描述 |
|---|---|---|
| `ps-inline` | `Ctrl+E` | 快速单候选补全 |
| `ps-popup` | 默认 `Alt+/` | 热键触发的多候选选择器 |

## 快速开始

```powershell
# Set up integration
nudge setup powershell --force
nudge restart

# Open a new PowerShell window, then:
# 单候选
git st<Ctrl+E>
# → git status

# 多候选 popup
docker<Alt+/>
# → 打开选择器（fzf/sk/peco/ConsoleGrid/GridView/builtin）
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

加载后你会看到类似：

- `Nudge loaded (manual mode (Ctrl+E) + popup (Alt+/)).`
- `Nudge loaded (auto mode + popup (Alt+/) + error diagnosis).`

## Popup 选择器

`ps-popup` 是纯手动模式：不会自动触发，只在按下热键时拉取多候选。

### 工作原理

1. 按 popup 热键（默认 `Alt+/`）
2. 集成脚本调用 `nudge complete --format list --shell-mode ps-popup`
3. daemon 返回 tab 分隔的候选行
4. 集成脚本用选择器后端展示并将所选命令写回命令行

### 行格式

```
risk<TAB>command<TAB>warning<TAB>why<TAB>diff
```

| 字段 | 说明 |
|---|---|
| `risk` | 风险等级：`low`、`high` |
| `command` | 建议命令 |
| `warning` | 高风险命令的安全提示 |
| `why` | 简短解释 |
| `diff` | 与输入相比的变化 |

### 选择器后端

通过 `NUDGE_PS_POPUP_BACKEND`（或通用回退 `NUDGE_POPUP_BACKEND`）配置：

| 取值 | 说明 |
|---|---|
| `auto`（默认） | 自动探测：`fzf` → `sk` → `peco` → `Out-ConsoleGridView` → `Out-GridView` → builtin |
| `fzf` | 使用 [fzf](https://github.com/junegunn/fzf) |
| `sk` | 使用 [skim](https://github.com/lotabout/skim) |
| `peco` | 使用 [peco](https://github.com/peco/peco) |
| `consolegrid` | 使用 `Out-ConsoleGridView` |
| `gridview` | 使用 `Out-GridView` |
| `builtin` | 内置编号列表兜底 |

### 高风险确认

高风险建议默认需要确认。

```powershell
# 关闭高风险确认（不推荐）
$env:NUDGE_PS_POPUP_CONFIRM_RISKY = "0"
```

## Popup 环境变量

PowerShell 专用变量（部分支持通用 `NUDGE_POPUP_*` 回退）：

| 变量 | 默认值 | 说明 |
|---|---|---|
| `NUDGE_PS_POPUP_ENABLED` | `1` | 开关 popup 热键 |
| `NUDGE_PS_POPUP_KEY` | `Alt+/` | popup 热键 chord |
| `NUDGE_PS_POPUP_BACKEND` | `auto` | 选择器后端 |
| `NUDGE_PS_POPUP_HEIGHT` | `70%` | 选择器高度（`fzf`/`sk`） |
| `NUDGE_PS_POPUP_CONFIRM_RISKY` | `1` | 应用高风险候选前是否确认 |

如果你在 macOS 上按 `Option+/` 会输入 `÷`，可改为：

```powershell
$env:NUDGE_PS_POPUP_KEY = "Ctrl+/"
# 如果当前宿主不识别 Ctrl+/，再试 "Ctrl+_"。
```

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

如果不满足自动模式要求，集成脚本会自动回退到手动模式并显示警告。

## 错误诊断

当 `diagnosis.enabled: true` 时，PowerShell 集成会捕获失败命令并提供修复建议。

### 工作原理

1. 每个命令执行后运行 prompt hook
2. 检查 `$LASTEXITCODE`（外部命令）和 `$Error`（PowerShell 异常）
3. 失败时调用 `nudge diagnose` 并传递错误上下文
4. 以黄色文字显示诊断和建议
5. 按 `Tab` 将建议命令写入当前命令行

### 配置

```yaml
diagnosis:
  enabled: true
```

### 捕获内容

| 错误来源 | 检测方式 | 发送内容 |
|---|---|---|
| 外部命令失败 | `$LASTEXITCODE ≠ 0` | 历史命令 + 退出码 |
| PowerShell 异常 | `$Error` 计数增加 | 异常消息 + 脚本栈 + 类别 |

### 交互式命令排除

与 Zsh 一致：交互式命令（vim、ssh 等）不会进入诊断。可通过以下命令查看和扩展：

```powershell
nudge info --field interactive_commands
```

### Tab 接受

当有诊断建议时，`Tab` 会插入建议；否则回退到默认 PowerShell Tab 补全。

## 快捷键

| 按键 | 操作 |
|---|---|
| `Ctrl+E` | 手动单候选补全 |
| 默认 `Alt+/` | popup 多候选选择 |
| `Tab` | 接受诊断建议（如有），否则执行默认 Tab 补全 |

## Profile 故障排查

### Profile 未加载

检查 profile 是否存在并包含 Nudge source 行：

```powershell
Test-Path $PROFILE
Get-Content $PROFILE | Select-String "nudge"
```

如果缺失，重新运行：

```powershell
nudge setup powershell --force
```

### 多 Profile

PowerShell 有多个 profile 路径（CurrentUserCurrentHost、CurrentUserAllHosts 等）。Nudge 写入 `$PROFILE`（CurrentUserCurrentHost）。如果你使用其他 profile，请手动复制 source 行。

### 执行策略

脚本被阻止时可能需要：

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Windows 环境说明

- IPC 使用命名管道（`\\.\pipe\nudge_{username}`），不是 Unix socket
- daemon 通过 `Start-Process` 且使用 `-WindowStyle Hidden` 启动
- 配置路径：`%USERPROFILE%\.nudge\config\`
- 日志路径：`%USERPROFILE%\.nudge\logs\`

## 故障排查

| 症状 | 可能原因 | 修复 |
|---|---|---|
| 没有 "Nudge loaded" 提示 | Profile 未加载 | `nudge setup powershell --force` |
| `Ctrl+E` 无响应 | 快捷键未注册 | `Get-PSReadLineKeyHandler \| Where-Object Key -eq 'Ctrl+e'` |
| popup 热键无响应 | popup 关闭或 chord 冲突 | 检查 `NUDGE_PS_POPUP_ENABLED`、`NUDGE_PS_POPUP_KEY` 和 `Get-PSReadLineKeyHandler` |
| popup 无候选 | daemon 不可达或上下文不足 | `nudge status` 后重试 |
| 诊断未显示 | `diagnosis.enabled` 为 false | 设为 `true` 并重启 |
| 加载提示 auto mode 警告 | PS < 7.2 或 PSReadLine < 2.2.0 | 升级或继续用手动模式 |
| daemon 启动失败 | 权限问题 | 检查 `nudge status` 与 Windows Event Log |

## 边界

- 完整 auto ghost-text 依赖 PowerShell 7.2+ 和 PSReadLine 2.2.0+
- popup 体验取决于可用选择器后端
- 行为依赖 PowerShell 宿主能力（ISE / Console / Windows Terminal）
- PSReadLine predictor 不支持 Zsh 的 overlay 显示模式
