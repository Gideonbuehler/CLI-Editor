use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum TokenType {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Normal,
}

impl TokenType {
    fn color(&self) -> Color {
        match self {
            TokenType::Keyword => Color::Magenta,
            TokenType::String => Color::Green,
            TokenType::Comment => Color::DarkGrey,
            TokenType::Number => Color::Cyan,
            TokenType::Function => Color::Yellow,
            TokenType::Type => Color::Blue,
            TokenType::Normal => Color::White,
        }
    }
}

#[derive(Clone)]
struct SyntaxHighlighter {
    language: Language,
}

#[derive(Clone, Copy, PartialEq)]
enum Language {
    Rust,
    Python,
    JavaScript,
    C,
    Bash,
    Plain,
}

impl Language {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" | "jsx" | "ts" | "tsx" => Language::JavaScript,
            "c" | "h" | "cpp" | "hpp" | "cc" => Language::C,
            "sh" | "bash" => Language::Bash,
            _ => Language::Plain,
        }
    }

    fn keywords(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while",
                "loop", "break", "continue", "return", "struct", "enum", "trait", "impl", "pub",
                "use", "mod", "crate", "self", "super", "as", "move", "ref", "unsafe", "async",
                "await", "dyn", "where", "type", "in",
            ],
            Language::Python => &[
                "def", "class", "if", "elif", "else", "for", "while", "break", "continue",
                "return", "try", "except", "finally", "with", "as", "import", "from", "pass",
                "raise", "assert", "lambda", "yield", "async", "await", "global", "nonlocal",
                "True", "False", "None", "and", "or", "not", "in", "is",
            ],
            Language::JavaScript => &[
                "function", "const", "let", "var", "if", "else", "for", "while", "break",
                "continue", "return", "class", "extends", "super", "this", "new", "try", "catch",
                "finally", "throw", "async", "await", "import", "export", "from", "default",
                "switch", "case", "typeof", "instanceof", "delete", "void", "yield",
            ],
            Language::C => &[
                "int", "char", "float", "double", "void", "struct", "union", "enum", "if",
                "else", "for", "while", "do", "break", "continue", "return", "switch", "case",
                "default", "sizeof", "typedef", "static", "const", "extern", "auto", "register",
                "volatile", "unsigned", "signed", "long", "short",
            ],
            Language::Bash => &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case",
                "esac", "function", "return", "exit", "break", "continue", "local", "export",
                "source", "alias", "echo", "read", "test",
            ],
            Language::Plain => &[],
        }
    }

    fn types(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Cell", "RefCell",
            ],
            Language::C => &["int", "char", "float", "double", "void", "size_t", "uint8_t", "uint16_t", "uint32_t"],
            _ => &[],
        }
    }
}

impl SyntaxHighlighter {
    fn new(language: Language) -> Self {
        Self { language }
    }

    fn highlight_line(&self, line: &str) -> Vec<(String, TokenType)> {
        if self.language == Language::Plain {
            return vec![(line.to_string(), TokenType::Normal)];
        }

        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = line.chars().peekable();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_comment = false;

        while let Some(ch) = chars.next() {
            // Handle comments
            if !in_string && self.is_comment_start(ch, chars.peek().copied()) {
                if !current.is_empty() {
                    self.push_token(&mut tokens, current.clone());
                    current.clear();
                }
                in_comment = true;
                current.push(ch);
                if let Some(next) = chars.peek() {
                    if *next == '/' || *next == '*' {
                        current.push(chars.next().unwrap());
                    }
                }
                continue;
            }

            if in_comment {
                current.push(ch);
                continue;
            }

            // Handle strings
            if (ch == '"' || ch == '\'' || ch == '`') && !in_string {
                if !current.is_empty() {
                    self.push_token(&mut tokens, current.clone());
                    current.clear();
                }
                in_string = true;
                string_char = ch;
                current.push(ch);
                continue;
            }

            if in_string {
                current.push(ch);
                if ch == string_char && current.chars().rev().nth(1) != Some('\\') {
                    tokens.push((current.clone(), TokenType::String));
                    current.clear();
                    in_string = false;
                }
                continue;
            }

            // Handle numbers
            if ch.is_numeric() && (current.is_empty() || current.chars().all(|c| c.is_numeric() || c == '.')) {
                current.push(ch);
                continue;
            }

            // Handle identifiers and keywords
            if ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
                continue;
            }

            // We hit a separator
            if !current.is_empty() {
                self.push_token(&mut tokens, current.clone());
                current.clear();
            }

            // Add the separator as is
            tokens.push((ch.to_string(), TokenType::Normal));
        }

