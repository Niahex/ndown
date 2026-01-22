use makepad_widgets::*;
use makepad_widgets::text_input::TextInputAction;
use makepad_draw::text::selection::Cursor;
use crate::editor_state::EditorState;
use crate::block::BlockType;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    BlockLabelBase = <Label> {
        width: Fill
        padding: 10
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    BlockInputBase = <TextInput> {
        width: Fill
        height: Fit
        padding: 10
        draw_bg: {
            color: #2a2a2a
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #ffffff
        }
    }
    
    BlockInputInactive = <TextInput> {
        width: Fill
        height: Fit
        padding: 10
        is_read_only: true
        draw_bg: {
            color: #1a1a1a
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #cccccc
        }
    }
    
    pub BlockEditor = {{BlockEditor}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 5
        
        BlockLabel = <BlockLabelBase> {}
        BlockInput = <BlockInputBase> {}
        BlockInputInactive = <BlockInputInactive> {}
    }
}

#[derive(Live, Widget)]
pub struct BlockEditor {
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[rust] area: Area,
    #[rust] pub editor_state: EditorState,
    #[rust] templates: ComponentMap<LiveId, LivePtr>,
    #[rust] items: ComponentMap<usize, WidgetRef>,
}

impl LiveHook for BlockEditor {
    fn before_apply(&mut self, _cx: &mut Cx, apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        if let ApplyFrom::UpdateFromDoc {..} = apply.from {
            self.templates.clear();
        }
        ::log::info!("BlockEditor before_apply called");
    }
    
    fn apply_value_instance(&mut self, cx: &mut Cx, apply: &mut Apply, index: usize, nodes: &[LiveNode]) -> usize {
        ::log::info!("BlockEditor apply_value_instance called for node: {:?}", nodes[index].id);
        
        if nodes[index].is_instance_prop() {
            if let Some(live_ptr) = apply.from.to_live_ptr(cx, index){
                let id = nodes[index].id;
                ::log::info!("Registering template: {:?} -> {:?}", id, live_ptr);
                self.templates.insert(id, live_ptr);
            }
        }
        nodes.skip_node(index)
    }
}

impl Widget for BlockEditor {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let mut navigation_target: Option<usize> = None;
        let mut should_delete_block = false;
        
