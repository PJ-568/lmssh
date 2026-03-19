# lmssh CI / Release 设计文档

日期：2026-03-19

## 1. 目标

为 `lmssh` 引入一套与 `swaybien/swb-sys-monitor` 风格一致的 GitHub Actions 流程，在 `master` 分支实现：

- Pull Request 自动检查。
- Push 到 `master` 自动检查。
- 检查通过后，若 `Cargo.toml` 中的版本尚未发布，则自动创建 tag 与 GitHub Release。

本次设计聚焦 GitHub Actions 工作流结构与 GitHub Release，不扩展到 crates.io 自动发布。

## 2. 参考对齐策略

参考仓库的关键结构如下：

- `push.yml`：监听 `master` 的 push，串联 `checks.yml` 与 `release.yml`。
- `pull_request.yml`：监听指向 `master` 的 PR，仅执行 `checks.yml`。
- `checks.yml`：封装可复用的 Rust 检查流水线。
- `release.yml`：封装可复用的发行流程，先判断版本是否为新版本，再打 tag、构建产物、创建 Release。

本项目将保持同样的拆分方式，尽量只做项目名、产物名、版本判定脚本的最小适配。

## 3. 工作流设计

### 3.1 `push.yml`

- 触发条件：
  - `workflow_dispatch`
  - `push` 到 `master`
- Job：
  - `checks`：复用 `./.github/workflows/checks.yml`
  - `release`：依赖 `checks` 成功后复用 `./.github/workflows/release.yml`

### 3.2 `pull_request.yml`

- 触发条件：
  - `workflow_dispatch`
  - 指向 `master` 的 `pull_request`
- Job：
  - `checks`：复用 `./.github/workflows/checks.yml`

### 3.3 `checks.yml`

检查内容与仓库开发规范对齐：

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo doc --no-deps --document-private-items`，并要求 rustdoc 无 warning

运行环境选择 `ubuntu-latest`，使用：

- `actions/checkout@v4`
- `MatteoH2O1999/setup-rust@v1`
- `Swatinem/rust-cache@v2`

### 3.4 `release.yml`

发行流程如下：

1. `checkout` 完整历史（`fetch-depth: 0`）。
2. 运行 `scripts/is-newer-version.bash`：
   - 从 `Cargo.toml` 读取当前版本。
   - 若不存在对应 tag（`v<version>`），输出该版本。
   - 若已存在对应 tag，输出 `0`。
3. 若输出不为 `0`：
   - 创建并推送 annotated tag。
   - 重新 checkout 工作树。
   - 安装 Rust 并缓存依赖。
   - `cargo build --release`。
   - 使用 `ncipollo/release-action@v1` 创建 GitHub Release。
   - 上传产物 `./target/release/lmssh`。
4. 若流程失败或取消，且前面已经推送 tag，则删除该 tag，避免残留半成品版本。

## 4. 版本驱动规则

版本源为 `Cargo.toml` 中的 `[package].version`。

规则：

- 当前版本没有同名 tag：视为“新版本”，允许自动发行。
- 当前版本已有 `v<version>` tag：视为“已发布”，跳过发行。

这样可以避免每次 push 到 `master` 都重复创建 release，同时保持流程足够简单，不额外引入 changelog 解析或语义化版本工具。

## 5. 安全与约束

- 仅在 `master` push 后触发发行。
- 发行前必须依赖检查通过。
- 自动 tag 使用 GitHub Actions 运行身份。
- 不引入 `--allow-dirty`、`--force` 等宽松行为。
- 不自动发布 crates.io，避免引入额外凭据与发布风险。

## 6. 验收口径

### 6.1 文件结构

新增：

- `.github/workflows/push.yml`
- `.github/workflows/pull_request.yml`
- `.github/workflows/checks.yml`
- `.github/workflows/release.yml`
- `scripts/is-newer-version.bash`

### 6.2 行为验收

- PR 到 `master` 时，仅触发检查工作流。
- Push 到 `master` 时，先检查，再尝试发行。
- 若版本未发布，则创建 `v<version>` tag 与 GitHub Release。
- 若版本已发布，则跳过 release 步骤，不报错。

### 6.3 本地验证

至少完成以下验证：

```shell
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

并额外验证：

- `scripts/is-newer-version.bash` 在“无 tag”与“已有 tag”两种场景下输出正确。
- YAML 文件路径引用正确，可被 GitHub Actions 识别。
