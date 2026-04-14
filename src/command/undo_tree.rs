use crate::buffer::TextBuffer;
use crate::command::EditCommand;

/// A node in the undo tree representing a single edit
#[derive(Clone, Debug)]
pub struct UndoNode {
    pub command: EditCommand,
    pub children: Vec<usize>,  // Indices of child nodes (branches)
    pub parent: Option<usize>, // Index of parent node
}

/// A named checkpoint in the undo history
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub name: String,
    pub node_id: Option<usize>, // None means "initial state" (before any edits)
    pub created_at: std::time::Instant,
}

/// Tree-based undo history with branching support
#[derive(Clone)]
pub struct UndoTree {
    nodes: Vec<UndoNode>,
    current: Option<usize>,       // Current position in tree (None = root/initial state)
    checkpoints: Vec<Checkpoint>,
}

impl UndoTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current: None,
            checkpoints: Vec::new(),
        }
    }

    /// Execute a command and add it to the tree
    pub fn execute(&mut self, command: EditCommand, buffer: &mut TextBuffer) {
        command.redo(buffer);

        let new_node = UndoNode {
            command,
            children: Vec::new(),
            parent: self.current,
        };

        let new_index = self.nodes.len();
        self.nodes.push(new_node);

        // Link parent to this new child
        if let Some(parent_idx) = self.current {
            self.nodes[parent_idx].children.push(new_index);
        }

        self.current = Some(new_index);
    }

    /// Undo the current command, moving up the tree
    pub fn undo(&mut self, buffer: &mut TextBuffer) -> bool {
        if let Some(current_idx) = self.current {
            self.nodes[current_idx].command.undo(buffer);
            self.current = self.nodes[current_idx].parent;
            true
        } else {
            false
        }
    }

    /// Redo along the most recent branch (last child)
    pub fn redo(&mut self, buffer: &mut TextBuffer) -> bool {
        let next = if let Some(current_idx) = self.current {
            self.nodes[current_idx].children.last().copied()
        } else if !self.nodes.is_empty() {
            // At root, find first node with no parent
            self.nodes.iter().position(|n| n.parent.is_none())
        } else {
            None
        };

        if let Some(next_idx) = next {
            self.nodes[next_idx].command.redo(buffer);
            self.current = Some(next_idx);
            true
        } else {
            false
        }
    }

    /// Redo along a specific branch
    pub fn redo_branch(&mut self, branch: usize, buffer: &mut TextBuffer) -> bool {
        let children = if let Some(current_idx) = self.current {
            self.nodes[current_idx].children.clone()
        } else {
            // At root, find all nodes with no parent
            self.nodes.iter()
                .enumerate()
                .filter(|(_, n)| n.parent.is_none())
                .map(|(i, _)| i)
                .collect()
        };

        if branch < children.len() {
            let next_idx = children[branch];
            self.nodes[next_idx].command.redo(buffer);
            self.current = Some(next_idx);
            true
        } else {
            false
        }
    }

    /// Create a named checkpoint at the current position
    /// Returns error if checkpoint with same name already exists
    pub fn create_checkpoint(&mut self, name: String) -> Result<(), String> {
        // Check for duplicate
        if self.checkpoints.iter().any(|cp| cp.name == name) {
            return Err(format!("Checkpoint '{}' already exists. Use :delete-checkpoint {} first.", name, name));
        }

        self.checkpoints.push(Checkpoint {
            name,
            node_id: self.current,
            created_at: std::time::Instant::now(),
        });
        Ok(())
    }

    /// List all checkpoints
    pub fn list_checkpoints(&self) -> Vec<&Checkpoint> {
        self.checkpoints.iter().collect()
    }

    /// Rewind to a named checkpoint
    pub fn rewind_to_checkpoint(&mut self, name: &str, buffer: &mut TextBuffer) -> Result<(), String> {
        let checkpoint = self.checkpoints
            .iter()
            .find(|cp| cp.name == name)
            .ok_or_else(|| format!("Checkpoint '{}' not found", name))?;

        let target_id = checkpoint.node_id;
        self.rewind_to_node(target_id, buffer);
        Ok(())
    }

    /// Rewind to a specific node (or root if None)
    fn rewind_to_node(&mut self, target: Option<usize>, buffer: &mut TextBuffer) {
        // Build path from root to current
        let current_path = self.path_from_root(self.current);
        // Build path from root to target
        let target_path = self.path_from_root(target);

        // Find common ancestor
        let mut common_len = 0;
        for (a, b) in current_path.iter().zip(target_path.iter()) {
            if a == b {
                common_len += 1;
            } else {
                break;
            }
        }

        // Undo from current back to common ancestor
        for &node_idx in current_path[common_len..].iter().rev() {
            self.nodes[node_idx].command.undo(buffer);
        }

        // Redo from common ancestor to target
        for &node_idx in &target_path[common_len..] {
            self.nodes[node_idx].command.redo(buffer);
        }

        self.current = target;
    }

    /// Get path from root to a node
    fn path_from_root(&self, node: Option<usize>) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = node;

        while let Some(idx) = current {
            path.push(idx);
            current = self.nodes[idx].parent;
        }

        path.reverse();
        path
    }

    /// Get number of branches at current position
    pub fn branch_count(&self) -> usize {
        if let Some(current_idx) = self.current {
            self.nodes[current_idx].children.len()
        } else {
            // At root, count nodes with no parent
            self.nodes.iter().filter(|n| n.parent.is_none()).count()
        }
    }

    /// Check if there are any undos available
    pub fn can_undo(&self) -> bool {
        self.current.is_some()
    }

    /// Check if there are any redos available
    pub fn can_redo(&self) -> bool {
        if let Some(current_idx) = self.current {
            !self.nodes[current_idx].children.is_empty()
        } else {
            !self.nodes.is_empty()
        }
    }

    /// Get current depth in the tree
    pub fn depth(&self) -> usize {
        self.path_from_root(self.current).len()
    }

    /// Get total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.current = None;
        self.checkpoints.clear();
    }

    /// Get info about current state for status display
    pub fn status_info(&self) -> String {
        let depth = self.depth();
        let total = self.node_count();
        let branches = self.branch_count();
        let checkpoints = self.checkpoints.len();

        if branches > 1 {
            format!("undo:{}/{} ({} branches, {} cp)", depth, total, branches, checkpoints)
        } else if checkpoints > 0 {
            format!("undo:{}/{} ({} cp)", depth, total, checkpoints)
        } else {
            format!("undo:{}/{}", depth, total)
        }
    }

    /// Delete a checkpoint by name
    pub fn delete_checkpoint(&mut self, name: &str) -> bool {
        let len_before = self.checkpoints.len();
        self.checkpoints.retain(|cp| cp.name != name);
        self.checkpoints.len() < len_before
    }
}

impl Default for UndoTree {
    fn default() -> Self {
        Self::new()
    }
}
