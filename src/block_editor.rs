use makepad_widgets::*;
use makepad_widgets::text_input::TextInputAction;
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
        draw_bg: {
            color: #2a2a2a
        }
        draw_text: {
            text_style: <THEME_FONT_REGULAR> {font_size: 14}
            color: #ffffff
        }
    }
    
    pub BlockEditor = {{BlockEditor}} {
        width: Fill, height: Fill
        flow: Down
        spacing: 5
        
        BlockLabel = <BlockLabelBase> {}
        BlockInput = <BlockInputBase> {}
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
        // Handle events for each item and capture their actions
        for (block_id, item) in self.items.iter_mut() {
            let block_id = *block_id;
            for action in cx.capture_actions(|cx| item.handle_event(cx, event, scope)) {
                if let Some(action) = action.as_widget_action() {
                    if let TextInputAction::Changed(text) = action.cast() {
                        if let Some(block) = self.editor_state.blocks_mut().get_mut(block_id) {
                            block.content = text;
                        }
                    }
                }
            }
        }
        
        // Check for clicks on items to change active block
        let mut clicked_index = None;
        for (index, item) in self.items.iter() {
            if let Hit::FingerDown(_) = event.hits_with_options(cx, item.area(), HitOptions::default()) {
                if *index != self.editor_state.active_block_index() {
                    clicked_index = Some(*index);
                    break;
                }
            }
        }
        
        // Handle click to change active block
        if let Some(new_active) = clicked_index {
            let old_active = self.editor_state.active_block_index();
            self.editor_state.set_active_block(new_active);
            
            // Remove old and new active to recreate with correct templates
            self.items.remove(&old_active);
            self.items.remove(&new_active);
            
            cx.redraw_all();
            return;
        }
        
        // Handle Enter key for creating blocks
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::ReturnKey && !ke.modifiers.shift {
                let old_active = self.editor_state.active_block_index();
                let new_index = old_active + 1;
                
                self.editor_state.create_block(new_index, crate::block::Block::text(String::new()));
                self.editor_state.set_active_block(new_index);
                
                // Remove old active and new active so they get recreated with correct templates
                self.items.remove(&old_active);
                self.items.remove(&new_index);
                
                cx.redraw_all();
            }
            
            // Handle Backspace to delete empty blocks
            if ke.key_code == KeyCode::Backspace {
                let active_index = self.editor_state.active_block_index();
                
                // Only delete if block is empty, not the first block, and more than 1 block exists
                if active_index > 0 && self.editor_state.blocks().len() > 1 {
                    if let Some(block) = self.editor_state.blocks().get(active_index) {
                        if block.content.is_empty() {
                            self.editor_state.delete_block(active_index);
                            self.editor_state.set_active_block(active_index - 1);
                            
                            // Clear items to force recreation
                            self.items.clear();
                            cx.redraw_all();
                        }
                    }
                }
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
                live_id!(BlockLabel)
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
