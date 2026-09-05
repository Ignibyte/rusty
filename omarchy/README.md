# Omarchy integration

What makes Rusty an Omarchy app rather than a program that happens to run there.
`install.sh` ties it together, and every step in it is idempotent.

| File | Purpose |
|---|---|
| `install.sh` | dependencies through `omarchy pkg add`, release builds of the three binaries into `~/.local/bin`, the desktop entry and icon, the two user services, and the pointers below |
| `rusty-mcp.service` | the back end over Streamable HTTP on localhost, wanted by `default.target`, restarted after any exit but a stop |
| `rusty-app.service` | the app, wanted by `graphical-session.target`, restarted when it is killed, left alone when it is quit; its command is `rusty session run` |
| `wayland-wm-oom.conf` | a drop-in for uwsm's compositor unit so Hyprland is the last of the session to go under memory pressure; pointed at, never applied |
| `com.ignibyte.rusty.desktop`, `com.ignibyte.rusty.svg` | the launcher entry and icon; `Exec` is `rusty session start` |
| `hyprland-bindings.conf` | SUPER+ALT+R: focus the window, or `rusty session start`; append it to `~/.config/hypr/bindings.conf` |
| `mcp-config.json` | the `mcpServers` entries for Claude Code and Codex (stdio) and for HTTP clients |

Conventions Rusty follows: apps run in their own user units or through `uwsm-app`, the
theme is read from `~/.local/state/omarchy/current/theme/` (Omarchy 4; before it,
`~/.config/omarchy/current/theme/`), windows are found and focused with
`omarchy-launch-or-focus`, and nothing under `/usr/share/omarchy/` is ever edited.

## The session

Rusty comes back on its own. The back end is a user service wanted by `default.target`,
so it runs whether or not a desktop is up, and `Restart=always` brings it back after any
exit except `systemctl --user stop`: a session teardown and earlyoom both send SIGTERM,
which `on-failure` would have treated as clean. The app is a user service wanted by
`graphical-session.target`, the target uwsm raises at login and lowers at logout, so a
login starts it, a kill or a crash starts it again two seconds later, and a quit (exit 0)
leaves it stopped. Five starts inside ten seconds trip systemd's start limit, which ends
a crash loop.

`rusty session` is the one path in, a noun of the app binary (TICKET-029; a
`rusty-session` script did this before, and the installer removes a stale copy). `start`
starts the back end, copies the display variables into the user manager when a compositor
started outside uwsm left them out, refuses to open a second window when a `rusty`
started from a terminal is still running, and starts the app unit. `stop` stops the app
and keeps the back end. `status` reads both units, posts an `initialize` to the port, and
lists the app's processes. `run` is the unit's command: it completes PATH with
`~/.local/bin` and `~/.cargo/bin`, where the agent CLIs tend to live, and opens the window
in the same process. Every other bare word is a store script or an error; `rusty help`
lists the nouns.

```bash
rusty session start
rusty session status
journalctl --user -u rusty-app -f      # or: journalctl -t rusty
journalctl --user -u rusty-mcp -f
```

No Wayland client outlives its compositor. When Hyprland dies the app dies with it, and
what survives is the state: the tmux sessions behind the agent tabs, `tabs.json` and
`workspace.json` under `~/.config/rusty/`, and the back end. The next login starts the app
unit, which reattaches all of it.

## Memory pressure

On 2026-09-03 a four-worker mutation audit pushed the dev box past its memory and
earlyoom killed Hyprland, because every process in a uwsm session, the compositor
included, sits at an OOM score of 200. Two protections put the compositor last. Neither
is applied by `install.sh`: one is another program's unit, the other needs root.

1. The compositor drop-in, user level. 100 is the user manager's own score and the lowest
   a user unit can set; measured on 2026-09-03, a request for less comes out at 100.

   ```bash
   install -Dm644 omarchy/wayland-wm-oom.conf \
     ~/.config/systemd/user/wayland-wm@hyprland.desktop.service.d/60-oom.conf
   systemctl --user daemon-reload      # takes effect at the next login
   ```

2. earlyoom's avoid list, root. In `/etc/default/earlyoom`, add `Hyprland` and
   `rusty-mcp` to the `--avoid` pattern, then `sudo systemctl restart earlyoom`:

   ```
   EARLYOOM_ARGS="-r 3600 --avoid '(^|/)(systemd|systemd-logind|dbus-daemon|dbus-broker|Hyprland|rusty-mcp)$'"
   ```

   earlyoom then kills anything else first. The kernel's own OOM killer still scores by
   the adjustment, which is what the drop-in is for.
