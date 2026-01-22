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
            Event::MouseDown(_) => {
                self.is_focused = true;
                cx.set_key_focus(self.area);
                cx.redraw_all();
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
        let spans = parse_inline_formatting(&self.text);
        let mut last_end = 0;
        
        for span in spans {
            // Render text before this span
            if span.range.start > last_end {
                let plain_text = &self.text[last_end..span.range.start];
                self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), plain_text);
            }
            
            // Render the formatted span
            match span.format {
                InlineFormat::Bold => {
                    let content = &self.text[span.range.start + 2..span.range.end - 2];
                    self.draw_text_bold.draw_walk(cx, Walk::fit(), Align::default(), content);
                }
                InlineFormat::Italic => {
                    let content = &self.text[span.range.start + 1..span.range.end - 1];
                    self.draw_text_italic.draw_walk(cx, Walk::fit(), Align::default(), content);
                }
                InlineFormat::Code => {
                    let content = &self.text[span.range.start + 1..span.range.end - 1];
                    self.draw_text_code.draw_walk(cx, Walk::fit(), Align::default(), content);
                }
            }
            
            last_end = span.range.end;
        }
        
        // Render remaining text
        if last_end < self.text.len() {
            self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), &self.text[last_end..]);
        }
    }
    
    fn draw_cursor_at_position(&mut self, cx: &mut Cx2d) {
        // Simple cursor at end for now
        let cursor_walk = Walk {
            width: Size::Fixed(2.0),
            height: Size::Fixed(20.0),
            ..Walk::default()
        };
        self.draw_cursor.draw_walk(cx, cursor_walk);
    }
}
