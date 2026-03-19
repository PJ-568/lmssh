#!/usr/bin/env bash
set -eu

FILE=.github/workflows/release.yml

test -f "$FILE"
grep -Fq 'workflow_call:' "$FILE"
grep -Fq 'fetch-depth: 0' "$FILE"
grep -Fq 'contents: write' "$FILE"
grep -Fq 'git fetch --tags --force' "$FILE"
grep -Fq './scripts/is-newer-version.bash' "$FILE"
grep -Fq 'git tag -a' "$FILE"
grep -Fq 'git push origin' "$FILE"
grep -Fq 'cargo build --release' "$FILE"
grep -Fq 'ncipollo/release-action@v1' "$FILE"
grep -Fq './target/release/lmssh' "$FILE"
grep -Fq 'git push --delete origin "${{ steps.push-tag.outputs.tag_name }}" || true' "$FILE"
grep -Fq 'Delete tag if failure or cancelled' "$FILE"
