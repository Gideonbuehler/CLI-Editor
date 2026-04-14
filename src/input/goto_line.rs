use crate::editor::{Editor, EditorMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
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
                if let Ok(line_num) = editor.input_buffer.parse::<usize>() {
                    let target = line_num.saturating_sub(1);
                    let (_, height) = terminal::size()?;
                    let visible_lines = editor.calculate_visible_lines(height);

                    let mut success = false;
                    let mut out_of_range = false;

                    {
                        let pane = editor.active_pane_mut();
                        if target < pane.buffer.line_count() {
                            pane.cursor.y = target;
                            pane.cursor.x = 0;
                            pane.adjust_scroll(visible_lines);
                            success = true;
                        } else {
                            out_of_range = true;
                        }
                    }

                    if success {
                        editor.message = Some(format!("Went to line {}", line_num));
                    } else if out_of_range {
                        editor.message = Some("Line number out of range".to_string());
                    }
                } else {
                    editor.message = Some("Invalid line number".to_string());
                }
            }
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        KeyCode::Esc => {
            editor.mode = EditorMode::Normal;
            editor.message = Some("Goto line cancelled".to_string());
            editor.needs_full_redraw = true;
        }
        KeyCode::Backspace => {
            editor.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            if c.is_numeric() {
                editor.input_buffer.push(c);
            }
        }
        _ => {}
    }
    Ok(())
}
