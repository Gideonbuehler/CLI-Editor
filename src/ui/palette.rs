use crate::command::registry::CommandRegistry;
use crate::editor::Editor;
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io;

pub fn draw_palette(
    editor: &Editor,
    stdout: &mut io::Stdout,
    registry: &CommandRegistry,
    width: u16,
    height: u16,
) -> io::Result<()> {
    let palette_width: u16 = 60.min(width - 4);
    let palette_height: u16 = 12.min(height - 4);
    let start_x = (width - palette_width) / 2;
    let start_y = (height - palette_height) / 2;

    let filtered = registry.filter(&editor.input_buffer);
    let max_visible = (palette_height - 3) as usize;

    // Draw border and background
    for row in 0..palette_height {
        queue!(stdout, cursor::MoveTo(start_x, start_y + row))?;
        queue!(stdout, SetBackgroundColor(Color::DarkGrey))?;

        if row == 0 {
            // Top border with title
            let title = " Command Palette ";
            let padding = (palette_width as usize - title.len()) / 2;
            queue!(
                stdout,
                SetForegroundColor(Color::White),
                Print("─".repeat(padding)),
                Print(title),
                Print("─".repeat(palette_width as usize - padding - title.len())),
            )?;
        } else if row == 1 {
            // Input line
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print(" > "),
                SetForegroundColor(Color::White),
                Print(&editor.input_buffer),
                Print(" ".repeat((palette_width as usize).saturating_sub(3 + editor.input_buffer.len()))),
            )?;
        } else if row == 2 {
            // Separator
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("─".repeat(palette_width as usize)),
            )?;
        } else if row == palette_height - 1 {
            // Bottom border
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("─".repeat(palette_width as usize)),
            )?;
        } else {
            // Command list
            let cmd_idx = (row - 3) as usize;
            if cmd_idx < filtered.len() && cmd_idx < max_visible {
                let cmd = &filtered[cmd_idx];
                let is_selected = cmd_idx == editor.palette_selected;

                if is_selected {
                    queue!(stdout, SetBackgroundColor(Color::Blue))?;
                }

                let prefix = if is_selected { " > " } else { "   " };
                let keybind = cmd.keybinding.unwrap_or("");
                let name_width = palette_width as usize - 6 - keybind.len();
                let name = if cmd.name.len() > name_width {
                    &cmd.name[..name_width]
                } else {
                    cmd.name
                };

                queue!(
                    stdout,
                    SetForegroundColor(if is_selected { Color::White } else { Color::Cyan }),
                    Print(prefix),
                    Print(name),
                    Print(" ".repeat(name_width - name.len())),
                    SetForegroundColor(Color::DarkGrey),
                    Print(keybind),
                    Print(" ".repeat(3)),
                )?;

                if is_selected {
                    queue!(stdout, SetBackgroundColor(Color::DarkGrey))?;
                }
            } else {
                queue!(stdout, Print(" ".repeat(palette_width as usize)))?;
            }
        }
        queue!(stdout, ResetColor)?;
    }

    Ok(())
}
