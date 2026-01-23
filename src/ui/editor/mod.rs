use makepad_widgets::*;

live_design!{
    use link::theme::*;
    use link::widgets::*;
    
    pub EditorArea = {{EditorArea}}{
        width: Fill, height: Fill
        show_bg: true
        draw_bg: { color: #2e3440 }
        draw_text: {
            text_style: <THEME_FONT_CODE> {}
            color: #eceff4
        }
        draw_cursor: {
            color: (THEME_COLOR_U_1)
        }
    }
}
 
#[derive(Live, Widget)] 
pub struct EditorArea{
    #[redraw] #[live] draw_bg: DrawColor,
    #[live] draw_text: DrawText,
    #[live] draw_cursor: DrawColor,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[area] area: Area,
    
    #[rust] lines: Vec<String>,
    #[rust] cursor_line: usize,
    #[rust] cursor_col: usize,
}

impl LiveHook for EditorArea{
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.lines = vec!["# Hello Markdown".to_string(), String::new(), "Type here...".to_string()];
        self.cursor_line = 0;
        self.cursor_col = 0;
    }
}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        match event {
            Event::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::ReturnKey => {
                        let current = &self.lines[self.cursor_line];
                        let after = current[self.cursor_col..].to_string();
                        self.lines[self.cursor_line].truncate(self.cursor_col);
                        self.lines.insert(self.cursor_line + 1, after);
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                        self.redraw(cx);
                    }
                    KeyCode::Backspace => {
                        if self.cursor_col > 0 {
                            self.lines[self.cursor_line].remove(self.cursor_col - 1);
                            self.cursor_col -= 1;
                        } else if self.cursor_line > 0 {
                            let line = self.lines.remove(self.cursor_line);
                            self.cursor_line -= 1;
                            self.cursor_col = self.lines[self.cursor_line].len();
                            self.lines[self.cursor_line].push_str(&line);
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowLeft => {
                        if self.cursor_col > 0 {
                            self.cursor_col -= 1;
                        } else if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            self.cursor_col = self.lines[self.cursor_line].len();
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowRight => {
                        if self.cursor_col < self.lines[self.cursor_line].len() {
                            self.cursor_col += 1;
                        } else if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            self.cursor_col = 0;
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowUp => {
                        if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowDown => {
                        if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                        }
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
            Event::TextInput(te) => {
                self.lines[self.cursor_line].insert_str(self.cursor_col, &te.input);
                self.cursor_col += te.input.len();
                self.redraw(cx);
            }
            _ => {}
        }
        
        match event.hits(cx, self.area) {
            Hit::FingerDown(_) => {
                cx.set_key_focus(self.area);
                self.redraw(cx);
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        let rect = cx.turtle().rect();
        self.draw_bg.draw_abs(cx, rect);
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            let y_pos = cx.turtle().pos().y;
            let x_pos = cx.turtle().pos().x;
            
            self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), line);
            
            if line_idx == self.cursor_line {
                let char_width = 9.2; // Approximate width for monospace font
                let cursor_x = x_pos + (self.cursor_col as f64 * char_width);
                self.draw_cursor.draw_abs(cx, Rect {
                    pos: dvec2(cursor_x, y_pos),
                    size: dvec2(2.0, 16.0),
                });
            }
            
            cx.turtle_new_line();
        }
        
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}
