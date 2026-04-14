use crate::editor::{Editor, EditorMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub fn process(editor: &mut Editor, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Navigation
        KeyEvent {
            code: KeyCode::Up,
            ..
        } | KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            editor.file_browser.move_up();
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        } | KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            editor.file_browser.move_down();
            editor.needs_full_redraw = true;
        }
        // Expand/collapse
        KeyEvent {
            code: KeyCode::Right,
            ..
        } | KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            editor.file_browser.expand();
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Left,
            ..
        } | KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            editor.file_browser.collapse();
            editor.needs_full_redraw = true;
        }
        // Toggle expand/collapse or open file
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            if let Some(entry) = editor.file_browser.selected_entry() {
                if entry.is_dir {
                    editor.file_browser.toggle_expand();
                    editor.needs_full_redraw = true;
                } else {
                    // Open the file
                    let path = entry.path.to_string_lossy().to_string();
                    editor.mode = EditorMode::Normal;
                    editor.open_file(path)?;
                    editor.needs_full_redraw = true;
                }
            }
        }
        // Toggle expand with space
        KeyEvent {
            code: KeyCode::Char(' '),
            ..
        } => {
            editor.file_browser.toggle_expand();
            editor.needs_full_redraw = true;
        }
        // Go up to parent directory (change root)
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            editor.file_browser.go_up_directory();
            editor.needs_full_redraw = true;
        }
        // Close file browser / return to editor
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        // Ctrl+B to toggle file browser off
        KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.file_browser.toggle();
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        // Tab to switch focus back to editor (keeping browser visible)
        KeyEvent {
            code: KeyCode::Tab,
            ..
        } => {
            editor.mode = EditorMode::Normal;
            editor.needs_full_redraw = true;
        }
        // Refresh file list
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.file_browser.refresh();
            editor.needs_full_redraw = true;
            editor.message = Some("File browser refreshed".to_string());
        }
        // Open file in new tab
        KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            if let Some(path) = editor.file_browser.selected_file_path() {
                editor.new_tab();
                editor.mode = EditorMode::Normal;
                editor.open_file(path.to_string_lossy().to_string())?;
                editor.needs_full_redraw = true;
            }
        }
        _ => {}
    }
    Ok(())
}
