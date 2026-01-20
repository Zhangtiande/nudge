# Nudge Roadmap

本文档概述了 Nudge 项目的未来发展计划和功能路线图。

## 状态说明
- 🎯 **计划中 (Planned)**: 已确定需求，尚未开始
- 🚧 **进行中 (In Progress)**: 正在开发
- ✅ **已完成 (Completed)**: 已实现并合并到主分支
- 🔄 **持续改进 (Ongoing)**: 长期优化项目

---

## 核心功能

### 1. 🎯 项目级感知 (Project-Aware Context)

**目标**: 通过检测命令关键词自动激活相应插件，为 LLM 提供项目相关的深度上下文。

#### 插件系统架构
- **触发机制**:
  - 基于命令前缀检测（如 `git`、`docker`、`npm` 等）
  - 基于当前目录特征文件检测（如 `package.json`、`Dockerfile`）
  - 可配置的插件优先级和超时时间

#### 计划插件

##### ✅ Git 插件 (已有，需扩展)
**当前功能**:
- 基本的 git 状态检测

**扩展功能**:
- 最近 5 条 commit 信息（含作者、时间、message）
- Staged/Unstaged 文件详情
- 当前分支与远程分支的差异
- Stash 列表
- 冲突文件检测
- Submodule 状态

##### 🎯 Docker 插件
**触发条件**: 命令以 `docker` 或 `docker-compose` 开头

**提供上下文**:
- 当前目录下的 `Dockerfile` 内容（前 100 行）
- `docker-compose.yml`/`docker-compose.yaml` 配置
- `.dockerignore` 规则
- 运行中的容器列表（`docker ps`）
- 镜像列表（最近 10 个）
- 卷和网络信息
- 容器日志摘要（最后 20 行，如果命令涉及特定容器）

##### 🎯 Node.js 插件
**触发条件**: 命令以 `npm`、`yarn`、`pnpm`、`node` 开头，或存在 `package.json`

**提供上下文**:
- `package.json` 内容（scripts、dependencies、devDependencies）
- `package-lock.json`/`yarn.lock`/`pnpm-lock.yaml` 元信息
- Node 版本（`node -v`）
- 已安装的全局包（前 20 个）
- `node_modules` 大小统计
- `.nvmrc`/`.node-version` 配置

##### 🎯 Python 插件
**触发条件**: 命令以 `python`、`pip`、`poetry`、`pipenv` 开头

**提供上下文**:
- `requirements.txt`/`pyproject.toml`/`Pipfile` 内容
- 当前激活的虚拟环境路径
- Python 版本（`python --version`）
- 已安装的包列表（`pip list`，前 30 个）
- `setup.py`/`setup.cfg` 配置

##### 🎯 Rust 插件
**触发条件**: 命令以 `cargo`、`rustc` 开头，或存在 `Cargo.toml`

**提供上下文**:
- `Cargo.toml` 配置（dependencies、dev-dependencies、workspace）
- `Cargo.lock` 元信息
- Rust 版本（`rustc --version`）
- 编译目标信息（`cargo metadata`）
- Workspace 成员列表

##### 🎯 Kubernetes 插件
**触发条件**: 命令以 `kubectl`、`k9s`、`helm` 开头

**提供上下文**:
- 当前 context 和 namespace（`kubectl config current-context`）
- YAML 配置文件（`*.yaml`、`*.yml`，Kubernetes 资源定义）
- Pods 状态（当前 namespace）
- 最近的事件（`kubectl get events`）
- Helm releases 列表

##### 🎯 Terraform 插件
**触发条件**: 命令以 `terraform` 开头，或存在 `*.tf` 文件

**提供上下文**:
- `*.tf` 文件列表和主要资源定义
- `terraform.tfvars` 配置
- 当前 workspace（`terraform workspace show`）
- State 文件元信息（资源数量）

##### 🎯 Database 插件
**触发条件**: 命令包含 `psql`、`mysql`、`mongo`、`redis-cli` 等

**提供上下文**:
- 连接字符串配置文件（已脱敏）
- 数据库版本信息
- 可用的数据库列表
- 最近执行的查询历史（如果有日志）

#### 技术实现
- 在 `src/daemon/context/plugins/` 下为每个工具创建独立模块
- 统一的 `Plugin` trait:
  ```rust
  trait Plugin {
      fn name(&self) -> &str;
      fn should_activate(&self, command_buffer: &str, cwd: &Path) -> bool;
      async fn gather_context(&self, cwd: &Path) -> Result<PluginContext>;
      fn timeout(&self) -> Duration; // 默认 100ms
      fn priority(&self) -> u8; // 默认 40
  }
  ```
- 配置文件支持启用/禁用特定插件
- 插件并行执行，超时自动跳过

---

### 2. 🎯 跨平台幽灵文字 (Cross-Platform Ghost Text)

**目标**: 在用户输入时实时显示 AI 建议，而不干扰当前输入，类似 IDE 的内联补全。

