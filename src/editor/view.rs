use crate::editor::model::block::{BlockLayoutCache, BlockType};
use crate::editor::model::document::Document;
use makepad_widgets::*;

pub struct EditorView<'a> {
    pub draw_bg: &'a mut DrawColor,
    pub draw_text_reg: &'a mut DrawText,
    pub draw_text_bold: &'a mut DrawText,
    pub draw_text_italic: &'a mut DrawText,
    pub draw_text_code: &'a mut DrawText,
    pub draw_text_header1: &'a mut DrawText,
    pub draw_text_header2: &'a mut DrawText,
    pub draw_text_header3: &'a mut DrawText,
    pub draw_text_header4: &'a mut DrawText,
    pub draw_text_header5: &'a mut DrawText,
    pub draw_text_quote: &'a mut DrawText,
    pub draw_cursor: &'a mut DrawColor,
    pub draw_selection: &'a mut DrawColor,
}

pub struct HitResult {
    pub block_idx: usize,
    pub char_idx: usize,
}

pub struct DrawParams<'a> {
    pub doc: &'a mut Document,
    pub layout: &'a Layout,
    pub rect: Rect,
    pub cursor: (usize, usize),
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub finger_hit: Option<DVec2>,
    pub scroll: DVec2,
    pub y_offsets_cache: &'a mut Vec<f64>,
}

impl<'a> EditorView<'a> {
    pub fn draw_document(
        &mut self,
        cx: &mut Cx2d,
        params: DrawParams, // Regroupement pour Clippy
    ) -> (f64, Option<HitResult>) {
        self.draw_bg.draw_abs(cx, params.rect);

        let start_y = params.rect.pos.y + params.layout.padding.top - params.scroll.y;
        let start_x = params.rect.pos.x + params.layout.padding.left - params.scroll.x;

        let mut hit_result = None;
        let is_cache_ready = params.y_offsets_cache.len() == params.doc.blocks.len();

        let start_block_idx = if is_cache_ready {
            let scroll_top = params.scroll.y;
            let idx = params.y_offsets_cache.partition_point(|&y| y < scroll_top);
            idx.saturating_sub(1)
        } else {
            0
        };

        let mut current_y = if is_cache_ready {
            start_y + params.y_offsets_cache[start_block_idx]
        } else {
            start_y
        };
        let mut content_y = if is_cache_ready {
            params.y_offsets_cache[start_block_idx]
        } else {
            0.0
        };

        let block_count = params.doc.blocks.len();

        for block_idx in start_block_idx..block_count {
            if !is_cache_ready {
                params.y_offsets_cache.push(content_y);
            }

            let block = &mut params.doc.blocks[block_idx];
            let base_height_fallback = match block.ty {
                BlockType::Heading1 => 33.9,
                BlockType::Heading2 => 26.6,
                BlockType::Heading3 => 21.8,
                BlockType::Heading4 => 19.4,
                BlockType::Heading5 => 16.9,
                BlockType::Quote => 24.2,
                _ => 21.8,
            };

            let is_below_screen =
                is_cache_ready && (current_y > params.rect.pos.y + params.rect.size.y);
            if is_below_screen {
                break;
            }

            let use_cached_layout = block.layout_cache.is_some() && !block.is_dirty;
            let mut block_height = 0.0;

            if use_cached_layout {
                block_height = block.layout_cache.as_ref().unwrap().height;
            }

            if use_cached_layout
                && !is_cache_ready
                && (current_y + block_height < params.rect.pos.y)
            {
                current_y += block_height + 5.0;
                content_y += block_height + 5.0;
                continue;
            }

            let mut current_x = start_x;
            let mut max_h_calc = 0.0;
            let mut char_count_so_far = 0;
            let mut found_cursor = false;
            let mut cursor_x_final = current_x;

            let mut char_iter = block.text.chars();

            for span in &block.styles {
                let base_draw = match block.ty {
                    BlockType::Heading1 => self.draw_text_header1 as &mut DrawText,
                    BlockType::Heading2 => self.draw_text_header2 as &mut DrawText,
                    BlockType::Heading3 => self.draw_text_header3 as &mut DrawText,
                    BlockType::Heading4 => self.draw_text_header4 as &mut DrawText,
                    BlockType::Heading5 => self.draw_text_header5 as &mut DrawText,
                    BlockType::Quote => self.draw_text_quote as &mut DrawText,
                    BlockType::CodeBlock => self.draw_text_code as &mut DrawText,
                    _ => self.draw_text_reg as &mut DrawText,
                };
                let draw_text = if span.style.is_code {
                    self.draw_text_code as &mut DrawText
                } else if span.style.is_bold {
                    self.draw_text_bold as &mut DrawText
                } else if span.style.is_italic {
                    self.draw_text_italic as &mut DrawText
                } else {
                    base_draw
                };

                if block.ty == BlockType::Heading1 {
                    draw_text.text_style.font_size = 29.0;
                } else if block.ty == BlockType::Heading2 {
                    draw_text.text_style.font_size = 21.8;
                } else if block.ty == BlockType::Heading3 {
                    draw_text.text_style.font_size = 19.4;
                } else if block.ty == BlockType::Heading4 {
                    draw_text.text_style.font_size = 16.9;
                } else if block.ty == BlockType::Heading5 {
                    draw_text.text_style.font_size = 14.5;
                } else {
                    draw_text.text_style.font_size = 12.1;
                }

                let mut span_text = String::with_capacity(span.len * 4);
                for _ in 0..span.len {
                    if let Some(c) = char_iter.next() {
                        span_text.push(c);
                    }
                }

                let text_layout =
                    draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), &span_text);
                let width = text_layout.size_in_lpxs.width as f64;
                let height = text_layout.size_in_lpxs.height as f64;

                max_h_calc = max_h_calc.max(height);

                let should_draw = current_y + height >= params.rect.pos.y
                    && current_y < params.rect.pos.y + params.rect.size.y;

                if should_draw {
                    if let Some(((sel_start_blk, sel_start_char), (sel_end_blk, sel_end_char))) =
                        params.selection
                    {
                        if block_idx >= sel_start_blk && block_idx <= sel_end_blk {
                            let blk_start = if block_idx == sel_start_blk {
                                sel_start_char
                            } else {
                                0
                            };
                            let blk_end = if block_idx == sel_end_blk {
                                sel_end_char
                            } else {
                                usize::MAX
                            };
                            let span_start = char_count_so_far;
                            let span_end = char_count_so_far + span.len;
                            let intersect_start = span_start.max(blk_start);
                            let intersect_end = span_end.min(blk_end);
                            if intersect_start < intersect_end {
                                let rel_start = intersect_start - span_start;
                                let rel_end = intersect_end - span_start;
                                let s_before: String = span_text.chars().take(rel_start).collect();
                                let w_before = if rel_start > 0 {
                                    draw_text
                                        .layout(
                                            cx,
                                            0.0,
                                            0.0,
                                            None,
                                            false,
                                            Align::default(),
                                            &s_before,
                                        )
                                        .size_in_lpxs
                                        .width as f64
                                } else {
                                    0.0
                                };
                                let s_sel: String = span_text
                                    .chars()
                                    .skip(rel_start)
                                    .take(rel_end - rel_start)
                                    .collect();
                                let w_sel = draw_text
                                    .layout(cx, 0.0, 0.0, None, false, Align::default(), &s_sel)
                                    .size_in_lpxs
                                    .width as f64;
                                self.draw_selection.draw_abs(
                                    cx,
                                    Rect {
                                        pos: dvec2(current_x + w_before, current_y),
                                        size: dvec2(w_sel, height),
                                    },
                                );
                            }
                        }
                    }

                    if let Some(pos) = params.finger_hit {
                        let r = Rect {
                            pos: dvec2(current_x, current_y),
                            size: dvec2(width, height),
                        };
                        if r.contains(pos) {
                            let rel_x = pos.x - current_x;
                            let avg_char_w = width / span.len as f64;
                            let local_char = (rel_x / avg_char_w).round() as usize;
                            let local_char = local_char.min(span.len);
                            hit_result = Some(HitResult {
                                block_idx,
                                char_idx: char_count_so_far + local_char,
                            });
                        }
                    }

                    draw_text.draw_abs(cx, dvec2(current_x, current_y), &span_text);

                    if block_idx == params.cursor.0
                        && !found_cursor
                        && params.cursor.1 >= char_count_so_far
                        && params.cursor.1 <= char_count_so_far + span.len
                    {
                        let local_idx = params.cursor.1 - char_count_so_far;
                        if local_idx == 0 {
                            cursor_x_final = current_x;
                        } else if local_idx == span.len {
                            cursor_x_final = current_x + width;
                        } else {
                            let sub_text: String = span_text.chars().take(local_idx).collect();
                            let sub_layout = draw_text.layout(
                                cx,
                                0.0,
                                0.0,
                                None,
                                false,
                                Align::default(),
                                &sub_text,
                            );
                            cursor_x_final = current_x + sub_layout.size_in_lpxs.width as f64;
                        }
                        found_cursor = true;
                    }
                }
                current_x += width;
                char_count_so_far += span.len;
            }

