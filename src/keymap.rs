//! Configurable key bindings.
//!
//! Upstream lazytail hard-codes every chord inside `handlers::input`.  This
//! module keeps that behaviour as the *default* table and lets a config file
//! override it:
//!
//! ```yaml
//! keys:
//!   follow: ["f", "F"]        # replace the default chord list for an action
//!   close_tab: []             # empty list unbinds the action entirely
//! ```
//!
//! Action names are the snake_case variants listed in [`Action::name`].
//! Unknown names are reported as an error at startup so a typo never silently
//! leaves the default in place.

use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

/// A single key chord, parsed from the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyStroke {
    Char(char),
    Ctrl(char),
    Alt(char),
    CtrlAlt(char),
    Space,
    Tab,
    BackTab,
    Enter,
    Esc,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

impl KeyStroke {
    /// Parse `"j"`, `"ctrl+d"`, `"C-d"`, `"alt+x"`, `"space"`, `"pagedown"`, …
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }

        // Emacs-style shorthand: "c-x" / "m-x".
        let bytes = raw.as_bytes();
        if bytes.len() == 3 && bytes[1] == b'-' {
            let c = raw.chars().nth(2)?;
            match bytes[0] as char {
                'c' | 'C' => return Some(Self::Ctrl(c.to_ascii_lowercase())),
                'm' | 'M' => return Some(Self::Alt(c.to_ascii_lowercase())),
                _ => {}
            }
        }

        let parts: Vec<&str> = raw.split('+').collect();
        if parts.len() > 1 {
            let (key, mods) = parts.split_last()?;
            let mut ctrl = false;
            let mut alt = false;
            for m in mods {
                match m.to_ascii_lowercase().as_str() {
                    "ctrl" | "control" | "c" => ctrl = true,
                    "alt" | "meta" | "m" => alt = true,
                    // Shift is encoded in the character itself (G vs g).
                    "shift" | "s" => {}
                    _ => return None,
                }
            }
            let key = key.trim();
            // Named keys with modifiers are not supported: nothing in the
            // default table needs them, and silently ignoring the modifier
            // would be worse than rejecting the binding.
            if key.chars().count() != 1 {
                return None;
            }
            let c = key.chars().next()?;
            return Some(match (ctrl, alt) {
                (true, true) => Self::CtrlAlt(c.to_ascii_lowercase()),
                (true, false) => Self::Ctrl(c.to_ascii_lowercase()),
                (false, true) => Self::Alt(c.to_ascii_lowercase()),
                (false, false) => Self::Char(c),
            });
        }

        let lower = raw.to_ascii_lowercase();
        let named = match lower.as_str() {
            "space" | "spc" => Some(Self::Space),
            "tab" => Some(Self::Tab),
            "backtab" | "btab" => Some(Self::BackTab),
            "enter" | "return" | "cr" => Some(Self::Enter),
            "esc" | "escape" => Some(Self::Esc),
            "backspace" | "bs" => Some(Self::Backspace),
            "delete" | "del" => Some(Self::Delete),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "pageup" | "pgup" => Some(Self::PageUp),
            "pagedown" | "pgdn" | "pgdown" => Some(Self::PageDown),
            _ => None,
        };
        if named.is_some() {
            return named;
        }

        if let Some(num) = lower.strip_prefix('f') {
            if let Ok(n) = num.parse::<u8>() {
                if (1..=24).contains(&n) {
                    return Some(Self::F(n));
                }
            }
        }

        let mut chars = raw.chars();
        let c = chars.next()?;
        if chars.next().is_none() {
            Some(Self::Char(c))
        } else {
            None
        }
    }

    /// Does this chord match a crossterm key event?
    pub fn matches(&self, code: KeyCode, mods: KeyModifiers) -> bool {
        let ctrl = mods.contains(KeyModifiers::CONTROL);
        let alt = mods.contains(KeyModifiers::ALT);
        match *self {
            Self::Char(c) => matches!(code, KeyCode::Char(x) if x == c) && !ctrl && !alt,
            Self::Ctrl(c) => {
                matches!(code, KeyCode::Char(x) if x.eq_ignore_ascii_case(&c)) && ctrl && !alt
            }
            Self::Alt(c) => {
                matches!(code, KeyCode::Char(x) if x.eq_ignore_ascii_case(&c)) && alt && !ctrl
            }
            Self::CtrlAlt(c) => {
                matches!(code, KeyCode::Char(x) if x.eq_ignore_ascii_case(&c)) && ctrl && alt
            }
            Self::Space => matches!(code, KeyCode::Char(' ')) && !ctrl && !alt,
            Self::Tab => code == KeyCode::Tab && !ctrl,
            Self::BackTab => code == KeyCode::BackTab,
            Self::Enter => code == KeyCode::Enter && !ctrl,
            Self::Esc => code == KeyCode::Esc,
            Self::Backspace => code == KeyCode::Backspace && !ctrl,
            Self::Delete => code == KeyCode::Delete && !ctrl,
            Self::Up => code == KeyCode::Up,
            Self::Down => code == KeyCode::Down,
            Self::Left => code == KeyCode::Left,
            Self::Right => code == KeyCode::Right,
            Self::Home => code == KeyCode::Home,
            Self::End => code == KeyCode::End,
            Self::PageUp => code == KeyCode::PageUp,
            Self::PageDown => code == KeyCode::PageDown,
            Self::F(n) => code == KeyCode::F(n),
        }
    }

    /// Human-readable form used in error messages and the help overlay.
    pub fn display(&self) -> String {
        match *self {
            Self::Char(c) => c.to_string(),
            Self::Ctrl(c) => format!("ctrl+{c}"),
            Self::Alt(c) => format!("alt+{c}"),
            Self::CtrlAlt(c) => format!("ctrl+alt+{c}"),
            Self::Space => "space".to_string(),
            Self::Tab => "tab".to_string(),
            Self::BackTab => "backtab".to_string(),
            Self::Enter => "enter".to_string(),
            Self::Esc => "esc".to_string(),
            Self::Backspace => "backspace".to_string(),
            Self::Delete => "delete".to_string(),
            Self::Up => "up".to_string(),
            Self::Down => "down".to_string(),
            Self::Left => "left".to_string(),
            Self::Right => "right".to_string(),
            Self::Home => "home".to_string(),
            Self::End => "end".to_string(),
            Self::PageUp => "pageup".to_string(),
            Self::PageDown => "pagedown".to_string(),
            Self::F(n) => format!("f{n}"),
        }
    }
}

