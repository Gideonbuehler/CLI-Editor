mod file_browser;
mod message_line;
pub mod palette;
mod render;
mod status_bar;
mod tab_bar;

use crate::editor::{Editor, EditorMode, SplitMode};
use crossterm::{cursor, queue, style::Print, terminal};
use std::io::{self, Write};

pub use render::{draw_line_at, draw_line_with_highlight, draw_line_with_syntax};

pub fn refresh_screen(editor: &mut Editor, stdout: &mut io::Stdout) -> io::Result<()> {
    let (width, height) = terminal::size()?;

    queue!(stdout, cursor::Hide)?;

    if editor.active_pane().has_selection() {
        editor.needs_full_redraw = true;
    }

    // Calculate file browser width
    let browser_width = file_browser::get_browser_width(editor);
    let editor_start_x = browser_width;
    let editor_width = width.saturating_sub(browser_width);

    // Calculate tab bar offset
    let tab_bar_height: u16 = if editor.show_tab_bar() { 1 } else { 0 };
    let content_start_y = tab_bar_height;
    let content_height = height.saturating_sub(2 + tab_bar_height); // -2 for status bar and message line

    // Adjust scroll for file browser if visible
    if editor.file_browser.visible {
        let fb_height = content_height as usize;
        editor.file_browser.adjust_scroll_for_height(fb_height.saturating_sub(2)); // -2 for header
    }

    if editor.needs_full_redraw {
        // Draw file browser if visible
        if editor.file_browser.visible {
            file_browser::draw_file_browser(editor, stdout, content_start_y, content_height)?;
        }

        // Draw tab bar if multiple tabs (offset by browser width)
        if editor.show_tab_bar() {
            tab_bar::draw_tab_bar_at(editor, stdout, editor_start_x, editor_width)?;
        }

        let tab = editor.current_tab();
        match tab.split_mode {
            SplitMode::None => {
                // Draw primary pane only
                render::draw_pane_content(editor, stdout, editor_start_x, content_start_y, editor_width, content_height, &tab.primary, true)?;
            }
            SplitMode::Horizontal => {
                let split_height = content_height.saturating_sub(1) / 2;
                let tab = editor.current_tab();
                let is_primary_active = tab.split_focus == 0;

                // Top pane (primary)
                render::draw_pane_content(editor, stdout, editor_start_x, content_start_y, editor_width, split_height, &tab.primary, is_primary_active)?;

                // Divider
                queue!(stdout, cursor::MoveTo(editor_start_x, content_start_y + split_height))?;
                for _ in 0..editor_width {
                    queue!(stdout, Print("─"))?;
                }

                // Bottom pane (secondary)
                if let Some(ref secondary) = tab.secondary {
                    render::draw_pane_content(editor, stdout, editor_start_x, content_start_y + split_height + 1, editor_width, split_height, secondary, !is_primary_active)?;
                }
            }
            SplitMode::Vertical => {
                let split_width = editor_width / 2;
                let tab = editor.current_tab();
                let is_primary_active = tab.split_focus == 0;

                // Left pane (primary)
                render::draw_pane_content(editor, stdout, editor_start_x, content_start_y, split_width, content_height, &tab.primary, is_primary_active)?;

                // Divider
                for row in 0..content_height {
                    queue!(stdout, cursor::MoveTo(editor_start_x + split_width, content_start_y + row), Print("│"))?;
                }

                // Right pane (secondary)
                if let Some(ref secondary) = tab.secondary {
                    render::draw_pane_content(editor, stdout, editor_start_x + split_width + 1, content_start_y, split_width - 1, content_height, secondary, !is_primary_active)?;
                }
            }
        }

        editor.needs_full_redraw = false;
    } else {
        // Partial redraw - only update changed lines
        let prev_y = editor.prev_cursor_y.take();
        let cur_y = editor.active_pane().cursor.y;
        let split_mode = editor.split_mode();
        let split_focus = editor.split_focus();

        match split_mode {
            SplitMode::None => {
                if let Some(py) = prev_y {
                    if py != cur_y {
                        draw_line_at(editor, stdout, editor_start_x, content_start_y, editor_width, py)?;
                    }
                }
                draw_line_at(editor, stdout, editor_start_x, content_start_y, editor_width, cur_y)?;
            }
            SplitMode::Horizontal => {
                let split_height = content_height.saturating_sub(1) / 2;
                let sy = if split_focus == 0 { content_start_y } else { content_start_y + split_height + 1 };
                if let Some(py) = prev_y {
                    if py != cur_y {
                        draw_line_at(editor, stdout, editor_start_x, sy, editor_width, py)?;
                    }
                }
                draw_line_at(editor, stdout, editor_start_x, sy, editor_width, cur_y)?;
            }
            SplitMode::Vertical => {
                let split_width = editor_width / 2;
                let (sx, w) = if split_focus == 0 { (editor_start_x, split_width) } else { (editor_start_x + split_width + 1, split_width - 1) };
                if let Some(py) = prev_y {
                    if py != cur_y {
                        draw_line_at(editor, stdout, sx, content_start_y, w, py)?;
                    }
                }
                draw_line_at(editor, stdout, sx, content_start_y, w, cur_y)?;
            }
        }
    }

    status_bar::draw_status_bar(editor, stdout, height)?;
    message_line::draw_message_line(editor, stdout, height)?;

    // Draw palette overlay if in palette mode
    if matches!(editor.mode, EditorMode::PaletteMode) {
        let registry = crate::command::registry::CommandRegistry::new();
        palette::draw_palette(editor, stdout, &registry, width, height)?;
    }

    position_cursor(editor, stdout, width, height)?;

    stdout.flush()?;
    Ok(())
}

