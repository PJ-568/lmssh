# lmssh（Rust）设计文档

日期：2026-03-17

## 1. 目标

在本仓库实现一个 Rust 版本的 SSH 蜜罐服务端，其对外行为与 `reference/` 目录中的 FakeSsh 项目尽量一致，作为可运行、可验收的服务。

核心目标：

- 常驻 SSH 服务端（非一次性 CLI）。
- 支持 **password** 登录（拒绝 publickey）。
- 提供交互式 shell：行编辑、命令历史、提示符等基本体验。
- 内建命令（不走 AI）：`cd` / `pwd` / `clear` / `history` / `exit` / `logout`。
- 拦截交互式 TUI 命令黑名单（如 `vim` / `top` / `tmux` 等），返回 `bash: <cmd>: command not found`。
- 其余命令：
  1) 对虚拟文件系统（VFS）做“副作用模拟”（mkdir/touch/rm/重定向创建文件等）；
  2) 组装 system prompt（仿 Debian 12 终端语气与约束）；
  3) 调用 OpenAI 兼容接口 `/v1/chat/completions`，以 SSE 流式返回输出。
- 输出防失控：最大字节、最大行数、连续编号行限制（参考 reference 的策略）。
- 会话日志：按 session 输出 JSONL 文件，记录连接、认证、命令、AI 输出、断开等事件。

非目标：

- 不执行真实系统命令（避免安全风险）。
- 不实现 SFTP、端口转发等 SSH 高级特性。

## 2. 验收口径（Definition of Done）

### 2.1 代码质量

必须通过且无警告（除非显式 `#[allow]` 并注明原因）：

```shell
cargo check --all-targets
cargo clippy
cargo test
```

### 2.2 端到端行为

```shell
cargo run -- -c config.toml
ssh root@127.0.0.1 -p 2222
```

验收点：

- password 认证成功/失败。
- 内建命令行为正确（`cd/pwd/clear/history/exit/logout`）。
- 黑名单命令返回 `command not found`。
- 其他命令触发 AI 流式输出，并满足截断策略。
- `logs/` 下生成 JSONL，并包含关键事件。

## 3. 配置与 CLI 契约

### 3.1 配置文件

- 默认路径：`~/.config/lmssh/config.toml`
- CLI 覆盖：`-c <path>`

### 3.2 config.toml 结构（建议）

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
base_url = "https://api.openai.com"
model = "gpt-4o-mini"
max_tokens = 512
temperature = 0.6

[logging]
dir = "logs"
```

### 3.3 Host key 策略

- 若 `ssh.host_key_path` 不存在：自动生成 ed25519 私钥并落盘。
- 若存在：加载复用，保证指纹稳定。

## 4. 技术选型

- SSH server：`russh`
- 异步运行时：`tokio`
- OpenAI 流式 HTTP：`reqwest`
- SSE 解析：`eventsource-stream`
- 配置：`serde` + `toml`（外加 `clap` 解析 `-c`）
- 运行时日志：`tracing`
- 业务 JSONL：`serde_json`

## 5. 架构与模块划分（单 crate、内部解耦）

采用单 crate（`lmssh`）快速落地，但内部按职责分层，便于后续拆分。

建议目录：

- `src/main.rs`：读取配置、初始化 tracing、启动 SSH server。
- `src/config.rs`：配置加载、默认路径、校验。
- `src/ssh/server.rs`：russh glue（监听、认证、channel 事件、shell/exec 分派、window change）。
- `src/session/`：
  - `session.rs`：会话状态（用户名、cwd、history、终端尺寸、限速器、VFS 等）。
  - `router.rs`：命令路由（内建/黑名单/AI）。
  - `output_guard.rs`：输出限制（bytes/lines/编号行）。
- `src/terminal/line_editor.rs`：行编辑器状态机。
- `src/vfs/`：虚拟文件系统与副作用模拟。
- `src/ai/client.rs`：OpenAI 兼容 chat.completions SSE streaming。
- `src/prompt.rs`：system prompt 构建（移植 reference 的提示词约束）。
- `src/logging/jsonl.rs`：会话事件模型与 JSONL 落盘。

## 6. VFS 规则

### 6.1 初始化数据

要求：尽量完整移植 `reference/src/FakeSsh/FileSystem/VirtualFileSystem.cs` 的初始化内容（目录、文件、文件内容）。

### 6.2 副作用模拟

至少支持：

- `mkdir`：创建目录（递归可选）。
- `touch`：创建空文件/更新时间戳（时间戳可简化）。
- `rm`：删除文件/目录（递归删除可选）。
- 重定向：`>` / `>>` 在 VFS 上创建/追加文件（内容可记录为空或记录部分文本）。

## 7. AI 调用与输出防护

### 7.1 接口

- `POST {openai.base_url}/v1/chat/completions`
- headers：`Authorization: Bearer <api_key>`
- body：`model`、`messages`、`stream=true`、`temperature`、`max_tokens`

### 7.2 防失控

流式输出过程中实时约束：

- 超过 `limits.max_output_length`：截断并结束。
- 超过 `limits.max_output_lines`：截断并结束。
- 连续编号行超过 `limits.max_numbered_lines`：截断并结束。

## 8. 日志（JSONL）

- 目录：`logging.dir`（默认 `logs/`）
- 每会话一个文件：`session_<uuid>.jsonl`（或包含时间戳）。
- 事件：
  - `Connected`
  - `AuthSuccess` / `AuthFailed`
  - `Command`
  - `AiResponse`（记录 bytes、是否截断等）
  - `Disconnected`
