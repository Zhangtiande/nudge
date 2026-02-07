# Zsh 指南

[English](../../en/shells/zsh.md) | [中文](zsh.md)

Zsh 是最完整的集成路径 -- 支持手动模式、自动幽灵文字、overlay 以及错误诊断。

## 模式

| 模式 | 触发方式 | 描述 |
|---|---|---|
| `zsh-inline` | `Ctrl+E` | 快速单候选项应用（始终可用） |
| `zsh-auto` | 输入时 | 实时幽灵文字或 overlay 建议 |

## 快速开始

手动模式（开箱即用）：

```bash
# Type a partial command, then press Ctrl+E
git st<Ctrl+E>
# → git status
```

启用自动模式：

```yaml
# ~/.nudge/config/config.yaml
trigger:
  mode: auto
```

```bash
nudge restart
```

## 幽灵文字所有权

`trigger.zsh_ghost_owner` 设置控制由谁渲染灰色建议文字。

| 值 | 幽灵文字由 | Nudge 显示方式 | 接受键 |
|---|---|---|---|
| `auto` | autosuggestions（如果存在），否则 Nudge | Overlay 或幽灵文字 | `Tab` 或 `Ctrl+G` |
| `nudge` | Nudge | 幽灵文字 | `Tab` |
| `autosuggestions` | zsh-autosuggestions | Overlay | `Ctrl+G` |

### `auto`（默认）

在加载时检测 `zsh-autosuggestions`：
- 存在 → Nudge 使用 overlay；幽灵文字由 autosuggestions 管理
- 不存在 → Nudge 接管幽灵文字所有权

### `nudge`

强制 Nudge 拥有幽灵文字，覆盖 autosuggestions。

```yaml
trigger:
  zsh_ghost_owner: nudge
```

### `autosuggestions`

将幽灵文字保留给 autosuggestions，Nudge 始终使用 overlay。

```yaml
trigger:
  zsh_ghost_owner: autosuggestions
```

## Overlay 后端

当 Nudge 使用 overlay 模式（不拥有幽灵文字）时，有两种渲染后端可用：

| 后端 | 方法 | 行为 |
|---|---|---|
| `message`（默认） | `zle -M` | 提示符下方的消息行；下次按键时清除 |
| `rprompt` | `RPS1` | 右侧提示符；持续显示直到下一个建议 |

```yaml
trigger:
  zsh_overlay_backend: rprompt  # or message
```

## 快捷键

| 按键 | 操作 | 条件 |
|---|---|---|
| `Ctrl+E` | 手动补全（立即发送 LLM 请求） | 始终 |
| `Tab` | 接受完整的幽灵建议 | Nudge 拥有幽灵文字时 |
| `Right Arrow` | 接受下一个单词 | Nudge 拥有幽灵文字时 |
| `Ctrl+G` | 接受 overlay 建议 | Overlay 模式 |
| `F1` | 切换解释详情（why/diff/risk） | 始终 |

## 错误诊断

当 `diagnosis.enabled: true` 时，Zsh 集成会自动分析失败的命令。

### 工作原理

1. 命令执行前（`preexec`）：将 stderr 重定向到临时文件
2. 命令执行后（`precmd`）：如果退出码不为 0，Nudge 将命令和捕获的 stderr 发送给 LLM
3. 诊断消息显示在提示符下方
4. 建议的修复以内联文字形式显示 -- 按 `Tab` 接受

### 配置

```yaml
diagnosis:
  enabled: true
  capture_stderr: true
  auto_suggest: true
  max_stderr_size: 4096
```

### 交互式命令排除

stderr 捕获会影响交互式程序（vim、ssh、top）。这些命令默认通过 `diagnosis.interactive_commands` 排除。在配置中添加自定义排除项：

```yaml
diagnosis:
  interactive_commands:
    - vim
    - nvim
    - ssh
    - your-custom-tool
```

检查当前列表：

```bash
nudge info --field interactive_commands
```

## 健康检查：`nudge doctor zsh`

```bash
nudge doctor zsh
```

执行的检查：

| 检查项 | 验证内容 |
|---|---|
| Integration sourced | `integration.zsh` 已加载 |
| Key bindings | `Ctrl+E`、`Tab`、`F1` 已注册 |
| Hooks | `precmd`、`preexec` 钩子已安装 |
| Daemon | 守护进程可通过 IPC 访问 |
| Ghost owner | 配置与运行时状态匹配 |
| Overlay backend | 后端功能正常 |

每项检查会输出 `OK` 或 `WARN`。修复警告：

```bash
nudge setup zsh --force
nudge restart
```

## 故障排查

| 症状 | 可能原因 | 修复方法 |
|---|---|---|
| 完全没有建议 | 守护进程未运行 | `nudge start` |
| `Ctrl+E` 无响应 | 集成脚本未加载 | `nudge setup zsh --force && source ~/.zshrc` |
| 幽灵文字闪烁 | 终端重绘冲突 | 尝试 `zsh_overlay_backend: rprompt` |
| 建议覆盖了 autosuggestions | 幽灵文字所有权冲突 | 设置 `zsh_ghost_owner: autosuggestions` |
| `F1` 无响应 | 终端将 F1 映射为帮助 | 检查终端按键设置 |
| 诊断未显示 | `diagnosis.enabled` 为 false | 设置为 `true` 并重启 |
| 交互式工具的 stderr 丢失 | 未在排除列表中 | 添加到 `interactive_commands` |
| cd 后建议过时 | 缓存未失效 | 应自动失效；使用 `RUST_LOG=debug` 检查 |

## 边界

- Overlay 密度取决于终端宽度
- 功能键映射（`F1`）在不同终端模拟器中可能有差异
- stderr 捕获是 Zsh 特有的功能，在其他 shell 中不可用
- 幽灵文字需要终端支持 ANSI 颜色
