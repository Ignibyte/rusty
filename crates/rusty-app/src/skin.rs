//! The look as data. A skin is a set of colour roles, the ones the Replit mock gives
//! its colours: a background and three panel levels, two line weights, three text
//! weights, an accent with a softer twin, gold for titles, an "alive" colour for state
//! and links, and red. The roles come from one of three sources the user picks in
//! Settings: a built-in preset (the mock's Amber phosphor first), the Omarchy desktop
//! theme mapped onto the roles, or a TOML file of the user's own under
//! `~/.config/rusty/themes/`. [`tokens`] turns the roles into every colour the QML shell
//! and the renderer bind to, the older token names included, so a theme is data and the
//! rest of the app never learns where it came from.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::omarchy::{blend, is_dark, shade, Palette};

/// Where the roles come from and how the look is switched.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Choice {
    /// `preset`, `omarchy` or `file`.
    pub source: String,
    /// The preset's id or the file's stem; ignored for `omarchy`.
    pub name: String,
    /// The CRT scanline overlay.
    pub scanlines: bool,
}

impl Default for Choice {
    fn default() -> Self {
        Self {
            source: "preset".into(),
            name: "amber-phosphor".into(),
            scanlines: true,
        }
    }
}

/// The colour roles, `#rrggbb` each, plus the face and the corner radius the skin asks
/// for.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Roles {
    /// The window's ground.
    pub bg: String,
    /// Sidebars and cards.
    pub panel: String,
    /// A step up from the panel: tab strips, headers.
    pub panel2: String,
    /// Two steps up: hovered rows, code, chips.
    pub panel3: String,
    /// Hairlines.
    pub line: String,
    /// Hairlines that should be seen: focused borders, boxes.
    pub line_bright: String,
    /// Labels, secondary text.
    pub muted: String,
    /// Body text.
    pub text: String,
    /// Titles and the active thing's text.
    pub bright: String,
    /// The active thing: marks, bars, underlines, the brand.
    pub accent: String,
    /// The accent where a whole surface takes it.
    pub accent_soft: String,
    /// Section titles, folders.
    pub gold: String,
    /// What is alive: links, checks, the online dot, the assistant.
    pub alive: String,
    /// Errors.
    pub red: String,
    /// The six semantic colours callouts and the terminal scheme name; derived from the
    /// roles unless a source sets them.
    pub green: String,
    /// See [`Roles::green`].
    pub yellow: String,
    /// See [`Roles::green`].
    pub blue: String,
    /// See [`Roles::green`].
    pub magenta: String,
    /// See [`Roles::green`].
    pub cyan: String,
    /// The face for every label; empty means the terminal font.
    pub font: String,
    /// Corner radius in pixels; the mock is square.
    pub radius: i32,
}

impl Default for Roles {
    fn default() -> Self {
        preset_roles(PRESETS[0])
    }
}

/// A built-in skin.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// The id the choice names.
    pub id: &'static str,
    /// The name Settings shows.
    pub title: &'static str,
    colours: [&'static str; 14],
}

/// The presets, the mock's first. The fourteen colours are the roles in declaration
/// order: bg, panel, panel2, panel3, line, line_bright, muted, text, bright, accent,
/// accent_soft, gold, alive, red.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "amber-phosphor",
        title: "Amber phosphor",
        colours: [
            "#090a08", "#0e100d", "#12150f", "#171a12", "#333820", "#656b32", "#747960", "#d7d8a8",
            "#fff1a6", "#ffb000", "#d68b00", "#ffd34f", "#69d8bb", "#e66e55",
        ],
    },
    Preset {
        id: "green-phosphor",
        title: "Green phosphor",
        colours: [
            "#060907", "#0a0f0b", "#0e140f", "#121a13", "#1f3324", "#3f6647", "#5f7a64", "#b9d6b3",
            "#e6ffe0", "#5cff9d", "#2fbf6f", "#c8f06a", "#7fd8ff", "#ff7a6e",
        ],
    },
    Preset {
        id: "ice",
        title: "Ice",
        colours: [
            "#070a10", "#0b0f17", "#0f141d", "#131a25", "#1f2a3a", "#3a4f6a", "#6b7b90", "#c5d1e0",
            "#eef5ff", "#62b6ff", "#3a86c8", "#ffd27a", "#6fe3c1", "#ff6f7d",
        ],
    },
    Preset {
        id: "paper",
        title: "Paper",
        colours: [
            "#f3eee2", "#ece6d6", "#e6dfcc", "#dfd7c1", "#cfc6ab", "#a89d7c", "#7d7660", "#3a3626",
            "#1e1b12", "#b8600d", "#8f4a0a", "#8a6a00", "#1c8a6b", "#c0392b",
        ],
    },
];

