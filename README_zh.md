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

| Shell | Linux | macOS | Windows | 集成脚本 |
|-------|-------|-------|---------|---------|
| Bash | ✅ | ✅ | ✅ (WSL/Git Bash) | `integration.bash` |
| Zsh | ✅ | ✅ | ✅ (WSL) | `integration.zsh` |
| PowerShell | ❌ | ❌ | ✅ | `integration.ps1` |
| CMD | ❌ | ❌ | ✅ | `integration.cmd` |

## 📦 安装

### 一键安装（推荐）

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

安装脚本会自动：
- ✅ 检测您的操作系统和架构
- ✅ 从 GitHub Releases 下载最新的预构建二进制文件
- ✅ 安装到您选择的位置（Unix 上可选 `/usr/local/bin` 或 `~/.local/bin`）
- ✅ 设置 Shell 集成（Bash/Zsh/PowerShell/CMD）
- ✅ 创建默认配置文件

#### 安装选项

**指定版本：**
```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --version 0.1.0

# Windows
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex -Command "& { $_ -Version '0.1.0' }"
```

**自定义安装位置：**
```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --prefix ~/.local

# Windows（先下载脚本）
.\install.ps1 -InstallDir "C:\Tools\nudge"
```

**跳过 Shell 集成：**
```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --skip-shell

# Windows
.\install.ps1 -SkipShell
```

**卸载：**
```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --uninstall

# Windows
.\install.ps1 -Uninstall
```

### 其他安装方式

<details>
<summary><b>从预构建二进制文件手动安装</b></summary>

从 [Releases 页面](https://github.com/Zhangtiande/nudge/releases/latest)下载适合您平台的最新版本。

**Linux/macOS:**
```bash
# 下载并解压（替换为您平台的二进制文件）
curl -L https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz | tar xz

# 移动到 PATH
sudo mv nudge /usr/local/bin/

# 设置 Shell 集成
cd /path/to/nudge/repo
./shell/setup-shell.sh
```

**Windows (PowerShell):**
```powershell
# 从 releases 页面下载并解压
# 手动添加到 PATH 或使用安装脚本
# 设置 Shell 集成
.\shell\setup-shell.ps1
```

</details>

<details>
<summary><b>从源码构建</b></summary>

```bash
# 克隆仓库
git clone https://github.com/Zhangtiande/nudge.git
cd nudge

# 构建发布版本
cargo build --release

# 安装（Unix）
sudo cp target/release/nudge /usr/local/bin/
./shell/setup-shell.sh

# 安装（Windows PowerShell）
# 将 target\release\nudge.exe 复制到 PATH 中的目录
# 然后运行：
.\shell\setup-shell.ps1
```

</details>

<details>
<summary><b>手动配置 Shell 集成</b></summary>

如果您希望手动设置 Shell 集成，请将相应的行添加到您的 Shell 配置文件：

**Bash** (`~/.bashrc`):
```bash
[ -f "$HOME/.config/nudge/integration.bash" ] && source "$HOME/.config/nudge/integration.bash"
```

**Zsh** (`~/.zshrc`):
```zsh
[ -f "$HOME/.config/nudge/integration.zsh" ] && source "$HOME/.config/nudge/integration.zsh"
```

**PowerShell**（添加到 `$PROFILE`）：
```powershell
if (Test-Path "$env:APPDATA\nudge\integration.ps1") {
    . "$env:APPDATA\nudge\integration.ps1"
}
```

**CMD**（添加到 AutoRun 注册表键 `HKCU:\Software\Microsoft\Command Processor`）：
```cmd
"%APPDATA%\nudge\integration.cmd"
```

</details>

## 🚀 使用方法

1. **启动守护进程**（支持懒加载自动启动，或手动启动）：
   ```bash
   nudge daemon --fork
   ```

2. **触发补全**：在输入命令时按 `Ctrl+E`

3. **查看状态**：
   ```bash
   nudge status
   ```

4. **停止守护进程**：
   ```bash
   nudge daemon stop
   ```

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

## 📄 许可证

MIT

## 🤝 贡献

欢迎贡献！请提交 Issue 或 Pull Request。
