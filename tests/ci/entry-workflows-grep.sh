#!/usr/bin/env bash
set -eu

PUSH_FILE=.github/workflows/push.yml
PR_FILE=.github/workflows/pull_request.yml

test -f "$PUSH_FILE"
test -f "$PR_FILE"

grep -Fq 'push:' "$PUSH_FILE"
grep -Fq 'branches:' "$PUSH_FILE"
grep -Fq -- '- master' "$PUSH_FILE"
grep -Fq 'uses: ./.github/workflows/checks.yml' "$PUSH_FILE"
grep -Fq 'uses: ./.github/workflows/release.yml' "$PUSH_FILE"

grep -Fq 'pull_request:' "$PR_FILE"
grep -Fq 'branches:' "$PR_FILE"
grep -Fq -- '- master' "$PR_FILE"
grep -Fq 'uses: ./.github/workflows/checks.yml' "$PR_FILE"
if grep -Fq 'release.yml' "$PR_FILE"; then
  exit 1
fi
