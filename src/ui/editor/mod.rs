use makepad_widgets::*;

live_design!{
    use link::theme::*;
    use link::widgets::*;
    
    pub EditorArea = {{EditorArea}}{
        width: Fill, height: Fill
        draw_bg: { color: #2e3440 }
        draw_text: {
            text_style: <THEME_FONT_CODE> {
                font_size: 10.0
            }
            color: #eceff4
        }
        draw_cursor: {
            color: #ffffff
        }
        draw_selection: {
            color: #4c566a
        }
        
        animator: {
            blink = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.1}}
                    apply: { draw_cursor: { color: #ffffff00 } }
                }
                on = {
                    from: {all: Forward {duration: 0.1}}
                    apply: { draw_cursor: { color: #ffffff } }
                }
            }
        }
    }
}

// --- STRUCTURES DE DONNÉES ---

#[derive(Clone, Debug)]
struct EditorState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    selection_anchor: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TokenType {
    Normal,
    Header,
    Bold,
    Italic,
    Code,
    Link,
}

struct Token {
    text: String,
    kind: TokenType,
}

// --- WIDGET ---

#[derive(Live, Widget)] 
pub struct EditorArea{
    #[redraw] #[live] draw_bg: DrawColor,
    #[live] draw_text: DrawText,
    #[live] draw_cursor: DrawColor,
    #[live] draw_selection: DrawColor,
    
    #[animator] animator: Animator,
    
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[area] area: Area,
    
    #[rust] lines: Vec<String>,
    #[rust] cursor_line: usize,
    #[rust] cursor_col: usize,
    #[rust] selection_anchor: Option<(usize, usize)>,
    
    #[rust] blink_timer: Timer,
    #[rust] cell_width: f64,
    #[rust] cell_height: f64,
    
    #[rust] undo_stack: Vec<EditorState>,
    #[rust] redo_stack: Vec<EditorState>,
    #[rust] is_dragging: bool,
}

impl LiveHook for EditorArea{
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.lines = vec![
            "# Welcome to ndown".to_string(), 
            "".to_string(),
            "This is a **bold** move.".to_string(),
            "Some *italic* text.".to_string(),
            "A code block: `let x = 42;`".to_string(),
            "".to_string(),
            "Start typing...".to_string()
        ];
        self.cursor_line = 6;
        self.cursor_col = 0;
        self.blink_timer = cx.start_timeout(0.5);
    }
}

impl EditorArea {
    // --- PARSING MARKDOWN SIMPLE ---
    
    fn get_color_for_token(&self, kind: TokenType) -> Vec4 {
        match kind {
            TokenType::Header => vec4(0.53, 0.75, 0.81, 1.0), // #88c0d0 (Nord Blue)
            TokenType::Bold => vec4(0.92, 0.79, 0.54, 1.0),   // #ebcb8b (Nord Yellow)
            TokenType::Italic => vec4(0.71, 0.55, 0.67, 1.0), // #b48ead (Nord Purple)
            TokenType::Code => vec4(0.63, 0.74, 0.54, 1.0),   // #a3be8c (Nord Green)
            TokenType::Link => vec4(0.36, 0.54, 0.66, 1.0),   // #5e81ac (Nord Dark Blue)
            TokenType::Normal => vec4(0.92, 0.93, 0.95, 1.0), // #eceff4 (Nord White)
        }
    }

