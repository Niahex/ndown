use makepad_widgets::*;

pub mod model; // Nouveau module enfant
use model::document::Document;
use model::block::{Block, BlockType};

pub mod view;
use view::{EditorView, DrawParams};

live_design!{
    use link::theme::*;
    use link::widgets::*;
    
    pub EditorArea = {{EditorArea}}{
        width: Fill, height: Fill
        draw_bg: { color: #2e3440 }
        
        scroll_bars: <ScrollBars> {}
        
        draw_text_reg: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: #eceff4 }
        draw_text_bold: { text_style: <THEME_FONT_BOLD> { font_size: 10.0 }, color: #eceff4 }
        draw_text_italic: { text_style: <THEME_FONT_ITALIC> { font_size: 10.0 }, color: #eceff4 }
        draw_text_code: { text_style: <THEME_FONT_CODE> { font_size: 10.0 }, color: #a3be8c }
        
        draw_text_header1: { text_style: <THEME_FONT_BOLD> { font_size: 24.0 }, color: #88c0d0 }
        draw_text_header2: { text_style: <THEME_FONT_BOLD> { font_size: 18.0 }, color: #81a1c1 }
        draw_text_quote: { text_style: <THEME_FONT_ITALIC> { font_size: 11.0 }, color: #d08770 }
        
        draw_cursor: { color: #ffffff }
        draw_selection: { color: #4c566a }
        
        animator: {
            blink = {
                default: off
                off = { from: {all: Forward {duration: 0.1}} apply: { draw_cursor: { color: #ffffff00 } } }
                on = { from: {all: Forward {duration: 0.1}} apply: { draw_cursor: { color: #ffffff } } }
            }
        }
    }
}

#[derive(Live, Widget)] 
pub struct EditorArea{
    #[redraw] #[live] draw_bg: DrawColor,
    #[live] scroll_bars: ScrollBars,
    
    #[live] draw_text_reg: DrawText,
    #[live] draw_text_bold: DrawText,
    #[live] draw_text_italic: DrawText,
    #[live] draw_text_code: DrawText,
    #[live] draw_text_header1: DrawText,
    #[live] draw_text_header2: DrawText,
    #[live] draw_text_quote: DrawText,
    
    #[live] draw_cursor: DrawColor,
    #[live] draw_selection: DrawColor,
    
    #[animator] animator: Animator,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[area] area: Area,
    
    #[rust] document: Document,
    #[rust] cursor_block: usize,
    #[rust] cursor_char: usize,
    #[rust] selection_anchor: Option<(usize, usize)>,
    
    #[rust] blink_timer: Timer,
    #[rust] is_dragging: bool,
    #[rust] deferred_finger_tap: Option<DVec2>,
    
    #[rust] block_y_offsets: Vec<f64>,
    #[rust] last_rendered_width: f64,
}

impl LiveHook for EditorArea{
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.document = Document::new();
        self.cursor_block = 0;
        self.cursor_char = 0;
        self.blink_timer = cx.start_timeout(0.5);
    }
}

impl EditorArea {
    fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some(anchor) = self.selection_anchor {
            let cursor = (self.cursor_block, self.cursor_char);
            if anchor < cursor { Some((anchor, cursor)) } else { Some((cursor, anchor)) }
        } else { None }
    }

    fn reset_blink(&mut self, cx: &mut Cx) {
         self.animator_play(cx, ids!(blink.on));
         cx.stop_timer(self.blink_timer);
         self.blink_timer = cx.start_timeout(0.5);
    }

    fn find_prev_word(&self) -> usize {
        if self.cursor_char == 0 { return 0; }
        let block = &self.document.blocks[self.cursor_block];
        let text = block.full_text();
        let chars: Vec<char> = text.chars().collect();
        let mut i = self.cursor_char;
        while i > 0 && chars[i-1].is_whitespace() { i -= 1; }
        while i > 0 && !chars[i-1].is_whitespace() { i -= 1; }
        i
    }

    fn find_next_word(&self) -> usize {
        let block = &self.document.blocks[self.cursor_block];
        let text = block.full_text();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = self.cursor_char;
        while i < len && chars[i].is_whitespace() { i += 1; }
        while i < len && !chars[i].is_whitespace() { i += 1; }
        i
    }
    
    fn invalidate_layout(&mut self) {
        self.block_y_offsets.clear();
    }
}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.blink_timer.is_event(event).is_some() {
            if self.animator_in_state(cx, ids!(blink.off)) {
                self.animator_play(cx, ids!(blink.on));
            } else {
                self.animator_play(cx, ids!(blink.off));
            }
            self.blink_timer = cx.start_timeout(0.5);
        }
        self.animator_handle_event(cx, event);
        self.scroll_bars.handle_event(cx, event, scope);

        match event.hits(cx, self.area) {
            Hit::FingerDown(fe) => {
                cx.set_key_focus(self.area);
                self.reset_blink(cx);
                self.is_dragging = true;
                self.deferred_finger_tap = Some(fe.abs);
                self.selection_anchor = None;
                self.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                if self.is_dragging {
                    self.deferred_finger_tap = Some(fe.abs);
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(_) => {
                self.is_dragging = false;
                if let Some(anchor) = self.selection_anchor {
                    if anchor == (self.cursor_block, self.cursor_char) {
                        self.selection_anchor = None;
                    }
                }
                self.deferred_finger_tap = None;
                self.redraw(cx);
            }
            
            Hit::KeyDown(ke) => {
                self.reset_blink(cx);
                let shift = ke.modifiers.shift;
                let ctrl = ke.modifiers.control || ke.modifiers.logo;
                
                if ctrl && ke.key_code == KeyCode::KeyS {
                    let doc_snapshot = self.document.snapshot();
                    std::thread::spawn(move || {
                        match doc_snapshot.save_to_file("story.md") {
                            Ok(_) => makepad_widgets::log!("Async Save: Document saved to story.md"),
                            Err(e) => makepad_widgets::log!("Async Save Error: {}", e),
                        }
                    });
                    return;
                }
                
                if shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some((self.cursor_block, self.cursor_char));
                    }
                } else if !ctrl { 
                    match ke.key_code {
                        KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                             self.selection_anchor = None;
                        }
                        _ => {}
                    }
                }

                match ke.key_code {
                    KeyCode::ArrowUp => {
                        if self.cursor_block > 0 {
                            self.cursor_block -= 1;
                            let len = self.document.blocks[self.cursor_block].text_len();
                            self.cursor_char = self.cursor_char.min(len);
                        }
                    }
                    KeyCode::ArrowDown => {
                        if self.cursor_block < self.document.blocks.len() - 1 {
                            self.cursor_block += 1;
                            let len = self.document.blocks[self.cursor_block].text_len();
                            self.cursor_char = self.cursor_char.min(len);
                        }
                    }
                    KeyCode::ArrowLeft => {
                        if ctrl {
                            self.cursor_char = self.find_prev_word();
                        } else if self.cursor_char > 0 {
                            self.cursor_char -= 1;
                        } else if self.cursor_block > 0 {
                            self.cursor_block -= 1;
                            self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                        }
                    }
                    KeyCode::ArrowRight => {
                        let len = self.document.blocks[self.cursor_block].text_len();
                        if ctrl {
                            self.cursor_char = self.find_next_word();
                        } else if self.cursor_char < len {
                            self.cursor_char += 1;
                        } else if self.cursor_block < self.document.blocks.len() - 1 {
                            self.cursor_block += 1;
                            self.cursor_char = 0;
                        }
                    }
                    KeyCode::ReturnKey => {
                        self.selection_anchor = None;
                        let new_block = Block::new(
                            self.document.generate_id(),
                            BlockType::Paragraph,
                            ""
                        );
                        self.document.blocks.insert(self.cursor_block + 1, new_block);
                        self.cursor_block += 1;
                        self.cursor_char = 0;
                        self.invalidate_layout();
                    }
                    KeyCode::Delete => {
                        if ctrl {
                             let end = self.find_next_word();
                             if end > self.cursor_char {
                                 let range = ((self.cursor_block, self.cursor_char), (self.cursor_block, end));
                                 self.document.delete_range(range.0, range.1);
                                 self.invalidate_layout();
                             }
                        } else if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.selection_anchor = None;
                                self.invalidate_layout();
                            }
                        } else {
                            let block = &mut self.document.blocks[self.cursor_block];
                            if self.cursor_char < block.text_len() {
                                self.document.remove_char_at(self.cursor_block, self.cursor_char);
                            } else if self.cursor_block < self.document.blocks.len() - 1 
                                && self.document.merge_block_with_prev(self.cursor_block + 1).is_some() {
                                    self.invalidate_layout();
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if ctrl {
                            let start = self.find_prev_word();
                            if start < self.cursor_char {
                                let range = ((self.cursor_block, start), (self.cursor_block, self.cursor_char));
                                let new_cursor = self.document.delete_range(range.0, range.1);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.invalidate_layout();
                            }
                        } else if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.selection_anchor = None;
                                self.invalidate_layout();
                            } else {
                                self.selection_anchor = None; 
                                if self.cursor_char > 0 {
                                    if self.document.remove_char_at(self.cursor_block, self.cursor_char - 1) {
                                        self.cursor_char -= 1;
                                    }
                                } else if self.cursor_block > 0 {
                                    let current_type = self.document.blocks[self.cursor_block].ty.clone();
                                    if current_type != BlockType::Paragraph {
                                        self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                                    } else if let Some(new_char_pos) = self.document.merge_block_with_prev(self.cursor_block) {
                                        self.cursor_block -= 1;
                                        self.cursor_char = new_char_pos;
                                        self.invalidate_layout();
                                    }
                                } else {
                                    let current_type = self.document.blocks[self.cursor_block].ty.clone();
                                    if current_type != BlockType::Paragraph {
                                        self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                                    }
                                }
                            }
                        } else if self.cursor_char > 0 {
                            if self.document.remove_char_at(self.cursor_block, self.cursor_char - 1) {
                                self.cursor_char -= 1;
                            }
                        } else if self.cursor_block > 0 {
                            let current_type = self.document.blocks[self.cursor_block].ty.clone();
                            if current_type != BlockType::Paragraph {
                                self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                            } else if let Some(new_char_pos) = self.document.merge_block_with_prev(self.cursor_block) {
                                self.cursor_block -= 1;
                                self.cursor_char = new_char_pos;
                                self.invalidate_layout();
                            }
                        } else {
                            let current_type = self.document.blocks[self.cursor_block].ty.clone();
                            if current_type != BlockType::Paragraph {
                                self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                            }
                        }
                    }
                    _ => {}
                }
                self.redraw(cx);
            }
            Hit::TextInput(te) => {
                if !te.input.is_empty() && !te.input.chars().any(|c| c.is_control()) {
                    let mut wrapped = false;
                    if let Some(((start_blk, start_char), (end_blk, end_char))) = self.get_selection_range() {
                        if start_blk == end_blk && start_blk == self.cursor_block && start_char != end_char
                            && (te.input == "*" || te.input == "`" || te.input == "_") {
                                self.document.wrap_selection(start_blk, start_char, end_char, &te.input);
                                self.selection_anchor = Some((start_blk, start_char + 1));
                                self.cursor_char = end_char + 1; 
                                wrapped = true;
                        }
                    }

                    if !wrapped {
                        if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.invalidate_layout();
                            }
                            self.selection_anchor = None;
                        }
                        
                        let added = self.document.insert_text_at(self.cursor_block, self.cursor_char, &te.input);
                        self.cursor_char += added;
                        
                        if let Some(removed_chars) = self.document.try_convert_block(self.cursor_block) {
                            self.cursor_char = self.cursor_char.saturating_sub(removed_chars);
                        }
                        
                        if te.input == " " && self.document.apply_inline_formatting(self.cursor_block) {
                            self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                        }
                    }
                    self.redraw(cx);
                }
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        self.scroll_bars.begin(cx, walk, self.layout);
        let rect = cx.turtle().rect();
        let scroll = cx.turtle().scroll(); 
        
        if rect.size.x != self.last_rendered_width {
            self.invalidate_layout();
            self.last_rendered_width = rect.size.x;
        }
        
        let selection = self.get_selection_range();
        let mut view = EditorView {
            draw_bg: &mut self.draw_bg,
            draw_text_reg: &mut self.draw_text_reg,
            draw_text_bold: &mut self.draw_text_bold,
            draw_text_italic: &mut self.draw_text_italic,
            draw_text_code: &mut self.draw_text_code,
            draw_text_header1: &mut self.draw_text_header1,
            draw_text_header2: &mut self.draw_text_header2,
            draw_text_quote: &mut self.draw_text_quote,
            draw_cursor: &mut self.draw_cursor,
            draw_selection: &mut self.draw_selection,
        };
        
        let is_cache_valid = !self.block_y_offsets.is_empty() && self.block_y_offsets.len() == self.document.blocks.len();
        if !is_cache_valid { self.block_y_offsets.clear(); }
        
        let (used_height, hit_res) = view.draw_document(
            cx, 
            DrawParams {
                doc: &mut self.document,
                layout: &self.layout,
                rect,
                cursor: (self.cursor_block, self.cursor_char),
                selection,
                finger_hit: self.deferred_finger_tap,
                scroll,
                y_offsets_cache: &mut self.block_y_offsets,
            }
        );
        
        if let Some(hit) = hit_res {
            self.cursor_block = hit.block_idx;
            self.cursor_char = hit.char_idx;
            if self.is_dragging && self.selection_anchor.is_none() {
                 self.selection_anchor = Some((self.cursor_block, self.cursor_char));
            }
        }
        
        self.scroll_bars.end(cx);
        cx.turtle_mut().set_used(rect.size.x, used_height);
        cx.end_turtle_with_area(&mut self.area);
        
        self.deferred_finger_tap = None;
        DrawStep::done()
    }
}