/// Something a key can be bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Shared / normal mode
    Quit,
    ScrollDown,
    ScrollUp,
    ViewportDown,
    ViewportUp,
    PageDown,
    PageUp,
    HalfPageDown,
    HalfPageUp,
    JumpToStart,
    JumpToEnd,
    ToggleFollow,
    StartFilter,
    StartLineJump,
    ShowHelp,
    EnterZMode,
    ToggleLineExpansion,
    CollapseAll,
    ToggleRaw,
    ToggleTimestamps,
    ToggleWrap,
    CopyLine,
    CloseTab,
    RefreshCombined,
    ClearFilter,
    FocusSourcePanel,
    SelectTab(u8),
    // z-pending mode
    ZCenter,
    ZTop,
    ZBottom,
    ZCancel,
    // Source panel mode
    PanelUp,
    PanelDown,
    PanelToggleExpand,
    PanelSelect,
    PanelCloseTab,
    PanelCopyPath,
    PanelUnfocus,
}

/// Which input mode an action lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindMode {
    Normal,
    ZPending,
    Panel,
}

impl Action {
    /// Stable config-file name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::ScrollDown => "scroll_down",
            Self::ScrollUp => "scroll_up",
            Self::ViewportDown => "viewport_down",
            Self::ViewportUp => "viewport_up",
            Self::PageDown => "page_down",
            Self::PageUp => "page_up",
            Self::HalfPageDown => "half_page_down",
            Self::HalfPageUp => "half_page_up",
            Self::JumpToStart => "jump_start",
            Self::JumpToEnd => "jump_end",
            Self::ToggleFollow => "follow",
            Self::StartFilter => "filter",
            Self::StartLineJump => "line_jump",
            Self::ShowHelp => "help",
            Self::EnterZMode => "z_mode",
            Self::ToggleLineExpansion => "toggle_line",
            Self::CollapseAll => "collapse_all",
            Self::ToggleRaw => "raw",
            Self::ToggleTimestamps => "timestamps",
            Self::ToggleWrap => "wrap",
            Self::CopyLine => "copy_line",
            Self::CloseTab => "close_tab",
            Self::RefreshCombined => "refresh",
            Self::ClearFilter => "clear_filter",
            Self::FocusSourcePanel => "source_panel",
            Self::SelectTab(n) => match n {
                1 => "tab_1",
                2 => "tab_2",
                3 => "tab_3",
                4 => "tab_4",
                5 => "tab_5",
                6 => "tab_6",
                7 => "tab_7",
                8 => "tab_8",
                _ => "tab_9",
            },
            Self::ZCenter => "z_center",
            Self::ZTop => "z_top",
            Self::ZBottom => "z_bottom",
            Self::ZCancel => "z_cancel",
            Self::PanelUp => "panel_up",
            Self::PanelDown => "panel_down",
            Self::PanelToggleExpand => "panel_toggle",
            Self::PanelSelect => "panel_select",
            Self::PanelCloseTab => "panel_close",
            Self::PanelCopyPath => "panel_copy_path",
            Self::PanelUnfocus => "panel_unfocus",
        }
    }

    /// All action names known to the config parser.
    pub fn names() -> Vec<&'static str> {
        let mut out = Vec::new();
        for (_, action, _) in DEFAULT_BINDINGS {
            if !out.contains(&action.name()) {
                out.push(action.name());
            }
        }
        out
    }
}

