use crate::model::block::{Block, BlockType, StyleSpan, StyleBits};
use std::fs::File;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct Document {
    pub blocks: Vec<Block>,
    next_id: u64,
    
    // Buffers de parsing
    temp_markdown_buf: String,
    temp_char_buf: Vec<char>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            blocks: vec![
                Block::new(1, BlockType::Heading1, "Bienvenue dans Ndown"),
                Block::new(2, BlockType::Paragraph, "Ceci est un éditeur basé sur des blocs."),
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
    
    pub fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let mut output = String::new();
        for (i, block) in self.blocks.iter().enumerate() {
            let prefix = match block.ty {
                BlockType::Heading1 => "# ",
                BlockType::Heading2 => "## ",
                BlockType::Heading3 => "### ",
                BlockType::Quote => "> ",
                _ => "",
            };
            output.push_str(prefix);
            output.push_str(&block.to_markdown());
            if i < self.blocks.len() - 1 {
                output.push('\n'); output.push('\n'); 
            }
        }
        let mut file = File::create(filename)?;
        file.write_all(output.as_bytes())?;
        Ok(())
    }

    pub fn try_convert_block(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx >= self.blocks.len() { return None; }
        let block = &mut self.blocks[block_idx];
        
        // Conversion de type: on regarde le texte brut (car les marqueurs # sont dans le texte visible au début)
        if block.ty == BlockType::Paragraph {
            if block.text.starts_with("# ") {
                block.ty = BlockType::Heading1;
                // On supprime les caractères du texte
                // Note: remove_range manual car String::remove est char par char
                block.text.replace_range(0..2, "");
                // On ajuste le premier style
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(2);
                }
                block.mark_dirty();
                return Some(2);
            }
            if block.text.starts_with("## ") {
                block.ty = BlockType::Heading2;
                block.text.replace_range(0..3, "");
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(3);
                }
                block.mark_dirty();
                return Some(3);
            }
            if block.text.starts_with("> ") {
                block.ty = BlockType::Quote;
                block.text.replace_range(0..2, "");
                if let Some(first) = block.styles.first_mut() {
                    first.len = first.len.saturating_sub(2);
                }
                block.mark_dirty();
                return Some(2);
            }
        }
        None
    }

    // Le Parser "Text + Styles"
    // Il reconstruit le markdown, le parse, et génère DEUX choses :
    // 1. Le nouveau texte brut (sans marqueurs) -> Zero copy si pas de changement ? Non, on recrée String.
    // 2. La nouvelle liste de styles.
    pub fn apply_inline_formatting(&mut self, block_idx: usize) -> bool {
        if block_idx >= self.blocks.len() { return false; }
        
        // 1. Markdown Source
        self.temp_markdown_buf.clear();
        self.blocks[block_idx].write_markdown_to(&mut self.temp_markdown_buf);
        let text = &self.temp_markdown_buf;
        
        if !text.contains('*') && !text.contains('`') { return false; }

        self.temp_char_buf.clear();
        self.temp_char_buf.extend(text.chars());
        let chars = &self.temp_char_buf;
        let len = chars.len();
        
        let mut new_styles: Vec<StyleSpan> = Vec::new();
        let mut new_text = String::with_capacity(text.len());
        
        let mut i = 0;
        
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_code = false;
        let mut changed = false;
        
        let mut pending_len = 0;

        // Fonction helper pour pousser un segment
        let mut push_segment = |count: usize, b: bool, it: bool, c: bool| {
            if count == 0 { return; }
            // Fusion avec le dernier style si identique
            if let Some(last) = new_styles.last_mut() {
                if last.style.is_bold == b && last.style.is_italic == it && last.style.is_code == c {
                    last.len += count;
                    return;
                }
            }
            new_styles.push(StyleSpan {
                len: count,
                style: StyleBits { is_bold: b, is_italic: it, is_code: c }
            });
        };

        while i < len {
            // CODE
            if !is_code && chars[i] == '`' {
                let mut j = i + 1;
                while j < len && chars[j] != '`' { j += 1; }
                if j < len { 
                    // Flush pending
                    push_segment(pending_len, is_bold, is_italic, is_code);
                    pending_len = 0;
                    
                    // Add content
                    for k in i+1..j { new_text.push(chars[k]); }
                    push_segment(j - (i + 1), false, false, true); // Code style only
                    
                    i = j + 1;
                    changed = true;
                    continue;
                }
            }
            
            // BOLD
            if !is_code && i + 1 < len && chars[i] == '*' && chars[i+1] == '*' {
                // Check closing
                let mut has_closing = false;
                if !is_bold {
                    let mut k = i + 2;
                    while k + 1 < len {
                        if chars[k] == '*' && chars[k+1] == '*' { has_closing = true; break; }
                        k += 1;
                    }
                } else { has_closing = true; }
                
                if has_closing {
                    push_segment(pending_len, is_bold, is_italic, is_code);
                    pending_len = 0;
                    is_bold = !is_bold;
                    i += 2;
                    changed = true;
                    continue;
                }
            }
            
            // ITALIC
            if !is_code && chars[i] == '*' {
                // Check closing
                let mut has_closing = false;
                if !is_italic {
                    let mut k = i + 1;
                    while k < len {
                        if chars[k] == '*' { 
                             if k + 1 < len && chars[k+1] == '*' { k += 2; continue; }
                            has_closing = true; break; 
                        }
                        k += 1;
                    }
                } else { has_closing = true; }
                
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
        if block_idx >= self.blocks.len() { return 0; }
        let block = &mut self.blocks[block_idx];
        block.mark_dirty();
        
        // Insertion dans le texte brut
        let byte_idx = block.text.char_indices().nth(char_idx).map(|(i,_)| i).unwrap_or(block.text.len());
        block.text.insert_str(byte_idx, text);
        
        // Mise à jour des styles
        let added_len = text.chars().count();
        let mut current_idx = 0;
        let mut inserted_style = false;
        
        // On cherche le span où on insère pour augmenter sa taille
        for span in &mut block.styles {
            if char_idx <= current_idx + span.len {
                span.len += added_len;
                inserted_style = true;
                break;
            }
            current_idx += span.len;
        }
        
        if !inserted_style {
            // Si à la toute fin, on ajoute au dernier ou on crée un nouveau
            if let Some(last) = block.styles.last_mut() {
                last.len += added_len;
            } else {
                block.styles.push(StyleSpan { len: added_len, style: StyleBits::default() });
            }
        }
        
        added_len
    }

    pub fn remove_char_at(&mut self, block_idx: usize, char_idx: usize) -> bool {
        if block_idx >= self.blocks.len() { return false; }
        let block = &mut self.blocks[block_idx];
        
        if char_idx >= block.text.chars().count() { return false; }
        
        // Remove from text
        let byte_idx = block.text.char_indices().nth(char_idx).map(|(i,_)| i).unwrap();
        block.text.remove(byte_idx);
        block.mark_dirty();
        
        // Update styles
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
            // Si le span devient vide, on l'enlève, SAUF si c'est le seul (pour garder le style courant ?)
            // Pour simplifier, on l'enlève. Si on tape ensuite, on recréera un style par défaut ou on héritera du précédent.
            // (La gestion fine du "style curseur" est complexe, ici on simplifie).
            if block.styles.len() > 1 {
                block.styles.remove(idx);
            }
        }
        
        true
    }

    pub fn wrap_selection(&mut self, block_idx: usize, start: usize, end: usize, marker: &str) {
        if block_idx >= self.blocks.len() { return; }
        // On insère les marqueurs dans le texte. Le parser s'occupera de transformer ça en style.
        self.insert_text_at(block_idx, end, marker);
        self.insert_text_at(block_idx, start, marker);
        self.apply_inline_formatting(block_idx);
    }

    pub fn merge_block_with_prev(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx == 0 || block_idx >= self.blocks.len() { return None; }
        let block = self.blocks.remove(block_idx);
        let prev_block = &mut self.blocks[block_idx - 1];
        
        let offset = prev_block.text_len();
        
        // Merge text
        prev_block.text.push_str(&block.text);
        // Merge styles
        prev_block.styles.extend(block.styles);
        // Optimize styles (merge adjacent identical styles)
        // TODO: faire une passe de clean
        
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
            return (start_blk, start_char);
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
                // Merge manually logic
                let prev = &mut self.blocks[start_blk];
                prev.text.push_str(&next_block.text);
                prev.styles.extend(next_block.styles);
                prev.mark_dirty();
            }
            return (start_blk, start_char);
        }
    }
}
