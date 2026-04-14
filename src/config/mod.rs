pub mod theme;
mod keybindings;

pub use theme::Theme;
pub use keybindings::{Keybindings, KeyAction};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub editor: EditorConfig,
    pub theme: String,
    pub colors: Option<Theme>,
    pub keybindings: Keybindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub show_line_numbers: bool,
    pub auto_indent: bool,
    pub word_wrap: bool,
    pub scroll_margin: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_spaces: true,
            show_line_numbers: true,
            auto_indent: true,
            word_wrap: false,
            scroll_margin: 3,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            theme: "default".to_string(),
            colors: None,
            keybindings: Keybindings::default(),
        }
    }
}

impl Config {
    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config path")?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("axis").join("config.toml"))
    }

    /// Get the resolved theme (custom colors or built-in)
    pub fn get_theme(&self) -> Theme {
        if let Some(custom) = &self.colors {
            custom.clone()
        } else {
            Theme::from_name(&self.theme)
        }
    }

    /// Generate a default config file content
    pub fn default_config_string() -> String {
        r##"# Axis Editor Configuration

[editor]
tab_size = 4
use_spaces = true
show_line_numbers = true
auto_indent = true
word_wrap = false
scroll_margin = 3

# Theme: "default", "monokai", "dracula", "solarized-dark", "nord"
theme = "default"

# Custom colors (optional - overrides theme)
# [colors]
# keyword = "#FF6188"
# string = "#FFD866"
# comment = "#6A737D"
# number = "#AE81FF"
# operator = "#F8F8F2"
# function = "#A6E22E"
# type = "#66D9EF"
# variable = "#F8F8F2"
# background = "#1E1E1E"
# foreground = "#F8F8F2"
# line_number = "#6A737D"
# selection = "#3E4451"
# status_bar = "#21252B"
# status_text = "#ABB2BF"

# Custom keybindings
# Format: action = "Key+Modifier" or action = ["Key1", "Key2"]
[keybindings]
# quit = "Ctrl+Q"
# save = "Ctrl+S"
# open = "Ctrl+O"
# find = "Ctrl+F"
# undo = "Ctrl+Z"
# redo = "Ctrl+Y"
# copy = "Ctrl+C"
# paste = "Ctrl+V"
# select_all = "Ctrl+A"
# command_palette = "Ctrl+P"
# goto_line = "Ctrl+G"
"##.to_string()
    }
}