/// The default table: exactly the chords upstream hard-codes, plus the extra
/// aliases this fork ships (`F` follow, `h` help, `ctrl+d`/`ctrl+u`,
/// `ctrl+f`/`ctrl+b`, `ctrl+k`, `esc`).
///
/// Each entry is (mode, action, default chords). An action may appear once per
/// mode; a config override replaces its chords in *every* mode it is bound in.
pub const DEFAULT_BINDINGS: &[(BindMode, Action, &[&str])] = &[
    (BindMode::Normal, Action::Quit, &["q", "ctrl+c"]),
    (BindMode::Normal, Action::ViewportDown, &["ctrl+e"]),
    (BindMode::Normal, Action::ViewportUp, &["ctrl+y"]),
    (BindMode::Normal, Action::ScrollDown, &["j", "down"]),
    (BindMode::Normal, Action::ScrollUp, &["k", "up"]),
    (BindMode::Normal, Action::PageDown, &["pagedown", "ctrl+f"]),
    (BindMode::Normal, Action::PageUp, &["pageup", "ctrl+b"]),
    (BindMode::Normal, Action::HalfPageDown, &["ctrl+d"]),
    (BindMode::Normal, Action::HalfPageUp, &["ctrl+u"]),
    (BindMode::Normal, Action::JumpToStart, &["g"]),
    (BindMode::Normal, Action::JumpToEnd, &["G"]),
    (BindMode::Normal, Action::ToggleFollow, &["f", "F"]),
    (BindMode::Normal, Action::StartFilter, &["/"]),
    (BindMode::Normal, Action::StartLineJump, &[":"]),
    (BindMode::Normal, Action::ShowHelp, &["?", "h"]),
    (BindMode::Normal, Action::EnterZMode, &["z"]),
    (BindMode::Normal, Action::ToggleLineExpansion, &["space"]),
    (BindMode::Normal, Action::CollapseAll, &["c"]),
    (BindMode::Normal, Action::ToggleRaw, &["r"]),
    (BindMode::Normal, Action::ToggleTimestamps, &["t"]),
    (BindMode::Normal, Action::ToggleWrap, &["w"]),
    (BindMode::Normal, Action::CopyLine, &["y"]),
    (BindMode::Normal, Action::CloseTab, &["x", "ctrl+w"]),
    (BindMode::Normal, Action::RefreshCombined, &["R"]),
    (BindMode::Normal, Action::ClearFilter, &["esc", "ctrl+k"]),
    (BindMode::Normal, Action::FocusSourcePanel, &["tab"]),
    (BindMode::Normal, Action::SelectTab(1), &["1"]),
    (BindMode::Normal, Action::SelectTab(2), &["2"]),
    (BindMode::Normal, Action::SelectTab(3), &["3"]),
    (BindMode::Normal, Action::SelectTab(4), &["4"]),
    (BindMode::Normal, Action::SelectTab(5), &["5"]),
    (BindMode::Normal, Action::SelectTab(6), &["6"]),
    (BindMode::Normal, Action::SelectTab(7), &["7"]),
    (BindMode::Normal, Action::SelectTab(8), &["8"]),
    (BindMode::Normal, Action::SelectTab(9), &["9"]),
    (BindMode::ZPending, Action::ZCenter, &["z"]),
    (BindMode::ZPending, Action::ZTop, &["t"]),
    (BindMode::ZPending, Action::ZBottom, &["b"]),
    (BindMode::ZPending, Action::ZCancel, &["esc"]),
    (BindMode::Panel, Action::Quit, &["q", "ctrl+c"]),
    (BindMode::Panel, Action::ShowHelp, &["?"]),
    (BindMode::Panel, Action::PanelUp, &["up", "k"]),
    (BindMode::Panel, Action::PanelDown, &["down", "j"]),
    (BindMode::Panel, Action::PanelToggleExpand, &["space"]),
    (BindMode::Panel, Action::PanelSelect, &["enter"]),
    (BindMode::Panel, Action::PanelCloseTab, &["x", "ctrl+w"]),
    (BindMode::Panel, Action::PanelCopyPath, &["y"]),
    (BindMode::Panel, Action::PanelUnfocus, &["esc", "tab"]),
];

