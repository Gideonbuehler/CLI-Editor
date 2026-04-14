use super::{Language, TokenType};

#[derive(Clone)]
pub struct SyntaxHighlighter {
    pub language: Language,
}

impl SyntaxHighlighter {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn highlight_line(&self, line: &str) -> Vec<(String, TokenType)> {
        if self.language == Language::Plain {
            return vec![(line.to_string(), TokenType::Normal)];
        }

        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = line.chars().peekable();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_comment = false;

        while let Some(ch) = chars.next() {
            // Handle comments
            if !in_string && self.is_comment_start(ch, chars.peek().copied()) {
                if !current.is_empty() {
                    self.push_token(&mut tokens, current.clone());
                    current.clear();
                }
                in_comment = true;
                current.push(ch);
                if let Some(next) = chars.peek() {
                    if *next == '/' || *next == '*' {
                        current.push(chars.next().unwrap());
                    }
                }
                continue;
            }

            if in_comment {
                current.push(ch);
                continue;
            }

            // Handle strings
            if (ch == '"' || ch == '\'' || ch == '`') && !in_string {
                if !current.is_empty() {
                    self.push_token(&mut tokens, current.clone());
                    current.clear();
                }
                in_string = true;
                string_char = ch;
                current.push(ch);
                continue;
            }

            if in_string {
                current.push(ch);
                if ch == string_char && current.chars().rev().nth(1) != Some('\\') {
                    tokens.push((current.clone(), TokenType::String));
                    current.clear();
                    in_string = false;
                }
                continue;
            }

            // Handle numbers
            if ch.is_numeric() && (current.is_empty() || current.chars().all(|c| c.is_numeric() || c == '.')) {
                current.push(ch);
                continue;
            }

            // Handle identifiers and keywords
            if ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
                continue;
            }

            // We hit a separator
            if !current.is_empty() {
                self.push_token(&mut tokens, current.clone());
                current.clear();
            }

            // Add the separator as is
            tokens.push((ch.to_string(), TokenType::Normal));
        }

        // Handle remaining content
        if in_comment {
            tokens.push((current, TokenType::Comment));
        } else if in_string {
            tokens.push((current, TokenType::String));
        } else if !current.is_empty() {
            self.push_token(&mut tokens, current);
        }

        tokens
    }

    fn is_comment_start(&self, ch: char, next: Option<char>) -> bool {
        match self.language {
            Language::Rust | Language::C | Language::JavaScript | Language::Java => {
                ch == '/' && (next == Some('/') || next == Some('*'))
            }
            Language::Python | Language::Bash => ch == '#',
            Language::Plain => false,
        }
    }

    fn push_token(&self, tokens: &mut Vec<(String, TokenType)>, token: String) {
        let token_type = if self.language.keywords().contains(&token.as_str()) {
            TokenType::Keyword
        } else if self.language.types().contains(&token.as_str()) {
            TokenType::Type
        } else if token.chars().all(|c| c.is_numeric() || c == '.') {
            TokenType::Number
        } else {
            TokenType::Normal
        };

        tokens.push((token, token_type));
    }
}
