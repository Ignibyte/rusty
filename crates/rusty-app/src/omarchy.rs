//! What the app takes from the Omarchy desktop: the current theme's colours, the
//! terminal font Alacritty uses, and a Konsole-format colour scheme generated from the
//! theme so the embedded terminal matches every other terminal on the box.
//!
//! Everything here has a fallback (Tokyo Night colours, JetBrainsMono Nerd Font), so the
//! app also runs on a machine that is not Omarchy.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The name of the generated scheme; the file lives under the user's config dir.
pub const SCHEME_NAME: &str = "Omarchy";

/// The colours the QML side binds to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    /// Window and rail background.
    pub background: String,
    /// Body text.
    pub foreground: String,
    /// Selection and highlights.
    pub accent: String,
    /// Terminal cursor.
    pub cursor: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: "#1a1b26".into(),
            foreground: "#a9b1d6".into(),
            accent: "#7aa2f7".into(),
            cursor: "#c0caf5".into(),
        }
    }
}

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// The active theme's directory: `RUSTY_OMARCHY_THEME_DIR` when set (screenshots of another
/// theme), else the one Omarchy keeps under the home directory (see [`theme_dir_under`]).
pub fn theme_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("RUSTY_OMARCHY_THEME_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    theme_dir_under(&home())
}

/// Where Omarchy keeps the active theme under `home`: since Omarchy 4 a generated
/// directory, `.local/state/omarchy/current/theme` (TICKET-029); before it, the link
/// `.config/omarchy/current/theme`. The newer wins when it exists.
pub fn theme_dir_under(home: &Path) -> PathBuf {
    let state = home.join(".local/state/omarchy/current/theme");
    if state.is_dir() {
        return state;
    }
    home.join(".config/omarchy/current/theme")
}

/// Alacritty's ANSI palette from the theme, as `red`, `bright-red`, ... plus the
/// selection background when the theme names one.
pub fn ansi_tokens() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(theme_dir().join("alacritty.toml")) else {
        return out;
    };
    let Ok(table) = text.parse::<toml::Table>() else {
        return out;
    };
    let Some(colors) = table.get("colors").and_then(|c| c.as_table()) else {
        return out;
    };
    for (section, prefix) in [("normal", ""), ("bright", "bright-")] {
        if let Some(t) = colors.get(section).and_then(|t| t.as_table()) {
            for (name, value) in t {
                if let Some(v) = value.as_str().filter(|v| is_hex_colour(v)) {
                    out.insert(format!("{prefix}{name}"), v.to_string());
                }
            }
        }
    }
    if let Some(v) = colors
        .get("selection")
        .and_then(|s| s.get("background"))
        .and_then(|v| v.as_str())
        .filter(|v| is_hex_colour(v))
    {
        out.insert("selection".to_string(), v.to_string());
    }
    out
}

fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let h = hex.trim_start_matches('#');
    let channel = |i: usize| {
        h.get(i..i + 2)
            .and_then(|s| u8::from_str_radix(s, 16).ok())
            .unwrap_or(0) as f32
    };
    (channel(0), channel(2), channel(4))
}

