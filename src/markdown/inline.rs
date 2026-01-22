use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum InlineFormat {
    Bold,
    Italic,
    Code,
    Link { text: String, url: String },
    WikiLink { text: String },
    Image { alt: String, url: String },
}

#[derive(Debug, Clone)]
pub struct InlineSpan {
    pub range: Range<usize>,
    pub format: InlineFormat,
}

pub fn parse_inline_formatting(text: &str) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        // Check for ![alt](url) - Images
        if i + 1 < chars.len() && chars[i] == '!' && chars[i + 1] == '[' {
            if let Some((end, alt, url)) = find_image_link(&chars, i) {
                spans.push(InlineSpan {
                    range: i..end,
                    format: InlineFormat::Image { alt, url },
                });
                i = end;
                continue;
            }
        }
        
        // Check for [[wiki]] - Wiki links
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some((end, text)) = find_wiki_link(&chars, i) {
                spans.push(InlineSpan {
                    range: i..end,
                    format: InlineFormat::WikiLink { text },
                });
                i = end;
                continue;
            }
        }
        
        // Check for [text](url) - Links
        if chars[i] == '[' {
            if let Some((end, text, url)) = find_markdown_link(&chars, i) {
                spans.push(InlineSpan {
                    range: i..end,
                    format: InlineFormat::Link { text, url },
                });
                i = end;
                continue;
            }
        }
        
        // Check for **bold**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing_pattern(&chars, i + 2, "**") {
                spans.push(InlineSpan {
                    range: i..end + 2,
                    format: InlineFormat::Bold,
                });
                i = end + 2;
                continue;
            }
        }
        
        // Check for *italic*
        if chars[i] == '*' {
            if let Some(end) = find_closing_char(&chars, i + 1, '*') {
                spans.push(InlineSpan {
                    range: i..end + 1,
                    format: InlineFormat::Italic,
                });
                i = end + 1;
                continue;
            }
        }
        
        // Check for `code`
        if chars[i] == '`' {
            if let Some(end) = find_closing_char(&chars, i + 1, '`') {
                spans.push(InlineSpan {
                    range: i..end + 1,
                    format: InlineFormat::Code,
                });
                i = end + 1;
                continue;
            }
        }
        
        i += 1;
    }
    
    spans
}

fn find_closing_pattern(chars: &[char], start: usize, pattern: &str) -> Option<usize> {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let pattern_len = pattern_chars.len();
    
    for i in start..chars.len() - pattern_len + 1 {
        if chars[i..i + pattern_len] == pattern_chars {
            return Some(i);
        }
    }
    None
}

fn find_closing_char(chars: &[char], start: usize, closing: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == closing {
            return Some(i);
        }
    }
    None
}

fn find_image_link(chars: &[char], start: usize) -> Option<(usize, String, String)> {
    // ![alt](url)
    if start + 3 >= chars.len() || chars[start] != '!' || chars[start + 1] != '[' {
        return None;
    }
    
    let alt_end = find_closing_char(chars, start + 2, ']')?;
    if alt_end + 1 >= chars.len() || chars[alt_end + 1] != '(' {
        return None;
    }
    
    let url_end = find_closing_char(chars, alt_end + 2, ')')?;
    
    let alt: String = chars[start + 2..alt_end].iter().collect();
    let url: String = chars[alt_end + 2..url_end].iter().collect();
    
    Some((url_end + 1, alt, url))
}

fn find_wiki_link(chars: &[char], start: usize) -> Option<(usize, String)> {
    // [[text]]
    if start + 3 >= chars.len() || chars[start] != '[' || chars[start + 1] != '[' {
        return None;
    }
    
    for i in start + 2..chars.len() - 1 {
        if chars[i] == ']' && chars[i + 1] == ']' {
            let text: String = chars[start + 2..i].iter().collect();
            return Some((i + 2, text));
        }
    }
    None
}

fn find_markdown_link(chars: &[char], start: usize) -> Option<(usize, String, String)> {
    // [text](url)
    if start >= chars.len() || chars[start] != '[' {
        return None;
    }
    
    let text_end = find_closing_char(chars, start + 1, ']')?;
    if text_end + 1 >= chars.len() || chars[text_end + 1] != '(' {
        return None;
    }
    
    let url_end = find_closing_char(chars, text_end + 2, ')')?;
    
    let text: String = chars[start + 1..text_end].iter().collect();
    let url: String = chars[text_end + 2..url_end].iter().collect();
    
    Some((url_end + 1, text, url))
}

pub fn extract_plain_text(text: &str) -> String {
    let spans = parse_inline_formatting(text);
    let mut result = String::new();
    let mut last_end = 0;
    
    for span in spans {
        // Add text before this span
        result.push_str(&text[last_end..span.range.start]);
        
        // Add the content without markers
        let content = match span.format {
            InlineFormat::Bold => &text[span.range.start + 2..span.range.end - 2],
            InlineFormat::Italic => &text[span.range.start + 1..span.range.end - 1],
            InlineFormat::Code => &text[span.range.start + 1..span.range.end - 1],
            InlineFormat::Link { ref text, .. } => text,
            InlineFormat::WikiLink { ref text } => text,
            InlineFormat::Image { ref alt, .. } => alt,
        };
        result.push_str(content);
        
        last_end = span.range.end;
    }
    
    // Add remaining text
    result.push_str(&text[last_end..]);
    result
}
