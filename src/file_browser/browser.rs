use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct FileBrowser {
    /// Whether the sidebar is visible
    pub visible: bool,
    /// Width of the sidebar in characters
    pub width: u16,
    /// Root directory being browsed
    pub root: PathBuf,
    /// Flat list of visible entries (expanded view)
    pub entries: Vec<FileEntry>,
    /// Set of expanded directory paths
    pub expanded: HashSet<PathBuf>,
    /// Currently selected entry index
    pub cursor: usize,
    /// Scroll offset for long lists
    pub scroll_offset: usize,
}

impl FileBrowser {
    pub fn new() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut browser = Self {
            visible: false,
            width: 30,
            root: root.clone(),
            entries: Vec::new(),
            expanded: HashSet::new(),
            cursor: 0,
            scroll_offset: 0,
        };
        // Start with root expanded
        browser.expanded.insert(root);
        browser.refresh();
        browser
    }

    /// Toggle sidebar visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.refresh();
        }
    }

    /// Set the root directory
    pub fn set_root(&mut self, path: PathBuf) {
        self.root = path.clone();
        self.expanded.clear();
        self.expanded.insert(path);
        self.cursor = 0;
        self.scroll_offset = 0;
        self.refresh();
    }

    /// Refresh the file list
    pub fn refresh(&mut self) {
        self.entries.clear();
        self.build_tree(&self.root.clone(), 0);
    }

    /// Build the flat list of entries from the directory tree
    fn build_tree(&mut self, dir: &Path, depth: usize) {
        let mut entries: Vec<FileEntry> = Vec::new();

        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files (starting with .)
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                entries.push(FileEntry {
                    path: path.clone(),
                    name,
                    is_dir,
                    depth,
                });
            }
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        // Add entries and recurse into expanded directories
        for entry in entries {
            let path = entry.path.clone();
            let is_dir = entry.is_dir;
            self.entries.push(entry);

            if is_dir && self.expanded.contains(&path) {
                self.build_tree(&path, depth + 1);
            }
        }
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.adjust_scroll();
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self) {
        if self.cursor < self.entries.len().saturating_sub(1) {
            self.cursor += 1;
            self.adjust_scroll();
        }
    }

    /// Adjust scroll offset to keep cursor visible
    fn adjust_scroll(&mut self) {
        // This will be called with visible_height later
    }

    /// Adjust scroll to fit within visible area
    pub fn adjust_scroll_for_height(&mut self, visible_height: usize) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor - visible_height + 1;
        }
    }

    /// Toggle expansion of current directory
    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                let path = entry.path.clone();
                if self.expanded.contains(&path) {
                    self.expanded.remove(&path);
                } else {
                    self.expanded.insert(path);
                }
                self.refresh();
            }
        }
    }

    /// Expand current directory (if collapsed)
    pub fn expand(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir && !self.expanded.contains(&entry.path) {
                self.expanded.insert(entry.path.clone());
                self.refresh();
            }
        }
    }

    /// Collapse current directory (if expanded) or go to parent
    pub fn collapse(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir && self.expanded.contains(&entry.path) {
                self.expanded.remove(&entry.path);
                self.refresh();
            } else {
                // Go to parent directory
                self.go_to_parent();
            }
        }
    }

    /// Move cursor to parent directory entry
    fn go_to_parent(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if let Some(parent) = entry.path.parent() {
                // Find the parent in the entries list
                for (i, e) in self.entries.iter().enumerate() {
                    if e.path == parent {
                        self.cursor = i;
                        return;
                    }
                }
            }
        }
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// Get the path of the currently selected file (if it's a file)
    pub fn selected_file_path(&self) -> Option<PathBuf> {
        self.selected_entry()
            .filter(|e| !e.is_dir)
            .map(|e| e.path.clone())
    }

    /// Navigate up to parent directory (change root)
    pub fn go_up_directory(&mut self) {
        if let Some(parent) = self.root.parent() {
            let parent = parent.to_path_buf();
            self.set_root(parent);
        }
    }
}
