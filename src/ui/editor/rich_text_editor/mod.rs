use makepad_widgets::*;

live_design! {
    use link::shaders::*;
    use link::theme::*;
    use link::widgets::*;

    pub RichTextEditor = {{RichTextEditor}} {
        width: Fill, height: Fill
        draw_bg: { color: (THEME_COLOR_BG_CONTAINER) }
        draw_text: {
            text_style: <THEME_FONT_CODE> {}
        }
        draw_cursor: {
            color: (THEME_COLOR_U_1)
        }
    }
}

#[derive(Live, Widget)]
pub struct RichTextEditor {
    #[deref] #[live] view: View,
    #[live] draw_bg: DrawColor,
    #[live] draw_text: DrawText,
    #[live] draw_cursor: DrawColor,
    
    #[rust] lines: Vec<String>,
    #[rust] cursor_line: usize,
    #[rust] cursor_col: usize,
}

impl LiveHook for RichTextEditor {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
    }
}

impl Widget for RichTextEditor {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        
        match event.hits(cx, self.view.area()) {
            Hit::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::ReturnKey => {
                        let current = &self.lines[self.cursor_line];
                        let after = current[self.cursor_col..].to_string();
                        self.lines[self.cursor_line].truncate(self.cursor_col);
                        self.lines.insert(self.cursor_line + 1, after);
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                        self.view.redraw(cx);
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
                        self.view.redraw(cx);
                    }
                    KeyCode::ArrowLeft => {
                        if self.cursor_col > 0 {
                            self.cursor_col -= 1;
                        } else if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            self.cursor_col = self.lines[self.cursor_line].len();
                        }
                        self.view.redraw(cx);
                    }
                    KeyCode::ArrowRight => {
                        if self.cursor_col < self.lines[self.cursor_line].len() {
                            self.cursor_col += 1;
                        } else if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            self.cursor_col = 0;
                        }
                        self.view.redraw(cx);
                    }
                    KeyCode::ArrowUp => {
                        if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                        }
                        self.view.redraw(cx);
                    }
                    KeyCode::ArrowDown => {
                        if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                        }
                        self.view.redraw(cx);
                    }
                    _ => {}
                }
            }
            Hit::TextInput(te) => {
                self.lines[self.cursor_line].insert_str(self.cursor_col, &te.input);
                self.cursor_col += te.input.len();
                self.view.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, Layout::default());
        
        let mut y = 10.0;
        let x = 10.0;
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            self.draw_text.draw_walk(cx, Walk::default(), Align::default(), line);
            
            if line_idx == self.cursor_line {
                let cursor_x = x + (self.cursor_col as f64 * 8.0);
                self.draw_cursor.draw_abs(cx, Rect {
                    pos: dvec2(cursor_x, y),
                    size: dvec2(2.0, 16.0),
                });
            }
            
            y += 20.0;
        }
        
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum RichTextEditorAction {
    None,
    Change,
}

impl RichTextEditorRef {
    pub fn set_text(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.lines = text.lines().map(|s| s.to_string()).collect();
            if inner.lines.is_empty() {
                inner.lines.push(String::new());
            }
        }
    }
    
    pub fn get_text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.lines.join("\n")
        } else {
            String::new()
        }
    }
}
