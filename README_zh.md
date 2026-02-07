# Nudge

![Nudge Cover](./assets/readme-cover.png)

> Nudge 是一个给开发者用的 Shell 补全助手，用项目上下文帮你更快、更稳地输入命令。

[English](./README.md) | [中文](./README_zh.md)

[![CI](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/ci.yml)
[![Release](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml/badge.svg)](https://github.com/Zhangtiande/nudge/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/Zhangtiande/nudge)](https://github.com/Zhangtiande/nudge/releases/latest)
[![License](https://img.shields.io/badge/license-personal%20free%20%7C%20commercial%20restricted-orange)](./LICENSE)

## 快速开始

Linux/macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

Windows（PowerShell）：

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

基础检查：

```bash
nudge status
nudge info
```

试一下补全：

1. 输入半截命令，例如 `git st`
2. 按 `Ctrl+E` 应用建议
3. 在 Bash 下按 `Alt+/` 打开多候选列表

## 为什么做这个

- 减少重复敲命令的时间
- 让建议跟当前项目状态相关
- 在应用高风险命令前给出明确提示

## 能力

- LLM 补全（结合历史、当前目录、插件上下文）
- 项目上下文感知：Git、Node.js、Python、Rust、Docker
- 高风险命令警告
- 失败命令诊断（Zsh / PowerShell）
- 建议缓存：LRU+TTL 加 stale-while-revalidate，重复查询低延迟
- Bash popup 选择器（`Alt+/`）：支持 fzf/sk/peco/builtin 后端的多候选浏览
- 多 Shell 支持：Zsh、Bash、PowerShell、CMD

## 边界

- Nudge 只给建议，不会替你执行命令
- 自动幽灵补全目前只有 Zsh 支持
- Bash/PowerShell/CMD 走手动触发（`Ctrl+E`）
- Bash popup 支持多候选，其它 shell 目前仍以单候选快速路径为主

## 用法

常用按键：

- `Ctrl+E`：手动补全（所有 shell，最快基础路径）
- `Alt+/`：Bash popup 候选选择
- `Tab`：Zsh 自动模式接受建议
- `Ctrl+G`：Zsh autosuggestions 接管 ghost text 时接受 overlay 建议
- `F1`：切换 Zsh 解释详情

核心命令：

```bash
nudge start
nudge stop
nudge restart
nudge status
nudge info
nudge doctor zsh
nudge doctor bash
```

## 安装方式

- 一键安装脚本：见 [docs/zh/installation.md](docs/zh/installation.md)
- 源码构建：`cargo build --release`
- 重装 shell 集成：`nudge setup <bash|zsh|powershell> --force`

## 配置

最小本地模型示例（`~/.nudge/config/config.yaml`）：

```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

trigger:
  mode: manual
```

远程 API 示例：

```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

更多配置： [docs/zh/configuration.md](docs/zh/configuration.md)

## 平台与 Shell 支持

| Shell | 手动 (`Ctrl+E`) | 自动 | Popup | 诊断 | 缓存 | 说明 |
|---|---|---|---|---|---|---|
| Zsh | 支持 | 支持 | 不支持 | 支持 | 支持 | 功能最完整 |
| Bash | 支持 | 不支持 | 支持 (`Alt+/`) | 计划中 | 支持 | 多候选选择器 |
| PowerShell 7.2+ | 支持 | 不支持 | 不支持 | 支持 | 支持 | 通过集成脚本/预测器 |
| CMD | 支持 | 不支持 | 不支持 | 不支持 | 支持 | 仅基础集成 |

## 文档索引

- [安装指南](docs/zh/installation.md) · [English](docs/en/installation.md)
- [配置参考](docs/zh/configuration.md) · [English](docs/en/configuration.md)
- [CLI 参考](docs/zh/cli-reference.md) · [English](docs/en/cli-reference.md)
- [自动模式指南](docs/zh/auto-mode.md) · [English](docs/en/auto-mode.md)
- [Shell 指南](docs/zh/shells/README.md) · [English](docs/en/shells/README.md)
- [FFI API](docs/zh/ffi-api.md) · [English](docs/en/ffi-api.md)
- [路线图](docs/zh/roadmap.md) · [English](docs/en/roadmap.md)

## 开发

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## 许可证

个人/非商业用途免费；商业用途受限且需要单独授权。详见 [LICENSE](./LICENSE)。
