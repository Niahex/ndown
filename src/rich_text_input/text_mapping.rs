use crate::rich_text_input::inline::{parse_inline_formatting, InlineFormat};

#[derive(Clone, Debug)]
pub struct VisualSegment {
    pub visual_start: usize,
    pub visual_end: usize,
    pub real_start: usize,
    pub real_end: usize,
    pub format: Option<InlineFormat>,
}

#[derive(Clone, Debug, Default)]
pub struct TextMapping {
    pub visual_text: String,
    pub segments: Vec<VisualSegment>,
}

impl TextMapping {
    pub fn from_text(text: &str) -> Self {
        let spans = parse_inline_formatting(text);
        let mut visual_text = String::new();
        let mut segments = Vec::new();
        let mut last_end = 0;
        let mut visual_pos = 0;

        for span in spans {
            // Add plain text before this span
            if span.range.start > last_end {
                let plain = &text[last_end..span.range.start];
                segments.push(VisualSegment {
                    visual_start: visual_pos,
                    visual_end: visual_pos + plain.len(),
                    real_start: last_end,
                    real_end: span.range.start,
                    format: None,
                });
                visual_text.push_str(plain);
                visual_pos += plain.len();
            }

            // Add formatted segment (without markers)
            let (content, format) = match &span.format {
                InlineFormat::Bold => {
                    let content = &text[span.range.start + 2..span.range.end - 2];
                    (content, Some(span.format.clone()))
                }
                InlineFormat::Italic => {
                    let content = &text[span.range.start + 1..span.range.end - 1];
                    (content, Some(span.format.clone()))
                }
                InlineFormat::Underline => {
                    let content = &text[span.range.start + 1..span.range.end - 1];
                    (content, Some(span.format.clone()))
                }
                InlineFormat::Code => {
                    let content = &text[span.range.start + 1..span.range.end - 1];
                    (content, Some(span.format.clone()))
                }
            };

            segments.push(VisualSegment {
                visual_start: visual_pos,
                visual_end: visual_pos + content.len(),
                real_start: span.range.start,
                real_end: span.range.end,
                format,
            });
            visual_text.push_str(content);
            visual_pos += content.len();
            last_end = span.range.end;
        }

        // Add remaining plain text
        if last_end < text.len() {
            let plain = &text[last_end..];
            segments.push(VisualSegment {
                visual_start: visual_pos,
                visual_end: visual_pos + plain.len(),
                real_start: last_end,
                real_end: text.len(),
                format: None,
            });
            visual_text.push_str(plain);
        }

        Self { visual_text, segments }
    }

    pub fn visual_to_real(&self, visual_pos: usize) -> usize {
        for seg in &self.segments {
            if visual_pos >= seg.visual_start && visual_pos <= seg.visual_end {
                let offset = visual_pos - seg.visual_start;
                return seg.real_start + offset;
            }
        }
        self.segments.last().map(|s| s.real_end).unwrap_or(0)
    }

    pub fn real_to_visual(&self, real_pos: usize) -> usize {
        for seg in &self.segments {
            if real_pos >= seg.real_start && real_pos <= seg.real_end {
                let offset = real_pos - seg.real_start;
                return seg.visual_start + offset;
            }
        }
        self.segments.last().map(|s| s.visual_end).unwrap_or(0)
    }
}
