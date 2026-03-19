#!/usr/bin/env bash
set -eu

FILE=.github/workflows/checks.yml

test -f "$FILE"
grep -Fq 'workflow_call:' "$FILE"
grep -Fq 'toolchain: stable' "$FILE"
grep -Fq 'cargo fmt --all -- --check' "$FILE"
grep -Fq 'cargo check --all-targets' "$FILE"
grep -Fq 'cargo clippy --all-targets -- -D warnings' "$FILE"
grep -Fq 'cargo test --all-targets' "$FILE"
grep -Fq 'cargo doc --no-deps --document-private-items' "$FILE"