fn preset_roles(preset: Preset) -> Roles {
    let c = preset.colours;
    fill(Roles {
        bg: c[0].into(),
        panel: c[1].into(),
        panel2: c[2].into(),
        panel3: c[3].into(),
        line: c[4].into(),
        line_bright: c[5].into(),
        muted: c[6].into(),
        text: c[7].into(),
        bright: c[8].into(),
        accent: c[9].into(),
        accent_soft: c[10].into(),
        gold: c[11].into(),
        alive: c[12].into(),
        red: c[13].into(),
        green: String::new(),
        yellow: String::new(),
        blue: String::new(),
        magenta: String::new(),
        cyan: String::new(),
        font: String::new(),
        radius: 0,
    })
}

/// Derive whatever a source left empty from the roles it set.
fn fill(mut r: Roles) -> Roles {
    let dark = is_dark(&r.bg);
    let up = |hex: &str, amount: f32| {
        if dark {
            shade(hex, amount)
        } else {
            shade(hex, -amount)
        }
    };
    if r.panel.is_empty() {
        r.panel = up(&r.bg, 0.03);
    }
    if r.panel2.is_empty() {
        r.panel2 = up(&r.bg, 0.06);
    }
    if r.panel3.is_empty() {
        r.panel3 = up(&r.bg, 0.09);
    }
    if r.line.is_empty() {
        r.line = blend(&r.text, &r.bg, 0.18);
    }
    if r.line_bright.is_empty() {
        r.line_bright = blend(&r.text, &r.bg, 0.36);
    }
    if r.muted.is_empty() {
        r.muted = blend(&r.text, &r.bg, 0.55);
    }
    if r.bright.is_empty() {
        r.bright = up(&r.text, 0.35);
    }
    if r.accent_soft.is_empty() {
        r.accent_soft = blend(&r.accent, &r.bg, 0.72);
    }
    if r.gold.is_empty() {
        r.gold = blend(&r.accent, &r.bright, 0.55);
    }
    if r.alive.is_empty() {
        r.alive = blend(&r.accent, &r.text, 0.5);
    }
    if r.red.is_empty() {
        r.red = "#e66e55".into();
    }
    if r.green.is_empty() {
        r.green = r.alive.clone();
    }
    if r.yellow.is_empty() {
        r.yellow = r.gold.clone();
    }
    if r.blue.is_empty() {
        r.blue = r.accent.clone();
    }
    if r.magenta.is_empty() {
        r.magenta = r.accent_soft.clone();
    }
    if r.cyan.is_empty() {
        r.cyan = r.alive.clone();
    }
    r
}

/// The Omarchy desktop theme mapped onto the roles: the palette's four colours, the
/// Alacritty ANSI colours for gold (yellow), alive (cyan) and red, and everything else
/// derived from the ground and the text.
pub fn from_omarchy(palette: &Palette, ansi: &BTreeMap<String, String>) -> Roles {
    let pick = |key: &str| ansi.get(key).cloned().unwrap_or_default();
    fill(Roles {
        bg: palette.background.clone(),
        panel: String::new(),
        panel2: String::new(),
        panel3: String::new(),
        line: String::new(),
        line_bright: String::new(),
        muted: String::new(),
        text: palette.foreground.clone(),
        bright: String::new(),
        accent: palette.accent.clone(),
        accent_soft: String::new(),
        gold: pick("yellow"),
        alive: pick("cyan"),
        red: pick("red"),
        green: pick("green"),
        yellow: pick("yellow"),
        blue: pick("blue"),
        magenta: pick("magenta"),
        cyan: pick("cyan"),
        font: String::new(),
        radius: 0,
    })
}

/// `~/.config/rusty/themes`, where a user's own skins live as `<name>.toml`.
pub fn themes_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rusty/themes")
}

/// The theme files present, by stem, sorted.
pub fn theme_files() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(themes_dir())
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    (path.extension().and_then(|x| x.to_str()) == Some("toml"))
                        .then(|| path.file_stem()?.to_str().map(str::to_string))
                        .flatten()
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// A skin from a theme file: a `[colors]` table of role names (`bg`, `panel`, `panel2`,
/// `panel3`, `line`, `line_bright`, `muted`, `text`, `bright`, `accent`,
/// `accent_soft`, `gold`, `alive`, `red`, and the six semantic names), each `#rrggbb`,
/// and an optional `[type]` table with `font` and `radius`. Missing roles derive from
/// `bg`, `text` and `accent`, which are the three a file must give.
pub fn from_file(name: &str) -> Option<Roles> {
    let text = std::fs::read_to_string(themes_dir().join(format!("{name}.toml"))).ok()?;
    parse_theme(&text)
}

