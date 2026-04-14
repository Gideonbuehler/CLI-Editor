use super::SplitMode;
use crate::pane::Pane;

/// A tab contains one or two panes with its own split state
pub struct Tab {
    /// Primary pane (always exists)
    pub primary: Pane,
    /// Secondary pane (exists when split)
    pub secondary: Option<Pane>,
    /// Current split mode for this tab
    pub split_mode: SplitMode,
    /// Which pane is focused: 0 = primary, 1 = secondary
    pub split_focus: usize,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            primary: Pane::new(),
            secondary: None,
            split_mode: SplitMode::None,
            split_focus: 0,
        }
    }

    /// Get the currently focused pane
    pub fn active_pane(&self) -> &Pane {
        if self.split_focus == 1 && self.secondary.is_some() {
            self.secondary.as_ref().unwrap()
        } else {
            &self.primary
        }
    }

    /// Get the currently focused pane mutably
    pub fn active_pane_mut(&mut self) -> &mut Pane {
        if self.split_focus == 1 && self.secondary.is_some() {
            self.secondary.as_mut().unwrap()
        } else {
            &mut self.primary
        }
    }

    /// Check if this tab has any unsaved changes
    pub fn is_modified(&self) -> bool {
        self.primary.modified || self.secondary.as_ref().map(|p| p.modified).unwrap_or(false)
    }

    /// Get the display name for this tab (uses primary pane's file)
    pub fn display_name(&self) -> String {
        if let Some(ref path) = self.primary.current_file {
            std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("untitled")
                .to_string()
        } else {
            "untitled".to_string()
        }
    }

    /// Create a horizontal split
    pub fn split_horizontal(&mut self) {
        if self.split_mode == SplitMode::None {
            self.secondary = Some(Pane::new());
            self.split_mode = SplitMode::Horizontal;
            self.split_focus = 0;
        }
    }

    /// Create a vertical split
    pub fn split_vertical(&mut self) {
        if self.split_mode == SplitMode::None {
            self.secondary = Some(Pane::new());
            self.split_mode = SplitMode::Vertical;
            self.split_focus = 0;
        }
    }

    /// Close the split, keeping the focused pane
    pub fn close_split(&mut self) {
        if self.split_mode != SplitMode::None {
            // If secondary pane was focused and exists, make it the primary
            if self.split_focus == 1 {
                if let Some(secondary) = self.secondary.take() {
                    self.primary = secondary;
                }
            } else {
                self.secondary = None;
            }
            self.split_mode = SplitMode::None;
            self.split_focus = 0;
        }
    }

    /// Switch focus between panes in this tab
    pub fn toggle_pane_focus(&mut self) {
        if self.split_mode != SplitMode::None && self.secondary.is_some() {
            self.split_focus = 1 - self.split_focus;
        }
    }

    /// Check if this tab is split
    pub fn is_split(&self) -> bool {
        self.split_mode != SplitMode::None
    }
}