        // Handle events for all items normally - let TextInputs handle their own clicks
        for (block_id, item) in self.items.iter_mut() {
            let block_id = *block_id;
            
            for action in cx.capture_actions(|cx| item.handle_event(cx, event, scope)) {
                if let Some(action) = action.as_widget_action() {
                    match action.cast() {
                        TextInputAction::Changed(text) => {
                            if let Some(block) = self.editor_state.blocks_mut().get_mut(block_id) {
                                block.content = text;
                            }
                        }
                        TextInputAction::KeyFocus => {
                            // When a TextInput gets focus, make it the active block
                            let old_active = self.editor_state.active_block_index();
                            if block_id != old_active {
                                self.editor_state.set_active_block(block_id);
                                self.items.remove(&old_active);
                                self.items.remove(&block_id);
                                cx.redraw_all();
                                return;
                            } else {
                                // Same block getting focus - position cursor at end
                                let text_input = item.as_text_input();
                                let text_len = text_input.text().len();
                                text_input.set_cursor(cx, Cursor {
                                    index: text_len,
                                    prefer_next_row: false,
                                }, false);
                                item.redraw(cx);
                            }
                        }
                        TextInputAction::KeyDownUnhandled(ke) => {
                            match ke.key_code {
                                KeyCode::ArrowUp if block_id > 0 => {
                                    navigation_target = Some(block_id - 1);
                                }
                                KeyCode::ArrowDown if block_id < self.editor_state.blocks().len() - 1 => {
                                    navigation_target = Some(block_id + 1);
                                }
                                KeyCode::Backspace if block_id > 0 && self.editor_state.blocks().len() > 1 => {
                                    if let Some(block) = self.editor_state.blocks().get(block_id) {
                                        if block.content.is_empty() {
                                            should_delete_block = true;
                                            navigation_target = Some(block_id - 1);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        if should_delete_block {
            let active_index = self.editor_state.active_block_index();
            self.editor_state.delete_block(active_index);
            self.items.clear();
            cx.redraw_all();
            return;
        }
        
        if let Some(new_active) = navigation_target {
            let old_active = self.editor_state.active_block_index();
            self.editor_state.set_active_block(new_active);
            self.items.remove(&old_active);
            self.items.remove(&new_active);
            cx.redraw_all();
        }
        
        // Handle keyboard shortcuts
        if let Event::KeyDown(ke) = event {
            match ke.key_code {
                KeyCode::ReturnKey if !ke.modifiers.shift => {
                    let old_active = self.editor_state.active_block_index();
                    let new_index = old_active + 1;
                    
                    self.editor_state.create_block(new_index, crate::block::Block::text(String::new()));
                    self.editor_state.set_active_block(new_index);
                    
                    // Remove old active and new active so they get recreated with correct templates
                    self.items.remove(&old_active);
                    self.items.remove(&new_index);
                    
                    cx.redraw_all();
                }
                
                // Ctrl+Home: go to first block
                KeyCode::Home if ke.modifiers.control => {
                    let old_active = self.editor_state.active_block_index();
                    self.editor_state.set_active_block(0);
                    self.items.remove(&old_active);
                    self.items.remove(&0);
                    cx.redraw_all();
                }
                
                // Ctrl+End: go to last block
                KeyCode::End if ke.modifiers.control => {
                    let old_active = self.editor_state.active_block_index();
                    let last_index = self.editor_state.blocks().len().saturating_sub(1);
                    self.editor_state.set_active_block(last_index);
                    self.items.remove(&old_active);
                    self.items.remove(&last_index);
                    cx.redraw_all();
                }
                
                // Page Up: go to previous block
                KeyCode::PageUp => {
                    let active_index = self.editor_state.active_block_index();
                    if active_index > 0 {
                        self.editor_state.set_active_block(active_index - 1);
                        self.items.remove(&active_index);
                        self.items.remove(&(active_index - 1));
                        cx.redraw_all();
                    }
                }
                
                // Page Down: go to next block
                KeyCode::PageDown => {
                    let active_index = self.editor_state.active_block_index();
                    if active_index < self.editor_state.blocks().len() - 1 {
                        self.editor_state.set_active_block(active_index + 1);
                        self.items.remove(&active_index);
                        self.items.remove(&(active_index + 1));
                        cx.redraw_all();
                    }
                }
                
                _ => {}
            }
        }
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        
        ::log::debug!("BlockEditor draw_walk: {} blocks, {} templates, {} items", 
            self.editor_state.blocks().len(),
            self.templates.len(),
            self.items.len()
        );
        
        let active_index = self.editor_state.active_block_index();
        
        // First pass: create/update all widgets
        for (index, block) in self.editor_state.blocks().iter().enumerate() {
            let is_active = index == active_index;
            
            let template_id = if is_active {
                live_id!(BlockInput)
            } else {
                live_id!(BlockInputInactive)
            };
            
            // Get or create widget for this block
            if !self.items.contains_key(&index) {
                if let Some(ptr) = self.templates.get(&template_id) {
                    let widget = WidgetRef::new_from_ptr(cx, Some(*ptr));
                    self.items.insert(index, widget);
                } else {
                    ::log::warn!("Template {:?} not found!", template_id);
                }
            }
            
            // Set text content
            if let Some(item) = self.items.get(&index) {
                let content = match &block.block_type {
                    BlockType::Text => block.content.clone(),
                    BlockType::Heading(level) => {
                        format!("{} {}", "#".repeat(*level as usize), block.content)
                    }
                };
                
                item.set_text(cx, &content);
            }
        }
        
        // Second pass: draw all widgets
        for (index, _block) in self.editor_state.blocks().iter().enumerate() {
            if let Some(item) = self.items.get_mut(&index) {
                item.draw_walk(cx, scope, Walk::fill_fit())?;
                
                // Set focus on active block after drawing
                if index == active_index {
                    item.set_key_focus(cx);
                }
            }
        }
        
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}
