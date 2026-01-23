use makepad_widgets::*;
use crate::editor_state::EditorState;
use crate::ui::{EventHandler, BlockRenderer};
use crate::storage::{Storage, LocalStorage};
use crate::rich_text_input::{RichTextInput, formatting::FormattingManager, RichTextInputAction};
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::rich_text_input::*;
    
    BlockLabelBase = <Label> {
        width: Fill, padding: 10
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    BlockInputBase = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #2a2a2a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
    }
    
    BlockInputH1 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #2a2a2a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 25}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
    }
    
    BlockInputH2 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #2a2a2a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 21}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
    }
    
    BlockInputH3 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #2a2a2a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 18}
            fn get_color(self) -> vec4 {
                return #ffffff;
            }
        }
    }
    
    BlockInputInactive = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #1a1a1a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            fn get_color(self) -> vec4 {
                return #cccccc;
            }
        }
    }
    
    BlockInputInactiveH1 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #1a1a1a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 25}
            fn get_color(self) -> vec4 {
                return #cccccc;
            }
        }
    }
    
    BlockInputInactiveH2 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #1a1a1a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 21}
            fn get_color(self) -> vec4 {
                return #cccccc;
            }
        }
    }
    
    BlockInputInactiveH3 = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { 
            fn pixel(self) -> vec4 {
                return #1a1a1a;
            }
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 18}
            fn get_color(self) -> vec4 {
                return #cccccc;
            }
        }
    }
    
    // List templates
    BlockInputList = <RichTextInputBase> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #ffffff
        }
    }
    
    BlockInputInactiveList = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    BlockInputOrderedList = <TextInput> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #ffffff
        }
    }
    
    BlockInputInactiveOrderedList = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    pub BlockEditor = {{BlockEditor}} {
        width: Fill, height: Fill
        flow: Down, spacing: 5
        
        BlockLabel = <BlockLabelBase> {}
        BlockInput = <BlockInputBase> {}
        BlockInputH1 = <BlockInputH1> {}
        BlockInputH2 = <BlockInputH2> {}
        BlockInputH3 = <BlockInputH3> {}
        BlockInputInactive = <BlockInputInactive> {}
        BlockInputInactiveH1 = <BlockInputInactiveH1> {}
        BlockInputInactiveH2 = <BlockInputInactiveH2> {}
        BlockInputInactiveH3 = <BlockInputInactiveH3> {}
        BlockInputList = <BlockInputList> {}
        BlockInputInactiveList = <BlockInputInactiveList> {}
        BlockInputOrderedList = <BlockInputOrderedList> {}
        BlockInputInactiveOrderedList = <BlockInputInactiveOrderedList> {}
        
        // Rich text editor for active blocks
        RichBlockEditor = <RichTextEditorBase> {}
    }
}

#[derive(Live, Widget)]
pub struct BlockEditor {
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[rust] area: Area,
    #[rust] pub editor_state: EditorState,
    #[rust] renderer: BlockRenderer,
    #[rust] event_handler: EventHandler,
    #[rust] storage: LocalStorage,
}

impl LiveHook for BlockEditor {
    fn before_apply(&mut self, _cx: &mut Cx, apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        if let ApplyFrom::UpdateFromDoc {..} = apply.from {
            self.renderer.clear_templates();
        }
    }
    
    fn apply_value_instance(&mut self, cx: &mut Cx, apply: &mut Apply, index: usize, nodes: &[LiveNode]) -> usize {
        if nodes[index].is_instance_prop() {
            if let Some(live_ptr) = apply.from.to_live_ptr(cx, index){
                let id = nodes[index].id;
                self.renderer.register_template(id, live_ptr);
            }
        }
        nodes.skip_node(index)
    }
}

impl Widget for BlockEditor {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.event_handler = EventHandler::new();
        
