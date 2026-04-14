use crossterm::style::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum TokenType {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Normal,
}

impl TokenType {
    pub fn color(&self) -> Color {
        match self {
            TokenType::Keyword => Color::Magenta,
            TokenType::String => Color::Green,
            TokenType::Comment => Color::DarkGrey,
            TokenType::Number => Color::Cyan,
            TokenType::Function => Color::Yellow,
            TokenType::Type => Color::Blue,
            TokenType::Normal => Color::White,
        }
    }
}
