use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_BLOCK_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq)]
pub struct BlockId(usize);

impl BlockId {
    pub fn new() -> Self {
        Self(NEXT_BLOCK_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Text,
    Heading(u8), // 1-5 for h1-h5
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub block_type: BlockType,
    pub content: String,
}

impl Block {
    pub fn new(block_type: BlockType, content: String) -> Self {
        Self {
            id: BlockId::new(),
            block_type,
            content,
        }
    }

    pub fn text(content: String) -> Self {
        Self::new(BlockType::Text, content)
    }

    pub fn heading(level: u8, content: String) -> Self {
        Self::new(BlockType::Heading(level), content)
    }
}