    fn tokenize_line(&self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        // 1. Check Header (Ligne entière)
        if line.starts_with("#") {
            tokens.push(Token {
                text: line.to_string(),
                kind: TokenType::Header,
            });
            return tokens;
        }
        
        // 2. Inline Parsing (Bold, Italic, Code)
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;
        let mut start = 0;
        
        while i < len {
            // Code Block `...`
            if chars[i] == '`' {
                // Flush text before
                if i > start {
                    tokens.push(Token { text: line[start..i].to_string(), kind: TokenType::Normal });
                }
                
                let code_start = i;
                i += 1;
                while i < len && chars[i] != '`' {
                    i += 1;
                }
                
                if i < len { i += 1; } // Consume closing backtick
                
                tokens.push(Token { text: line[code_start..i].to_string(), kind: TokenType::Code });
                start = i;
                continue;
            }
            
            // Bold **...**
            if i + 1 < len && chars[i] == '*' && chars[i+1] == '*' {
                 if i > start {
                    tokens.push(Token { text: line[start..i].to_string(), kind: TokenType::Normal });
                }
                
                let bold_start = i;
                i += 2; // Skip **
                while i + 1 < len && !(chars[i] == '*' && chars[i+1] == '*') {
                    i += 1;
                }
                
                if i + 1 < len { i += 2; } // Consume closing **
                
                tokens.push(Token { text: line[bold_start..i].to_string(), kind: TokenType::Bold });
                start = i;
                continue;
            }
            
            // Italic *...* (Simple check, might conflict with bold if not careful, but order matters above)
            // Note: Parser très naïf qui peut échouer sur des cas complexes
            if chars[i] == '*' {
                 if i > start {
                    tokens.push(Token { text: line[start..i].to_string(), kind: TokenType::Normal });
                }
                
                let italic_start = i;
                i += 1;
                while i < len && chars[i] != '*' {
                    i += 1;
                }
                
                if i < len { i += 1; } // Consume closing *
                
                tokens.push(Token { text: line[italic_start..i].to_string(), kind: TokenType::Italic });
                start = i;
                continue;
            }

            i += 1;
        }
        
        // Flush remaining text
        if start < len {
            tokens.push(Token { text: line[start..].to_string(), kind: TokenType::Normal });
        }
        
        if tokens.is_empty() {
             tokens.push(Token { text: String::new(), kind: TokenType::Normal });
        }
        
        tokens
    }

    // --- UTILS (Existants) ---
    
    fn reset_blink(&mut self, cx: &mut Cx) {
        self.animator_play(cx, ids!(blink.on));
        cx.stop_timer(self.blink_timer);
        self.blink_timer = cx.start_timeout(0.5);
    }

