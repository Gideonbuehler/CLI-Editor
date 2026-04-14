use crate::editor::{Editor, EditorMode};
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io;

pub fn draw_file_browser(
    editor: &Editor,
    stdout: &mut io::Stdout,
    start_y: u16,
    height: u16,
) -> io::Result<()> {
    let browser = &editor.file_browser;
    let width = browser.width;
    let is_focused = matches!(editor.mode, EditorMode::FileBrowser);

    // Draw header
    queue!(stdout, cursor::MoveTo(0, start_y))?;
    if is_focused {
        queue!(
            stdout,
            SetBackgroundColor(Color::Blue),
            SetForegroundColor(Color::White)
        )?;
    } else {
        queue!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White)
        )?;
    }

    let header = " FILES";
    let header_pad = (width as usize).saturating_sub(header.len());
    queue!(stdout, Print(header), Print(" ".repeat(header_pad)))?;
    queue!(stdout, ResetColor)?;

    // Draw current directory (truncated if needed)
    queue!(stdout, cursor::MoveTo(0, start_y + 1))?;
    queue!(
        stdout,
        SetBackgroundColor(Color::Black),
        SetForegroundColor(Color::DarkGrey)
    )?;
    let root_str = browser.root.to_string_lossy();
    let root_display = if root_str.len() > (width as usize - 2) {
        format!("..{}", &root_str[root_str.len() - (width as usize - 4)..])
    } else {
        root_str.to_string()
    };
    let root_pad = (width as usize).saturating_sub(root_display.len() + 1);
    queue!(stdout, Print(" "), Print(&root_display), Print(" ".repeat(root_pad)))?;
    queue!(stdout, ResetColor)?;

    // Calculate visible area for entries
    let entries_start_y = start_y + 2;
    let entries_height = height.saturating_sub(2) as usize;

    // Draw entries
    for row in 0..entries_height {
        let entry_idx = browser.scroll_offset + row;
        let screen_y = entries_start_y + row as u16;

        queue!(stdout, cursor::MoveTo(0, screen_y))?;

        if entry_idx < browser.entries.len() {
            let entry = &browser.entries[entry_idx];
            let is_selected = entry_idx == browser.cursor;

            // Set colors based on selection and focus
            if is_selected && is_focused {
                queue!(
                    stdout,
                    SetBackgroundColor(Color::Blue),
                    SetForegroundColor(Color::White)
                )?;
            } else if is_selected {
                queue!(
                    stdout,
                    SetBackgroundColor(Color::DarkGrey),
                    SetForegroundColor(Color::White)
                )?;
            } else {
                queue!(stdout, SetBackgroundColor(Color::Black))?;
                if entry.is_dir {
                    queue!(stdout, SetForegroundColor(Color::Cyan))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::White))?;
                }
            }

            // Build the display string with indentation
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if browser.expanded.contains(&entry.path) {
                    "v "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            let name = &entry.name;
            let full_text = format!("{}{}{}", indent, icon, name);

            // Truncate if too long
            let display_text = if full_text.len() > width as usize {
                format!("{}...", &full_text[..width as usize - 3])
            } else {
                full_text
            };

            let padding = (width as usize).saturating_sub(display_text.len());
            queue!(stdout, Print(&display_text), Print(" ".repeat(padding)))?;
            queue!(stdout, ResetColor)?;
        } else {
            // Empty row
            queue!(
                stdout,
                SetBackgroundColor(Color::Black),
                Print(" ".repeat(width as usize)),
                ResetColor
            )?;
        }
    }

    // Draw vertical border on the right
    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
    for row in 0..height {
        queue!(
            stdout,
            cursor::MoveTo(width, start_y + row),
            Print("│")
        )?;
    }
    queue!(stdout, ResetColor)?;

    Ok(())
}

/// Get the width of the file browser (or 0 if hidden)
pub fn get_browser_width(editor: &Editor) -> u16 {
    if editor.file_browser.visible {
        editor.file_browser.width + 1 // +1 for border
    } else {
        0
    }
}