#### 功能特性
- **实时建议**: 用户输入时即时显示 AI 预测
- **非侵入式**: 建议以灰色/半透明形式显示在光标后
- **快捷接受**:
  - `Tab` 键接受整个建议
  - `Ctrl+→` 接受下一个单词
  - `Esc` 忽略建议
- **增量流式**: 支持 LLM 流式响应，逐字显示

#### 技术挑战
- **终端能力检测**:
  - 需支持 ANSI 转义序列
  - 检测 true color 支持（24-bit）
  - 检测光标定位能力
- **Shell 集成**:
  - Bash: `PROMPT_COMMAND` 或 `DEBUG` trap
  - Zsh: `precmd`/`preexec` hooks
  - PowerShell: `PSReadLine` module 集成
  - CMD: 受限（可能需要 ConPTY）
- **性能优化**:
  - 去抖动（debounce）输入事件
  - 本地缓存高频命令
  - 流式响应渲染

#### 实现阶段
1. **Phase 1**: Unix shells (Bash/Zsh) 支持，使用 ANSI 转义序列
2. **Phase 2**: PowerShell 集成，通过 `PSReadLine` API
3. **Phase 3**: Windows CMD 探索（可能需要第三方工具）
4. **Phase 4**: 终端兼容性适配（Alacritty, iTerm2, Windows Terminal 等）

#### 配置选项
```yaml
ghost_text:
  enabled: true
  color: "gray" # 或 RGB: [128, 128, 128]
  trigger_delay_ms: 200 # 输入停止后的延迟
  min_input_length: 3 # 触发建议的最小输入长度
  accept_key: "tab" # 可选: tab, right, ctrl-e
```

---

### 3. 🎯 错误现场还原 (Error Context Recovery)

**目标**: 当命令执行失败时，自动收集错误上下文并提供智能修复建议。

#### 核心功能

##### 错误拦截
- **Shell Hook**: 在 `PROMPT_COMMAND`/`precmd` 中检测上一条命令的退出码
- **自动触发**: 退出码非 0 时，自动调用 `nudge diagnose`
- **手动触发**: 用户可执行 `nudge fix-last` 主动请求帮助

##### 上下文收集
- **命令信息**:
  - 失败的完整命令
  - 退出码
  - 执行时间
- **错误输出**:
  - stderr 内容（最后 100 行）
  - stdout 相关输出
- **环境信息**:
  - 当前工作目录
  - 环境变量（相关部分，已脱敏）
  - Shell 类型和版本
- **系统状态**:
  - 磁盘空间（如果是 ENOSPC 错误）
  - 文件权限（如果是 EACCES 错误）
  - 网络连通性（如果是网络相关错误）
  - 进程列表（如果是端口占用）
- **相关文件**:
  - 命令涉及的文件内容（前 50 行）
  - 配置文件（如 `.gitconfig`、`package.json`）

##### 智能分析
- **错误分类**:
  - 语法错误（command not found）
  - 权限错误（permission denied）
  - 文件不存在（no such file or directory）
  - 网络错误（connection refused, timeout）
  - 资源不足（out of memory, disk full）
  - 依赖缺失（library not found）
- **根因分析**: LLM 基于上下文推断根本原因
- **修复建议**:
  - 提供 3-5 个可能的修复命令
  - 解释每个建议的原理
  - 标注危险操作（如 `sudo`）
- **学习能力**: 记录常见错误和修复模式

##### 输出格式
```
❌ Command failed with exit code 127
   $ git pul origin main

🔍 Error Analysis:
   • Command not found: 'pul' is not a valid git command
   • Did you mean: 'pull'?

💡 Suggested fixes:
   1. git pull origin main
      → Fix typo: 'pul' → 'pull'

   2. git fetch origin && git merge origin/main
      → Alternative: fetch and merge separately

📋 Context:
   • Current branch: feature/add-roadmap
   • Remote: origin (https://github.com/user/nudge.git)
   • 2 uncommitted changes

Run a fix: nudge apply-fix <number>
```

##### 交互式修复
- `nudge apply-fix 1`: 直接执行第 1 个建议
- `nudge explain-fix 1`: 详细解释第 1 个建议
- `nudge fix-last --auto`: 自动执行最可能的修复（需用户确认）

#### 技术实现
- 新增子命令: `nudge diagnose`, `nudge fix-last`, `nudge apply-fix`
- 错误历史存储: `~/.config/nudge/error_history.jsonl`（最近 50 条）
- Shell 集成: 在 integration scripts 中添加错误 hook
- 安全检查: 对建议的命令运行安全扫描

---

## 扩展功能

### 4. 🎯 智能命令历史分析 (Smart History Analytics)

**目标**: 深度分析用户的命令历史，提供个性化建议。

- **频率统计**: 识别高频命令模式
- **时间关联**: 分析不同时间段的命令习惯（工作时间 vs 个人时间）
- **序列学习**: 识别常见的命令组合（如 `git add . && git commit && git push`）
- **别名推荐**: 为高频长命令推荐 alias
- **错误预防**: 基于历史错误避免重复失误

### 5. 🎯 多模型支持与动态切换

