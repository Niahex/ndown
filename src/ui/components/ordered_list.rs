use crate::markdown::parser::ListType;

pub struct OrderedListManager {
    pub max_indent_level: usize,
}

impl OrderedListManager {
    pub fn new() -> Self {
        Self {
            max_indent_level: 1, // Max 1 tab (2 levels: 1. and 1.1)
        }
    }
    
    pub fn format_number(&self, level: usize, main_num: u32, sub_num: Option<u32>) -> String {
        match level {
            0 => format!("{}.", main_num),
            1 => format!("{}.{}.", main_num, sub_num.unwrap_or(1)),
            _ => format!("{}.", main_num), // Fallback to main level
        }
    }
    
    pub fn increase_indent(&self, content: &str, current_level: usize, prev_main_number: Option<u32>) -> Option<String> {
        if current_level >= self.max_indent_level {
            return None; // Max indent reached
        }
        
        let indent = "  ".repeat(current_level + 1);
        
        // Extract the text content after the number
        let trimmed = content.trim_start();
        let text_content = if let Some(dot_pos) = trimmed.find(". ") {
            &trimmed[dot_pos + 2..]
        } else {
            ""
        };
        
        // Use previous main number, or extract from current content
        let main_num = prev_main_number.unwrap_or_else(|| {
            self.extract_main_number(content).unwrap_or(1)
        });
        
        Some(format!("{}{}.1. {}", indent, main_num, text_content))
    }
    
    pub fn decrease_indent(&self, content: &str) -> Option<String> {
        let trimmed = content.trim_start();
        let indent_level = (content.len() - trimmed.len()) / 2;
        
        if indent_level > 0 {
            let new_indent = "  ".repeat(indent_level - 1);
            
            // Extract the text content after the number
            let text_content = if let Some(dot_pos) = trimmed.find(". ") {
                &trimmed[dot_pos + 2..]
            } else {
                ""
            };
            
            // If it's a sub-item (1.1.), convert back to next main level
            let dot_count = trimmed.chars().filter(|&c| c == '.').count();
            
            if dot_count > 1 {
                // Extract main number and increment for next main item
                if let Some(main_num) = self.extract_main_number(content) {
                    let result = format!("{}{}. {}", new_indent, main_num + 1, text_content);
                    return Some(result);
                }
            }
            
            // Otherwise just decrease indent
            let result = format!("{}{}", new_indent, trimmed);
            return Some(result);
        }
        
        None
    }
    
    fn extract_main_number(&self, content: &str) -> Option<u32> {
        let trimmed = content.trim_start();
        if let Some(first_dot) = trimmed.find('.') {
            if let Ok(num) = trimmed[..first_dot].parse::<u32>() {
                return Some(num);
            }
        }
        None
    }
    
    pub fn renumber_list_sequence(&self, blocks: &mut [crate::block::Block], start_index: usize) {
        let mut main_counter = 1u32;
        let mut sub_counter = 1u32;
        let mut current_main = 1u32;
        
        for i in start_index..blocks.len() {
            if let Some(list_info) = crate::markdown::parser::detect_list_item(&blocks[i].content) {
                if list_info.list_type == ListType::Ordered {
                    let indent_spaces = "  ".repeat(list_info.indent_level);
                    let content_part = &list_info.content;
                    
                    match list_info.indent_level {
                        0 => {
                            // Main level (1., 2., 3.)
                            blocks[i].content = format!("{}{}. {}", indent_spaces, main_counter, content_part);
                            current_main = main_counter;
                            main_counter += 1;
                            sub_counter = 1; // Reset sub counter
                        }
                        1 => {
                            // Sub level (1.1., 1.2., 2.1.)
                            blocks[i].content = format!("{}{}.{}. {}", indent_spaces, current_main, sub_counter, content_part);
                            sub_counter += 1;
                        }
                        _ => {} // No deeper levels
                    }
                } else {
                    break; // Stop at non-ordered list
                }
            } else {
                break; // Stop at non-list item
            }
        }
    }
}

impl Default for OrderedListManager {
    fn default() -> Self {
        Self::new()
    }
}
