use crate::command::registry::CommandRegistry;
use crate::editor::{Editor, EditorMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub fn process(editor: &mut Editor, key_event: KeyEvent) -> io::Result<()> {
    // Handle Ctrl+H as backspace (Linux terminal compatibility)
    if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == KeyCode::Char('h') {
        if editor.input_buffer.is_empty() {
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        } else {
            editor.input_buffer.pop();
            editor.palette_selected = 0;
        }
        return Ok(());
    }

    let registry = CommandRegistry::new();
    let filtered = registry.filter(&editor.input_buffer);
    let max_items = filtered.len();

    match key_event.code {
        KeyCode::Enter => {
            if !filtered.is_empty() && editor.palette_selected < filtered.len() {
                let command = filtered[editor.palette_selected];

                // Execute the selected command
                match (command.action)(editor, "") {
                    Ok(()) => {
                        // Only reset to Normal if the command didn't change the mode
                        if matches!(editor.mode, EditorMode::PaletteMode) {
                            editor.mode = EditorMode::Normal;
                            if editor.message.is_none() {
                                editor.message = Some(format!("Executed: {}", command.name));
                            }
                        }
                    }
                    Err(e) => {
                        editor.mode = EditorMode::Normal;
                        editor.message = Some(format!("Error: {}", e));
                    }
                }
                editor.needs_full_redraw = true;
            }
        }
        KeyCode::Esc => {
            editor.mode = EditorMode::Normal;
            editor.message = None;
            editor.needs_full_redraw = true;
        }
        KeyCode::Backspace => {
            if editor.input_buffer.is_empty() {
                editor.mode = EditorMode::Normal;
                editor.needs_full_redraw = true;
            } else {
                editor.input_buffer.pop();
                editor.palette_selected = 0;
            }
        }
        KeyCode::Up => {
            if editor.palette_selected > 0 {
                editor.palette_selected -= 1;
            }
        }
        KeyCode::Down => {
            if max_items > 0 && editor.palette_selected < max_items - 1 {
                editor.palette_selected += 1;
            }
        }
        KeyCode::Char(c) => {
            editor.input_buffer.push(c);
            editor.palette_selected = 0;
        }
        _ => {}
    }

    editor.needs_full_redraw = true;
    Ok(())
}
