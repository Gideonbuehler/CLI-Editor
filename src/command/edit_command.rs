use crate::buffer::TextBuffer;

#[derive(Clone, Debug)]
pub enum EditCommand {
    InsertChar { row: usize, col: usize, ch: char },
    DeleteChar { row: usize, col: usize, ch: char },
    InsertNewline { row: usize, col: usize },
    DeleteNewline { row: usize, deleted_line: String },
    ClearAll { old_content: Vec<String> },
}

impl EditCommand {
    pub fn undo(&self, buffer: &mut TextBuffer) {
        match self {
            EditCommand::InsertChar { row, col, .. } => {
                if *row < buffer.lines.len() && *col < buffer.lines[*row].len() {
                    buffer.lines[*row].remove(*col);
                }
            }
            EditCommand::DeleteChar { row, col, ch } => {
                if *row < buffer.lines.len() {
                    buffer.lines[*row].insert(*col, *ch);
                }
            }
            EditCommand::InsertNewline { row, col: _ } => {
                if *row + 1 < buffer.lines.len() {
                    let line = buffer.lines.remove(*row + 1);
                    buffer.lines[*row].push_str(&line);
                }
            }
            EditCommand::DeleteNewline { row, deleted_line } => {
                if *row < buffer.lines.len() {
                    let current_len = buffer.lines[*row].len();
                    let deleted_len = deleted_line.len();
                    buffer.lines[*row].truncate(current_len - deleted_len);
                    buffer.lines.insert(*row + 1, deleted_line.clone());
                }
            }
            EditCommand::ClearAll { old_content } => {
                buffer.lines = old_content.clone();
            }
        }
    }

    pub fn redo(&self, buffer: &mut TextBuffer) {
        match self {
            EditCommand::InsertChar { row, col, ch } => {
                if *row < buffer.lines.len() {
                    buffer.lines[*row].insert(*col, *ch);
                }
            }
            EditCommand::DeleteChar { row, col, .. } => {
                if *row < buffer.lines.len() && *col < buffer.lines[*row].len() {
                    buffer.lines[*row].remove(*col);
                }
            }
            EditCommand::InsertNewline { row, col } => {
                if *row < buffer.lines.len() {
                    let current_line = &buffer.lines[*row];
                    let new_line = current_line[*col..].to_string();
                    buffer.lines[*row].truncate(*col);
                    buffer.lines.insert(*row + 1, new_line);
                }
            }
            EditCommand::DeleteNewline { row, .. } => {
                if *row > 0 && *row < buffer.lines.len() {
                    let line = buffer.lines.remove(*row);
                    buffer.lines[*row - 1].push_str(&line);
                }
            }
            EditCommand::ClearAll { .. } => {
                buffer.lines = vec![String::new()];
            }
        }
    }
}