/// See [`from_file`].
pub fn parse_theme(text: &str) -> Option<Roles> {
    let table = text.parse::<toml::Table>().ok()?;
    let colours = table.get("colors").and_then(|c| c.as_table())?;
    let colour = |key: &str| -> String {
        colours
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|v| is_hex(v))
            .map(str::to_string)
            .unwrap_or_default()
    };
    let bg = colour("bg");
    let text = colour("text");
    let accent = colour("accent");
    if bg.is_empty() || text.is_empty() || accent.is_empty() {
        return None;
    }
    let kind = table.get("type").and_then(|t| t.as_table());
    let font = kind
        .and_then(|t| t.get("font"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let radius = kind
        .and_then(|t| t.get("radius"))
        .and_then(|v| v.as_integer())
        .unwrap_or(0)
        .clamp(0, 24) as i32;
    Some(fill(Roles {
        bg,
        panel: colour("panel"),
        panel2: colour("panel2"),
        panel3: colour("panel3"),
        line: colour("line"),
        line_bright: colour("line_bright"),
        muted: colour("muted"),
        text,
        bright: colour("bright"),
        accent,
        accent_soft: colour("accent_soft"),
        gold: colour("gold"),
        alive: colour("alive"),
        red: colour("red"),
        green: colour("green"),
        yellow: colour("yellow"),
        blue: colour("blue"),
        magenta: colour("magenta"),
        cyan: colour("cyan"),
        font,
        radius,
    }))
}

fn is_hex(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// The roles a choice names, and the title Settings shows for it. A preset or file
/// that does not exist falls back to the first preset, so a stale choice never leaves
/// the window unpainted.
pub fn resolve(
    choice: &Choice,
    palette: &Palette,
    ansi: &BTreeMap<String, String>,
) -> (Roles, String) {
    match choice.source.as_str() {
        "omarchy" => (from_omarchy(palette, ansi), "Follow Omarchy".to_string()),
        "file" => match from_file(&choice.name) {
            Some(roles) => (roles, choice.name.clone()),
            None => (preset_roles(PRESETS[0]), PRESETS[0].title.to_string()),
        },
        _ => {
            let preset = PRESETS
                .iter()
                .find(|p| p.id == choice.name)
                .copied()
                .unwrap_or(PRESETS[0]);
            (preset_roles(preset), preset.title.to_string())
        }
    }
}

/// Every skin the user can pick, as JSON for Settings: `[{source, name, title}]`.
pub fn choices_json() -> String {
    let mut list: Vec<serde_json::Value> = PRESETS
        .iter()
        .map(|p| serde_json::json!({"source": "preset", "name": p.id, "title": p.title}))
        .collect();
    list.push(serde_json::json!({"source": "omarchy", "name": "", "title": "Follow Omarchy"}));
    for name in theme_files() {
        list.push(serde_json::json!({"source": "file", "name": name, "title": name}));
    }
    serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
}

/// Every colour the shell and the renderer bind to, the roles and what derives from
/// them, under the names the QML already uses and the new ones.
pub fn tokens(r: &Roles) -> BTreeMap<String, String> {
    let dark = is_dark(&r.bg);
    let mut t: BTreeMap<String, String> = BTreeMap::new();
    let mut put = |key: &str, value: String| {
        t.insert(key.to_string(), value);
    };
    put("background", r.bg.clone());
    put("foreground", r.text.clone());
    put("accent", r.accent.clone());
    put("dark", if dark { "true" } else { "false" }.to_string());
    put("panel", r.panel.clone());
    put("panel2", r.panel2.clone());
    put("panel3", r.panel3.clone());
    put("surface", r.panel.clone());
    put("surface-alt", r.panel2.clone());
    put("line", r.line.clone());
    put("line-bright", r.line_bright.clone());
    put("muted", r.muted.clone());
    put("faint", blend(&r.muted, &r.bg, 0.72));
    put("bright", r.bright.clone());
    put("accent-soft", r.accent_soft.clone());
    put("gold", r.gold.clone());
    put("alive", r.alive.clone());
    put("red", r.red.clone());
    put("green", r.green.clone());
    put("yellow", r.yellow.clone());
    put("blue", r.blue.clone());
    put("magenta", r.magenta.clone());
    put("cyan", r.cyan.clone());
    put("link", r.alive.clone());
    put("selection", blend(&r.accent, &r.bg, 0.35));
    put("code", blend(&r.alive, &r.text, 0.35));
    put("code-bg", r.panel.clone());
    put("tag", r.accent.clone());
    put("tag-bg", blend(&r.accent, &r.bg, 0.14));
    put("mark", blend(&r.gold, &r.bg, 0.4));
    put("hover", r.panel3.clone());
    put("active", blend(&r.accent, &r.bg, 0.2));
    put("h1", r.bright.clone());
    put("h2", r.gold.clone());
    put("h3", r.gold.clone());
    put("h4", r.text.clone());
    put("h5", r.text.clone());
    put("h6", r.muted.clone());
    put("graph-line", r.line_bright.clone());
    put("graph-node", r.accent.clone());
    put("graph-node-tag", r.alive.clone());
    put("graph-node-attachment", r.gold.clone());
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_fill_and_resolve() {
        let (roles, title) = resolve(&Choice::default(), &Palette::default(), &BTreeMap::new());
        assert_eq!(title, "Amber phosphor");
        assert_eq!(roles.accent, "#ffb000");
        assert_eq!(roles.cyan, roles.alive);
        assert_eq!(roles.yellow, roles.gold);
        let unknown = Choice {
            source: "preset".into(),
            name: "nothing".into(),
            scanlines: false,
        };
        assert_eq!(
            resolve(&unknown, &Palette::default(), &BTreeMap::new()).1,
            "Amber phosphor"
        );
        for p in PRESETS {
            let r = preset_roles(*p);
            assert!(r.colours_are_hex(), "{}", p.id);
        }
    }

    #[test]
    fn omarchy_maps_onto_the_roles() {
        let mut ansi = BTreeMap::new();
        ansi.insert("yellow".to_string(), "#e0af68".to_string());
        ansi.insert("cyan".to_string(), "#7dcfff".to_string());
        ansi.insert("red".to_string(), "#f7768e".to_string());
        let roles = from_omarchy(&Palette::default(), &ansi);
        assert_eq!(roles.bg, "#1a1b26");
        assert_eq!(roles.gold, "#e0af68");
        assert_eq!(roles.alive, "#7dcfff");
        assert_eq!(roles.red, "#f7768e");
        assert!(!roles.panel.is_empty() && !roles.line.is_empty() && !roles.bright.is_empty());
        assert_eq!(roles.blue, roles.accent);
    }

    #[test]
    fn a_theme_file_needs_three_colours_and_derives_the_rest() {
        let roles = parse_theme(
            "[colors]\nbg = \"#101010\"\ntext = \"#e0e0e0\"\naccent = \"#ff8800\"\n[type]\nfont = \"Iosevka\"\nradius = 6\n",
        )
        .unwrap();
        assert_eq!(roles.font, "Iosevka");
        assert_eq!(roles.radius, 6);
        assert!(!roles.gold.is_empty() && !roles.alive.is_empty());
        assert!(parse_theme("[colors]\nbg = \"#101010\"\n").is_none());
        assert!(
            parse_theme("[colors]\nbg = \"red\"\ntext = \"#fff\"\naccent = \"#000000\"\n")
                .is_none()
        );
    }

    #[test]
    fn tokens_carry_the_old_names_and_the_new() {
        let t = tokens(&Roles::default());
        for key in [
            "background",
            "surface",
            "surface-alt",
            "line",
            "muted",
            "faint",
            "hover",
            "active",
            "panel3",
            "line-bright",
            "bright",
            "gold",
            "alive",
            "h1",
            "graph-node",
        ] {
            assert!(t.get(key).is_some_and(|v| v.starts_with('#')), "{key}");
        }
        assert_eq!(t["link"], t["alive"]);
        assert_eq!(t["surface"], t["panel"]);
    }

    impl Roles {
        fn colours_are_hex(&self) -> bool {
            [
                &self.bg,
                &self.panel,
                &self.panel2,
                &self.panel3,
                &self.line,
                &self.line_bright,
                &self.muted,
                &self.text,
                &self.bright,
                &self.accent,
                &self.accent_soft,
                &self.gold,
                &self.alive,
                &self.red,
                &self.green,
                &self.yellow,
                &self.blue,
                &self.magenta,
                &self.cyan,
            ]
            .iter()
            .all(|c| is_hex(c))
        }
    }
}
