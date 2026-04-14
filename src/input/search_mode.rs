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
            perform_search(editor);
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        KeyCode::Esc => {
            editor.mode = EditorMode::Normal;
            editor.message = Some("Search cancelled".to_string());
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

pub fn start_search(editor: &mut Editor) {
    editor.mode = EditorMode::Search;
    editor.input_buffer.clear();
    editor.message = Some("Search: ".to_string());
    editor.needs_full_redraw = true;
}

pub fn perform_search(editor: &mut Editor) {
    if editor.input_buffer.is_empty() {
        editor.message = Some("Search cancelled".to_string());
        return;
    }

    let search_query = editor.input_buffer.clone();
    let (_, height) = terminal::size().unwrap_or((80, 24));
    let visible_lines = editor.calculate_visible_lines(height);

    let search_result = {
        let pane = editor.active_pane_mut();
        pane.search_query = search_query.clone();

        let start_pos = if let Some((row, col)) = pane.last_search_pos {
            if col + 1 < pane.buffer.get_line(row).map(|l| l.len()).unwrap_or(0) {
                (row, col + 1)
            } else if row + 1 < pane.buffer.line_count() {
                (row + 1, 0)
            } else {
                (0, 0)
            }
        } else {
            (pane.cursor.y, pane.cursor.x)
        };

        pane.buffer.search(&pane.search_query, start_pos.0, start_pos.1)
    };

    if let Some((row, col)) = search_result {
        let pane = editor.active_pane_mut();
        pane.cursor.y = row;
        pane.cursor.x = col;
        pane.last_search_pos = Some((row, col));
        pane.adjust_scroll(visible_lines);
        editor.message = Some(format!("Found at line {}, col {}", row + 1, col + 1));
        editor.needs_full_redraw = true;
    } else {
        let pane = editor.active_pane_mut();
        pane.last_search_pos = None;
        editor.message = Some(format!("Not found: {}", search_query));
    }
}

pub fn find_next(editor: &mut Editor) {
    let search_query = editor.active_pane().search_query.clone();
    if !search_query.is_empty() {
        editor.input_buffer = search_query;
        perform_search(editor);
    }
}
