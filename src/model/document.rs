use crate::model::block::{Block, BlockType, TextSpan};

#[derive(Clone, Debug)]
pub struct Document {
    pub blocks: Vec<Block>,
    next_id: u64,
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
    
    pub fn try_convert_block(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx >= self.blocks.len() { return None; }
        
        let block = &mut self.blocks[block_idx];
        
        if block.ty == BlockType::Paragraph {
            let text = block.full_text();
            
            if text.starts_with("# ") {
                block.ty = BlockType::Heading1;
                block.content = vec![TextSpan::new(&text[2..])];
                return Some(2);
            }
            if text.starts_with("## ") {
                block.ty = BlockType::Heading2;
                block.content = vec![TextSpan::new(&text[3..])];
                return Some(3);
            }
            if text.starts_with("> ") {
                block.ty = BlockType::Quote;
                block.content = vec![TextSpan::new(&text[2..])];
                return Some(2);
            }
        }
        None
    }

    pub fn apply_inline_formatting(&mut self, block_idx: usize) -> bool {
        if block_idx >= self.blocks.len() { return false; }
        
        let block = &mut self.blocks[block_idx];
        let text = block.to_markdown(); 
        
        if !text.contains('*') && !text.contains('`') {
            return false;
        }

        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;
        
        let mut current_text = String::new();
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_code = false;
        
        let mut changed = false;

        while i < len {
            if !is_code && chars[i] == '`' {
                let mut j = i + 1;
                while j < len && chars[j] != '`' { j += 1; }
                
                if j < len { 
                    if !current_text.is_empty() {
                        let mut s = TextSpan::new(&current_text);
                        s.is_bold = is_bold; s.is_italic = is_italic;
                        spans.push(s);
                        current_text.clear();
                    }
                    let content: String = chars[i+1..j].iter().collect();
                    let mut s = TextSpan::new(&content);
                    s.is_code = true;
                    spans.push(s);
                    i = j + 1;
                    changed = true;
                    continue;
                }
            }
            
            if !is_code && i + 1 < len && chars[i] == '*' && chars[i+1] == '*' {
                if !current_text.is_empty() {
                    let mut s = TextSpan::new(&current_text);
                    s.is_bold = is_bold; s.is_italic = is_italic;
                    spans.push(s);
                    current_text.clear();
                }
                
                let mut has_closing = false;
                if !is_bold {
                    let mut k = i + 2;
                    while k + 1 < len {
                        if chars[k] == '*' && chars[k+1] == '*' { has_closing = true; break; }
                        k += 1;
                    }
                } else {
                    has_closing = true; 
                }

                if has_closing {
                    is_bold = !is_bold;
                    i += 2;
                    changed = true;
                    continue;
                }
            }
            
            if !is_code && chars[i] == '*' {
                if !current_text.is_empty() {
                    let mut s = TextSpan::new(&current_text);
                    s.is_bold = is_bold; s.is_italic = is_italic;
                    spans.push(s);
                    current_text.clear();
                }
                
                let mut has_closing = false;
                if !is_italic {
                    let mut k = i + 1;
                    while k < len {
                        if chars[k] == '*' { 
                             if k + 1 < len && chars[k+1] == '*' {
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
                    is_italic = !is_italic;
                    i += 1;
                    changed = true;
                    continue;
                }
            }
            
            current_text.push(chars[i]);
            i += 1;
        }
        
        if !current_text.is_empty() {
            let mut s = TextSpan::new(&current_text);
            s.is_bold = is_bold; s.is_italic = is_italic;
            spans.push(s);
        }
        
        if changed {
            block.content = spans;
        }
        
        changed
    }

    pub fn insert_text_at(&mut self, block_idx: usize, char_idx: usize, text: &str) -> usize {
        if block_idx >= self.blocks.len() { return 0; }
        let block = &mut self.blocks[block_idx];
        
        let mut current_idx = 0;
        let mut inserted = false;
        let added_len = text.chars().count();

        for span in &mut block.content {
            let span_len = span.len();
            
            if char_idx <= current_idx + span_len {
                let local_idx = char_idx - current_idx;
                let byte_idx = span.text.char_indices().nth(local_idx).map(|(i,_)| i).unwrap_or(span.text.len());
                span.text.insert_str(byte_idx, text);
                inserted = true;
                break;
            }
            current_idx += span_len;
        }
        
        if !inserted {
            if let Some(last) = block.content.last_mut() {
                last.text.push_str(text);
            } else {
                block.content.push(TextSpan::new(text));
            }
        }
        
        added_len
    }

    pub fn remove_char_at(&mut self, block_idx: usize, char_idx: usize) -> bool {
        if block_idx >= self.blocks.len() { return false; }
        let block = &mut self.blocks[block_idx];
        
        let mut current_idx = 0;
        for (_i, span) in block.content.iter_mut().enumerate() {
            let span_len = span.len();
            
            if char_idx < current_idx + span_len {
                let local_idx = char_idx - current_idx;
                let byte_idx = span.text.char_indices().nth(local_idx).map(|(i,_)| i).unwrap();
                span.text.remove(byte_idx);
                return true;
            }
            current_idx += span_len;
        }
        false
    }

    pub fn wrap_selection(&mut self, block_idx: usize, start: usize, end: usize, marker: &str) {
        if block_idx >= self.blocks.len() { return; }
        self.insert_text_at(block_idx, end, marker);
        self.insert_text_at(block_idx, start, marker);
        self.apply_inline_formatting(block_idx);
    }

    pub fn merge_block_with_prev(&mut self, block_idx: usize) -> Option<usize> {
        if block_idx == 0 || block_idx >= self.blocks.len() { return None; }
        
        let block = self.blocks.remove(block_idx);
        let prev_block = &mut self.blocks[block_idx - 1];
        
        let offset = prev_block.text_len();
        prev_block.content.extend(block.content);
        
        Some(offset)
    }

    // Supprime une plage de texte, potentiellement sur plusieurs blocs.
    // Retourne la nouvelle position du curseur (bloc, char).
    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) -> (usize, usize) {
        let (start_blk, start_char) = start;
        let (end_blk, end_char) = end;

        if start_blk == end_blk {
            let block = &mut self.blocks[start_blk];
            // Suppression naive: on reconstruit le texte sans la partie supprimée et on reparse
            // C'est plus simple que de manipuler les spans manuellement pour un MVP
            let full_text = block.full_text(); // TODO: utiliser to_markdown pour préserver styles ?
            // Attention: full_text perd les styles. Pour bien faire il faudrait couper dans les spans.
            // Pour l'instant on fait delete char par char, c'est lent mais sûr pour les spans.
            
            let count = end_char - start_char;
            for _ in 0..count {
                self.remove_char_at(start_blk, start_char);
            }
            return (start_blk, start_char);
        } else {
            // Multiline delete
            // 1. Delete end of first block
            let first_len = self.blocks[start_blk].text_len();
            for _ in start_char..first_len {
                self.remove_char_at(start_blk, start_char);
            }
            
            // 2. Delete start of last block
            for _ in 0..end_char {
                 self.remove_char_at(end_blk, 0);
            }
            
            // 3. Remove intermediate blocks
            if end_blk > start_blk + 1 {
                let to_remove = end_blk - start_blk - 1;
                for _ in 0..to_remove {
                    self.blocks.remove(start_blk + 1);
                }
            }
            
            // 4. Merge last block into first
            // Après l'étape 3, le 'last block' est devenu start_blk + 1
            if start_blk + 1 < self.blocks.len() {
                let next_block = self.blocks.remove(start_blk + 1);
                self.blocks[start_blk].content.extend(next_block.content);
            }
            
            return (start_blk, start_char);
        }
    }
}
