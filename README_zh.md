# Nudge

> 给你的终端一个温柔的提示 - LLM 驱动的命令行自动补全

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Nudge 使用大语言模型，根据你的 Shell 历史记录、当前目录上下文和项目状态来预测和补全命令行输入。

## 功能特性

- **AI 智能补全** - 使用 LLM 理解上下文，提供相关命令建议
- **项目感知** - 自动检测 Git、Node.js、Python、Rust、Docker 项目并提供深度上下文
- **历史感知** - 从 Shell 历史记录中学习，支持相似命令搜索（类似 Ctrl+R）
- **系统感知** - 根据操作系统、架构和 Shell 类型调整建议
- **错误诊断** - 命令失败时自动分析错误并提供修复建议
- **隐私优先** - 发送给 LLM 前自动清理敏感数据（API 密钥、密码等）
- **安全警告** - 标记潜在危险命令（rm -rf、mkfs 等）
- **多 Shell 支持** - 支持 Bash、Zsh、PowerShell 和 CMD
- **跨平台** - 支持 Linux、macOS 和 Windows
- **响应迅速** - 本地 LLM 响应时间 <200ms
- **自动模式** - 输入时实时显示幽灵文字建议（仅 Zsh）

## 演示

**Zsh 自动模式** - 输入时实时显示幽灵文字建议：

https://github.com/user-attachments/assets/766247e1-1cf2-47da-96e7-045415ede013

## 快速开始

### 安装

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

安装脚本会自动下载二进制文件、配置 Shell 集成并启动守护进程。

手动安装或从源码构建，请参阅 [安装指南](docs/installation.md)。

### 基本使用

安装完成后，在输入命令时按 `Ctrl+E` 即可触发补全。

```bash
# 启动守护进程（如果未自动启动）
nudge start

# 查看状态
nudge status

# 显示运行时信息
nudge info
```

### 配置

创建配置文件 `~/.config/nudge/config.yaml`（Linux/macOS）或 `%APPDATA%\nudge\config\config.yaml`（Windows）：

```yaml
model:
  endpoint: "http://localhost:11434/v1"  # Ollama 默认地址
  model_name: "codellama:7b"

trigger:
  mode: "manual"        # "manual" 或 "auto"
  auto_delay_ms: 500    # 自动模式防抖延迟

diagnosis:
  enabled: true         # 启用错误诊断
```

完整配置选项请参阅 [配置参考](docs/configuration.md)。

## 触发模式

| 模式 | 描述 | 支持的 Shell |
|------|------|--------------|
| **手动模式** | 按 `Ctrl+E` 触发 | 所有 Shell |
| **自动模式** | 输入时自动显示幽灵文字 | 仅 Zsh |

| 按键 | 操作 |
|------|------|
| `Ctrl+E` | 触发补全 |
| `Tab` | 接受建议（自动模式） |
| `Right Arrow` | 接受下一个单词（Zsh） |

## 错误诊断

当命令失败时，Nudge 会结合完整的项目上下文分析错误并提供修复建议。

**Zsh:**
```
$ gti status
zsh: command not found: gti
❌ Typo: 'gti' should be 'git'

git status          ← 按 Tab 接受
```

**PowerShell:**
```
PS> gti status
[Error] Command not found: 'gti'
[Tip] Typo: did you mean 'git'?

PS> █               ← 按 Tab 接受
```

在配置中启用：
```yaml
diagnosis:
  enabled: true
```

> [!CAUTION]
> 启用错误诊断时，命令执行期间 stderr 会被临时捕获。这意味着 `cargo build`、`npm install`、`docker pull` 等工具的进度输出会在命令**完成后**才显示，而非实时显示。如需实时查看 stderr 输出，请设置 `diagnosis.enabled: false` 禁用诊断功能。

## 项目感知上下文

Nudge 自动检测项目类型并为 LLM 提供相关上下文：

| 项目类型 | 检测方式 | 提供的上下文 |
|----------|----------|--------------|
| **Git** | `.git` 目录 | 分支、暂存文件、最近提交 |
| **Node.js** | `package.json` | 脚本、依赖、包管理器 |
| **Python** | `pyproject.toml`、`requirements.txt` | 依赖、虚拟环境、Python 版本 |
| **Rust** | `Cargo.toml` | 依赖、编译目标、workspace 信息 |
| **Docker** | `Dockerfile`、`compose.yaml` | 服务、镜像、运行中的容器 |

## LLM 提供商

### Ollama（本地部署）

```bash
ollama pull codellama:7b
ollama serve
```

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
```

### OpenAI

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

### 阿里云 DashScope（通义千问）

```yaml
model:
  endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1"
  model_name: "qwen-coder-plus"
  api_key_env: "DASHSCOPE_API_KEY"
```

## 平台支持

| 平台 | 架构 | 下载 |
|------|------|------|
| Linux | x86_64 (glibc) | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz) |
| Linux | x86_64 (musl) | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz) |
| Linux | aarch64 | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-aarch64.tar.gz) |
| macOS | x86_64 (Intel) | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-x86_64.tar.gz) |
| macOS | aarch64 (Apple Silicon) | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-aarch64.tar.gz) |
| Windows | x86_64 | [下载](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip) |

### Shell 支持

| Shell | 手动模式 | 自动模式 | 错误诊断 |
|-------|----------|----------|----------|
| Zsh | ✅ | ✅ | ✅ |
| Bash | ✅ | ❌ | 计划中 |
| PowerShell 7.2+ | ✅ | ❌ | ✅ |
| PowerShell 5.1 | ✅ | ❌ | ✅ |
| CMD | ✅ | ❌ | ❌ |

## 架构设计

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge 二进制文件                            │
├─────────────────────────────┬───────────────────────────────────────┤
│          客户端模式          │              守护进程模式              │
├─────────────────────────────┼───────────────────────────────────────┤
│  • 捕获输入缓冲区/光标位置   │  • IPC 服务器（Socket/Named Pipe）    │
│  • 通过 IPC 发送请求        │  • 上下文引擎                          │
│  • 输出补全结果             │    ├─ 历史记录、当前目录、系统信息      │
│                             │    └─ 插件（Git、Node、Python...）     │
│                             │  • LLM 连接器                         │
│                             │  • 敏感数据清理器 & 安全检查器          │
└─────────────────────────────┴───────────────────────────────────────┘
```

## 文档

- [安装指南](docs/installation.md)
- [配置参考](docs/configuration.md)
- [CLI 参考](docs/cli-reference.md)
- [自动模式指南](docs/auto-mode.md)
- [路线图](ROADMAP.md)

## 开发

```bash
cargo build --release
cargo test
cargo clippy
cargo fmt
```

## 贡献

欢迎贡献！请提交 Issue 或 Pull Request。

## 许可证

MIT
