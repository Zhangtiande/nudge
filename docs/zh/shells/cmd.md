# CMD 指南

[English](../../en/shells/cmd.md) | [中文](cmd.md)

CMD（命令提示符）通过 doskey 宏提供基本的手动补全。它是功能最有限的集成路径。

## 模式

| 模式 | 触发方式 | 描述 |
|---|---|---|
| `cmd-inline` | `nudge-complete <command>` | 通过 doskey 宏的手动补全 |

## 快速开始

```cmd
REM Load integration (or add to AutoRun registry key)
path\to\integration.cmd

REM Get a completion suggestion
nudge-complete git st
```

## 设置

CMD 不支持 `nudge setup`。你需要手动加载集成脚本。

### 方式一：每次会话手动加载

```cmd
%USERPROFILE%\.nudge\shell\integration.cmd
```

### 方式二：AutoRun 注册表（持久化）

将脚本添加到 CMD 的 AutoRun 注册表键，使其在每次 CMD 启动时自动加载：

```cmd
reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_SZ /d "%USERPROFILE%\.nudge\shell\integration.cmd" /f
```

移除：

```cmd
reg delete "HKCU\Software\Microsoft\Command Processor" /v AutoRun /f
```

## 可用命令

集成脚本创建以下 doskey 宏：

| 宏 | 描述 |
|---|---|
| `nudge-complete <args>` | 获取给定命令前缀的补全 |
| `nudge-suggest <args>` | `nudge-complete` 的别名 |
| `nudge-start` | 启动守护进程 |
| `nudge-stop` | 停止守护进程 |
| `nudge-status` | 检查守护进程状态 |

### 示例

```cmd
nudge-complete docker run
REM Outputs: docker run -it --rm ubuntu:latest /bin/bash
```

## 与其他 Shell 的差异

- **无快捷键绑定**：CMD 不支持 readline 风格的按键绑定。你必须显式输入 `nudge-complete` 命令。
- **无内联替换**：建议以输出形式打印。你需要复制粘贴或重新输入。
- **无自动模式**：没有幽灵文字或实时建议。
- **无诊断功能**：CMD 中不提供错误诊断。

## 守护进程自动启动

集成脚本在首次加载时检查守护进程是否正在运行，如果需要则启动它：

```cmd
nudge status >nul 2>&1
if errorlevel 1 (
    start /b nudge start >nul 2>&1
)
```

在交互式会话中（当 `PROMPT` 已定义时）会显示 "Nudge loaded" 消息。

## 已知限制

- 没有真正的快捷键支持 -- 需要输入 `nudge-complete` 命令
- 没有内联命令替换 -- 输出必须手动使用
- 没有 stderr 捕获或错误诊断
- 没有弹出选择器或多候选项显示
- 会话跟踪使用 `%RANDOM%`，唯一性有限
- Doskey 宏作用域限于当前会话（CMD 关闭后丢失，除非使用 AutoRun）

## 边界

- CMD 集成设计上是精简的
- 如需更好的 Windows 体验，请使用 PowerShell
- 如果需要 Windows 上的自动模式，需要 PowerShell 7.2+
