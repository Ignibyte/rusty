#!/usr/bin/env bash
# Install Rusty's back end on this machine: the two binaries, the user service and a
# reminder of the MCP config. Every step is idempotent, so run it again after pulling.
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

echo "==> dependencies"
missing=()
for pkg in tmux qt6-base qt6-declarative qmltermwidget; do
  pacman -Q "$pkg" >/dev/null 2>&1 || missing+=("$pkg")
done
if [ "${#missing[@]}" -gt 0 ]; then
  if command -v omarchy >/dev/null 2>&1; then
    echo "    installing: ${missing[*]}"
    omarchy pkg add "${missing[@]}"
  else
    echo "    install these first: ${missing[*]}" >&2
    exit 1
  fi
else
  echo "    tmux, qt6-base, qt6-declarative, qmltermwidget: present"
fi

echo "==> building rusty-mcp and rusty-cli into $bin"
cargo install --path "$repo/crates/rusty-mcp" --root "$HOME/.local" --force --locked
cargo install --path "$repo/crates/rusty-cli" --root "$HOME/.local" --force --locked
if pkg-config --exists Qt6Quick 2>/dev/null || command -v qmake6 >/dev/null 2>&1; then
  echo "==> building the desktop app (rusty)"
  cargo install --path "$repo/crates/rusty-app" --root "$HOME/.local" --force --locked
  install -Dm644 "$here/com.ignibyte.rusty.svg" "$HOME/.local/share/icons/hicolor/scalable/apps/com.ignibyte.rusty.svg"
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

echo "==> desktop"
echo "    launch or focus:  omarchy-launch-or-focus rusty"
if [ -f "$HOME/.config/hypr/bindings.conf" ] && ! grep -q 'omarchy-launch-or-focus rusty' "$HOME/.config/hypr/bindings.conf"; then
  echo "    key binding:      append $here/hyprland-bindings.conf to ~/.config/hypr/bindings.conf (SUPER+ALT+R)"
fi

echo "==> MCP config"
echo "    agents (stdio): add to .mcp.json  ->  \"rusty\": {\"type\": \"stdio\", \"command\": \"rusty-mcp\"}"
echo "    app (http):     $mcp_url   (both forms in $here/mcp-config.json)"
echo "done"
