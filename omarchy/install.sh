#!/usr/bin/env bash
# Install Rusty on this machine: the three binaries, the two user services and a
# reminder of the MCP config. Every step is idempotent, so run it
# again after pulling.
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
app_built=""
if pkg-config --exists Qt6Quick 2>/dev/null || command -v qmake6 >/dev/null 2>&1; then
  echo "==> building the desktop app (rusty)"
  cargo install --path "$repo/crates/rusty-app" --root "$HOME/.local" --force --locked
  install -Dm644 "$here/com.ignibyte.rusty.svg" "$HOME/.local/share/icons/hicolor/scalable/apps/com.ignibyte.rusty.svg"
  install -Dm644 "$here/com.ignibyte.rusty.desktop" "$HOME/.local/share/applications/com.ignibyte.rusty.desktop"
  sed -i "s|^Exec=.*|Exec=$bin/rusty session start|" "$HOME/.local/share/applications/com.ignibyte.rusty.desktop"
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  app_built=yes
else
  echo "==> qt6-base and qt6-declarative are not installed; skipping the desktop app"
fi
case ":$PATH:" in
  *":$bin:"*) ;;
  *) echo "    note: $bin is not on PATH yet" ;;
esac

echo "==> the user services"
# Earlier installs put a rusty-session wrapper here; `rusty session` replaced it (TICKET-029).
rm -f "$bin/rusty-session"
install -Dm644 "$here/rusty-mcp.service" "$unit_dir/rusty-mcp.service"
install -Dm644 "$here/rusty-app.service" "$unit_dir/rusty-app.service"
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
if [ -n "$app_built" ]; then
  systemctl --user enable rusty-app >/dev/null
  if systemctl --user is-active --quiet graphical-session.target; then
    "$bin/rusty" session start
  else
    echo "    no graphical session; the app starts with the next login, or now with: rusty session start"
  fi
fi

echo "==> desktop"
echo "    launch or focus:  omarchy-launch-or-focus '^(rusty|com\\.ignibyte\\.rusty)\$' 'rusty session start'"
if [ -f "$HOME/.config/hypr/bindings.conf" ] && ! grep -q 'rusty session start' "$HOME/.config/hypr/bindings.conf"; then
  echo "    key binding:      append $here/hyprland-bindings.conf to ~/.config/hypr/bindings.conf (SUPER+ALT+R), or replace an older Rusty line"
fi
echo "    under memory pressure, two steps this script leaves to you (another program's unit; root):"
echo "      compositor last:  install -Dm644 $here/wayland-wm-oom.conf ~/.config/systemd/user/wayland-wm@hyprland.desktop.service.d/60-oom.conf && systemctl --user daemon-reload"
echo "      earlyoom:         add Hyprland to --avoid in /etc/default/earlyoom (see $here/README.md)"

echo "==> MCP config"
echo "    agents (stdio): add to .mcp.json  ->  \"rusty\": {\"type\": \"stdio\", \"command\": \"rusty-mcp\"}"
echo "    app (http):     $mcp_url   (both forms in $here/mcp-config.json)"
echo "==> notes"
echo "    notes live in the vault under notes/; an older ~/.rusty/notes folder moves in once with: rusty-cli notes adopt"
echo "done"
