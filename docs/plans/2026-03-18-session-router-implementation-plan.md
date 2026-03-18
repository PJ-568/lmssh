# Session / Router Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 引入会话状态与命令路由：内建命令（cd/pwd/clear/history/exit/logout）、黑名单拦截、其余命令走 VFS 副作用 + system prompt + OpenAI SSE。

**Architecture:**
- 新增 `session` 模块：`SessionState` 保存会话状态；`Router` 纯逻辑产出 `Action`（不直接做 IO）。
- 由上层（未来的 SSH glue）执行 `Action`：写回输出、断开连接、或调用 AI 流式并套上输出防护。

**Tech Stack:** Rust 2024, tokio, reqwest, serde, toml, clippy, cargo test.

---

## Conventions

- **TDD 强制**：每个新增行为先写测试、跑红、再写最小实现、跑绿。
- 内建命令输出使用 `\n`（不输出 `\r\n`），保持与 system prompt 约束一致。
- `history` 输出**形似 bash**：右对齐编号 + 两空格 + 命令，示例：`"    1  pwd\n"`。
- `clear` 只要能清屏：输出 `"\x1b[2J\x1b[H"`。

---

### Task 1: 定义 session action 与 session state 骨架

**Files:**
- Create: `src/session/mod.rs`
- Create: `src/session/action.rs`
- Create: `src/session/session.rs`
- Modify: `src/lib.rs`
- Test: `tests/session_action.rs`

**Step 1: Write the failing test**

`tests/session_action.rs`

```rust
use lmssh::session::Action;

#[test]
fn action_is_constructible() {
  let a = Action::Disconnect;
  match a {
    Action::Disconnect => {}
    _ => panic!("unexpected"),
  }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test session_action`

Expected: FAIL（找不到 `lmssh::session` 或 `Action`）。

**Step 3: Write minimal implementation**

- `Action`（纯数据）：
  - `SendText(String)`
  - `NoOutput`
  - `Disconnect`
  - `AiRequest { system_prompt: String, user_command: String }`
- `SessionState`（先最小字段，Task 2/3 再补）：
  - `username/hostname/cwd/client_ip/terminal/term_width/term_height`
  - `history: Vec<String>`
  - `recent_commands: Vec<(String, String)>`
  - `vfs: VirtualFileSystem`

**Step 4: Run test to verify it passes**

Run: `cargo test --test session_action`

Expected: PASS

**Step 5: Commit**

Commit message:

`【新增，session】添加会话动作与状态骨架`

---

### Task 2: Router - 内建命令 pwd / exit / logout

**Files:**
- Create: `src/session/router.rs`
- Modify: `src/session/mod.rs`
- Test: `tests/router_builtins_basic.rs`

**Step 1: Write the failing tests**

`tests/router_builtins_basic.rs`

```rust
use lmssh::session::{Action, Router, SessionState};

fn mk_session() -> SessionState {
  SessionState::new_for_test("root", "debian", "/root")
}

#[test]
fn pwd_outputs_cwd_with_newline() {
  let mut s = mk_session();
  let r = Router::default();
  let act = r.handle_command(&mut s, "pwd");
  assert_eq!(act, Action::SendText("/root\n".to_string()));
}

#[test]
fn exit_disconnects() {
  let mut s = mk_session();
  let r = Router::default();
  assert_eq!(r.handle_command(&mut s, "exit"), Action::Disconnect);
  assert_eq!(r.handle_command(&mut s, "logout"), Action::Disconnect);
}
```

**Step 2: Run tests to verify RED**

Run: `cargo test --test router_builtins_basic`

Expected: FAIL（Router/SessionState helper 不存在）。

**Step 3: Minimal implementation**

- `Router::handle_command`：识别 `pwd/exit/logout`。
- `SessionState::new_for_test(...)`：仅用于测试构造（如果你不想要该函数，可改用 `SessionState { .. }` 直接构造，但要保持测试可读）。

**Step 4: Verify GREEN**

Run: `cargo test --test router_builtins_basic`

Expected: PASS

**Step 5: Commit**

`【新增，session】支持 pwd 与退出命令`

---

### Task 3: Router - 内建命令 clear

**Files:**
- Modify: `src/session/router.rs`
- Test: `tests/router_clear.rs`

