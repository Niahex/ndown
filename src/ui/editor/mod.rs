use makepad_widgets::*;
use crate::model::document::Document;
use crate::model::block::{Block, BlockType};

pub mod view;
use view::EditorView;

live_design!{
    use link::theme::*;
    use link::widgets::*;
    
    pub EditorArea = {{EditorArea}}{
        width: Fill, height: Fill
        draw_bg: { color: #2e3440 }
        
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
    #[rust] selection_anchor: Option<(usize, usize)>, // (block, char)
    
    #[rust] blink_timer: Timer,
    #[rust] is_dragging: bool,
    
    // Cache simple pour le hit testing approximatif
    #[rust] line_heights: Vec<f64>,
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
            // Comparaison simple des tuples (block, char) fonctionne car usize impl Ord
            if anchor < cursor { Some((anchor, cursor)) } else { Some((cursor, anchor)) }
        } else { None }
    }

    fn reset_blink(&mut self, cx: &mut Cx) {
         self.animator_play(cx, ids!(blink.on));
         cx.stop_timer(self.blink_timer);
         self.blink_timer = cx.start_timeout(0.5);
    }
}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
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
            Hit::FingerDown(fe) => {
                cx.set_key_focus(self.area);
                self.reset_blink(cx);
                self.is_dragging = true;
                
                // Hit Test Approximatif
                let rect = self.area.rect(cx);
                let rel = fe.abs - rect.pos - self.layout.padding.left_top();
                
                // Trouver le bloc (Y)
                let mut y_acc = 0.0;
                let mut found_block = 0;
                for (i, h) in self.line_heights.iter().enumerate() {
                    if rel.y < y_acc + *h {
                        found_block = i;
                        break;
                    }
                    y_acc += *h + 5.0; // + Padding
                    found_block = i;
                }
                
                // Trouver le char (X) - Approx simple
                // Pour faire mieux, il faudrait le cache des largeurs de char
                let avg_width = 8.0; 
                let col = (rel.x / avg_width).round().max(0.0) as usize;
                let len = if found_block < self.document.blocks.len() {
                    self.document.blocks[found_block].text_len()
                } else { 0 };
                
                self.cursor_block = found_block.min(self.document.blocks.len().saturating_sub(1));
                self.cursor_char = col.min(len);
                
                if fe.modifiers.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some((self.cursor_block, self.cursor_char));
                    }
                } else {
                    self.selection_anchor = Some((self.cursor_block, self.cursor_char));
                }
                self.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                if self.is_dragging {
                    let rect = self.area.rect(cx);
                    let rel = fe.abs - rect.pos - self.layout.padding.left_top();
                    
                    let mut y_acc = 0.0;
                    let mut found_block = 0;
                    for (i, h) in self.line_heights.iter().enumerate() {
                        if rel.y < y_acc + *h {
                            found_block = i;
                            break;
                        }
                        y_acc += *h + 5.0;
                        found_block = i;
                    }
                     
                    let avg_width = 8.0; 
                    let col = (rel.x / avg_width).round().max(0.0) as usize;
                    let len = if found_block < self.document.blocks.len() {
                        self.document.blocks[found_block].text_len()
                    } else { 0 };
                    
                    self.cursor_block = found_block.min(self.document.blocks.len().saturating_sub(1));
                    self.cursor_char = col.min(len);
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(_) => {
                self.is_dragging = false;
                if let Some(anchor) = self.selection_anchor {
                    if anchor == (self.cursor_block, self.cursor_char) {
                        self.selection_anchor = None; // Clic simple = pas de sélection
                    }
                }
                self.redraw(cx);
            }
            
            Hit::KeyDown(ke) => {
                self.reset_blink(cx);
                let shift = ke.modifiers.shift;
                
                // Gestion Sélection Shift
                if shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some((self.cursor_block, self.cursor_char));
                    }
                } else if !ke.modifiers.control && !ke.modifiers.logo { 
                    // Si on bouge sans shift ni ctrl, on perd la sélection
                    // Sauf si c'est une commande spéciale qui gère elle-même
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
                        if self.cursor_char > 0 {
                            self.cursor_char -= 1;
                        } else if self.cursor_block > 0 {
                            self.cursor_block -= 1;
                            self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                        }
                    }
                    KeyCode::ArrowRight => {
                        let len = self.document.blocks[self.cursor_block].text_len();
                        if self.cursor_char < len {
                            self.cursor_char += 1;
                        } else if self.cursor_block < self.document.blocks.len() - 1 {
                            self.cursor_block += 1;
                            self.cursor_char = 0;
                        }
                    }
                    KeyCode::ReturnKey => {
                        self.selection_anchor = None; // Reset sel
                        let new_block = Block::new(
                            self.document.generate_id(),
                            BlockType::Paragraph,
                            ""
                        );
                        self.document.blocks.insert(self.cursor_block + 1, new_block);
                        self.cursor_block += 1;
                        self.cursor_char = 0;
                    }
                    KeyCode::Backspace => {
                        // TODO: Supprimer la sélection si active
                        if self.selection_anchor.is_some() {
                            // Implémenter delete_selection
                            self.selection_anchor = None; 
                            // Pour l'instant on fait rien pour éviter le crash
                        } else {
                            if self.cursor_char > 0 {
                                if self.document.remove_char_at(self.cursor_block, self.cursor_char - 1) {
                                    self.cursor_char -= 1;
                                }
                            } else if self.cursor_block > 0 {
                                self.cursor_block -= 1;
                                self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                            }
                        }
                    }
                    _ => {}
                }
                self.redraw(cx);
            }
            Hit::TextInput(te) => {
                if !te.input.is_empty() {
                    // GESTION WRAP SELECTION (*, **, `)
                    let mut wrapped = false;
                    if let Some(((start_blk, start_char), (end_blk, end_char))) = self.get_selection_range() {
                        // Supporte uniquement la sélection dans un seul bloc pour l'instant
                        if start_blk == end_blk && start_blk == self.cursor_block {
                            if te.input == "*" || te.input == "`" || te.input == "_" {
                                // WRAP !
                                // Si c'est "*", on regarde si on veut faire "**" (compliqué en 1 étape, simple toggle ici)
                                self.document.wrap_selection(start_blk, start_char, end_char, &te.input);
                                
                                // On décale la sélection pour inclure les nouveaux marqueurs ?
                                // Ou on la reset.
                                // Pour l'UX, souvent on garde la sélection sur le texte interne
                                self.selection_anchor = Some((start_blk, start_char + 1));
                                self.cursor_char = end_char + 1;
                                wrapped = true;
                            }
                        }
                    }

                    if !wrapped {
                        // Comportement standard : remplacer la sélection (TODO) ou insérer
                        if self.selection_anchor.is_some() {
                             // TODO: Delete selection before insert
                             self.selection_anchor = None;
                        }
                        
                        let added = self.document.insert_text_at(self.cursor_block, self.cursor_char, &te.input);
                        self.cursor_char += added;
                        
                        if let Some(removed_chars) = self.document.try_convert_block(self.cursor_block) {
                            self.cursor_char = self.cursor_char.saturating_sub(removed_chars);
                        }
                        
                        if te.input == " " {
                            if self.document.apply_inline_formatting(self.cursor_block) {
                                self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                            }
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
        let rect = cx.turtle().rect();
        
        // Pre-calculate selection to avoid borrow checker issues
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
        
        // On récupère la hauteur et le hit result (même si on l'utilise pas encore ici pour le layout cache)
        let (used_height, _) = view.draw_document(
            cx, 
            &self.document, 
            &self.layout, 
            rect, 
            (self.cursor_block, self.cursor_char),
            selection,
            true,
            None // Pas de hit test pendant le draw pour l'instant
        );
        
        // Mise à jour du cache de hauteur de lignes pour le hit test approximatif
        // Pour faire simple, on recalcule ou on suppose. 
        // Ici je reconstruis un cache naïf basé sur les types de blocs
        self.line_heights.clear();
        for block in &self.document.blocks {
             let h = match block.ty {
                BlockType::Heading1 => 28.0,
                BlockType::Heading2 => 22.0,
                BlockType::Quote => 20.0,
                _ => 18.0
            };
            self.line_heights.push(h); // TODO: Récupérer la vraie hauteur depuis draw_document
        }

        cx.turtle_mut().set_used(rect.size.x, used_height);
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}
