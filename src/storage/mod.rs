use std::path::Path;
use crate::block::Block;

pub trait Storage {
    type Error;
    
    fn save_blocks(&self, path: &Path, blocks: &[Block]) -> Result<(), Self::Error>;
    fn load_blocks(&self, path: &Path) -> Result<Vec<Block>, Self::Error>;
}

pub mod local_storage;
pub use local_storage::LocalStorage;