        // Handle remaining content
        if in_comment {
            tokens.push((current, TokenType::Comment));
        } else if in_string {
            tokens.push((current, TokenType::String));
        } else if !current.is_empty() {
            self.push_token(&mut tokens, current);
        }

        tokens
    }

    fn is_comment_start(&self, ch: char, next: Option<char>) -> bool {
        match self.language {
            Language::Rust | Language::C | Language::JavaScript => {
                ch == '/' && (next == Some('/') || next == Some('*'))
            }
            Language::Python | Language::Bash => ch == '#',
            Language::Plain => false,
        }
    }

    fn push_token(&self, tokens: &mut Vec<(String, TokenType)>, token: String) {
        let token_type = if self.language.keywords().contains(&token.as_str()) {
            TokenType::Keyword
        } else if self.language.types().contains(&token.as_str()) {
            TokenType::Type
        } else if token.chars().all(|c| c.is_numeric() || c == '.') {
            TokenType::Number
        } else {
            TokenType::Normal
        };

        tokens.push((token, token_type));
    }
}

// Command pattern for undo/redo
#[derive(Clone, Debug)]
enum EditCommand {
    InsertChar { row: usize, col: usize, ch: char },
    DeleteChar { row: usize, col: usize, ch: char },
    InsertNewline { row: usize, col: usize },
    DeleteNewline { row: usize, deleted_line: String },
    ClearAll { old_content: Vec<String> }, // Add this
}

impl EditCommand {
    fn undo(&self, buffer: &mut TextBuffer) {
    match self {
        EditCommand::InsertChar { row, col, .. } => {
            if *row < buffer.lines.len() && *col < buffer.lines[*row].len() {
                buffer.lines[*row].remove(*col);
            }
        }
        EditCommand::DeleteChar { row, col, ch } => {
            if *row < buffer.lines.len() {
                buffer.lines[*row].insert(*col, *ch);
            }
        }
        EditCommand::InsertNewline { row, col: _ } => {
            if *row + 1 < buffer.lines.len() {
                let line = buffer.lines.remove(*row + 1);
                buffer.lines[*row].push_str(&line);
            }
        }
        EditCommand::DeleteNewline { row, deleted_line } => {
            if *row < buffer.lines.len() {
                let current_len = buffer.lines[*row].len();
                let deleted_len = deleted_line.len();
                buffer.lines[*row].truncate(current_len - deleted_len);
                buffer.lines.insert(*row + 1, deleted_line.clone());
            }
        }
        EditCommand::ClearAll { old_content } => {
            buffer.lines = old_content.clone();
        }
    }
}

fn redo(&self, buffer: &mut TextBuffer) {
    match self {
        EditCommand::InsertChar { row, col, ch } => {
            if *row < buffer.lines.len() {
                buffer.lines[*row].insert(*col, *ch);
            }
        }
        EditCommand::DeleteChar { row, col, .. } => {
            if *row < buffer.lines.len() && *col < buffer.lines[*row].len() {
                buffer.lines[*row].remove(*col);
            }
        }
        EditCommand::InsertNewline { row, col } => {
            if *row < buffer.lines.len() {
                let current_line = &buffer.lines[*row];
                let new_line = current_line[*col..].to_string();
                buffer.lines[*row].truncate(*col);
                buffer.lines.insert(*row + 1, new_line);
            }
        }
        EditCommand::DeleteNewline { row, .. } => {
            if *row > 0 && *row < buffer.lines.len() {
                let line = buffer.lines.remove(*row);
                buffer.lines[*row - 1].push_str(&line);
            }
        }
        EditCommand::ClearAll { .. } => {
            buffer.lines = vec![String::new()];
        }
    }
}
}

#[derive(Clone)]
struct TextBuffer {
    lines: Vec<String>,
}

