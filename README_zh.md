# Nudge

> 给你的终端一个温柔的提示 - LLM 驱动的命令行自动补全

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

---

Nudge 使用大语言模型，根据你的 Shell 历史记录、当前目录上下文和 Git 仓库状态来预测和补全命令行输入。

## ✨ 功能特性

| 功能 | 描述 |
|------|------|
| 🤖 **AI 智能补全** | 使用 LLM 理解上下文，提供相关命令建议 |
| 📝 **历史感知** | 从 Shell 历史记录中学习，提供个性化建议 |
| 🔍 **相似指令搜索** | 自动从历史记录中查找相似命令（类似 Bash Ctrl+R） |
| 🖥️ **系统感知** | 根据您的操作系统、架构和 Shell 类型调整建议 |
| 📁 **上下文感知** | 考虑当前目录文件和 Git 状态 |
| 🔒 **隐私优先** | 发送给 LLM 前自动清理敏感数据（API 密钥、密码等） |
| ⚠️ **安全警告** | 标记潜在危险命令（rm -rf、mkfs 等） |
| 🐚 **多 Shell 支持** | 支持 Bash、Zsh、PowerShell 和 CMD |
| 🌐 **跨平台** | 支持 Linux、macOS 和 Windows |
| ⚡ **响应迅速** | 本地 LLM 响应时间 <200ms |
| 👻 **自动模式** | 输入时实时显示幽灵文字建议（类似 GitHub Copilot） |

## 🎬 演示

**Zsh 自动模式** - 输入时实时显示幽灵文字建议：

https://github.com/Zhangtiande/nudge/raw/main/zsh_demo.mp4

## 📋 前置要求

- **Rust**（从源码构建）
- **Ollama**（本地 LLM 推理）或 OpenAI API 访问权限

## 🖥️ 平台支持

Nudge 为多个平台提供预构建的二进制文件。构建状态和可用下载请查看[最新版本](https://github.com/Zhangtiande/nudge/releases/latest)页面。

> **构建状态**: [![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
> 查看 [Actions](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml) 页面获取每个平台的详细构建状态。

| 平台 | 架构 | 二进制文件 | 下载 |
|------|------|-----------|------|
| **Linux** | x86_64 (glibc) | `nudge-linux-x86_64.tar.gz` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz) |
| **Linux** | x86_64 (musl) | `nudge-linux-x86_64-musl.tar.gz` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz) |
| **Linux** | aarch64 (ARM64) | `nudge-linux-aarch64.tar.gz` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-aarch64.tar.gz) |
| **macOS** | x86_64 (Intel) | `nudge-macos-x86_64.tar.gz` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-x86_64.tar.gz) |
| **macOS** | aarch64 (Apple Silicon) | `nudge-macos-aarch64.tar.gz` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-aarch64.tar.gz) |
| **Windows** | x86_64 | `nudge-windows-x86_64.zip` | [📥 下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip) |

> **注意**: 下载链接仅在发布构建成功后可用。如果某个平台的构建失败，其二进制文件将不会出现在发布中。

### Shell 支持

| Shell | Linux | macOS | Windows | 自动模式 | 集成脚本 |
|-------|-------|-------|---------|----------|---------|
| Bash | ✅ | ✅ | ✅ (WSL/Git Bash) | 🚧 (计划中) | `integration.bash` |
| Zsh | ✅ | ✅ | ✅ (WSL) | ✅ (POSTDISPLAY) | `integration.zsh` |
| PowerShell 7.2+ | ❌ | ❌ | ✅ | 🚧 (计划中) | `integration.ps1` |
| PowerShell 5.1 | ❌ | ❌ | ✅ | ❌ (仅手动模式) | `integration.ps1` |
| CMD | ❌ | ❌ | ✅ | ❌ (仅手动模式) | `integration.cmd` |

> **注意**: 自动模式目前仅在 **Zsh** 中完全支持。Bash 和 PowerShell 的支持正在开发中。

## 📦 安装

### 快速安装

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

安装脚本会自动下载二进制文件、添加到 PATH、配置 Shell 集成并启动守护进程。

手动安装、自定义选项或从源码构建，请参阅 [安装指南](docs/installation.md)（英文）。

## 🚀 使用方法

### 快速开始

安装完成后，守护进程应该会自动运行。只需在输入命令时按 `Ctrl+E` 即可触发补全。

### 触发模式

Nudge 支持两种触发模式：

**手动模式**（默认）：按 `Ctrl+E` 按需触发补全。

**自动模式**：输入时自动显示建议，以幽灵文字（光标后的灰色文字）形式呈现。

```yaml
# 在 config.yaml 中启用自动模式
trigger:
  mode: auto              # "manual" 或 "auto"
  auto_delay_ms: 500      # 触发前的防抖延迟
```

| 按键 | 操作 |
|------|------|
| `Ctrl+E` | 触发补全（两种模式） |
| `Tab` | 接受完整建议（自动模式） |
| `Right Arrow` | 接受下一个单词（Zsh/PowerShell） |

详细的自动模式文档，请参阅 [Auto Mode Guide](docs/auto-mode.md)（英文）。

如果需要手动配置 Shell 集成：

```bash
nudge setup
```

然后重启 Shell 或重新加载配置文件：

```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc

# PowerShell
. $PROFILE
```

### 常用命令

```bash
# 启动守护进程
nudge start

# 查看守护进程状态
nudge status

# 停止守护进程
nudge stop

# 重启守护进程（配置更改后）
nudge restart

# 显示运行时信息
nudge info

# 以 JSON 格式显示运行时信息
nudge info --json

# 获取特定字段（在脚本中使用）
nudge info --field config_dir
```

