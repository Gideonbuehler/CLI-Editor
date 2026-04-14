use crate::buffer::{Cursor, TextBuffer};
use crate::command::{EditCommand, UndoTree};
use crate::syntax::{Language, SyntaxHighlighter};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Pane {
    pub buffer: TextBuffer,
    pub cursor: Cursor,
    pub offset_y: usize,
    pub offset_x: usize,
    pub undo_tree: UndoTree,
    pub current_file: Option<PathBuf>,
    pub modified: bool,
    pub search_query: String,
    pub last_search_pos: Option<(usize, usize)>,
    pub highlighter: SyntaxHighlighter,
    pub selection_start: Option<(usize, usize)>,
}

impl Pane {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            cursor: Cursor { x: 0, y: 0 },
            offset_y: 0,
            offset_x: 0,
            undo_tree: UndoTree::new(),
            current_file: None,
            modified: false,
            search_query: String::new(),
            last_search_pos: None,
            highlighter: SyntaxHighlighter::new(Language::Plain),
            selection_start: None,
        }
    }

    pub fn execute_command(&mut self, command: EditCommand) {
        self.undo_tree.execute(command, &mut self.buffer);
        self.modified = true;
    }

    pub fn undo(&mut self) {
        if self.undo_tree.undo(&mut self.buffer) {
            self.modified = self.undo_tree.can_undo();
        }
    }

    pub fn redo(&mut self) {
        if self.undo_tree.redo(&mut self.buffer) {
            self.modified = true;
        }
    }

    pub fn adjust_scroll(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            self.offset_y = 0;
            return;
        }

        let max_offset = self.buffer.line_count().saturating_sub(visible_lines);

        if self.cursor.y < self.offset_y {
            self.offset_y = self.cursor.y;
        } else if self.cursor.y >= self.offset_y + visible_lines {
            self.offset_y = self.cursor.y - visible_lines + 1;
        }

        self.offset_y = self.offset_y.min(max_offset);
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Returns (start, end) positions ordered so start <= end
    pub fn selection_bounds(&self) -> Option<((usize, usize), (usize, usize))> {
        self.selection_start.map(|start| {
            let end = (self.cursor.y, self.cursor.x);
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let ((start_row, start_col), (end_row, end_col)) = self.selection_bounds()?;
        let mut result = String::new();

        for row in start_row..=end_row {
            if let Some(line) = self.buffer.get_line(row) {
                let from = if row == start_row { start_col } else { 0 };
                let to = if row == end_row { end_col.min(line.len()) } else { line.len() };
                if from <= to && from <= line.len() {
                    result.push_str(&line[from..to]);
                }
                if row < end_row {
                    result.push('\n');
                }
            }
        }

        Some(result)
    }

    pub fn delete_selection(&mut self) {
        let bounds = match self.selection_bounds() {
            Some(b) => b,
            None => return,
        };
        let ((start_row, start_col), (end_row, end_col)) = bounds;

        // Work backwards to avoid invalidating positions
        if start_row == end_row {
            // Single line: delete chars from end_col back to start_col
            if let Some(line) = self.buffer.get_line(start_row) {
                let end = end_col.min(line.len());
                for col in (start_col..end).rev() {
                    if let Some(ch) = self.buffer.get_line(start_row).and_then(|l| l.chars().nth(col)) {
                        let command = EditCommand::DeleteChar {
                            row: start_row,
                            col,
                            ch,
                        };
                        self.execute_command(command);
                    }
                }
            }
        } else {
            // Multi-line deletion:
            // 1. Delete chars on end_row from col 0..end_col, then merge end_row into previous
            // 2. Delete all middle lines
            // 3. Delete chars on start_row from start_col..end

            // First: delete the tail of the end line (chars 0..end_col)
            if let Some(line) = self.buffer.get_line(end_row) {
                let end = end_col.min(line.len());
                for col in (0..end).rev() {
                    if let Some(ch) = self.buffer.get_line(end_row).and_then(|l| l.chars().nth(col)) {
                        let command = EditCommand::DeleteChar {
                            row: end_row,
                            col,
                            ch,
                        };
                        self.execute_command(command);
                    }
                }
            }

            // Merge end_row with the line above it, and delete middle lines
            // Delete lines from end_row down to start_row+1
            for row in (start_row + 1..=end_row).rev() {
                if let Some(deleted_line) = self.buffer.get_line(row).cloned() {
                    let command = EditCommand::DeleteNewline {
                        row,
                        deleted_line,
                    };
                    self.execute_command(command);
                }
            }

            // Delete remaining chars on start_row from start_col..end
            if let Some(line) = self.buffer.get_line(start_row) {
                let line_len = line.len();
                let end = line_len.min(start_col + (line_len - start_col));
                for col in (start_col..end).rev() {
                    if let Some(ch) = self.buffer.get_line(start_row).and_then(|l| l.chars().nth(col)) {
                        let command = EditCommand::DeleteChar {
                            row: start_row,
                            col,
                            ch,
                        };
                        self.execute_command(command);
                    }
                }
            }
        }

        // Move cursor to selection start
        self.cursor.y = start_row;
        self.cursor.x = start_col;
        self.selection_start = None;
    }
}
