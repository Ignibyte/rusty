#!/usr/bin/env python3
"""Rusty prototype: a QML shell with agent terminals over tmux, themed by Omarchy.

Throwaway spike for milestone M0. It settles three things before the Rust app is built:
how qmltermwidget renders Claude Code and Codex, whether tmux-backed sessions give
restart-proof terminals, and whether Omarchy's theme carries into the app.

Terminal modes (Settings tab, persisted in ~/.config/rusty/prototype.json):

- ``embedded``: the built-in terminal widget, using Omarchy's Alacritty colours and font,
  so it looks like the Alacritty next door.
- ``alacritty``: the tab launches or focuses a real Alacritty window attached to the same
  tmux session. Wayland cannot embed a foreign window, so this is the honest version of
  "use my terminal". Both modes share the session; switching keeps the conversation.

Run under the Hyprland session:  python3 prototype/main.py
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

from PySide6.QtCore import Property, QObject, Signal, Slot
from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine

CONFIG = Path.home() / ".config/rusty/prototype.json"
THEME_DIR = Path.home() / ".config/omarchy/current/theme"
ALACRITTY_USER = Path.home() / ".config/alacritty/alacritty.toml"
SCHEME_NAME = "Omarchy"
SCHEME_LOCAL = Path.home() / ".config/rusty" / f"{SCHEME_NAME}.colorscheme"
# qmltermwidget only scans its own directory for schemes; the prototype installs the
# generated file there by hand (see README). The real app will register a custom dir.
SCHEME_SYSTEM = Path("/usr/lib/qt6/qml/QMLTermWidget/color-schemes") / f"{SCHEME_NAME}.colorscheme"
FALLBACK = {"background": "#1a1b26", "foreground": "#a9b1d6", "accent": "#7aa2f7", "cursor": "#c0caf5"}
MODES = ("embedded", "alacritty")


def omarchy_colors() -> dict[str, str]:
    """The current Omarchy theme's colours from colors.toml (key = "#hex" lines)."""
    colors = dict(FALLBACK)
    try:
        for line in (THEME_DIR / "colors.toml").read_text().splitlines():
            m = re.match(r'\s*([a-z0-9_]+)\s*=\s*"(#[0-9a-fA-F]{6})"', line)
            if m:
                colors[m.group(1)] = m.group(2)
    except OSError:
        pass
    return colors


def alacritty_font() -> str:
    """The font family Alacritty uses here, so the embedded terminal matches it."""
    try:
        return tomllib.loads(ALACRITTY_USER.read_text())["font"]["normal"]["family"]
    except (OSError, KeyError, tomllib.TOMLDecodeError):
        return "JetBrainsMono Nerd Font"


def write_omarchy_scheme() -> Path | None:
    """Turn the theme's alacritty.toml into a Konsole-format colour scheme file."""
    try:
        theme = tomllib.loads((THEME_DIR / "alacritty.toml").read_text())["colors"]
    except (OSError, KeyError, tomllib.TOMLDecodeError):
        return None

    def rgb(hex_color: str) -> str:
        h = hex_color.lstrip("#")
        return ",".join(str(int(h[i : i + 2], 16)) for i in (0, 2, 4))

    names = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"]
    parts = ["[General]", f"Description={SCHEME_NAME} (from the current Omarchy theme)", "Opacity=1", ""]

    def section(title: str, color: str) -> None:
        parts.extend([f"[{title}]", "Bold=false", f"Color={rgb(color)}", ""])

    primary = theme["primary"]
    section("Background", primary["background"])
    section("BackgroundIntense", primary["background"])
    section("Foreground", primary["foreground"])
    section("ForegroundIntense", theme.get("bright", {}).get("white", primary["foreground"]))
    for i, name in enumerate(names):
        section(f"Color{i}", theme["normal"][name])
        section(f"Color{i}Intense", theme.get("bright", theme["normal"])[name])
    SCHEME_LOCAL.parent.mkdir(parents=True, exist_ok=True)
    SCHEME_LOCAL.write_text("\n".join(parts))
    return SCHEME_LOCAL


def load_config() -> dict:
    try:
        return json.loads(CONFIG.read_text())
    except (OSError, json.JSONDecodeError):
        return {}


def save_config(cfg: dict) -> None:
    CONFIG.parent.mkdir(parents=True, exist_ok=True)
    CONFIG.write_text(json.dumps(cfg, indent=2) + "\n")


def find_window(window_class: str) -> bool:
    """True when a Hyprland client with this class exists."""
    try:
        out = subprocess.run(["hyprctl", "clients", "-j"], capture_output=True, text=True, timeout=5).stdout
        return any(c.get("class") == window_class for c in json.loads(out))
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return False


def launch_or_focus_alacritty(session: str, program: str) -> str:
    """Focus the Alacritty window for this session, or open one attached to it."""
    window_class = session  # sessions are already named rusty-<agent>
    if find_window(window_class):
        subprocess.run(["hyprctl", "dispatch", "focuswindow", f"class:^({window_class})$"], check=False)
        return "focused"
    cmd = [
        "uwsm-app", "--", "alacritty", "--class", window_class, "-T", f"Rusty · {program}",
        "-e", "tmux", "new-session", "-A", "-s", session, program,
    ]
    subprocess.Popen(cmd, start_new_session=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return "launched"


class Settings(QObject):
    """What the QML side reads and writes: the terminal mode and the launchers."""

    terminalModeChanged = Signal()

    def __init__(self) -> None:
        super().__init__()
        self._cfg = load_config()
        if self._cfg.get("terminal") not in MODES:
            self._cfg["terminal"] = "embedded"

    def _get_mode(self) -> str:
        return self._cfg["terminal"]

    def _set_mode(self, mode: str) -> None:
        if mode in MODES and mode != self._cfg["terminal"]:
            self._cfg["terminal"] = mode
            save_config(self._cfg)
            self.terminalModeChanged.emit()

    terminalMode = Property(str, _get_mode, _set_mode, notify=terminalModeChanged)

    @Slot(str, str, result=str)
    def launchOrFocus(self, session: str, program: str) -> str:
        return launch_or_focus_alacritty(session, program)

    @Slot(str, result=bool)
    def isOpen(self, session: str) -> bool:
        return find_window(session)


def main() -> int:
    os.environ.setdefault("QT_QPA_PLATFORM", "wayland")
    app = QGuiApplication(sys.argv)
    app.setApplicationName("rusty")
    app.setDesktopFileName("rusty")  # becomes the Wayland app_id
    write_omarchy_scheme()
    scheme = SCHEME_NAME if SCHEME_SYSTEM.exists() else "Linux"
    settings = Settings()
    engine = QQmlApplicationEngine()
    ctx = engine.rootContext()
    ctx.setContextProperty("omarchyTheme", omarchy_colors())
    ctx.setContextProperty("homeDir", str(Path.home()))
    ctx.setContextProperty("termFont", alacritty_font())
    ctx.setContextProperty("termScheme", scheme)
    ctx.setContextProperty("settings", settings)
    ctx.setContextProperty("configPath", str(CONFIG))
    ctx.setContextProperty("startTab", int(os.environ.get("RUSTY_PROTO_TAB", "0")))
    engine.load(str(Path(__file__).with_name("Main.qml")))
    if not engine.rootObjects():
        return 1
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
