# Auto 模式指南

[English](../en/auto-mode.md) | [中文](auto-mode.md)

Auto 模式在你输入时提供实时 ghost text 建议，无需按快捷键。

## 适用范围

- 支持的 shell：**仅 Zsh**（完整 ghost text 和 overlay）
- PowerShell 7.2+：通过 PSReadLine predictor 提供实验性 auto 模式（参见 [PowerShell 指南](shells/powershell.md)）
- Bash/CMD：仅支持手动触发（`Ctrl+E`）

## 快速启用

编辑 `~/.nudge/config/config.yaml`：

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: auto
  zsh_overlay_backend: message
```

应用配置：

```bash
nudge restart
```

## Ghost Text 所有权（`zsh_ghost_owner`）

此设置决定由谁渲染光标后方的灰色建议文本。

### `auto`（默认）

Nudge 自动检测 `zsh-autosuggestions` 是否已加载：

- **如果 autosuggestions 存在**：Nudge 将 ghost text 交由 autosuggestions 处理，自身建议通过 overlay 行（message 或 rprompt）显示。按 `Ctrl+G` 接受 Nudge 的建议。
- **如果 autosuggestions 不存在**：Nudge 接管 ghost text 所有权。按 `Tab` 接受建议。

这是大多数用户最安全的选项。

### `nudge`

强制 Nudge 接管 ghost text，忽略其他插件。这会将 `zsh-autosuggestions` 的 ghost text 替换为 Nudge 的 LLM 建议。

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: nudge
```

适合偏好 LLM 建议而非基于历史的 autosuggestions 的用户。

### `autosuggestions`

将 ghost text 保留给 `zsh-autosuggestions`。Nudge 始终使用 overlay 显示。

```yaml
trigger:
  mode: auto
  zsh_ghost_owner: autosuggestions
```

适合依赖 autosuggestions 进行快速历史回溯、同时希望 Nudge 建议单独显示的用户。

## Overlay 后端（`zsh_overlay_backend`）

当 Nudge **不拥有** ghost text 时（即 `autosuggestions` 模式或 `auto` 模式下 autosuggestions 存在），它通过 overlay 显示建议。有两种后端可选：

### `message`（默认）

使用 `zle -M` 在提示符下方渲染一行消息。

- 更少的提示符重绘闪烁
- 在下一次按键时消失
- 兼容所有终端

### `rprompt`

在右侧提示符（`RPS1`）中渲染建议。

- 持续显示直到新建议到达
- 可能与现有的右侧提示符自定义冲突
- 对较短的建议有更好的可见性

```yaml
trigger:
  zsh_overlay_backend: rprompt
```

## 事件驱动的获取机制

Auto 模式不使用轮询，其工作流程如下：

1. 你输入一个字符 → Zsh widget 触发
2. 集成脚本检查 `$PENDING` 和 `$KEYS_QUEUED_COUNT` — 如果还有排队的按键，则跳过（防抖）
3. 经过 `auto_delay_ms`（默认 500ms）后，触发后台获取
4. 当响应到达时，generation 计数器防止过时的响应覆盖较新的响应
5. 建议被渲染（ghost text 或 overlay，取决于所有权设置）

这种设计在快速输入时避免了不必要的 LLM 调用，同时保持建议的实时性。

### 调整 `auto_delay_ms`

| 值 | 行为 |
|---|---|
| `200` | 非常灵敏，更多 LLM 调用 |
| `500` | 平衡（默认） |
| `1000` | 保守，更少调用 |

```yaml
trigger:
  auto_delay_ms: 300
```

## 快捷键绑定（Zsh Auto 模式）

| 按键 | 操作 | 条件 |
|---|---|---|
| `Tab` | 接受完整建议 | Nudge 拥有 ghost text |
| `Right Arrow` | 接受下一个单词 | Nudge 拥有 ghost text |
| `Ctrl+G` | 接受 Nudge 建议 | Overlay 模式（autosuggestions 拥有 ghost） |
| `F1` | 切换解释详情 | 始终可用（显示 `why`、`diff`、`risk`） |
| `Ctrl+E` | 手动补全 | 始终可用（绕过 auto，立即请求） |

## 建议缓存集成

Auto 模式从建议缓存中获益显著：

- 重复的前缀会立即返回缓存结果（无需 LLM 调用）
- 上下文变化（cd、git commit）会自动使相关缓存条目失效
- Stale-while-revalidate 策略立即返回旧结果，同时在后台刷新
- Auto 模式使用 5 分钟 TTL（可通过 `cache.ttl_auto_ms` 配置）

## 故障排查

检查当前设置：

```bash
nudge doctor zsh
nudge info --field zsh_ghost_owner
nudge info --field zsh_overlay_backend
nudge info --field trigger_mode
```

如果行为看起来过时或异常：

```bash
nudge setup zsh --force
nudge restart
```

**常见问题**：

| 症状 | 可能原因 | 解决方法 |
|---|---|---|
| 没有建议出现 | Daemon 未运行 | `nudge start` |
| Ghost text 闪烁 | 终端重绘冲突 | 尝试 `zsh_overlay_backend: rprompt` |
| 建议覆盖了 autosuggestions | Ghost 所有权冲突 | 设置 `zsh_ghost_owner: autosuggestions` |
| 建议到达太晚 | `auto_delay_ms` 过高 | 降低到 300ms |
| `Ctrl+G` 无响应 | 不在 overlay 模式 | 检查 `nudge info --field zsh_ghost_owner` |

## 边界

- 没有跨 shell 的 auto 模式回退 — Bash/CMD 保持手动触发
- 部分终端可能对功能键映射不同；请使用 `nudge doctor zsh` 验证
- Auto 模式需要 daemon 正在运行；它不会自动启动 daemon
- Ghost text 渲染依赖终端的 ANSI 支持
