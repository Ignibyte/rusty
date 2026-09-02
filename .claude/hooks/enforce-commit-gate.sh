#!/usr/bin/env bash
# PreToolUse on Bash: a `git commit` that includes gated files needs a receipt that matches
# the worktree (bin/gate.sh --diff). Docs-only commits pass. `--no-verify` is refused.
set -Eeuo pipefail
HOOK_INPUT=$(cat)
command -v jq >/dev/null 2>&1 || exit 0
# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib-hook-helpers.sh"
# shellcheck source=bin/lib-gate.sh
source "$(dirname "${BASH_SOURCE[0]}")/../../bin/lib-gate.sh"

cmd=$(hook_field '.tool_input.command') || true
[[ -n "$cmd" ]] || exit 0
grep -qE '(^|[[:space:];&|])git([[:space:]]+-C[[:space:]]+[^[:space:]]+)?[[:space:]]+commit([[:space:]]|$)' <<<"$cmd" || exit 0
grep -qE -- '--no-verify' <<<"$cmd" && hook_block "COMMIT GATE: --no-verify is not allowed (CONSTITUTION §15)."

root=$(hook_root) || exit 0
# Which files would this commit carry: staged now, or (with -a) modified tracked files.
staged=$(git -C "$root" diff --cached --name-only 2>/dev/null; if grep -qE '(^|[[:space:]])-(a|am|-all)([[:space:]]|$)' <<<"$cmd"; then git -C "$root" diff --name-only 2>/dev/null; fi)
gated=""
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  if rusty_is_gated "$f"; then gated+="$f"$'\n'; fi
done <<<"$staged"
[[ -n "$gated" ]] || exit 0

if msg=$(rusty_verify_receipt); then
  exit 0
fi
hook_block "COMMIT GATE: this commit carries gated files ($(printf '%s' "$gated" | head -3 | tr '\n' ' ')…) and $msg"
