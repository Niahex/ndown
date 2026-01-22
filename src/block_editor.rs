use makepad_widgets::*;
use makepad_widgets::text_input::TextInputAction;
use crate::editor_state::EditorState;
use crate::ui::{EventHandler, BlockRenderer};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    BlockLabelBase = <Label> {
        width: Fill, padding: 10
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    BlockInputBase = <TextInput> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #ffffff
        }
    }
    
    BlockInputH1 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 25}
            color: #ffffff
        }
    }
    
    BlockInputH2 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 21}
            color: #ffffff
        }
    }
    
    BlockInputH3 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        draw_bg: { color: #2a2a2a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 18}
            color: #ffffff
        }
    }
    
    BlockInputInactive = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    BlockInputInactiveH1 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 25}
            color: #cccccc
        }
    }
    
    BlockInputInactiveH2 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 21}
            color: #cccccc
        }
    }
    
    BlockInputInactiveH3 = <TextInput> {
        width: Fill, height: Fit, padding: 10
        is_read_only: true
        draw_bg: { color: #1a1a1a }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 18}
            color: #cccccc
        }
    }
    
    // List templates
    BlockInputList = <TextInput> {
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
                        TextInputAction::Changed(text) => {
                            self.event_handler.handle_text_changed(block_id, text, &mut self.editor_state);
                        }
                        TextInputAction::KeyFocus => {
                            self.event_handler.handle_key_focus(block_id, &mut self.editor_state, item, cx);
                        }
                        TextInputAction::KeyDownUnhandled(ke) => {
                            self.event_handler.handle_navigation(block_id, ke.key_code, &self.editor_state);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Handle global key events before TextInput processes them
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
