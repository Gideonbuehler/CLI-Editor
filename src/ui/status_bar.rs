use crate::editor::{Editor, SplitMode};
use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io;

pub fn draw_status_bar(editor: &Editor, stdout: &mut io::Stdout, height: u16) -> io::Result<()> {
    let pane = editor.active_pane();
    let filename = pane
        .current_file
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("[No Name]");

    let modified_indicator = if pane.modified { " [+]" } else { "" };

    let split_indicator = match editor.split_mode() {
        SplitMode::None => String::new(),
        SplitMode::Horizontal => format!(" [H-Split:{}]", editor.split_focus() + 1),
        SplitMode::Vertical => format!(" [V-Split:{}]", editor.split_focus() + 1),
    };

    // Show tab info only if more than one tab
    let tab_info = if editor.tab_count() > 1 {
        format!(" Tab {}/{}", editor.focused_tab_index() + 1, editor.tab_count())
    } else {
        String::new()
    };

    queue!(
        stdout,
        cursor::MoveTo(0, height - 2),
        SetBackgroundColor(Color::DarkGrey),
        SetForegroundColor(Color::White),
        Print(format!(
            " {}{} | Line {}/{} Col {}{}{}",
            filename,
            modified_indicator,
            pane.cursor.y + 1,
            pane.buffer.line_count(),
            pane.cursor.x + 1,
            tab_info,
            split_indicator
        )),
        terminal::Clear(ClearType::UntilNewLine),
        ResetColor
    )?;

    Ok(())
}
