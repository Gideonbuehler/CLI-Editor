use crate::editor::Editor;

pub struct Command {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub keybinding: Option<&'static str>,
    pub action: fn(&mut Editor, &str) -> Result<(), String>,
}

pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: vec![
                Command {
                    name: "quit",
                    aliases: &["q"],
                    description: "Quit the editor",
                    keybinding: Some("Ctrl+Q"),
                    action: |editor, _| {
                        if editor.active_pane().modified {
                            Err("File modified! Use :q! to force quit".to_string())
                        } else {
                            editor.should_quit = true;
                            Ok(())
                        }
                    },
                },
                Command {
                    name: "force-quit",
                    aliases: &["q!"],
                    description: "Force quit without saving",
                    keybinding: None,
                    action: |editor, _| {
                        editor.should_quit = true;
                        Ok(())
                    },
                },
                Command {
                    name: "save",
                    aliases: &["w", "write"],
                    description: "Save the current file",
                    keybinding: Some("Ctrl+S"),
                    action: |editor, args| {
                        if !args.is_empty() {
                            editor.save_file_as(args.to_string()).map_err(|e| e.to_string())
                        } else {
                            editor.save_file().map_err(|e| e.to_string())
                        }
                    },
                },
                Command {
                    name: "save-quit",
                    aliases: &["wq", "x"],
                    description: "Save and quit",
                    keybinding: None,
                    action: |editor, _| {
                        editor.save_file().map_err(|e| e.to_string())?;
                        editor.should_quit = true;
                        Ok(())
                    },
                },
                Command {
                    name: "open",
                    aliases: &["o", "e", "edit"],
                    description: "Open a file",
                    keybinding: Some("Ctrl+O"),
                    action: |editor, args| {
                        if args.is_empty() {
                            Err("Usage: :open <filename>".to_string())
                        } else {
                            editor.open_file(args.to_string()).map_err(|e| e.to_string())
                        }
                    },
                },
                Command {
                    name: "goto",
                    aliases: &["g"],
                    description: "Go to a line number",
                    keybinding: Some("Ctrl+G"),
                    action: |editor, args| {
                        let line_num: usize = args.parse().map_err(|_| "Invalid line number".to_string())?;
                        let target = line_num.saturating_sub(1);
                        let pane = editor.active_pane_mut();
                        if target < pane.buffer.line_count() {
                            pane.cursor.y = target;
                            pane.cursor.x = 0;
                            editor.needs_full_redraw = true;
                            Ok(())
                        } else {
                            Err("Line number out of range".to_string())
                        }
                    },
                },
                Command {
                    name: "find",
                    aliases: &["/", "search"],
                    description: "Search for text",
                    keybinding: Some("Ctrl+F"),
                    action: |editor, args| {
                        if args.is_empty() {
                            crate::input::search_mode::start_search(editor);
                        } else {
                            editor.input_buffer = args.to_string();
                            crate::input::search_mode::perform_search(editor);
                        }
                        Ok(())
                    },
                },
                Command {
                    name: "find-next",
                    aliases: &["n"],
                    description: "Find next match",
                    keybinding: Some("Ctrl+N"),
                    action: |editor, _| {
                        crate::input::search_mode::find_next(editor);
                        Ok(())
                    },
                },
                Command {
                    name: "select-all",
                    aliases: &[],
                    description: "Select all text",
                    keybinding: Some("Ctrl+A"),
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.selection_start = Some((0, 0));
                        let last_row = pane.buffer.line_count() - 1;
                        let last_col = pane.buffer.get_line(last_row).map(|l| l.len()).unwrap_or(0);
                        pane.cursor.y = last_row;
                        pane.cursor.x = last_col;
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "copy",
                    aliases: &["y", "yank"],
                    description: "Copy selection or line",
                    keybinding: Some("Ctrl+C"),
                    action: |editor, _| {
                        if editor.active_pane().has_selection() {
                            if let Some(text) = editor.active_pane().get_selected_text() {
                                if let Some(clipboard) = &mut editor.clipboard {
                                    let _ = clipboard.set_text(text);
                                }
                            }
                            editor.active_pane_mut().clear_selection();
                        } else {
                            if let Some(line) = editor.active_pane().buffer.get_line(editor.active_pane().cursor.y).cloned() {
                                if let Some(clipboard) = &mut editor.clipboard {
                                    let _ = clipboard.set_text(line);
                                }
                            }
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "paste",
                    aliases: &["p"],
                    description: "Paste from clipboard",
                    keybinding: Some("Ctrl+V"),
                    action: |editor, _| {
                        if let Some(clipboard) = &mut editor.clipboard {
                            if let Ok(text) = clipboard.get_text() {
                                let pane = editor.active_pane_mut();
                                for ch in text.chars() {
                                    if ch == '\n' {
                                        let command = crate::command::EditCommand::InsertNewline {
                                            row: pane.cursor.y,
                                            col: pane.cursor.x,
                                        };
                                        pane.execute_command(command);
                                        pane.cursor.y += 1;
                                        pane.cursor.x = 0;
                                    } else if ch != '\r' {
                                        let command = crate::command::EditCommand::InsertChar {
                                            row: pane.cursor.y,
                                            col: pane.cursor.x,
                                            ch,
                                        };
                                        pane.execute_command(command);
                                        pane.cursor.x += 1;
                                    }
                                }
                            }
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "undo",
                    aliases: &["u"],
                    description: "Undo last change",
                    keybinding: Some("Ctrl+Z"),
                    action: |editor, _| {
                        editor.active_pane_mut().undo();
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "redo",
                    aliases: &[],
                    description: "Redo last change",
                    keybinding: Some("Ctrl+Y"),
                    action: |editor, _| {
                        editor.active_pane_mut().redo();
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "split-h",
                    aliases: &["sp", "hsplit"],
                    description: "Split horizontally",
                    keybinding: Some("Ctrl+H"),
                    action: |editor, _| {
                        editor.split_horizontal();
                        Ok(())
                    },
                },
                Command {
                    name: "split-v",
                    aliases: &["vs", "vsplit"],
                    description: "Split vertically",
                    keybinding: Some("Ctrl+K"),
                    action: |editor, _| {
                        editor.split_vertical();
                        Ok(())
                    },
                },
                Command {
                    name: "close-split",
                    aliases: &["close"],
                    description: "Close current split",
                    keybinding: Some("Ctrl+X"),
                    action: |editor, _| {
                        if editor.active_pane().modified {
                            Err("File modified! Save first or use :close! to force close".to_string())
                        } else {
                            editor.close_split();
                            Ok(())
                        }
                    },
                },
                Command {
                    name: "force-close-split",
                    aliases: &["close!"],
                    description: "Force close split without saving",
                    keybinding: None,
                    action: |editor, _| {
                        editor.close_split();
                        Ok(())
                    },
                },
                Command {
                    name: "toggle-numbers",
                    aliases: &["numbers"],
                    description: "Toggle line numbers",
                    keybinding: Some("Ctrl+L"),
                    action: |editor, _| {
                        editor.toggle_line_numbers();
                        Ok(())
                    },
                },
                Command {
                    name: "new",
                    aliases: &[],
                    description: "Create new buffer",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.buffer = crate::buffer::TextBuffer::new();
                        pane.cursor.x = 0;
                        pane.cursor.y = 0;
                        pane.current_file = None;
                        pane.modified = false;
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "cut",
                    aliases: &["x"],
                    description: "Cut selection or line",
                    keybinding: None,
                    action: |editor, _| {
                        if editor.active_pane().has_selection() {
                            // Copy selection
                            if let Some(text) = editor.active_pane().get_selected_text() {
                                if let Some(clipboard) = &mut editor.clipboard {
                                    let _ = clipboard.set_text(text);
                                }
                            }
                            // Delete selection
                            editor.active_pane_mut().delete_selection();
                        } else {
                            // Copy and delete current line
                            let row = editor.active_pane().cursor.y;
                            if let Some(line) = editor.active_pane().buffer.get_line(row).cloned() {
                                if let Some(clipboard) = &mut editor.clipboard {
                                    let _ = clipboard.set_text(line);
                                }
                            }
                            // Delete the line
                            let pane = editor.active_pane_mut();
                            if pane.buffer.line_count() > 1 {
                                pane.buffer.lines.remove(row);
                                if pane.cursor.y >= pane.buffer.line_count() {
                                    pane.cursor.y = pane.buffer.line_count() - 1;
                                }
                                pane.cursor.x = 0;
                                pane.modified = true;
                            } else {
                                pane.buffer.lines[0].clear();
                                pane.cursor.x = 0;
                                pane.modified = true;
                            }
                            pane.undo_tree.clear();
                        }
                        editor.needs_full_redraw = true;
                        editor.message = Some("Cut".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "delete-line",
                    aliases: &["dd"],
                    description: "Delete current line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if pane.buffer.line_count() > 1 {
                            pane.buffer.lines.remove(row);
                            if pane.cursor.y >= pane.buffer.line_count() {
                                pane.cursor.y = pane.buffer.line_count() - 1;
                            }
                            pane.cursor.x = 0;
                            pane.modified = true;
                        } else {
                            pane.buffer.lines[0].clear();
                            pane.cursor.x = 0;
                            pane.modified = true;
                        }
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "duplicate-line",
                    aliases: &["dup"],
                    description: "Duplicate current line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if let Some(line) = pane.buffer.get_line(row).cloned() {
                            pane.buffer.lines.insert(row + 1, line);
                            pane.cursor.y += 1;
                            pane.modified = true;
                        }
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "top",
                    aliases: &["gg"],
                    description: "Go to first line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.cursor.y = 0;
                        pane.cursor.x = 0;
                        pane.offset_y = 0;
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "bottom",
                    aliases: &["G"],
                    description: "Go to last line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.cursor.y = pane.buffer.line_count().saturating_sub(1);
                        pane.cursor.x = 0;
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "next-pane",
                    aliases: &["pane"],
                    description: "Switch to next pane",
                    keybinding: Some("Ctrl+W"),
                    action: |editor, _| {
                        editor.next_pane();
                        Ok(())
                    },
                },
                Command {
                    name: "reload",
                    aliases: &["e!"],
                    description: "Reload file from disk",
                    keybinding: None,
                    action: |editor, _| {
                        if let Some(path) = editor.active_pane().current_file.clone() {
                            let path_str = path.to_string_lossy().to_string();
                            editor.open_file(path_str).map_err(|e| e.to_string())?;
                            editor.message = Some("File reloaded".to_string());
                            Ok(())
                        } else {
                            Err("No file to reload".to_string())
                        }
                    },
                },
                Command {
                    name: "clear",
                    aliases: &[],
                    description: "Clear all text",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.buffer.lines = vec![String::new()];
                        pane.cursor.x = 0;
                        pane.cursor.y = 0;
                        pane.offset_y = 0;
                        pane.modified = true;
                        pane.selection_start = None;
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "replace",
                    aliases: &["s", "sub"],
                    description: "Replace text (old/new or old/new/g)",
                    keybinding: None,
                    action: |editor, args| {
                        let parts: Vec<&str> = args.split('/').collect();
                        if parts.len() < 2 {
                            return Err("Usage: :replace old/new or :replace old/new/g".to_string());
                        }
                        let old = parts[0];
                        let new = parts[1];
                        let global = parts.len() > 2 && parts[2] == "g";

                        let pane = editor.active_pane_mut();
                        let mut count = 0;

                        if global {
                            // Replace all occurrences in file
                            for line in pane.buffer.lines.iter_mut() {
                                while line.contains(old) {
                                    *line = line.replacen(old, new, 1);
                                    count += 1;
                                }
                            }
                        } else {
                            // Replace first occurrence in file (search from beginning)
                            for row in 0..pane.buffer.lines.len() {
                                if let Some(col) = pane.buffer.lines[row].find(old) {
                                    let line = &mut pane.buffer.lines[row];
                                    *line = format!("{}{}{}", &line[..col], new, &line[col + old.len()..]);
                                    pane.cursor.y = row;
                                    pane.cursor.x = col;
                                    count = 1;
                                    break;
                                }
                            }
                        }

                        if count > 0 {
                            pane.modified = true;
                            pane.undo_tree.clear();
                            editor.needs_full_redraw = true;
                            editor.message = Some(format!("Replaced {} occurrence(s)", count));
                            Ok(())
                        } else {
                            Err(format!("'{}' not found", old))
                        }
                    },
                },
                Command {
                    name: "word-count",
                    aliases: &["wc"],
                    description: "Count words, lines, chars",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane();
                        let lines = pane.buffer.line_count();
                        let chars: usize = pane.buffer.lines.iter().map(|l| l.len()).sum();
                        let words: usize = pane.buffer.lines.iter()
                            .map(|l| l.split_whitespace().count())
                            .sum();
                        editor.message = Some(format!("{} lines, {} words, {} chars", lines, words, chars));
                        Ok(())
                    },
                },
                Command {
                    name: "indent",
                    aliases: &[">"],
                    description: "Indent current line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if row < pane.buffer.lines.len() {
                            pane.buffer.lines[row].insert_str(0, "    ");
                            pane.cursor.x += 4;
                            pane.modified = true;
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "unindent",
                    aliases: &["<"],
                    description: "Unindent current line",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if row < pane.buffer.lines.len() {
                            let line = &pane.buffer.lines[row];
                            let spaces = line.chars().take_while(|c| *c == ' ').count().min(4);
                            if spaces > 0 {
                                pane.buffer.lines[row] = pane.buffer.lines[row][spaces..].to_string();
                                pane.cursor.x = pane.cursor.x.saturating_sub(spaces);
                                pane.modified = true;
                            }
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "help",
                    aliases: &["?"],
                    description: "Show available commands",
                    keybinding: None,
                    action: |editor, _| {
                        editor.message = Some("Use Ctrl+P for command palette, or :command".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "sort",
                    aliases: &[],
                    description: "Sort all lines alphabetically",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.buffer.lines.sort();
                        pane.modified = true;
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        editor.message = Some("Lines sorted (undo history cleared)".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "reverse",
                    aliases: &[],
                    description: "Reverse line order",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        pane.buffer.lines.reverse();
                        pane.modified = true;
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        editor.message = Some("Lines reversed (undo history cleared)".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "uppercase",
                    aliases: &["upper"],
                    description: "Convert selection/line to uppercase",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if row < pane.buffer.lines.len() {
                            pane.buffer.lines[row] = pane.buffer.lines[row].to_uppercase();
                            pane.modified = true;
                            pane.undo_tree.clear();
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "lowercase",
                    aliases: &["lower"],
                    description: "Convert selection/line to lowercase",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        let row = pane.cursor.y;
                        if row < pane.buffer.lines.len() {
                            pane.buffer.lines[row] = pane.buffer.lines[row].to_lowercase();
                            pane.modified = true;
                            pane.undo_tree.clear();
                        }
                        editor.needs_full_redraw = true;
                        Ok(())
                    },
                },
                Command {
                    name: "trim",
                    aliases: &[],
                    description: "Trim trailing whitespace from all lines",
                    keybinding: None,
                    action: |editor, _| {
                        let pane = editor.active_pane_mut();
                        for line in pane.buffer.lines.iter_mut() {
                            *line = line.trim_end().to_string();
                        }
                        pane.modified = true;
                        pane.undo_tree.clear();
                        editor.needs_full_redraw = true;
                        editor.message = Some("Trimmed trailing whitespace".to_string());
                        Ok(())
                    },
                },
                // Checkpoint and branching undo commands
                Command {
                    name: "checkpoint",
                    aliases: &["cp", "mark"],
                    description: "Create named checkpoint (e.g., :checkpoint before-refactor)",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            return Err("Usage: :checkpoint <name>".to_string());
                        }
                        let name = args.trim().to_string();
                        editor.active_pane_mut().undo_tree.create_checkpoint(name.clone())?;
                        editor.message = Some(format!("Checkpoint '{}' created", name));
                        Ok(())
                    },
                },
                Command {
                    name: "checkpoints",
                    aliases: &["cps", "marks"],
                    description: "List all checkpoints",
                    keybinding: None,
                    action: |editor, _| {
                        let checkpoints = editor.active_pane().undo_tree.list_checkpoints();
                        if checkpoints.is_empty() {
                            editor.message = Some("No checkpoints".to_string());
                        } else {
                            let names: Vec<&str> = checkpoints.iter().map(|cp| cp.name.as_str()).collect();
                            editor.message = Some(format!("Checkpoints: {}", names.join(", ")));
                        }
                        Ok(())
                    },
                },
                Command {
                    name: "rewind",
                    aliases: &["rw", "restore"],
                    description: "Rewind to checkpoint (e.g., :rewind before-refactor)",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            return Err("Usage: :rewind <checkpoint-name>".to_string());
                        }
                        let name = args.trim();
                        let pane = editor.active_pane_mut();
                        pane.undo_tree.rewind_to_checkpoint(name, &mut pane.buffer)?;
                        pane.modified = true;
                        editor.needs_full_redraw = true;
                        editor.message = Some(format!("Rewound to '{}'", name));
                        Ok(())
                    },
                },
                Command {
                    name: "delete-checkpoint",
                    aliases: &["dcp"],
                    description: "Delete a checkpoint",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            return Err("Usage: :delete-checkpoint <name>".to_string());
                        }
                        let name = args.trim();
                        if editor.active_pane_mut().undo_tree.delete_checkpoint(name) {
                            editor.message = Some(format!("Checkpoint '{}' deleted", name));
                            Ok(())
                        } else {
                            Err(format!("Checkpoint '{}' not found", name))
                        }
                    },
                },
                Command {
                    name: "branches",
                    aliases: &["br"],
                    description: "Show number of undo branches at current position",
                    keybinding: None,
                    action: |editor, _| {
                        let info = editor.active_pane().undo_tree.status_info();
                        editor.message = Some(info);
                        Ok(())
                    },
                },
                Command {
                    name: "redo-branch",
                    aliases: &["rb"],
                    description: "Redo along specific branch (e.g., :redo-branch 0)",
                    keybinding: None,
                    action: |editor, args| {
                        let branch: usize = args.trim().parse().map_err(|_| "Invalid branch number".to_string())?;
                        let pane = editor.active_pane_mut();
                        if pane.undo_tree.redo_branch(branch, &mut pane.buffer) {
                            pane.modified = true;
                            editor.needs_full_redraw = true;
                            editor.message = Some(format!("Redone branch {}", branch));
                            Ok(())
                        } else {
                            Err(format!("Branch {} not available", branch))
                        }
                    },
                },
                Command {
                    name: "undo-status",
                    aliases: &["us"],
                    description: "Show undo tree status",
                    keybinding: None,
                    action: |editor, _| {
                        let info = editor.active_pane().undo_tree.status_info();
                        editor.message = Some(info);
                        Ok(())
                    },
                },
                // Configuration commands
                Command {
                    name: "theme",
                    aliases: &[],
                    description: "Set color theme (default, monokai, dracula, nord, gruvbox)",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            let themes = crate::config::theme::available_themes();
                            editor.message = Some(format!("Current: {}. Available: {}",
                                editor.config.theme, themes.join(", ")));
                            return Ok(());
                        }
                        let name = args.trim().to_lowercase();
                        let themes = crate::config::theme::available_themes();
                        if themes.contains(&name.as_str()) {
                            editor.config.theme = name.clone();
                            editor.theme = crate::config::Theme::from_name(&name);
                            editor.needs_full_redraw = true;
                            editor.message = Some(format!("Theme set to '{}'", name));
                            Ok(())
                        } else {
                            Err(format!("Unknown theme '{}'. Available: {}", name, themes.join(", ")))
                        }
                    },
                },
                Command {
                    name: "set",
                    aliases: &[],
                    description: "Set editor option (e.g., :set tabsize 4)",
                    keybinding: None,
                    action: |editor, args| {
                        let parts: Vec<&str> = args.split_whitespace().collect();
                        if parts.is_empty() {
                            editor.message = Some(format!(
                                "tabsize={} spaces={} linenumbers={} autoindent={}",
                                editor.config.editor.tab_size,
                                editor.config.editor.use_spaces,
                                editor.config.editor.show_line_numbers,
                                editor.config.editor.auto_indent
                            ));
                            return Ok(());
                        }

                        match parts[0].to_lowercase().as_str() {
                            "tabsize" | "ts" => {
                                if parts.len() < 2 {
                                    return Err("Usage: :set tabsize <number>".to_string());
                                }
                                let size: usize = parts[1].parse().map_err(|_| "Invalid number")?;
                                if size == 0 || size > 16 {
                                    return Err("Tab size must be 1-16".to_string());
                                }
                                editor.config.editor.tab_size = size;
                                editor.message = Some(format!("Tab size set to {}", size));
                            }
                            "spaces" => {
                                let val = parts.get(1).map(|s| *s).unwrap_or("true");
                                editor.config.editor.use_spaces = val == "true" || val == "on" || val == "1";
                                editor.message = Some(format!("Use spaces: {}", editor.config.editor.use_spaces));
                            }
                            "linenumbers" | "ln" | "numbers" => {
                                let val = parts.get(1).map(|s| *s).unwrap_or("toggle");
                                if val == "toggle" {
                                    editor.show_line_numbers = !editor.show_line_numbers;
                                } else {
                                    editor.show_line_numbers = val == "true" || val == "on" || val == "1";
                                }
                                editor.config.editor.show_line_numbers = editor.show_line_numbers;
                                editor.needs_full_redraw = true;
                                editor.message = Some(format!("Line numbers: {}", editor.show_line_numbers));
                            }
                            "autoindent" | "ai" => {
                                let val = parts.get(1).map(|s| *s).unwrap_or("toggle");
                                if val == "toggle" {
                                    editor.config.editor.auto_indent = !editor.config.editor.auto_indent;
                                } else {
                                    editor.config.editor.auto_indent = val == "true" || val == "on" || val == "1";
                                }
                                editor.message = Some(format!("Auto indent: {}", editor.config.editor.auto_indent));
                            }
                            "wordwrap" | "wrap" => {
                                let val = parts.get(1).map(|s| *s).unwrap_or("toggle");
                                if val == "toggle" {
                                    editor.config.editor.word_wrap = !editor.config.editor.word_wrap;
                                } else {
                                    editor.config.editor.word_wrap = val == "true" || val == "on" || val == "1";
                                }
                                editor.needs_full_redraw = true;
                                editor.message = Some(format!("Word wrap: {}", editor.config.editor.word_wrap));
                            }
                            _ => {
                                return Err(format!("Unknown option: {}. Try: tabsize, spaces, linenumbers, autoindent, wordwrap", parts[0]));
                            }
                        }
                        Ok(())
                    },
                },
                Command {
                    name: "config",
                    aliases: &["cfg"],
                    description: "Show config file path or generate default config",
                    keybinding: None,
                    action: |editor, args| {
                        match args.trim() {
                            "path" | "" => {
                                if let Some(path) = crate::config::Config::config_path() {
                                    editor.message = Some(format!("Config: {}", path.display()));
                                } else {
                                    editor.message = Some("Could not determine config path".to_string());
                                }
                            }
                            "save" => {
                                editor.config.save()?;
                                if let Some(path) = crate::config::Config::config_path() {
                                    editor.message = Some(format!("Config saved to {}", path.display()));
                                }
                            }
                            "reload" => {
                                editor.config = crate::config::Config::load();
                                editor.theme = editor.config.get_theme();
                                editor.show_line_numbers = editor.config.editor.show_line_numbers;
                                editor.needs_full_redraw = true;
                                editor.message = Some("Config reloaded".to_string());
                            }
                            "default" => {
                                let default = crate::config::Config::default_config_string();
                                editor.message = Some(format!("Default config:\n{}", &default[..100.min(default.len())]));
                            }
                            _ => {
                                return Err("Usage: :config [path|save|reload|default]".to_string());
                            }
                        }
                        Ok(())
                    },
                },
                // Tab commands
                Command {
                    name: "tabnew",
                    aliases: &["tn", "new"],
                    description: "Open a new empty tab",
                    keybinding: Some("Ctrl+T"),
                    action: |editor, args| {
                        editor.new_tab();
                        if !args.is_empty() {
                            // Open file in new tab
                            editor.open_file(args.to_string()).map_err(|e| e.to_string())?;
                        }
                        Ok(())
                    },
                },
                Command {
                    name: "tabnext",
                    aliases: &["tn", "tabn"],
                    description: "Switch to next tab",
                    keybinding: Some("Ctrl+Tab"),
                    action: |editor, _| {
                        editor.next_tab();
                        Ok(())
                    },
                },
                Command {
                    name: "tabprev",
                    aliases: &["tp", "tabp"],
                    description: "Switch to previous tab",
                    keybinding: Some("Ctrl+Shift+Tab"),
                    action: |editor, _| {
                        editor.prev_tab();
                        Ok(())
                    },
                },
                Command {
                    name: "tabclose",
                    aliases: &["tc", "tabc"],
                    description: "Close current tab",
                    keybinding: None,
                    action: |editor, _| {
                        if editor.active_pane().modified {
                            return Err("File modified! Save first or use :tabclose! to force close".to_string());
                        }
                        if editor.close_tab() {
                            Ok(())
                        } else {
                            Err("Cannot close last tab. Use :quit to exit.".to_string())
                        }
                    },
                },
                Command {
                    name: "tabclose!",
                    aliases: &["tc!"],
                    description: "Force close current tab without saving",
                    keybinding: None,
                    action: |editor, _| {
                        if editor.close_tab() {
                            Ok(())
                        } else {
                            Err("Cannot close last tab. Use :q! to exit.".to_string())
                        }
                    },
                },
                Command {
                    name: "tab",
                    aliases: &["t"],
                    description: "Switch to tab by number (e.g., :tab 2)",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            editor.message = Some(format!("Tab {}/{}", editor.focused_tab_index() + 1, editor.tab_count()));
                            return Ok(());
                        }
                        let tab_num: usize = args.parse().map_err(|_| "Invalid tab number".to_string())?;
                        if tab_num < 1 || tab_num > editor.tab_count() {
                            return Err(format!("Tab {} doesn't exist (1-{})", tab_num, editor.tab_count()));
                        }
                        editor.goto_tab(tab_num - 1);
                        Ok(())
                    },
                },
                Command {
                    name: "tabs",
                    aliases: &["ls", "buffers"],
                    description: "List all open tabs",
                    keybinding: None,
                    action: |editor, _| {
                        let mut tab_list = String::new();
                        for (idx, tab) in editor.tabs.iter().enumerate() {
                            let name = tab.display_name();
                            let modified = if tab.is_modified() { "*" } else { "" };
                            let split = if tab.is_split() { "⧫" } else { "" };
                            let active = if idx == editor.focused_tab_index() { ">" } else { " " };
                            tab_list.push_str(&format!("{}{}: {}{}{} ", active, idx + 1, name, modified, split));
                        }
                        editor.message = Some(tab_list);
                        Ok(())
                    },
                },
                Command {
                    name: "tabonly",
                    aliases: &["tabo"],
                    description: "Close all other tabs",
                    keybinding: None,
                    action: |editor, _| {
                        // Check if any other tab is modified
                        for (idx, tab) in editor.tabs.iter().enumerate() {
                            if idx != editor.active_tab && tab.is_modified() {
                                return Err(format!("Tab {} is modified! Save first or use :tabonly!", idx + 1));
                            }
                        }
                        // Keep only the active tab
                        let active_tab = editor.tabs.remove(editor.active_tab);
                        editor.tabs.clear();
                        editor.tabs.push(active_tab);
                        editor.active_tab = 0;
                        editor.needs_full_redraw = true;
                        editor.message = Some("Closed all other tabs".to_string());
                        Ok(())
                    },
                },
                Command {
                    name: "tabedit",
                    aliases: &["tabe"],
                    description: "Open file in new tab",
                    keybinding: None,
                    action: |editor, args| {
                        if args.is_empty() {
                            return Err("Usage: :tabedit <filename>".to_string());
                        }
                        editor.new_tab();
                        editor.open_file(args.to_string()).map_err(|e| e.to_string())
                    },
                },
            ],
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == name)
    }

    pub fn find_by_alias(&self, alias: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == alias || c.aliases.contains(&alias))
    }

    pub fn filter(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            return self.commands.iter().collect();
        }
        let query_lower = query.to_lowercase();
        let mut results: Vec<_> = self.commands
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower) ||
                c.description.to_lowercase().contains(&query_lower) ||
                c.aliases.iter().any(|a| a.to_lowercase().contains(&query_lower))
            })
            .collect();

        // Sort by relevance: exact prefix match first
        results.sort_by(|a, b| {
            let a_starts = a.name.to_lowercase().starts_with(&query_lower);
            let b_starts = b.name.to_lowercase().starts_with(&query_lower);
            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        results
    }

    pub fn all(&self) -> &[Command] {
        &self.commands
    }
}
