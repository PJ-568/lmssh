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
