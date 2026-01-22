use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum InlineFormat {
    Bold,
    Italic,
    Code,
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
        };
        result.push_str(content);
        
        last_end = span.range.end;
    }
    
    // Add remaining text
    result.push_str(&text[last_end..]);
    result
}
