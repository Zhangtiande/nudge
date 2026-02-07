# Shell 指南

[English](../../en/shells/README.md) | [中文](README.md)

各 shell 的行为、集成模式和功能可用性。

## 功能矩阵

| Shell | 模式 | 快速路径 | 弹出选择器 | Auto Ghost | 诊断 | 缓存 | 插件 |
|---|---|---|---|---|---|---|---|
| **Zsh** | `zsh-inline`, `zsh-auto` | `Ctrl+E` | 否 | 是 | 是 | 是 | 全部 |
| **Bash** | `bash-inline`, `bash-popup` | `Ctrl+E` | `Alt+/` (fzf/sk/peco/builtin) | 否 | 计划中 | 是 | 全部 |
| **PowerShell** | `ps-inline` | `Ctrl+E` | 否 | 否 | 是 | 是 | 全部 |
| **CMD** | `cmd-inline` | `Ctrl+E` | 否 | 否 | 否 | 是 | 全部 |

**一句话定位**：

- **Zsh**：全功能旗舰 — auto ghost text、overlay、诊断、所有快捷键绑定
- **Bash**：最佳多候选体验 — 弹出选择器，带风险预览
- **PowerShell**：Windows 原生路径 — PSReadLine 集成，带错误诊断
- **CMD**：最小可用 — 使用 doskey 宏实现基本补全

## 基准规则

`Ctrl+E` 手动路径始终是所有 shell 中最快的单候选基准。

## 各 Shell 文档

- [Zsh](zsh.md) — Auto 模式、ghost 所有权、overlay、诊断
- [Bash](bash.md) — 弹出选择器、多候选工作流
- [PowerShell](powershell.md) — PSReadLine predictor、诊断
- [CMD](cmd.md) — Doskey 宏、基本设置
