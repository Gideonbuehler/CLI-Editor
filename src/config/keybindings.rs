use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    Quit,
    ForceQuit,
    Save,
    SaveAs,
    Open,
    Find,
    FindNext,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    SelectAll,
    CommandPalette,
    CommandMode,
    GotoLine,
    SplitHorizontal,
    SplitVertical,
    NextPane,
    CloseSplit,
    ToggleLineNumbers,
    NewBuffer,
}

impl KeyAction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "quit" => Some(Self::Quit),
            "force_quit" | "force-quit" => Some(Self::ForceQuit),
            "save" => Some(Self::Save),
            "save_as" | "save-as" => Some(Self::SaveAs),
            "open" => Some(Self::Open),
            "find" | "search" => Some(Self::Find),
            "find_next" | "find-next" => Some(Self::FindNext),
            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "copy" => Some(Self::Copy),
            "cut" => Some(Self::Cut),
            "paste" => Some(Self::Paste),
            "select_all" | "select-all" => Some(Self::SelectAll),
            "command_palette" | "command-palette" | "palette" => Some(Self::CommandPalette),
            "command_mode" | "command-mode" => Some(Self::CommandMode),
            "goto_line" | "goto-line" | "goto" => Some(Self::GotoLine),
            "split_horizontal" | "split-horizontal" | "hsplit" => Some(Self::SplitHorizontal),
            "split_vertical" | "split-vertical" | "vsplit" => Some(Self::SplitVertical),
            "next_pane" | "next-pane" => Some(Self::NextPane),
            "close_split" | "close-split" => Some(Self::CloseSplit),
            "toggle_line_numbers" | "toggle-line-numbers" => Some(Self::ToggleLineNumbers),
            "new_buffer" | "new-buffer" | "new" => Some(Self::NewBuffer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Keybindings {
    #[serde(flatten)]
    pub bindings: HashMap<String, String>,
}

impl Keybindings {
    /// Get the action for a key event
    pub fn get_action(&self, key: &KeyEvent) -> Option<KeyAction> {
        let key_str = key_event_to_string(key);

        for (action_name, binding) in &self.bindings {
            if binding.eq_ignore_ascii_case(&key_str) {
                return KeyAction::from_str(action_name);
            }
        }

        // Fall back to defaults
        Self::default_action(key)
    }

    /// Get the default action for a key (built-in keybindings)
    fn default_action(key: &KeyEvent) -> Option<KeyAction> {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('q')) => Some(KeyAction::Quit),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => Some(KeyAction::Save),
            (KeyModifiers::CONTROL, KeyCode::Char('o')) => Some(KeyAction::Open),
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => Some(KeyAction::Find),
            (KeyModifiers::CONTROL, KeyCode::Char('n')) => Some(KeyAction::FindNext),
            (KeyModifiers::CONTROL, KeyCode::Char('z')) => Some(KeyAction::Undo),
            (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(KeyAction::Redo),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(KeyAction::Copy),
            (KeyModifiers::CONTROL, KeyCode::Char('x')) => Some(KeyAction::CloseSplit),
            (KeyModifiers::CONTROL, KeyCode::Char('v')) => Some(KeyAction::Paste),
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => Some(KeyAction::SelectAll),
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => Some(KeyAction::CommandPalette),
            (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(KeyAction::GotoLine),
            // Note: Ctrl+H is reserved for backspace on Linux terminals
            // Use :hsplit command or configure in keybindings
            (KeyModifiers::CONTROL, KeyCode::Char('\\')) => Some(KeyAction::SplitHorizontal),
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => Some(KeyAction::SplitVertical),
            (KeyModifiers::CONTROL, KeyCode::Char('w')) => Some(KeyAction::NextPane),
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(KeyAction::ToggleLineNumbers),
            _ => None,
        }
    }

    /// Check if a specific action is bound to a key
    pub fn is_bound(&self, action: KeyAction, key: &KeyEvent) -> bool {
        self.get_action(key) == Some(action)
    }
}

/// Convert a KeyEvent to a string representation like "Ctrl+S"
fn key_event_to_string(key: &KeyEvent) -> String {
    let mut parts = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_name = match key.code {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        _ => return String::new(),
    };

    parts.push(&key_name);
    parts.join("+")
}

/// Parse a key string like "Ctrl+S" into KeyEvent components
pub fn parse_key_string(s: &str) -> Option<(KeyModifiers, KeyCode)> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = KeyModifiers::NONE;
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "enter" | "return" => key_code = Some(KeyCode::Enter),
            "esc" | "escape" => key_code = Some(KeyCode::Esc),
            "backspace" => key_code = Some(KeyCode::Backspace),
            "tab" => key_code = Some(KeyCode::Tab),
            "delete" | "del" => key_code = Some(KeyCode::Delete),
            "insert" | "ins" => key_code = Some(KeyCode::Insert),
            "home" => key_code = Some(KeyCode::Home),
            "end" => key_code = Some(KeyCode::End),
            "pageup" | "pgup" => key_code = Some(KeyCode::PageUp),
            "pagedown" | "pgdn" => key_code = Some(KeyCode::PageDown),
            "up" => key_code = Some(KeyCode::Up),
            "down" => key_code = Some(KeyCode::Down),
            "left" => key_code = Some(KeyCode::Left),
            "right" => key_code = Some(KeyCode::Right),
            s if s.starts_with('f') && s.len() <= 3 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    key_code = Some(KeyCode::F(n));
                }
            }
            s if s.len() == 1 => {
                key_code = Some(KeyCode::Char(s.chars().next().unwrap().to_ascii_lowercase()));
            }
            _ => {}
        }
    }

    key_code.map(|code| (modifiers, code))
}