**Step 1: Write failing test**

```rust
use lmssh::session::{Action, Router, SessionState};

#[test]
fn clear_sends_ansi_clear_screen() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  assert_eq!(
    r.handle_command(&mut s, "clear"),
    Action::SendText("\x1b[2J\x1b[H".to_string())
  );
}
```

**Step 2: Verify RED**

Run: `cargo test --test router_clear`

Expected: FAIL

**Step 3: Implement**

- `clear` → `Action::SendText("\x1b[2J\x1b[H".into())`

**Step 4: Verify GREEN**

Run: `cargo test --test router_clear`

Expected: PASS

**Step 5: Commit**

`【新增，session】支持 clear 清屏`

---

### Task 4: Router - 内建命令 history（形似 bash）

**Files:**
- Modify: `src/session/router.rs`
- Modify: `src/session/session.rs`
- Test: `tests/router_history.rs`

**Step 1: Write failing tests**

```rust
use lmssh::session::{Action, Router, SessionState};

#[test]
fn history_formats_like_bash() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  s.push_history("pwd");
  s.push_history("ls -la");
  let r = Router::default();

  let act = r.handle_command(&mut s, "history");
  let Action::SendText(text) = act else { panic!("expected SendText"); };

  assert_eq!(text, "    1  pwd\n    2  ls -la\n");
}
```

**Step 2: Verify RED**

Run: `cargo test --test router_history`

Expected: FAIL

**Step 3: Minimal implementation**

- `SessionState::push_history(cmd)`：只收非空/非全空白命令。
- `history`：遍历 `history`，编号从 1 开始，使用 `format!("{i:>5}  {cmd}\n")`。
- 输出受限：先不加 guard（guard 在 Task 8/9），但至少不要输出解释性文字。

**Step 4: Verify GREEN**

Run: `cargo test --test router_history`

Expected: PASS

**Step 5: Commit**

`【新增，session】支持 history 并仿 bash 格式化`

---

### Task 5: Router - 内建命令 cd（含相对路径与错误）

**Files:**
- Modify: `src/session/router.rs`
- Modify: `src/session/session.rs`
- Modify: `src/vfs/tree.rs`（如需要：暴露 path resolve helper；优先不改）
- Test: `tests/router_cd.rs`

**Step 1: Write failing tests**

```rust
use lmssh::session::{Action, Router, SessionState};

#[test]
fn cd_to_existing_dir_changes_cwd_and_no_output() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  // seed 里有 /tmp
  let act = r.handle_command(&mut s, "cd /tmp");
  assert_eq!(act, Action::NoOutput);
  assert_eq!(s.cwd(), "/tmp");
}

#[test]
fn cd_to_missing_dir_prints_error() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  let act = r.handle_command(&mut s, "cd /no_such_dir");
  assert_eq!(
    act,
    Action::SendText("bash: cd: /no_such_dir: No such file or directory\n".to_string())
  );
  assert_eq!(s.cwd(), "/root");
}
```

**Step 2: Verify RED**

Run: `cargo test --test router_cd`

Expected: FAIL

**Step 3: Minimal implementation**

- 解析参数：
  - `cd` 无参：切到 home（root→`/root`，否则 `/home/<user>`）
  - 相对路径：拼接 cwd（`/` 特判）
  - 处理 `.` / `..`：复用 vfs 的 `normalize_path` 行为（若当前未暴露，则在 session 里实现一个相同逻辑的 `resolve_path(cwd, path)`，后续再去重）
- 判断目录存在：`vfs.is_dir(target)`。

**Step 4: Verify GREEN**

Run: `cargo test --test router_cd`

Expected: PASS

**Step 5: Commit**

`【新增，session】支持 cd 并校验目录存在`

---

### Task 6: Router - 黑名单命令返回 command not found

**Files:**
- Modify: `src/session/router.rs`
- Create: `src/session/blacklist.rs`
- Modify: `src/session/mod.rs`
- Test: `tests/router_blacklist.rs`

**Step 1: Write failing test**

```rust
use lmssh::session::{Action, Router, SessionState};

#[test]
fn blacklisted_command_returns_command_not_found() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  let act = r.handle_command(&mut s, "vim");
  assert_eq!(act, Action::SendText("bash: vim: command not found\n".to_string()));
}
```