/// Resolved bindings, one lookup table per input mode.
#[derive(Debug, Clone)]
pub struct Keymap {
    normal: HashMap<KeyStroke, Action>,
    z_pending: HashMap<KeyStroke, Action>,
    panel: HashMap<KeyStroke, Action>,
    /// Human-readable notes about chords bound to more than one action.
    conflicts: Vec<String>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::build(&HashMap::new()).expect("default keymap is valid")
    }
}

impl Keymap {
    /// Build a keymap from the default table with user overrides applied.
    ///
    /// A user list *replaces* the default list for that action; an empty list
    /// removes every chord bound to it.
    pub fn build(overrides: &HashMap<String, Vec<String>>) -> Result<Self, String> {
        // Parse user overrides once, validating every action name and chord.
        let mut custom: HashMap<String, Vec<KeyStroke>> = HashMap::new();
        for (name, raws) in overrides {
            if !Action::names().contains(&name.as_str()) {
                return Err(format!(
                    "unknown key action `{name}` (known: {})",
                    Action::names().join(", ")
                ));
            }
            let mut parsed = Vec::new();
            for raw in raws {
                let stroke = KeyStroke::parse(raw)
                    .ok_or_else(|| format!("cannot parse key `{raw}` for action `{name}`"))?;
                if !parsed.contains(&stroke) {
                    parsed.push(stroke);
                }
            }
            custom.insert(name.clone(), parsed);
        }

        let mut map = Self {
            normal: HashMap::new(),
            z_pending: HashMap::new(),
            panel: HashMap::new(),
            conflicts: Vec::new(),
        };

        for (mode, action, defaults) in DEFAULT_BINDINGS {
            let strokes = match custom.get(action.name()) {
                Some(parsed) => parsed.clone(),
                None => defaults
                    .iter()
                    .filter_map(|raw| KeyStroke::parse(raw))
                    .collect(),
            };
            let table = match mode {
                BindMode::Normal => &mut map.normal,
                BindMode::ZPending => &mut map.z_pending,
                BindMode::Panel => &mut map.panel,
            };
            for stroke in strokes {
                if let Some(previous) = table.insert(stroke, *action) {
                    if previous != *action {
                        map.conflicts.push(format!(
                            "`{}` is bound to both `{}` and `{}` in {:?} mode; `{}` wins",
                            stroke.display(),
                            previous.name(),
                            action.name(),
                            mode,
                            action.name()
                        ));
                    }
                }
            }
        }

        Ok(map)
    }

    /// Look up a chord in the given mode.
    pub fn resolve(&self, mode: BindMode, code: KeyCode, mods: KeyModifiers) -> Option<Action> {
        let table = match mode {
            BindMode::Normal => &self.normal,
            BindMode::ZPending => &self.z_pending,
            BindMode::Panel => &self.panel,
        };
        table
            .iter()
            .find(|(stroke, _)| stroke.matches(code, mods))
            .map(|(_, action)| *action)
    }

    /// Notes about chords that ended up bound to more than one action.
    pub fn conflicts(&self) -> &[String] {
        &self.conflicts
    }

