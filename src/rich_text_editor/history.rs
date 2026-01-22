use crate::rich_text_editor::types::*;

pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    current_index: usize,
    max_entries: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            max_entries: 50,
        }
    }
}

impl HistoryManager {
    pub fn push(&mut self, entry: HistoryEntry) {
        // Remove entries after current index if we're not at the end
        if self.current_index < self.entries.len() {
            self.entries.truncate(self.current_index);
        }
        
        // Add new entry
        self.entries.push(entry);
        
        // Maintain max entries limit
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        } else {
            self.current_index += 1;
        }
    }
    
    pub fn undo(&mut self) -> Option<&HistoryEntry> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<&HistoryEntry> {
        if self.current_index < self.entries.len() {
            let entry = self.entries.get(self.current_index);
            self.current_index += 1;
            entry
        } else {
            None
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.current_index < self.entries.len()
    }
}
