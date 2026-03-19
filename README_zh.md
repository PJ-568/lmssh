# lmssh - 语言模型 SSH 蜜罐

> [English](README.md) | 简体中文

lmssh 是一个用 Rust 编写的 SSH 蜜罐服务器，模拟真实的 shell 环境，同时集成 OpenAI 的语言模型以生成智能响应。它作为安全研究和监控的欺骗性服务器，记录所有会话活动以供分析。

## 功能特性

- **SSH 服务器**：完整的 SSHv2 服务器，支持密码认证（拒绝公钥）
- **交互式 Shell**：行编辑、命令历史和逼真的提示符模拟
- **内置命令**：`cd`、`pwd`、`clear`、`history`、`exit`、`logout`
- **命令黑名单**：拦截交互式 TUI 命令（vim、top、tmux 等）并返回 "command not found"
- **AI 集成**：将不支持的命令转发到 OpenAI 兼容 API，以流式方式返回响应
- **输出防护**：通过可配置的长度、行数和编号行限制，防止失控输出
- **虚拟文件系统**：模拟文件系统操作（mkdir、touch、rm、重定向）而不执行真实命令
- **会话日志**：全面的 JSONL 日志记录，包括连接、认证、命令和 AI 响应事件
- **可配置**：基于 TOML 的配置文件，提供合理的默认值

## 快速开始

### 前提条件

- Rust 1.70+ 和 Cargo
- OpenAI API 密钥（或兼容的端点）

### 安装

```bash
git clone <仓库地址>
cd lmssh
cargo build --release
```

### 配置

在 `~/.config/lmssh/config.toml` 创建配置文件：

```toml
[ssh]
listen_addr = "0.0.0.0:2222"
hostname = "debian"
banner = "SSH-2.0-OpenSSH_9.2p1 Debian-2+deb12u3"
host_key_path = "data/host_ed25519"
session_timeout_seconds = 120
max_input_rate_per_second = 32

[limits]
max_output_length = 8192
max_output_lines = 200
max_numbered_lines = 15

[[users]]
username = "root"
password = "password"

[openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
max_tokens = 512
temperature = 0.6

[logging]
dir = "logs"
```

### 运行

```bash
cargo run -- -c config.toml
```

通过 SSH 连接：

```bash
ssh root@127.0.0.1 -p 2222
# 密码：password
```

## 使用方法

连接后，您将看到逼真的 shell 提示符：

```
root@debian:/root#
```

- 内置命令按预期工作
- 黑名单命令返回 "command not found"
- 其他命令触发 AI 生成的响应，以实时流式传输
- 文件系统操作在虚拟环境中模拟

## 架构设计

项目按模块化组件组织：

- **config**：配置加载和默认值
- **ssh**：使用 russh 实现的 SSH 服务器
- **session**：会话状态、命令路由和输出防护
- **terminal**：交互式输入的行编辑器
- **vfs**：具有副作用模拟的虚拟文件系统
- **ai**：用于流式聊天完成的 OpenAI 客户端
- **logging**：每个会话的 JSONL 事件日志记录
- **prompt**：用于 AI 交互的系统提示词构建

## 开发

### 构建

```bash
cargo check --all-targets
cargo clippy
cargo test
```

### 测试

全面的测试套件涵盖：

- SSH 处理器冒烟测试
- 配置加载
- 命令路由和黑名单
- AI 请求构建和流式传输
- 会话状态管理
- 输出防护限制

运行所有测试：

```bash
cargo test
```

### 代码质量

项目执行严格的质量标准：

- 不允许警告（除非有明确理由的 `#[allow]` 标注）
- 所有测试必须通过
- Clippy 检查符合 Rust 惯用法
- 模块化设计，关注点分离清晰

## 日志记录

会话事件记录到配置目录（默认为 `logs/`）中的 JSONL 文件。每个会话创建单独的文件，包含以下事件：

- `Connected`：客户端连接
- `AuthSuccess`/`AuthFailed`：认证尝试
- `Command`：用户输入的命令
- `AiResponse`：AI 生成的响应及字节数
- `Disconnected`：会话终止

## 安全考虑

- **不执行真实命令**：所有命令要么是内置的，要么被黑名单拦截，要么转发给 AI
- **虚拟文件系统**：文件操作在虚拟环境中模拟，不接触主机文件系统
- **输出限制**：防止通过 excessive AI 输出导致拒绝服务
- **会话隔离**：每个连接在独立的虚拟环境中运行
- **受控认证**：仅支持密码认证，可配置凭据

## 许可证

采用 [Apache License, Version 2.0](LICENSE) 许可。

## 贡献指南

[贡献指北](CONTRIBUTING.md)