    /// Every chord bound to an action, for the help overlay and verbose output.
    pub fn chords_for(&self, mode: BindMode, action: Action) -> Vec<String> {
        let table = match mode {
            BindMode::Normal => &self.normal,
            BindMode::ZPending => &self.z_pending,
            BindMode::Panel => &self.panel,
        };
        let mut out: Vec<String> = table
            .iter()
            .filter(|(_, bound)| **bound == action)
            .map(|(stroke, _)| stroke.display())
            .collect();
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(raw: &str) -> KeyStroke {
        KeyStroke::parse(raw).unwrap_or_else(|| panic!("cannot parse {raw}"))
    }

    #[test]
    fn parses_plain_chars_case_sensitively() {
        assert_eq!(stroke("g"), KeyStroke::Char('g'));
        assert_eq!(stroke("G"), KeyStroke::Char('G'));
        assert_ne!(stroke("g"), stroke("G"));
    }

    #[test]
    fn parses_modifiers() {
        assert_eq!(stroke("ctrl+d"), KeyStroke::Ctrl('d'));
        assert_eq!(stroke("C-d"), KeyStroke::Ctrl('d'));
        assert_eq!(stroke("alt+x"), KeyStroke::Alt('x'));
        assert_eq!(stroke("m-x"), KeyStroke::Alt('x'));
        assert_eq!(stroke("ctrl+alt+k"), KeyStroke::CtrlAlt('k'));
    }

    #[test]
    fn parses_named_keys() {
        assert_eq!(stroke("space"), KeyStroke::Space);
        assert_eq!(stroke("pagedown"), KeyStroke::PageDown);
        assert_eq!(stroke("pgdn"), KeyStroke::PageDown);
        assert_eq!(stroke("esc"), KeyStroke::Esc);
        assert_eq!(stroke("f5"), KeyStroke::F(5));
        assert_eq!(KeyStroke::parse("nonsense"), None);
        assert_eq!(KeyStroke::parse(""), None);
    }

    #[test]
    fn default_keymap_matches_upstream_chords() {
        let map = Keymap::default();
        let normal = |c: char| map.resolve(BindMode::Normal, KeyCode::Char(c), KeyModifiers::NONE);
        assert_eq!(normal('q'), Some(Action::Quit));
        assert_eq!(normal('f'), Some(Action::ToggleFollow));
        assert_eq!(normal('F'), Some(Action::ToggleFollow));
        assert_eq!(normal('/'), Some(Action::StartFilter));
        assert_eq!(normal(':'), Some(Action::StartLineJump));
        assert_eq!(normal('h'), Some(Action::ShowHelp));
        assert_eq!(normal('c'), Some(Action::CollapseAll));
        assert_eq!(normal('x'), Some(Action::CloseTab));
        assert_eq!(normal('3'), Some(Action::SelectTab(3)));
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Char('f'), KeyModifiers::CONTROL),
            Some(Action::PageDown)
        );
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Esc, KeyModifiers::NONE),
            Some(Action::ClearFilter)
        );
    }

    #[test]
    fn z_and_panel_modes_are_separate() {
        let map = Keymap::default();
        assert_eq!(
            map.resolve(BindMode::ZPending, KeyCode::Char('z'), KeyModifiers::NONE),
            Some(Action::ZCenter)
        );
        assert_eq!(
            map.resolve(BindMode::Panel, KeyCode::Char('j'), KeyModifiers::NONE),
            Some(Action::PanelDown)
        );
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Char('j'), KeyModifiers::NONE),
            Some(Action::ScrollDown)
        );
    }

    #[test]
    fn overrides_replace_defaults() {
        let mut overrides = HashMap::new();
        overrides.insert("follow".to_string(), vec!["F".to_string()]);
        let map = Keymap::build(&overrides).unwrap();
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Char('f'), KeyModifiers::NONE),
            None
        );
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Char('F'), KeyModifiers::NONE),
            Some(Action::ToggleFollow)
        );
    }

    #[test]
    fn empty_list_unbinds() {
        let mut overrides = HashMap::new();
        overrides.insert("close_tab".to_string(), vec![]);
        let map = Keymap::build(&overrides).unwrap();
        assert_eq!(
            map.resolve(BindMode::Normal, KeyCode::Char('x'), KeyModifiers::NONE),
            None
        );
    }

    #[test]
    fn unknown_action_is_rejected() {
        let mut overrides = HashMap::new();
        overrides.insert("folow".to_string(), vec!["f".to_string()]);
        let err = Keymap::build(&overrides).unwrap_err();
        assert!(err.contains("unknown key action `folow`"), "{err}");
    }

    #[test]
    fn bad_chord_is_rejected() {
        let mut overrides = HashMap::new();
        overrides.insert("follow".to_string(), vec!["ctrl+".to_string()]);
        assert!(Keymap::build(&overrides).is_err());
    }
}
