use std::{fs, io, path::Path};
use crate::{block::{Block, BlockType}, storage::Storage};

pub struct LocalStorage;

impl LocalStorage {
    pub fn new() -> Self {
        Self
    }
    
    fn blocks_to_markdown(blocks: &[Block]) -> String {
        blocks.iter()
            .map(|block| match &block.block_type {
                BlockType::Text => block.content.clone(),
                BlockType::Heading(level) => {
                    if block.content.starts_with(&"#".repeat(*level as usize)) {
                        block.content.clone()
                    } else {
                        format!("{} {}", "#".repeat(*level as usize), block.content)
                    }
                }
                BlockType::List => block.content.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    
    fn markdown_to_blocks(content: &str) -> Vec<Block> {
        if content.trim().is_empty() {
            return vec![Block::text(String::new())];
        }
        
        content.split("\n\n")
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let trimmed = line.trim();
                
                // Detect block type
                if let Some(level) = crate::markdown::parser::detect_heading_level(trimmed) {
                    Block::heading(level, trimmed.to_string())
                } else if crate::markdown::parser::is_list_item(trimmed) {
                    Block::list(trimmed.to_string())
                } else {
                    Block::text(trimmed.to_string())
                }
            })
            .collect()
    }
}

impl Storage for LocalStorage {
    type Error = io::Error;
    
    fn save_blocks(&self, path: &Path, blocks: &[Block]) -> Result<(), Self::Error> {
        let markdown_content = Self::blocks_to_markdown(blocks);
        fs::write(path, markdown_content)
    }
    
    fn load_blocks(&self, path: &Path) -> Result<Vec<Block>, Self::Error> {
        let content = fs::read_to_string(path)?;
        Ok(Self::markdown_to_blocks(&content))
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}
