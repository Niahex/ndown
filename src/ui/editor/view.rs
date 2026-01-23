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
        finger_hit: Option<DVec2>,
        scroll: DVec2,
        y_offsets_cache: &mut Vec<f64>
    ) -> (f64, Option<HitResult>) { 
        
        self.draw_bg.draw_abs(cx, rect);
        
        let start_y = rect.pos.y + layout.padding.top - scroll.y;
        let start_x = rect.pos.x + layout.padding.left - scroll.x;
        
        let mut hit_result = None;
        let is_cache_ready = y_offsets_cache.len() == doc.blocks.len();
        
        let start_block_idx = if is_cache_ready {
            let scroll_top = scroll.y;
            let idx = y_offsets_cache.partition_point(|&y| y < scroll_top);
            idx.saturating_sub(1)
        } else { 0 };
        
        let mut current_y = if is_cache_ready { start_y + y_offsets_cache[start_block_idx] } else { start_y };
        let mut content_y = if is_cache_ready { y_offsets_cache[start_block_idx] } else { 0.0 };
        
        for (i, block) in doc.blocks.iter().enumerate().skip(start_block_idx) {
            let block_idx = i;
            
            if !is_cache_ready {
                y_offsets_cache.push(content_y);
            }
            
            let base_height = match block.ty {
                BlockType::Heading1 => 28.0,
                BlockType::Heading2 => 22.0,
                BlockType::Quote => 20.0,
                _ => 18.0
            };
            
            if is_cache_ready && current_y > rect.pos.y + rect.size.y { break; }
            
            if !is_cache_ready {
                if current_y + base_height < rect.pos.y {
                    let h = base_height + 5.0;
                    current_y += h; content_y += h;
                    continue;
                }
                if current_y > rect.pos.y + rect.size.y {
                    let h = base_height + 5.0;
                    current_y += h; content_y += h;
                    continue;
                }
            }

            let mut current_x = start_x;
            let mut max_height = 0.0;
            let mut char_count_so_far = 0;
            let mut found_cursor = false;
            let mut cursor_x_final = current_x;
            let mut block_rect = Rect { pos: dvec2(start_x, current_y), size: dvec2(0.0, 0.0) };
            
            // --- DRAW SPANS (ZERO COPY) ---
            // On utilise un iterateur de chars sur block.text
            let mut char_iter = block.text.chars();
            
            for span in &block.styles {
                let base_draw = match block.ty {
                    BlockType::Heading1 => self.draw_text_header1 as &mut DrawText,
                    BlockType::Heading2 => self.draw_text_header2 as &mut DrawText,
                    BlockType::Quote => self.draw_text_quote as &mut DrawText,
                    BlockType::CodeBlock => self.draw_text_code as &mut DrawText,
                    _ => self.draw_text_reg as &mut DrawText,
                };
                
                let draw_text = if span.style.is_code { self.draw_text_code as &mut DrawText }
                else if span.style.is_bold { self.draw_text_bold as &mut DrawText }
                else if span.style.is_italic { self.draw_text_italic as &mut DrawText }
                else { base_draw };
                
                if block.ty == BlockType::Heading1 { draw_text.text_style.font_size = 24.0; }
                else if block.ty == BlockType::Heading2 { draw_text.text_style.font_size = 18.0; }
                else { draw_text.text_style.font_size = 10.0; }
                
                // Extraction du texte du span (allocation ici car draw_abs veut &str et on a un iterateur)
                // Pour optimiser, on pourrait utiliser des indices de bytes si on les stockait.
                // Ici on consomme l'itérateur pour reconstruire une string TEMPORAIRE pour le draw.
                // C'est le seul endroit où on alloue encore.
                // Pour du vrai zero-copy, makepad devrait supporter le dessin caractère par caractère sans layout lourd,
                // ou on devrait stocker des byte indices dans StyleSpan au lieu de char len.
                // Mais String::from_iter est très optimisé.
                let mut span_text = String::with_capacity(span.len * 4);
                for _ in 0..span.len {
                    if let Some(c) = char_iter.next() { span_text.push(c); }
                }
                
                let text_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &span_text);
                let width = text_layout.size_in_lpxs.width as f64;
                let height = text_layout.size_in_lpxs.height as f64;
                
                max_height = max_height.max(height);
                
                // SELECTION
                if let Some(((sel_start_blk, sel_start_char), (sel_end_blk, sel_end_char))) = selection {
                    if block_idx >= sel_start_blk && block_idx <= sel_end_blk {
                        let blk_start = if block_idx == sel_start_blk { sel_start_char } else { 0 };
                        let blk_end = if block_idx == sel_end_blk { sel_end_char } else { usize::MAX };
                        let span_start = char_count_so_far;
                        let span_end = char_count_so_far + span.len;
                        let intersect_start = span_start.max(blk_start);
                        let intersect_end = span_end.min(blk_end);
                        
                        if intersect_start < intersect_end {
                            let rel_start = intersect_start - span_start;
                            let rel_end = intersect_end - span_start;
                            let s_before: String = span_text.chars().take(rel_start).collect();
                            let w_before = if rel_start > 0 {
                                draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &s_before).size_in_lpxs.width as f64
                            } else { 0.0 };
                            let s_sel: String = span_text.chars().skip(rel_start).take(rel_end - rel_start).collect();
                            let w_sel = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &s_sel).size_in_lpxs.width as f64;
                            self.draw_selection.draw_abs(cx, Rect { pos: dvec2(current_x + w_before, current_y), size: dvec2(w_sel, height) });
                        }
                    }
                }
                
                // HIT TEST
                if let Some(pos) = finger_hit {
                    let rect = Rect { pos: dvec2(current_x, current_y), size: dvec2(width, height) };
                    if rect.contains(pos) {
                        let rel_x = pos.x - current_x;
                        let avg_char_w = width / span.len as f64;
                        let local_char = (rel_x / avg_char_w).round() as usize;
                        let local_char = local_char.min(span.len);
                        hit_result = Some(HitResult { block_idx, char_idx: char_count_so_far + local_char });
                    }
                }

                draw_text.draw_abs(cx, dvec2(current_x, current_y), &span_text);
                
                // CURSOR
                let span_len = span.len;
                if block_idx == cursor.0 && !found_cursor {
                    if cursor.1 >= char_count_so_far && cursor.1 <= char_count_so_far + span_len {
                        let local_idx = cursor.1 - char_count_so_far;
                        if local_idx == 0 { cursor_x_final = current_x; }
                        else if local_idx == span_len { cursor_x_final = current_x + width; }
                        else {
                            let sub_text: String = span_text.chars().take(local_idx).collect();
                            let sub_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &sub_text);
                            cursor_x_final = current_x + sub_layout.size_in_lpxs.width as f64;
                        }
                        found_cursor = true;
                    }
                }
                current_x += width;
                char_count_so_far += span_len;
            }
            
            let line_height = if max_height > 1.0 { max_height } else { base_height };
            block_rect.size = dvec2(current_x - start_x, line_height);
            
            if block_idx == cursor.0 {
                if !found_cursor && cursor.1 == char_count_so_far { cursor_x_final = current_x; }
                self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(cursor_x_final, current_y), size: dvec2(2.0, line_height) });
            }
            
            if hit_result.is_none() {
                if let Some(pos) = finger_hit {
                    if pos.y >= current_y && pos.y < current_y + line_height + 5.0 {
                        if pos.x >= current_x { hit_result = Some(HitResult { block_idx, char_idx: char_count_so_far }); }
                        else if pos.x < start_x { hit_result = Some(HitResult { block_idx, char_idx: 0 }); }
                    }
                }
            }

            current_y += line_height + 5.0; 
            content_y += line_height + 5.0;
        }
        
        let total_height = if is_cache_ready {
            if let Some(last_y) = y_offsets_cache.last() { last_y + 50.0 } else { content_y }
        } else { content_y };
        
        (total_height + layout.padding.top + layout.padding.bottom, hit_result)
    }
}