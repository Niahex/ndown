use makepad_widgets::*;
use crate::markdown::inline::{parse_inline_formatting, InlineFormat, extract_plain_text};

pub struct RichTextRenderer;

impl RichTextRenderer {
    pub fn render_to_textflow(cx: &mut Cx2d, tf: &mut TextFlow, text: &str) {
        let spans = parse_inline_formatting(text);
        let mut last_end = 0;
        
        for span in spans {
            // Render text before this span
            if span.range.start > last_end {
                let plain_text = &text[last_end..span.range.start];
                tf.draw_text(cx, plain_text);
            }
            
            // Render the formatted span
            let content = match span.format {
                InlineFormat::Bold => {
                    tf.bold.push();
                    let content = &text[span.range.start + 2..span.range.end - 2];
                    tf.draw_text(cx, content);
                    tf.bold.pop();
                    content
                }
                InlineFormat::Italic => {
                    tf.italic.push();
                    let content = &text[span.range.start + 1..span.range.end - 1];
                    tf.draw_text(cx, content);
                    tf.italic.pop();
                    content
                }
                InlineFormat::Code => {
                    tf.fixed.push();
                    let content = &text[span.range.start + 1..span.range.end - 1];
                    tf.draw_text(cx, content);
                    tf.fixed.pop();
                    content
                }
            };
            
            last_end = span.range.end;
        }
        
        // Render remaining text
        if last_end < text.len() {
            tf.draw_text(cx, &text[last_end..]);
        }
    }
    
    pub fn has_inline_formatting(text: &str) -> bool {
        !parse_inline_formatting(text).is_empty()
    }
}
