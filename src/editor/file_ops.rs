use super::{Editor, EditorMode};
use crate::buffer::{Cursor, TextBuffer};
use crate::syntax::{Language, SyntaxHighlighter};
use std::fs;
use std::io;
use std::path::PathBuf;

impl Editor {
    pub fn save_file(&mut self) -> io::Result<()> {
        let pane = self.active_pane_mut();
        if let Some(path) = &pane.current_file.clone() {
            fs::write(path, pane.buffer.to_string())?;
            pane.modified = false;
            self.message = Some(format!("Saved to {}", path.display()));
            Ok(())
        } else {
            self.mode = EditorMode::SavePrompt;
            self.input_buffer.clear();
            self.message = Some("Enter filename: ".to_string());
            self.needs_full_redraw = true;
            Ok(())
        }
    }

    pub fn save_file_as(&mut self, filename: String) -> io::Result<()> {
        let path = PathBuf::from(filename);
        let pane = self.active_pane_mut();
        fs::write(&path, pane.buffer.to_string())?;
        pane.current_file = Some(path.clone());
        pane.modified = false;
        self.message = Some(format!("Saved to {}", path.display()));
        Ok(())
    }

    pub fn open_file(&mut self, filename: String) -> io::Result<()> {
        let path = PathBuf::from(filename);
        let content = fs::read_to_string(&path)?;
        let pane = self.active_pane_mut();
        pane.buffer = TextBuffer::from_string(content);
        pane.current_file = Some(path.clone());
        pane.modified = false;
        pane.cursor = Cursor { x: 0, y: 0 };
        pane.offset_y = 0;
        pane.undo_tree.clear();

        // Detect language from file extension
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                pane.highlighter = SyntaxHighlighter::new(Language::from_extension(ext_str));
            }
        }

        self.message = Some(format!("Opened {}", path.display()));
        self.needs_full_redraw = true;
        Ok(())
    }
}
