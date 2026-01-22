use crate::markdown::parser::{ListInfo, ListType};

pub struct IndentationManager {
    pub spaces_per_level: usize,
}

impl IndentationManager {
    pub fn new() -> Self {
        Self {
            spaces_per_level: 2,
        }
    }
    
    pub fn increase_indent(&self, text: &str) -> String {
        let indent = " ".repeat(self.spaces_per_level);
        format!("{}{}", indent, text)
    }
    
    pub fn decrease_indent(&self, text: &str) -> String {
        if text.starts_with(&" ".repeat(self.spaces_per_level)) {
            text[self.spaces_per_level..].to_string()
        } else {
            text.to_string()
        }
    }
    
    pub fn get_indent_level(&self, text: &str) -> usize {
        let leading_spaces = text.len() - text.trim_start().len();
        leading_spaces / self.spaces_per_level
    }
    
    pub fn create_list_item(&self, list_type: ListType, indent_level: usize, content: &str) -> String {
        let indent = " ".repeat(indent_level * self.spaces_per_level);
        match list_type {
            ListType::Unordered => format!("{}• {}", indent, content),
            ListType::Ordered => format!("{}1. {}", indent, content),
        }
    }
    
    pub fn continue_list_item(&self, list_info: &ListInfo) -> String {
        let indent = " ".repeat(list_info.indent_level * self.spaces_per_level);
        match list_info.list_type {
            ListType::Unordered => format!("{}{} ", indent, list_info.marker),
            ListType::Ordered => format!("{}1. ", indent),
        }
    }
}

impl Default for IndentationManager {
    fn default() -> Self {
        Self::new()
    }
}
