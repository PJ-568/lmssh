#!/usr/bin/env bash
set -eu

VERSION=$(
  sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1
)

if [ -z "$VERSION" ]; then
  printf 'failed to read version from Cargo.toml\n' >&2
  exit 1
fi

if [ -z "$(git tag --list "v${VERSION}")" ]; then
  printf '%s\n' "$VERSION"
else
  printf '0\n'
fi
