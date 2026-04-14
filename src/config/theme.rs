use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub number: String,
    pub operator: String,
    pub function: String,
    pub r#type: String,
    pub variable: String,
    pub background: String,
    pub foreground: String,
    pub line_number: String,
    pub line_number_active: String,
    pub selection: String,
    pub status_bar: String,
    pub status_text: String,
    pub error: String,
    pub warning: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "monokai" => Self::monokai(),
            "dracula" => Self::dracula(),
            "solarized-dark" | "solarized" => Self::solarized_dark(),
            "nord" => Self::nord(),
            "gruvbox" => Self::gruvbox(),
            _ => Self::default_theme(),
        }
    }

    pub fn default_theme() -> Self {
        Self {
            keyword: "#569CD6".to_string(),     // Blue
            string: "#CE9178".to_string(),      // Orange
            comment: "#6A9955".to_string(),     // Green
            number: "#B5CEA8".to_string(),      // Light green
            operator: "#D4D4D4".to_string(),    // Light gray
            function: "#DCDCAA".to_string(),    // Yellow
            r#type: "#4EC9B0".to_string(),      // Teal
            variable: "#9CDCFE".to_string(),    // Light blue
            background: "#1E1E1E".to_string(),  // Dark gray
            foreground: "#D4D4D4".to_string(),  // Light gray
            line_number: "#858585".to_string(), // Gray
            line_number_active: "#C6C6C6".to_string(),
            selection: "#264F78".to_string(),   // Blue selection
            status_bar: "#007ACC".to_string(),  // Blue
            status_text: "#FFFFFF".to_string(), // White
            error: "#F44747".to_string(),       // Red
            warning: "#CCA700".to_string(),     // Yellow
        }
    }

    pub fn monokai() -> Self {
        Self {
            keyword: "#F92672".to_string(),     // Pink
            string: "#E6DB74".to_string(),      // Yellow
            comment: "#75715E".to_string(),     // Brown/gray
            number: "#AE81FF".to_string(),      // Purple
            operator: "#F8F8F2".to_string(),    // White
            function: "#A6E22E".to_string(),    // Green
            r#type: "#66D9EF".to_string(),      // Cyan
            variable: "#F8F8F2".to_string(),    // White
            background: "#272822".to_string(),  // Dark
            foreground: "#F8F8F2".to_string(),  // White
            line_number: "#75715E".to_string(),
            line_number_active: "#F8F8F2".to_string(),
            selection: "#49483E".to_string(),
            status_bar: "#75715E".to_string(),
            status_text: "#F8F8F2".to_string(),
            error: "#F92672".to_string(),
            warning: "#E6DB74".to_string(),
        }
    }

    pub fn dracula() -> Self {
        Self {
            keyword: "#FF79C6".to_string(),     // Pink
            string: "#F1FA8C".to_string(),      // Yellow
            comment: "#6272A4".to_string(),     // Purple/gray
            number: "#BD93F9".to_string(),      // Purple
            operator: "#F8F8F2".to_string(),    // White
            function: "#50FA7B".to_string(),    // Green
            r#type: "#8BE9FD".to_string(),      // Cyan
            variable: "#F8F8F2".to_string(),    // White
            background: "#282A36".to_string(),  // Dark
            foreground: "#F8F8F2".to_string(),  // White
            line_number: "#6272A4".to_string(),
            line_number_active: "#F8F8F2".to_string(),
            selection: "#44475A".to_string(),
            status_bar: "#44475A".to_string(),
            status_text: "#F8F8F2".to_string(),
            error: "#FF5555".to_string(),
            warning: "#FFB86C".to_string(),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            keyword: "#859900".to_string(),     // Green
            string: "#2AA198".to_string(),      // Cyan
            comment: "#586E75".to_string(),     // Gray
            number: "#D33682".to_string(),      // Magenta
            operator: "#839496".to_string(),    // Base0
            function: "#268BD2".to_string(),    // Blue
            r#type: "#B58900".to_string(),      // Yellow
            variable: "#839496".to_string(),    // Base0
            background: "#002B36".to_string(),  // Base03
            foreground: "#839496".to_string(),  // Base0
            line_number: "#586E75".to_string(),
            line_number_active: "#93A1A1".to_string(),
            selection: "#073642".to_string(),
            status_bar: "#073642".to_string(),
            status_text: "#93A1A1".to_string(),
            error: "#DC322F".to_string(),
            warning: "#B58900".to_string(),
        }
    }

    pub fn nord() -> Self {
        Self {
            keyword: "#81A1C1".to_string(),     // Blue
            string: "#A3BE8C".to_string(),      // Green
            comment: "#616E88".to_string(),     // Gray
            number: "#B48EAD".to_string(),      // Purple
            operator: "#D8DEE9".to_string(),    // White
            function: "#88C0D0".to_string(),    // Cyan
            r#type: "#8FBCBB".to_string(),      // Teal
            variable: "#D8DEE9".to_string(),    // White
            background: "#2E3440".to_string(),  // Dark
            foreground: "#D8DEE9".to_string(),  // White
            line_number: "#4C566A".to_string(),
            line_number_active: "#D8DEE9".to_string(),
            selection: "#434C5E".to_string(),
            status_bar: "#3B4252".to_string(),
            status_text: "#ECEFF4".to_string(),
            error: "#BF616A".to_string(),
            warning: "#EBCB8B".to_string(),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            keyword: "#FB4934".to_string(),     // Red
            string: "#B8BB26".to_string(),      // Green
            comment: "#928374".to_string(),     // Gray
            number: "#D3869B".to_string(),      // Purple
            operator: "#EBDBB2".to_string(),    // White
            function: "#FABD2F".to_string(),    // Yellow
            r#type: "#83A598".to_string(),      // Blue
            variable: "#EBDBB2".to_string(),    // White
            background: "#282828".to_string(),  // Dark
            foreground: "#EBDBB2".to_string(),  // White
            line_number: "#665C54".to_string(),
            line_number_active: "#EBDBB2".to_string(),
            selection: "#3C3836".to_string(),
            status_bar: "#3C3836".to_string(),
            status_text: "#EBDBB2".to_string(),
            error: "#FB4934".to_string(),
            warning: "#FABD2F".to_string(),
        }
    }

    /// Parse a hex color string to crossterm Color
    pub fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb { r, g, b };
            }
        }
        Color::White
    }

    // Convenience methods to get crossterm colors
    pub fn keyword_color(&self) -> Color { Self::parse_color(&self.keyword) }
    pub fn string_color(&self) -> Color { Self::parse_color(&self.string) }
    pub fn comment_color(&self) -> Color { Self::parse_color(&self.comment) }
    pub fn number_color(&self) -> Color { Self::parse_color(&self.number) }
    pub fn operator_color(&self) -> Color { Self::parse_color(&self.operator) }
    pub fn function_color(&self) -> Color { Self::parse_color(&self.function) }
    pub fn type_color(&self) -> Color { Self::parse_color(&self.r#type) }
    pub fn variable_color(&self) -> Color { Self::parse_color(&self.variable) }
    pub fn foreground_color(&self) -> Color { Self::parse_color(&self.foreground) }
    pub fn line_number_color(&self) -> Color { Self::parse_color(&self.line_number) }
    pub fn status_bar_color(&self) -> Color { Self::parse_color(&self.status_bar) }
    pub fn status_text_color(&self) -> Color { Self::parse_color(&self.status_text) }
    pub fn error_color(&self) -> Color { Self::parse_color(&self.error) }
}

/// List of available theme names
pub fn available_themes() -> Vec<&'static str> {
    vec!["default", "monokai", "dracula", "solarized-dark", "nord", "gruvbox"]
}
