#[derive(Clone, Debug)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub char_index: usize,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            char_index: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextSelection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

impl TextSelection {
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }
    
    pub fn is_empty(&self) -> bool {
        self.start.char_index == self.end.char_index
    }
    
    pub fn get_range(&self) -> (usize, usize) {
        let start = self.start.char_index.min(self.end.char_index);
        let end = self.start.char_index.max(self.end.char_index);
        (start, end)
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub text: String,
    pub cursor: CursorPosition,
    pub selection: Option<TextSelection>,
}

impl HistoryEntry {
    pub fn new(text: String, cursor: CursorPosition, selection: Option<TextSelection>) -> Self {
        Self { text, cursor, selection }
    }
}
