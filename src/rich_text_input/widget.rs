use makepad_widgets::*;
use crate::markdown::inline::InlineFormat;
use crate::rich_text_input::{
    cursor::CursorManager, 
    events::EventManager, 
    history::HistoryManager, 
    types::*, 
    formatting::FormattingManager,
    text_mapping::TextMapping
};

live_design! {
    RichTextInputBase = {{RichTextInput}} {
        width: Fill, height: Fit
        padding: 10
        
        draw_bg: {
            fn pixel(self) -> vec4 {
                return mix(#2a2a2a, #3a3a3a, self.hover);
            }
        }
        
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
        
        draw_text_bold: {
            text_style: <THEME_FONT_BOLD> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
        
        draw_text_italic: {
            text_style: <THEME_FONT_ITALIC> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
        
        draw_text_code: {
            text_style: <THEME_FONT_CODE> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #88c0d0;
            }
        }
        
        draw_text_link: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #5e81ac;
            }
        }
        
        draw_text_wiki: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #a3be8c;
            }
        }
        
        draw_cursor: {
            fn pixel(self) -> vec4 {
                return #ffffff;
            }
        }
        
        draw_selection: {
            fn pixel(self) -> vec4 {
                return #x3b82f6;
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct RichTextInput {
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[rust] area: Area,
    
    #[live] draw_bg: DrawQuad,
    #[live] draw_text: DrawText,
    #[live] draw_text_bold: DrawText,
    #[live] draw_text_italic: DrawText,
    #[live] draw_text_code: DrawText,
    #[live] draw_text_link: DrawText,
    #[live] draw_text_wiki: DrawText,
    #[live] draw_cursor: DrawQuad,
    #[live] draw_selection: DrawQuad,
    
    #[rust] text: String,
    #[rust] is_focused: bool,
    #[rust] hover: f32,
    #[rust] text_positions: Vec<(usize, f64)>,
    #[rust] is_dragging: bool,
    #[rust] clipboard_content: Option<String>,
    #[rust] text_mapping: TextMapping,
    
    // Modular components
    #[rust] cursor: CursorManager,
    #[rust] events: EventManager,
    #[rust] history: HistoryManager,
}

impl LiveHook for RichTextInput {
    fn after_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        self.draw_bg.redraw(cx);
    }
}

impl Widget for RichTextInput {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Handle clipboard events
        match event {
            Event::TextCopy(tc) => {
                if let Some(selected) = self.cursor.get_selected_text(&self.text) {
                    *tc.response.borrow_mut() = Some(selected.clone());
                    self.clipboard_content = Some(selected);
                }
                return;
            }
            Event::TextCut(tc) => {
                if let Some(selected) = self.cursor.get_selected_text(&self.text) {
                    *tc.response.borrow_mut() = Some(selected.clone());
                    self.clipboard_content = Some(selected);
                    self.delete_selection();
                    cx.redraw_all();
                }
                return;
            }
            _ => {}
        }
        
        match event {
            Event::KeyDown(ke) => {
                self.handle_key_down(cx, ke);
            }
            Event::KeyUp(_ke) => {
                self.events.stop_key_repeat(cx);
            }
            Event::TextInput(ti) => {
                self.handle_text_input(cx, &ti.input);
            }
            Event::MouseDown(me) => {
                self.handle_mouse_down(cx, me);
            }
            Event::MouseMove(me) => {
                self.handle_mouse_move(cx, me);
            }
            Event::Timer(te) => {
                self.handle_timer(cx, te);
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        // Draw background
        self.draw_bg.draw_vars.set_uniform(cx, &[id!(hover)], &[self.hover]);
        self.draw_bg.draw_walk(cx, Walk::fill());
        
        // Draw selection background
        if let Some(selection) = self.cursor.selection.clone() {
            self.draw_selection_background(cx, &selection);
        }
        
        // Render rich text
        self.render_rich_text(cx);
        
        // Draw cursor
        if self.is_focused {
            self.draw_cursor_at_position(cx);
        }
        
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl RichTextInput {
    // Public API
    pub fn text(&self) -> &str {
        &self.text
    }
    
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.update_mapping();
        self.cursor.position.char_index = self.cursor.position.char_index.min(self.text.len());
        self.cursor.clear_selection();
    }
    
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        self.save_undo_state();
        self.delete_selection();
        self.text.insert_str(self.cursor.position.char_index, text);
        self.cursor.position.char_index += text.len();
        self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
        self.update_mapping();
    }
    
    fn update_mapping(&mut self) {
        self.text_mapping = TextMapping::from_text(&self.text);
    }
    
    // Event handlers
    fn handle_key_down(&mut self, cx: &mut Cx, ke: &KeyEvent) {
        match ke.key_code {
            KeyCode::ArrowLeft => {
                if ke.modifiers.shift && self.cursor.selection.is_none() {
                    self.cursor.start_selection(self.cursor.position.clone());
                } else if !ke.modifiers.shift {
                    self.cursor.clear_selection();
                }
                
                if self.cursor.position.char_index > 0 {
                    self.cursor.position.char_index -= 1;
                    self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                    
                    if ke.modifiers.shift {
                        self.cursor.update_selection(self.cursor.position.clone());
                    }
                }
                cx.redraw_all();
            }
            KeyCode::ArrowRight => {
                if ke.modifiers.shift && self.cursor.selection.is_none() {
                    self.cursor.start_selection(self.cursor.position.clone());
                } else if !ke.modifiers.shift {
                    self.cursor.clear_selection();
                }
                
                if self.cursor.position.char_index < self.text.len() {
                    self.cursor.position.char_index += 1;
                    self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                    
                    if ke.modifiers.shift {
                        self.cursor.update_selection(self.cursor.position.clone());
                    }
                }
                cx.redraw_all();
            }
            KeyCode::ReturnKey => {
                self.save_undo_state();
                self.delete_selection();
                
                if ke.modifiers.shift {
                    self.text.insert(self.cursor.position.char_index, '\n');
                    self.cursor.position.char_index += 1;
                } else {
                    self.text.insert(self.cursor.position.char_index, '\n');
                    self.cursor.position.char_index += 1;
                }
                self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                cx.redraw_all();
            }
            KeyCode::Backspace => {
                self.save_undo_state();
                self.perform_backspace();
                self.events.start_key_repeat(cx, KeyCode::Backspace);
                cx.redraw_all();
            }
            KeyCode::Delete => {
                self.save_undo_state();
                self.perform_delete();
                self.events.start_key_repeat(cx, KeyCode::Delete);
                cx.redraw_all();
            }
            KeyCode::KeyB if ke.modifiers.control => {
                self.save_undo_state();
                self.apply_formatting_toggle("**");
                cx.redraw_all();
            }
            KeyCode::KeyI if ke.modifiers.control => {
                self.save_undo_state();
                self.apply_formatting_toggle("*");
                cx.redraw_all();
            }
            KeyCode::KeyC if ke.modifiers.control => {
                if let Some(selected) = self.cursor.get_selected_text(&self.text) {
                    cx.copy_to_clipboard(&selected);
                }
            }
            KeyCode::KeyX if ke.modifiers.control => {
                if let Some(selected) = self.cursor.get_selected_text(&self.text) {
                    cx.copy_to_clipboard(&selected);
                    self.delete_selection();
                    cx.redraw_all();
                }
            }
            KeyCode::KeyV if ke.modifiers.control => {
                if let Some(clipboard_text) = &self.clipboard_content.clone() {
                    self.paste_text(clipboard_text);
                    cx.redraw_all();
                }
            }
            KeyCode::KeyZ if ke.modifiers.control && !ke.modifiers.shift => {
                self.undo();
                cx.redraw_all();
            }
            KeyCode::KeyZ if ke.modifiers.control && ke.modifiers.shift => {
                self.redo();
                cx.redraw_all();
            }
            KeyCode::KeyY if ke.modifiers.control => {
                self.redo();
                cx.redraw_all();
            }
            KeyCode::KeyA if ke.modifiers.control => {
                self.cursor.start_selection(CursorPosition { line: 0, column: 0, char_index: 0 });
                self.cursor.position.char_index = self.text.len();
                self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                self.cursor.update_selection(self.cursor.position.clone());
                cx.redraw_all();
            }
            _ => {}
        }
    }
    
    fn handle_text_input(&mut self, cx: &mut Cx, input: &str) {
        self.save_undo_state();
        self.delete_selection();
        self.text.insert_str(self.cursor.position.char_index, input);
        self.cursor.position.char_index += input.len();
        self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
        self.update_mapping();
        cx.redraw_all();
    }
    
    fn handle_mouse_down(&mut self, cx: &mut Cx, me: &MouseDownEvent) {
        if self.area.rect(cx).contains(me.abs) {
            self.is_focused = true;
            cx.set_key_focus(self.area);
            
            let click_x = me.abs.x - self.area.rect(cx).pos.x - 10.0;
            let new_cursor_pos = self.find_cursor_position_from_x(click_x);
            
            if self.events.is_double_click(me.abs, me.time) {
                self.select_word_at_position(new_cursor_pos);
            } else {
                self.cursor.position.char_index = new_cursor_pos;
                self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                self.cursor.clear_selection();
                self.is_dragging = true;
            }
            
            cx.redraw_all();
        }
    }
    
    fn handle_mouse_move(&mut self, cx: &mut Cx, me: &MouseMoveEvent) {
        let rect = self.area.rect(cx);
        let was_hover = self.hover > 0.5;
        let is_hover = rect.contains(me.abs);
        
        if was_hover != is_hover {
            self.hover = if is_hover { 1.0 } else { 0.0 };
            cx.redraw_all();
        }
        
        if self.is_dragging && rect.contains(me.abs) {
            let click_x = me.abs.x - rect.pos.x - 10.0;
            let new_pos = self.find_cursor_position_from_x(click_x);
            
            if self.cursor.selection.is_none() {
                self.cursor.start_selection(self.cursor.position.clone());
            }
            
            self.cursor.position.char_index = new_pos;
            self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
            self.cursor.update_selection(self.cursor.position.clone());
            cx.redraw_all();
        }
    }
    
    fn handle_timer(&mut self, cx: &mut Cx, te: &TimerEvent) {
        if te.timer_id == self.events.repeat_timer.0 {
            if let Some(key) = self.events.repeat_key {
                match key {
                    KeyCode::Backspace => self.perform_backspace(),
                    KeyCode::Delete => self.perform_delete(),
                    _ => {}
                }
                self.events.repeat_timer = cx.start_timeout(0.05);
                cx.redraw_all();
            }
        }
    }
    
    // Helper methods
    fn save_undo_state(&mut self) {
        let entry = HistoryEntry::new(
            self.text.clone(),
            self.cursor.position.clone(),
            self.cursor.selection.clone(),
        );
        self.history.push(entry);
    }
    
    fn undo(&mut self) {
        if let Some(entry) = self.history.undo() {
            self.text = entry.text.clone();
            self.cursor.position = entry.cursor.clone();
            self.cursor.selection = entry.selection.clone();
        }
    }
    
    fn redo(&mut self) {
        if let Some(entry) = self.history.redo() {
            self.text = entry.text.clone();
            self.cursor.position = entry.cursor.clone();
            self.cursor.selection = entry.selection.clone();
        }
    }
    
    fn delete_selection(&mut self) {
        if let Some(selection) = &self.cursor.selection {
            let (start, end) = selection.get_range();
            if start != end {
                self.text.drain(start..end);
                self.cursor.position.char_index = start;
                self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
                self.update_mapping();
            }
            self.cursor.clear_selection();
        }
    }
    
    fn paste_text(&mut self, text: &str) {
        self.save_undo_state();
        self.delete_selection();
        self.text.insert_str(self.cursor.position.char_index, text);
        self.cursor.position.char_index += text.len();
        self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
        self.update_mapping();
    }
    
    fn perform_backspace(&mut self) {
        if self.cursor.selection.is_some() {
            self.delete_selection();
        } else if self.cursor.position.char_index > 0 {
            self.text.remove(self.cursor.position.char_index - 1);
            self.cursor.position.char_index -= 1;
            self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
        }
        self.update_mapping();
    }
    
    fn perform_delete(&mut self) {
        if self.cursor.selection.is_some() {
            self.delete_selection();
        } else if self.cursor.position.char_index < self.text.len() {
            self.text.remove(self.cursor.position.char_index);
        }
        self.update_mapping();
    }
    
    fn select_word_at_position(&mut self, pos: usize) {
        let chars: Vec<char> = self.text.chars().collect();
        if chars.is_empty() || pos >= chars.len() {
            return;
        }
        
        let mut start = pos;
        let mut end = pos;
        
        while start > 0 && chars[start - 1].is_alphanumeric() {
            start -= 1;
        }
        
        while end < chars.len() && chars[end].is_alphanumeric() {
            end += 1;
        }
        
        if start != end {
            let start_pos = self.cursor.char_index_to_position(&self.text, start);
            let end_pos = self.cursor.char_index_to_position(&self.text, end);
            self.cursor.selection = Some(TextSelection::new(start_pos, end_pos.clone()));
            self.cursor.position = end_pos;
        }
    }
    
    fn render_rich_text(&mut self, cx: &mut Cx2d) {
        self.text_positions.clear();
        let mut current_x = 0.0;
        
        for segment in &self.text_mapping.segments {
            let visual_text = &self.text_mapping.visual_text[segment.visual_start..segment.visual_end];
            
            // Store positions for visual text (for cursor positioning)
            for (i, _) in visual_text.char_indices() {
                self.text_positions.push((segment.visual_start + i, current_x));
                current_x += 8.0;
            }
            
            // Render with appropriate style
            match &segment.format {
                None => {
                    self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::Bold) => {
                    self.draw_text_bold.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::Italic) => {
                    self.draw_text_italic.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::Code) => {
                    self.draw_text_code.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::Link { .. }) => {
                    self.draw_text_link.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::WikiLink { .. }) => {
                    self.draw_text_wiki.draw_walk(cx, Walk::fit(), Align::default(), visual_text);
                }
                Some(InlineFormat::Image { .. }) => {
                    let display = format!("[IMG: {}]", visual_text);
                    self.draw_text_code.draw_walk(cx, Walk::fit(), Align::default(), &display);
                }
            }
        }
    }
    
    fn find_cursor_position_from_x(&self, click_x: f64) -> usize {
        if self.text_positions.is_empty() {
            return 0;
        }
        
        let mut best_pos = 0;
        let mut best_distance = f64::MAX;
        
        for &(char_idx, x_pos) in &self.text_positions {
            let distance = (click_x - x_pos).abs();
            if distance < best_distance {
                best_distance = distance;
                best_pos = char_idx;
            }
        }
        
        if let Some(&(_, last_x)) = self.text_positions.last() {
            if click_x > last_x + 4.0 {
                return self.text.len();
            }
        }
        
        best_pos.min(self.text.len())
    }
    
    fn draw_selection_background(&mut self, cx: &mut Cx2d, selection: &TextSelection) {
        let (start, end) = selection.get_range();
        if start == end {
            return;
        }
        
        let start_x = self.text_positions.iter()
            .find(|(idx, _)| *idx == start)
            .map(|(_, x)| *x)
            .unwrap_or(0.0);
            
        let end_x = self.text_positions.iter()
            .find(|(idx, _)| *idx == end)
            .map(|(_, x)| *x + 8.0)
            .unwrap_or(start_x + 8.0);
        
        let selection_walk = Walk {
            width: Size::Fixed((end_x - start_x).max(1.0)),
            height: Size::Fixed(20.0),
            margin: Margin { left: start_x, ..Margin::default() },
            ..Walk::default()
        };
        
        self.draw_selection.draw_walk(cx, selection_walk);
    }
    
    fn draw_cursor_at_position(&mut self, cx: &mut Cx2d) {
        let cursor_x = if let Some(&(_, x_pos)) = self.text_positions.iter().find(|(idx, _)| *idx == self.cursor.position.char_index) {
            x_pos
        } else if self.cursor.position.char_index >= self.text.len() && !self.text_positions.is_empty() {
            self.text_positions.last().unwrap().1 + 8.0
        } else {
            0.0
        };
        
        let cursor_walk = Walk {
            width: Size::Fixed(2.0),
            height: Size::Fixed(20.0),
            margin: Margin { left: cursor_x, ..Margin::default() },
            ..Walk::default()
        };
        self.draw_cursor.draw_walk(cx, cursor_walk);
    }
    
    // Formatting methods
    fn apply_formatting_toggle(&mut self, marker: &str) {
        if let Some(selection) = &self.cursor.selection {
            // Wrap selection
            let (start, end) = selection.get_range();
            self.text = FormattingManager::wrap_selection(&self.text, start, end, marker);
            self.cursor.position.char_index = end + marker.len() * 2;
        } else {
            // Toggle at cursor
            let (new_text, new_cursor) = match marker {
                "**" => FormattingManager::toggle_bold(&self.text, self.cursor.position.char_index),
                "*" => FormattingManager::toggle_italic(&self.text, self.cursor.position.char_index),
                "`" => FormattingManager::toggle_code(&self.text, self.cursor.position.char_index),
                _ => (self.text.clone(), self.cursor.position.char_index),
            };
            self.text = new_text;
            self.cursor.position.char_index = new_cursor;
        }
        self.cursor.position = self.cursor.char_index_to_position(&self.text, self.cursor.position.char_index);
        self.cursor.clear_selection();
        self.update_mapping();
    }
}
