#[derive(Clone)]
pub struct TextBuffer {
    pub lines: Vec<String>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn from_string(content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if row < self.lines.len() {
            self.lines[row].insert(col, ch);
        }
    }

    pub fn delete_char(&mut self, row: usize, col: usize) -> Option<char> {
        if row < self.lines.len() && col > 0 && col <= self.lines[row].len() {
            Some(self.lines[row].remove(col - 1))
        } else {
            None
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if row < self.lines.len() {
            let current_line = &self.lines[row];
            let new_line = current_line[col..].to_string();
            self.lines[row].truncate(col);
            self.lines.insert(row + 1, new_line);
        }
    }

    pub fn delete_newline(&mut self, row: usize) -> Option<String> {
        if row > 0 && row < self.lines.len() {
            let line = self.lines.remove(row);
            self.lines[row - 1].push_str(&line);
            Some(line)
        } else {
            None
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, row: usize) -> Option<&String> {
        self.lines.get(row)
    }

    pub fn search(&self, query: &str, start_row: usize, start_col: usize) -> Option<(usize, usize)> {
        if query.is_empty() {
            return None;
        }

        // Search from current position to end
        for row in start_row..self.lines.len() {
            let search_col = if row == start_row { start_col } else { 0 };
            if let Some(col) = self.lines[row][search_col..].find(query) {
                return Some((row, search_col + col));
            }
        }

        // Wrap around: search from beginning to start position
        for row in 0..=start_row {
            let end_col = if row == start_row { start_col } else { self.lines[row].len() };
            if let Some(col) = self.lines[row][..end_col].find(query) {
                return Some((row, col));
            }
        }

        None
    }
}
