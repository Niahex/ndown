use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum InlineFormat {
    Bold,
    Italic,
    Underline,
    Code,
}

#[derive(Clone, Debug)]
pub struct InlineSpan {
    pub range: Range<usize>,
    pub format: InlineFormat,
}

pub fn parse_inline_formatting(text: &str) -> Vec<InlineSpan> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Bold: **text**
        if i + 3 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, "**") {
                spans.push(InlineSpan {
                    range: i..end + 2,
                    format: InlineFormat::Bold,
                });
                i = end + 2;
                continue;
            }
        }
        
        // Italic: *text*
        if i + 2 < chars.len() && chars[i] == '*' && (i == 0 || chars[i - 1] != '*') {
            if let Some(end) = find_closing(&chars, i + 1, "*") {
                spans.push(InlineSpan {
                    range: i..end + 1,
                    format: InlineFormat::Italic,
                });
                i = end + 1;
                continue;
            }
        }
        
        // Underline: _text_
        if i + 2 < chars.len() && chars[i] == '_' {
            if let Some(end) = find_closing(&chars, i + 1, "_") {
                spans.push(InlineSpan {
                    range: i..end + 1,
                    format: InlineFormat::Underline,
                });
                i = end + 1;
                continue;
            }
        }
        
        // Code: `text`
        if i + 2 < chars.len() && chars[i] == '`' {
            if let Some(end) = find_closing(&chars, i + 1, "`") {
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

fn find_closing(chars: &[char], start: usize, marker: &str) -> Option<usize> {
    let marker_chars: Vec<char> = marker.chars().collect();
    let marker_len = marker_chars.len();
    
    for i in start..chars.len() {
        if i + marker_len <= chars.len() {
            let matches = (0..marker_len).all(|j| chars[i + j] == marker_chars[j]);
            if matches {
                return Some(i + marker_len - 1);
            }
        }
    }
    None
}