pub fn position_cursor(editor: &Editor, stdout: &mut io::Stdout, width: u16, height: u16) -> io::Result<()> {
    let tab_bar_height: u16 = if editor.show_tab_bar() { 1 } else { 0 };
    let content_height = height.saturating_sub(2 + tab_bar_height);
    let split_mode = editor.split_mode();
    let split_focus = editor.split_focus();
    let browser_width = file_browser::get_browser_width(editor);
    let editor_width = width.saturating_sub(browser_width);

    match editor.mode {
        EditorMode::Normal => {
            let pane = editor.active_pane();
            let line_num_width = editor.get_line_number_width();
            let screen_y = pane.cursor.y.saturating_sub(pane.offset_y);

            let (cursor_x, cursor_y) = match split_mode {
                SplitMode::None => {
                    (browser_width as usize + line_num_width + pane.cursor.x, tab_bar_height as usize + screen_y)
                }
                SplitMode::Horizontal => {
                    let split_height = content_height.saturating_sub(1) / 2;
                    if split_focus == 0 {
                        (browser_width as usize + line_num_width + pane.cursor.x, tab_bar_height as usize + screen_y)
                    } else {
                        (browser_width as usize + line_num_width + pane.cursor.x, tab_bar_height as usize + split_height as usize + 1 + screen_y)
                    }
                }
                SplitMode::Vertical => {
                    let split_width = editor_width / 2;
                    if split_focus == 0 {
                        (browser_width as usize + line_num_width + pane.cursor.x, tab_bar_height as usize + screen_y)
                    } else {
                        (browser_width as usize + split_width as usize + 1 + line_num_width + pane.cursor.x, tab_bar_height as usize + screen_y)
                    }
                }
            };

            queue!(
                stdout,
                cursor::MoveTo(cursor_x as u16, cursor_y as u16),
                cursor::Show
            )?;
        }
        EditorMode::FileBrowser => {
            // Hide cursor in file browser mode (selection is shown via highlighting)
            queue!(stdout, cursor::Hide)?;
        }
        EditorMode::Search | EditorMode::SavePrompt | EditorMode::OpenPrompt | EditorMode::GotoLinePrompt => {
            let prompt_len = editor.message.as_ref().map(|m| m.len()).unwrap_or(0);
            queue!(
                stdout,
                cursor::MoveTo((prompt_len + editor.input_buffer.len()) as u16, height - 1),
                cursor::Show
            )?;
        }
        EditorMode::CommandMode => {
            // Position cursor after ":"
            queue!(
                stdout,
                cursor::MoveTo((1 + editor.input_buffer.len()) as u16, height - 1),
                cursor::Show
            )?;
        }
        EditorMode::PaletteMode => {
            // Position cursor in palette input field
            let palette_width: u16 = 60.min(width - 4);
            let start_x = (width - palette_width) / 2;
            let start_y = (height - 12.min(height - 4)) / 2;
            queue!(
                stdout,
                cursor::MoveTo(start_x + 3 + editor.input_buffer.len() as u16, start_y + 1),
                cursor::Show
            )?;
        }
    }

    Ok(())
}
