pub fn detect_heading_level(content: &str) -> Option<u8> {
    let trimmed = content.trim_start();
    if trimmed.starts_with("### ") {
        Some(3)
    } else if trimmed.starts_with("## ") {
        Some(2)
    } else if trimmed.starts_with("# ") {
        Some(1)
    } else {
        None
    }
}

pub fn detect_list_item(content: &str) -> Option<ListInfo> {
    let indent_count = content.len() - content.trim_start().len();
    let trimmed = content.trim_start();
    
    // Unordered lists
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let list_content = &trimmed[2..];
        return Some(ListInfo {
            list_type: ListType::Unordered,
            indent_level: indent_count / 2, // 2 spaces per level
            content: list_content.to_string(),
            marker: trimmed.chars().next().unwrap(),
        });
    }
    
    // Ordered lists (1. 2. etc.)
    if let Some(dot_pos) = trimmed.find(". ") {
        if dot_pos > 0 && trimmed[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
            let list_content = &trimmed[dot_pos + 2..];
            return Some(ListInfo {
                list_type: ListType::Ordered,
                indent_level: indent_count / 2,
                content: list_content.to_string(),
                marker: '1', // Default marker for ordered lists
            });
        }
    }
    
    None
}

pub fn is_list_item(content: &str) -> bool {
    detect_list_item(content).is_some()
}

pub fn detect_numbered_list(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) 
        && trimmed.contains(". ")
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListInfo {
    pub list_type: ListType,
    pub indent_level: usize,
    pub content: String,
    pub marker: char,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    Unordered,
    Ordered,
}
