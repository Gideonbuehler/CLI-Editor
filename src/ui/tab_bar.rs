use crate::editor::Editor;
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io;

pub fn draw_tab_bar(editor: &Editor, stdout: &mut io::Stdout, width: u16) -> io::Result<()> {
    draw_tab_bar_at(editor, stdout, 0, width)
}

pub fn draw_tab_bar_at(editor: &Editor, stdout: &mut io::Stdout, start_x: u16, width: u16) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(start_x, 0))?;

    // Background for tab bar
    queue!(stdout, SetBackgroundColor(Color::DarkGrey))?;

    let mut x_pos: usize = 0;

    for (idx, tab) in editor.tabs.iter().enumerate() {
        let is_active = idx == editor.active_tab;

        // Get tab name from primary pane
        let name = tab.display_name();

        // Add modified indicator if any pane in tab is modified
        let modified = if tab.is_modified() { "*" } else { "" };

        // Add split indicator if this tab is split
        let split_indicator = if tab.is_split() { "⧫" } else { "" };

        // Format: " name* " or " name*⧫ " for split tabs
        let tab_text = format!(" {}{}{} ", name, modified, split_indicator);
        let tab_width = tab_text.len();

        // Check if tab fits
        if x_pos + tab_width > width as usize {
            // Draw overflow indicator
            queue!(
                stdout,
                SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::White),
                Print("..."),
                ResetColor
            )?;
            break;
        }

        // Set colors based on state
        if is_active {
            // Currently active tab
            queue!(
                stdout,
                SetBackgroundColor(Color::Blue),
                SetForegroundColor(Color::White)
            )?;
        } else {
            // Inactive tab
            queue!(
                stdout,
                SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::Grey)
            )?;
        }

        queue!(stdout, Print(&tab_text))?;

        // Separator between tabs
        queue!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::Grey),
            Print("│")
        )?;

        x_pos += tab_width + 1;
    }

    // Fill rest of tab bar
    let remaining = (width as usize).saturating_sub(x_pos);
    if remaining > 0 {
        queue!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            Print(" ".repeat(remaining))
        )?;
    }

    queue!(stdout, ResetColor)?;

    Ok(())
}
