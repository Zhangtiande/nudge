# 配置参考

[English](../en/configuration.md) | [中文](configuration.md)

本文档说明如何在不修改源代码的情况下配置 Nudge。

## 配置文件

| 文件 | 用途 |
|---|---|
| `~/.nudge/config/config.default.yaml` | 随附的默认配置。升级时自动更新。**请勿编辑。** |
| `~/.nudge/config/config.yaml` | 你的自定义覆盖。升级时会保留。 |

环境变量覆盖：设置 `NUDGE_CONFIG` 指向自定义配置路径。

加载顺序：内置 Rust 默认值 → `config.default.yaml` → `config.yaml`。用户的覆盖通过深度合并优先生效。

## 最小配置

本地模型（Ollama）：

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

trigger:
  mode: manual
```

远程模型（OpenAI）：

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

## 完整配置架构

### `model` — LLM 连接

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `endpoint` | string | `http://localhost:11434/v1` | OpenAI 兼容的 API 端点 |
| `model_name` | string | `codellama:7b` | 模型标识符 |
| `api_key` | string | _(无)_ | API 密钥（直接指定，优先于环境变量） |
| `api_key_env` | string | _(无)_ | 存储 API 密钥的环境变量名称 |
| `timeout_ms` | int | `5000` | 请求超时时间（毫秒） |

### `context` — 发送给 LLM 的上下文

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `history_window` | int | `20` | 最近历史记录条目数 |
| `include_cwd_listing` | bool | `true` | 包含当前目录的文件列表 |
| `include_exit_code` | bool | `true` | 包含上一条命令的退出码 |
| `include_system_info` | bool | `true` | 包含操作系统、架构、Shell 类型 |
| `similar_commands_enabled` | bool | `true` | 在历史记录中搜索相似命令 |
| `similar_commands_window` | int | `200` | 相似性搜索的历史深度 |
| `similar_commands_max` | int | `5` | 返回的最大相似命令数 |
| `max_files_in_listing` | int | `50` | 目录列表中的最大文件数 |
| `max_total_tokens` | int | `4000` | 所有上下文的 token 预算 |
| `priorities.history` | int | `80` | 历史记录的优先级权重 |
| `priorities.cwd_listing` | int | `60` | 目录列表的优先级权重 |
| `priorities.plugins` | int | `40` | 插件输出的优先级权重 |

**上下文截断**：当总上下文超过 `max_total_tokens` 时，优先级较低的项目会先被移除。Token 估算使用基于单词的启发式方法，乘以 1.3 倍系数。

### `plugins` — 项目上下文

每个插件遵循相同的模式：`enabled`、`timeout_ms`、可选的 `priority` 以及插件特定设置。

#### `plugins.git`

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `true` | 启用 Git 上下文 |
| `depth` | string | `standard` | `light`（仅分支）、`standard`（+暂存/未暂存）、`detailed`（+提交记录） |
| `recent_commits` | int | `5` | `detailed` 模式下显示的提交数 |

Git 操作有严格的 50ms 内部超时，以防止阻塞。

#### `plugins.docker`

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `true` | 启用 Docker 上下文 |
| `timeout_ms` | int | `100` | 命令执行超时 |
| `max_containers` | int | `10` | 上下文中的最大容器数 |
| `max_images` | int | `10` | 上下文中的最大镜像数 |
| `show_containers` | bool | `true` | 包含正在运行的容器 |
| `include_compose` | bool | `true` | 包含 docker-compose 服务 |
| `include_dockerfile` | bool | `true` | 包含 Dockerfile 预览（前 50 行） |

#### `plugins.node`

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `true` | 启用 Node.js 上下文 |
| `timeout_ms` | int | `100` | 文件操作超时 |
| `max_dependencies` | int | `50` | 列出的最大依赖数 |

读取 `package.json` 获取脚本和依赖信息。

#### `plugins.rust`

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `true` | 启用 Rust/Cargo 上下文 |
| `timeout_ms` | int | `100` | 文件操作超时 |
| `max_dependencies` | int | `50` | 列出的最大依赖数 |

读取 `Cargo.toml` 获取工作区和依赖信息。

#### `plugins.python`

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `true` | 启用 Python 上下文 |
| `timeout_ms` | int | `100` | 文件操作超时 |
| `max_dependencies` | int | `50` | 列出的最大依赖数 |

读取 `pyproject.toml`、`requirements.txt`，并检测虚拟环境（uv、poetry、pip）。