impl TextBuffer {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    fn from_string(content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
        }
    }

    fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if row < self.lines.len() {
            self.lines[row].insert(col, ch);
        }
    }

    fn delete_char(&mut self, row: usize, col: usize) -> Option<char> {
        if row < self.lines.len() && col > 0 && col <= self.lines[row].len() {
            Some(self.lines[row].remove(col - 1))
        } else {
            None
        }
    }

    fn insert_newline(&mut self, row: usize, col: usize) {
        if row < self.lines.len() {
            let current_line = &self.lines[row];
            let new_line = current_line[col..].to_string();
            self.lines[row].truncate(col);
            self.lines.insert(row + 1, new_line);
        }
    }

    fn delete_newline(&mut self, row: usize) -> Option<String> {
        if row > 0 && row < self.lines.len() {
            let line = self.lines.remove(row);
            self.lines[row - 1].push_str(&line);
            Some(line)
        } else {
            None
        }
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn get_line(&self, row: usize) -> Option<&String> {
        self.lines.get(row)
    }

    fn search(&self, query: &str, start_row: usize, start_col: usize) -> Option<(usize, usize)> {
        if query.is_empty() {
            return None;
        }

        // Search from current position to end
        for row in start_row..self.lines.len() {
            let search_col = if row == start_row { start_col } else { 0 };
            if let Some(col) = self.lines[row][search_col..].find(query) {
                return Some((row, search_col + col));
            }
        }

        // Wrap around: search from beginning to start position
        for row in 0..=start_row {
            let end_col = if row == start_row { start_col } else { self.lines[row].len() };
            if let Some(col) = self.lines[row][..end_col].find(query) {
                return Some((row, col));
            }
        }

        None
    }
}

#[derive(Clone)]
struct Cursor {
    x: usize,
    y: usize,
}

enum EditorMode {
    Normal,
    Search,
    SavePrompt,
    OpenPrompt,
}

#[derive(Clone)]
struct Pane {
    buffer: TextBuffer,
    cursor: Cursor,
    offset_y: usize,
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    current_file: Option<PathBuf>,
    modified: bool,
    search_query: String,
    last_search_pos: Option<(usize, usize)>,
    highlighter: SyntaxHighlighter, // Add this
}

impl Pane {
    fn new() -> Self {
    Self {
        buffer: TextBuffer::new(),
        cursor: Cursor { x: 0, y: 0 },
        offset_y: 0,
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
        current_file: None,
        modified: false,
        search_query: String::new(),
        last_search_pos: None,
        highlighter: SyntaxHighlighter::new(Language::Plain), // Add this
    }
}

    fn execute_command(&mut self, command: EditCommand) {
        command.redo(&mut self.buffer);
        self.undo_stack.push(command);
        self.redo_stack.clear();
        self.modified = true;
    }

    fn undo(&mut self) {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(&mut self.buffer);
            self.redo_stack.push(command);
            self.modified = !self.undo_stack.is_empty();
        }
    }

    fn redo(&mut self) {
        if let Some(command) = self.redo_stack.pop() {
            command.redo(&mut self.buffer);
            self.undo_stack.push(command);
            self.modified = true;
        }
    }

    fn adjust_scroll(&mut self, visible_lines: usize) {
        if self.cursor.y < self.offset_y {
            self.offset_y = self.cursor.y;
        } else if self.cursor.y >= self.offset_y + visible_lines {
            self.offset_y = self.cursor.y - visible_lines + 1;
        }
    }
}

enum SplitMode {
    None,
    Horizontal,
    Vertical,
}

struct Editor {
    panes: Vec<Pane>,
    active_pane: usize,
    split_mode: SplitMode,
    should_quit: bool,
    needs_full_redraw: bool,
    mode: EditorMode,
    input_buffer: String,
    message: Option<String>,
    show_line_numbers: bool,
    quit_warning_shown: bool, // Add this line
}

impl Editor {
    fn new() -> Self {
    Self {
        panes: vec![Pane::new()],
        active_pane: 0,
        split_mode: SplitMode::None,
        should_quit: false,
        needs_full_redraw: true,
        mode: EditorMode::Normal,
        input_buffer: String::new(),
        message: None,
        show_line_numbers: true,
        quit_warning_shown: false, // Add this line
    }
}

    fn active_pane(&self) -> &Pane {
        &self.panes[self.active_pane]
    }

    fn active_pane_mut(&mut self) -> &mut Pane {
        &mut self.panes[self.active_pane]
    }

    fn split_horizontal(&mut self) {
        if self.panes.len() < 2 {
            // Create a new empty pane instead of cloning
            self.panes.push(Pane::new());
            self.split_mode = SplitMode::Horizontal;
            self.needs_full_redraw = true;
        }
    }

