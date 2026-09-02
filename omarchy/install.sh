#!/usr/bin/env bash
# Install Rusty's back end on this machine: the two binaries, the user service, the
# Obsidian vault registration when Obsidian is installed, and a reminder of the MCP
# config. Every step is idempotent, so run it again after pulling.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/.." && pwd)"
bin="$HOME/.local/bin"
unit_dir="$HOME/.config/systemd/user"
mcp_url="http://127.0.0.1:4174/mcp"

need() { command -v "$1" >/dev/null 2>&1 || { echo "missing: $1" >&2; exit 1; }; }
need cargo
need systemctl
need curl

echo "==> building rusty-mcp and rusty-cli into $bin"
cargo install --path "$repo/crates/rusty-mcp" --root "$HOME/.local" --force --locked
cargo install --path "$repo/crates/rusty-cli" --root "$HOME/.local" --force --locked
if pkg-config --exists Qt6Quick 2>/dev/null || command -v qmake6 >/dev/null 2>&1; then
  echo "==> building the desktop app (rusty)"
  cargo install --path "$repo/crates/rusty-app" --root "$HOME/.local" --force --locked
  install -Dm644 "$here/com.ignibyte.rusty.desktop" "$HOME/.local/share/applications/com.ignibyte.rusty.desktop"
  sed -i "s|^Exec=.*|Exec=$bin/rusty|" "$HOME/.local/share/applications/com.ignibyte.rusty.desktop"
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
else
  echo "==> qt6-base and qt6-declarative are not installed; skipping the desktop app"
fi
case ":$PATH:" in
  *":$bin:"*) ;;
  *) echo "    note: $bin is not on PATH yet" ;;
esac

echo "==> user service rusty-mcp"
install -Dm644 "$here/rusty-mcp.service" "$unit_dir/rusty-mcp.service"
systemctl --user daemon-reload
systemctl --user enable rusty-mcp >/dev/null
systemctl --user restart rusty-mcp
answered=""
for _ in $(seq 1 30); do
  if curl -fs -o /dev/null -X POST "$mcp_url" \
      -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' \
      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"install","version":"0"}}}' 2>/dev/null; then
    answered=yes
    break
  fi
  sleep 0.5
done
if [ -z "$answered" ]; then
  echo "    rusty-mcp did not answer on $mcp_url; see: journalctl --user -u rusty-mcp" >&2
  exit 1
fi
echo "    answering on $mcp_url"

echo "==> obsidian"
if command -v obsidian >/dev/null 2>&1; then
  flags="$HOME/.config/obsidian/user-flags.conf"
  if [ -f "$flags" ] && grep -qx -- '-disable-gpu' "$flags"; then
    sed -i 's/^-disable-gpu$/--disable-gpu/' "$flags"
    echo "    user-flags.conf: -disable-gpu became --disable-gpu (the CLI read it as a command)"
  fi
  if "$bin/rusty-cli" obsidian register >/dev/null 2>&1; then
    echo "    brain registered as an Obsidian vault, CLI switched on"
  else
    "$bin/rusty-cli" obsidian configure >/dev/null 2>&1 || true
    echo "    Obsidian is running: quit it once and run  rusty-cli obsidian register"
  fi
else
  echo "    Obsidian is not installed; the obsidian_* tools will say so when called"
fi

echo "==> MCP config"
echo "    agents (stdio): add to .mcp.json  ->  \"rusty\": {\"type\": \"stdio\", \"command\": \"rusty-mcp\"}"
echo "    app (http):     $mcp_url   (both forms in $here/mcp-config.json)"
echo "done"