    fn save_state(&mut self) {
        self.undo_stack.push(EditorState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
            selection_anchor: self.selection_anchor,
        });
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev_state) = self.undo_stack.pop() {
            self.redo_stack.push(EditorState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
                selection_anchor: self.selection_anchor,
            });
            self.lines = prev_state.lines;
            self.cursor_line = prev_state.cursor_line;
            self.cursor_col = prev_state.cursor_col;
            self.selection_anchor = prev_state.selection_anchor;
        }
    }

    fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(EditorState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
                selection_anchor: self.selection_anchor,
            });
            self.lines = next_state.lines;
            self.cursor_line = next_state.cursor_line;
            self.cursor_col = next_state.cursor_col;
            self.selection_anchor = next_state.selection_anchor;
        }
    }

    fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some(anchor) = self.selection_anchor {
            let cursor = (self.cursor_line, self.cursor_col);
            if anchor < cursor {
                Some((anchor, cursor))
            } else {
                Some((cursor, anchor))
            }
        } else {
            None
        }
    }
    
    fn get_selected_text(&self) -> String {
        if let Some(((start_l, start_c), (end_l, end_c))) = self.get_selection_range() {
            let mut result = String::new();
            if start_l == end_l {
                let line = &self.lines[start_l];
                let start_byte = line.char_indices().nth(start_c).map(|(i,_)| i).unwrap_or(line.len());
                let end_byte = line.char_indices().nth(end_c).map(|(i,_)| i).unwrap_or(line.len());
                result.push_str(&line[start_byte..end_byte]);
            } else {
                let l1 = &self.lines[start_l];
                let b1 = l1.char_indices().nth(start_c).map(|(i,_)| i).unwrap_or(l1.len());
                result.push_str(&l1[b1..]);
                result.push('\n');
                for i in (start_l + 1)..end_l {
                    result.push_str(&self.lines[i]);
                    result.push('\n');
                }
                let l2 = &self.lines[end_l];
                let b2 = l2.char_indices().nth(end_c).map(|(i,_)| i).unwrap_or(l2.len());
                result.push_str(&l2[..b2]);
            }
            return result;
        }
        String::new()
    }

    fn delete_selection(&mut self) {
        if let Some(((start_line, start_col), (end_line, end_col))) = self.get_selection_range() {
            self.save_state();
            if start_line == end_line {
                let line = &self.lines[start_line];
                let start_byte = line.char_indices().nth(start_col).map(|(i,_)| i).unwrap_or(line.len());
                let end_byte = line.char_indices().nth(end_col).map(|(i,_)| i).unwrap_or(line.len());
                self.lines[start_line].replace_range(start_byte..end_byte, "");
            } else {
                let start_line_str = &self.lines[start_line];
                let start_byte = start_line_str.char_indices().nth(start_col).map(|(i,_)| i).unwrap_or(start_line_str.len());
                let start_part = start_line_str[..start_byte].to_string();
                
                let end_line_str = &self.lines[end_line];
                let end_byte = end_line_str.char_indices().nth(end_col).map(|(i,_)| i).unwrap_or(end_line_str.len());
                let end_part = end_line_str[end_byte..].to_string();
                
                self.lines.drain(start_line + 1..=end_line);
                self.lines[start_line] = start_part + &end_part;
            }
            self.cursor_line = start_line;
            self.cursor_col = start_col;
            self.selection_anchor = None;
        }
    }

    fn move_cursor_to(&mut self, line: usize, col: usize) {
        self.cursor_line = line.min(self.lines.len().saturating_sub(1));
        let line_len = self.lines[self.cursor_line].chars().count();
        self.cursor_col = col.min(line_len);
    }
    
    fn pos_to_grid(&self, cx: &Cx, abs: DVec2) -> (usize, usize) {
        let rect = self.area.rect(cx);
        let rel = abs - rect.pos;
        let line = (rel.y / self.cell_height).floor().max(0.0) as usize;
        let col = (rel.x / self.cell_width).round().max(0.0) as usize;
        let line = line.min(self.lines.len().saturating_sub(1));
        let line_len = self.lines[line].chars().count();
        let col = col.min(line_len);
        (line, col)
    }

    fn find_prev_word_start(&self) -> (usize, usize) {
        let mut line_idx = self.cursor_line;
        let mut col_idx = self.cursor_col;
        if col_idx == 0 {
            if line_idx > 0 {
                return (line_idx - 1, self.lines[line_idx - 1].chars().count());
            }
            return (0, 0);
        }
        let line = &self.lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        while col_idx > 0 && col_idx <= chars.len() && chars[col_idx - 1].is_whitespace() {
            col_idx -= 1;
        }
        while col_idx > 0 && col_idx <= chars.len() && !chars[col_idx - 1].is_whitespace() {
            col_idx -= 1;
        }
        (line_idx, col_idx)
    }

    fn find_next_word_end(&self) -> (usize, usize) {
        let mut line_idx = self.cursor_line;
        let mut col_idx = self.cursor_col;
        let line = &self.lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        if col_idx >= len {
            if line_idx < self.lines.len() - 1 {
                return (line_idx + 1, 0);
            }
            return (line_idx, len);
        }
        while col_idx < len && !chars[col_idx].is_whitespace() {
            col_idx += 1;
        }
        while col_idx < len && chars[col_idx].is_whitespace() {
            col_idx += 1;
        }
        (line_idx, col_idx)
    }

    fn handle_backspace(&mut self) {
        self.save_state();
        if self.selection_anchor.is_some() {
            self.delete_selection();
            return;
        }
        if self.cursor_col > 0 {
            let line = &self.lines[self.cursor_line];
            if let Some((prev_byte_idx, _)) = line.char_indices().nth(self.cursor_col - 1) {
                 self.lines[self.cursor_line].remove(prev_byte_idx); 
                 self.cursor_col -= 1;
            }
        } else if self.cursor_line > 0 {
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            let prev_line_len = self.lines[self.cursor_line].chars().count();
            self.lines[self.cursor_line].push_str(&current_line);
            self.cursor_col = prev_line_len;
        }
    }
    
    fn measure_cells(&mut self, cx: &mut Cx2d) {
        let text = self.draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), "M");
        if let Some(row) = text.rows.first() {
            if let Some(glyph) = row.glyphs.first() {
                self.cell_width = glyph.advance_in_lpxs() as f64;
                self.cell_height = (glyph.ascender_in_lpxs() - glyph.descender_in_lpxs()) as f64;
                if self.cell_width < 1.0 { self.cell_width = 8.0; }
                if self.cell_height < 1.0 { self.cell_height = 16.0; }
            }
        }
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

        match event.hits(cx, self.area) {
            Hit::KeyFocus(_) => {
                // Force IME update on focus
                let cursor_x = (self.cursor_col as f64 * self.cell_width);
                let cursor_y = (self.cursor_line as f64 * self.cell_height * 1.15);
                cx.show_text_ime(self.area, dvec2(cursor_x, cursor_y));
                self.redraw(cx);
            }
            Hit::KeyFocusLost(_) => {
            }
            
            Hit::KeyDown(ke) => {
                let shift = ke.modifiers.shift;
                let ctrl = ke.modifiers.control || ke.modifiers.logo;
                self.reset_blink(cx);
                
                if ke.key_code == KeyCode::KeyZ && ctrl {
                    if shift { self.redo(); } else { self.undo(); }
                    self.redraw(cx);
                    return;
                }
                
                if ke.key_code == KeyCode::KeyC && ctrl {
                    let text = self.get_selected_text();
                    if !text.is_empty() { cx.copy_to_clipboard(&text); }
                    return;
                }

                if ke.key_code == KeyCode::KeyX && ctrl {
                    self.save_state();
                    let text = self.get_selected_text();
                    if !text.is_empty() {
                        cx.copy_to_clipboard(&text);
                        self.delete_selection();
                        self.redraw(cx);
                    }
                    return;
                }
                
                match ke.key_code {
                    KeyCode::ReturnKey => {
                        self.save_state();
                        self.delete_selection();
                        let current_line = &self.lines[self.cursor_line];
                        let byte_idx = current_line.char_indices().nth(self.cursor_col).map(|(i,_)| i).unwrap_or(current_line.len());
                        let after = current_line[byte_idx..].to_string();
                        self.lines[self.cursor_line].truncate(byte_idx);
                        self.lines.insert(self.cursor_line + 1, after);
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                        self.redraw(cx);
                    }
                    KeyCode::Backspace => {
                        if ctrl {
                            self.save_state();
                            if self.selection_anchor.is_none() {
                                let (target_line, target_col) = self.find_prev_word_start();
                                self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                                self.cursor_line = target_line;
                                self.cursor_col = target_col;
                            }
                            self.delete_selection();
                        } else {
                            self.handle_backspace();
                        }
                        self.redraw(cx);
                    }
                    KeyCode::Delete => {
                        self.save_state();
                        if ctrl {
                             if self.selection_anchor.is_none() {
                                let (target_line, target_col) = self.find_next_word_end();
                                self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                                self.cursor_line = target_line;
                                self.cursor_col = target_col;
                            }
                            self.delete_selection();
                        } else if self.selection_anchor.is_some() {
                            self.delete_selection();
                        } else {
                            let line = &self.lines[self.cursor_line];
                            if self.cursor_col < line.chars().count() {
                                let byte_idx = line.char_indices().nth(self.cursor_col).map(|(i,_)| i).unwrap();
                                self.lines[self.cursor_line].remove(byte_idx);
                            } else if self.cursor_line < self.lines.len() - 1 {
                                let next_line = self.lines.remove(self.cursor_line + 1);
                                self.lines[self.cursor_line].push_str(&next_line);
                            }
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowLeft => {
                        if shift && self.selection_anchor.is_none() {
                            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                        } else if !shift {
                            self.selection_anchor = None;
                        }
                        if self.cursor_col > 0 {
                            self.cursor_col -= 1;
                        } else if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            self.cursor_col = self.lines[self.cursor_line].chars().count();
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowRight => {
                        if shift && self.selection_anchor.is_none() {
                            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                        } else if !shift {
                            self.selection_anchor = None;
                        }
                        if self.cursor_col < self.lines[self.cursor_line].chars().count() {
                            self.cursor_col += 1;
                        } else if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            self.cursor_col = 0;
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowUp => {
                        if shift && self.selection_anchor.is_none() {
                            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                        } else if !shift {
                            self.selection_anchor = None;
                        }
                        if self.cursor_line > 0 {
                            self.cursor_line -= 1;
                            let line_len = self.lines[self.cursor_line].chars().count();
                            self.cursor_col = self.cursor_col.min(line_len);
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ArrowDown => {
                        if shift && self.selection_anchor.is_none() {
                            self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                        } else if !shift {
                            self.selection_anchor = None;
                        }
                        if self.cursor_line < self.lines.len() - 1 {
                            self.cursor_line += 1;
                            let line_len = self.lines[self.cursor_line].chars().count();
                            self.cursor_col = self.cursor_col.min(line_len);
                        }
                        self.redraw(cx);
                    }
                    KeyCode::KeyA if ctrl => {
                        self.selection_anchor = Some((0, 0));
                        self.cursor_line = self.lines.len().saturating_sub(1);
                        self.cursor_col = self.lines[self.cursor_line].chars().count();
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
            
            Hit::TextInput(te) => {
                let clean_input: String = te.input.chars().filter(|c| !c.is_control()).collect();
                if clean_input.len() > 0 {
                    self.save_state();
                    self.reset_blink(cx);
                    self.delete_selection();
                    
                    let line = &self.lines[self.cursor_line];
                    let byte_idx = line.char_indices().nth(self.cursor_col).map(|(i,_)| i).unwrap_or(line.len());
                    
                    self.lines[self.cursor_line].insert_str(byte_idx, &clean_input);
                    self.cursor_col += clean_input.chars().count();
                    self.redraw(cx);
                }
            }

            Hit::FingerHoverIn(_) | Hit::FingerHoverOver(_) => {
                cx.set_cursor(MouseCursor::Text);
            }
            Hit::FingerDown(fe) => {
                cx.set_key_focus(self.area);
                self.reset_blink(cx);
                let (line, col) = self.pos_to_grid(cx, fe.abs);
                if fe.modifiers.shift {
                     if self.selection_anchor.is_none() {
                        self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                    }
                } else {
                    self.selection_anchor = None;
                }
                self.move_cursor_to(line, col);
                if !fe.modifiers.shift {
                    self.selection_anchor = Some((self.cursor_line, self.cursor_col));
                }
                self.is_dragging = true;
                self.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                if self.is_dragging {
                    let (line, col) = self.pos_to_grid(cx, fe.abs);
                    self.move_cursor_to(line, col);
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(_) => {
                self.is_dragging = false;
                if let Some(anchor) = self.selection_anchor {
                    if anchor == (self.cursor_line, self.cursor_col) {
                         self.selection_anchor = None;
                    }
                }
                self.redraw(cx);
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        self.measure_cells(cx);
        let rect = cx.turtle().rect();
        self.draw_bg.draw_abs(cx, rect);
        
        let line_height = self.cell_height * 1.15; 
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            let pos = cx.turtle().pos();
            let start_x = pos.x;
            let y_pos = pos.y;
            
            // Dessin Sélection
            if let Some(((start_l, start_c), (end_l, end_c))) = self.get_selection_range() {
                if line_idx >= start_l && line_idx <= end_l {
                    let sel_start_col = if line_idx == start_l { start_c } else { 0 };
                    let sel_end_col = if line_idx == end_l { end_c } else { line.chars().count() };
                    let sel_x = start_x + (sel_start_col as f64 * self.cell_width);
                    let mut sel_w = (sel_end_col.saturating_sub(sel_start_col)) as f64 * self.cell_width;
                    if line_idx != end_l { sel_w += self.cell_width * 0.5; }
                    if sel_w < 1.0 { sel_w = self.cell_width * 0.5; } 
                    self.draw_selection.draw_abs(cx, Rect {
                        pos: dvec2(sel_x, y_pos),
                        size: dvec2(sel_w, line_height),
                    });
                }
            }
            
            // --- DESSIN PAR TOKENS (Markdown) ---
            let tokens = self.tokenize_line(line);
            for token in tokens {
                self.draw_text.color = self.get_color_for_token(token.kind);
                self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), &token.text);
            }
            
            // Dessin Curseur
            if line_idx == self.cursor_line {
                let cursor_x = start_x + (self.cursor_col as f64 * self.cell_width);
                self.draw_cursor.draw_abs(cx, Rect {
                    pos: dvec2(cursor_x, y_pos),
                    size: dvec2(2.0, line_height),
                });
            }
            
            cx.turtle_new_line();
        }
        
        cx.end_turtle_with_area(&mut self.area);
        
        if cx.has_key_focus(self.area) {
             let cursor_x = (self.cursor_col as f64 * self.cell_width);
             let cursor_y = (self.cursor_line as f64 * self.cell_height * 1.15); 
             cx.show_text_ime(self.area, dvec2(cursor_x, cursor_y));
        }
        
        DrawStep::done()
    }
}
