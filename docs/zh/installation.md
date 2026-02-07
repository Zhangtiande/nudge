# 安装指南

[English](../en/installation.md) | [中文](installation.md)

本指南涵盖 Nudge 的安装、Shell 集成设置以及验证一切是否正常工作。

## 快速安装

Linux/macOS：

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

Windows (PowerShell)：

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

安装程序会运行一个交互式向导，询问你的 LLM 端点和首选触发模式。

## 安装程序做了什么

1. 将 `nudge` 二进制文件下载到 `~/.nudge/bin/`（Windows 上为 `%USERPROFILE%\.nudge\bin\`）
2. 将 `~/.nudge/bin` 添加到你的 `PATH`
3. 将默认配置写入 `~/.nudge/config/config.default.yaml`
4. 根据向导的回答生成用户配置 `~/.nudge/config/config.yaml`
5. 将 Shell 集成脚本复制到 `~/.nudge/shell/`
6. 在你的 Shell 配置文件（`.bashrc`、`.zshrc` 或 PowerShell `$PROFILE`）中添加 `source` 行
7. 启动 daemon

## 安装选项

```bash
# Install a specific version
./scripts/install.sh --version 0.5.0

# Custom install prefix
./scripts/install.sh --prefix "$HOME/.local"

# Skip shell profile modification
./scripts/install.sh --skip-shell

# Use a locally built binary
./scripts/install.sh --local

# Uninstall
./scripts/install.sh --uninstall
```

## 从源码构建

```bash
git clone https://github.com/Zhangtiande/nudge.git
cd nudge
cargo build --release
./scripts/install.sh --local
```

## Shell 集成设置

`nudge setup` 命令会写入集成脚本并将其挂接到你的 Shell 配置文件中。

```bash
nudge setup bash          # Set up Bash integration
nudge setup zsh           # Set up Zsh integration
nudge setup powershell    # Set up PowerShell integration
```

使用 `--force` 覆盖已有的集成文件：

```bash
nudge setup zsh --force
```

设置完成后，重启你的 Shell 或重新加载配置文件：

```bash
source ~/.zshrc   # or ~/.bashrc
```

CMD 没有自动设置路径。请参阅 [CMD 指南](shells/cmd.md) 进行手动设置。

## 安装后验证

按顺序运行以下命令以确认安装正常：

```bash
# 1. Check daemon is running
nudge status
# Expected: "Running (pid: ...)"

# 2. Show configuration summary
nudge info
# Shows: config_dir, socket_path, trigger_mode, etc.

# 3. Verify integration script exists
nudge info --field integration_script
# Should print a path like ~/.nudge/shell/integration.zsh

# 4. Run health check (Zsh/Bash)
nudge doctor zsh    # or: nudge doctor bash
# Reports key bindings, hooks, and integration status

# 5. Try a completion
# Type a partial command, press Ctrl+E
```

如果 `nudge status` 显示 `Not running`，请启动 daemon：

```bash
nudge start
```

## 文件布局

安装完成后，你的 `~/.nudge/` 目录结构如下：

```
~/.nudge/
├── bin/
│   └── nudge              # Binary
├── config/
│   ├── config.default.yaml  # Shipped defaults (do not edit)
│   └── config.yaml          # Your overrides
├── shell/
│   ├── integration.bash
│   ├── integration.zsh
│   ├── integration.ps1
│   └── integration.cmd
├── logs/                    # When file_enabled: true
└── nudge.sock               # Unix domain socket (runtime)
```

在 Windows 上，将 `~/.nudge/` 替换为 `%USERPROFILE%\.nudge\`，将 socket 替换为命名管道 `\\.\pipe\nudge_{username}`。

## 升级

重新运行安装程序即可。它会保留你的 `config.yaml` 并更新其他所有内容：

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

升级后，刷新 Shell 集成以获取新功能：

```bash
nudge setup zsh --force
nudge restart
```

## 卸载

```bash
./scripts/install.sh --uninstall
```

或者手动操作：

1. 删除 `~/.nudge/`
2. 从你的 Shell 配置文件中移除 `source` 行
3. 停止 daemon：`nudge stop`

## 边界

- CMD 没有通过 `nudge setup` 进行自动配置的功能
- Bash 不支持真正的自动 ghost-text 模式
- 安装程序需要 `curl`（Unix）或 PowerShell 5.1+（Windows）
