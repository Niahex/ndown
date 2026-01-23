use crate::markdown::inline::{parse_inline_formatting, InlineFormat};

pub struct FormattingManager;

impl FormattingManager {
    pub fn toggle_bold(text: &str, cursor_pos: usize) -> (String, usize) {
        Self::toggle_format(text, cursor_pos, "**", InlineFormat::Bold)
    }
    
    pub fn toggle_italic(text: &str, cursor_pos: usize) -> (String, usize) {
        Self::toggle_format(text, cursor_pos, "*", InlineFormat::Italic)
    }
    
    pub fn toggle_code(text: &str, cursor_pos: usize) -> (String, usize) {
        Self::toggle_format(text, cursor_pos, "`", InlineFormat::Code)
    }
    
    pub fn wrap_selection(text: &str, start: usize, end: usize, marker: &str) -> String {
        if start >= end || end > text.len() {
            return text.to_string();
        }
        
        let before = &text[..start];
        let selection = &text[start..end];
        let after = &text[end..];
        
        format!("{}{}{}{}{}", before, marker, selection, marker, after)
    }
    
    fn toggle_format(text: &str, cursor_pos: usize, marker: &str, format_type: InlineFormat) -> (String, usize) {
        let spans = parse_inline_formatting(text);
        
        // Check if cursor is inside existing span
        for span in &spans {
            if span.format == format_type && cursor_pos >= span.range.start && cursor_pos <= span.range.end {
                // Remove formatting
                let before = &text[..span.range.start];
                let content = match format_type {
                    InlineFormat::Bold => &text[span.range.start + 2..span.range.end - 2],
                    InlineFormat::Italic => &text[span.range.start + 1..span.range.end - 1],
                    InlineFormat::Code => &text[span.range.start + 1..span.range.end - 1],
                    InlineFormat::Link { ref text, .. } => text,
                    InlineFormat::WikiLink { ref text } => text,
                    InlineFormat::Image { ref alt, .. } => alt,
                };
                let after = &text[span.range.end..];
                
                let new_text = format!("{}{}{}", before, content, after);
                let new_cursor = cursor_pos - marker.len();
                return (new_text, new_cursor);
            }
        }
        
        // Add new formatting
        let before = &text[..cursor_pos];
        let after = &text[cursor_pos..];
        let new_text = format!("{}{}{}{}", before, marker, marker, after);
        let new_cursor = cursor_pos + marker.len();
        
        (new_text, new_cursor)
    }
}
