# Bash 指南

[English](../../en/shells/bash.md) | [中文](bash.md)

Bash 提供手动补全以及多候选弹出选择器 -- 这是浏览和比较建议的最佳方式。

## 模式

| 模式 | 触发方式 | 描述 |
|---|---|---|
| `bash-inline` | `Ctrl+E` | 快速单候选项应用 |
| `bash-popup` | `Alt+/` | 带风险预览的多候选选择器 |

## 快速开始

```bash
# Single candidate — type and press Ctrl+E
git st<Ctrl+E>
# → git status

# Multi-candidate — type and press Alt+/
docker<Alt+/>
# → Opens popup with numbered candidates to choose from
```

## 弹出选择器

弹出选择器（`Alt+/`）是 Bash 的亮点功能。它从 LLM 请求多个候选项，并以交互式列表呈现。

### 工作原理

1. 按下 `Alt+/` → 集成脚本调用 `nudge complete --format list`
2. 守护进程返回包含元数据的多个候选项
3. 集成脚本启动选择器 UI（fzf、sk、peco 或内置选择器）
4. 选择一个候选项 → 替换当前命令行

### 行格式

每个候选行包含以制表符分隔的字段：

```
risk<TAB>command<TAB>warning<TAB>why<TAB>diff
```

| 字段 | 描述 |
|---|---|
| `risk` | 风险等级：`safe`、`moderate`、`dangerous` |
| `command` | 建议的命令 |
| `warning` | 安全警告（安全时为空） |
| `why` | 命令功能的简要说明 |
| `diff` | 与原始输入相比的变更内容 |

### 选择器后端

`NUDGE_POPUP_BACKEND` 环境变量控制使用哪个选择器：

| 值 | 描述 |
|---|---|
| `auto`（默认） | 自动检测：依次尝试 fzf → sk → peco → builtin |
| `fzf` | 使用 [fzf](https://github.com/junegunn/fzf) |
| `sk` | 使用 [skim](https://github.com/lotabout/skim) |
| `peco` | 使用 [peco](https://github.com/peco/peco) |
| `builtin` | 内置编号列表（无外部依赖） |

安装选择器以获得最佳体验：

```bash
# macOS
brew install fzf

# Ubuntu/Debian
sudo apt install fzf

# Or use without installing anything (builtin fallback)
```

### 候选项生成

守护进程在单次请求中向 LLM 请求多个建议。如果 LLM 返回的候选项少于预期，本地历史记录可能会填充剩余位置。LLM 生成的候选项始终排在最前面。

### 风险确认

默认情况下，高风险候选项（例如 `rm -rf`、破坏性操作）在应用前需要明确确认。

```bash
# Disable confirmation for risky commands (not recommended)
export NUDGE_POPUP_CONFIRM_RISKY=0
```

## 环境变量

| 变量 | 默认值 | 描述 |
|---|---|---|
| `NUDGE_POPUP_BACKEND` | `auto` | 选择器后端：`auto`、`fzf`、`sk`、`peco`、`builtin` |
| `NUDGE_POPUP_SHOW_PREVIEW` | `1` | 显示命令详情的预览面板 |
| `NUDGE_POPUP_HEIGHT` | `70%` | 选择器窗口高度 |
| `NUDGE_POPUP_CONFIRM_RISKY` | `1` | 对危险命令要求确认 |

## 与 fzf/sk/peco 配合使用

### fzf（推荐）

fzf 提供模糊搜索、预览面板和鼠标支持：

```bash
export NUDGE_POPUP_BACKEND=fzf
export NUDGE_POPUP_SHOW_PREVIEW=1
```

在弹出窗口中，使用方向键导航，按 Enter 选择。

### sk (skim)

Skim 是基于 Rust 的 fzf 替代品，具有类似功能：

```bash
export NUDGE_POPUP_BACKEND=sk
```

### peco

Peco 提供更简洁的选择界面：

```bash
export NUDGE_POPUP_BACKEND=peco
```

### builtin

内置选择器显示编号列表。输入编号并按 Enter：

```bash
export NUDGE_POPUP_BACKEND=builtin
```

```
 1) [safe]      git status
 2) [safe]      git stash
 3) [moderate]  git stash drop
Select [1-3]:
```

## 故障排查

| 症状 | 可能原因 | 修复方法 |
|---|---|---|
| `Ctrl+E` 无响应 | 集成脚本未加载 | `nudge setup bash --force && source ~/.bashrc` |
| `Alt+/` 无响应 | 快捷键未设置 | 检查 `bind -P \| grep nudge` |
| 弹出窗口显示空列表 | 守护进程未运行 | `nudge start` |
| 找不到 fzf | 未安装 | `brew install fzf` 或 `apt install fzf` |
| 内置选择器显示乱码 | 终端编码问题 | 确保使用 UTF-8 区域设置 |

## 边界

- Bash 中没有真正的自动幽灵文字（受 Bash readline 限制）
- 弹出窗口的用户体验取决于选择器后端的可用性
- Bash 中没有错误诊断功能（计划在未来版本中支持）
- 推荐使用 Bash 4.0+ 以获得完整的 readline 绑定支持
