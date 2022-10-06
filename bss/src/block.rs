use crate::merkle::Merkle;
use serde::{Serialize};
#[derive(Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Merkle,
}

#[derive(Serialize)]
pub struct BlockHeader {
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u128,
}