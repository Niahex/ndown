use makepad_widgets::*;
use crate::markdown::inline::{parse_inline_formatting, InlineFormat};

live_design! {
    RichTextEditorBase = {{RichTextEditor}} {
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
        
        draw_cursor: {
            fn pixel(self) -> vec4 {
                return #ffffff;
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct RichTextEditor {
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[rust] area: Area,
    
    #[live] draw_bg: DrawQuad,
    #[live] draw_text: DrawText,
    #[live] draw_text_bold: DrawText,
    #[live] draw_text_italic: DrawText,
    #[live] draw_text_code: DrawText,
    #[live] draw_cursor: DrawQuad,
    
    #[rust] text: String,
    #[rust] cursor_pos: usize,
    #[rust] is_focused: bool,
    #[rust] hover: f32,
    #[rust] text_positions: Vec<(usize, f64)>, // (char_index, x_position)
}

impl LiveHook for RichTextEditor {
    fn after_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        self.draw_bg.redraw(cx);
    }
}

impl Widget for RichTextEditor {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::ArrowLeft => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            cx.redraw_all();
                        }
                    }
                    KeyCode::ArrowRight => {
                        if self.cursor_pos < self.text.len() {
                            self.cursor_pos += 1;
                            cx.redraw_all();
                        }
                    }
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            self.text.remove(self.cursor_pos - 1);
                            self.cursor_pos -= 1;
                            cx.redraw_all();
                        }
                    }
                    _ => {}
                }
            }
            Event::TextInput(ti) => {
                self.text.insert_str(self.cursor_pos, &ti.input);
                self.cursor_pos += ti.input.len();
                cx.redraw_all();
            }
            Event::MouseDown(me) => {
                if self.area.rect(cx).contains(me.abs) {
                    self.is_focused = true;
                    cx.set_key_focus(self.area);
                    
                    // Calculate cursor position from click
                    let click_x = me.abs.x - self.area.rect(cx).pos.x - 10.0; // Account for padding
                    self.cursor_pos = self.find_cursor_position_from_x(click_x);
                    
                    cx.redraw_all();
                }
            }
            Event::MouseMove(me) => {
                let was_hover = self.hover > 0.5;
                let is_hover = self.area.rect(cx).contains(me.abs);
                if was_hover != is_hover {
                    self.hover = if is_hover { 1.0 } else { 0.0 };
                    cx.redraw_all();
                }
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        // Draw background
        self.draw_bg.draw_vars.set_uniform(cx, &[id!(hover)], &[self.hover]);
        self.draw_bg.draw_walk(cx, Walk::fill());
        
        // Parse and render rich text
        self.render_rich_text(cx);
        
        // Draw cursor if focused
        if self.is_focused {
            self.draw_cursor_at_position(cx);
        }
        
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl RichTextEditor {
    pub fn text(&self) -> &str {
        &self.text
    }
    
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_pos = self.cursor_pos.min(self.text.len());
    }
    
    fn render_rich_text(&mut self, cx: &mut Cx2d) {
        self.text_positions.clear();
        let spans = parse_inline_formatting(&self.text);
        let mut last_end = 0;
        let mut current_x = 0.0;
        
        // Clone text to avoid borrow issues
        let text_clone = self.text.clone();
        
        for span in spans {
            // Render text before this span
            if span.range.start > last_end {
                let plain_text = &text_clone[last_end..span.range.start];
                current_x = self.render_text_segment(cx, plain_text, current_x, last_end);
            }
            
            // Render the formatted span
            let (content, is_bold, is_italic, is_code) = match span.format {
                InlineFormat::Bold => {
                    let content = &text_clone[span.range.start + 2..span.range.end - 2];
                    (content, true, false, false)
                }
                InlineFormat::Italic => {
                    let content = &text_clone[span.range.start + 1..span.range.end - 1];
                    (content, false, true, false)
                }
                InlineFormat::Code => {
                    let content = &text_clone[span.range.start + 1..span.range.end - 1];
                    (content, false, false, true)
                }
            };
            
            current_x = self.render_formatted_segment(cx, content, current_x, span.range.start, is_bold, is_italic, is_code);
            last_end = span.range.end;
        }
        
        // Render remaining text
        if last_end < text_clone.len() {
            self.render_text_segment(cx, &text_clone[last_end..], current_x, last_end);
        }
    }
    
    fn render_text_segment(&mut self, cx: &mut Cx2d, text: &str, start_x: f64, char_offset: usize) -> f64 {
        let mut x = start_x;
        
        // Record position for each character
        for (i, _) in text.char_indices() {
            self.text_positions.push((char_offset + i, x));
            x += 8.0; // Rough estimate for 14px font
        }
        
        // Draw the text
        self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), text);
        x
    }
    
    fn render_formatted_segment(&mut self, cx: &mut Cx2d, text: &str, start_x: f64, char_offset: usize, is_bold: bool, is_italic: bool, is_code: bool) -> f64 {
        let mut x = start_x;
        
        // Record position for each character
        for (i, _) in text.char_indices() {
            self.text_positions.push((char_offset + i, x));
            x += 8.0;
        }
        
        // Draw with appropriate style
        if is_bold {
            self.draw_text_bold.draw_walk(cx, Walk::fit(), Align::default(), text);
        } else if is_italic {
            self.draw_text_italic.draw_walk(cx, Walk::fit(), Align::default(), text);
        } else if is_code {
            self.draw_text_code.draw_walk(cx, Walk::fit(), Align::default(), text);
        }
        
        x
    }
    
    fn find_cursor_position_from_x(&self, click_x: f64) -> usize {
        if self.text_positions.is_empty() {
            return 0;
        }
        
        // Find closest character position
        let mut best_pos = 0;
        let mut best_distance = f64::MAX;
        
        for &(char_idx, x_pos) in &self.text_positions {
            let distance = (click_x - x_pos).abs();
            if distance < best_distance {
                best_distance = distance;
                best_pos = char_idx;
            }
        }
        
        // Check if click is after the last character
        if let Some(&(_, last_x)) = self.text_positions.last() {
            if click_x > last_x + 4.0 { // Half character width
                return self.text.len();
            }
        }
        
        best_pos.min(self.text.len())
    }
    
    fn draw_cursor_at_position(&mut self, cx: &mut Cx2d) {
        let cursor_x = if let Some(&(_, x_pos)) = self.text_positions.iter().find(|(idx, _)| *idx == self.cursor_pos) {
            x_pos
        } else if self.cursor_pos >= self.text.len() && !self.text_positions.is_empty() {
            // Cursor at end
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
}
