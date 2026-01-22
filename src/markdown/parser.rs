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
            let number_str = &trimmed[..dot_pos];
            return Some(ListInfo {
                list_type: ListType::Ordered,
                indent_level: indent_count / 2,
                content: list_content.to_string(),
                marker: number_str.chars().next().unwrap_or('1'),
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

pub fn detect_blockquote(content: &str) -> Option<BlockQuoteInfo> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with('>') {
        return None;
    }
    
    let mut level = 0;
    let mut chars = trimmed.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '>' {
            level += 1;
        } else if ch == ' ' {
            break;
        } else {
            break;
        }
    }
    
    if level > 0 {
        let content_start = trimmed.find(|c: char| c != '>' && c != ' ').unwrap_or(trimmed.len());
        let content = trimmed[content_start..].to_string();
        Some(BlockQuoteInfo { level, content })
    } else {
        None
    }
}

pub fn detect_checkbox(content: &str) -> Option<CheckboxInfo> {
    let trimmed = content.trim_start();
    
    if trimmed.starts_with("- [ ] ") {
        Some(CheckboxInfo {
            checked: false,
            content: trimmed[6..].to_string(),
        })
    } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        Some(CheckboxInfo {
            checked: true,
            content: trimmed[6..].to_string(),
        })
    } else {
        None
    }
}

pub fn detect_divider(content: &str) -> Option<DividerType> {
    let trimmed = content.trim();
    
    if trimmed == "---" || trimmed.chars().all(|c| c == '-') && trimmed.len() >= 3 {
        Some(DividerType::Simple)
    } else if trimmed == "===" || trimmed.chars().all(|c| c == '=') && trimmed.len() >= 3 {
        Some(DividerType::Double)
    } else if trimmed == "-.-" || (trimmed.chars().enumerate().all(|(i, c)| {
        if i % 2 == 0 { c == '-' } else { c == '.' }
    }) && trimmed.len() >= 3) {
        Some(DividerType::Dotted)
    } else {
        None
    }
}

pub fn detect_code_block(content: &str) -> Option<CodeBlockInfo> {
    let trimmed = content.trim_start();
    
    if trimmed.starts_with("```") {
        let first_line = trimmed.lines().next().unwrap_or("");
        let language = if first_line.len() > 3 {
            Some(first_line[3..].trim().to_string())
        } else {
            None
        };
        
        let content = trimmed.lines().skip(1).collect::<Vec<_>>().join("\n");
        
        Some(CodeBlockInfo { language, content })
    } else {
        None
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct BlockQuoteInfo {
    pub level: usize,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckboxInfo {
    pub checked: bool,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DividerType {
    Simple,    // ---
    Double,    // ===
    Dotted,    // -.-
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlockInfo {
    pub language: Option<String>,
    pub content: String,
}
