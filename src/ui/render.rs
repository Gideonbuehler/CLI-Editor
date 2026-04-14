use crate::editor::Editor;
use crate::pane::Pane;
use crate::syntax::SyntaxHighlighter;
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io;

/// Draw pane content with a direct Pane reference
pub fn draw_pane_content(
    editor: &Editor,
    stdout: &mut io::Stdout,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
    pane: &Pane,
    is_active: bool,
) -> io::Result<()> {
    let line_num_width = editor.get_line_number_width();
    let text_width = width.saturating_sub(line_num_width as u16);

    // Treat all buffer rows as content rows to avoid scanning the full buffer each redraw.
    let last_content_line = pane.buffer.line_count().saturating_sub(1);

    for screen_row in 0..height as usize {
        let file_row = screen_row + pane.offset_y;
        let screen_y = start_y + screen_row as u16;

        queue!(stdout, cursor::MoveTo(start_x, screen_y))?;

        if editor.show_line_numbers {
            if file_row < pane.buffer.line_count() && file_row <= last_content_line.max(pane.cursor.y) {
                queue!(
                    stdout,
                    SetForegroundColor(if is_active { Color::Yellow } else { Color::DarkGrey }),
                    Print(format!("{:>width$} ", file_row + 1, width = line_num_width - 1)),
                    ResetColor
                )?;
            } else if file_row < pane.buffer.line_count() {
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("{:>width$} ", "~", width = line_num_width - 1)),
                    ResetColor
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("{:>width$} ", "~", width = line_num_width - 1)),
                    ResetColor
                )?;
            }
        }

        if file_row < pane.buffer.line_count() {
            if let Some(line) = pane.buffer.get_line(file_row) {
                let display_line = if line.len() > text_width as usize {
                    &line[..text_width as usize]
                } else {
                    line
                };

                let selection_range = if let Some(start_pos) = pane.selection_start {
                    let end_pos = (pane.cursor.y, pane.cursor.x);
                    let (start, end) = if start_pos < end_pos { (start_pos, end_pos) } else { (end_pos, start_pos) };

                    if file_row > start.0 && file_row < end.0 {
                        Some((0, line.len()))
                    } else if file_row == start.0 && file_row == end.0 {
                        Some((start.1, end.1))
                    } else if file_row == start.0 {
                        Some((start.1, line.len()))
                    } else if file_row == end.0 {
                        Some((0, end.1))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if !pane.search_query.is_empty() && line.contains(&pane.search_query) {
                    draw_line_with_highlight(stdout, display_line, &pane.search_query)?;
                } else {
                    draw_line_with_syntax(stdout, display_line, &pane.highlighter, selection_range)?;
                }
            }
        } else if !editor.show_line_numbers {
            queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
            queue!(stdout, Print("~"))?;
            queue!(stdout, ResetColor)?;
        }

        let mut used_width = line_num_width;
        if file_row < pane.buffer.line_count() {
            if let Some(line) = pane.buffer.get_line(file_row) {
                if line.len() > text_width as usize {
                    used_width += text_width as usize;
                } else {
                    used_width += line.len();
                }
            }
        } else if !editor.show_line_numbers {
            used_width += 1;
        }

        let remaining = width as usize - used_width;
        if remaining > 0 {
            queue!(stdout, Print(" ".repeat(remaining)))?;
        }
    }

    Ok(())
}

pub fn draw_line_at(
    editor: &Editor,
    stdout: &mut io::Stdout,
    start_x: u16,
    start_y: u16,
    width: u16,
    file_row: usize,
) -> io::Result<()> {
    let pane = editor.active_pane();
    let line_num_width = editor.get_line_number_width();
    let text_width = width.saturating_sub(line_num_width as u16);

    if file_row < pane.offset_y {
        return Ok(());
    }
    let screen_y = file_row.saturating_sub(pane.offset_y);
    let actual_y = start_y + screen_y as u16;

    queue!(stdout, cursor::MoveTo(start_x, actual_y))?;

    let is_cursor_line = file_row == pane.cursor.y;

    if editor.show_line_numbers {
        if file_row < pane.buffer.line_count() {
            queue!(
                stdout,
                SetForegroundColor(if is_cursor_line { Color::Yellow } else { Color::DarkGrey }),
                Print(format!("{:>width$} ", file_row + 1, width = line_num_width - 1)),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("{:>width$} ", "~", width = line_num_width - 1)),
                ResetColor
            )?;
        }
    }

    if let Some(line) = pane.buffer.get_line(file_row) {
        let display_line = if line.len() > text_width as usize {
            &line[..text_width as usize]
        } else {
            line
        };

        let selection_range = if let Some(start_pos) = pane.selection_start {
            let end_pos = (pane.cursor.y, pane.cursor.x);
            let (start, end) = if start_pos < end_pos { (start_pos, end_pos) } else { (end_pos, start_pos) };
            if file_row > start.0 && file_row < end.0 {
                Some((0, line.len()))
            } else if file_row == start.0 && file_row == end.0 {
                Some((start.1, end.1))
            } else if file_row == start.0 {
                Some((start.1, line.len()))
            } else if file_row == end.0 {
                Some((0, end.1))
            } else {
                None
            }
        } else {
            None
        };

        if !pane.search_query.is_empty() && line.contains(&pane.search_query) {
            draw_line_with_highlight(stdout, display_line, &pane.search_query)?;
        } else {
            draw_line_with_syntax(stdout, display_line, &pane.highlighter, selection_range)?;
        }
    }

    let current_x = line_num_width + if let Some(line) = pane.buffer.get_line(file_row) {
        if line.len() > text_width as usize {
            text_width as usize
        } else {
            line.len()
        }
    } else {
        0
    };

    let remaining = width.saturating_sub(current_x as u16);
    if remaining > 0 {
        queue!(stdout, Print(" ".repeat(remaining as usize)))?;
    }

    Ok(())
}

pub fn draw_line_with_syntax(
    stdout: &mut io::Stdout,
    line: &str,
    highlighter: &SyntaxHighlighter,
    selection_range: Option<(usize, usize)>,
) -> io::Result<()> {
    let tokens = highlighter.highlight_line(line);

    if let Some((sel_start, sel_end)) = selection_range {
        let mut current_col = 0;
        for (text, token_type) in tokens {
            let color = token_type.color();
            for ch in text.chars() {
                let is_selected = current_col >= sel_start && current_col < sel_end;
                if is_selected {
                    queue!(
                        stdout,
                        SetBackgroundColor(Color::DarkBlue),
                        SetForegroundColor(Color::White),
                        Print(ch),
                        ResetColor
                    )?;
                } else {
                    queue!(stdout, SetForegroundColor(color), Print(ch), ResetColor)?;
                }
                current_col += 1;
            }
        }
    } else {
        for (text, token_type) in tokens {
            queue!(
                stdout,
                SetForegroundColor(token_type.color()),
                Print(text),
                ResetColor
            )?;
        }
    }

    Ok(())
}

pub fn draw_line_with_highlight(
    stdout: &mut io::Stdout,
    line: &str,
    query: &str,
) -> io::Result<()> {
    let mut last_end = 0;
    for (idx, _) in line.match_indices(query) {
        if idx > last_end {
            queue!(stdout, Print(&line[last_end..idx]))?;
        }
        queue!(
            stdout,
            SetBackgroundColor(Color::Yellow),
            SetForegroundColor(Color::Black),
            Print(&line[idx..idx + query.len()]),
            ResetColor
        )?;
        last_end = idx + query.len();
    }
    if last_end < line.len() {
        queue!(stdout, Print(&line[last_end..]))?;
    }
    Ok(())
}