**目标**: 支持多个 LLM 后端，根据场景智能选择。

- **模型配置**:
  - 快速模型（如 GPT-3.5, Llama 3.1 8B）: 简单补全
  - 强大模型（如 GPT-4, Claude Opus）: 复杂诊断
- **动态切换**:
  - 根据任务复杂度自动选择
  - 用户可通过 `nudge config set-model <name>` 切换
- **成本优化**: 本地模型优先，必要时降级到云端

### 6. 🎯 自定义提示词模板 (Custom Prompt Templates)

**目标**: 允许用户自定义 LLM 提示词，适配不同需求。

- **模板系统**:
  - 默认模板: `~/.config/nudge/prompts/default.txt`
  - 场景模板: `completion.txt`, `diagnosis.txt`, `explanation.txt`
- **变量替换**: `{{command}}`, `{{history}}`, `{{cwd}}` 等
- **社区分享**: 提供模板市场

### 7. 🔄 性能优化 (Ongoing)

- **缓存机制**:
  - LLM 响应缓存（相似查询）
  - 上下文缓存（Git 状态、文件列表）
- **并发控制**: 限制并发请求数，避免 API rate limit
- **增量更新**: 只更新变化的上下文部分
- **内存优化**: 减少 daemon 内存占用

### 8. 🎯 用户习惯学习 (Habit Learning)

**目标**: 学习用户的个人编程风格和偏好。

- **命令风格**: 记住用户喜欢用 `git commit -m` 还是 `git commit -v`
- **参数偏好**: 学习常用的标志组合（如 `ls -alh`）
- **工作流模式**: 识别用户的工作流（TDD、Git Flow 等）
- **隐私保护**: 所有学习数据本地存储，可随时清除

### 9. 🎯 社区插件系统

**目标**: 允许社区贡献自定义插件。

- **插件 API**:
  - 基于 WASM 的沙箱插件系统
  - 或基于外部脚本（Python、JavaScript）
- **插件市场**:
  - 官方插件仓库
  - 一键安装: `nudge plugin install <name>`
- **插件示例**:
  - AWS CLI 插件（EC2、S3 上下文）
  - Jira 插件（关联 issue）
  - CI/CD 插件（GitHub Actions、GitLab CI）

### 10. 🎯 Shell 集成增强

- **更好的 Zsh 集成**: 利用 `zsh/system` 模块
- **Fish Shell 支持**: 添加 Fish 的自动补全集成
- **Nushell 支持**: 探索 Nushell 的插件机制
- **Tmux/Screen 集成**: 在多窗格环境中同步上下文

### 11. 🎯 可观测性与调试

- **详细日志**: 可配置的日志级别（TRACE、DEBUG、INFO）
- **性能追踪**: 每个组件的耗时统计
- **诊断命令**: `nudge doctor` 检查配置和连接
- **Debug 模式**: `nudge complete --debug` 显示完整的上下文和提示词

---

## 质量与工程

### 测试覆盖率提升 🔄
- 目标: 达到 80% 以上的代码覆盖率
- 增加集成测试
- 添加跨平台测试（Windows、macOS、Linux）
- Fuzzing 测试（命令注入、路径遍历等）

### 文档改进 🔄
- 完善 API 文档
- 添加架构设计文档
- 创建贡献者指南
- 多语言文档支持（中文、英文）

### 持续集成增强 🔄
- 增加性能基准测试
- 自动化发布流程
- 依赖安全扫描
- 跨平台编译测试

---

## 发布计划

### v0.2.0 - 项目感知 (Q1 2026)
- ✅ Git 插件扩展
- 🎯 Docker 插件
- 🎯 Node.js 插件
- 🎯 Python 插件
- 🎯 Rust 插件

### v0.3.0 - 错误诊断 (Q2 2026)
- 🎯 错误现场还原
- 🎯 智能修复建议
- 🎯 交互式修复

### v0.4.0 - 幽灵文字 (Q3 2026)
- 🎯 Unix shells 幽灵文字
- 🎯 PowerShell 幽灵文字
- 🎯 流式响应

### v0.5.0 - 社区生态 (Q4 2026)
- 🎯 插件系统
- 🎯 自定义提示词模板
- 🎯 用户习惯学习

### v1.0.0 - 稳定版 (Q1 2027)
- 完整功能集
- 生产环境就绪
- 完善的文档

---

## 贡献指南

欢迎社区贡献！如果你有兴趣实现 Roadmap 中的某个功能：

1. 在 [Issues](https://github.com/your-org/nudge/issues) 中创建或认领对应的任务
2. Fork 项目并创建 feature 分支
3. 参考 `CLAUDE.md` 中的开发规范
4. 提交 PR 并等待审核

对于重大功能变更，请先开 issue 讨论设计方案。

---

## 反馈与建议

如果你有任何建议或想法，欢迎：
- 提交 [GitHub Issue](https://github.com/your-org/nudge/issues)
- 加入 [Discussions](https://github.com/your-org/nudge/discussions)
- 发送邮件至: nudge-dev@example.com

---

*Last updated: 2026-01-20*