完整的 CLI 参考，请参阅 [CLI Reference](docs/cli-reference.md)（英文）。

## ⚙️ 配置

详细配置选项请参阅 [配置参考文档](docs/configuration.md)。

**快速配置示例**（Linux/macOS: `~/.config/nudge/config.yaml`，Windows: `%APPDATA%\nudge\config\config.yaml`）：

```yaml
# 模型配置
model:
  endpoint: "http://localhost:11434/v1"  # Ollama 默认地址
  model_name: "codellama:7b"
  timeout_ms: 5000

# 上下文设置
context:
  history_window: 20              # 历史命令窗口大小
  include_cwd_listing: true       # 包含当前目录文件列表
  include_system_info: true       # 包含系统信息（操作系统、架构、Shell、用户）
  similar_commands_enabled: true  # 启用相似命令搜索（类似 Ctrl+R）
  similar_commands_window: 200    # 搜索最近 200 条历史记录
  similar_commands_max: 5         # 最多返回 5 条相似命令
  max_files_in_listing: 50        # 最大文件数
  max_total_tokens: 4000          # 最大 token 数

# 触发设置
trigger:
  mode: "manual"            # "manual" 或 "auto"
  hotkey: "\C-e"            # Ctrl+E
  auto_delay_ms: 500        # 自动模式的防抖延迟

# Git 插件
plugins:
  git:
    enabled: true
    depth: standard  # light（轻量）、standard（标准）、detailed（详细）

# 隐私设置
privacy:
  sanitize_enabled: true   # 启用敏感数据清理
  block_dangerous: true    # 标记危险命令

# 日志设置
log:
  level: "info"            # 日志级别: trace/debug/info/warn/error
  file_enabled: false      # 启用文件日志（按天轮转）
```

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge 二进制文件                            │
├─────────────────────────────┬───────────────────────────────────────┤
│          客户端模式          │              守护进程模式              │
├─────────────────────────────┼───────────────────────────────────────┤
│  • 捕获输入缓冲区/光标位置   │  • IPC 服务器                         │
│  • 通过 IPC 发送请求        │    ├─ Unix: Unix Domain Socket        │
│  • 输出补全结果             │    └─ Windows: Named Pipe             │
│                             │  • 上下文引擎                          │
│                             │    ├─ 历史记录读取器                   │
│                             │    ├─ 当前目录扫描器                   │
│                             │    └─ Git 插件                        │
│                             │  • LLM 连接器                         │
│                             │  • 敏感数据清理器                      │
└─────────────────────────────┴───────────────────────────────────────┘
```

**工作流程：**

1. Shell 钩子在按下快捷键时捕获输入缓冲区
2. 客户端通过 IPC（Unix Socket 或 Named Pipe）向守护进程发送请求
3. 守护进程收集上下文（历史记录、当前目录文件、Git 状态）
4. 清理器移除敏感数据
5. LLM 生成补全建议
6. 安全检查标记危险命令
7. 客户端将建议输出到 Shell

## 🔌 LLM 提供商

### 本地部署 (Ollama)

```bash
# 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 拉取模型
ollama pull codellama:7b

# 启动 Ollama 服务
ollama serve
```

### OpenAI / 兼容 API

```yaml
# ~/.config/nudge/config.yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-3.5-turbo"
  api_key_env: "OPENAI_API_KEY"
```

```bash
export OPENAI_API_KEY="sk-..."
```

### 阿里云 DashScope (通义千问)

```yaml
model:
  endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1"
  model_name: "qwen3-coder-flash"
  api_key_env: "DASHSCOPE_API_KEY"
```

```bash
export DASHSCOPE_API_KEY="sk-..."
```

## 🛠️ 开发

```bash
# 运行测试
cargo test

# 带调试日志运行
RUST_LOG=debug cargo run -- daemon --foreground

# 代码检查
cargo clippy

# 代码格式化
cargo fmt
```

## 🗺️ 未来规划

Nudge 正在积极发展，许多激动人心的功能已在规划中。以下是即将推出的功能预览：

### 🎯 即将推出的功能

| 功能 | 描述 | 状态 |
|------|------|------|
| **项目级感知** | 根据命令关键词（docker、npm 等）自动激活插件，提供深度项目上下文 | 🎯 计划中 |
| **错误现场还原** | 命令失败时自动收集错误上下文并提供智能修复建议 | 🎯 计划中 |
| **智能历史分析** | 分析命令模式，为高频命令推荐别名 | 🎯 计划中 |
| **社区插件系统** | 基于 WASM 的插件市场，支持自定义上下文提供器 | 🎯 计划中 |

### 🔌 计划中的插件

除了 Git 之外，还将为以下工具提供上下文支持：
- **Docker**: Dockerfile、compose 文件、运行中的容器
- **Node.js**: package.json、脚本、依赖项
- **Python**: requirements.txt、虚拟环境、pip 包列表
- **Rust**: Cargo.toml、workspace 信息
- **Kubernetes**: kubectl context、pods、资源
- **Terraform**: .tf 文件、workspace、state
- **数据库**: 连接配置、schema 信息

**📖 完整路线图**: 查看 [ROADMAP.md](./ROADMAP.md) 了解详细的功能规格、技术实现方案和发布时间表。

## 📄 许可证

MIT

## 🤝 贡献

欢迎贡献！请提交 Issue 或 Pull Request。
