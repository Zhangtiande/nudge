# Nudge Roadmap

本文档概述了 Nudge 项目的未来发展计划和功能路线图。

## 状态说明

- ✅ **已完成** - 已实现并发布
- 🚧 **进行中** - 正在开发
- 🎯 **计划中** - 已确定需求，尚未开始

---

## 版本历史

### v0.3.0 - 自动模式 ✅

- Zsh 自动模式（幽灵文字建议）
- 防抖延迟配置
- Tab 接受完整建议
- Right Arrow 接受下一个单词

### v0.4.0 - 错误诊断增强 ✅

**目标**: 增强错误诊断功能，提供与命令补全相同级别的项目感知能力。

**已完成**:
- ✅ 错误诊断基础功能（Zsh、PowerShell）
- ✅ stderr 捕获和分析
- ✅ Tab 键接受修复建议
- ✅ 诊断上下文与补全上下文统一
- ✅ 诊断支持完整项目上下文（Git、Node、Python、Rust、Docker）
- ✅ **建议缓存** (v0.4.2): LRU+TTL 缓存，stale-while-revalidate 策略
  - 缓存 key: prefix + cwd + git_state + shell_mode
  - TTL: auto=5min, manual=10min
  - 上下文变化自动失效

**改进内容**:
- 诊断现在包含系统信息（OS、架构、Shell 类型）
- 诊断现在包含完整命令历史（基于 session）
- 诊断现在包含所有项目插件上下文
- 统一的 `GatherParams` 抽象，复用上下文收集逻辑

---

## 未来版本

### v0.5.0 - 智能历史分析 🎯

- 频率统计与别名推荐
- 常见命令序列学习
- 错误模式识别与预防

### v0.6.0 - 扩展插件 🎯

| 插件 | 触发条件 | 提供上下文 |
|------|----------|------------|
| **Kubernetes** | `kubectl/helm` | 当前 context、pods、配置文件 |
| **Terraform** | `terraform` 或 `*.tf` | 资源定义、workspace、state |
| **Database** | `psql/mysql/mongo` | 版本、数据库列表、连接配置 |

### v0.7.0 - 社区生态 🎯

- WASM 插件系统
- 自定义提示词模板
- 插件市场

### v1.0.0 - 稳定版 🎯

- 完整功能集
- 生产环境就绪
- 完善的文档和测试覆盖

---

## 已完成功能

### 核心功能

| 功能 | 版本 | 描述 |
|------|------|------|
| AI 命令补全 | v0.1.0 | LLM 驱动的命令建议 |
| 多 Shell 支持 | v0.1.0 | Bash、Zsh、PowerShell、CMD |
| 隐私保护 | v0.1.0 | 敏感数据自动清理 |
| 安全警告 | v0.1.0 | 危险命令检测 |
| Git 插件 | v0.2.0 | 分支、提交、状态上下文 |
| 自动模式 | v0.3.0 | Zsh 幽灵文字建议 |
| 错误诊断 | v0.3.0 | 失败命令分析和修复建议 |
| Docker 插件 | v0.3.0 | 容器、镜像、compose 上下文 |
| Node.js 插件 | v0.3.0 | package.json、脚本、依赖 |
| Python 插件 | v0.3.0 | pyproject.toml、依赖、虚拟环境 |
| Rust 插件 | v0.3.0 | Cargo.toml、依赖、workspace |
| 诊断项目感知 | v0.4.0 | 诊断使用完整项目上下文 |
| 建议缓存 | v0.4.2 | LRU+TTL 缓存减少 LLM 调用 |

### 平台支持

| Shell | 手动模式 | 自动模式 | 错误诊断 |
|-------|----------|----------|----------|
| Zsh | ✅ v0.1.0 | ✅ v0.3.0 | ✅ v0.3.0 |
| Bash | ✅ v0.1.0 | ❌ | 🎯 计划中 |
| PowerShell 7.2+ | ✅ v0.1.0 | ❌ | ✅ v0.3.0 |
| PowerShell 5.1 | ✅ v0.1.0 | ❌ | ✅ v0.3.0 |
| CMD | ✅ v0.1.0 | ❌ | ❌ |

---

## 贡献

欢迎参与 Nudge 的开发！

- 在 [Issues](https://github.com/Zhangtiande/nudge/issues) 中创建或认领任务
- 参考 `CLAUDE.md` 中的开发规范
- 加入 [Discussions](https://github.com/Zhangtiande/nudge/discussions) 讨论设计方案

---

*Last updated: 2026-02-05*
