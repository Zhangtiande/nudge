# CLI 参考

[English](../en/cli-reference.md) | [中文](cli-reference.md)

Nudge 日常操作的实用命令参考。

## 快速示例

```bash
nudge status          # Is the daemon running?
nudge info            # Show config summary
nudge doctor zsh      # Check Zsh integration health
nudge restart         # Restart daemon with latest config
```

## 命令

### `nudge daemon [--foreground|--fork]`

启动后台 daemon，通过 IPC 处理补全请求。

- `--foreground`：在当前终端中运行（适合配合 `RUST_LOG=debug` 进行调试）
- `--fork`：后台运行（通过 `nudge start` 启动时的默认行为）

### `nudge start`

以后台模式启动 daemon。等同于 `nudge daemon --fork`。

### `nudge stop`

通过发送关闭信号停止正在运行的 daemon。

### `nudge restart`

停止然后重新启动 daemon。在配置更改后使用。

### `nudge status`

输出 daemon 是否正在运行及其 PID。

### `nudge complete`

向 daemon 请求补全。此命令由 Shell 集成脚本调用；通常你不需要直接使用它。

```bash
nudge complete \
  --buffer "git st" \
  --cursor 6 \
  --cwd /path/to/project \
  --session "zsh-12345" \
  --shell-mode "zsh-inline" \
  --format plain
```

**参数**：

| 标志 | 必需 | 描述 |
|---|---|---|
| `--buffer` | 是 | 当前命令行内容 |
| `--cursor` | 是 | 光标位置（字节偏移） |
| `--cwd` | 是 | 工作目录 |
| `--session` | 是 | 会话标识符，用于保持连续性 |
| `--shell-mode` | 否 | 集成脚本的提示：`zsh-inline`、`zsh-auto`、`bash-inline`、`bash-popup`、`ps-inline`、`cmd-inline` |
| `--format` | 否 | 输出格式（见下文） |
| `--last-exit-code` | 否 | 上一条命令的退出码 |

**输出格式**（`--format`）：

| 格式 | 输出 | 使用场景 |
|---|---|---|
| `plain` | 单条建议字符串 | 内联应用（Ctrl+E 路径） |
| `list` | Tab 分隔的行：`risk\tcommand\twarning\twhy\tdiff` | 弹出选择器（Alt+/ 路径） |
| `json` | 包含 `suggestion`、`warning`、`candidates` 的 JSON 对象 | 程序化调用 |

### `nudge info [--json] [--field <name>]`

显示当前 Nudge 安装的运行时信息。

- `--json`：以 JSON 对象输出
- `--field <name>`：打印单个字段的值

**常用 `--field` 键**：

| 键 | 返回值 |
|---|---|
| `config_dir` | 配置目录路径 |
| `config_file` | 用户配置文件路径 |
| `default_config_file` | 默认配置文件路径 |
| `socket_path` | IPC socket/pipe 路径 |
| `integration_script` | Shell 集成脚本路径 |
| `daemon_status` | `running` 或 `stopped` |
| `shell_type` | 检测到的 Shell 类型 |
| `trigger_mode` | `manual` 或 `auto` |
| `trigger_hotkey` | 当前快捷键绑定 |
| `zsh_ghost_owner` | `auto`、`nudge` 或 `autosuggestions` |
| `zsh_overlay_backend` | `message` 或 `rprompt` |
| `diagnosis_enabled` | `true` 或 `false` |
| `interactive_commands` | 逗号分隔的列表 |

### `nudge doctor [zsh|bash]`

针对特定 Shell 运行集成健康检查。

```bash
nudge doctor zsh
```

**检查内容**：

- Shell 集成脚本已被加载
- 快捷键绑定已注册（`Ctrl+E`、`Tab`、`Alt+/` 等）
- Hook 已安装（`precmd`、`preexec` 等）
- Daemon 可达
- 配置值一致

**解读输出**：每项检查会输出 `OK` 或 `WARN` 以及简短说明。如果看到警告，运行 `nudge setup <shell> --force` 刷新集成。

### `nudge setup [bash|zsh|powershell] [--force]`

写入 Shell 集成脚本并将 `source` hook 添加到你的配置文件。

- `bash`：写入 `.bashrc`
- `zsh`：写入 `.zshrc`
- `powershell`：写入 PowerShell `$PROFILE`
- `--force`：覆盖已有的集成文件

### `nudge diagnose`

分析失败的命令并建议修复方案。当 `diagnosis.enabled: true` 时，由 Shell 集成脚本自动调用。

```bash
nudge diagnose \
  --exit-code 1 \
  --command "cargo build" \
  --cwd /path/to/project \
  --session "zsh-12345" \
  --stderr-file /tmp/nudge_stderr_12345 \
  --format plain
```

**参数**：

| 标志 | 必需 | 描述 |
|---|---|---|
| `--exit-code` | 是 | 失败命令的退出码 |
| `--command` | 是 | 失败的命令 |
| `--cwd` | 是 | 工作目录 |
| `--session` | 是 | 会话标识符 |
| `--stderr-file` | 否 | 捕获的 stderr 文件路径 |
| `--error-record` | 否 | JSON 错误记录（PowerShell） |
| `--format` | 否 | 输出格式：`plain` 或 `json` |

**输出**（plain 格式）：两行 — 第一行是诊断信息，第二行是建议的修复命令。Shell 集成会显示诊断结果，并允许你按 Tab 接受修复建议。

## 典型工作流

**初始设置检查**：

```bash
nudge setup zsh --force
nudge restart
nudge status
nudge info
nudge doctor zsh
```

**调试 daemon 行为**：

```bash
RUST_LOG=debug nudge daemon --foreground
# In another terminal, trigger completion to see logs
```

**检查缓存行为**：

```bash
RUST_LOG=debug nudge daemon --foreground
# Look for "cache hit" / "cache miss" / "stale-revalidate" in output
```

## 边界

- `nudge complete` 是集成/内部路径；正常使用时请优先使用 Shell 快捷键
- `--shell-mode` 是集成脚本提供的提示；不要依赖未文档化的值
- `nudge diagnose` 的输出格式可能会在不同版本间发生变化
