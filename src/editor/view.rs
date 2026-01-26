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
    pub draw_code_bg: &'a mut DrawColor,
    pub draw_text_code_header: &'a mut DrawText,
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
        let cache_len = params.y_offsets_cache.len();
        let scroll_top = params.scroll.y;

        // Partial Cache Usage:
        // Use the valid part of the cache to find the start block.
        // If the cache was truncated due to a change at block N, cache_len will be N.
        // If we are scrolled before N, we can use the cache.
        // If we are scrolled after N, start_block_idx will be N (or close to it), 
        // and we will recalculate positions from there.
        
        let start_block_idx = if cache_len > 0 {
            let idx = params.y_offsets_cache.partition_point(|&y| y < scroll_top);
            idx.saturating_sub(1)
        } else {
            0
        };

        // Initialize content_y based on the cache if possible
        let mut content_y = if start_block_idx < cache_len {
            params.y_offsets_cache[start_block_idx]
        } else {
             // Fallback: If for some reason start_block_idx is out of bounds (e.g. cache empty), start at 0.
             // If we are resuming from the end of a partial cache (start_block_idx == cache_len),
             // we technically need the previous block's bottom position.
             // However, params.y_offsets_cache stores the TOP of the block.
             // So cache[i] is top of block i.
             // If we start at block i, content_y should be cache[i].
             // If i >= cache_len, we can't look it up.
             // We must start layout from the last known valid block or 0.
             
             // Simplification: If start_block_idx >= cache_len, we force start from the last valid block in cache?
             // But partition_point returns at most cache_len.
             // So start_block_idx <= cache_len - 1 (due to saturating_sub(1)).
             // EXCEPT if partition_point returns 0 (scroll at top).
             
             // So start_block_idx is guaranteed to be < cache_len IF cache_len > 0.
             // If cache_len == 0, start_block_idx is 0, and we use 0.0.
             0.0
        };
        
        // Edge case correction: 
        // If we are starting layout *beyond* the cache (e.g. cache truncated at 50, but we need to draw block 60),
        // we can't jump to 60. We must bridge the gap.
        // But the loop below iterates from start_block_idx.
        // If cache_len=50, partition_point returns 50 (all < scroll).
        // start_block_idx = 49.
        // content_y = cache[49].
        // Loop starts at 49.
        // Block 49 is processed (cached layout used).
        // Next iteration is 50. Cache empty. content_y updated. Pushed.
        // This works perfectly!
        
        let mut current_y = start_y + content_y;
        
        let mut list_counters: Vec<u32> = vec![0; 10];

        // Optimization: Find the start of the current list "cluster" to avoid O(N) iteration from the beginning of the document.
        let mut scan_start_idx = 0;
        if start_block_idx > 0 {
            let mut i = start_block_idx;
            while i > 0 {
                let block = &params.doc.blocks[i - 1];
                if block.ty != BlockType::OrderedListItem && block.ty != BlockType::ListItem {
                    scan_start_idx = i;
                    break;
                }
                i -= 1;
            }
        }

        for i in scan_start_idx..start_block_idx {
            let block = &params.doc.blocks[i];
            if block.ty != BlockType::OrderedListItem && block.ty != BlockType::ListItem {
                list_counters.fill(0);
            } else if block.ty == BlockType::OrderedListItem {
                let level = block.indent as usize;
                if level >= list_counters.len() {
                    list_counters.resize(level + 1, 0);
                }
                list_counters[level] += 1;
                for counter in list_counters.iter_mut().skip(level + 1) {
                    *counter = 0;
                }
            }
        }

        let block_count = params.doc.blocks.len();

        for block_idx in start_block_idx..block_count {
            if block_idx >= params.y_offsets_cache.len() {
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
                BlockType::ListItem => 21.8,
                BlockType::OrderedListItem => 21.8,
                _ => 21.8,
            };

            let is_below_screen = current_y > params.rect.pos.y + params.rect.size.y;
            let must_rebuild_cache = cache_len < block_count;

            if is_below_screen && !must_rebuild_cache {
                break;
            }

            let use_cached_layout = block.layout_cache.is_some() && !block.is_dirty;
            let mut block_height = 0.0;

            if use_cached_layout {
                block_height = block.layout_cache.as_ref().unwrap().height;
            }

            // Vertical Culling (Above Screen)
            if use_cached_layout && (current_y + block_height < params.rect.pos.y) {
                current_y += block_height + 5.0;
                content_y += block_height + 5.0;
                continue;
            }

            let mut current_x = start_x;

            if block.ty == BlockType::CodeBlock {
                let bg_h = if use_cached_layout { 
                    block_height 
                } else {
                    let text_layout = self.draw_text_code.layout(cx, 0.0, 0.0, None, false, Align::default(), &block.text);
                    text_layout.size_in_lpxs.height as f64 + 42.0
                };
                self.draw_code_bg.draw_abs(cx, Rect {
                    pos: dvec2(start_x, current_y),
                    size: dvec2(params.rect.size.x - params.layout.padding.left - params.layout.padding.right, bg_h)
                });
                
                // Draw header
                self.draw_text_code_header.draw_abs(cx, dvec2(start_x + 10.0, current_y + 5.0), "language");
                
                current_y += 32.0; // Space for header (22) + top margin (10)
                current_x += 15.0; // Left margin
            }

            if block.ty != BlockType::OrderedListItem && block.ty != BlockType::ListItem {
                list_counters.fill(0);
            }

            if block.ty == BlockType::ListItem {
                current_x += (block.indent as f64) * 20.0;
                if current_y >= params.rect.pos.y
                    && current_y < params.rect.pos.y + params.rect.size.y
                {
                    self.draw_text_reg
                        .draw_abs(cx, dvec2(current_x, current_y), "• ");
                }
                current_x += 15.0;
            } else if block.ty == BlockType::OrderedListItem {
                let level = block.indent as usize;
                if level >= list_counters.len() {
                    list_counters.resize(level + 1, 0);
                }
                list_counters[level] += 1;
                for counter in list_counters.iter_mut().skip(level + 1) {
                    *counter = 0;
                }

                current_x += (block.indent as f64) * 20.0;

                let prefix = format!("{}. ", list_counters[level]);
                let prefix_layout =
                    self.draw_text_reg
                        .layout(cx, 0.0, 0.0, None, false, Align::default(), &prefix);
                let prefix_width = prefix_layout.size_in_lpxs.width as f64;

                if current_y >= params.rect.pos.y
                    && current_y < params.rect.pos.y + params.rect.size.y
                {
                    self.draw_text_reg
                        .draw_abs(cx, dvec2(current_x, current_y), &prefix);
                }
                current_x += prefix_width + 5.0;
            }

            let mut max_h_calc = 0.0;
            let mut char_count_so_far = 0;
            let mut found_cursor = false;
            let mut cursor_x_final = current_x;
            let mut cursor_y_final = current_y;
            let mut cursor_h_final = 20.0;

            let mut char_iter = block.text.chars();
            let mut current_byte_offset = 0;

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

                // Optimization: Zero-copy slicing
                let mut span_byte_len = 0;
                for _ in 0..span.len {
                    if let Some(c) = char_iter.next() {
                        span_byte_len += c.len_utf8();
                    }
                }
                let span_end_byte = current_byte_offset + span_byte_len;
                // Ensure we don't panic if indices are out of bounds (though they shouldn't be)
                let span_text = if span_end_byte <= block.text.len() {
                    &block.text[current_byte_offset..span_end_byte]
                } else {
                    ""
                };
                current_byte_offset = span_end_byte;

                let text_layout =
                    draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), span_text);
                let width = text_layout.size_in_lpxs.width as f64;
                let height = text_layout.size_in_lpxs.height as f64;

                max_h_calc = max_h_calc.max(height);

                let should_draw = current_y + height >= params.rect.pos.y
                    && current_y < params.rect.pos.y + params.rect.size.y;

                if should_draw {
                    if span.style.is_code {
                        self.draw_code_bg.draw_abs(
                            cx,
                            Rect {
                                pos: dvec2(current_x, current_y),
                                size: dvec2(width, height),
                            },
                        );
                    }

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

                                // Optimization: Slice instead of collect
                                let mut byte_start_sel = 0;
                                let mut byte_len_sel = 0;
                                for (i, c) in span_text.chars().enumerate() {
                                    if i < rel_start {
                                        byte_start_sel += c.len_utf8();
                                    } else if i < rel_end {
                                        byte_len_sel += c.len_utf8();
                                    } else {
                                        break;
                                    }
                                }
                                let s_before = &span_text[0..byte_start_sel];
                                let w_before = if rel_start > 0 {
                                    draw_text
                                        .layout(
                                            cx,
                                            0.0,
                                            0.0,
                                            None,
                                            false,
                                            Align::default(),
                                            s_before,
                                        )
                                        .size_in_lpxs
                                        .width as f64
                                } else {
                                    0.0
                                };

                                let s_sel = &span_text
                                    [byte_start_sel..byte_start_sel + byte_len_sel];
                                let w_sel = draw_text
                                    .layout(cx, 0.0, 0.0, None, false, Align::default(), s_sel)
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

                    if hit_result.is_none() {
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
                    }

                    draw_text.draw_abs(cx, dvec2(current_x, current_y), span_text);

                    if block_idx == params.cursor.0
                        && !found_cursor
                        && params.cursor.1 >= char_count_so_far
                        && params.cursor.1 <= char_count_so_far + span.len
                    {
                        let local_idx = params.cursor.1 - char_count_so_far;
                        
                        let single_line_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), "A");
                        cursor_h_final = if single_line_layout.size_in_lpxs.height > 0.0 {
                            single_line_layout.size_in_lpxs.height as f64
                        } else {
                            20.0
                        };

                        if local_idx == 0 {
                            cursor_x_final = current_x;
                            cursor_y_final = current_y;
                        } else {
                            let mut byte_len_sub = 0;
                            for (i, c) in span_text.chars().enumerate().take(local_idx) {
                                byte_len_sub += c.len_utf8();
                            }
                            let sub_text = &span_text[0..byte_len_sub];
                            let lines: Vec<&str> = sub_text.split('\n').collect();
                            let line_count = lines.len().saturating_sub(1);
                            let last_line = lines.last().cloned().unwrap_or("");
                            
                            let last_line_layout = draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), last_line);
                            
                            cursor_x_final = current_x + last_line_layout.size_in_lpxs.width as f64;
                            cursor_y_final = current_y + (line_count as f64 * cursor_h_final);
                        }
                        found_cursor = true;
                    }
                }
                current_x += width;
                char_count_so_far += span.len;
            }

            let base_calc_height = if max_h_calc > 1.0 {
                max_h_calc
            } else {
                base_height_fallback
            };

            let final_height = if block.ty == BlockType::CodeBlock {
                base_calc_height + 42.0 // header (22) + top margin (10) + bottom margin (10)
            } else {
                base_calc_height
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
                if cursor_y_final + cursor_h_final >= params.rect.pos.y
                    && cursor_y_final < params.rect.pos.y + params.rect.size.y
                {
                    self.draw_cursor.draw_abs(
                        cx,
                        Rect {
                            pos: dvec2(cursor_x_final, cursor_y_final),
                            size: dvec2(2.0, cursor_h_final),
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

        let total_h = if params.y_offsets_cache.len() == params.doc.blocks.len() {
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
