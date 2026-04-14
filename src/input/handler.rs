use crate::editor::{Editor, EditorMode};
use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
use crossterm::terminal;
use std::io;

pub fn process_keypress(editor: &mut Editor) -> io::Result<()> {
    let event = event::read()?;

    match event {
        Event::Key(key_event) => {
            if key_event.kind == KeyEventKind::Press {
                // Track offset before processing to detect scroll changes
                let old_offset = editor.active_pane().offset_y;

                match editor.mode {
                    EditorMode::Normal => super::normal_mode::process(editor, key_event)?,
                    EditorMode::Search => super::search_mode::process(editor, key_event)?,
                    EditorMode::SavePrompt => super::save_prompt::process(editor, key_event)?,
                    EditorMode::OpenPrompt => super::open_prompt::process(editor, key_event)?,
                    EditorMode::GotoLinePrompt => super::goto_line::process(editor, key_event)?,
                    EditorMode::CommandMode => super::command_mode::process(editor, key_event)?,
                    EditorMode::PaletteMode => super::palette_mode::process(editor, key_event)?,
                    EditorMode::FileBrowser => super::file_browser_mode::process(editor, key_event)?,
                }

                // If scroll offset changed, force full redraw
                if editor.active_pane().offset_y != old_offset {
                    editor.needs_full_redraw = true;
                }
            }
        }
        Event::Mouse(mouse_event) => {
            match mouse_event.kind {
                MouseEventKind::ScrollUp => {
                    let (_, height) = terminal::size()?;
                    let visible_lines = editor.calculate_visible_lines(height);
                    if visible_lines == 0 {
                        return Ok(());
                    }
                    let pane = editor.active_pane_mut();

                    // Scroll up by 3 lines
                    if pane.offset_y > 0 {
                        pane.offset_y = pane.offset_y.saturating_sub(3);
                        // Keep cursor in view
                        if pane.cursor.y > pane.offset_y + visible_lines - 1 {
                            pane.cursor.y = pane.offset_y + visible_lines - 1;
                        }
                        editor.needs_full_redraw = true;
                    }
                }
                MouseEventKind::ScrollDown => {
                    let (_, height) = terminal::size()?;
                    let visible_lines = editor.calculate_visible_lines(height);
                    if visible_lines == 0 {
                        return Ok(());
                    }
                    let pane = editor.active_pane_mut();
                    let max_offset = pane.buffer.line_count().saturating_sub(visible_lines);

                    // Scroll down by 3 lines
                    if pane.offset_y < max_offset {
                        pane.offset_y = (pane.offset_y + 3).min(max_offset);
                        // Keep cursor in view
                        if pane.cursor.y < pane.offset_y {
                            pane.cursor.y = pane.offset_y;
                        }
                        editor.needs_full_redraw = true;
                    }
                }
                _ => {}
            }
        }
        Event::Resize(_, _) => {
            editor.needs_full_redraw = true;
        }
        _ => {}
    }

    Ok(())
}