**Step 2: Verify RED**

Run: `cargo test --test router_blacklist`

Expected: FAIL

**Step 3: Minimal implementation**

- 黑名单先按设计文档子集：`vim/vi/nano/emacs/top/htop/tmux/screen/less/more` 等（从 reference `ClientSession.cs` 的 InteractiveCommands/SOFT blacklist 挑最关键的；保持列表集中在一个文件里）。
- 匹配规则：只看命令第一个 token（空格前）。

**Step 4: Verify GREEN**

Run: `cargo test --test router_blacklist`

Expected: PASS

**Step 5: Commit**

`【新增，session】拦截黑名单命令并返回 command not found`

---

### Task 7: Router - 非内建/非黑名单：VFS 副作用 + prompt + AiRequest

**Files:**
- Modify: `src/session/router.rs`
- Modify: `src/session/session.rs`
- Test: `tests/router_ai_request.rs`

**Step 1: Write failing tests**

```rust
use lmssh::session::{Action, Router, SessionState};

#[test]
fn other_command_produces_ai_request_and_applies_vfs_side_effects() {
  let mut s = SessionState::new_for_test("root", "debian", "/root");
  let r = Router::default();
  let act = r.handle_command(&mut s, "echo hi > /tmp/out.txt");

  // side effect
  assert_eq!(s.vfs().read_file("/tmp/out.txt"), Some("hi\n"));

  // action
  let Action::AiRequest { system_prompt, user_command } = act else {
    panic!("expected AiRequest");
  };
  assert!(system_prompt.contains("禁止markdown"));
  assert_eq!(user_command, "echo hi > /tmp/out.txt");
}
```

**Step 2: Verify RED**

Run: `cargo test --test router_ai_request`

Expected: FAIL

**Step 3: Minimal implementation**

- 在 `SessionState` 中提供只读访问器：`vfs()`、`cwd()`。
- `handle_command` 其它分支：
  - 调 `apply_side_effects(&mut vfs, cmd)`
  - 构造 `PromptContext` 并调 `build_system_prompt`（recent_commands 先传空或当前保存的窗口）
  - 返回 `Action::AiRequest { ... }`

**Step 4: Verify GREEN**

Run: `cargo test --test router_ai_request`

Expected: PASS

**Step 5: Commit**

`【新增，session】非内建命令路由到 AI 并应用 VFS 副作用`

---

### Task 8: Output guard（bytes/lines/连续编号行）

**Files:**
- Create: `src/session/output_guard.rs`
- Modify: `src/session/mod.rs`
- Test: `tests/output_guard.rs`

**Step 1: Write failing tests**

```rust
use lmssh::session::OutputGuard;

#[test]
fn guard_truncates_when_too_many_lines() {
  let mut g = OutputGuard::new(10_000, 3, 15);
  assert!(!g.push("a\n"));
  assert!(!g.push("b\n"));
  assert!(g.push("c\n")); // reaching limit should stop
}

#[test]
fn guard_truncates_on_numbered_lines() {
  let mut g = OutputGuard::new(10_000, 200, 2);
  assert!(!g.push("1. a\n"));
  assert!(g.push("2. b\n"));
}
```

**Step 2: Verify RED**

Run: `cargo test --test output_guard`

Expected: FAIL

**Step 3: Minimal implementation**

- `OutputGuard::push(chunk: &str) -> bool`：返回 `true` 表示应停止继续输出。
- 统计：
  - bytes：`chunk.as_bytes().len()` 累加
  - lines：统计 `\n` 数
  - numbered lines：当行首匹配 `^\s*\d+[\.)]` 视为编号行，连续计数；遇到非编号行重置

**Step 4: Verify GREEN**

Run: `cargo test --test output_guard`

Expected: PASS

**Step 5: Commit**

`【新增，session】添加 AI 输出防护 output guard`

---

### Task 9: 回归验证

Run（必须无 warning）：

```sh
cargo check --all-targets
cargo clippy
cargo test
```

---

## Notes

- 本计划不接入 `russh` 事件循环；只实现纯逻辑 Router + OutputGuard，便于后续 Task 把它们挂到 SSH session。
- 提交频率按 Task 拆分；如果你希望合并提交，可在执行前再调整。
