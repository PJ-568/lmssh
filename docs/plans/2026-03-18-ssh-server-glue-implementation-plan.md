# SSH Server Glue Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 `lmssh` 成为可运行的 SSH 蜜罐：password 登录、交互式 shell、命令路由（内建/黑名单/AI）、AI 输出防护、JSONL 会话日志。

**Architecture:**
- 用 `russh` 实现 server + per-connection handler。
- 每个 session 持有 `SessionState + LineEditor`；输入可持续处理并允许排队命令。
- 每次回车生成命令后进入一个 async 任务：获取 `output_lock`，执行 Router Action；若是 AI，则流式输出并套 `OutputGuard`。

**Tech Stack:** russh 0.45, tokio, reqwest, serde/toml, clap.

---

## Reference alignment notes

- reference 的排队策略：输入始终进入 LineEditor 缓冲，完整命令进入 pending queue；命令执行时通过 `_outputLock` 串行（见 `reference/src/FakeSsh/Terminal/LineEditor.cs:40-76` 与 `reference/src/FakeSsh/Ssh/ClientSession.cs:225-260`）。
- 我们按同样语义实现：**输入不阻塞、命令执行串行**。

---

### Task 1: 补齐 Config 结构（limits/users/logging）

**Files:**
- Modify: `src/config.rs`
- Test: `tests/config_full.rs`

**Step 1: Write failing test**

`tests/config_full.rs`

```rust
use lmssh::config::Config;

#[test]
fn load_full_config() {
  let toml = r#"
    [ssh]
    listen_addr = "127.0.0.1:2222"
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
    api_key = "sk-test"
    base_url = "https://api.openai.com"
    model = "gpt-4o-mini"
    max_tokens = 512
    temperature = 0.6

    [logging]
    dir = "logs"
  "#;

  let cfg = Config::load_from_str(toml).unwrap();
  assert_eq!(cfg.users.len(), 1);
  assert_eq!(cfg.limits.max_output_lines, 200);
  assert_eq!(cfg.logging.dir, "logs");
}
```

**Step 2: Verify RED**

Run: `cargo test --test config_full`

Expected: FAIL（缺 limits/users/logging 字段）。

**Step 3: Minimal implementation**

- 在 `Config` 增加：`limits: LimitsConfig`、`logging: LoggingConfig`、`users: Vec<UserConfig>`
- 为新增结构体加 `Default`（保持 `#[serde(default)]` 可用）

**Step 4: Verify GREEN**

Run: `cargo test --test config_full`

Expected: PASS

**Step 5: Commit**

`【新增，配置】补齐 limits/users/logging 配置结构`

---

### Task 2: 运行时入口 main：读取 config + 初始化 hostkey + 启动 server

**Files:**
- Modify: `src/main.rs`
- Create: `src/ssh/server.rs`
- Modify: `src/ssh/mod.rs`

**Step 1: Write failing test (smoke)**

`tests/server_smoke.rs`

```rust
#[test]
fn server_module_compiles() {
  // compile-time smoke
  let _ = lmssh::ssh::server::ServerConfig { listen_addr: "127.0.0.1:2222".into() };
}
```

**Step 2: Verify RED**

Run: `cargo test --test server_smoke`

Expected: FAIL（server 模块不存在）。

**Step 3: Minimal implementation**

- 新增 `ssh::server`：
  - `ServerConfig { listen_addr: String }`
  - `async fn run_server(cfg: &Config) -> Result<()>`
- main：
  - clap `-c` 读取路径
  - load config
  - ensure_host_key
  - 调 `run_server`

**Step 4: Verify GREEN**

Run: `cargo test --test server_smoke`

Expected: PASS

**Step 5: Commit**

`【新增，ssh】添加 server 启动骨架`

---

### Task 3: russh Server/Handler：password auth + 拒绝 publickey

**Files:**
- Modify: `src/ssh/server.rs`
- Test: `tests/auth_logic.rs`

**Step 1: Write failing test**

`tests/auth_logic.rs`

```rust
use lmssh::ssh::server::check_password;

#[test]
fn password_auth_accepts_config_users() {
  let users = vec![("root".to_string(), "password".to_string())];
  assert!(check_password(&users, "root", "password"));
  assert!(!check_password(&users, "root", "wrong"));
  assert!(!check_password(&users, "nobody", "password"));
}
```

**Step 2: Verify RED**

Run: `cargo test --test auth_logic`

Expected: FAIL（函数不存在）。

**Step 3: Minimal implementation**

- `check_password(users, username, password)`
- 在 russh handler 的 auth 回调中：
  - password：走 `check_password`
  - publickey：永远 reject

**Step 4: Verify GREEN**

Run: `cargo test --test auth_logic`

Expected: PASS

**Step 5: Commit**

`【新增，ssh】实现 password 认证并拒绝 publickey`

---

### Task 4: 交互式 shell：LineEditor -> Router -> Action 执行（支持命令排队）

**Files:**
- Modify: `src/ssh/server.rs`
- Create: `src/session/executor.rs`
- Modify: `src/session/mod.rs`
- Test: `tests/queue_semantics.rs`

**Step 1: Write failing test**

`tests/queue_semantics.rs`

```rust
use lmssh::terminal::LineEditor;

#[test]
fn line_editor_queues_multiple_commands() {
  let mut ed = LineEditor::new();
  let out = ed.process_bytes(b"pwd\rwhoami\r");
  // 这里我们只验证“能同时产出两个 commands”，后续谁负责执行由 executor 处理。
  assert_eq!(out.commands, vec!["pwd".to_string(), "whoami".to_string()]);
}
```

**Step 2: Verify RED**

Run: `cargo test --test queue_semantics`

Expected: FAIL（目前 line editor 不支持同一批 bytes 产生 2 条命令；需要升级）

**Step 3: Minimal implementation**

- 扩展 `LineEditor`：支持在一段输入内出现多个 `\r` 时产生多个 commands（参考 reference 的 pendingCommands）。
- 增加 `session::executor`：
  - 接收 commands 队列（Vec<String>）
  - 用 `tokio::sync::Semaphore(1)` 串行执行（模拟 `_outputLock`）

**Step 4: Verify GREEN**

Run: `cargo test --test queue_semantics`

Expected: PASS

**Step 5: Commit**

`【新增，terminal】支持命令排队与串行执行语义`

---

### Task 5: AI 输出串流：接入 OpenAiClient + OutputGuard + <NO_OUTPUT>

**Files:**
- Modify: `src/session/executor.rs`
- Modify: `src/ai/client.rs`
- Test: `tests/output_guard_integration.rs`

**Step 1: Write failing test**

让 executor 在接收一个“模拟流”（Vec<String> chunks）时能被 OutputGuard 截断并返回 truncated=true。

**Step 2: Verify RED**

Run: `cargo test --test output_guard_integration`

Expected: FAIL

**Step 3: Minimal implementation**

- executor: 对每个 chunk：
  - `should_stop = guard.push(chunk)`
  - stop 后结束流
- `<NO_OUTPUT>`：若最终输出为该标记则不发送任何文本

**Step 4: Verify GREEN**

**Step 5: Commit**

`【新增，session】AI 输出接入 output guard 并处理 <NO_OUTPUT>`

---

### Task 6: 端到端手工验收指引（不自动化）

Run:

```sh
cargo run -- -c config.toml
ssh root@127.0.0.1 -p 2222
```

检查：pwd/cd/history/clear/exit、黑名单、AI 命令输出、logs/session_*.jsonl。
