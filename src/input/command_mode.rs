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
            editor.tab_completion_index = 0;
            editor.tab_completion_base.clear();
        }
        return Ok(());
    }

    match key_event.code {
        KeyCode::Enter => {
            execute_command(editor);
            // Only reset to Normal if the command didn't change the mode
            if matches!(editor.mode, EditorMode::CommandMode) {
                editor.mode = EditorMode::Normal;
            }
            editor.needs_full_redraw = true;
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
                // Reset tab completion when editing
                editor.tab_completion_index = 0;
                editor.tab_completion_base.clear();
            }
        }
        KeyCode::Tab => {
            // Autocomplete
            autocomplete(editor);
        }
        KeyCode::Char(c) => {
            editor.input_buffer.push(c);
            // Reset tab completion when typing
            editor.tab_completion_index = 0;
            editor.tab_completion_base.clear();
        }
        _ => {}
    }
    Ok(())
}

fn execute_command(editor: &mut Editor) {
    let input = editor.input_buffer.trim().to_string();
    if input.is_empty() {
        return;
    }

    // Check if it's a line number (e.g., :123)
    if let Ok(line_num) = input.parse::<usize>() {
        let target = line_num.saturating_sub(1);
        let pane = editor.active_pane_mut();
        if target < pane.buffer.line_count() {
            pane.cursor.y = target;
            pane.cursor.x = 0;
            editor.message = Some(format!("Line {}", line_num));
            editor.needs_full_redraw = true;
        } else {
            editor.message = Some("Line number out of range".to_string());
        }
        return;
    }

    // Parse command and arguments
    // Handle special cases like :s/foo/bar where command and args are joined by /
    let (cmd_name, args) = if input.starts_with("s/") || input.starts_with("sub/") || input.starts_with("replace/") {
        // Replace command with / separator
        if let Some(idx) = input.find('/') {
            (&input[..idx], &input[idx + 1..])
        } else {
            (input.as_str(), "")
        }
    } else if input.starts_with('/') {
        // Search command: /pattern
        ("find", &input[1..])
    } else {
        // Normal space-separated command
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = if parts.len() > 1 { parts[1] } else { "" };
        (cmd, arg)
    };

    let registry = CommandRegistry::new();

    if let Some(command) = registry.find_by_alias(cmd_name) {
        match (command.action)(editor, args) {
            Ok(()) => {
                if editor.message.is_none() {
                    editor.message = Some(format!(":{}", input));
                }
            }
            Err(e) => {
                editor.message = Some(format!("Error: {}", e));
            }
        }
    } else {
        editor.message = Some(format!("Unknown command: {}", cmd_name));
    }
}

fn autocomplete(editor: &mut Editor) {
    let registry = CommandRegistry::new();

    // Use the base query for filtering (what user originally typed)
    let query = if editor.tab_completion_base.is_empty() {
        // First tab press - save the original input as base
        editor.tab_completion_base = editor.input_buffer.clone();
        &editor.input_buffer
    } else {
        // Subsequent tab press - use saved base
        &editor.tab_completion_base
    };

    let filtered = registry.filter(query);

    if filtered.is_empty() {
        return;
    }

    if filtered.len() == 1 {
        // Only one match - complete it
        editor.input_buffer = filtered[0].name.to_string();
        editor.tab_completion_index = 0;
        editor.tab_completion_base.clear();
        editor.needs_full_redraw = true;
    } else {
        // Multiple matches - cycle through them
        let index = editor.tab_completion_index % filtered.len();
        editor.input_buffer = filtered[index].name.to_string();
        editor.tab_completion_index = (editor.tab_completion_index + 1) % filtered.len();
        editor.needs_full_redraw = true;
    }
}
