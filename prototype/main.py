#!/usr/bin/env python3
"""Rusty prototype: does a QML terminal tab running Claude inside tmux feel right?

Throwaway spike for milestone M0. It answers three questions before any Rust UI is
written: how qmltermwidget renders Claude Code's TUI, whether attaching to a tmux session
gives us restart-proof terminals, and whether Omarchy's theme colours carry into the app.

Run under the Hyprland session:  python3 prototype/main.py
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

from PySide6.QtGui import QGuiApplication
from PySide6.QtQml import QQmlApplicationEngine

THEME_FILE = Path.home() / ".config/omarchy/current/theme/colors.toml"
FALLBACK = {"background": "#1a1b26", "foreground": "#a9b1d6", "accent": "#7aa2f7", "cursor": "#c0caf5"}


def omarchy_colors() -> dict[str, str]:
    """The current Omarchy theme's colours, from its colors.toml (key = "#hex" lines)."""
    colors = dict(FALLBACK)
    try:
        for line in THEME_FILE.read_text().splitlines():
            m = re.match(r'\s*([a-z0-9_]+)\s*=\s*"(#[0-9a-fA-F]{6})"', line)
            if m:
                colors[m.group(1)] = m.group(2)
    except OSError:
        pass
    return colors


def main() -> int:
    os.environ.setdefault("QT_QPA_PLATFORM", "wayland")
    app = QGuiApplication(sys.argv)
    app.setApplicationName("rusty")
    app.setDesktopFileName("rusty")  # becomes the Wayland app_id
    engine = QQmlApplicationEngine()
    engine.rootContext().setContextProperty("omarchyTheme", omarchy_colors())
    engine.rootContext().setContextProperty("homeDir", str(Path.home()))
    engine.load(str(Path(__file__).with_name("Main.qml")))
    if not engine.rootObjects():
        return 1
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
