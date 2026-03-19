# CI / Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `lmssh` 增加与参考仓库结构一致的 GitHub Actions 检查与自动发行流程。

**Architecture:** 通过 `push.yml` / `pull_request.yml` 作为入口，复用 `checks.yml` 与 `release.yml` 两个可复用工作流。发行逻辑由 `scripts/is-newer-version.bash` 判定当前 `Cargo.toml` 版本是否已经存在对应 tag，再决定是否创建 GitHub Release。

**Tech Stack:** GitHub Actions YAML, Bash, Rust Cargo toolchain, Git tags.

---

## Conventions

- 配置文件也要最小化改动，尽量与参考仓库保持相同结构。
- shell 脚本使用 `set -eu`，保持失败即退出。
- 产物名固定为 `lmssh`，来自 `Cargo.toml` 的 package 名。
- 发行只覆盖 GitHub Release，不引入 crates.io 发布。

### Task 1: 为版本检测脚本编写红灯验证

**Files:**
- Create: `scripts/is-newer-version.bash`
- Test: `tests/ci/is-newer-version-test.sh`

**Step 1: Write the failing test**

创建 `tests/ci/is-newer-version-test.sh`，覆盖两个场景：

1. 临时 git 仓库中存在 `Cargo.toml` 且没有 `v<version>` tag，脚本输出版本号。
2. 创建对应 tag 后再次运行，脚本输出 `0`。

测试脚本示意：

```bash
#!/usr/bin/env bash
set -eu

ROOT=$(mktemp -d)
trap 'rm -rf "$ROOT"' EXIT

mkdir -p "$ROOT/repo/scripts"
cp scripts/is-newer-version.bash "$ROOT/repo/scripts/"

cat > "$ROOT/repo/Cargo.toml" <<'EOF'
[package]
name = "demo"
version = "1.2.3"
edition = "2024"
EOF

git -C "$ROOT/repo" init >/dev/null
git -C "$ROOT/repo" add Cargo.toml scripts/is-newer-version.bash
git -C "$ROOT/repo" -c user.name=test -c user.email=test@example.com commit -m init >/dev/null

test "$(cd "$ROOT/repo" && ./scripts/is-newer-version.bash)" = "1.2.3"

git -C "$ROOT/repo" tag -a v1.2.3 -m "Release v1.2.3"

test "$(cd "$ROOT/repo" && ./scripts/is-newer-version.bash)" = "0"
```

**Step 2: Run test to verify it fails**

Run: `bash tests/ci/is-newer-version-test.sh`

Expected: FAIL（脚本尚不存在）。

**Step 3: Write minimal implementation**

脚本最小实现要求：

- 使用 `sed` 从 `Cargo.toml` 读取第一个 `version = "..."`。
- 若未找到版本，输出错误并退出非零。
- 若 `git tag --list "v${VERSION}"` 返回空，则输出版本号。
- 否则输出 `0`。

**Step 4: Run test to verify it passes**

Run: `bash tests/ci/is-newer-version-test.sh`

Expected: PASS

**Step 5: Commit**

`【新增，维护】添加版本检测脚本`

---

### Task 2: 添加可复用检查工作流

**Files:**
- Create: `.github/workflows/checks.yml`
- Test: `tests/ci/checks-workflow-grep.sh`

**Step 1: Write the failing test**

创建 `tests/ci/checks-workflow-grep.sh`，检查文件存在且包含这些关键片段：

- `workflow_call`
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo doc --no-deps --document-private-items`

**Step 2: Run test to verify it fails**

Run: `bash tests/ci/checks-workflow-grep.sh`

Expected: FAIL（workflow 文件尚不存在）。

**Step 3: Write minimal implementation**

按参考仓库结构写入 `checks.yml`，包括：

- `workflow_dispatch`
- `workflow_call`
- checkout / setup-rust / rust-cache
- fmt / check / clippy / test / doc 五个检查步骤

**Step 4: Run test to verify it passes**

Run: `bash tests/ci/checks-workflow-grep.sh`

Expected: PASS

**Step 5: Commit**

`【新增，维护】添加可复用检查工作流`

---

### Task 3: 添加入口工作流 push / pull_request

**Files:**
- Create: `.github/workflows/push.yml`
- Create: `.github/workflows/pull_request.yml`
- Test: `tests/ci/entry-workflows-grep.sh`

**Step 1: Write the failing test**

创建 `tests/ci/entry-workflows-grep.sh`，检查：

- `push.yml` 监听 `push` 到 `master`
- `push.yml` 同时复用 `checks.yml` 与 `release.yml`
- `pull_request.yml` 监听 `pull_request` 到 `master`
- `pull_request.yml` 只复用 `checks.yml`

**Step 2: Run test to verify it fails**

Run: `bash tests/ci/entry-workflows-grep.sh`

Expected: FAIL

**Step 3: Write minimal implementation**

文件内容尽量与参考仓库一致，只替换本仓库所需的路径与名称。

**Step 4: Run test to verify it passes**

Run: `bash tests/ci/entry-workflows-grep.sh`

Expected: PASS

**Step 5: Commit**

`【新增，维护】添加工作流入口文件`

---

### Task 4: 添加可复用发行工作流

**Files:**
- Create: `.github/workflows/release.yml`
- Test: `tests/ci/release-workflow-grep.sh`

**Step 1: Write the failing test**

创建 `tests/ci/release-workflow-grep.sh`，检查：

- `workflow_call`
- `fetch-depth: 0`
- 调用 `./scripts/is-newer-version.bash`
- 创建并推送 tag
- `cargo build --release`
- `ncipollo/release-action@v1`
- 上传 `./target/release/lmssh`
- 失败或取消时删除 tag

**Step 2: Run test to verify it fails**

Run: `bash tests/ci/release-workflow-grep.sh`

Expected: FAIL

**Step 3: Write minimal implementation**

对齐参考仓库的 `release.yml`，但：

- 去掉 crates.io 发布步骤。
- 保留版本判断、push tag、构建、创建 GitHub Release、失败回滚 tag。
- 产物改为 `lmssh`。

**Step 4: Run test to verify it passes**

Run: `bash tests/ci/release-workflow-grep.sh`

Expected: PASS

**Step 5: Commit**

`【新增，维护】添加自动发行工作流`

---

### Task 5: 完整验证与文本整理

**Files:**
- Modify: `.github/workflows/*.yml`
- Modify: `scripts/is-newer-version.bash`
- Modify: `tests/ci/*.sh`

**Step 1: Run targeted CI tests**

Run:

```bash
bash tests/ci/is-newer-version-test.sh
bash tests/ci/checks-workflow-grep.sh
bash tests/ci/entry-workflows-grep.sh
bash tests/ci/release-workflow-grep.sh
```

Expected: PASS

**Step 2: Run repository verification**

Run:

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Expected: 全部通过且无 warnings

**Step 3: Lint text formatting**

Run: `autocorrect --lint .github scripts tests/ci docs/plans`

Expected: 无关键排版问题；如有问题，修复后重跑。

**Step 4: Commit**

`【新增，维护】接入 GitHub Actions 检查与发行`