        // Handle events for all items
        for (block_id, item) in self.renderer.items_iter_mut() {
            let block_id = *block_id;
            
            for action in cx.capture_actions(|cx| item.handle_event(cx, event, scope)) {
                if let Some(action) = action.as_widget_action() {
                    match action.cast() {
                        RichTextInputAction::Changed(text) => {
                            self.event_handler.handle_text_changed(block_id, text, &mut self.editor_state);
                        }
                        RichTextInputAction::KeyFocus => {
                            self.event_handler.handle_key_focus(block_id, &mut self.editor_state, item, cx);
                        }
                        RichTextInputAction::KeyDownUnhandled(ke) => {
                            self.event_handler.handle_navigation(block_id, ke.key_code, &self.editor_state);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Handle global key events before RichTextInput processes them
        if let Event::KeyDown(ke) = event {
            match ke.key_code {
                KeyCode::ReturnKey if !ke.modifiers.shift => {
                    let active_index = self.editor_state.active_block_index();
                    self.event_handler.handle_enter_key(active_index, &self.editor_state);
                }
                KeyCode::Tab => {
                    let active_index = self.editor_state.active_block_index();
                    if self.event_handler.handle_tab_key(active_index, ke.modifiers.shift, &mut self.editor_state) {
                        // Keep focus on current block after tab
                        if let Some(item) = self.renderer.items_iter_mut().find(|(idx, _)| **idx == active_index) {
                            item.1.set_key_focus(cx);
                        }
                        cx.redraw_all();
                    }
                }
                KeyCode::Backspace => {
                    let active_index = self.editor_state.active_block_index();
                    if let Some(block) = self.editor_state.blocks().get(active_index) {
                        if block.content.trim().is_empty() && active_index > 0 && self.editor_state.blocks().len() > 1 {
                            self.editor_state.delete_block(active_index);
                            self.editor_state.set_active_block(active_index - 1);
                            self.renderer.clear_items();
                            cx.redraw_all();
                            return;
                        }
                    }
                }
                KeyCode::KeyS if ke.modifiers.control => {
                    self.save_file();
                }
                KeyCode::KeyO if ke.modifiers.control => {
                    // For now, load the untitled.md file as a test
                    let path = PathBuf::from("untitled.md");
                    self.load_file(path);
                    cx.redraw_all();
                }
                KeyCode::KeyB if ke.modifiers.control => {
                    self.toggle_bold(cx);
                }
                KeyCode::KeyI if ke.modifiers.control => {
                    self.toggle_italic(cx);
                }
                _ => {}
            }
        }
        
        // Apply changes
        for block_id in &self.event_handler.blocks_to_recreate {
            self.renderer.remove_item(block_id);
        }
        
        if self.event_handler.should_delete_block {
            let active_index = self.editor_state.active_block_index();
            self.editor_state.delete_block(active_index);
            self.renderer.clear_items();
            cx.redraw_all();
            return;
        }
        
        // Handle new block creation
        if let Some((insert_index, content)) = &self.event_handler.should_create_new_block {
            let block_type = if crate::markdown::parser::is_list_item(content) {
                crate::block::BlockType::List
            } else {
                crate::block::BlockType::Text
            };
            
            let new_block = crate::block::Block::new(block_type, content.clone());
            self.editor_state.create_block(*insert_index, new_block);
            self.editor_state.set_active_block(*insert_index);
            
            self.renderer.clear_items();
            cx.redraw_all();
            return;
        }
        
        // Handle list renumbering
        if let Some(start_index) = self.event_handler.should_renumber_lists {
            // Find the start of the list sequence
            let mut list_start = start_index;
            while list_start > 0 {
                if let Some(prev_block) = self.editor_state.blocks().get(list_start - 1) {
                    if let Some(list_info) = crate::markdown::parser::detect_list_item(&prev_block.content) {
                        if list_info.list_type == crate::markdown::parser::ListType::Ordered {
                            list_start -= 1;
                            continue;
                        }
                    }
                }
                break;
            }
            
            // Renumber from the start of the sequence
            let blocks = self.editor_state.blocks_mut();
            self.event_handler.ordered_list_manager.renumber_list_sequence(blocks, list_start);
            
            self.renderer.clear_items();
            cx.redraw_all();
            return;
        }
        
        if let Some(new_active) = self.event_handler.navigation_target {
            let old_active = self.editor_state.active_block_index();
            self.editor_state.set_active_block(new_active);
            self.renderer.remove_item(&old_active);
            self.renderer.remove_item(&new_active);
            cx.redraw_all();
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        let active_index = self.editor_state.active_block_index();
        
        for (index, block) in self.editor_state.blocks().iter().enumerate() {
            let is_active = index == active_index;
            let _ = self.renderer.render_block(cx, scope, index, block, is_active);
        }
        
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl BlockEditor {
    fn save_file(&mut self) {
        let path = if let Some(current_file) = self.editor_state.current_file() {
            current_file.clone()
        } else {
            // Default file name for new documents
            PathBuf::from("untitled.md")
        };
        
        match self.storage.save_blocks(&path, self.editor_state.blocks()) {
            Ok(()) => {
                self.editor_state.mark_saved();
                self.editor_state.set_current_file(path.clone());
                ::log::info!("File saved: {:?}", path);
            }
            Err(e) => {
                ::log::error!("Failed to save file: {}", e);
            }
        }
    }
    
    pub fn load_file(&mut self, path: PathBuf) {
        match self.storage.load_blocks(&path) {
            Ok(blocks) => {
                *self.editor_state.blocks_mut() = blocks;
                self.editor_state.set_current_file(path.clone());
                self.editor_state.mark_saved();
                self.editor_state.set_active_block(0);
                self.renderer.clear_items();
                ::log::info!("File loaded: {:?}", path);
            }
            Err(e) => {
                ::log::error!("Failed to load file: {}", e);
            }
        }
    }
    
    fn toggle_bold(&mut self, cx: &mut Cx) {
        let active_index = self.editor_state.active_block_index();
        if let Some(item) = self.renderer.items_iter_mut().find(|(idx, _)| **idx == active_index) {
            let text_input = item.1.as_text_input();
            let current_text = text_input.text();
            let cursor = text_input.cursor().index;
            
            let (new_text, new_cursor) = FormattingManager::toggle_bold(&current_text, cursor);
            text_input.set_text(cx, &new_text);
            text_input.set_cursor(cx, makepad_draw::text::selection::Cursor {
                index: new_cursor,
                prefer_next_row: false,
            }, false);
            
            // Update the block content
            if let Some(block) = self.editor_state.blocks_mut().get_mut(active_index) {
                block.content = new_text.to_string();
                self.editor_state.mark_modified();
            }
            
            item.1.redraw(cx);
        }
    }
    
    fn toggle_italic(&mut self, cx: &mut Cx) {
        let active_index = self.editor_state.active_block_index();
        if let Some(item) = self.renderer.items_iter_mut().find(|(idx, _)| **idx == active_index) {
            let text_input = item.1.as_text_input();
            let current_text = text_input.text();
            let cursor = text_input.cursor().index;
            
            let (new_text, new_cursor) = FormattingManager::toggle_italic(&current_text, cursor);
            text_input.set_text(cx, &new_text);
            text_input.set_cursor(cx, makepad_draw::text::selection::Cursor {
                index: new_cursor,
                prefer_next_row: false,
            }, false);
            
            // Update the block content
            if let Some(block) = self.editor_state.blocks_mut().get_mut(active_index) {
                block.content = new_text.to_string();
                self.editor_state.mark_modified();
            }
            
            item.1.redraw(cx);
        }
    }
}
