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
            // Pour la conversion de type de bloc, on regarde le texte brut visible
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

    // Parser robuste basé sur une pile pour gérer les styles imbriqués
    pub fn apply_inline_formatting(&mut self, block_idx: usize) -> bool {
        if block_idx >= self.blocks.len() { return false; }
        
        let block = &mut self.blocks[block_idx];
        let text = block.to_markdown(); 
        
        // Debug
        // makepad_widgets::log!("Parsing: '{}'", text);
        
        if !text.contains('*') && !text.contains('`') {
            return false;
        }

        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;
        
        // États courants
        let mut current_text = String::new();
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_code = false;
        
        let mut changed = false;

        while i < len {
            // CODE `...`
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
            
            // BOLD **...**
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
            
            // ITALIC *...*
            if !is_code && chars[i] == '*' {
                // IMPORTANT: Vérifier que ce n'est PAS un morceau de bold (déjà checké au dessus, mais attention)
                // Si on a `***`, le bold a déjà consommé les deux premiers. Le 3ème tombe ici.
                
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
                            // Il faut s'assurer que ce n'est pas un bold !
                            // Si k est le début de `**`, c'est pas bon pour italic
                            if k + 1 < len && chars[k+1] == '*' {
                                // C'est un bold, on continue de chercher
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
            // makepad_widgets::log!("Changed! Spans: {:?}", spans.len());
            block.content = spans;
        }
        
        changed
    }

    pub fn insert_text_at(&mut self, block_idx: usize, char_idx: usize, text: &str) -> usize {
        if block_idx >= self.blocks.len() { return 0; }
        let block = &mut self.blocks[block_idx];
        
        let mut current_idx = 0;
        let mut inserted = false;
        let added_len = text.chars().count(); // Longueur en chars pour le curseur

        for span in &mut block.content {
            let span_len = span.len();
            
            // Si on est dans le span ou juste à la fin
            if char_idx <= current_idx + span_len {
                let local_idx = char_idx - current_idx;
                let byte_idx = span.text.char_indices().nth(local_idx).map(|(i,_)| i).unwrap_or(span.text.len());
                span.text.insert_str(byte_idx, text);
                inserted = true;
                break;
            }
            current_idx += span_len;
        }
        
        // Si on n'a pas inséré (fin de ligne absolue ou bloc vide), on ajoute au dernier
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
        for (i, span) in block.content.iter_mut().enumerate() {
            let span_len = span.len();
            
            if char_idx < current_idx + span_len {
                let local_idx = char_idx - current_idx;
                let byte_idx = span.text.char_indices().nth(local_idx).map(|(i,_)| i).unwrap();
                span.text.remove(byte_idx);
                
                // Si le span devient vide, on pourrait le supprimer (sauf si c'est le seul)
                // Mais gardons ça simple pour l'instant
                return true;
            }
            current_idx += span_len;
        }
        false
    }
}