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
        
        // Styles de police pour les blocs
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
    #[rust] cursor_block: usize, // Index du bloc courant
    #[rust] cursor_char: usize,  // Position dans le texte du bloc
    
    #[rust] blink_timer: Timer,
}

impl LiveHook for EditorArea{
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.document = Document::new();
        self.cursor_block = 0;
        self.cursor_char = 0;
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
            Hit::KeyFocus(_) => {
                // IME a gérer plus tard avec la position précise
                self.redraw(cx);
            }
            Hit::FingerDown(_fe) => {
                cx.set_key_focus(self.area);
                // TODO: Hit testing sur les blocs
                self.redraw(cx);
            }
            Hit::KeyDown(ke) => {
                // Gestion navigation basique
                match ke.key_code {
                    KeyCode::ArrowUp => {
                        if self.cursor_block > 0 {
                            self.cursor_block -= 1;
                            // Clamp char index
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
                        // Diviser le bloc ou créer un nouveau bloc
                        // MVP: Nouveau bloc vide dessous
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
                        // Suppression via le modèle (gère les spans)
                        if self.cursor_char > 0 {
                            if self.document.remove_char_at(self.cursor_block, self.cursor_char - 1) {
                                self.cursor_char -= 1;
                            }
                        } else if self.cursor_block > 0 {
                            // Fusion de blocs (TODO)
                            self.cursor_block -= 1;
                            self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                        }
                    }
                    _ => {}
                }
                self.redraw(cx);
            }
            Hit::TextInput(te) => {
                if !te.input.is_empty() {
                    // Insertion via le modèle (gère les spans)
                    let added = self.document.insert_text_at(self.cursor_block, self.cursor_char, &te.input);
                    self.cursor_char += added;
                    
                    // WYSIWYG Trigger: Convertir "# " en Heading
                    if let Some(removed_chars) = self.document.try_convert_block(self.cursor_block) {
                        self.cursor_char = self.cursor_char.saturating_sub(removed_chars);
                    }
                    
                    // WYSIWYG Inline: Convertir **gras** etc. seulement sur ESPACE
                    if te.input == " " {
                        if self.document.apply_inline_formatting(self.cursor_block) {
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
        let rect = cx.turtle().rect();
        
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
        
        let used_height = view.draw_document(
            cx, 
            &self.document, 
            &self.layout, 
            rect, 
            (self.cursor_block, self.cursor_char),
            None, // Selection pas encore réimplémentée
            true
        );
        
        cx.turtle_mut().set_used(rect.size.x, used_height);
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}