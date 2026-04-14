use crate::command::EditCommand;
use crate::editor::{Editor, EditorMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use std::io;

pub fn process(editor: &mut Editor, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        KeyEvent {
            code: KeyCode::Tab,
            ..
        } => {
            let pane = editor.active_pane_mut();
            pane.clear_selection();
            for _ in 0..4 {
                let command = EditCommand::InsertChar {
                    row: pane.cursor.y,
                    col: pane.cursor.x,
                    ch: ' ',
                };
                pane.execute_command(command);
                pane.cursor.x += 1;
            }
            editor.message = None;
        }
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            if editor.active_pane().modified && !editor.quit_warning_shown {
                editor.message = Some("File modified! Press Ctrl-Q again to quit".to_string());
                editor.quit_warning_shown = true;
            } else {
                editor.should_quit = true;
            }
        }
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.save_file()?;
        }
        KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.mode = EditorMode::OpenPrompt;
            editor.input_buffer.clear();
            editor.message = Some("Open file: ".to_string());
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            super::search_mode::start_search(editor);
        }
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.mode = EditorMode::GotoLinePrompt;
            editor.input_buffer.clear();
            editor.message = Some("Go to line: ".to_string());
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            super::search_mode::find_next(editor);
        }
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.toggle_line_numbers();
        }
        // File browser toggle
        KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.file_browser.toggle();
            if editor.file_browser.visible {
                editor.mode = EditorMode::FileBrowser;
            }
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.next_pane();
        }
        // Ctrl+H is backspace on many Linux terminals
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // Treat as backspace (same as KeyCode::Backspace handler)
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if editor.active_pane().has_selection() {
                editor.active_pane_mut().delete_selection();
                editor.needs_full_redraw = true;
                editor.message = None;
                return Ok(());
            }
            let pane = editor.active_pane_mut();
            if pane.cursor.x > 0 {
                if let Some(ch) = pane.buffer.get_line(pane.cursor.y).and_then(|line| {
                    if pane.cursor.x > 0 {
                        line.chars().nth(pane.cursor.x - 1)
                    } else {
                        None
                    }
                }) {
                    let command = EditCommand::DeleteChar {
                        row: pane.cursor.y,
                        col: pane.cursor.x - 1,
                        ch,
                    };
                    pane.execute_command(command);
                    pane.cursor.x -= 1;
                }
            } else if pane.cursor.y > 0 {
                let prev_line_len = pane
                    .buffer
                    .get_line(pane.cursor.y - 1)
                    .map(|l| l.len())
                    .unwrap_or(0);
                if let Some(deleted_line) = pane.buffer.get_line(pane.cursor.y).map(|l| l.clone()) {
                    let command = EditCommand::DeleteNewline {
                        row: pane.cursor.y,
                        deleted_line,
                    };
                    pane.execute_command(command);
                    pane.cursor.y -= 1;
                    pane.cursor.x = prev_line_len;
                    pane.adjust_scroll(visible_lines);
                    editor.needs_full_redraw = true;
                }
            }
            editor.message = None;
        }
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.split_vertical();
        }
        // Tab navigation keybindings
        KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.new_tab();
        }
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } if !key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            editor.next_tab();
        }
        KeyEvent {
            code: KeyCode::Char('e') | KeyCode::Char('E'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::SHIFT) => {
            editor.prev_tab();
        }
        // Alt+1 through Alt+9 for quick tab switching
        KeyEvent {
            code: KeyCode::Char(c @ '1'..='9'),
            modifiers: KeyModifiers::ALT,
            ..
        } => {
            let tab_num = c.to_digit(10).unwrap() as usize;
            if tab_num <= editor.tab_count() {
                editor.goto_tab(tab_num - 1);
            }
        }
        KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.active_pane_mut().clear_selection();
            editor.active_pane_mut().redo();
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.active_pane_mut().clear_selection();
            editor.active_pane_mut().undo();
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            let pane = editor.active_pane_mut();
            pane.selection_start = Some((0, 0));
            let last_row = pane.buffer.line_count() - 1;
            let last_col = pane.buffer.get_line(last_row).map(|l| l.len()).unwrap_or(0);
            pane.cursor.y = last_row;
            pane.cursor.x = last_col;
            editor.needs_full_redraw = true;
            editor.message = Some("Selected all".to_string());
        }
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            if editor.active_pane().has_selection() {
                let selected = editor.active_pane().get_selected_text();
                if let Some(text) = selected {
                    if let Some(clipboard) = &mut editor.clipboard {
                        let _ = clipboard.set_text(text);
                        editor.message = Some("Selection copied".to_string());
                    }
                }
                editor.active_pane_mut().clear_selection();
                editor.needs_full_redraw = true;
            } else {
                let line_content = {
                    let pane = editor.active_pane();
                    pane.buffer.get_line(pane.cursor.y).cloned()
                };
                if let Some(line) = line_content {
                    if let Some(clipboard) = &mut editor.clipboard {
                        let _ = clipboard.set_text(line);
                        editor.message = Some("Line copied to clipboard".to_string());
                    }
                }
            }
        }
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            editor.close_split();
        }
        KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            if editor.active_pane().has_selection() {
                editor.active_pane_mut().delete_selection();
                editor.needs_full_redraw = true;
            }
            if let Some(clipboard) = &mut editor.clipboard {
                if let Ok(text) = clipboard.get_text() {
                    let pane = editor.active_pane_mut();
                    for ch in text.chars() {
                        if ch == '\n' {
                            let command = EditCommand::InsertNewline {
                                row: pane.cursor.y,
                                col: pane.cursor.x,
                            };
                            pane.execute_command(command);
                            pane.cursor.y += 1;
                            pane.cursor.x = 0;
                        } else if ch != '\r' {
                            let command = EditCommand::InsertChar {
                                row: pane.cursor.y,
                                col: pane.cursor.x,
                                ch,
                            };
                            pane.execute_command(command);
                            pane.cursor.x += 1;
                        }
                    }
                    editor.needs_full_redraw = true;
                }
            }
        }
        KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // Open command palette
            editor.mode = EditorMode::PaletteMode;
            editor.input_buffer.clear();
            editor.palette_selected = 0;
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char(':'),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            ..
        } => {
            // Enter command mode
            editor.mode = EditorMode::CommandMode;
            editor.input_buffer.clear();
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            ..
        } => {
            if editor.active_pane().has_selection() {
                editor.active_pane_mut().delete_selection();
                editor.needs_full_redraw = true;
            }
            let pane = editor.active_pane_mut();
            let command = EditCommand::InsertChar {
                row: pane.cursor.y,
                col: pane.cursor.x,
                ch: c,
            };
            pane.execute_command(command);
            pane.cursor.x += 1;
            editor.message = None;
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            if editor.active_pane().has_selection() {
                editor.active_pane_mut().delete_selection();
                editor.needs_full_redraw = true;
            }
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            let pane = editor.active_pane_mut();

            let current_row = pane.cursor.y;
            let current_line = pane.buffer.get_line(current_row).cloned().unwrap_or_default();
            let indent: String = current_line.chars().take_while(|c| c.is_whitespace()).collect();
            let should_indent = current_line.trim_end().ends_with('{');

            let command = EditCommand::InsertNewline {
                row: pane.cursor.y,
                col: pane.cursor.x,
            };
            pane.execute_command(command);
            pane.cursor.y += 1;
            pane.cursor.x = 0;

            for c in indent.chars() {
                let command = EditCommand::InsertChar {
                    row: pane.cursor.y,
                    col: pane.cursor.x,
                    ch: c,
                };
                pane.execute_command(command);
                pane.cursor.x += 1;
            }

            if should_indent {
                for _ in 0..4 {
                    let command = EditCommand::InsertChar {
                        row: pane.cursor.y,
                        col: pane.cursor.x,
                        ch: ' ',
                    };
                    pane.execute_command(command);
                    pane.cursor.x += 1;
                }
            }

            pane.adjust_scroll(visible_lines);
            editor.message = None;
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if editor.active_pane().has_selection() {
                editor.active_pane_mut().delete_selection();
                editor.needs_full_redraw = true;
                editor.message = None;
                return Ok(());
            }
            let pane = editor.active_pane_mut();
            if pane.cursor.x > 0 {
                if let Some(ch) = pane.buffer.get_line(pane.cursor.y).and_then(|line| {
                    if pane.cursor.x > 0 {
                        line.chars().nth(pane.cursor.x - 1)
                    } else {
                        None
                    }
                }) {
                    let command = EditCommand::DeleteChar {
                        row: pane.cursor.y,
                        col: pane.cursor.x - 1,
                        ch,
                    };
                    pane.execute_command(command);
                    pane.cursor.x -= 1;
                }
            } else if pane.cursor.y > 0 {
                let prev_line_len = pane
                    .buffer
                    .get_line(pane.cursor.y - 1)
                    .map(|l| l.len())
                    .unwrap_or(0);
                if let Some(deleted_line) = pane.buffer.get_line(pane.cursor.y).map(|l| l.clone()) {
                    let command = EditCommand::DeleteNewline {
                        row: pane.cursor.y,
                        deleted_line,
                    };
                    pane.execute_command(command);
                    pane.cursor.y -= 1;
                    pane.cursor.x = prev_line_len;
                    pane.adjust_scroll(visible_lines);
                    editor.needs_full_redraw = true;
                }
            }
            editor.message = None;
        }
        KeyEvent {
            code: KeyCode::Left,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
                editor.needs_full_redraw = true;
            }
            let old_y = editor.active_pane().cursor.y;
            {
                let pane = editor.active_pane_mut();
                if pane.cursor.x > 0 {
                    pane.cursor.x -= 1;
                } else if pane.cursor.y > 0 {
                    pane.cursor.y -= 1;
                    pane.cursor.x = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    pane.adjust_scroll(visible_lines);
                }
            }
            if editor.active_pane().cursor.y != old_y {
                editor.prev_cursor_y = Some(old_y);
            }
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::Right,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
                editor.needs_full_redraw = true;
            }
            let old_y = editor.active_pane().cursor.y;
            {
                let pane = editor.active_pane_mut();
                if let Some(line) = pane.buffer.get_line(pane.cursor.y) {
                    if pane.cursor.x < line.len() {
                        pane.cursor.x += 1;
                    } else if pane.cursor.y < pane.buffer.line_count() - 1 {
                        pane.cursor.y += 1;
                        pane.cursor.x = 0;
                        pane.adjust_scroll(visible_lines);
                    }
                }
            }
            if editor.active_pane().cursor.y != old_y {
                editor.prev_cursor_y = Some(old_y);
            }
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::Up,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
            }
            let old_y = editor.active_pane().cursor.y;
            {
                let pane = editor.active_pane_mut();
                if pane.cursor.y > 0 {
                    pane.cursor.y -= 1;
                    let line_len = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    if pane.cursor.x > line_len {
                        pane.cursor.x = line_len;
                    }
                    pane.adjust_scroll(visible_lines);
                }
            }
            editor.prev_cursor_y = Some(old_y);
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::Down,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
            }
            let old_y = editor.active_pane().cursor.y;
            {
                let pane = editor.active_pane_mut();
                if pane.cursor.y < pane.buffer.line_count() - 1 {
                    pane.cursor.y += 1;
                    let line_len = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    if pane.cursor.x > line_len {
                        pane.cursor.x = line_len;
                    }
                    pane.adjust_scroll(visible_lines);
                }
            }
            editor.prev_cursor_y = Some(old_y);
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::Home,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
                editor.needs_full_redraw = true;
            }
            editor.active_pane_mut().cursor.x = 0;
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::End,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
                editor.needs_full_redraw = true;
            }
            {
                let pane = editor.active_pane_mut();
                if let Some(line) = pane.buffer.get_line(pane.cursor.y) {
                    pane.cursor.x = line.len();
                }
            }
            if shifting { editor.needs_full_redraw = true; }
        }
        KeyEvent {
            code: KeyCode::PageUp,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
            }
            {
                let pane = editor.active_pane_mut();
                pane.cursor.y = pane.cursor.y.saturating_sub(visible_lines);
                pane.adjust_scroll(visible_lines);
            }
            editor.needs_full_redraw = true;
        }
        KeyEvent {
            code: KeyCode::PageDown,
            modifiers,
            ..
        } => {
            let shifting = modifiers.contains(KeyModifiers::SHIFT);
            let (_, height) = terminal::size()?;
            let visible_lines = editor.calculate_visible_lines(height);
            if shifting && editor.active_pane().selection_start.is_none() {
                let (y, x) = (editor.active_pane().cursor.y, editor.active_pane().cursor.x);
                editor.active_pane_mut().selection_start = Some((y, x));
            } else if !shifting && editor.active_pane().has_selection() {
                editor.active_pane_mut().clear_selection();
            }
            {
                let pane = editor.active_pane_mut();
                pane.cursor.y = (pane.cursor.y + visible_lines).min(pane.buffer.line_count() - 1);
                pane.adjust_scroll(visible_lines);
            }
            editor.needs_full_redraw = true;
        }
        _ => {}
    }
    Ok(())
}
