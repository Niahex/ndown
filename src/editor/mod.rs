use makepad_widgets::*;
use crate::TOKIO_RUNTIME;

pub mod model; // Nouveau module enfant
use model::block::{Block, BlockType};
use model::document::Document;

pub mod view;
use view::{DrawParams, EditorView};

#[derive(Clone, DefaultNone, Debug)]
pub enum EditorAction {
    FileLoaded(String),
    AsyncFileLoaded(String, Vec<Block>),
    AsyncFileSaved(String),
    AsyncError(String),
    None,
}

impl EditorAreaRef {
    pub fn load_file(&self, cx: &mut Cx, filename: String) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.load_file_async(cx, filename);
        }
    }
}

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub THEME_FONT_REGULAR = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Regular.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_BOLD = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Bold.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_ITALIC = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Italic.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_CODE = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuSansMonoNerdFont-Regular.ttf", 0.0, 0.0),
        }
    }

    pub EditorArea = {{EditorArea}}{
        width: Fill, height: Fill
        padding: 25.0
        draw_bg: { color: (NORD_POLAR_0) }

        scroll_bars: <ScrollBars> {}

        draw_text_reg: { text_style: <THEME_FONT_REGULAR> { font_size: 12.1 }, color: (NORD_SNOW_2) }
        draw_text_bold: { text_style: <THEME_FONT_BOLD> { font_size: 12.1 }, color: (NORD_SNOW_2) }
        draw_text_italic: { text_style: <THEME_FONT_ITALIC> { font_size: 12.1 }, color: (NORD_SNOW_2) }
        draw_text_code: { text_style: <THEME_FONT_CODE> { font_size: 12.1 }, color: (NORD_AURORA_GREEN) }

        draw_text_header1: { text_style: <THEME_FONT_BOLD> { font_size: 29.0 }, color: (NORD_FROST_1) }
        draw_text_header2: { text_style: <THEME_FONT_BOLD> { font_size: 21.8 }, color: (NORD_FROST_2) }
        draw_text_header3: { text_style: <THEME_FONT_BOLD> { font_size: 19.4 }, color: (NORD_FROST_2) }
        draw_text_header4: { text_style: <THEME_FONT_BOLD> { font_size: 16.9 }, color: (NORD_FROST_2) }
        draw_text_header5: { text_style: <THEME_FONT_BOLD> { font_size: 14.5 }, color: (NORD_FROST_2) }
        draw_text_quote: { text_style: <THEME_FONT_ITALIC> { font_size: 13.3 }, color: (NORD_AURORA_ORANGE) }

        draw_cursor: { color: #ffffff }
        draw_selection: { color: (NORD_POLAR_3) }
        draw_code_bg: { color: (NORD_POLAR_2) }

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
pub struct EditorArea {
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    scroll_bars: ScrollBars,

    #[live]
    draw_text_reg: DrawText,
    #[live]
    draw_text_bold: DrawText,
    #[live]
    draw_text_italic: DrawText,
    #[live]
    draw_text_code: DrawText,
    #[live]
    draw_text_header1: DrawText,
    #[live]
    draw_text_header2: DrawText,
    #[live]
    draw_text_header3: DrawText,
    #[live]
    draw_text_header4: DrawText,
    #[live]
    draw_text_header5: DrawText,
    #[live]
    draw_text_quote: DrawText,

    #[live]
    draw_cursor: DrawColor,
    #[live]
    draw_selection: DrawColor,
    #[live]
    draw_code_bg: DrawColor,

    #[animator]
    animator: Animator,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[area]
    area: Area,

    #[rust]
    document: Document,
    #[rust]
    cursor_block: usize,
    #[rust]
    cursor_char: usize,
    #[rust]
    selection_anchor: Option<(usize, usize)>,

    #[rust]
    blink_timer: Timer,
    #[rust]
    is_dragging: bool,
    #[rust]
    deferred_finger_tap: Option<DVec2>,
    #[rust]
    clipboard: Option<arboard::Clipboard>,

    #[rust]
    block_y_offsets: Vec<f64>,
    #[rust]
    last_rendered_width: f64,
    #[rust]
    current_file: Option<String>,
}

impl LiveHook for EditorArea {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.document = Document::new();
        self.cursor_block = 0;
        self.cursor_char = 0;
        self.blink_timer = cx.start_timeout(0.5);
        self.clipboard = arboard::Clipboard::new().ok();
    }
}

impl EditorArea {
    fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some(anchor) = self.selection_anchor {
            let cursor = (self.cursor_block, self.cursor_char);
            if anchor < cursor {
                Some((anchor, cursor))
            } else {
                Some((cursor, anchor))
            }
        } else {
            None
        }
    }

    fn reset_blink(&mut self, cx: &mut Cx) {
        self.animator_play(cx, ids!(blink.on));
        cx.stop_timer(self.blink_timer);
        self.blink_timer = cx.start_timeout(0.5);
    }

    fn find_prev_word(&self) -> usize {
        if self.cursor_char == 0 {
            return 0;
        }
        let block = &self.document.blocks[self.cursor_block];
        let text = block.full_text();
        
        // Skip whitespace backwards
        let mut i = self.cursor_char;
        while i > 0 {
            if let Some(c) = text.chars().nth(i - 1) {
                 if !c.is_whitespace() {
                     break;
                 }
            }
            i -= 1;
        }
        
        // Skip non-whitespace backwards
        while i > 0 {
             if let Some(c) = text.chars().nth(i - 1) {
                 if c.is_whitespace() {
                     break;
                 }
             }
             i -= 1;
        }
        i
    }

    fn find_next_word(&self) -> usize {
        let block = &self.document.blocks[self.cursor_block];
        let text = block.full_text();
        let len = block.text_len();
        let mut i = self.cursor_char;
        
        // Skip whitespace forwards
        while i < len {
            if let Some(c) = text.chars().nth(i) {
                if !c.is_whitespace() {
                    break;
                }
            }
             i += 1;
        }
        
        // Skip non-whitespace forwards
        while i < len {
             if let Some(c) = text.chars().nth(i) {
                if c.is_whitespace() {
                    break;
                }
            }
            i += 1;
        }
        i
    }

    fn invalidate_layout(&mut self) {
        self.block_y_offsets.clear();
    }

    fn invalidate_layout_from(&mut self, block_idx: usize) {
        if block_idx < self.block_y_offsets.len() {
            self.block_y_offsets.truncate(block_idx);
        }
    }

    pub fn load_file_async(&mut self, _cx: &mut Cx, filename: String) {
        let filename_clone = filename.clone();
        self.current_file = Some(filename.clone());
        
        TOKIO_RUNTIME.spawn(async move {
            use std::io::BufRead;
            if let Ok(file) = std::fs::File::open(&filename_clone) {
                let reader = std::io::BufReader::new(file);
                let mut new_blocks = Vec::with_capacity(1024);
                let mut id_gen = 1000;
                
                for line_res in reader.lines() {
                    if let Ok(line) = line_res {
                        let ty = if line.starts_with("# ") {
                            BlockType::Heading1
                        } else if line.starts_with("## ") {
                            BlockType::Heading2
                        } else if line.starts_with("### ") {
                            BlockType::Heading3
                        } else if line.starts_with("#### ") {
                            BlockType::Heading4
                        } else if line.starts_with("##### ") {
                            BlockType::Heading5
                        } else if line.starts_with("> ") {
                            BlockType::Quote
                        } else {
                            BlockType::Paragraph
                        };

                        let text = if matches!(ty, BlockType::Heading1) {
                            &line[2..]
                        } else if matches!(ty, BlockType::Heading2) {
                            &line[3..]
                        } else if matches!(ty, BlockType::Heading3) {
                            &line[4..]
                        } else if matches!(ty, BlockType::Heading4) {
                            &line[5..]
                        } else if matches!(ty, BlockType::Heading5) {
                            &line[6..]
                        } else if matches!(ty, BlockType::Quote) {
                            &line[2..]
                        } else {
                            &line
                        };

                        let block = Block::new(id_gen, ty, text);
                        id_gen += 1;
                        new_blocks.push(block);
                    }
                }

                if new_blocks.is_empty() {
                    new_blocks.push(Block::new(id_gen, BlockType::Paragraph, ""));
                }

                Cx::post_action(EditorAction::AsyncFileLoaded(filename_clone, new_blocks));
            } else {
                Cx::post_action(EditorAction::AsyncError(format!("Failed to open {}", filename_clone)));
            }
        });
    }

    pub fn set_document(&mut self, doc: Document) {
        self.document = doc;
        self.cursor_block = 0;
        self.cursor_char = 0;
        self.invalidate_layout();
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

            Hit::TextCopy(e) => {
                if let Some((start, end)) = self.get_selection_range() {
                    let text = self.document.get_text_in_range(start, end);
                    *e.response.borrow_mut() = Some(text);
                } else {
                    *e.response.borrow_mut() = None;
                }
            }

            Hit::TextCut(e) => {
                if let Some((start, end)) = self.get_selection_range() {
                    let text = self.document.get_text_in_range(start, end);
                    *e.response.borrow_mut() = Some(text);
                    self.document.delete_range(start, end);
                    self.cursor_block = start.0;
                    self.cursor_char = start.1;
                    self.selection_anchor = None;
                    self.invalidate_layout_from(start.0);
                    self.redraw(cx);
                } else {
                    *e.response.borrow_mut() = None;
                }
            }

            Hit::KeyDown(ke) => {
                self.reset_blink(cx);
                let shift = ke.modifiers.shift;
                let ctrl = ke.modifiers.control || ke.modifiers.logo;

                if ctrl && ke.key_code == KeyCode::KeyS {
                    let doc_snapshot = self.document.snapshot();
                    let filename = self.current_file.clone().unwrap_or_else(|| "story.md".to_string());
                    
                    TOKIO_RUNTIME.spawn(async move {
                        match doc_snapshot.save_to_file(&filename) {
                            Ok(_) => {
                                makepad_widgets::log!("Async Save: Document saved to {}", filename);
                                Cx::post_action(EditorAction::AsyncFileSaved(filename));
                            },
                            Err(e) => {
                                makepad_widgets::log!("Async Save Error: {}", e);
                                Cx::post_action(EditorAction::AsyncError(e.to_string()));
                            }
                        }
                    });
                    return;
                }

                if ctrl && ke.key_code == KeyCode::KeyA {
                    self.selection_anchor = Some((self.cursor_block, 0));
                    self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                    self.redraw(cx);
                    return;
                }

                if ctrl && ke.key_code == KeyCode::KeyC {
                    if let Some((start, end)) = self.get_selection_range() {
                        let text = self.document.get_text_in_range(start, end);
                        if let Some(clipboard) = &mut self.clipboard {
                            let _ = clipboard.set_text(text);
                        } else {
                            cx.copy_to_clipboard(&text);
                        }
                    }
                    return;
                }

                if ctrl && ke.key_code == KeyCode::KeyX {
                    if let Some((start, end)) = self.get_selection_range() {
                        let text = self.document.get_text_in_range(start, end);
                        if let Some(clipboard) = &mut self.clipboard {
                            let _ = clipboard.set_text(text);
                        } else {
                            cx.copy_to_clipboard(&text);
                        }
                        self.document.delete_range(start, end);
                        self.cursor_block = start.0;
                        self.cursor_char = start.1;
                        self.selection_anchor = None;
                        self.invalidate_layout_from(start.0);
                        self.redraw(cx);
                    }
                    return;
                }

                if ctrl && ke.key_code == KeyCode::KeyV {
                    let text_opt = if let Some(clipboard) = &mut self.clipboard {
                        clipboard.get_text().ok()
                    } else {
                        None
                    };

                    if let Some(text) = text_opt {
                        if !text.is_empty() {
                            let mut start_block = self.cursor_block;
                            if let Some((start, end)) = self.get_selection_range() {
                                self.document.delete_range(start, end);
                                self.cursor_block = start.0;
                                self.cursor_char = start.1;
                                start_block = start.0;
                                self.selection_anchor = None;
                            }

                            // Remove carriage returns
                            let text = text.replace("\r\n", "\n").replace('\r', "\n");

                            let mut parts = text.split('\n');
                            if let Some(first) = parts.next() {
                                let added = self.document.insert_text_at(
                                    self.cursor_block,
                                    self.cursor_char,
                                    first,
                                );
                                self.cursor_char += added;
                            }

                            for part in parts {
                                // Split block at cursor
                                let current_block = &mut self.document.blocks[self.cursor_block];
                                let rest_text: String =
                                    current_block.text.chars().skip(self.cursor_char).collect();
                                let rest_len = rest_text.chars().count();

                                // Truncate current block
                                let current_len = current_block.text_len();
                                for _ in 0..rest_len {
                                    self.document
                                        .remove_char_at(self.cursor_block, current_len - rest_len);
                                }

                                // Create new block with rest
                                let new_block = Block::new(
                                    self.document.generate_id(),
                                    BlockType::Paragraph,
                                    &rest_text,
                                );

                                self.document
                                    .blocks
                                    .insert(self.cursor_block + 1, new_block);
                                self.cursor_block += 1;
                                self.cursor_char = 0;

                                let added =
                                    self.document.insert_text_at(self.cursor_block, 0, part);
                                self.cursor_char += added;
                            }

                            self.invalidate_layout_from(start_block);
                            self.redraw(cx);
                        }
                    }
                    return;
                }

                if ctrl && (ke.key_code == KeyCode::KeyB || ke.key_code == KeyCode::KeyI) {
                    if let Some(((start_blk, start_char), (end_blk, end_char))) =
                        self.get_selection_range()
                    {
                        if start_blk == end_blk {
                            let style_type = if ke.key_code == KeyCode::KeyB { 0 } else { 1 };
                            self.document
                                .toggle_formatting(start_blk, start_char, end_char, style_type);
                            // La longueur du texte ne change pas avec toggle_formatting (juste les styles)
                            // Donc pas besoin de toucher au curseur
                            // On garde la selection pour pouvoir re-toggeler si besoin
                        }
                    } else {
                        let marker = if ke.key_code == KeyCode::KeyB {
                            "**"
                        } else {
                            "*"
                        };
                        let insert_text = if ke.key_code == KeyCode::KeyB {
                            "****"
                        } else {
                            "**"
                        };
                        self.document.insert_text_at(
                            self.cursor_block,
                            self.cursor_char,
                            insert_text,
                        );
                        self.cursor_char += marker.len();
                    }
                    self.redraw(cx);
                    return;
                }

                if shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some((self.cursor_block, self.cursor_char));
                    }
                } else if !ctrl {
                    match ke.key_code {
                        KeyCode::ArrowUp
                        | KeyCode::ArrowDown
                        | KeyCode::ArrowLeft
                        | KeyCode::ArrowRight => {
                            self.selection_anchor = None;
                        }
                        _ => {}
                    }
                }

                if ke.key_code == KeyCode::Tab {
                    let current_ty = self.document.blocks[self.cursor_block].ty.clone();
                    if current_ty == BlockType::ListItem || current_ty == BlockType::OrderedListItem
                    {
                        if shift {
                            if self.document.blocks[self.cursor_block].indent > 0 {
                                self.document.blocks[self.cursor_block].indent -= 1;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else if self.document.blocks[self.cursor_block].indent < 10 {
                            // Max indentation
                            self.document.blocks[self.cursor_block].indent += 1;
                            self.invalidate_layout_from(self.cursor_block);
                        }
                    }
                    self.redraw(cx);
                    return; // Consommer l'événement Tab pour ne pas perdre le focus ou insérer de tab
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
                        let current_ty = self.document.blocks[self.cursor_block].ty.clone();
                        let current_len = self.document.blocks[self.cursor_block].text_len();

                        // Exit list if empty item
                        if (current_ty == BlockType::ListItem
                            || current_ty == BlockType::OrderedListItem)
                            && current_len == 0
                        {
                            if self.document.blocks[self.cursor_block].indent > 0 {
                                self.document.blocks[self.cursor_block].indent -= 1;
                            } else {
                                self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                            }
                            self.invalidate_layout_from(self.cursor_block);
                            self.redraw(cx);
                            return;
                        }

                        let (new_ty, new_indent) = if current_ty == BlockType::ListItem {
                            (
                                BlockType::ListItem,
                                self.document.blocks[self.cursor_block].indent,
                            )
                        } else if current_ty == BlockType::OrderedListItem {
                            (
                                BlockType::OrderedListItem,
                                self.document.blocks[self.cursor_block].indent,
                            )
                        } else {
                            (BlockType::Paragraph, 0)
                        };

                        let mut new_block = Block::new(self.document.generate_id(), new_ty, "");
                        new_block.indent = new_indent;

                        self.document
                            .blocks
                            .insert(self.cursor_block + 1, new_block);
                        let invalid_from = self.cursor_block;
                        self.cursor_block += 1;
                        self.cursor_char = 0;
                        self.invalidate_layout_from(invalid_from);
                    }
                    KeyCode::Delete => {
                        if ctrl {
                            let end = self.find_next_word();
                            if end > self.cursor_char {
                                let range = (
                                    (self.cursor_block, self.cursor_char),
                                    (self.cursor_block, end),
                                );
                                self.document.delete_range(range.0, range.1);
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.selection_anchor = None;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else {
                            let block = &mut self.document.blocks[self.cursor_block];
                            if self.cursor_char < block.text_len() {
                                self.document
                                    .remove_char_at(self.cursor_block, self.cursor_char);
                                // Layout might change (wrapping)
                                self.invalidate_layout_from(self.cursor_block);
                            } else if self.cursor_block < self.document.blocks.len() - 1
                                && self
                                    .document
                                    .merge_block_with_prev(self.cursor_block + 1)
                                    .is_some()
                            {
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if ctrl {
                            let start = self.find_prev_word();
                            if start < self.cursor_char {
                                let range = (
                                    (self.cursor_block, start),
                                    (self.cursor_block, self.cursor_char),
                                );
                                let new_cursor = self.document.delete_range(range.0, range.1);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.selection_anchor = None;
                                self.invalidate_layout_from(self.cursor_block);
                            } else {
                                self.selection_anchor = None;
                                if self.cursor_char > 0 {
                                    if self
                                        .document
                                        .remove_char_at(self.cursor_block, self.cursor_char - 1)
                                    {
                                        self.cursor_char -= 1;
                                        self.invalidate_layout_from(self.cursor_block);
                                    }
                                } else if self.cursor_block > 0 {
                                    let current_type =
                                        self.document.blocks[self.cursor_block].ty.clone();
                                    if current_type != BlockType::Paragraph {
                                        self.document.blocks[self.cursor_block].ty =
                                            BlockType::Paragraph;
                                        self.invalidate_layout_from(self.cursor_block);
                                    } else if let Some(new_char_pos) =
                                        self.document.merge_block_with_prev(self.cursor_block)
                                    {
                                        self.cursor_block -= 1;
                                        self.cursor_char = new_char_pos;
                                        self.invalidate_layout_from(self.cursor_block);
                                    }
                                } else {
                                    let current_type =
                                        self.document.blocks[self.cursor_block].ty.clone();
                                    if current_type != BlockType::Paragraph {
                                        self.document.blocks[self.cursor_block].ty =
                                            BlockType::Paragraph;
                                        self.invalidate_layout_from(self.cursor_block);
                                    }
                                }
                            }
                        } else if self.cursor_char > 0 {
                            if self
                                .document
                                .remove_char_at(self.cursor_block, self.cursor_char - 1)
                            {
                                self.cursor_char -= 1;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else if self.cursor_block > 0 {
                            let current_type = self.document.blocks[self.cursor_block].ty.clone();
                            if current_type != BlockType::Paragraph {
                                self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                                self.invalidate_layout_from(self.cursor_block);
                            } else if let Some(new_char_pos) =
                                self.document.merge_block_with_prev(self.cursor_block)
                            {
                                self.cursor_block -= 1;
                                self.cursor_char = new_char_pos;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        } else {
                            let current_type = self.document.blocks[self.cursor_block].ty.clone();
                            if current_type != BlockType::Paragraph {
                                self.document.blocks[self.cursor_block].ty = BlockType::Paragraph;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                        }
                    }
                    _ => {}
                }
                self.redraw(cx);
            }
            Hit::TextInput(te) => {
                let is_valid_input = !te.input.chars().any(|c| c.is_control())
                    || te.input.contains('\n')
                    || te.input.contains('\r')
                    || te.input.contains('\t');

                if !te.input.is_empty() && is_valid_input {
                    let mut wrapped = false;
                    if let Some(((start_blk, start_char), (end_blk, end_char))) =
                        self.get_selection_range()
                    {
                        if start_blk == end_blk
                            && start_blk == self.cursor_block
                            && start_char != end_char
                            && (te.input == "*" || te.input == "`" || te.input == "_")
                        {
                            let consumed = self
                                .document
                                .wrap_selection(start_blk, start_char, end_char, &te.input);
                            if !consumed {
                                self.selection_anchor = Some((start_blk, start_char + 1));
                                self.cursor_char = end_char + 1;
                            } else {
                                self.cursor_char = end_char;
                                self.selection_anchor = Some((start_blk, start_char));
                            }
                            wrapped = true;
                            // Formatting change doesn't usually change layout height unless font size changes or code block
                            // but wrap_selection might change width.
                            // We can invalidate safely.
                            self.invalidate_layout_from(start_blk);
                        }
                    }

                    if !wrapped {
                        if let Some((start, end)) = self.get_selection_range() {
                            if start != end {
                                let new_cursor = self.document.delete_range(start, end);
                                self.cursor_block = new_cursor.0;
                                self.cursor_char = new_cursor.1;
                                self.invalidate_layout_from(self.cursor_block);
                            }
                            self.selection_anchor = None;
                        }

                        let added = self.document.insert_text_at(
                            self.cursor_block,
                            self.cursor_char,
                            &te.input,
                        );
                        self.cursor_char += added;

                        if let Some(removed_chars) =
                            self.document.try_convert_block(self.cursor_block)
                        {
                            self.cursor_char = self.cursor_char.saturating_sub(removed_chars);
                        }

                        if te.input == " "
                            && self.document.apply_inline_formatting(self.cursor_block)
                        {
                            self.cursor_char = self.document.blocks[self.cursor_block].text_len();
                        }
                        self.invalidate_layout_from(self.cursor_block);
                    }
                    self.redraw(cx);
                }
            }
            _ => {}
        }

        // Handle Async Actions
        if let Event::Actions(actions) = event {
            for action in actions {
                let editor_action: EditorAction = action.cast();
                match editor_action {
                    EditorAction::AsyncFileLoaded(path, blocks) => {
                        self.document.blocks = blocks;
                        self.cursor_block = 0;
                        self.cursor_char = 0;
                        self.current_file = Some(path.clone());
                        self.invalidate_layout();
                        self.redraw(cx);
                        cx.widget_action(self.widget_uid(), &scope.path, EditorAction::FileLoaded(path));
                    }
                    EditorAction::AsyncFileSaved(path) => {
                        makepad_widgets::log!("Successfully saved to {}", path);
                    }
                    EditorAction::AsyncError(err) => {
                        makepad_widgets::log!("Async Error: {}", err);
                    }
                    _ => {}
                }
            }
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
            draw_text_header3: &mut self.draw_text_header3,
            draw_text_header4: &mut self.draw_text_header4,
            draw_text_header5: &mut self.draw_text_header5,
            draw_text_quote: &mut self.draw_text_quote,
            draw_cursor: &mut self.draw_cursor,
            draw_selection: &mut self.draw_selection,
            draw_code_bg: &mut self.draw_code_bg,
        };

        let is_cache_valid = !self.block_y_offsets.is_empty()
            && self.block_y_offsets.len() == self.document.blocks.len();
        if !is_cache_valid {
            self.block_y_offsets.clear();
        }

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
            },
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
