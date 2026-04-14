use crate::command::registry::CommandRegistry;
use crate::editor::{Editor, EditorMode};
use crossterm::{
    cursor, queue,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{self, ClearType},
};
use std::io;

pub fn draw_message_line(editor: &Editor, stdout: &mut io::Stdout, height: u16) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(0, height - 1))?;

    match &editor.mode {
        EditorMode::Normal => {
            if let Some(msg) = &editor.message {
                queue!(stdout, Print(msg))?;
            } else {
                queue!(
                    stdout,
                    Print("^Q:Quit ^S:Save ^O:Open ^F:Search ^N:Next ^Z:Undo ^Y:Redo ^A:SelAll ^P:Cmds  :cmd")
                )?;
            }
        }
        EditorMode::Search | EditorMode::SavePrompt | EditorMode::OpenPrompt | EditorMode::GotoLinePrompt => {
            if let Some(msg) = &editor.message {
                queue!(stdout, Print(format!("{}{}", msg, editor.input_buffer)))?;
            }
        }
        EditorMode::CommandMode => {
            queue!(stdout, Print(format!(":{}", editor.input_buffer)))?;

            // Show inline command hints
            if !editor.input_buffer.is_empty() {
                let registry = CommandRegistry::new();
                let filtered = registry.filter(&editor.input_buffer);

                if !filtered.is_empty() {
                    // Build hint string from matching commands (show up to 5)
                    let hints: Vec<&str> = filtered.iter()
                        .take(5)
                        .map(|cmd| cmd.name)
                        .collect();

                    let hint_text = format!("  → {}", hints.join(" | "));
                    queue!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        Print(hint_text),
                        ResetColor
                    )?;
                }
            }
        }
        EditorMode::PaletteMode => {
            // Palette mode draws its own UI overlay
        }
        EditorMode::FileBrowser => {
            queue!(
                stdout,
                Print("↑↓:Navigate  Enter:Open  ←→:Collapse/Expand  Backspace:Parent  Tab/Esc:Editor  ^B:Close")
            )?;
        }
    }
    queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;

    Ok(())
}
