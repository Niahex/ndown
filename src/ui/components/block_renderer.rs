use makepad_widgets::*;
use crate::block::BlockType;
use crate::ui::get_template_for_block;

#[derive(Default)]
pub struct BlockRenderer {
    templates: ComponentMap<LiveId, LivePtr>,
    items: ComponentMap<usize, WidgetRef>,
}

impl BlockRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_template(&mut self, id: LiveId, ptr: LivePtr) {
        self.templates.insert(id, ptr);
    }

    pub fn clear_templates(&mut self) {
        self.templates.clear();
    }

    pub fn remove_item(&mut self, index: &usize) {
        self.items.remove(index);
    }

    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    pub fn get_item(&self, index: &usize) -> Option<&WidgetRef> {
        self.items.get(index)
    }

    pub fn get_item_mut(&mut self, index: &usize) -> Option<&mut WidgetRef> {
        self.items.get_mut(index)
    }

    pub fn items_iter_mut(&mut self) -> impl Iterator<Item = (&usize, &mut WidgetRef)> {
        self.items.iter_mut()
    }

    pub fn render_block(&mut self, cx: &mut Cx2d, scope: &mut Scope, index: usize, block: &crate::block::Block, is_active: bool) -> Result<(), ()> {
        let content = match &block.block_type {
            BlockType::Text => block.content.clone(),
            BlockType::Heading(level) => {
                format!("{} {}", "#".repeat(*level as usize), block.content)
            }
            BlockType::List => block.content.clone(),
        };
        
        let template_id = get_template_for_block(&content, is_active);
        
        // Get or create widget
        if !self.items.contains_key(&index) {
            if let Some(ptr) = self.templates.get(&template_id) {
                let widget = WidgetRef::new_from_ptr(cx, Some(*ptr));
                self.items.insert(index, widget);
            } else {
                return Err(());
            }
        }
        
        // Set content and draw
        if let Some(item) = self.items.get(&index) {
            item.set_text(cx, &content);
            let _ = item.draw_walk(cx, scope, Walk::fill_fit());
            
            // Set focus if active
            if is_active {
                item.set_key_focus(cx);
            }
        }
        
        Ok(())
    }
}
