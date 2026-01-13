# Nudge

> 给你的终端一个温柔的提示 - LLM 驱动的命令行自动补全

[English](./README.md) | [中文](./README_zh.md)

---

Nudge 使用大语言模型，根据你的 Shell 历史记录、当前目录上下文和 Git 仓库状态来预测和补全命令行输入。

## ✨ 功能特性

| 功能 | 描述 |
|------|------|
| 🤖 **AI 智能补全** | 使用 LLM 理解上下文，提供相关命令建议 |
| 📝 **历史感知** | 从 Shell 历史记录中学习，提供个性化建议 |
| 📁 **上下文感知** | 考虑当前目录文件和 Git 状态 |
| 🔒 **隐私优先** | 发送给 LLM 前自动清理敏感数据（API 密钥、密码等） |
| ⚠️ **安全警告** | 标记潜在危险命令（rm -rf、mkfs 等） |
| 🐚 **多 Shell 支持** | 支持 Bash 和 Zsh |
| ⚡ **响应迅速** | 本地 LLM 响应时间 <200ms |

## 📋 前置要求

- **Rust**（从源码构建）
- **Ollama**（本地 LLM 推理）或 OpenAI API 访问权限

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/user/nudge.git
cd nudge

# 构建发布版本
cargo build --release

# 安装到 /usr/local/bin
sudo cp target/release/nudge /usr/local/bin/

# 运行安装脚本
./shell/install.sh
```

### 快速配置

安装后，将以下内容添加到你的 Shell 配置文件：

**Bash** (`~/.bashrc`):
```bash
[ -f "$HOME/.config/nudge/integration.bash" ] && source "$HOME/.config/nudge/integration.bash"
```

**Zsh** (`~/.zshrc`):
```zsh
[ -f "$HOME/.config/nudge/integration.zsh" ] && source "$HOME/.config/nudge/integration.zsh"
```

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

配置文件位置：`~/.config/nudge/config.yaml`

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
```

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Nudge 二进制文件                            │
├─────────────────────────────┬───────────────────────────────────────┤
│          客户端模式          │              守护进程模式              │
├─────────────────────────────┼───────────────────────────────────────┤
│  • 捕获输入缓冲区/光标位置   │  • IPC 服务器（Unix Socket）          │
│  • 通过 IPC 发送请求        │  • 上下文引擎                          │
│  • 输出补全结果             │    ├─ 历史记录读取器                   │
│                             │    ├─ 当前目录扫描器                   │
│                             │    └─ Git 插件                        │
│                             │  • LLM 连接器                         │
│                             │  • 敏感数据清理器                      │
└─────────────────────────────┴───────────────────────────────────────┘
```

**工作流程：**

1. Shell 钩子在按下快捷键时捕获输入缓冲区
2. 客户端通过 Unix Socket 向守护进程发送请求
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