### `trigger` — 补全触发方式

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `mode` | string | `manual` | `manual` 或 `auto` |
| `hotkey` | string | `\C-e` | 手动触发的 Readline 格式快捷键 |
| `auto_delay_ms` | int | `500` | 自动模式的防抖延迟 |
| `zsh_ghost_owner` | string | `auto` | `auto`、`nudge` 或 `autosuggestions` |
| `zsh_overlay_backend` | string | `message` | `message` 或 `rprompt` |

详见 [自动模式指南](auto-mode.md) 以了解 ghost ownership 和 overlay backend 的详细说明。

### `cache` — 建议缓存

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `capacity` | int | `1024` | 最大缓存条目数（LRU 淘汰） |
| `prefix_bytes` | int | `80` | 用于键哈希的命令前缀最大字节数 |
| `ttl_auto_ms` | int | `300000` | 自动模式条目的 TTL（5 分钟） |
| `ttl_manual_ms` | int | `600000` | 手动模式条目的 TTL（10 分钟） |
| `ttl_negative_ms` | int | `30000` | 失败/空结果的 TTL（30 秒） |
| `stale_ratio` | float | `0.8` | Stale-while-revalidate 阈值（0.0–1.0） |

**缓存键**：`sk:v1:{prefix_hash}:{cwd_hash}:{git_hash}:{shell_mode}`。任何上下文变化（目录、Git 状态）都会自动使相关条目失效。

**Stale-while-revalidate**：当条目达到 `stale_ratio x TTL` 的存活时间时，会立即返回该条目，同时在后台触发刷新。这在不长时间提供过期数据的前提下，提供了低延迟响应。

### `privacy` — 脱敏与安全

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `sanitize_enabled` | bool | `true` | 在调用 LLM 前从上下文中移除敏感信息 |
| `custom_patterns` | list | `[]` | 额外的脱敏正则表达式 |
| `block_dangerous` | bool | `true` | 阻止危险命令（rm -rf、fork bomb 等） |
| `custom_blocked` | list | `[]` | 额外的危险命令匹配模式 |

### `log` — 日志

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `level` | string | `info` | `trace`、`debug`、`info`、`warn`、`error` |
| `file_enabled` | bool | `false` | 将日志写入 `~/.nudge/logs/`，按天轮转 |

### `diagnosis` — 错误诊断

| 键 | 类型 | 默认值 | 描述 |
|---|---|---|---|
| `enabled` | bool | `false` | 启用失败命令诊断 |
| `capture_stderr` | bool | `true` | 捕获 stderr 用于分析（Zsh） |
| `auto_suggest` | bool | `true` | 显示内联修复建议（按 Tab 接受） |
| `max_stderr_size` | int | `4096` | 发送给 LLM 的最大 stderr 字节数 |
| `timeout_ms` | int | `5000` | 诊断请求超时 |
| `interactive_commands` | list | _(见下文)_ | 跳过 stderr 捕获的命令 |

默认 `interactive_commands`：vim、nvim、vi、nano、emacs、code、ssh、telnet、mosh、top、htop、btop、less、more、man、fzf、sk、tmux、screen、python、python3、ipython、node、irb、psql、mysql、sqlite3、watch、tail。

### `system_prompt` — 自定义 LLM 提示词

覆盖发送给 LLM 的默认系统提示词：

```yaml
system_prompt: |
  You are a helpful command-line assistant.
  Suggest commands that are safe and follow best practices.
  Always explain what the command does if it's complex.
```

## 实用配置方案

**延迟优先** — 最小化上下文收集：

```yaml
context:
  history_window: 10
  include_cwd_listing: false
plugins:
  docker:
    enabled: false
cache:
  ttl_manual_ms: 900000  # 15 min cache
```

**安全优先** — 最大化保护：

```yaml
privacy:
  sanitize_enabled: true
  block_dangerous: true
diagnosis:
  enabled: true
  capture_stderr: true
```

**节省带宽** — 本地模型配合激进缓存：

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
cache:
  capacity: 2048
  ttl_auto_ms: 600000  # 10 min
context:
  max_total_tokens: 2000
```

## 验证与观察

```bash
nudge info --json          # Full config dump
nudge info --field trigger_mode
nudge doctor zsh           # Integration health check
RUST_LOG=debug nudge daemon --foreground  # Watch cache hits/misses
```

## 边界

- Bash/CMD 仅通过配置无法获得真正的自动 ghost-text 模式
- 过于激进的自定义正则表达式可能会隐藏有用的上下文
- `system_prompt` 会替换整个默认提示词 — 请包含你需要的所有指令
- 缓存仅存在于内存中；daemon 重启时会被清空
