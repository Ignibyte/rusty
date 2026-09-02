#!/usr/bin/env bash
# Install the pipeline tools project-local and pinned, under .dev/ (ignored): CodeGraph
# (semantic code graph; MCP server + CLI) and the git pre-commit shim that verifies the
# gate receipt. Idempotent. Needs node and npm on PATH.
set -Eeuo pipefail
root=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "run inside the repo" >&2; exit 1; }
cd "$root"

codegraph_version="1.5.0"
prefix="$root/.dev/pipeline-tools/codegraph"

command -v npm >/dev/null 2>&1 || { echo "npm is required (node 22 or newer)" >&2; exit 1; }

echo "==> CodeGraph $codegraph_version (project-local, telemetry off)"
mkdir -p "$prefix"
export CODEGRAPH_TELEMETRY=0 DO_NOT_TRACK=1
if [[ -x "$prefix/node_modules/.bin/codegraph" ]] && [[ "$(cat "$prefix/.version" 2>/dev/null)" == "$codegraph_version" ]]; then
  echo "    present"
else
  (cd "$prefix" && npm install --no-audit --no-fund --silent "@colbymchenry/codegraph@$codegraph_version")
  echo "$codegraph_version" >"$prefix/.version"
  echo "    installed"
fi

echo "==> index this repo (init on the first run, incremental after)"
if [[ -d "$root/.codegraph" ]]; then
  "$root/scripts/codegraph.sh" index --quiet . || echo "    index failed; run scripts/codegraph.sh index . later" >&2
else
  "$root/scripts/codegraph.sh" init . </dev/null || echo "    init failed; run scripts/codegraph.sh init . later" >&2
fi

echo "==> git pre-commit shim (verifies the gate receipt for gated files)"
hook="$(git rev-parse --git-dir)/hooks/pre-commit"
cat >"$hook" <<'EOF'
#!/usr/bin/env bash
# Installed by scripts/setup-pipeline-tools.sh. Gated files need a matching gate receipt.
root=$(git rev-parse --show-toplevel)
source "$root/bin/lib-gate.sh"
gated=0
while IFS= read -r f; do rusty_is_gated "$f" && gated=1; done < <(git diff --cached --name-only)
[[ $gated -eq 1 ]] || exit 0
if msg=$(rusty_verify_receipt); then exit 0; fi
echo "pre-commit: gated files staged and $msg" >&2
exit 1
EOF
chmod +x "$hook"
echo "    $hook"

echo "done. MCP servers: .mcp.json (Claude Code), .codex/config.toml (Codex). Restart the session once so the codegraph server is picked up."
