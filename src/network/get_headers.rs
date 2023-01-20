use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GetHeader {
    pub hash_count: u32,
    pub block_locator_hashes: Vec<String>,
    pub hash_stop: Vec<String>,
}
