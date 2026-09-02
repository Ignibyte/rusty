#!/usr/bin/env python3
"""Rusty prototype: a QML shell with agent terminals over tmux, themed by Omarchy.

Throwaway spike for milestone M0. It settles three things before the Rust app is built:
how qmltermwidget renders Claude Code and Codex, whether tmux-backed sessions give
restart-proof terminals, and whether Omarchy's theme carries into the app.

The agent tabs are the built-in terminal widget, coloured from the current Omarchy theme's
Alacritty palette and using its font, attached to tmux sessions. The Settings tab is a
placeholder: settings arrive with the features that need them.

Run under the Hyprland session:  python3 prototype/main.py
"""

from __future__ import annotations

import os
import re
import sys
import tomllib
from pathlib import Path

from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine

THEME_DIR = Path.home() / ".config/omarchy/current/theme"
ALACRITTY_USER = Path.home() / ".config/alacritty/alacritty.toml"
SCHEME_NAME = "Omarchy"
SCHEME_LOCAL = Path.home() / ".config/rusty" / f"{SCHEME_NAME}.colorscheme"
# qmltermwidget only scans its own directory for schemes; the prototype installs the
# generated file there by hand (see README). The real app will register a custom dir.
SCHEME_SYSTEM = Path("/usr/lib/qt6/qml/QMLTermWidget/color-schemes") / f"{SCHEME_NAME}.colorscheme"
FALLBACK = {"background": "#1a1b26", "foreground": "#a9b1d6", "accent": "#7aa2f7", "cursor": "#c0caf5"}


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




def main() -> int:
    os.environ.setdefault("QT_QPA_PLATFORM", "wayland")
    app = QGuiApplication(sys.argv)
    app.setApplicationName("rusty")
    app.setDesktopFileName("rusty")  # becomes the Wayland app_id
    write_omarchy_scheme()
    scheme = SCHEME_NAME if SCHEME_SYSTEM.exists() else "Linux"
    engine = QQmlApplicationEngine()
    ctx = engine.rootContext()
    ctx.setContextProperty("omarchyTheme", omarchy_colors())
    ctx.setContextProperty("homeDir", str(Path.home()))
    ctx.setContextProperty("termFont", alacritty_font())
    ctx.setContextProperty("termScheme", scheme)
    ctx.setContextProperty("startTab", int(os.environ.get("RUSTY_PROTO_TAB", "0")))
    engine.load(str(Path(__file__).with_name("Main.qml")))
    if not engine.rootObjects():
        return 1
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
