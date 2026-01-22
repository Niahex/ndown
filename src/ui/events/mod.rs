use makepad_widgets::*;
use makepad_draw::text::selection::Cursor;
use crate::editor_state::EditorState;
use crate::markdown::parser::{detect_heading_level, detect_list_item, is_list_item, ListType};
use crate::ui::components::indentation::IndentationManager;
use crate::ui::components::ordered_list::OrderedListManager;
use crate::block::BlockType;

#[derive(Default)]
pub struct EventHandler {
    pub navigation_target: Option<usize>,
    pub should_delete_block: bool,
    pub blocks_to_recreate: Vec<usize>,
    pub should_create_new_block: Option<(usize, String)>,
    pub should_renumber_lists: Option<usize>,
    indent_manager: IndentationManager,
    pub ordered_list_manager: OrderedListManager,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            indent_manager: IndentationManager::new(),
            ordered_list_manager: OrderedListManager::new(),
            ..Default::default()
        }
    }

    pub fn handle_text_changed(&mut self, block_id: usize, text: String, editor_state: &mut EditorState) {
        if let Some(block) = editor_state.blocks_mut().get_mut(block_id) {
            let old_content = block.content.clone();
            block.content = text.clone();
            
            // Check if heading level changed
            let old_level = detect_heading_level(&old_content);
            let new_level = detect_heading_level(&text);
            
            // Check if list status changed
            let was_list = is_list_item(&old_content);
            let is_list = is_list_item(&text);
            
            if old_level != new_level || was_list != is_list {
                // Update block type
                if detect_list_item(&text).is_some() {
                    block.block_type = BlockType::List;
                } else {
                    block.block_type = BlockType::Text;
                }
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

    pub fn handle_enter_key(&mut self, block_id: usize, editor_state: &EditorState) {
        if let Some(block) = editor_state.blocks().get(block_id) {
            if let Some(list_info) = detect_list_item(&block.content) {
                // If list item is empty, convert to normal text
                if list_info.content.trim().is_empty() {
                    self.should_create_new_block = Some((block_id + 1, String::new()));
                    return;
                }
                
                // Create new list item
                let new_content = self.indent_manager.continue_list_item(&list_info);
                self.should_create_new_block = Some((block_id + 1, new_content));
            } else {
                // Normal text block
                self.should_create_new_block = Some((block_id + 1, String::new()));
            }
        }
    }

    pub fn handle_tab_key(&mut self, block_id: usize, shift_pressed: bool, editor_state: &mut EditorState) -> bool {
        // First, get the list info and previous main number before borrowing mutably
        let (list_info, prev_main_num) = if let Some(block) = editor_state.blocks().get(block_id) {
            if let Some(list_info) = detect_list_item(&block.content) {
                let prev_main_num = if list_info.list_type == ListType::Ordered && !shift_pressed {
                    self.find_previous_main_number(block_id, editor_state)
                } else {
                    None
                };
                (list_info, prev_main_num)
            } else {
                return false;
            }
        } else {
            return false;
        };
        
        // Now handle the mutation
        if let Some(block) = editor_state.blocks_mut().get_mut(block_id) {
            match list_info.list_type {
                ListType::Unordered => {
                    if shift_pressed {
                        block.content = self.indent_manager.decrease_indent(&block.content);
                    } else {
                        block.content = self.indent_manager.increase_indent(&block.content);
                    }
                    self.blocks_to_recreate.push(block_id);
                    return true;
                }
                ListType::Ordered => {
                    // Tab/Shift+Tab disabled for ordered lists due to complexity
                    // Will be implemented later
                    return false;
                }
            }
        }
        false
    }
    
    fn find_previous_main_number(&self, block_id: usize, editor_state: &EditorState) -> Option<u32> {
        // Look backwards for the previous main-level ordered list item
        for i in (0..block_id).rev() {
            if let Some(block) = editor_state.blocks().get(i) {
                if let Some(list_info) = detect_list_item(&block.content) {
                    if list_info.list_type == ListType::Ordered && list_info.indent_level == 0 {
                        // Found previous main item, extract its number
                        let trimmed = block.content.trim_start();
                        if let Some(dot_pos) = trimmed.find('.') {
                            if let Ok(num) = trimmed[..dot_pos].parse::<u32>() {
                                return Some(num);
                            }
                        }
                    }
                } else {
                    break; // Stop at non-list item
                }
            }
        }
        None
    }
}
