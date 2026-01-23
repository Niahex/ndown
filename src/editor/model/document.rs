use crate::editor::model::block::{Block, BlockType, StyleBits, StyleSpan};
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Clone, Debug)]
pub struct Document {
    pub blocks: Vec<Block>,
    next_id: u64,
    temp_markdown_buf: String,
    temp_char_buf: Vec<char>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            blocks: vec![
                Block::new(1, BlockType::Heading1, "Bienvenue dans Ndown"),
                Block::new(
                    2,
                    BlockType::Paragraph,
                    "Ceci est un éditeur basé sur des blocs.",
                ),
                Block::new(3, BlockType::Quote, "Essayez de taper # titre ou **gras**."),
            ],
            next_id: 4,
            temp_markdown_buf: String::with_capacity(1024),
            temp_char_buf: Vec::with_capacity(1024),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    // Crée une copie "légère" du document (sans les buffers de cache) pour l'export asynchrone
    pub fn snapshot(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            next_id: self.next_id,
            temp_markdown_buf: String::new(), // Pas d'allocation inutile
            temp_char_buf: Vec::new(),
        }
    }

    // Streaming Save (Memory efficient)
    pub fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        for (i, block) in self.blocks.iter().enumerate() {
            let prefix = match block.ty {
                BlockType::Heading1 => "# ",
                BlockType::Heading2 => "## ",
                BlockType::Heading3 => "### ",
                BlockType::Quote => "> ",
                _ => "",
            };
            writer.write_all(prefix.as_bytes())?;

            block.write_markdown_to_writer(&mut writer)?;

            if i < self.blocks.len() - 1 {
                writer.write_all(b"\n\n")?;
            }
        }
        writer.flush()?;
        Ok(())
    }

    pub fn try_convert_block(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx >= self.blocks.len() {
            return None;
        }
        let block = &mut self.blocks[block_idx];

        let removed = if block.ty == BlockType::Paragraph {
            if block.text.starts_with("# ") {
                block.ty = BlockType::Heading1;
                block.text.replace_range(0..2, "");
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(2);
                }
                Some(2)
            } else if block.text.starts_with("## ") {
                block.ty = BlockType::Heading2;
                block.text.replace_range(0..3, "");
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(3);
                }
                Some(3)
            } else if block.text.starts_with("> ") {
                block.ty = BlockType::Quote;
                block.text.replace_range(0..2, "");
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(2);
                }
                Some(2)
            } else {
                None
            }
        } else {
            None
        };

        if removed.is_some() {
            block.mark_dirty();
        }
        removed
    }

    pub fn apply_inline_formatting(&mut self, block_idx: usize) -> bool {
        if block_idx >= self.blocks.len() {
            return false;
        }

        self.temp_markdown_buf.clear();
        self.blocks[block_idx].write_markdown_to(&mut self.temp_markdown_buf);
        let text = &self.temp_markdown_buf;

        if !text.contains('*') && !text.contains('`') {
            return false;
        }

        self.temp_char_buf.clear();
        self.temp_char_buf.extend(text.chars());
        let chars = &self.temp_char_buf;
        let len = chars.len();

        let mut new_styles: Vec<StyleSpan> = Vec::new();
        let mut new_text = String::with_capacity(text.len());

        let mut i = 0;
        let mut is_bold = false;
        let mut is_italic = false;
        let is_code = false;
        let mut changed = false;
        let mut pending_len = 0;

        let mut push_segment = |count: usize, b: bool, it: bool, c: bool| {
            if count == 0 {
                return;
            }
            if let Some(last) = new_styles.last_mut() {
                if last.style.is_bold == b && last.style.is_italic == it && last.style.is_code == c
                {
                    last.len += count;
                    return;
                }
            }
            new_styles.push(StyleSpan {
                len: count,
                style: StyleBits {
                    is_bold: b,
                    is_italic: it,
                    is_code: c,
                },
            });
        };

        while i < len {
            if !is_code && chars[i] == '`' {
                let mut j = i + 1;
                while j < len && chars[j] != '`' {
                    j += 1;
                }
                if j < len {
                    push_segment(pending_len, is_bold, is_italic, is_code);
                    pending_len = 0;
                    for &c in chars.iter().take(j).skip(i + 1) {
                        new_text.push(c);
                    }
                    push_segment(j - (i + 1), false, false, true);
                    i = j + 1;
                    changed = true;
                    continue;
                }
            }

            if !is_code && i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
                let mut has_closing = false;
                if !is_bold {
                    let mut k = i + 2;
                    while k + 1 < len {
                        if chars[k] == '*' && chars[k + 1] == '*' {
                            has_closing = true;
                            break;
                        }
                        k += 1;
                    }
                } else {
                    has_closing = true;
                }

                if has_closing {
                    push_segment(pending_len, is_bold, is_italic, is_code);
                    pending_len = 0;
                    is_bold = !is_bold;
                    i += 2;
                    changed = true;
                    continue;
                }
            }

            if !is_code && chars[i] == '*' {
                let mut has_closing = false;
                if !is_italic {
                    let mut k = i + 1;
                    while k < len {
                        if chars[k] == '*' {
                            if k + 1 < len && chars[k + 1] == '*' {
                                k += 2;
                                continue;
                            }
                            has_closing = true;
                            break;
                        }
                        k += 1;
                    }
                } else {
                    has_closing = true;
                }

                if has_closing {
                    push_segment(pending_len, is_bold, is_italic, is_code);
                    pending_len = 0;
                    is_italic = !is_italic;
                    i += 1;
                    changed = true;
                    continue;
                }
            }

            new_text.push(chars[i]);
            pending_len += 1;
            i += 1;
        }

        push_segment(pending_len, is_bold, is_italic, is_code);

        if changed {
            let block = &mut self.blocks[block_idx];
            block.text = new_text;
            block.styles = new_styles;
            block.mark_dirty();
        }

        changed
    }

    pub fn insert_text_at(&mut self, block_idx: usize, char_idx: usize, text: &str) -> usize {
        if block_idx >= self.blocks.len() {
            return 0;
        }
        let block = &mut self.blocks[block_idx];
        block.mark_dirty();

        let byte_idx = block
            .text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(block.text.len());
        block.text.insert_str(byte_idx, text);

        let added_len = text.chars().count();
        let mut current_idx = 0;
        let mut inserted_style = false;

        for span in &mut block.styles {
            if char_idx <= current_idx + span.len {
                span.len += added_len;
                inserted_style = true;
                break;
            }
            current_idx += span.len;
        }

        if !inserted_style {
            if let Some(last) = block.styles.last_mut() {
                last.len += added_len;
            } else {
                block.styles.push(StyleSpan {
                    len: added_len,
                    style: StyleBits::default(),
                });
            }
        }

        added_len
    }

    pub fn remove_char_at(&mut self, block_idx: usize, char_idx: usize) -> bool {
        if block_idx >= self.blocks.len() {
            return false;
        }
        let block = &mut self.blocks[block_idx];

        if char_idx >= block.text.chars().count() {
            return false;
        }

        let byte_idx = block
            .text
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap();
        block.text.remove(byte_idx);
        block.mark_dirty();

        let mut current_idx = 0;
        let mut span_to_remove = None;

        for (i, span) in block.styles.iter_mut().enumerate() {
            if char_idx < current_idx + span.len {
                span.len -= 1;
                if span.len == 0 {
                    span_to_remove = Some(i);
                }
                break;
            }
            current_idx += span.len;
        }

        if let Some(idx) = span_to_remove {
            if block.styles.len() > 1 {
                block.styles.remove(idx);
            }
        }

        true
    }

    pub fn wrap_selection(&mut self, block_idx: usize, start: usize, end: usize, marker: &str) {
        if block_idx >= self.blocks.len() {
            return;
        }
        self.insert_text_at(block_idx, end, marker);
        self.insert_text_at(block_idx, start, marker);
        self.apply_inline_formatting(block_idx);
    }

    pub fn merge_block_with_prev(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx == 0 || block_idx >= self.blocks.len() {
            return None;
        }
        let block = self.blocks.remove(block_idx);
        let prev_block = &mut self.blocks[block_idx - 1];
        let offset = prev_block.text_len();
        prev_block.text.push_str(&block.text);
        prev_block.styles.extend(block.styles);
        prev_block.mark_dirty();
        Some(offset)
    }

    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) -> (usize, usize) {
        let (start_blk, start_char) = start;
        let (end_blk, end_char) = end;

        if start_blk == end_blk {
            let count = end_char - start_char;
            for _ in 0..count {
                self.remove_char_at(start_blk, start_char);
            }
            (start_blk, start_char)
        } else {
            let first_len = self.blocks[start_blk].text_len();
            for _ in start_char..first_len {
                self.remove_char_at(start_blk, start_char);
            }
            for _ in 0..end_char {
                self.remove_char_at(end_blk, 0);
            }
            if end_blk > start_blk + 1 {
                let to_remove = end_blk - start_blk - 1;
                for _ in 0..to_remove {
                    self.blocks.remove(start_blk + 1);
                }
            }
            if start_blk + 1 < self.blocks.len() {
                let next_block = self.blocks.remove(start_blk + 1);
                let prev = &mut self.blocks[start_blk];
                prev.text.push_str(&next_block.text);
                prev.styles.extend(next_block.styles);
                prev.mark_dirty();
            }
            (start_blk, start_char)
        }
    }
}
