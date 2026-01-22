use makepad_widgets::*;
use makepad_draw::text::selection::Cursor;
use crate::editor_state::EditorState;
use crate::markdown::detect_heading_level;

#[derive(Default)]
pub struct EventHandler {
    pub navigation_target: Option<usize>,
    pub should_delete_block: bool,
    pub blocks_to_recreate: Vec<usize>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_text_changed(&mut self, block_id: usize, text: String, editor_state: &mut EditorState) {
        if let Some(block) = editor_state.blocks_mut().get_mut(block_id) {
            let old_content = block.content.clone();
            block.content = text.clone();
            
            // Check if heading level changed
            let old_level = detect_heading_level(&old_content);
            let new_level = detect_heading_level(&text);
            
            if old_level != new_level {
                self.blocks_to_recreate.push(block_id);
            }
        }
    }

    pub fn handle_key_focus(&mut self, block_id: usize, editor_state: &mut EditorState, item: &WidgetRef, cx: &mut Cx) {
        let old_active = editor_state.active_block_index();
        if block_id != old_active {
            editor_state.set_active_block(block_id);
            self.blocks_to_recreate.push(old_active);
            self.blocks_to_recreate.push(block_id);
        } else {
            // Position cursor at end
            let text_input = item.as_text_input();
            let text_len = text_input.text().len();
            text_input.set_cursor(cx, Cursor {
                index: text_len,
                prefer_next_row: false,
            }, false);
            item.redraw(cx);
        }
    }

    pub fn handle_navigation(&mut self, block_id: usize, key_code: KeyCode, editor_state: &EditorState) {
        match key_code {
            KeyCode::ArrowUp if block_id > 0 => {
                self.navigation_target = Some(block_id - 1);
            }
            KeyCode::ArrowDown if block_id < editor_state.blocks().len() - 1 => {
                self.navigation_target = Some(block_id + 1);
            }
            KeyCode::Backspace if block_id > 0 && editor_state.blocks().len() > 1 => {
                if let Some(block) = editor_state.blocks().get(block_id) {
                    if block.content.is_empty() {
                        self.should_delete_block = true;
                        self.navigation_target = Some(block_id - 1);
                    }
                }
            }
            _ => {}
        }
    }
}
