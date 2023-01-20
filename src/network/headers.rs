use crate::components::block::BlockHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Header {
    pub hash_count: u32,
    pub headers: BlockHeader,
}
