use crate::rich_text_input::types::*;

pub struct CursorManager {
    pub position: CursorPosition,
    pub selection: Option<TextSelection>,
    pub drag_start: Option<CursorPosition>,
}

impl Default for CursorManager {
    fn default() -> Self {
        Self {
            position: CursorPosition::default(),
            selection: None,
            drag_start: None,
        }
    }
}

impl CursorManager {
    pub fn char_index_to_position(&self, text: &str, char_index: usize) -> CursorPosition {
        let mut line = 0;
        let mut column = 0;
        let mut current_index = 0;
        
        for ch in text.chars() {
            if current_index >= char_index {
                break;
            }
            
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            current_index += 1;
        }
        
        CursorPosition { line, column, char_index }
    }
    
    pub fn position_to_char_index(&self, text: &str, line: usize, column: usize) -> usize {
        let mut current_line = 0;
        let mut current_column = 0;
        let mut char_index = 0;
        
        for ch in text.chars() {
            if current_line == line && current_column == column {
                break;
            }
            
            if ch == '\n' {
                current_line += 1;
                current_column = 0;
            } else {
                current_column += 1;
            }
            char_index += 1;
        }
        
        char_index
    }
    
    pub fn get_selected_text(&self, text: &str) -> Option<String> {
        if let Some(selection) = &self.selection {
            let (start, end) = selection.get_range();
            Some(text.chars().skip(start).take(end - start).collect())
        } else {
            None
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
    
    pub fn start_selection(&mut self, start_pos: CursorPosition) {
        self.drag_start = Some(start_pos.clone());
        self.selection = Some(TextSelection::new(start_pos.clone(), start_pos));
    }
    
    pub fn update_selection(&mut self, end_pos: CursorPosition) {
        if let Some(start) = &self.drag_start {
            self.selection = Some(TextSelection::new(start.clone(), end_pos));
        }
    }
    
    pub fn end_selection(&mut self) {
        self.drag_start = None;
    }
}
