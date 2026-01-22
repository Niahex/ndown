use crate::block::Block;

#[derive(Debug)]
pub struct EditorState {
    blocks: Vec<Block>,
    active_block_index: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            blocks: vec![Block::text(String::new())],
            active_block_index: 0,
        }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
    
    pub fn blocks_mut(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }

    pub fn active_block_index(&self) -> usize {
        self.active_block_index
    }

    pub fn active_block(&self) -> Option<&Block> {
        self.blocks.get(self.active_block_index)
    }

    pub fn active_block_mut(&mut self) -> Option<&mut Block> {
        self.blocks.get_mut(self.active_block_index)
    }

    pub fn create_block(&mut self, index: usize, block: Block) {
        if index <= self.blocks.len() {
            self.blocks.insert(index, block);
            ::log::debug!("Created block at index {}", index);
        }
    }

    pub fn delete_block(&mut self, index: usize) -> Option<Block> {
        if index < self.blocks.len() && self.blocks.len() > 1 {
            let block = self.blocks.remove(index);
            if self.active_block_index >= self.blocks.len() {
                self.active_block_index = self.blocks.len().saturating_sub(1);
            }
            ::log::debug!("Deleted block at index {}", index);
            Some(block)
        } else {
            None
        }
    }

    pub fn set_active_block(&mut self, index: usize) {
        if index < self.blocks.len() {
            self.active_block_index = index;
            ::log::debug!("Active block changed to index {}", index);
        }
    }

    pub fn next_block(&mut self) {
        if self.active_block_index + 1 < self.blocks.len() {
            self.active_block_index += 1;
            ::log::debug!("Moved to next block: {}", self.active_block_index);
        }
    }

    pub fn prev_block(&mut self) {
        if self.active_block_index > 0 {
            self.active_block_index -= 1;
            ::log::debug!("Moved to previous block: {}", self.active_block_index);
        }
    }
}
