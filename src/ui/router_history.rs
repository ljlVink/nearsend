use gpui::{App, Global, SharedString};
use std::collections::VecDeque;

/// Router history entry
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub pathname: SharedString,
}

impl HistoryEntry {
    pub fn new(pathname: impl Into<SharedString>) -> Self {
        Self {
            pathname: pathname.into(),
        }
    }
}

/// Router history manager
/// Manages navigation history for back operations
pub struct RouterHistory {
    /// History stack (back stack)
    history: VecDeque<HistoryEntry>,
    /// Maximum history size
    max_size: usize,
    /// Current position in history
    current_index: usize,
}

impl RouterHistory {
    /// Creates a new RouterHistory with default settings
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            max_size: 50,
            current_index: 0,
        }
    }

    /// Pushes a new entry to history (called when navigating to a new page)
    pub fn push(&mut self, entry: HistoryEntry) {
        if self
            .history
            .back()
            .map(|last| last.pathname == entry.pathname)
            .unwrap_or(false)
        {
            self.current_index = self.history.len();
            return;
        }

        // Remove all entries after current position (forward history)
        while self.history.len() > self.current_index {
            self.history.pop_back();
        }

        // Add new entry
        self.history.push_back(entry);
        self.current_index = self.history.len();

        // Trim history if exceeds max size
        while self.history.len() > self.max_size {
            self.history.pop_front();
            self.current_index = self.current_index.saturating_sub(1);
        }
    }

    /// Goes back to previous entry
    /// Returns the entry to navigate to, or None if at root
    pub fn go_back(&mut self) -> Option<HistoryEntry> {
        if self.current_index > 1 {
            self.current_index -= 1;
            self.history.get(self.current_index - 1).cloned()
        } else {
            None
        }
    }

    /// Checks if can go back
    pub fn can_go_back(&self) -> bool {
        self.current_index > 1
    }

    /// Resets history to a single entry.
    pub fn reset(&mut self, entry: HistoryEntry) {
        self.history.clear();
        self.current_index = 0;
        self.push(entry);
    }
}

impl Default for RouterHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global router history state
pub struct RouterHistoryState {
    pub history: RouterHistory,
}

impl Global for RouterHistoryState {}

impl RouterHistoryState {
    /// Initialize router history with initial route
    pub fn init(cx: &mut App, initial_path: &str) {
        let mut history = RouterHistory::new();
        history.push(HistoryEntry::new(initial_path.to_string()));
        cx.set_global(Self { history });
    }

    /// Get mutable global router history
    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}
