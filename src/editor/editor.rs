use super::{EditorMode, SplitMode, Tab};
use crate::config::{Config, Theme};
use crate::file_browser::FileBrowser;
use crate::pane::Pane;
use arboard::Clipboard;
use crossterm::{event::{EnableMouseCapture, DisableMouseCapture}, execute, terminal};
use std::io;

pub struct Editor {
    // Tabs - each tab has its own split state
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab: usize,
    pub(crate) should_quit: bool,
    pub(crate) mode: EditorMode,
    pub(crate) message: Option<String>,
    pub(crate) input_buffer: String,
    pub(crate) quit_warning_shown: bool,
    pub(crate) needs_full_redraw: bool,
    pub(crate) show_line_numbers: bool,
    pub(crate) clipboard: Option<Clipboard>,
    pub(crate) prev_cursor_y: Option<usize>,
    // Palette state
    pub(crate) palette_selected: usize,
    // Tab completion state
    pub(crate) tab_completion_index: usize,
    pub(crate) tab_completion_base: String,
    // File browser
    pub(crate) file_browser: FileBrowser,
    // Configuration
    pub(crate) config: Config,
    pub(crate) theme: Theme,
}

impl Editor {
    pub fn new() -> Self {
        let config = Config::load();
        let theme = config.get_theme();
        let show_line_numbers = config.editor.show_line_numbers;

        Self {
            tabs: vec![Tab::new()],
            active_tab: 0,
            should_quit: false,
            mode: EditorMode::Normal,
            message: None,
            input_buffer: String::new(),
            quit_warning_shown: false,
            needs_full_redraw: true,
            show_line_numbers,
            clipboard: Clipboard::new().ok(),
            prev_cursor_y: None,
            palette_selected: 0,
            tab_completion_index: 0,
            tab_completion_base: String::new(),
            file_browser: FileBrowser::new(),
            config,
            theme,
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    /// Get the current tab mutably
    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /// Get the currently focused pane (respects per-tab split focus)
    pub fn active_pane(&self) -> &Pane {
        self.current_tab().active_pane()
    }

    /// Get the currently focused pane mutably
    pub fn active_pane_mut(&mut self) -> &mut Pane {
        self.current_tab_mut().active_pane_mut()
    }

    /// Get the index of the currently focused tab
    pub fn focused_tab_index(&self) -> usize {
        self.active_tab
    }

    /// Get the current tab's split mode
    pub fn split_mode(&self) -> SplitMode {
        self.current_tab().split_mode
    }

    /// Get the current tab's split focus
    pub fn split_focus(&self) -> usize {
        self.current_tab().split_focus
    }

    // === Tab Management ===

    /// Create a new empty tab
    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.active_tab = self.tabs.len() - 1;
        self.needs_full_redraw = true;
        self.message = Some(format!("New tab {}", self.tabs.len()));
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
            self.needs_full_redraw = true;
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.needs_full_redraw = true;
        }
    }

    /// Switch to specific tab by index (1-based for user, 0-based internal)
    pub fn goto_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
            self.needs_full_redraw = true;
        }
    }

    /// Close current tab
    pub fn close_tab(&mut self) -> bool {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);

            // Adjust active_tab
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }

            self.needs_full_redraw = true;
            true
        } else {
            false
        }
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    // === Split Management (per-tab) ===

    pub fn split_horizontal(&mut self) {
        self.current_tab_mut().split_horizontal();
        self.needs_full_redraw = true;
    }

    pub fn split_vertical(&mut self) {
        self.current_tab_mut().split_vertical();
        self.needs_full_redraw = true;
    }

    pub fn close_split(&mut self) {
        self.current_tab_mut().close_split();
        self.needs_full_redraw = true;
    }

    /// Switch focus between split panes in current tab
    pub fn next_pane(&mut self) {
        let tab = self.current_tab_mut();
        if tab.is_split() {
            tab.toggle_pane_focus();
            self.needs_full_redraw = true;
        } else if self.tabs.len() > 1 {
            // In non-split mode, cycle through tabs
            self.next_tab();
        }
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.needs_full_redraw = true;
    }

    pub fn get_line_number_width(&self) -> usize {
        if !self.show_line_numbers {
            return 0;
        }
        let max_line = self.active_pane().buffer.line_count();
        format!("{}", max_line).len() + 1
    }

    pub fn calculate_visible_lines(&self, height: u16) -> usize {
        // Account for: tab bar (1) + status bar (1) + message line (1) = 3 lines overhead
        // In split mode, also subtract 1 for the divider
        let tab_bar_height = if self.tabs.len() > 1 { 1 } else { 0 };
        match self.split_mode() {
            SplitMode::None => (height as usize).saturating_sub(2 + tab_bar_height),
            SplitMode::Horizontal => ((height as usize).saturating_sub(3 + tab_bar_height)) / 2,
            SplitMode::Vertical => (height as usize).saturating_sub(2 + tab_bar_height),
        }
    }

    /// Check if tab bar should be shown
    pub fn show_tab_bar(&self) -> bool {
        self.tabs.len() > 1
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;

        let result = self.main_loop(&mut stdout);

        execute!(stdout, DisableMouseCapture, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn main_loop(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        loop {
            crate::ui::refresh_screen(self, stdout)?;

            if self.should_quit {
                break;
            }

            crate::input::process_keypress(self)?;
        }
        Ok(())
    }
}
