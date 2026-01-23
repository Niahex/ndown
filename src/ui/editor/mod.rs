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
        
        // Configuration de l'animation pour le curseur
        animator: {
            blink = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.1}}
                    apply: { draw_cursor: { color: #ffffff00 } } // Transparent
                }
                on = {
                    from: {all: Forward {duration: 0.1}}
                    apply: { draw_cursor: { color: #ffffff } } // Visible
                }
            }
        }
    }
}
 
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
}

impl LiveHook for EditorArea{
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.lines = vec!["# Editor Ready".to_string(), "Type something...".to_string()];
        self.cursor_line = 1;
        self.cursor_col = 0;
        
        // Lancer le timer de clignotement
        self.blink_timer = cx.start_timeout(0.5);
    }
}

impl EditorArea {
    fn reset_blink(&mut self, cx: &mut Cx) {
        self.animator_play(cx, ids!(blink.on));
        cx.stop_timer(self.blink_timer);
        self.blink_timer = cx.start_timeout(0.5);
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

    fn delete_selection(&mut self) {
        if let Some(((start_line, start_col), (end_line, end_col))) = self.get_selection_range() {
            if start_line == end_line {
                // Suppression simple sur une ligne
                if start_line < self.lines.len() {
                    let line = &self.lines[start_line];
                    // Conversion char index -> byte index
                    let start_byte = line.char_indices().nth(start_col).map(|(i,_)| i).unwrap_or(line.len());
                    let end_byte = line.char_indices().nth(end_col).map(|(i,_)| i).unwrap_or(line.len());
                    
                    self.lines[start_line].replace_range(start_byte..end_byte, "");
                }
            } else {
                // Suppression multiligne
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
        self.cursor_line = line;
        self.cursor_col = col;
    }

    fn find_prev_word_start(&self) -> (usize, usize) {
        let mut line_idx = self.cursor_line;
        let mut col_idx = self.cursor_col;
        
        // Si début de ligne, on remonte à la fin de la ligne précédente
        if col_idx == 0 {
            if line_idx > 0 {
                return (line_idx - 1, self.lines[line_idx - 1].chars().count());
            }
            return (0, 0);
        }

        let line = &self.lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        
        // Si on est juste après un mot (ou des espaces), on recule d'abord les espaces
        while col_idx > 0 && col_idx <= chars.len() && chars[col_idx - 1].is_whitespace() {
            col_idx -= 1;
        }
        
        // Ensuite on recule tant qu'on n'a pas atteint un espace ou le début
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

        // Si fin de ligne, on descend au début de la ligne suivante
        if col_idx >= len {
            if line_idx < self.lines.len() - 1 {
                return (line_idx + 1, 0);
            }
            return (line_idx, len);
        }

        // Si on est sur un mot, on avance jusqu'à la fin du mot (ou espace)
        while col_idx < len && !chars[col_idx].is_whitespace() {
            col_idx += 1;
        }

        // Ensuite on avance les espaces
        while col_idx < len && chars[col_idx].is_whitespace() {
            col_idx += 1;
        }
        
        (line_idx, col_idx)
    }

    fn handle_backspace(&mut self) {
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
    
    // Mesure la taille réelle d'un caractère avec la police actuelle
    fn measure_cells(&mut self, cx: &mut Cx2d) {
        // On mesure un caractère "large" standard pour le code
        let text = self.draw_text.layout(cx, 0.0, 0.0, None, false, Align::default(), "M");
        if let Some(row) = text.rows.first() {
            if let Some(glyph) = row.glyphs.first() {
                self.cell_width = glyph.advance_in_lpxs() as f64;
                self.cell_height = (glyph.ascender_in_lpxs() - glyph.descender_in_lpxs()) as f64;
                
                // Si la mesure échoue (ex: police pas chargée), on met une valeur par défaut
                if self.cell_width < 1.0 { self.cell_width = 8.0; }
                if self.cell_height < 1.0 { self.cell_height = 16.0; }
            }
        }
    }
}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Gestion de l'animation (clignotement)
        if self.blink_timer.is_event(event).is_some() {
            if self.animator_in_state(cx, ids!(blink.off)) {
                self.animator_play(cx, ids!(blink.on));
            } else {
                self.animator_play(cx, ids!(blink.off));
            }
            self.blink_timer = cx.start_timeout(0.5);
        }
        self.animator_handle_event(cx, event);

        match event {
            Event::KeyDown(ke) => {
                let shift = ke.modifiers.shift;
                let ctrl = ke.modifiers.control || ke.modifiers.logo;
                
                self.reset_blink(cx); // Reset blink on activity
                
                match ke.key_code {
                    KeyCode::ReturnKey => {
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
                    _ => {}
                }
            }
            Event::TextInput(te) => {
                // IMPORTANT: Ignorer les caractères de contrôle (comme Backspace \x7f)
                // Cela empêche l'insertion de carrés bizarres
                let clean_input: String = te.input.chars().filter(|c| !c.is_control()).collect();
                
                if clean_input.len() > 0 {
                    self.reset_blink(cx);
                    self.delete_selection();
                    
                    let line = &self.lines[self.cursor_line];
                    let byte_idx = line.char_indices().nth(self.cursor_col).map(|(i,_)| i).unwrap_or(line.len());
                    
                    self.lines[self.cursor_line].insert_str(byte_idx, &clean_input);
                    self.cursor_col += clean_input.chars().count();
                    self.redraw(cx);
                }
            }
            _ => {}
        }
        
        match event.hits(cx, self.area) {
            Hit::FingerDown(_) => {
                cx.set_key_focus(self.area);
                self.reset_blink(cx);
                self.redraw(cx);
            }
            _ => {}
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        // 1. Calculer la taille des cellules (Police)
        self.measure_cells(cx);
        
        let rect = cx.turtle().rect();
        self.draw_bg.draw_abs(cx, rect);
        
        let line_height = self.cell_height * 1.2; // Un peu d'espacement vertical
        
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
                     // Sélection du saut de ligne
                    if line_idx != end_l { sel_w += self.cell_width * 0.5; }
                    if sel_w < 1.0 { sel_w = self.cell_width * 0.5; } 

                    self.draw_selection.draw_abs(cx, Rect {
                        pos: dvec2(sel_x, y_pos),
                        size: dvec2(sel_w, line_height),
                    });
                }
            }
            
            // Dessin Texte
            self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), line);
            
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
        DrawStep::done()
    }
}