# Prototype

A PySide6 spike for milestone M0, deleted once the Rust app exists. It proves the terminal
tab (qmltermwidget attached to tmux), the Omarchy theming, and the "embedded or Alacritty"
terminal setting. See the module docstring in `main.py`.

Requirements on Omarchy: `pyside6`, `qmltermwidget`, `qt6-wayland`, `tmux`, `alacritty`.

The embedded terminal takes its colours from the current Omarchy theme's `alacritty.toml`.
qmltermwidget only loads schemes from its own directory, so install the generated file once:

```bash
python3 -c 'import main; main.write_omarchy_scheme()'
sudo cp ~/.config/rusty/Omarchy.colorscheme /usr/lib/qt6/qml/QMLTermWidget/color-schemes/
```

Without it the widget falls back to its `Linux` scheme. The real app registers a custom
scheme directory instead.