fn rgb_to_hex(r: f32, g: f32, b: f32) -> String {
    let c = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

/// `t` of `a` over `b`.
pub fn blend(a: &str, b: &str, t: f32) -> String {
    let (ar, ag, ab) = hex_to_rgb(a);
    let (br, bg, bb) = hex_to_rgb(b);
    rgb_to_hex(
        ar * t + br * (1.0 - t),
        ag * t + bg * (1.0 - t),
        ab * t + bb * (1.0 - t),
    )
}

/// Lighten (positive) or darken (negative) by a fraction of the way to white or black.
pub fn shade(hex: &str, amount: f32) -> String {
    let (r, g, b) = hex_to_rgb(hex);
    let towards = if amount >= 0.0 { 255.0 } else { 0.0 };
    let t = amount.abs();
    rgb_to_hex(
        r + (towards - r) * t,
        g + (towards - g) * t,
        b + (towards - b) * t,
    )
}

/// Whether a colour reads as dark (relative luminance under a half).
pub fn is_dark(hex: &str) -> bool {
    let (r, g, b) = hex_to_rgb(hex);
    (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0 < 0.5
}

/// The current theme's colours from `colors.toml`; defaults for anything missing.
pub fn palette() -> Palette {
    let mut palette = Palette::default();
    let Ok(text) = std::fs::read_to_string(theme_dir().join("colors.toml")) else {
        return palette;
    };
    let Ok(table) = text.parse::<toml::Table>() else {
        return palette;
    };
    let pick = |key: &str, slot: &mut String| {
        if let Some(v) = table.get(key).and_then(|v| v.as_str()) {
            if is_hex_colour(v) {
                *slot = v.to_string();
            }
        }
    };
    pick("background", &mut palette.background);
    pick("foreground", &mut palette.foreground);
    pick("accent", &mut palette.accent);
    pick("cursor", &mut palette.cursor);
    palette
}

fn is_hex_colour(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// The font family Alacritty uses here, so the embedded terminal matches it.
pub fn terminal_font() -> String {
    let path = home().join(".config/alacritty/alacritty.toml");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|t| {
            t.get("font")?
                .get("normal")?
                .get("family")?
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_else(|| "JetBrainsMono Nerd Font".to_string())
}

/// The directory the terminal widget is pointed at for colour schemes:
/// `~/.config/rusty/color-schemes`. qmltermwidget reads the `COLORSCHEMES_DIR`
/// environment variable at load, which is how a user-owned file gets in without
/// touching the plugin's own directory.
pub fn scheme_dir() -> PathBuf {
    home().join(".config/rusty/color-schemes")
}

/// Where the generated scheme file goes.
pub fn scheme_path() -> PathBuf {
    scheme_dir().join(format!("{SCHEME_NAME}.colorscheme"))
}

/// Write a Konsole-format colour scheme from the theme's `alacritty.toml` and return
/// its path, or `None` when the theme has no Alacritty palette.
pub fn write_scheme() -> Option<PathBuf> {
    let text = std::fs::read_to_string(theme_dir().join("alacritty.toml")).ok()?;
    let table = text.parse::<toml::Table>().ok()?;
    let colors = table.get("colors")?.as_table()?;
    let body = render_scheme(colors)?;
    let path = scheme_path();
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, body).ok()?;
    Some(path)
}

/// The scheme file text for an Alacritty `[colors]` table.
fn render_scheme(colors: &toml::Table) -> Option<String> {
    let get = |section: &str, key: &str| -> Option<String> {
        colors
            .get(section)?
            .get(key)?
            .as_str()
            .filter(|v| is_hex_colour(v))
            .map(str::to_string)
    };
    let background = get("primary", "background")?;
    let foreground = get("primary", "foreground")?;
    let mut out = format!(
        "[General]\nDescription={SCHEME_NAME} (from the current Omarchy theme)\nOpacity=1\n\n"
    );
    let mut section = |title: &str, colour: &str| {
        out.push_str(&format!(
            "[{title}]\nBold=false\nColor={}\n\n",
            rgb_triplet(colour)
        ));
    };
    section("Background", &background);
    section("BackgroundIntense", &background);
    section("Foreground", &foreground);
    section(
        "ForegroundIntense",
        &get("bright", "white").unwrap_or_else(|| foreground.clone()),
    );
    let names = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];
    for (i, name) in names.iter().enumerate() {
        let normal = get("normal", name)?;
        let bright = get("bright", name).unwrap_or_else(|| normal.clone());
        section(&format!("Color{i}"), &normal);
        section(&format!("Color{i}Intense"), &bright);
    }
    Some(out)
}

/// `#rrggbb` as the `r,g,b` decimal form Konsole schemes use.
fn rgb_triplet(hex: &str) -> String {
    let h = hex.trim_start_matches('#');
    let channel = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
    format!("{},{},{}", channel(0), channel(2), channel(4))
}

/// Everything the QML shell needs at start, gathered once.
#[derive(Debug, Clone)]
pub struct Look {
    /// Theme colours.
    pub palette: Palette,
    /// Terminal font family.
    pub font: String,
    /// The colour scheme name to hand the terminal: the generated `Omarchy` scheme,
    /// or the widget's built-in `Linux` scheme when the theme gave us nothing.
    pub scheme: String,
    /// Extra values worth showing on the Settings page.
    pub facts: BTreeMap<String, String>,
}

impl Look {
    /// Read the desktop and generate the scheme file.
    pub fn gather() -> Self {
        let palette = palette();
        let font = terminal_font();
        let scheme = match write_scheme() {
            Some(_) => SCHEME_NAME.to_string(),
            None => "Linux".to_string(),
        };
        let mut facts = BTreeMap::new();
        facts.insert(
            "theme dir".into(),
            theme_dir().to_string_lossy().into_owned(),
        );
        facts.insert("terminal font".into(), font.clone());
        facts.insert("colour scheme".into(), scheme.clone());
        Self {
            palette,
            font,
            scheme,
            facts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_renders_from_an_alacritty_palette() {
        let toml = r##"
[primary]
background = "#1a1b26"
foreground = "#a9b1d6"
[normal]
black = "#32344a"
red = "#f7768e"
green = "#9ece6a"
yellow = "#e0af68"
blue = "#7aa2f7"
magenta = "#ad8ee6"
cyan = "#449dab"
white = "#787c99"
[bright]
black = "#444b6a"
red = "#ff7a93"
green = "#b9f27c"
yellow = "#ff9e64"
blue = "#7da6ff"
magenta = "#bb9af7"
cyan = "#0db9d7"
white = "#acb0d0"
"##;
        let table: toml::Table = toml.parse().unwrap();
        let scheme = render_scheme(&table).unwrap();
        assert!(scheme.starts_with("[General]\nDescription=Omarchy"));
        assert!(scheme.contains("[Background]\nBold=false\nColor=26,27,38\n"));
        assert!(scheme.contains("[Color1]\nBold=false\nColor=247,118,142\n"));
        assert!(scheme.contains("[Color7Intense]\nBold=false\nColor=172,176,208\n"));
        assert_eq!(scheme.matches("[Color").count(), 16);
    }

    #[test]
    fn hex_colours_are_checked() {
        assert!(is_hex_colour("#7aa2f7"));
        assert!(!is_hex_colour("7aa2f7"));
        assert!(!is_hex_colour("#7aa2f"));
        assert!(!is_hex_colour("#zzzzzz"));
        assert_eq!(rgb_triplet("#ff0080"), "255,0,128");
    }

    #[test]
    fn colour_math() {
        assert_eq!(blend("#ffffff", "#000000", 0.5), "#808080");
        assert_eq!(shade("#000000", 0.5), "#808080");
        assert_eq!(shade("#ffffff", -0.5), "#808080");
        assert!(is_dark("#1a1b26") && !is_dark("#f3eee2"));
    }

    #[test]
    fn the_theme_dir_is_omarchy_fours_then_omarchy_threes() {
        let home = std::env::temp_dir().join(format!("rusty-theme-dir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        assert_eq!(
            theme_dir_under(&home),
            home.join(".config/omarchy/current/theme")
        );
        std::fs::create_dir_all(home.join(".local/state/omarchy/current/theme")).unwrap();
        assert_eq!(
            theme_dir_under(&home),
            home.join(".local/state/omarchy/current/theme")
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn defaults_hold_without_omarchy() {
        let p = Palette::default();
        assert_eq!(p.accent, "#7aa2f7");
        assert!(!terminal_font().is_empty());
    }
}
