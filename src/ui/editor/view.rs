use makepad_widgets::*;
use crate::model::block::BlockType;
use crate::model::document::Document;

pub struct EditorView<'a> {
    pub draw_bg: &'a mut DrawColor,
    pub draw_text_reg: &'a mut DrawText,
    pub draw_text_bold: &'a mut DrawText,
    pub draw_text_italic: &'a mut DrawText,
    pub draw_text_code: &'a mut DrawText,
    pub draw_text_header1: &'a mut DrawText,
    pub draw_text_header2: &'a mut DrawText,
    pub draw_text_quote: &'a mut DrawText,
    
    pub draw_cursor: &'a mut DrawColor,
    pub draw_selection: &'a mut DrawColor,
}

impl<'a> EditorView<'a> {
    pub fn draw_document(
        &mut self,
        cx: &mut Cx2d,
        doc: &Document,
        layout: &Layout,
        rect: Rect,
        cursor: (usize, usize), // Block Index, Char Index global au bloc
        _selection: Option<((usize, usize), (usize, usize))>,
        _blink_visible: bool
    ) -> f64 { 
        
        self.draw_bg.draw_abs(cx, rect);
        
        let start_y = rect.pos.y + layout.padding.top;
        let start_x = rect.pos.x + layout.padding.left;
        let mut current_y = start_y;
        
        for (block_idx, block) in doc.blocks.iter().enumerate() {
            let mut current_x = start_x;
            let mut max_height = 0.0;
            let mut char_count_so_far = 0;
            let mut found_cursor = false;
            let mut cursor_x_final = start_x;
            
            // On itère sur les spans pour le rendu riche
            for span in &block.content {
                // Choix du style de base selon le bloc
                let base_draw = match block.ty {
                    BlockType::Heading1 => self.draw_text_header1 as &mut DrawText,
                    BlockType::Heading2 => self.draw_text_header2 as &mut DrawText,
                    BlockType::Quote => self.draw_text_quote as &mut DrawText,
                    BlockType::CodeBlock => self.draw_text_code as &mut DrawText,
                    _ => self.draw_text_reg as &mut DrawText,
                };
                
                // Application des variations inline (Gras/Italique/Code)
                // Note: Ici on "emprunte" le style d'un autre draw_text si besoin, 
                // ou on modifie les propriétés du base_draw.
                // Pour faire simple, on va switcher de draw_text.
                
                let draw_text = if span.is_code {
                    self.draw_text_code as &mut DrawText
                } else if span.is_bold {
                    self.draw_text_bold as &mut DrawText
                } else if span.is_italic {
                    self.draw_text_italic as &mut DrawText
                } else {
                    base_draw
                };
                
                // Si c'est un header, on force la taille même si c'est bold/italic (TODO: Mixer les styles)
                if block.ty == BlockType::Heading1 { draw_text.text_style.font_size = 24.0; }
                if block.ty == BlockType::Heading2 { draw_text.text_style.font_size = 18.0; }
                
                let text = &span.text;
                let text_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), text);
                let width = text_layout.size_in_lpxs.width as f64;
                let height = text_layout.size_in_lpxs.height as f64;
                
                max_height = max_height.max(height);
                
                // Dessin
                draw_text.draw_abs(cx, dvec2(current_x, current_y), text);
                
                // Calcul Curseur
                let span_len = span.len();
                if block_idx == cursor.0 && !found_cursor {
                    if cursor.1 >= char_count_so_far && cursor.1 <= char_count_so_far + span_len {
                        let local_idx = cursor.1 - char_count_so_far;
                        if local_idx == 0 {
                            cursor_x_final = current_x;
                        } else if local_idx == span_len {
                            cursor_x_final = current_x + width;
                        } else {
                            let sub_text: String = text.chars().take(local_idx).collect();
                            let sub_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &sub_text);
                            cursor_x_final = current_x + sub_layout.size_in_lpxs.width as f64;
                        }
                        found_cursor = true;
                    }
                }
                
                current_x += width;
                char_count_so_far += span_len;
            }
            
            // Si bloc vide ou curseur à la toute fin
            if block_idx == cursor.0 {
                if !found_cursor && cursor.1 == char_count_so_far {
                    cursor_x_final = current_x;
                }
                // Si max_height est 0 (ligne vide), on prend une valeur par défaut
                let cursor_height = if max_height > 1.0 { max_height } else { 
                    match block.ty {
                        BlockType::Heading1 => 28.0,
                        BlockType::Heading2 => 22.0,
                        _ => 18.0 
                    }
                };
                
                self.draw_cursor.draw_abs(cx, Rect {
                    pos: dvec2(cursor_x_final, current_y),
                    size: dvec2(2.0, cursor_height),
                });
            }
            
            let line_height = if max_height > 1.0 { max_height } else { 18.0 };
            current_y += line_height + 5.0; // Padding fixe
        }
        
        current_y - rect.pos.y
    }
}