            let final_height = if max_h_calc > 1.0 {
                max_h_calc
            } else {
                base_height_fallback
            };
            if !use_cached_layout {
                block.layout_cache = Some(BlockLayoutCache {
                    height: final_height,
                    width: current_x - start_x,
                });
                block.is_dirty = false;
            }

            if block_idx == params.cursor.0 {
                if !found_cursor && params.cursor.1 == char_count_so_far {
                    cursor_x_final = current_x;
                }
                if current_y + final_height >= params.rect.pos.y
                    && current_y < params.rect.pos.y + params.rect.size.y
                {
                    self.draw_cursor.draw_abs(
                        cx,
                        Rect {
                            pos: dvec2(cursor_x_final, current_y),
                            size: dvec2(2.0, final_height),
                        },
                    );
                }
            }

            if hit_result.is_none() {
                if let Some(pos) = params.finger_hit {
                    if pos.y >= current_y && pos.y < current_y + final_height + 5.0 {
                        if pos.x >= current_x {
                            hit_result = Some(HitResult {
                                block_idx,
                                char_idx: char_count_so_far,
                            });
                        } else if pos.x < start_x {
                            hit_result = Some(HitResult {
                                block_idx,
                                char_idx: 0,
                            });
                        }
                    }
                }
            }

            current_y += final_height + 5.0;
            content_y += final_height + 5.0;
        }

        let total_h = if is_cache_ready {
            if let Some(last_y) = params.y_offsets_cache.last() {
                last_y + 50.0
            } else {
                content_y
            }
        } else {
            content_y
        };

        (
            total_h + params.layout.padding.top + params.layout.padding.bottom,
            hit_result,
        )
    }
}
