# Nudge Roadmap

本文档概述了 Nudge 项目的未来发展计划和功能路线图。

## 状态说明
- 🎯 **计划中 (Planned)**: 已确定需求，尚未开始
- 🚧 **进行中 (In Progress)**: 正在开发
- ✅ **已完成 (Completed)**: 已实现并合并到主分支
- 🔄 **持续改进 (Ongoing)**: 长期优化项目

---

## 核心功能

### 1. 🎯 项目感知 (Project-Aware Context) - v0.4.0 计划中

**目标**: 检测命令关键词自动激活相应插件，为 LLM 提供项目深度上下文。

**架构**:
- 命令前缀检测 (如 `git`、`docker`、`npm`)
- 特征文件检测 (如 `package.json`、`Dockerfile`)
- 可配置的优先级和超时时间 (默认 100ms)

**计划插件** (v0.4.0 - v0.5.0):

| 插件 | 触发条件 | 提供上下文 |
|------|---------|-----------|
| **Git** | `git` 命令 | 分支、commit 历史、diff、冲突文件 |
| **Docker** | `docker*` 命令 | Dockerfile、compose、容器/镜像列表 |
| **Node.js** | `npm/yarn` 或 `package.json` | package.json、scripts、依赖树 |
| **Python** | `python/pip/poetry` | requirements、虚拟环境、版本 |
| **Rust** | `cargo` 或 `Cargo.toml` | 依赖、workspace、编译目标 |
| **Kubernetes** | `kubectl/helm` | 当前 context、pods、配置文件 |
| **Terraform** | `terraform` 或 `*.tf` | 资源定义、workspace、state |
| **Database** | `psql/mysql/mongo` | 版本、数据库列表、连接配置 |

---

### 2. ✅ 自动模式 (Auto Mode) - v0.3.0 已实现

**目标**: 在用户输入时实时显示 AI 建议，而不干扰当前输入，类似 IDE 的内联补全。

**已实现功能**:
- ✅ **实时建议** (Zsh): 用户输入时即时显示 AI 预测
- ✅ **非侵入式**: 建议以灰色/半透明形式显示在光标后
- ✅ **快捷接受**: `Tab` 接受整个建议，`Right Arrow` 接受下一个单词
- ✅ **防抖动**: 可配置的延迟时间 (默认 500ms)
- ✅ **手动模式**: 所有平台都支持 `Ctrl+E` 快捷键触发

**平台支持**:
| Shell | 自动模式 | 手动模式 | 说明 |
|-------|---------|---------|------|
| Zsh | ✅ 完成 | ✅ | 推荐 - 完全支持 |
| Bash | ❌ 不支持 | ✅ | Readline 架构限制 |
| PowerShell 7.2+ | ❌ 不支持 | ✅ | PSReadLine 超时限制 |
| PowerShell 5.1 | ❌ 不支持 | ✅ | 无 Predictor API |
| CMD | ❌ 不支持 | ✅ | 无异步机制 |

**配置选项**:
```yaml
trigger:
  mode: auto              # "manual" (所有平台) 或 "auto" (仅 Zsh)
  hotkey: "\C-e"          # 手动模式快捷键
  auto_delay_ms: 500      # 自动模式防抖延迟 (Zsh 只)
```

---

### 3. 🎯 错误诊断 (Error Context Recovery)

**目标**: 当命令失败时，自动收集错误上下文并提供智能修复建议。

**核心功能**:
- **错误拦截**: 在 PROMPT_COMMAND/precmd 中检测退出码
- **上下文收集**:
  - 失败的命令、错误输出、相关文件内容
  - 系统状态（磁盘空间、权限、网络连接）
- **智能分析**: LLM 推断根本原因并提供 3-5 个修复建议
- **交互式修复**: `nudge fix-last` 触发，`nudge apply-fix <N>` 执行

**输出格式**:
```
❌ Command failed with exit code 127
   $ git pul origin main

💡 Suggested fixes:
   1. git pull origin main        → Fix typo: 'pul' → 'pull'
   2. git fetch origin main       → Alternative approach
```

---

## 扩展功能

### 4. 🎯 智能历史分析 (Smart History Analytics)
- 频率统计与别名推荐
- 时间关联分析（工作时间 vs 个人时间）
- 常见命令序列学习
- 错误模式识别与预防

### 5. 🎯 多模型支持 (Multi-Model Support)
- 快速模型 vs 强大模型动态切换
- 成本优化：本地模型优先
- `nudge config set-model <name>` 切换

### 6. 🎯 自定义提示词 (Custom Prompt Templates)
- 用户可编辑：`~/.config/nudge/prompts/`
- 变量替换：`{{command}}`、`{{history}}` 等
- 社区模板市场

### 7. 🔄 性能优化 (Ongoing)
- LLM 响应缓存、上下文缓存
- 并发控制、增量更新
- 内存优化

### 8. 🎯 用户习惯学习 (Habit Learning)
- 记住命令风格偏好
- 识别工作流模式（TDD、Git Flow 等）
- 本地存储，隐私优先

### 9. 🎯 社区插件系统 (Plugin Marketplace)
- WASM 沙箱或脚本插件
- `nudge plugin install <name>`
- 官方插件仓库

### 10. 🎯 Shell 增强 (Shell Enhancements)
- Fish Shell 支持
- Nushell 支持
- Tmux/Screen 集成

### 11. 🎯 可观测性 (Observability)
- `nudge doctor` 诊断工具
- `nudge complete --debug` 显示完整上下文
- 性能追踪

## 发布计划

| 版本 | 目标 | 时间线 | 功能 |
|------|------|--------|------|
| **v0.3.0** | ✅ 自动模式 | 2026-01-23 已发布 | Zsh 自动模式、FFI 层、PSReadLine 集成 |
| **v0.4.0** | 项目感知 | Q1 2026 | Docker、Node.js、Python、Rust 插件 |
| **v0.5.0** | 错误诊断 | Q2 2026 | 错误现场还原、智能修复建议 |
| **v0.6.0** | 社区生态 | Q3 2026 | 插件系统、提示词模板、习惯学习 |
| **v1.0.0** | 稳定版 | Q4 2026 | 完整功能集、生产环境就绪 |

## 质量保证

- 📊 **性能基准**: 手动 vs 自动模式延迟对比
- 🧪 **测试覆盖**: 目标 80% 代码覆盖率
- 🔒 **安全扫描**: 依赖审计、Fuzzing 测试
- 📚 **文档**: API 文档、架构设计、贡献指南

## 贡献与反馈

欢迎参与 Nudge 的开发！

- 📝 在 [Issues](https://github.com/Zhangtiande/nudge/issues) 中创建或认领任务
- 🔗 参考 `CLAUDE.md` 中的开发规范
- 💬 加入 [Discussions](https://github.com/Zhangtiande/nudge/discussions) 讨论设计方案

---

*Last updated: 2026-01-29*
