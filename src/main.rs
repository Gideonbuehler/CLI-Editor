mod buffer;
mod command;
mod config;
mod editor;
mod file_browser;
mod input;
mod pane;
mod syntax;
mod ui;

use editor::Editor;
use std::io;

fn main() -> io::Result<()> {
    let mut editor = Editor::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if let Err(e) = editor.open_file(args[1].clone()) {
            eprintln!("Error opening file: {}", e);
        }
    }

    editor.run()
}
