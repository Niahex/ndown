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

pub struct HitResult {
    pub block_idx: usize,
    pub char_idx: usize,
}

impl<'a> EditorView<'a> {
    pub fn draw_document(
        &mut self,
        cx: &mut Cx2d,
        doc: &Document,
        layout: &Layout,
        rect: Rect,
        cursor: (usize, usize), 
        selection: Option<((usize, usize), (usize, usize))>,
        _blink_visible: bool,
        finger_hit: Option<DVec2> // Position absolue de la souris pour hit-test
    ) -> (f64, Option<HitResult>) { // Retourne (hauteur totale, résultat du hit)
        
        self.draw_bg.draw_abs(cx, rect);
        
        let start_y = rect.pos.y + layout.padding.top;
        let start_x = rect.pos.x + layout.padding.left;
        let mut current_y = start_y;
        
        let mut hit_result = None;
        
        for (block_idx, block) in doc.blocks.iter().enumerate() {
            let mut current_x = start_x;
            let mut max_height = 0.0;
            let mut char_count_so_far = 0;
            let mut found_cursor = false;
            let mut cursor_x_final = start_x;
            
            // Layout global pour hit-test approximatif si on clique dans le vide à droite
            let mut block_rect = Rect { pos: dvec2(start_x, current_y), size: dvec2(0.0, 0.0) };
            
            // Calculer la hauteur de ligne estimée avant la boucle (pour hit test vertical)
            let base_height = match block.ty {
                BlockType::Heading1 => 28.0,
                BlockType::Heading2 => 22.0,
                BlockType::Quote => 20.0,
                _ => 18.0
            };
            
            // --- DRAW SPANS ---
            for span in &block.content {
                let base_draw = match block.ty {
                    BlockType::Heading1 => self.draw_text_header1 as &mut DrawText,
                    BlockType::Heading2 => self.draw_text_header2 as &mut DrawText,
                    BlockType::Quote => self.draw_text_quote as &mut DrawText,
                    BlockType::CodeBlock => self.draw_text_code as &mut DrawText,
                    _ => self.draw_text_reg as &mut DrawText,
                };
                
                let draw_text = if span.is_code { self.draw_text_code as &mut DrawText }
                else if span.is_bold { self.draw_text_bold as &mut DrawText }
                else if span.is_italic { self.draw_text_italic as &mut DrawText }
                else { base_draw };
                
                if block.ty == BlockType::Heading1 { 
                    draw_text.text_style.font_size = 24.0; 
                } else if block.ty == BlockType::Heading2 { 
                    draw_text.text_style.font_size = 18.0; 
                } else {
                    draw_text.text_style.font_size = 10.0;
                }
                
                let text = &span.text;
                let text_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), text);
                let width = text_layout.size_in_lpxs.width as f64;
                let height = text_layout.size_in_lpxs.height as f64;
                
                max_height = max_height.max(height);
                
                // SELECTION RENDERING
                if let Some(((sel_start_blk, sel_start_char), (sel_end_blk, sel_end_char))) = selection {
                    // Check intersection
                    if block_idx >= sel_start_blk && block_idx <= sel_end_blk {
                        let blk_start = if block_idx == sel_start_blk { sel_start_char } else { 0 };
                        let blk_end = if block_idx == sel_end_blk { sel_end_char } else { usize::MAX };
                        
                        let span_start = char_count_so_far;
                        let span_end = char_count_so_far + span.len();
                        
                        let intersect_start = span_start.max(blk_start);
                        let intersect_end = span_end.min(blk_end);
                        
                        if intersect_start < intersect_end {
                            // Calculate sub-layout for precise rect
                            let rel_start = intersect_start - span_start;
                            let rel_end = intersect_end - span_start;
                            
                            let s_before: String = text.chars().take(rel_start).collect();
                            let w_before = if rel_start > 0 {
                                draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &s_before).size_in_lpxs.width as f64
                            } else { 0.0 };
                            
                            let s_sel: String = text.chars().skip(rel_start).take(rel_end - rel_start).collect();
                            let w_sel = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &s_sel).size_in_lpxs.width as f64;
                            
                            self.draw_selection.draw_abs(cx, Rect {
                                pos: dvec2(current_x + w_before, current_y),
                                size: dvec2(w_sel, height),
                            });
                        }
                    }
                }
                
                // HIT TESTING (Per Span for precision)
                if let Some(pos) = finger_hit {
                    let rect = Rect { pos: dvec2(current_x, current_y), size: dvec2(width, height) };
                    if rect.contains(pos) {
                        // Find precise char
                        let rel_x = pos.x - current_x;
                        // Approximate char index based on avg width (simpler than iterating layouts)
                        // TODO: Use binary search on layouts for perfect precision
                        let avg_char_w = width / span.len() as f64;
                        let local_char = (rel_x / avg_char_w).round() as usize;
                        let local_char = local_char.min(span.len());
                        hit_result = Some(HitResult {
                            block_idx,
                            char_idx: char_count_so_far + local_char
                        });
                    }
                }

                draw_text.draw_abs(cx, dvec2(current_x, current_y), text);
                
                // CURSOR CALC
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
            
            // BLOCK RECT FINALIZATION (for row hit test fallback)
            let line_height = if max_height > 1.0 { max_height } else { base_height };
            block_rect.size = dvec2(current_x - start_x, line_height);
            
            // Draw cursor
            if block_idx == cursor.0 {
                if !found_cursor && cursor.1 == char_count_so_far {
                    cursor_x_final = current_x;
                }
                self.draw_cursor.draw_abs(cx, Rect {
                    pos: dvec2(cursor_x_final, current_y),
                    size: dvec2(2.0, line_height),
                });
            }
            
            // Fallback Hit Test (Click on the right of the line)
            if hit_result.is_none() {
                if let Some(pos) = finger_hit {
                    // Check Y band
                    if pos.y >= current_y && pos.y < current_y + line_height + 5.0 {
                        // If X is beyond text, clamp to end
                        if pos.x >= current_x {
                            hit_result = Some(HitResult {
                                block_idx,
                                char_idx: char_count_so_far
                            });
                        } else if pos.x < start_x {
                             hit_result = Some(HitResult {
                                block_idx,
                                char_idx: 0
                            });
                        }
                    }
                }
            }

            current_y += line_height + 5.0; // Padding
        }
        
        (current_y - rect.pos.y, hit_result)
    }
}