    fn split_vertical(&mut self) {
        if self.panes.len() < 2 {
            // Create a new empty pane instead of cloning
            self.panes.push(Pane::new());
            self.split_mode = SplitMode::Vertical;
            self.needs_full_redraw = true;
        }
    }

    fn close_split(&mut self) {
        if self.panes.len() > 1 {
            self.panes.remove(self.active_pane);
            if self.active_pane >= self.panes.len() {
                self.active_pane = self.panes.len() - 1;
            }
            self.split_mode = SplitMode::None;
            self.needs_full_redraw = true;
        }
    }

    fn next_pane(&mut self) {
        if self.panes.len() > 1 {
            self.active_pane = (self.active_pane + 1) % self.panes.len();
            self.needs_full_redraw = true;
        }
    }

    fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.needs_full_redraw = true;
    }

    fn get_line_number_width(&self) -> usize {
        if !self.show_line_numbers {
            return 0;
        }
        let max_line = self.active_pane().buffer.line_count();
        format!("{}", max_line).len() + 1
    }

    fn save_file(&mut self) -> io::Result<()> {
        let pane = self.active_pane_mut();
        if let Some(path) = &pane.current_file.clone() {
            fs::write(path, pane.buffer.to_string())?;
            pane.modified = false;
            self.message = Some(format!("Saved to {}", path.display()));
            Ok(())
        } else {
            self.mode = EditorMode::SavePrompt;
            self.input_buffer.clear();
            self.message = Some("Enter filename: ".to_string());
            self.needs_full_redraw = true;
            Ok(())
        }
    }

    fn save_file_as(&mut self, filename: String) -> io::Result<()> {
        let path = PathBuf::from(filename);
        let pane = self.active_pane_mut();
        fs::write(&path, pane.buffer.to_string())?;
        pane.current_file = Some(path.clone());
        pane.modified = false;
        self.message = Some(format!("Saved to {}", path.display()));
        Ok(())
    }

    fn open_file(&mut self, filename: String) -> io::Result<()> {
    let path = PathBuf::from(filename);
    let content = fs::read_to_string(&path)?;
    let pane = self.active_pane_mut();
    pane.buffer = TextBuffer::from_string(content);
    pane.current_file = Some(path.clone());
    pane.modified = false;
    pane.cursor = Cursor { x: 0, y: 0 };
    pane.offset_y = 0;
    pane.undo_stack.clear();
    pane.redo_stack.clear();
    
    // Detect language from file extension
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            pane.highlighter = SyntaxHighlighter::new(Language::from_extension(ext_str));
        }
    }
    
    self.message = Some(format!("Opened {}", path.display()));
    self.needs_full_redraw = true;
    Ok(())
}

    fn start_search(&mut self) {
        self.mode = EditorMode::Search;
        self.input_buffer.clear();
        self.message = Some("Search: ".to_string());
        self.needs_full_redraw = true;
    }

    fn perform_search(&mut self) {
    if self.input_buffer.is_empty() {
        self.message = Some("Search cancelled".to_string());
        return;
    }

    // Clone the search query to avoid borrow issues
    let search_query = self.input_buffer.clone();
    let (_, height) = terminal::size().unwrap_or((80, 24));
    let visible_lines = self.calculate_visible_lines(height);
    
    // Scope the mutable borrow
    let search_result = {
        let pane = self.active_pane_mut();
        pane.search_query = search_query.clone();

        let start_pos = if let Some((row, col)) = pane.last_search_pos {
            if col + 1 < pane.buffer.get_line(row).map(|l| l.len()).unwrap_or(0) {
                (row, col + 1)
            } else if row + 1 < pane.buffer.line_count() {
                (row + 1, 0)
            } else {
                (0, 0)
            }
        } else {
            (pane.cursor.y, pane.cursor.x)
        };

        pane.buffer.search(&pane.search_query, start_pos.0, start_pos.1)
    }; // Mutable borrow ends here

    // Now we can safely borrow again
    if let Some((row, col)) = search_result {
        let pane = self.active_pane_mut();
        pane.cursor.y = row;
        pane.cursor.x = col;
        pane.last_search_pos = Some((row, col));
        pane.adjust_scroll(visible_lines);
        self.message = Some(format!("Found at line {}, col {}", row + 1, col + 1));
        self.needs_full_redraw = true;
    } else {
        let pane = self.active_pane_mut();
        pane.last_search_pos = None;
        self.message = Some(format!("Not found: {}", search_query));
    }
}

    fn find_next(&mut self) {
        let search_query = self.active_pane().search_query.clone();
        if !search_query.is_empty() {
            self.input_buffer = search_query;
            self.perform_search();
        }
    }

    fn calculate_visible_lines(&self, height: u16) -> usize {
        match self.split_mode {
            SplitMode::None => (height - 2) as usize,
            SplitMode::Horizontal => ((height - 3) / 2) as usize,
            SplitMode::Vertical => (height - 2) as usize,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen)?;

        let result = self.main_loop(&mut stdout);

        execute!(stdout, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn main_loop(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        loop {
            self.refresh_screen(stdout)?;

            if self.should_quit {
                break;
            }

            self.process_keypress()?;
        }
        Ok(())
    }

    fn refresh_screen(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (width, height) = terminal::size()?;

        queue!(stdout, cursor::Hide)?;

        if self.needs_full_redraw {
            queue!(stdout, terminal::Clear(ClearType::All))?;

            match self.split_mode {
                SplitMode::None => {
                    self.draw_pane(stdout, 0, 0, width, height - 2, 0)?;
                }
                SplitMode::Horizontal => {
                    let split_height = (height - 3) / 2;
                    self.draw_pane(stdout, 0, 0, width, split_height, 0)?;

                    queue!(stdout, cursor::MoveTo(0, split_height))?;
                    for _ in 0..width {
                        queue!(stdout, Print("─"))?;
                    }

                    self.draw_pane(stdout, 0, split_height + 1, width, split_height, 1)?;
                }
                SplitMode::Vertical => {
                    let split_width = width / 2;
                    self.draw_pane(stdout, 0, 0, split_width, height - 2, 0)?;

                    for row in 0..(height - 2) {
                        queue!(stdout, cursor::MoveTo(split_width, row), Print("│"))?;
                    }

                    self.draw_pane(stdout, split_width + 1, 0, split_width - 1, height - 2, 1)?;
                }
            }

            self.needs_full_redraw = false;
        } else {
            match self.split_mode {
                SplitMode::None => {
                    self.draw_current_line(stdout, 0, 0, width)?;
                }
                SplitMode::Horizontal => {
                    let split_height = (height - 3) / 2;
                    if self.active_pane == 0 {
                        self.draw_current_line(stdout, 0, 0, width)?;
                    } else {
                        self.draw_current_line(stdout, 0, split_height + 1, width)?;
                    }
                }
                SplitMode::Vertical => {
                    let split_width = width / 2;
                    if self.active_pane == 0 {
                        self.draw_current_line(stdout, 0, 0, split_width)?;
                    } else {
                        self.draw_current_line(stdout, split_width + 1, 0, split_width - 1)?;
                    }
                }
            }
        }

        self.draw_status_bar(stdout, height)?;
        self.draw_message_line(stdout, height)?;
        self.position_cursor(stdout, width, height)?;

        stdout.flush()?;
        Ok(())
    }

    fn draw_pane(
    &self,
    stdout: &mut io::Stdout,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
    pane_idx: usize,
) -> io::Result<()> {
    if pane_idx >= self.panes.len() {
        return Ok(());
    }

    let pane = &self.panes[pane_idx];
    let is_active = pane_idx == self.active_pane;
    let line_num_width = self.get_line_number_width();
    let text_width = width.saturating_sub(line_num_width as u16);

    // Find last line with content
    let mut last_content_line = 0;
    for (idx, line) in pane.buffer.lines.iter().enumerate() {
        if !line.trim().is_empty() {
            last_content_line = idx;
        }
    }
    // Ensure at least line 1 is shown
    last_content_line = last_content_line.max(0);

    for screen_row in 0..height as usize {
        let file_row = screen_row + pane.offset_y;
        let screen_y = start_y + screen_row as u16;

        queue!(stdout, cursor::MoveTo(start_x, screen_y))?;

        if self.show_line_numbers {
            // Only show line numbers up to last content line or current line, whichever is greater
            if file_row < pane.buffer.line_count() && file_row <= last_content_line.max(pane.cursor.y) {
                queue!(
                    stdout,
                    SetForegroundColor(if is_active { Color::Yellow } else { Color::DarkGrey }),
                    Print(format!("{:>width$} ", file_row + 1, width = line_num_width - 1)),
                    ResetColor
                )?;
            } else if file_row < pane.buffer.line_count() {
                // Lines after last content but within buffer - show tilde
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("{:>width$} ", "~", width = line_num_width - 1)),
                    ResetColor
                )?;
            } else {
                // Beyond buffer - show tilde
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

        if !pane.search_query.is_empty() && line.contains(&pane.search_query) {
            self.draw_line_with_highlight(stdout, display_line, &pane.search_query)?;
        } else {
            // Use syntax highlighting
            self.draw_line_with_syntax(stdout, display_line, &pane.highlighter)?;
        }
    }
} else if !self.show_line_numbers {
            queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
            queue!(stdout, Print("~"))?;
            queue!(stdout, ResetColor)?;
        }

        queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;
    }

    Ok(())
}

    fn draw_line_with_syntax(
    &self,
    stdout: &mut io::Stdout,
    line: &str,
    highlighter: &SyntaxHighlighter,
) -> io::Result<()> {
    let tokens = highlighter.highlight_line(line);
    
    for (text, token_type) in tokens {
        queue!(
            stdout,
            SetForegroundColor(token_type.color()),
            Print(text),
            ResetColor
        )?;
    }
    
    Ok(())
}

    fn draw_current_line(
    &self,
    stdout: &mut io::Stdout,
    start_x: u16,
    start_y: u16,
    width: u16,
) -> io::Result<()> {
    let pane = self.active_pane();
    let line_num_width = self.get_line_number_width();
    let text_width = width.saturating_sub(line_num_width as u16);

    let screen_y = pane.cursor.y.saturating_sub(pane.offset_y);
    let actual_y = start_y + screen_y as u16;

    queue!(stdout, cursor::MoveTo(start_x, actual_y))?;

    if self.show_line_numbers {
        queue!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!("{:>width$} ", pane.cursor.y + 1, width = line_num_width - 1)),
            ResetColor
        )?;
    }

    if let Some(line) = pane.buffer.get_line(pane.cursor.y) {
        let display_line = if line.len() > text_width as usize {
            &line[..text_width as usize]
        } else {
            line
        };

        if !pane.search_query.is_empty() && line.contains(&pane.search_query) {
            self.draw_line_with_highlight(stdout, display_line, &pane.search_query)?;
        } else {
            self.draw_line_with_syntax(stdout, display_line, &pane.highlighter)?;
        }
    }
    queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;

    Ok(())
}

    fn draw_line_with_highlight(
        &self,
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

    fn draw_status_bar(&self, stdout: &mut io::Stdout, height: u16) -> io::Result<()> {
        let pane = self.active_pane();
        let filename = pane
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");

        let modified_indicator = if pane.modified { " [+]" } else { "" };
        let split_indicator = match self.split_mode {
            SplitMode::None => "",
            SplitMode::Horizontal => " [H-Split]",
            SplitMode::Vertical => " [V-Split]",
        };

        queue!(
            stdout,
            cursor::MoveTo(0, height - 2),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(format!(
                " {} | Pane {}/{} | Line {}/{} Col {}{}{}",
                filename,
                self.active_pane + 1,
                self.panes.len(),
                pane.cursor.y + 1,
                pane.buffer.line_count(),
                pane.cursor.x + 1,
                modified_indicator,
                split_indicator
            )),
            terminal::Clear(ClearType::UntilNewLine),
            ResetColor
        )?;

        Ok(())
    }

    fn draw_message_line(&self, stdout: &mut io::Stdout, height: u16) -> io::Result<()> {
        queue!(stdout, cursor::MoveTo(0, height - 1))?;

        match &self.mode {
            EditorMode::Normal => {
                if let Some(msg) = &self.message {
                    queue!(stdout, Print(msg))?;
                } else {
                    queue!(
                        stdout,
                        Print("^Q:Quit ^S:Save ^O:Open ^F:Search ^N:Next ^Z:Undo ^Y:Redo ^H:HSplit ^V:VSplit ^W:NextPane ^X:CloseSplit ^L:LineNum")
                    )?;
                }
            }
            EditorMode::Search | EditorMode::SavePrompt | EditorMode::OpenPrompt => {
                if let Some(msg) = &self.message {
                    queue!(stdout, Print(format!("{}{}", msg, self.input_buffer)))?;
                }
            }
        }
        queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;

        Ok(())
    }

    fn position_cursor(&self, stdout: &mut io::Stdout, width: u16, height: u16) -> io::Result<()> {
        match self.mode {
            EditorMode::Normal => {
                let pane = self.active_pane();
                let line_num_width = self.get_line_number_width();
                let screen_y = pane.cursor.y.saturating_sub(pane.offset_y);

                let (cursor_x, cursor_y) = match self.split_mode {
                    SplitMode::None => {
                        (line_num_width + pane.cursor.x, screen_y)
                    }
                    SplitMode::Horizontal => {
                        let split_height = (height - 3) / 2;
                        if self.active_pane == 0 {
                            (line_num_width + pane.cursor.x, screen_y)
                        } else {
                            (line_num_width + pane.cursor.x, split_height as usize + 1 + screen_y)
                        }
                    }
                    SplitMode::Vertical => {
                        let split_width = width / 2;
                        if self.active_pane == 0 {
                            (line_num_width + pane.cursor.x, screen_y)
                        } else {
                            (split_width as usize + 1 + line_num_width + pane.cursor.x, screen_y)
                        }
                    }
                };

                queue!(
                    stdout,
                    cursor::MoveTo(cursor_x as u16, cursor_y as u16),
                    cursor::Show
                )?;
            }
            EditorMode::Search | EditorMode::SavePrompt | EditorMode::OpenPrompt => {
                let prompt_len = self.message.as_ref().map(|m| m.len()).unwrap_or(0);
                queue!(
                    stdout,
                    cursor::MoveTo((prompt_len + self.input_buffer.len()) as u16, height - 1),
                    cursor::Show
                )?;
            }
        }

        Ok(())
    }

    fn process_keypress(&mut self) -> io::Result<()> {
        let event = event::read()?;

        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match self.mode {
                    EditorMode::Normal => self.process_normal_mode(key_event)?,
                    EditorMode::Search => self.process_search_mode(key_event)?,
                    EditorMode::SavePrompt => self.process_save_prompt(key_event)?,
                    EditorMode::OpenPrompt => self.process_open_prompt(key_event)?,
                }
            }
        }

        Ok(())
    }

    fn process_normal_mode(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event {
            KeyEvent {
                code: KeyCode::Tab,
                ..
            } => {
                let pane = self.active_pane_mut();
                // Insert 4 spaces (or you can use a real tab character '\t')
                for _ in 0..4 {
                    let command = EditCommand::InsertChar {
                        row: pane.cursor.y,
                        col: pane.cursor.x,
                        ch: ' ',
                    };
                    pane.execute_command(command);
                    pane.cursor.x += 1;
                }
                self.message = None;
            }
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if self.active_pane().modified && !self.quit_warning_shown {
                    self.message = Some("File modified! Press Ctrl-Q again to quit".to_string());
                    self.quit_warning_shown = true;
                } else {
                    self.should_quit = true;
                }
            }
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.save_file()?;
            }
            KeyEvent {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.mode = EditorMode::OpenPrompt;
                self.input_buffer.clear();
                self.message = Some("Open file: ".to_string());
                self.needs_full_redraw = true;
            }
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.start_search();
            }
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.find_next();
            }
            KeyEvent {
                code: KeyCode::Char('z'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.active_pane_mut().undo();
                self.needs_full_redraw = true;
            }
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.active_pane_mut().redo();
                self.needs_full_redraw = true;
            }
            KeyEvent {
                code: KeyCode::Char('\\'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.split_horizontal();
            }
            KeyEvent {
                code: KeyCode::Char('/'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.split_vertical();
            }
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.next_pane();
            }
            KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.close_split();
            }
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.toggle_line_numbers();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => {
                let pane = self.active_pane_mut();
                let command = EditCommand::InsertChar {
                    row: pane.cursor.y,
                    col: pane.cursor.x,
                    ch: c,
                };
                pane.execute_command(command);
                pane.cursor.x += 1;
                self.message = None;
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                let command = EditCommand::InsertNewline {
                    row: pane.cursor.y,
                    col: pane.cursor.x,
                };
                pane.execute_command(command);
                pane.cursor.y += 1;
                pane.cursor.x = 0;
                pane.adjust_scroll(visible_lines);
                self.message = None;
                self.needs_full_redraw = true; // Add this line
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                if pane.cursor.x > 0 {
                    if let Some(ch) = pane.buffer.get_line(pane.cursor.y).and_then(|line| {
                        if pane.cursor.x > 0 {
                            line.chars().nth(pane.cursor.x - 1)
                        } else {
                            None
                        }
                    }) {
                        let command = EditCommand::DeleteChar {
                            row: pane.cursor.y,
                            col: pane.cursor.x - 1,
                            ch,
                        };
                        pane.execute_command(command);
                        pane.cursor.x -= 1;
                    }
                } else if pane.cursor.y > 0 {
                let prev_line_len = pane
                    .buffer
                    .get_line(pane.cursor.y - 1)
                    .map(|l| l.len())
                    .unwrap_or(0);
                if let Some(deleted_line) = pane.buffer.get_line(pane.cursor.y).map(|l| l.clone())
                {
                    let command = EditCommand::DeleteNewline {
                        row: pane.cursor.y,
                        deleted_line,
                    };
                    pane.execute_command(command);
                    pane.cursor.y -= 1;
                    pane.cursor.x = prev_line_len;
                    pane.adjust_scroll(visible_lines);
                    self.needs_full_redraw = true; // Add this line
                }
            }
                self.message = None;
            }
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                if pane.cursor.x > 0 {
                    pane.cursor.x -= 1;
                } else if pane.cursor.y > 0 {
                    pane.cursor.y -= 1;
                    pane.cursor.x = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    pane.adjust_scroll(visible_lines);
                }
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                if let Some(line) = pane.buffer.get_line(pane.cursor.y) {
                    if pane.cursor.x < line.len() {
                        pane.cursor.x += 1;
                    } else if pane.cursor.y < pane.buffer.line_count() - 1 {
                        pane.cursor.y += 1;
                        pane.cursor.x = 0;
                        pane.adjust_scroll(visible_lines);
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                if pane.cursor.y > 0 {
                    pane.cursor.y -= 1;
                    let line_len = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    if pane.cursor.x > line_len {
                        pane.cursor.x = line_len;
                    }
                    pane.adjust_scroll(visible_lines);
                }
            }
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                if pane.cursor.y < pane.buffer.line_count() - 1 {
                    pane.cursor.y += 1;
                    let line_len = pane.buffer.get_line(pane.cursor.y).map(|l| l.len()).unwrap_or(0);
                    if pane.cursor.x > line_len {
                        pane.cursor.x = line_len;
                    }
                    pane.adjust_scroll(visible_lines);
                }
            }
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                self.active_pane_mut().cursor.x = 0;
            }
            KeyEvent {
                code: KeyCode::End,
                ..
            } => {
                let pane = self.active_pane_mut();
                if let Some(line) = pane.buffer.get_line(pane.cursor.y) {
                    pane.cursor.x = line.len();
                }
            }
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                pane.cursor.y = pane.cursor.y.saturating_sub(visible_lines);
                pane.adjust_scroll(visible_lines);
            }
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => {
                let (_, height) = terminal::size()?;
                let visible_lines = self.calculate_visible_lines(height);
                let pane = self.active_pane_mut();
                pane.cursor.y = (pane.cursor.y + visible_lines).min(pane.buffer.line_count() - 1);
                pane.adjust_scroll(visible_lines);
            }
            _ => {}
        }
        Ok(())
    }

    fn process_search_mode(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                self.perform_search();
                self.mode = EditorMode::Normal;
                self.needs_full_redraw = true;
            }
            KeyCode::Esc => {
                self.mode = EditorMode::Normal;
                self.message = Some("Search cancelled".to_string());
                self.needs_full_redraw = true;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn process_save_prompt(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    if let Err(e) = self.save_file_as(self.input_buffer.clone()) {
                        self.message = Some(format!("Error saving: {}", e));
                    }
                }
                self.mode = EditorMode::Normal;
                self.needs_full_redraw = true;
            }
            KeyCode::Esc => {
                self.mode = EditorMode::Normal;
                self.message = Some("Save cancelled".to_string());
                self.needs_full_redraw = true;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn process_open_prompt(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    if let Err(e) = self.open_file(self.input_buffer.clone()) {
                        self.message = Some(format!("Error opening: {}", e));
                    }
                }
                self.mode = EditorMode::Normal;
                self.needs_full_redraw = true;
            }
            KeyCode::Esc => {
                self.mode = EditorMode::Normal;
                self.message = Some("Open cancelled".to_string());
                self.needs_full_redraw = true;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }
}

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