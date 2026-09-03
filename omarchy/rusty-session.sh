#!/usr/bin/env bash
# rusty-session: bring Rusty up, take it down, or say where it stands.
#
#   rusty-session up       the back end, then the app unit; safe to run again
#   rusty-session down     stop the app unit; the back end keeps serving
#   rusty-session status   both units, the port, the app's processes
#   rusty-session run      what rusty-app.service runs: PATH completed, then exec rusty
#
# One path into a running Rusty: the desktop entry, the launch key and the installer all
# call `up`. The units are omarchy/rusty-mcp.service and omarchy/rusty-app.service.
set -euo pipefail

mcp_unit=rusty-mcp.service
app_unit=rusty-app.service
mcp_url=http://127.0.0.1:4174/mcp

usage() {
  cat <<'USAGE'
usage: rusty-session up | down | status | run
  up       start the back end, then the app unit (safe to run again)
  down     stop the app unit; the back end keeps serving
  status   both units, the port, the app's processes
  run      what rusty-app.service runs: PATH completed, then exec rusty
USAGE
}

# The session variables the app needs. uwsm imports them into the user manager at login;
# a compositor started another way may not, so `up` copies them from its own environment.
display_vars=(WAYLAND_DISPLAY DISPLAY XDG_CURRENT_DESKTOP XDG_SESSION_TYPE HYPRLAND_INSTANCE_SIGNATURE)

manager_has() {
  systemctl --user show-environment 2>/dev/null | grep -q "^$1="
}

import_display() {
  local missing=() name
  for name in "${display_vars[@]}"; do
    if [ -n "${!name:-}" ] && ! manager_has "$name"; then
      missing+=("$name")
    fi
  done
  if [ "${#missing[@]}" -gt 0 ]; then
    systemctl --user import-environment "${missing[@]}"
    echo "imported into the user manager: ${missing[*]}"
  fi
}

# A rusty process that is not the unit's own: started from a terminal or a launcher.
# Starting the unit beside it would open a second window.
unmanaged_app() {
  local main pid
  main=$(systemctl --user show -p MainPID --value "$app_unit" 2>/dev/null || echo 0)
  for pid in $(pgrep -x rusty 2>/dev/null || true); do
    [ "$pid" != "$main" ] && return 0
  done
  return 1
}

up() {
  systemctl --user start "$mcp_unit"
  if systemctl --user is-active --quiet "$app_unit"; then
    :
  elif unmanaged_app; then
    echo "rusty is running outside $app_unit; quit it, then run: rusty-session up"
  else
    if ! manager_has WAYLAND_DISPLAY && ! manager_has DISPLAY; then
      import_display
    fi
    systemctl --user start "$app_unit"
  fi
  status
}

down() {
  systemctl --user stop "$app_unit"
  echo "$app_unit stopped; $mcp_unit keeps serving"
}

status() {
  local unit pids
  for unit in "$mcp_unit" "$app_unit"; do
    printf '%-18s %s\n' "$unit" "$(systemctl --user is-active "$unit" 2>/dev/null || true)"
  done
  if curl -fs -o /dev/null -X POST "$mcp_url" \
      -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' \
      -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"rusty-session","version":"0"}}}' 2>/dev/null; then
    printf '%-18s answering on %s\n' "back end" "$mcp_url"
  else
    printf '%-18s not answering on %s\n' "back end" "$mcp_url"
  fi
  pids=$(pgrep -x rusty 2>/dev/null | tr '\n' ' ' || true)
  printf '%-18s %s\n' "app process" "${pids:-none}"
}

run() {
  case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) PATH="$HOME/.local/bin:$PATH" ;; esac
  if [ -d "$HOME/.cargo/bin" ]; then
    case ":$PATH:" in *":$HOME/.cargo/bin:"*) ;; *) PATH="$PATH:$HOME/.cargo/bin" ;; esac
  fi
  export PATH
  if ! command -v rusty >/dev/null 2>&1; then
    echo "rusty is not on PATH; omarchy/install.sh builds it" >&2
    exit 0
  fi
  exec rusty "$@"
}

case "${1:-}" in
  up) up ;;
  down) down ;;
  status) status ;;
  run) shift; run "$@" ;;
  ''|-h|--help|help) usage ;;
  *) usage >&2; exit 2 ;;
esac
