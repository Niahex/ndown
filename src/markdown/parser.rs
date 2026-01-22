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

pub fn detect_list_item(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with("- ") || trimmed.starts_with("* ")
}

pub fn detect_numbered_list(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) 
        && trimmed.contains(". ")
}
