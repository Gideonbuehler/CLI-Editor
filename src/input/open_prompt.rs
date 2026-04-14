use crate::editor::{Editor, EditorMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub fn process(editor: &mut Editor, key_event: KeyEvent) -> io::Result<()> {
    // Handle Ctrl+H as backspace (Linux terminal compatibility)
    if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('h') {
        editor.input_buffer.pop();
        return Ok(());
    }

    match key_event.code {
        KeyCode::Enter => {
            if !editor.input_buffer.is_empty() {
                if let Err(e) = editor.open_file(editor.input_buffer.clone()) {
                    editor.message = Some(format!("Error opening: {}", e));
                }
            }
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        KeyCode::Esc => {
            editor.mode = EditorMode::Normal;
            editor.message = Some("Open cancelled".to_string());
            editor.needs_full_redraw = true;
        }
        KeyCode::Backspace => {
            editor.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            editor.input_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}
